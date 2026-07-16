//! Semantic analysis for Pine Script — a static pre-check that runs after
//! parsing and before execution.
//!
//! # Example
//!
//! ```
//! use pine_ast::Program;
//! use pine_lexer::Lexer;
//! use pine_parser::Parser;
//!
//! let src = "x = clse + 1\n"; // typo: `clse`
//! let tokens = Lexer::new(src).tokenize().unwrap();
//! let program = Program::new(Parser::new(tokens).parse().unwrap());
//!
//! let errors = pine_sema::analyze(&program);
//! assert_eq!(errors.len(), 1);
//! assert_eq!(errors[0].rule, "undeclared-variable");
//! ```

mod analyzer;
mod builtins;
mod scope;

pub use analyzer::Analyzer;
pub use pine_diagnostics::{Diagnostic, Severity};
pub use scope::SymbolKind;

use pine_ast::Program;

/// Run semantic analysis over a parsed program and return every error found.
/// An empty result means the program passed all implemented semantic checks.
pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    Analyzer::new().analyze(program)
}
