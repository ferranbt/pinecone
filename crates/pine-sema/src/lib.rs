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
//! use pine_core::DefaultPineOutput;
//! use pine_interpreter::Value;
//!
//! // The built-ins the runtime registers; here just `close`.
//! let mut builtins: HashMap<String, Value<DefaultPineOutput>> = HashMap::new();
//! builtins.insert("close".to_string(), Value::Na);
//!
//! let errors = pine_sema::analyze(&program, &builtins, None);
//! assert_eq!(errors.len(), 1);
//! assert_eq!(errors[0].rule, "undeclared-variable");
//! ```

mod analyzer;
mod scope;
mod symbols;

pub use analyzer::Analyzer;
pub use pine_core::LibraryLoader;
pub use pine_diagnostics::{Diagnostic, Severity};
pub use scope::SymbolKind;
pub use symbols::{FileId, ScopeId, ScopeKind, Symbol, SymbolId, SymbolTable};

use pine_ast::Program;
use pine_core::PineOutput;
use pine_interpreter::Value;
use std::collections::HashMap;

/// Run semantic analysis over a parsed program and return every error found.
/// An empty result means the program passed all implemented semantic checks.
///
/// `builtins` is the runtime's registered built-ins (from
/// `pine_builtins::register_namespace_objects` plus the per-bar variables) — the
/// names that resolve without a user declaration. It is taken as the full value
/// map so later passes can inspect the objects' types.
///
/// `loader`, when present, resolves `import`ed libraries so `alias.export`
/// resolves cross-file (and a library's own errors are reported, tagged with the
/// library path). Without it, imports declare only the alias.
pub fn analyze<O: PineOutput>(
    program: &Program,
    builtins: &HashMap<String, Value<O>>,
    loader: Option<&dyn LibraryLoader>,
) -> Vec<Diagnostic> {
    Analyzer::new(builtins, loader).analyze(program)
}

/// Analyze a program and also return the [`SymbolTable`] reconstructed from the
/// same walk — the durable declarations a tool (language server) queries.
pub fn analyze_with_symbols<O: PineOutput>(
    program: &Program,
    builtins: &HashMap<String, Value<O>>,
    loader: Option<&dyn LibraryLoader>,
) -> (Vec<Diagnostic>, SymbolTable) {
    Analyzer::new(builtins, loader).into_analysis(program)
}
