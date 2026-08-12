//! Shared helpers for the per-pass unit tests.

use pine_parser::Parser;

use pine_diagnostics::Diagnostic;

/// Lint a Pine snippet and return only the diagnostics for `rule`, in order.
/// Parses through [`Parser::parse_source`] so `// @skip` comments are honored.
/// Panics on lex/parse errors.
pub(crate) fn for_rule(src: &str, rule: &str) -> Vec<Diagnostic> {
    let program = Parser::parse_source(src).expect("snippet should parse");

    crate::lint(&program)
        .into_iter()
        .filter(|d| d.rule == rule)
        .collect()
}
