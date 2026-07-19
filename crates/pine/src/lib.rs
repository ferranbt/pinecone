// Re-export all public types from sub-crates
pub use pine_ast as ast;
pub use pine_builtins as builtins;
pub use pine_interpreter as interpreter;
pub use pine_lexer as lexer;
pub use pine_parser as parser;

use interpreter::{Series, Value};
use pine_ast::Program;
use pine_core::{PineVersion, Timeframe, VersionError};
use pine_diagnostics::Diagnostic;
use pine_interpreter::{Bar, DefaultPineOutput, Interpreter, PineOutput, RuntimeError};
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

/// The series every script can reference, refreshed on each bar: the bar's own
/// OHLCV plus the values Pine derives from them.
fn bar_series(bar: &Bar) -> [(&'static str, f64); 8] {
    [
        ("open", bar.open),
        ("high", bar.high),
        ("low", bar.low),
        ("close", bar.close),
        ("volume", bar.volume),
        ("hl2", (bar.high + bar.low) / 2.0),
        ("hlc3", (bar.high + bar.low + bar.close) / 3.0),
        ("ohlc4", (bar.open + bar.high + bar.low + bar.close) / 4.0),
    ]
}

/// A compiled PineScript program that can be executed multiple times
///
/// This represents a parsed PineScript program that maintains state
/// across multiple bar executions, just like in TradingView.
pub struct Script<O: PineOutput = DefaultPineOutput> {
    program: Program,
    interpreter: Interpreter<O>,
}

impl Script<DefaultPineOutput> {
    /// Compile PineScript source code into a Script with default output
    pub fn compile(source: &str, timeframe: Option<Timeframe>) -> Result<Self, Error> {
        let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);

        // Which words are keywords depends on the version: `type` is an
        // ordinary identifier in v4 but a keyword from v5 on.
        let mut lexer = Lexer::with_version(source, version);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let program = Program::new(statements);

        // Create interpreter and load builtin namespace objects
        let mut interpreter = Interpreter::new();
        let namespaces = pine_builtins::register_namespace_objects(timeframe.unwrap_or_default());

        // Register namespace objects as const variables
        for (name, value) in namespaces {
            interpreter.set_const_variable(&name, value);
        }

        // The per-bar series exist from the start; `execute` refreshes their
        // values on every bar.
        for (name, _) in bar_series(&Bar::default()) {
            interpreter.set_variable(name, Value::Na);
        }

        // Semantic pre-check: reject invalid programs before execution. Nothing
        // has run yet, so the interpreter's registered variables are exactly the
        // predefined names a script may reference.
        let diagnostics = pine_sema::analyze(&program, interpreter.variable_names());
        if !diagnostics.is_empty() {
            return Err(Error::Sema(diagnostics));
        }

        Ok(Self {
            program,
            interpreter,
        })
    }
}

impl<O: PineOutput> Script<O> {
    pub fn compile_with_variables(
        source: &str,
        custom_variables: HashMap<String, Value<O>>,
    ) -> Result<Self, Error> {
        let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);

        // Which words are keywords depends on the version: `type` is an
        // ordinary identifier in v4 but a keyword from v5 on.
        let mut lexer = Lexer::with_version(source, version);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let program = Program::new(statements);

        // Create interpreter with custom output type
        let mut interpreter: Interpreter<O> = Interpreter::new();

        // Register custom variables
        for (name, value) in custom_variables {
            interpreter.set_const_variable(&name, value);
        }

        Ok(Self {
            program,
            interpreter,
        })
    }

    pub fn execute(&mut self, bar: &Bar) -> Result<O, Error> {
        let (name, value) = pine_builtins::register_bar_state(bar);
        self.interpreter.set_const_variable(name, value);

        // `timeframe.change` compares this bar against the previous one.
        self.interpreter.set_bar_time(bar.time);

        for (name, value) in bar_series(bar) {
            self.interpreter.set_variable(
                name,
                Value::Series(Series {
                    id: name.to_string(),
                    current: Box::new(Value::Number(value)),
                }),
            );
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

    /// Set the historical data provider for accessing past bar data
    ///
    /// This is required for TA functions that need to look back at historical values.
    pub fn set_historical_provider(
        &mut self,
        provider: Box<dyn pine_interpreter::HistoricalDataProvider<O>>,
    ) {
        self.interpreter.set_historical_provider(provider);
    }

    pub fn set_library_loader(&mut self, provider: Box<dyn pine_interpreter::LibraryLoader>) {
        self.interpreter.set_library_loader(provider);
    }

    /// Get a mutable reference to the interpreter
    ///
    /// This allows direct access to the interpreter for advanced use cases like
    /// updating the historical provider state between bar executions.
    pub fn interpreter_mut(&mut self) -> &mut Interpreter<O> {
        &mut self.interpreter
    }
}
