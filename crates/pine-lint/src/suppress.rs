//! `// @skip(...)` inline suppression of lint findings.
//!
//! A `// @skip` comment silences findings on its own line (a trailing comment)
//! and on the line directly below it (a comment written above the statement):
//!
//! ```text
//! // @skip(request-lookahead)
//! htf = request.security(sym, "D", close, lookahead = barmerge.lookahead_on)
//!
//! x = request.security(sym, "D", close)  // @skip
//! ```
//!
//! `@skip` with no parentheses silences every rule; `@skip(a, b)` silences only
//! the named rules. The directives are read from the [`Program`]'s comment
//! trivia, so no re-scan of the source is needed.
//!
//! [`Program`]: pine_ast::Program

use std::collections::{HashMap, HashSet};

use pine_ast::Comment;
use pine_diagnostics::Diagnostic;

/// Which rules a directive silences on its target lines.
enum Filter {
    All,
    Only(HashSet<String>),
}

impl Filter {
    fn matches(&self, rule: &str) -> bool {
        match self {
            Filter::All => true,
            Filter::Only(rules) => rules.contains(rule),
        }
    }
}

/// The `@skip` directives found in a program's comments, indexed by the 1-based
/// line they sit on.
pub struct Suppressions {
    by_line: HashMap<u32, Filter>,
}

impl Suppressions {
    pub fn from_comments(comments: &[Comment]) -> Self {
        let mut by_line = HashMap::new();
        for comment in comments {
            if let Some(filter) = parse_directive(&comment.text) {
                by_line.insert(comment.line, filter);
            }
        }
        Self { by_line }
    }

    /// Whether `diagnostic` is silenced by a directive on its line (trailing
    /// comment) or the line above it (comment over the statement).
    pub fn suppresses(&self, diagnostic: &Diagnostic) -> bool {
        let Some((line, _)) = diagnostic.pos else {
            return false;
        };
        [line, line.saturating_sub(1)].iter().any(|l| {
            self.by_line
                .get(l)
                .is_some_and(|f| f.matches(diagnostic.rule))
        })
    }
}

/// Parse a `@skip` / `@skip(a, b)` directive from a comment's text, if present.
fn parse_directive(text: &str) -> Option<Filter> {
    let after = text[text.find("@skip")? + "@skip".len()..].trim_start();
    let Some(after) = after.strip_prefix('(') else {
        return Some(Filter::All);
    };
    let rules: HashSet<String> = after[..after.find(')')?]
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    Some(if rules.is_empty() {
        Filter::All
    } else {
        Filter::Only(rules)
    })
}

#[cfg(test)]
mod tests {
    use super::Suppressions;
    use pine_ast::Comment;
    use pine_diagnostics::Diagnostic;

    fn comment(line: u32, text: &str) -> Comment {
        Comment {
            line,
            text: text.to_string(),
        }
    }

    fn diag(rule: &'static str, line: u32) -> Diagnostic {
        Diagnostic::warning(rule, Some((line, 1)), "x")
    }

    #[test]
    fn skips_named_rule_on_line_above_and_same_line() {
        let s = Suppressions::from_comments(&[comment(1, " @skip(a)"), comment(3, " @skip(a)")]);
        assert!(s.suppresses(&diag("a", 2))); // directive on the line above
        assert!(s.suppresses(&diag("a", 3))); // trailing directive on the same line
    }

    #[test]
    fn bare_skip_silences_any_rule_but_named_skip_is_scoped() {
        let s = Suppressions::from_comments(&[comment(1, " @skip"), comment(3, " @skip(a)")]);
        assert!(s.suppresses(&diag("anything", 2))); // bare @skip
        assert!(s.suppresses(&diag("a", 4)));
        assert!(!s.suppresses(&diag("b", 4))); // rule not named
    }

    #[test]
    fn does_not_reach_unrelated_lines() {
        let s = Suppressions::from_comments(&[comment(1, " @skip(a)")]);
        assert!(!s.suppresses(&diag("a", 3))); // two lines below the directive
    }

    #[test]
    fn ignores_ordinary_comments() {
        let s = Suppressions::from_comments(&[comment(1, " just a note")]);
        assert!(!s.suppresses(&diag("a", 1)));
        assert!(!s.suppresses(&diag("a", 2)));
    }

    #[test]
    fn skip_filters_a_real_finding_end_to_end() {
        use crate::test_util::for_rule;
        assert_eq!(for_rule("b = a == na\n", "eq-na").len(), 1);
        assert!(for_rule("// @skip(eq-na)\nb = a == na\n", "eq-na").is_empty());
        assert!(for_rule("b = a == na  // @skip\n", "eq-na").is_empty());
        assert_eq!(for_rule("// @skip(other)\nb = a == na\n", "eq-na").len(), 1);
    }
}
