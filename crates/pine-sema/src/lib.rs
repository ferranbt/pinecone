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
//! use std::collections::HashMap;
//! use pine_interpreter::{DefaultPineOutput, Value};
//!
//! // The built-ins the runtime registers; here just `close`.
//! let mut builtins: HashMap<String, Value<DefaultPineOutput>> = HashMap::new();
//! builtins.insert("close".to_string(), Value::Na);
//!
//! let errors = pine_sema::analyze(&program, &builtins);
//! assert_eq!(errors.len(), 1);
//! assert_eq!(errors[0].rule, "undeclared-variable");
//! ```

mod analyzer;
mod scope;

pub use analyzer::Analyzer;
pub use pine_diagnostics::{Diagnostic, Severity};
pub use scope::SymbolKind;

use pine_ast::Program;
use pine_interpreter::{PineOutput, Value};
use std::collections::HashMap;

/// Run semantic analysis over a parsed program and return every error found.
/// An empty result means the program passed all implemented semantic checks.
///
/// `builtins` is the runtime's registered built-ins (from
/// `pine_builtins::register_namespace_objects` plus the per-bar variables) — the
/// names that resolve without a user declaration. It is taken as the full value
/// map so later passes can inspect the objects' types.
pub fn analyze<O: PineOutput>(
    program: &Program,
    builtins: &HashMap<String, Value<O>>,
) -> Vec<Diagnostic> {
    Analyzer::new(builtins).analyze(program)
}
