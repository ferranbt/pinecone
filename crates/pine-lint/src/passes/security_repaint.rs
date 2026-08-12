//! `security-repaint`: `request.security(sym, tf, close)` with a bare current-bar
//! source and no offset.
//!
//! Requesting a current-bar series (`close`, `high`, …) from another timeframe
//! returns the *forming* higher-timeframe bar on realtime bars, so the value
//! changes intrabar and the historical vs. realtime series disagree — a repaint.
//! The non-repainting idiom offsets the expression to a confirmed bar
//! (`close[1]`). Only a bare source is flagged, to stay quiet on the many
//! deliberate uses; `lookahead_on` (the worse case) is left to
//! [`super::RequestLookahead`].

use pine_ast::{Argument, Expr};

use crate::pass::LintPass;
use pine_ast::visitor::{walk_expr, Visitor};
use pine_diagnostics::Diagnostic;

const RULE: &str = "security-repaint";

/// The built-in current-bar price/volume series.
const SOURCES: &[&str] = &[
    "open", "high", "low", "close", "hl2", "hlc3", "ohlc4", "hlcc4", "volume",
];

#[derive(Default)]
pub struct SecurityRepaint {
    diagnostics: Vec<Diagnostic>,
}

fn dotted(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Variable { name, .. } => Some(name.clone()),
        Expr::MemberAccess { object, member, .. } => Some(format!("{}.{}", dotted(object)?, member)),
        _ => None,
    }
}

/// The `index`-th positional argument, or the one named `name`.
fn arg<'a>(args: &'a [Argument], index: usize, name: &str) -> Option<&'a Expr> {
    for a in args {
        if let Argument::Named { name: n, value } = a {
            if n == name {
                return Some(value);
            }
        }
    }
    args.iter()
        .filter_map(|a| match a {
            Argument::Positional(e) => Some(e),
            Argument::Named { .. } => None,
        })
        .nth(index)
}

fn is_bare_source(expr: &Expr) -> bool {
    matches!(expr, Expr::Variable { name, .. } if SOURCES.contains(&name.as_str()))
}

fn uses_lookahead_on(args: &[Argument]) -> bool {
    args.iter().any(|a| {
        let value = match a {
            Argument::Positional(e) => e,
            Argument::Named { value, .. } => value,
        };
        dotted(value).as_deref() == Some("barmerge.lookahead_on")
    })
}

impl Visitor for SecurityRepaint {
    fn visit_expr(&mut self, expr: &Expr) {
        if let Expr::Call { callee, args, loc, .. } = expr {
            if dotted(callee).as_deref() == Some("request.security")
                && arg(args, 2, "expression").is_some_and(is_bare_source)
                && !uses_lookahead_on(args)
            {
                self.diagnostics.push(Diagnostic::warning(
                    RULE,
                    loc.position(),
                    "requesting a current-bar series from another timeframe repaints on the \
                     still-forming bar (its value changes intrabar); offset the expression to a \
                     confirmed bar, e.g. `close[1]`",
                ));
            }
        }
        walk_expr(self, expr);
    }
}

impl LintPass for SecurityRepaint {
    fn name(&self) -> &'static str {
        RULE
    }

    fn finish(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::for_rule;

    #[test]
    fn flags_bare_source_request() {
        let src = "x = request.security(syminfo.tickerid, \"D\", close)\n";
        assert_eq!(for_rule(src, "security-repaint").len(), 1);
    }

    #[test]
    fn ignores_offset_and_expressions_and_lookahead() {
        // Offset to a confirmed bar — the non-repainting idiom.
        assert!(for_rule("x = request.security(syminfo.tickerid, \"D\", close[1])\n", "security-repaint").is_empty());
        // A computed expression, not a bare source.
        assert!(for_rule("x = request.security(syminfo.tickerid, \"D\", ta.sma(close, 20))\n", "security-repaint").is_empty());
        // lookahead_on is RequestLookahead's concern, not this one.
        assert!(for_rule(
            "x = request.security(syminfo.tickerid, \"D\", close, lookahead = barmerge.lookahead_on)\n",
            "security-repaint"
        )
        .is_empty());
    }
}
