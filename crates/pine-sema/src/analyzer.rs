//! The semantic analyzer: a scope-aware walk that emits Tier 1 (name
//! resolution) and Tier 4 (structural) errors.
//!
//! This intentionally does **not** use the shared [`pine_ast::Visitor`]. That
//! traversal is for observational passes; sema needs to push/pop a scope at
//! every block boundary, hoist declarations, and track context (loop depth,
//! global-vs-local), which the default recurse-everything walk doesn't express.
//! So we hand-write the recursion and interleave the scope bookkeeping.

use std::collections::HashMap;

use pine_ast::{Argument, Expr, Program, Stmt};
use pine_interpreter::{PineOutput, Value};

use crate::scope::{is_global_only, ScopeStack, SymbolKind};
use pine_diagnostics::Diagnostic;

pub struct Analyzer<'a, O: PineOutput> {
    scopes: ScopeStack,
    diagnostics: Vec<Diagnostic>,
    /// Number of enclosing loops in the *current function*. Reset across
    /// function boundaries — a loop never spans a function.
    loop_depth: u32,
    /// The runtime's registered built-ins — namespaces, global functions, and
    /// per-bar variables that exist without a user declaration. Supplied by the
    /// caller rather than hardcoded here. Kept as the full value map (not just
    /// names) so later passes can inspect the objects' types.
    builtins: &'a HashMap<String, Value<O>>,
}

impl<'a, O: PineOutput> Analyzer<'a, O> {
    pub fn new(builtins: &'a HashMap<String, Value<O>>) -> Self {
        Self {
            scopes: ScopeStack::new(),
            diagnostics: Vec::new(),
            loop_depth: 0,
            builtins,
        }
    }

    fn is_builtin(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    /// Analyze a whole program, returning the errors found.
    pub fn analyze(mut self, program: &Program) -> Vec<Diagnostic> {
        for stmt in &program.statements {
            self.check_stmt(stmt);
        }
        self.diagnostics
    }

    fn emit(&mut self, rule: &'static str, pos: Option<(u32, u32)>, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(rule, pos, message));
    }

    /// Declare `name` in the current scope, reporting a duplicate if it already
    /// exists there. Pine has no hoisting — names become visible in source
    /// order — so this is called at each declaration's position.
    fn declare(&mut self, name: &str, kind: SymbolKind) {
        if self.scopes.declare(name, kind).is_some() {
            self.emit(
                "duplicate-declaration",
                None,
                format!("`{name}` is already declared in this scope"),
            );
        }
    }

    /// Visit a non-loop nested block (an `if`/`else` branch) in its own scope.
    fn block(&mut self, body: &[Stmt]) {
        self.scopes.push();
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.scopes.pop();
    }

    /// Visit a loop body: its own scope, with `loop_depth` raised so
    /// `break`/`continue` are legal inside it.
    fn loop_body(&mut self, body: &[Stmt]) {
        self.loop_depth += 1;
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.loop_depth -= 1;
    }

    /// Visit a function/method/lambda body in a fresh scope with `params`
    /// bound. Loop context does not cross into a function.
    fn function_body<'p>(
        &mut self,
        params: impl Iterator<Item = (&'p str, Option<&'p Expr>)>,
        body: &[Stmt],
    ) {
        self.scopes.push();
        let saved_loop_depth = self.loop_depth;
        self.loop_depth = 0;
        for (name, default) in params {
            if let Some(default) = default {
                self.check_expr(default);
            }
            self.scopes.declare(name, SymbolKind::Var);
        }
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.loop_depth = saved_loop_depth;
        self.scopes.pop();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                // Check the initializer *before* declaring the name, so a
                // self-reference (`x = x`) resolves against the outer scope.
                if let Some(init) = initializer {
                    self.check_expr(init);
                }
                if self.scopes.declare(name, SymbolKind::Var).is_some() {
                    self.emit(
                        "duplicate-declaration",
                        None,
                        format!(
                            "`{name}` is already declared in this scope (use `:=` to reassign)"
                        ),
                    );
                }
            }
            Stmt::Assignment { target, value } => {
                self.check_expr(value);
                self.check_assign_target(target);
            }
            Stmt::TupleAssignment { names, value } => {
                self.check_expr(value);
                for name in names {
                    if self.scopes.declare(name, SymbolKind::Var).is_some() {
                        self.emit(
                            "duplicate-declaration",
                            None,
                            format!("`{name}` is already declared in this scope"),
                        );
                    }
                }
            }
            Stmt::Expression(expr) => self.check_expr(expr),
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                self.check_expr(condition);
                self.block(then_branch);
                for (cond, body) in else_if_branches {
                    self.check_expr(cond);
                    self.block(body);
                }
                if let Some(body) = else_branch {
                    self.block(body);
                }
            }
            Stmt::For {
                var_name,
                from,
                to,
                body,
            } => {
                self.check_expr(from);
                self.check_expr(to);
                self.scopes.push();
                self.scopes.declare(var_name, SymbolKind::Var);
                self.loop_body(body);
                self.scopes.pop();
            }
            Stmt::ForIn {
                index_var,
                item_var,
                collection,
                body,
            } => {
                self.check_expr(collection);
                self.scopes.push();
                if let Some(idx) = index_var {
                    self.scopes.declare(idx, SymbolKind::Var);
                }
                self.scopes.declare(item_var, SymbolKind::Var);
                self.loop_body(body);
                self.scopes.pop();
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition);
                self.scopes.push();
                self.loop_body(body);
                self.scopes.pop();
            }
            Stmt::Break => self.check_loop_keyword("break"),
            Stmt::Continue => self.check_loop_keyword("continue"),
            Stmt::FunctionDecl {
                name, params, body, ..
            } => {
                // Declare the name first so the body may reference it.
                self.declare(name, SymbolKind::Function);
                self.function_body(
                    params
                        .iter()
                        .map(|p| (p.name.as_str(), p.default_value.as_ref())),
                    body,
                );
            }
            Stmt::MethodDecl { params, body, .. } => {
                // Methods may share a name (overload by receiver type), so the
                // name is not declared/duplicate-checked; just check the body.
                self.function_body(
                    params
                        .iter()
                        .map(|p| (p.name.as_str(), p.default_value.as_ref())),
                    body,
                );
            }
            Stmt::TypeDecl { name, .. } => self.declare(name, SymbolKind::Type),
            Stmt::EnumDecl { name, .. } => self.declare(name, SymbolKind::Enum),
            Stmt::Import { alias, .. } => self.declare(alias, SymbolKind::Import),
            // `export` re-exports an already-declared item; nothing to resolve.
            Stmt::Export { .. } => {}
        }
    }

    fn check_loop_keyword(&mut self, keyword: &str) {
        if self.loop_depth == 0 {
            self.emit(
                "break-outside-loop",
                None,
                format!("`{keyword}` is only valid inside a loop"),
            );
        }
    }

    /// Validate the left-hand side of a `:=` reassignment.
    fn check_assign_target(&mut self, target: &Expr) {
        match target {
            Expr::Variable(name) => match self.scopes.resolve(name) {
                Some(SymbolKind::Var) => {}
                Some(other) => self.emit(
                    "invalid-assignment",
                    None,
                    format!("cannot assign to `{name}`, it is a {}", other.noun()),
                ),
                None if self.is_builtin(name) => self.emit(
                    "reassign-builtin",
                    None,
                    format!("cannot reassign built-in `{name}`"),
                ),
                None => self.emit(
                    "invalid-assignment",
                    None,
                    format!(
                        "cannot assign to undeclared variable `{name}` (declare it with `=` first)"
                    ),
                ),
            },
            // `obj.field := …` or `arr[i] := …`: validate the object/index.
            other => self.check_expr(other),
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(name) => {
                if self.scopes.resolve(name).is_none() && !self.is_builtin(name) {
                    self.emit(
                        "undeclared-variable",
                        None,
                        format!("undeclared variable `{name}`"),
                    );
                }
            }
            Expr::Call {
                callee, args, loc, ..
            } => {
                if let Expr::Variable(fname) = callee.as_ref() {
                    let pos = loc.position();
                    if is_global_only(fname) && !self.scopes.at_global() {
                        self.emit(
                            "global-scope-required",
                            pos,
                            format!("`{fname}` may only be called in the global scope"),
                        );
                    }
                    if self.scopes.resolve(fname).is_none() && !self.is_builtin(fname) {
                        self.emit(
                            "unknown-function",
                            pos,
                            format!("unknown function `{fname}`"),
                        );
                    }
                } else {
                    self.check_expr(callee);
                }
                for arg in args {
                    match arg {
                        Argument::Positional(e) => self.check_expr(e),
                        Argument::Named { value, .. } => self.check_expr(value),
                    }
                }
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr(left);
                self.check_expr(right);
            }
            Expr::Unary { expr, .. } => self.check_expr(expr),
            Expr::Index { expr, index } => {
                self.check_expr(expr);
                self.check_expr(index);
            }
            // Members are not validated (that is Tier 3 signature checking);
            // only the base object must resolve.
            Expr::MemberAccess { object, .. } => self.check_expr(object),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.check_expr(condition);
                self.check_expr(then_expr);
                self.check_expr(else_expr);
            }
            Expr::IfExpr {
                condition,
                then_expr,
                else_if_branches,
                else_expr,
            } => {
                self.check_expr(condition);
                self.check_expr(then_expr);
                for (cond, e) in else_if_branches {
                    self.check_expr(cond);
                    self.check_expr(e);
                }
                if let Some(e) = else_expr {
                    self.check_expr(e);
                }
            }
            Expr::Switch { value, cases } => {
                self.check_expr(value);
                for (pattern, result) in cases {
                    self.check_expr(pattern);
                    self.check_expr(result);
                }
            }
            Expr::Array(elements) => {
                for e in elements {
                    self.check_expr(e);
                }
            }
            // A lambda: its own scope with parameters bound.
            Expr::Function { params, body } => {
                self.function_body(
                    params
                        .iter()
                        .map(|p| (p.name.as_str(), p.default_value.as_ref())),
                    body,
                );
            }
            Expr::Literal(_) => {}
        }
    }
}

