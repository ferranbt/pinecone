//! Shared helpers for the per-pass unit tests.

use pine_ast::Program;
use pine_lexer::Lexer;
use pine_parser::Parser;

use crate::diagnostic::Diagnostic;

/// Lint a Pine snippet and return only the diagnostics for `rule`, in order.
/// Panics on lex/parse errors.
pub(crate) fn for_rule(src: &str, rule: &str) -> Vec<Diagnostic> {
    let tokens = Lexer::new(src).tokenize().expect("snippet should lex");
    let statements = Parser::new(tokens).parse().expect("snippet should parse");

    crate::lint(&Program::new(statements))
        .into_iter()
        .filter(|d| d.rule == rule)
        .collect()
}
