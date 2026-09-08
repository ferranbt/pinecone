// Re-export all public types from sub-crates
pub use pine_ast as ast;
pub use pine_broker as broker;
pub use pine_builtins as builtins;
use pine_builtins::DefaultPineOutput;
pub use pine_core as core;
pub use pine_data as data;
pub use pine_diagnostics as diagnostics;
pub use pine_format as format;
pub use pine_interpreter as interpreter;
pub use pine_lexer as lexer;
pub use pine_lint as lint;
pub use pine_parser as parser;
pub use pine_sema as sema;

mod run;

pub use pine_broker::{Backtest, Metrics};
pub use pine_core::{DataProvider, DirLoader, FileResolver, LibraryLoader};
pub use run::{Run, RunResult};

use pine_ast::Program;
use pine_core::{Bar, Data, PineVersion, Timeframe, VersionError};
use pine_core::{FullPineOutput, PineOutput};
use pine_diagnostics::Diagnostic;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use pine_lexer::{Lexer, LexerError};
use pine_parser::{Parser, ParserError};
use std::collections::HashMap;
use std::rc::Rc;

/// Error type for Pine operations
#[derive(Debug)]
pub enum Error {
    Lexer(LexerError),
    Parser(ParserError),
    Runtime(RuntimeError),
    /// Semantic analysis failed; the program is invalid. Carries every
    /// diagnostic found.
    Sema(Vec<Diagnostic>),
    /// The script's `//@version=N` annotation names a version this toolchain
    /// cannot compile.
    Version(VersionError),
    /// No bars to run over: neither data nor a provider was given, or the
    /// provider could not produce the requested feed.
    Data(pine_core::ProviderError),
}

impl Error {
    /// The 1-based `(line, column)` an editor should point at, when the error
    /// carries a position. Version errors have none.
    pub fn location(&self) -> Option<(u32, u32)> {
        match self {
            Error::Lexer(e) => Some(e.location()),
            Error::Parser(e) => Some(e.location()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(e) => write!(f, "Lexer error: {}", e),
            Error::Parser(e) => write!(f, "Parser error: {}", e),
            Error::Runtime(e) => write!(f, "Runtime error: {}", e),
            Error::Version(e) => write!(f, "Version error: {}", e),
            Error::Data(e) => write!(f, "Data error: {}", e),
            // One diagnostic per line, so multiple errors are simply appended.
            Error::Sema(diags) => {
                for (i, d) in diags.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{}", d)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<RuntimeError> for Error {
    fn from(e: RuntimeError) -> Self {
        Error::Runtime(e)
    }
}

impl From<LexerError> for Error {
    fn from(e: LexerError) -> Self {
        Error::Lexer(e)
    }
}

impl From<ParserError> for Error {
    fn from(e: ParserError) -> Self {
        Error::Parser(e)
    }
}

impl From<VersionError> for Error {
    fn from(e: VersionError) -> Self {
        Error::Version(e)
    }
}

pub struct Analysis {
    pub diagnostics: Vec<Diagnostic>,
    pub symbols: sema::SymbolTable,
}

/// Parse, semantically analyze and lint `source`.
pub fn analyze(source: &str, loader: Option<&dyn LibraryLoader>) -> Result<Analysis, Error> {
    let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);
    let tokens = Lexer::with_version(source, version).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;

    let (mut env, _): (HashMap<String, Value<DefaultPineOutput>>, _) =
        pine_builtins::register_namespace_objects(version, None, None);
    for (name, value) in pine_builtins::per_bar_variables(&Bar::default(), None) {
        env.insert(name, value);
    }

    let (mut diagnostics, symbols) = pine_sema::analyze_with_symbols(&program, &env, loader);
    diagnostics.extend(pine_lint::lint(&program));
    diagnostics.sort_by_key(|d| d.pos.unwrap_or((u32::MAX, u32::MAX)));
    Ok(Analysis {
        diagnostics,
        symbols,
    })
}

/// The diagnostics from [`analyze`].
pub fn check(source: &str, loader: Option<&dyn LibraryLoader>) -> Result<Vec<Diagnostic>, Error> {
    Ok(analyze(source, loader)?.diagnostics)
}

/// Collect a program's declared metadata — `indicator`/`library`, `input.*`, and
/// any custom records the output type keeps — without market data.
pub fn decode_metadata<O>(
    source: &str,
    custom_variables: HashMap<String, Value<O>>,
) -> Result<O, Error>
where
    O: FullPineOutput,
{
    let (program, version) = parse_program(source)?;
    let (mut consts, _) = pine_builtins::register_namespace_objects(version, None, None);
    for (name, value) in custom_variables {
        consts.insert(name, value);
    }
    let mut interpreter = Interpreter::<O>::new();
    interpreter.set_const_variables(consts);

    let _ = interpreter.execute(&program);
    Ok(interpreter.output)
}

/// Parse and lint `source`, returning only the lint findings (no semantic
/// analysis). `// @skip(...)` directives are honored.
pub fn lint_source(source: &str) -> Result<Vec<Diagnostic>, Error> {
    let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);
    let tokens = Lexer::with_version(source, version).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    Ok(pine_lint::lint(&program))
}

/// Decode `input.*` overrides from a JSON object — `{"Length": 20, "Smooth":
/// true, "Source": "close"}` — keyed by input title, for
/// [`ScriptBuilder::with_inputs`].
pub fn inputs_from_json(
    json: &str,
) -> Result<HashMap<String, pine_core::InputValue>, serde_json::Error> {
    serde_json::from_str(json)
}

pub struct ScriptBuilder<O: PineOutput> {
    source: String,
    custom_variables: HashMap<String, Value<O>>,
    inputs: HashMap<String, pine_core::InputValue>,
    library_loader: Option<Box<dyn LibraryLoader>>,
    request_provider: Option<Box<dyn DataProvider>>,
    ticker: Option<String>,
    timeframe: Timeframe,
    data: Option<Data>,
    bar_count: Option<usize>,
    broker_factory: Option<Box<dyn pine_broker::BrokerFactory>>,
}

impl<O: PineOutput> ScriptBuilder<O> {
    pub fn with_code(source: &str) -> ScriptBuilder<O> {
        Self {
            source: source.to_string(),
            custom_variables: HashMap::new(),
            inputs: HashMap::new(),
            library_loader: None,
            request_provider: None,
            ticker: None,
            timeframe: Timeframe::default(),
            data: None,
            bar_count: None,
            broker_factory: None,
        }
    }

    /// Host overrides for the script's `input.*` calls, keyed by input title.
    /// Each `input.*` returns (and validates) the override for its title if one
    /// is present, else its declared default. See [`inputs_from_json`].
    pub fn with_inputs(mut self, inputs: HashMap<String, pine_core::InputValue>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Host-supplied variables the script can reference, registered as consts
    /// alongside the builtin namespaces.
    pub fn with_custom_variables(mut self, variables: HashMap<String, Value<O>>) -> Self {
        self.custom_variables = variables;
        self
    }

    /// Resolves `import` statements. Without one, importing a library fails.
    pub fn with_library_loader(mut self, loader: Box<dyn LibraryLoader>) -> Self {
        self.library_loader = Some(loader);
        self
    }

    /// Supplies bars for `request.security`. Without one, `request.security`
    /// returns na.
    pub fn with_request_provider(mut self, provider: Box<dyn DataProvider>) -> Self {
        self.request_provider = Some(provider);
        self
    }

    /// Swaps the broker a `strategy` trades against. Without one, the built-in
    /// [`DefaultBrokerFactory`](pine_broker::DefaultBrokerFactory) is used.
    pub fn with_broker(mut self, factory: Box<dyn pine_broker::BrokerFactory>) -> Self {
        self.broker_factory = Some(factory);
        self
    }

    pub fn with_ticker(mut self, ticker: String) -> Self {
        self.ticker = Some(ticker);
        self
    }

    /// The chart timeframe exposed to the script as `timeframe.*`. Without one,
    /// the namespace is populated with defaults.
    pub fn with_timeframe(mut self, timeframe: Timeframe) -> Self {
        self.timeframe = timeframe;
        self
    }

    /// Run over only the last `bar_count` bars of the feed. Without one, the
    /// whole feed is used.
    pub fn with_bar_count(mut self, bar_count: usize) -> Self {
        self.bar_count = Some(bar_count);
        self
    }

    /// The market to run over: the bars, and the symbol and timeframe they
    /// belong to.
    ///
    /// The data describes itself, so it fills in `syminfo.*` and `timeframe.*`
    /// too. An explicit [`ScriptBuilder::with_syminfo`] or
    /// [`ScriptBuilder::with_timeframe`] still wins, whichever order they are
    /// called in.
    pub fn with_data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    /// Compile PineScript source code into a Script with default output
    pub fn compile(self) -> Result<Script<O>, Error>
    where
        O: FullPineOutput,
    {
        let data = match self.data {
            Some(data) => data,
            None => {
                let provider = self
                    .request_provider
                    .as_ref()
                    .ok_or_else(|| Error::Data("no data or request provider set".into()))?;

                let ticker = self.ticker.clone().unwrap_or_default();
                provider
                    .request(&ticker, self.timeframe.clone())
                    .map_err(Error::Data)?
            }
        };

        let syminfo = data.syminfo;
        let timeframe = self.timeframe;

        // Keep only the last `bar_count` bars when the caller limited the run.
        let mut bars = data.bars;
        if let Some(n) = self.bar_count {
            let len = bars.len();
            bars = bars.split_off(len.saturating_sub(n.max(1)));
        }

        // The chart's bar spacing, so `request.security_lower_tf` can reject a
        // request that is not actually lower than the chart timeframe.
        let chart_period = bars
            .windows(2)
            .next()
            .map(|pair| pair[1].time - pair[0].time);

        let source = self.source.as_str();
        let (program, version) = parse_program(source)?;

        let (mut consts, advances) = pine_builtins::register_namespace_objects(
            version,
            Some(syminfo),
            Some(timeframe.clone()),
        );
        for (name, value) in self.custom_variables {
            consts.insert(name, value);
        }

        let mut builtins = consts.clone();
        for (name, value) in pine_builtins::per_bar_variables(&Bar::default(), None) {
            builtins.insert(name, value);
        }

        let errors: Vec<_> =
            pine_sema::analyze(&program, &builtins, self.library_loader.as_deref())
                .into_iter()
                .filter(|diagnostic| diagnostic.severity == pine_diagnostics::Severity::Error)
                .collect();
        if !errors.is_empty() {
            return Err(Error::Sema(errors));
        }

        // Create interpreter and load builtin namespace objects
        let mut interpreter = Interpreter::new();
        interpreter.library_loader = self.library_loader;
        interpreter.request_provider = self.request_provider.map(Rc::from);
        interpreter.chart_period = chart_period;
        if let Some(broker_factory) = self.broker_factory {
            interpreter.broker_factory = Some(broker_factory);
        }
        interpreter.set_const_variables(consts);
        interpreter.per_bar_advances = advances.pre;
        interpreter.per_bar_post_advances = advances.post;
        interpreter.inputs = self.inputs;

        Ok(Script {
            program,
            interpreter,
            bars,
        })
    }
}

/// A compiled PineScript program, and the bars it will run over.
///
/// State accumulates across bars — series history, `var` locals, and every
/// stateful builtin's window — exactly as it does in TradingView. That makes a
/// `Script` single-use: [`Script::run`] takes it by value so a second run
/// cannot inherit the first one's state.
pub struct Script<O: PineOutput> {
    program: Program,
    interpreter: Interpreter<O>,
    /// Bars from the builder's source; empty when none was given.
    bars: Vec<Bar>,
}

impl<O: PineOutput> Script<O> {
    /// Run one bar. Private: bars must be replayed in order from the first, so
    /// [`Script::run`] is the only way in.
    pub fn execute(&mut self, bar: &Bar, last_bar: Option<&Bar>) -> Result<O, Error> {
        use interpreter::Value;

        self.interpreter.current_time = Some(bar.time);
        self.interpreter.current_bar = Some(bar.clone());

        for (name, value) in pine_builtins::per_bar_variables(bar, last_bar) {
            if matches!(value, Value::Series(_)) {
                self.interpreter.advance_series(&name, value);
            } else {
                self.interpreter.set_variable(&name, value);
            }
        }

        self.interpreter.execute(&self.program).map_err(Error::from)
    }

    pub fn run_fn<F>(mut self, mut on_output: F) -> Result<(), Error>
    where
        F: FnMut(O),
    {
        let bars = std::mem::take(&mut self.bars);
        let last_bar = bars.last().cloned();
        for bar in &bars {
            let output = self.execute(bar, last_bar.as_ref())?;
            on_output(output);
        }
        Ok(())
    }

    /// Replay the script over every bar from its source, returning what each
    /// one produced.
    pub fn run(mut self) -> Result<Run<O>, Error> {
        let bars = std::mem::take(&mut self.bars);
        let last_bar = bars.last().cloned();
        let outputs = bars
            .iter()
            .map(|bar| self.execute(bar, last_bar.as_ref()))
            .collect::<Result<Vec<O>, Error>>()?;
        let broker = self.interpreter.broker.take();
        Ok(Run { outputs, broker })
    }
}

pub fn execute(source: &str, data: Data) -> Result<(), Error> {
    ScriptBuilder::<DefaultPineOutput>::with_code(source)
        .with_data(data)
        .compile()?
        .run()
        .map(|_| ())
}

fn parse_program(source: &str) -> Result<(Program, PineVersion), Error> {
    let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);

    let mut lexer = Lexer::with_version(source, version);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    Ok((Program::new(statements), version))
}

#[cfg(test)]
mod tests {
    use super::inputs_from_json;
    use pine_core::InputValue;

    #[test]
    fn decodes_input_overrides_from_json() {
        let map = inputs_from_json(r#"{"Length": 20, "Ratio": 1.5, "On": true, "Mode": "fast"}"#)
            .unwrap();
        assert_eq!(map["Length"], InputValue::Int(20));
        assert_eq!(map["Ratio"], InputValue::Float(1.5));
        assert_eq!(map["On"], InputValue::Bool(true));
        assert_eq!(map["Mode"], InputValue::Str("fast".to_string()));
    }
}
