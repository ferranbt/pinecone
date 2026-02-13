// Re-export all public types from sub-crates
pub use pine_ast as ast;
pub use pine_builtins as builtins;
pub use pine_interpreter as interpreter;
pub use pine_lexer as lexer;
pub use pine_parser as parser;

use pine_ast::Program;
use pine_interpreter::{Bar, DefaultPineOutput, Interpreter, PineOutput, RuntimeError, Value};
use pine_lexer::{Lexer, LexerError};
use pine_parser::{Parser, ParserError};
use std::collections::HashMap;

/// Error type for Pine operations
#[derive(Debug)]
pub enum Error {
    Lexer(LexerError),
    Parser(ParserError),
    Runtime(RuntimeError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(e) => write!(f, "Lexer error: {}", e),
            Error::Parser(e) => write!(f, "Parser error: {}", e),
            Error::Runtime(e) => write!(f, "Runtime error: {}", e),
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
    pub fn compile(source: &str) -> Result<Self, Error> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let program = Program::new(statements);

        // Create interpreter and load builtin namespace objects
        let mut interpreter = Interpreter::new();
        let namespaces = pine_builtins::register_namespace_objects();

        // Register namespace objects as const variables
        for (name, value) in namespaces {
            interpreter.set_const_variable(&name, value);
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
        let mut lexer = Lexer::new(source);
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
