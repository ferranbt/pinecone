//! `eq-na`: comparing to `na` with `==` / `!=`.
//!
//! In Pine Script, `na` propagates through comparisons: `x == na` evaluates to
//! `na` (falsey), *never* `true`, even when `x` is itself `na`. The idiom is the
//! built-in `na(x)` / `not na(x)`. So `x == na` and `x != na` are almost always
//! bugs — the branch they guard silently never (or always) runs.

use pine_ast::{BinOp, Expr, Literal};

use crate::pass::LintPass;
use pine_ast::visitor::{walk_expr, Visitor};
use pine_diagnostics::Diagnostic;

const RULE: &str = "eq-na";

#[derive(Default)]
pub struct EqNa {
    diagnostics: Vec<Diagnostic>,
}

fn is_na(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(Literal::Na))
}

impl Visitor for EqNa {
    fn visit_expr(&mut self, expr: &Expr) {
        if let Expr::Binary {
            left,
            op: op @ (BinOp::Eq | BinOp::NotEq),
            right,
            loc,
        } = expr
        {
            if is_na(left) || is_na(right) {
                let suggestion = if *op == BinOp::Eq {
                    "na(x)"
                } else {
                    "not na(x)"
                };
                let operator = if *op == BinOp::Eq { "==" } else { "!=" };
                self.diagnostics.push(Diagnostic::error(
                    RULE,
                    loc.position(),
                    format!(
                        "comparing to `na` with `{operator}` always yields `na`, not a boolean; \
                         use `{suggestion}` instead"
                    ),
                ));
            }
        }
        // Keep descending so nested comparisons are caught too.
        walk_expr(self, expr);
    }
}

impl LintPass for EqNa {
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
    fn flags_comparison_with_na() {
        // Either side, `==` or `!=`, reported at the comparison's line.
        let hits = for_rule("a = 1\nb = a == na\nc = na != a\n", "eq-na");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].pos, Some((2, 7)));
    }

    #[test]
    fn ignores_the_proper_na_call() {
        assert!(for_rule("x = na(close)\n", "eq-na").is_empty());
    }
}
