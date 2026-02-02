// Re-export all public types from sub-crates
pub use pine_ast as ast;
pub use pine_builtins as builtins;
pub use pine_interpreter as interpreter;
pub use pine_lexer as lexer;
pub use pine_parser as parser;

use pine_ast::Program;
use pine_interpreter::{Bar, Interpreter, RuntimeError, Value};
use pine_lexer::{Lexer, LexerError};
use pine_parser::{Parser, ParserError};

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
pub struct Script {
    program: Program,
    interpreter: Interpreter,
}

impl Script {
    /// Compile PineScript source code into a Script
    pub fn compile(source: &str) -> Result<Self, Error> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let program = Program::new(statements);

        // Create interpreter and load builtin namespace objects
        let mut interpreter = Interpreter::new();
        let namespaces = pine_builtins::register_namespace_objects();
        for (name, value) in namespaces {
            interpreter.set_variable(&name, value);
        }

        Ok(Self {
            program,
            interpreter,
        })
    }

    /// Execute the script with a single bar
    ///
    /// This maintains interpreter state across multiple calls,
    /// allowing variables to persist between bars.
    pub fn execute(&mut self, bar: &Bar) -> Result<(), Error> {
        // Load bar data as variables in the interpreter context
        self.interpreter.set_variable("open", Value::Number(bar.open));
        self.interpreter.set_variable("high", Value::Number(bar.high));
        self.interpreter.set_variable("low", Value::Number(bar.low));
        self.interpreter.set_variable("close", Value::Number(bar.close));
        self.interpreter.set_variable("volume", Value::Number(bar.volume));

        self.interpreter.execute(&self.program)?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use interpreter::Value;

    #[test]
    fn test_builtin_array_operations() -> Result<(), Error> {
        let source = r#"
            var new_float_fn = array.new_float
            var a = new_float_fn(3, 10.0)
            var size_fn = array.size
            var size = size_fn(a)
        "#;

        let mut script = Script::compile(source)?;
        script.execute(&Bar::default())?;

        assert_eq!(script.interpreter.get_variable("size"), Some(&Value::Number(3.0)));

        Ok(())
    }
}
