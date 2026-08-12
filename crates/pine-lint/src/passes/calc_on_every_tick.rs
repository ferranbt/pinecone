//! `calc-on-every-tick`: `strategy(..., calc_on_every_tick = true)`.
//!
//! With `calc_on_every_tick = true` a strategy recomputes on every intrabar
//! tick, so orders and signals can appear mid-bar and then vanish before the bar
//! closes — the classic "my backtest doesn't match live" repaint. The default
//! (compute on bar close) makes historical and realtime behaviour agree. It is a
//! legitimate option, so this is a heads-up rather than an error.

use pine_ast::{Argument, Expr, Literal};

use crate::pass::LintPass;
use pine_ast::visitor::{walk_expr, Visitor};
use pine_diagnostics::Diagnostic;

const RULE: &str = "calc-on-every-tick";

#[derive(Default)]
pub struct CalcOnEveryTick {
    diagnostics: Vec<Diagnostic>,
}

fn is_true(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(Literal::Bool(true)))
}

impl Visitor for CalcOnEveryTick {
    fn visit_expr(&mut self, expr: &Expr) {
        if let Expr::Call {
            callee, args, loc, ..
        } = expr
        {
            let is_strategy =
                matches!(callee.as_ref(), Expr::Variable { name, .. } if name == "strategy");
            if is_strategy {
                for arg in args {
                    if let Argument::Named { name, value } = arg {
                        if name == "calc_on_every_tick" && is_true(value) {
                            self.diagnostics.push(Diagnostic::warning(
                                RULE,
                                loc.position(),
                                "`calc_on_every_tick = true` recomputes the strategy intrabar, so \
                                 signals can appear and then disappear before the bar closes \
                                 (repainting); the default computes on bar close, matching backtest \
                                 and live",
                            ));
                        }
                    }
                }
            }
        }
        walk_expr(self, expr);
    }
}

impl LintPass for CalcOnEveryTick {
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
    fn flags_calc_on_every_tick_true() {
        let src = "strategy(\"S\", calc_on_every_tick = true)\n";
        assert_eq!(for_rule(src, "calc-on-every-tick").len(), 1);
    }

    #[test]
    fn ignores_false_or_absent() {
        assert!(for_rule(
            "strategy(\"S\", calc_on_every_tick = false)\n",
            "calc-on-every-tick"
        )
        .is_empty());
        assert!(for_rule("strategy(\"S\")\n", "calc-on-every-tick").is_empty());
    }
}
