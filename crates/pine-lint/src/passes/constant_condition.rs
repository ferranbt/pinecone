//! `constant-condition`: a branch/loop condition that is a literal boolean.
//!
//! `if true`, `x ? a : b` where the test is `false`, `while false`, etc. are
//! almost always leftover debugging or a mistake: one branch is dead code and
//! the condition carries no information. This pass flags conditions that are a
//! bare boolean literal across `if` statements, `while` loops, ternaries, and
//! `if`-expressions.
//!
//! Note: the condition literal does not yet carry a [`pine_ast::Loc`], so these
//! findings are currently reported without a line. Threading a location through
//! literals/statements is the natural next span-plumbing step.

use pine_ast::{Expr, Literal, Stmt};

use crate::pass::LintPass;
use pine_ast::visitor::{walk_expr, walk_stmt, Visitor};
use pine_diagnostics::Diagnostic;

const RULE: &str = "constant-condition";

#[derive(Default)]
pub struct ConstantCondition {
    diagnostics: Vec<Diagnostic>,
}

impl ConstantCondition {
    fn check(&mut self, condition: &Expr, context: &str) {
        if let Expr::Literal(Literal::Bool(value)) = condition {
            self.diagnostics.push(Diagnostic::warning(
                RULE,
                None,
                format!("`{context}` condition is always `{value}`; the branch is constant"),
            ));
        }
    }
}

impl Visitor for ConstantCondition {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::If { condition, .. } => self.check(condition, "if"),
            Stmt::While { condition, .. } => self.check(condition, "while"),
            _ => {}
        }
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ternary { condition, .. } => self.check(condition, "ternary"),
            Expr::IfExpr { condition, .. } => self.check(condition, "if"),
            _ => {}
        }
        walk_expr(self, expr);
    }
}

impl LintPass for ConstantCondition {
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
    fn flags_literal_conditions_only() {
        assert_eq!(
            for_rule("if true\n    x = 1\n", "constant-condition").len(),
            1
        );
        assert!(for_rule("if close > open\n    x = 1\n", "constant-condition").is_empty());
    }
}
