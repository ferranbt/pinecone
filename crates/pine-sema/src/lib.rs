//! Semantic analysis for Pine Script — a static pre-check that runs after
//! parsing and before execution.
//!
//! Where [`pine-lint`](../pine_lint/index.html) reports *warnings* about
//! legal-but-suspect code, `pine-sema` reports *errors*: programs that real
//! Pine would reject. It answers questions a single AST node can't — "does this
//! name resolve?", "is `break` inside a loop?", "is `plot()` at global scope?" —
//! by building a scope/symbol table (Tier 0) and walking the tree with that
//! context.
//!
//! Current coverage:
//! - **Tier 1 — name resolution:** undeclared variables, unknown functions,
//!   assigning to an undeclared/non-variable name, reassigning a built-in,
//!   duplicate declarations, `break`/`continue` outside a loop.
//! - **Tier 4 — structural:** plot-family functions must be called at global
//!   scope.
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
pub use scope::SymbolKind;
// The diagnostic type is shared across the toolchain; re-exported for
// convenience so existing `pine_sema::Diagnostic` paths keep working.
pub use pine_diagnostics::{Diagnostic, Severity};

use pine_ast::Program;

/// Run semantic analysis over a parsed program and return every error found.
/// An empty result means the program passed all implemented semantic checks.
pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    Analyzer::new().analyze(program)
}
