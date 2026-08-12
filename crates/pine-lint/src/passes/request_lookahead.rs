//! `request-lookahead`: `request.security(..., lookahead = barmerge.lookahead_on)`.
//!
//! `barmerge.lookahead_on` makes a higher-timeframe request return the requested
//! bar's *final* value from its first intrabar — data that would not have been
//! known yet on historical bars. That is lookahead bias: the backtest sees the
//! future, and live behaviour won't match. It is only safe on an already-offset
//! series (`close[1]`) or for `time`; otherwise it should be `lookahead_off`.

use pine_ast::{Argument, Expr};

use crate::pass::LintPass;
use pine_ast::visitor::{walk_expr, Visitor};
use pine_diagnostics::Diagnostic;

const RULE: &str = "request-lookahead";

#[derive(Default)]
pub struct RequestLookahead {
    diagnostics: Vec<Diagnostic>,
}

/// A dotted path as written (`request.security`, `barmerge.lookahead_on`), or
/// `None` for anything that is not a name or a chain of member accesses.
fn dotted(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Variable { name, .. } => Some(name.clone()),
        Expr::MemberAccess { object, member, .. } => Some(format!("{}.{}", dotted(object)?, member)),
        _ => None,
    }
}

fn arg_value(arg: &Argument) -> &Expr {
    match arg {
        Argument::Positional(e) => e,
        Argument::Named { value, .. } => value,
    }
}

impl Visitor for RequestLookahead {
    fn visit_expr(&mut self, expr: &Expr) {
        if let Expr::Call {
            callee, args, loc, ..
        } = expr
        {
            if dotted(callee).as_deref() == Some("request.security")
                && args
                    .iter()
                    .any(|a| dotted(arg_value(a)).as_deref() == Some("barmerge.lookahead_on"))
            {
                self.diagnostics.push(Diagnostic::warning(
                    RULE,
                    loc.position(),
                    "`request.security` with `barmerge.lookahead_on` reads the higher-timeframe \
                     bar's final value before it would be known on historical bars (lookahead \
                     bias); use it only with a history-offset series like `close[1]`, or \
                     `barmerge.lookahead_off`",
                ));
            }
        }
        walk_expr(self, expr);
    }
}

impl LintPass for RequestLookahead {
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
    fn flags_lookahead_on_in_security() {
        let src = "x = request.security(syminfo.tickerid, \"D\", close, lookahead = barmerge.lookahead_on)\n";
        assert_eq!(for_rule(src, "request-lookahead").len(), 1);
    }

    #[test]
    fn ignores_default_and_lookahead_off() {
        assert!(for_rule("x = request.security(syminfo.tickerid, \"D\", close)\n", "request-lookahead").is_empty());
        assert!(for_rule(
            "x = request.security(syminfo.tickerid, \"D\", close, lookahead = barmerge.lookahead_off)\n",
            "request-lookahead"
        )
        .is_empty());
    }
}