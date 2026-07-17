//! Semantic analysis for Pine Script — a static pre-check that runs after
//! parsing and before execution.
//!
//! Sema has no registry of its own: the caller supplies the predefined names
//! (built-ins, namespaces, constants) and sema simply resolves every reference
//! against them plus the script's own declarations. It never needs to know what
//! a built-in *is*.
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
//! let errors = pine_sema::analyze(&program, ["close"]);
//! assert_eq!(errors.len(), 1);
//! assert_eq!(errors[0].rule, "undeclared-variable");
//! ```

mod analyzer;
mod scope;

pub use analyzer::Analyzer;
pub use pine_diagnostics::{Diagnostic, Severity};
pub use scope::SymbolKind;

use pine_ast::Program;

/// Run semantic analysis over a parsed program and return every error found.
///
/// `builtins` are the predefined names available to the script. An empty result
/// means the program passed all implemented semantic checks.
pub fn analyze<'a>(
    program: &Program,
    builtins: impl IntoIterator<Item = &'a str>,
) -> Vec<Diagnostic> {
    Analyzer::new(builtins).analyze(program)
}
