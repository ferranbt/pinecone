//! Static analysis for Pine Script ASTs.
//!
//! `pine-lint` walks a parsed [`pine_ast::Program`] and reports likely bugs and
//! non-idiomatic constructs as [`Diagnostic`]s. It is organized around three
//! pieces:
//!
//! - [`Visitor`] + the `walk_*` functions — the reusable traversal core.
//! - [`LintPass`] — one check; a visitor that collects diagnostics.
//! - [`lint`] — the driver that runs every registered pass over a program.
//!
//! # Example
//!
//! ```
//! use pine_ast::Program;
//! use pine_lexer::Lexer;
//! use pine_parser::Parser;
//!
//! let src = "x = close == na\n";
//! let tokens = Lexer::new(src).tokenize().unwrap();
//! let program = Program::new(Parser::new(tokens).parse().unwrap());
//!
//! let diagnostics = pine_lint::lint(&program);
//! assert_eq!(diagnostics.len(), 1);
//! assert_eq!(diagnostics[0].rule, "eq-na");
//! ```

mod diagnostic;
mod pass;
mod passes;
mod visitor;

#[cfg(test)]
mod test_util;

pub use diagnostic::{Diagnostic, Severity};
pub use pass::{lint, lint_with, LintPass};
pub use visitor::{walk_block, walk_expr, walk_program, walk_stmt, Visitor};
