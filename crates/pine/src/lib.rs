// Re-export all public types from sub-crates
pub use pine_ast as ast;
pub use pine_builtins as builtins;
pub use pine_interpreter as interpreter;
pub use pine_lexer as lexer;
pub use pine_parser as parser;

use pine_ast::Program;
use pine_core::{PineVersion, SymInfo, Timeframe, VersionError};
use pine_diagnostics::Diagnostic;
use pine_interpreter::{
    Bar, BoxOutput, HistoricalDataProvider, InputOutput, Interpreter, LabelOutput, LibraryLoader,
    LineOutput, LogOutput, PineOutput, PlotOutput, RuntimeError, TableOutput, Value,
};
use pine_lexer::{Lexer, LexerError};
use pine_parser::{Parser, ParserError};
use std::collections::HashMap;

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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(e) => write!(f, "Lexer error: {}", e),
            Error::Parser(e) => write!(f, "Parser error: {}", e),
            Error::Runtime(e) => write!(f, "Runtime error: {}", e),
            Error::Version(e) => write!(f, "Version error: {}", e),
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

pub struct ScriptBuilder<O: PineOutput> {
    source: String,
    custom_variables: HashMap<String, Value<O>>,
    historical_provider: Option<Box<dyn HistoricalDataProvider<O>>>,
    library_loader: Option<Box<dyn LibraryLoader>>,
    syminfo: Option<SymInfo>,
    timeframe: Option<Timeframe>,
}

impl<O: PineOutput> ScriptBuilder<O> {
    pub fn with_code(source: &str) -> ScriptBuilder<O> {
        Self {
            source: source.to_string(),
            custom_variables: HashMap::new(),
            historical_provider: None,
            library_loader: None,
            syminfo: None,
            timeframe: None,
        }
    }

    /// Host-supplied variables the script can reference, registered as consts
    /// alongside the builtin namespaces.
    pub fn with_custom_variables(mut self, variables: HashMap<String, Value<O>>) -> Self {
        self.custom_variables = variables;
        self
    }

    pub fn with_historical_provider(
        mut self,
        provider: Box<dyn HistoricalDataProvider<O>>,
    ) -> Self {
        self.historical_provider = Some(provider);
        self
    }

    /// Resolves `import` statements. Without one, importing a library fails.
    pub fn with_library_loader(mut self, loader: Box<dyn LibraryLoader>) -> Self {
        self.library_loader = Some(loader);
        self
    }

    /// Symbol information exposed to the script as `syminfo.*`
    pub fn with_syminfo(mut self, syminfo: SymInfo) -> Self {
        self.syminfo = Some(syminfo);
        self
    }

    /// The chart timeframe exposed to the script as `timeframe.*`. Without one,
    /// the namespace is populated with defaults.
    pub fn with_timeframe(mut self, timeframe: Timeframe) -> Self {
        self.timeframe = Some(timeframe);
        self
    }

    /// Compile PineScript source code into a Script with default output
    pub fn compile(self) -> Result<Script<O>, Error>
    where
        O: LogOutput
            + PlotOutput
            + LabelOutput
            + BoxOutput
            + InputOutput
            + LineOutput
            + TableOutput,
    {
        let source = self.source.as_str();
        let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let program = Program::new(statements);

        // Semantic pre-check: reject invalid programs before execution.
        let diagnostics = pine_sema::analyze(&program);
        if !diagnostics.is_empty() {
            return Err(Error::Sema(diagnostics));
        }

        // Create interpreter and load builtin namespace objects
        let mut interpreter = Interpreter::new();
        if let Some(historical_provider) = self.historical_provider {
            interpreter.set_historical_provider(historical_provider);
        }
        if let Some(library_loader) = self.library_loader {
            interpreter.set_library_loader(library_loader);
        }

        let namespaces =
            pine_builtins::register_namespace_objects(version, self.syminfo, self.timeframe);

        // Register namespace objects as const variables
        for (name, value) in namespaces {
            interpreter.set_const_variable(&name, value);
        }

        for (name, value) in self.custom_variables {
            interpreter.set_const_variable(&name, value);
        }

        Ok(Script {
            program,
            interpreter,
        })
    }
}

/// A compiled PineScript program that can be executed multiple times
///
/// This represents a parsed PineScript program that maintains state
/// across multiple bar executions, just like in TradingView.
pub struct Script<O: PineOutput> {
    program: Program,
    interpreter: Interpreter<O>,
}

impl<O: PineOutput> Script<O> {
    pub fn execute(&mut self, bar: &Bar) -> Result<O, Error> {
        // Load bar data as Series variables so TA functions can access historical data
        use interpreter::{Series, Value};

        self.interpreter.set_variable(
            "open",
            Value::Series(Series {
                id: "open".to_string(),
                current: Box::new(Value::Number(bar.open)),
            }),
        );
        self.interpreter.set_variable(
            "high",
            Value::Series(Series {
                id: "high".to_string(),
                current: Box::new(Value::Number(bar.high)),
            }),
        );
        self.interpreter.set_variable(
            "low",
            Value::Series(Series {
                id: "low".to_string(),
                current: Box::new(Value::Number(bar.low)),
            }),
        );
        self.interpreter.set_variable(
            "close",
            Value::Series(Series {
                id: "close".to_string(),
                current: Box::new(Value::Number(bar.close)),
            }),
        );
        self.interpreter.set_variable(
            "volume",
            Value::Series(Series {
                id: "volume".to_string(),
                current: Box::new(Value::Number(bar.volume)),
            }),
        );

        // Per-bar namespaces (barstate) are rebuilt from this bar's flags.
        for (name, value) in pine_builtins::register_per_bar(bar) {
            self.interpreter.set_variable(&name, value);
        }

        let output = self.interpreter.execute(&self.program)?;
        Ok(output)
    }

    /// Execute the script with multiple bars sequentially
    ///
    /// Each bar is processed in order, maintaining state between them.
    pub fn execute_bars(&mut self, bars: &[Bar]) -> Result<(), Error> {
        for bar in bars {
            self.execute(bar)?;
        }
        Ok(())
    }
}
