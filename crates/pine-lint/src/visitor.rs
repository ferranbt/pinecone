//! AST traversal via the visitor pattern.
//!
//! The design mirrors `rustc`/`syn`: the [`Visitor`] trait provides `visit_*`
//! methods whose **default implementations recurse** by delegating to the free
//! `walk_*` functions. A visitor overrides only the node kinds it cares about,
//! and calls the matching `walk_*` (or `walk_expr`/`walk_stmt`) to keep
//! descending into children. The traversal shape lives here, written once, so
//! individual lint passes never re-implement tree-walking.
//!
//! All methods take `&mut self` so a pass can accumulate state (e.g. collected
//! diagnostics) as it walks. The `?Sized` bounds on the `walk_*` functions let
//! them drive a `dyn Visitor`, which the lint driver relies on.

use pine_ast::{Argument, Expr, Program, Stmt};

/// A read-only traversal over the AST.
///
/// Override the `visit_*` methods for the nodes you care about; call the
/// corresponding `walk_*` free function (or [`walk_expr`]/[`walk_stmt`]) from
/// your override to continue into child nodes. Omitting that call prunes the
/// subtree — occasionally useful, but usually you want to recurse.
pub trait Visitor {
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }
}

pub fn walk_program<V: Visitor + ?Sized>(v: &mut V, program: &Program) {
    for stmt in &program.statements {
        v.visit_stmt(stmt);
    }
}

pub fn walk_block<V: Visitor + ?Sized>(v: &mut V, body: &[Stmt]) {
    for stmt in body {
        v.visit_stmt(stmt);
    }
}

pub fn walk_stmt<V: Visitor + ?Sized>(v: &mut V, stmt: &Stmt) {
    match stmt {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(init) = initializer {
                v.visit_expr(init);
            }
        }
        Stmt::Assignment { target, value } => {
            v.visit_expr(target);
            v.visit_expr(value);
        }
        Stmt::TupleAssignment { value, .. } => {
            v.visit_expr(value);
        }
        Stmt::Expression(expr) => {
            v.visit_expr(expr);
        }
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            v.visit_expr(condition);
            walk_block(v, then_branch);
            for (cond, body) in else_if_branches {
                v.visit_expr(cond);
                walk_block(v, body);
            }
            if let Some(body) = else_branch {
                walk_block(v, body);
            }
        }
        Stmt::For {
            from, to, body, ..
        } => {
            v.visit_expr(from);
            v.visit_expr(to);
            walk_block(v, body);
        }
        Stmt::ForIn {
            collection, body, ..
        } => {
            v.visit_expr(collection);
            walk_block(v, body);
        }
        Stmt::While { condition, body } => {
            v.visit_expr(condition);
            walk_block(v, body);
        }
        Stmt::FunctionDecl { body, .. } | Stmt::MethodDecl { body, .. } => {
            walk_block(v, body);
        }
        // Leaf / declaration statements with no child expressions to walk.
        Stmt::Break
        | Stmt::Continue
        | Stmt::TypeDecl { .. }
        | Stmt::EnumDecl { .. }
        | Stmt::Export { .. }
        | Stmt::Import { .. } => {}
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(v: &mut V, expr: &Expr) {
    match expr {
        Expr::Binary { left, right, .. } => {
            v.visit_expr(left);
            v.visit_expr(right);
        }
        Expr::Unary { expr, .. } => {
            v.visit_expr(expr);
        }
        Expr::Call { callee, args, .. } => {
            v.visit_expr(callee);
            for arg in args {
                match arg {
                    Argument::Positional(e) => v.visit_expr(e),
                    Argument::Named { value, .. } => v.visit_expr(value),
                }
            }
        }
        Expr::Index { expr, index } => {
            v.visit_expr(expr);
            v.visit_expr(index);
        }
        Expr::MemberAccess { object, .. } => {
            v.visit_expr(object);
        }
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            v.visit_expr(condition);
            v.visit_expr(then_expr);
            v.visit_expr(else_expr);
        }
        Expr::Function { body, .. } => {
            walk_block(v, body);
        }
        Expr::Array(elements) => {
            for e in elements {
                v.visit_expr(e);
            }
        }
        Expr::Switch { value, cases } => {
            v.visit_expr(value);
            for (pattern, result) in cases {
                v.visit_expr(pattern);
                v.visit_expr(result);
            }
        }
        Expr::IfExpr {
            condition,
            then_expr,
            else_if_branches,
            else_expr,
        } => {
            v.visit_expr(condition);
            v.visit_expr(then_expr);
            for (cond, e) in else_if_branches {
                v.visit_expr(cond);
                v.visit_expr(e);
            }
            if let Some(e) = else_expr {
                v.visit_expr(e);
            }
        }
        // Leaf expressions.
        Expr::Literal(_) | Expr::Variable(_) => {}
    }
}
