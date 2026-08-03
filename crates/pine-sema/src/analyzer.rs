//! The semantic analyzer: a scope-aware walk that emits Tier 1 (name
//! resolution) and Tier 4 (structural) errors.
//!
//! This intentionally does **not** use the shared [`pine_ast::Visitor`]. That
//! traversal is for observational passes; sema needs to push/pop a scope at
//! every block boundary, hoist declarations, and track context (loop depth,
//! global-vs-local), which the default recurse-everything walk doesn't express.
//! So we hand-write the recursion and interleave the scope bookkeeping.

use std::collections::{HashMap, HashSet};

use pine_ast::{Argument, Expr, FunctionParam, Literal, Loc, Program, Stmt};
use pine_core::PineOutput;
use pine_interpreter::{BuiltinSignature, Value};

use crate::scope::{is_global_only, SymbolKind};
use crate::symbols::{ScopeId, ScopeKind, Symbol, SymbolId, SymbolTable};
use pine_diagnostics::Diagnostic;

pub struct Analyzer<'a, O: PineOutput> {
    diagnostics: Vec<Diagnostic>,
    /// Number of enclosing loops in the *current function*. Reset across
    /// function boundaries — a loop never spans a function.
    loop_depth: u32,
    /// The runtime's registered built-ins — namespaces, global functions, and
    /// per-bar variables that exist without a user declaration. Supplied by the
    /// caller rather than hardcoded here. Kept as the full value map (not just
    /// names) so later passes can inspect the objects' types.
    builtins: &'a HashMap<String, Value<O>>,
    /// How many script declarations (`indicator`/`strategy`/`library`) have been
    /// seen. A script may have at most one.
    declarations: u32,
    /// User-declared free functions, `name -> (required, total)` parameter
    /// counts, recorded as each declaration is walked so a later call can be
    /// arity-checked. Methods are excluded (their receiver complicates arity).
    functions: HashMap<String, (usize, usize)>,
    /// Every user type/enum name, collected up front so an annotation may refer
    /// to a type declared later in the file.
    user_types: HashSet<String>,
    /// The functions whose bodies enclose the point being analyzed; the last is
    /// the one a call is currently being made from, so the call can be added to
    /// the graph as an edge out of it.
    fn_stack: Vec<String>,
    /// The call graph, one entry per call: `(caller, callee, call-site)`. Built
    /// during the walk and scanned afterwards — a callee that reaches back to its
    /// caller closes a cycle, which is recursion (Pine forbids it).
    call_edges: Vec<CallEdge>,
    /// The durable symbol table — the analyzer resolves names against it and
    /// records declarations into it as it walks. `scope_ids` tracks the current
    /// scope (its last element is the innermost).
    symbols: SymbolTable,
    scope_ids: Vec<ScopeId>,
}

/// One call-graph edge: `(caller, callee, call-site position)`.
type CallEdge = (String, String, Option<(u32, u32)>);

/// The script-declaration functions — a script must have exactly one.
const SCRIPT_DECLARATIONS: &[&str] = &["study", "indicator", "strategy", "library"];

/// The built-in type names an annotation may name without a user declaration.
/// Qualifiers (`series`/`simple`/`const`/`input`) are a separate field, and the
/// generic collection forms (`array<…>`) are not captured as annotations, so a
/// bare `array`/`matrix`/`map` is all that reaches here for those.
const BUILTIN_TYPES: &[&str] = &[
    "int", "float", "bool", "string", "color", "line", "linefill", "label", "box", "table",
    "polyline", "array", "matrix", "map",
];

/// The called name as written, for diagnostics: `plot` or `ta.sma`.
fn callee_name(callee: &Expr) -> String {
    match callee {
        Expr::Variable { name, .. } => name.clone(),
        Expr::MemberAccess { object, member, .. } => match object.as_ref() {
            Expr::Variable { name: namespace, .. } => format!("{namespace}.{member}"),
            _ => member.clone(),
        },
        _ => String::new(),
    }
}

/// Whether `start` reaches `target` in the call graph. A direct self-call
/// (`start == target`) counts, so this finds a node that participates in a cycle.
fn reaches(start: &str, target: &str, adjacency: &HashMap<&str, Vec<&str>>) -> bool {
    if start == target {
        return true;
    }
    let mut stack = vec![start];
    let mut seen = HashSet::new();
    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }
        if let Some(callees) = adjacency.get(node) {
            for &callee in callees {
                if callee == target {
                    return true;
                }
                stack.push(callee);
            }
        }
    }
    false
}

/// How to name a literal's type in a diagnostic.
fn describe_literal(literal: &Literal) -> &'static str {
    match literal {
        Literal::Int(_) | Literal::Number(_) => "a number",
        Literal::String(_) => "a string",
        Literal::Bool(_) => "a bool",
        Literal::HexColor(_) => "a color",
        Literal::Na => "na",
    }
}

impl<'a, O: PineOutput> Analyzer<'a, O> {
    pub fn new(builtins: &'a HashMap<String, Value<O>>) -> Self {
        Self {
            diagnostics: Vec::new(),
            loop_depth: 0,
            builtins,
            declarations: 0,
            functions: HashMap::new(),
            user_types: HashSet::new(),
            fn_stack: Vec::new(),
            call_edges: Vec::new(),
            symbols: SymbolTable::new(),
            scope_ids: vec![SymbolTable::GLOBAL],
        }
    }

    /// The scope symbols are currently recorded into (mirrors the top of the
    /// `ScopeStack`).
    fn current_scope(&self) -> ScopeId {
        *self.scope_ids.last().expect("scope stack is never empty")
    }

    /// Open a nested scope in the symbol tree and make it current.
    fn enter_scope(&mut self, kind: ScopeKind) {
        let child = self.symbols.open_scope(self.current_scope(), kind);
        self.scope_ids.push(child);
    }

    /// Close the scope opened by [`enter_scope`](Self::enter_scope). Its symbols
    /// stay in the table (as a child scope) but are no longer on the resolution
    /// path, since resolution walks parents.
    fn exit_scope(&mut self) {
        self.scope_ids.pop();
    }

    /// Record a bare-name declaration in the symbol table at the current scope.
    fn record(&mut self, symbol: Symbol) -> SymbolId {
        self.symbols.declare(symbol)
    }

    /// Resolve `name` from the current scope outward — the analyzer's single
    /// name lookup, against the symbol table.
    fn resolve(&self, name: &str) -> Option<SymbolKind> {
        self.symbols
            .resolve(self.current_scope(), name)
            .map(|symbol| symbol.kind)
    }

    /// Record a use of `name` at `loc` in the occurrence index, if it resolves
    /// to a user symbol (builtins and unresolved names have no symbol to point
    /// at). This is what makes find-references and rename possible.
    fn record_use(&mut self, name: &str, loc: Loc) {
        let scope = self.current_scope();
        if let Some(id) = self.symbols.resolve_id(scope, name) {
            self.symbols.record_use(loc.position(), id);
        }
    }

    fn is_builtin(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    /// The arguments the called builtin accepts, if the callee names one.
    ///
    /// Resolves both a bare name (`plot`) and a namespaced one (`ta.sma`, whose
    /// namespace is an object of builtins). A name the script has declared
    /// itself shadows the builtin, and a builtin written by hand carries no
    /// parameters — both yield `None`, so nothing is checked.
    fn builtin_signature(&self, callee: &Expr) -> Option<BuiltinSignature> {
        let value = match callee {
            Expr::Variable { name, .. } => {
                if self.resolve(name).is_some() {
                    return None;
                }
                self.builtins.get(name)?.clone()
            }
            Expr::MemberAccess { object, member, .. } => {
                let Expr::Variable { name: namespace, .. } = object.as_ref() else {
                    return None;
                };
                if self.resolve(namespace).is_some() {
                    return None;
                }
                match self.builtins.get(namespace)? {
                    Value::Object { fields, .. } => fields.borrow().get(member)?.clone(),
                    _ => return None,
                }
            }
            _ => return None,
        };

        match value {
            Value::BuiltinFunction(builtin) if !builtin.signature.params.is_empty() => {
                Some(builtin.signature)
            }
            _ => None,
        }
    }

    /// Check a call's arguments against the builtin's parameters: too many
    /// arguments, too few, an unknown named argument, and an argument whose
    /// literal type the parameter cannot accept.
    fn check_builtin_args(
        &mut self,
        name: &str,
        signature: &BuiltinSignature,
        args: &[Argument],
        pos: Option<(u32, u32)>,
    ) {
        let positional = args
            .iter()
            .filter(|arg| matches!(arg, Argument::Positional(_)))
            .count();

        if let Some(max) = signature.max_positional() {
            if positional > max {
                self.emit(
                    "too-many-arguments",
                    pos,
                    format!("`{name}` takes at most {max} arguments, found {positional}"),
                );
            }
        }

        let mut index = 0;
        for arg in args {
            let (param, value) = match arg {
                Argument::Positional(value) => {
                    let param = signature.positional(index);
                    index += 1;
                    (param, value)
                }
                Argument::Named { name: label, value } => match signature.named(label) {
                    Some(param) => (Some(param), value),
                    None => {
                        self.emit(
                            "unknown-argument",
                            pos,
                            format!("`{name}` has no argument named `{label}`"),
                        );
                        continue;
                    }
                },
            };

            // Only a literal's type is known without inference; anything else
            // is left to the runtime.
            let (Some(param), Expr::Literal(literal)) = (param, value) else {
                continue;
            };
            if !param.ty.accepts(literal) {
                let found = describe_literal(literal);
                let expected = param.ty.describe();
                let label = param.name.clone();
                self.emit(
                    "argument-type",
                    pos,
                    format!("`{name}` expects {expected} for `{label}`, found {found}"),
                );
            }
        }

        // A valid call supplies at least one argument per required parameter, so
        // fewer arguments than required parameters is a guaranteed missing one.
        // Counting arguments rather than matching them to positions keeps this
        // sound for overloads with an optional leading parameter, such as
        // `ta.highest(length)`, where `source` may be dropped.
        let required = signature
            .params
            .iter()
            .filter(|param| param.required)
            .count();
        if args.len() < required {
            self.emit(
                "too-few-arguments",
                pos,
                format!(
                    "`{name}` requires at least {required} arguments, found {}",
                    args.len()
                ),
            );
        }
    }

    /// Walk the program once: emit diagnostics and fill the symbol table.
    fn run(&mut self, program: &Program) {
        // Types are global and may be referenced by an annotation before their
        // declaration, so collect their names before the ordered walk.
        for stmt in &program.statements {
            match stmt {
                Stmt::TypeDecl { name, .. } | Stmt::EnumDecl { name, .. } => {
                    self.user_types.insert(name.clone());
                }
                _ => {}
            }
        }
        for stmt in &program.statements {
            self.check_stmt(stmt);
        }
        self.detect_recursion();
    }

    /// Analyze a whole program, returning the errors found.
    pub fn analyze(mut self, program: &Program) -> Vec<Diagnostic> {
        self.run(program);
        self.diagnostics
    }

    /// Analyze a whole program, returning both the errors and the symbol table
    /// reconstructed from the same walk.
    pub fn into_analysis(mut self, program: &Program) -> (Vec<Diagnostic>, SymbolTable) {
        self.run(program);
        (self.diagnostics, self.symbols)
    }

    /// Scan the call graph for cycles. An edge whose callee can reach its caller
    /// again closes a cycle — the caller is (directly or indirectly) recursive,
    /// which Pine forbids. Reported at the offending call site.
    fn detect_recursion(&mut self) {
        let cycles: Vec<CallEdge> = {
            let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
            for (caller, callee, _) in &self.call_edges {
                adjacency.entry(caller).or_default().push(callee);
            }
            self.call_edges
                .iter()
                .filter(|(caller, callee, _)| reaches(callee, caller, &adjacency))
                .cloned()
                .collect()
        };
        for (caller, callee, pos) in cycles {
            let message = if caller == callee {
                format!("`{caller}` calls itself; Pine does not allow recursion")
            } else {
                format!("`{caller}` and `{callee}` call each other; Pine does not allow recursion")
            };
            self.emit("recursion", pos, message);
        }
    }

    fn emit(&mut self, rule: &'static str, pos: Option<(u32, u32)>, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(rule, pos, message));
    }

    fn warn(&mut self, rule: &'static str, pos: Option<(u32, u32)>, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::warning(rule, pos, message));
    }

    /// Declaring a name that a built-in already owns hides the built-in. Pine
    /// allows it but warns, so this is a warning, not an error.
    fn check_shadow(&mut self, name: &str) {
        if self.is_builtin(name) {
            self.warn(
                "shadows-builtin",
                None,
                format!("declaration of `{name}` shadows a built-in"),
            );
        }
    }

    /// Reject a type annotation that names neither a built-in type nor a
    /// user-declared type/enum. Array suffixes (`Foo[]`) share their element's
    /// type name.
    fn check_type_annotation(&mut self, annotation: Option<&String>) {
        let Some(annotation) = annotation else {
            return;
        };
        let base = annotation.trim_end_matches("[]");
        if BUILTIN_TYPES.contains(&base) || self.user_types.contains(base) {
            return;
        }
        self.emit("unknown-type", None, format!("unknown type `{base}`"));
    }

    /// Check a call's argument count against a user function's parameters, using
    /// the same count-based bounds as the built-in checker.
    fn check_call_arity(
        &mut self,
        name: &str,
        supplied: usize,
        required: usize,
        total: usize,
        pos: Option<(u32, u32)>,
    ) {
        if supplied < required {
            self.emit(
                "too-few-arguments",
                pos,
                format!("`{name}` requires at least {required} arguments, found {supplied}"),
            );
        } else if supplied > total {
            self.emit(
                "too-many-arguments",
                pos,
                format!("`{name}` takes at most {total} arguments, found {supplied}"),
            );
        }
    }

    /// Record a user function, check its parameter annotations, and walk its body
    /// with the name on the recursion stack. The name is declared *before* the
    /// body so a call to it inside reads as recursion.
    fn analyze_function(&mut self, name: &str, loc: Loc, params: &[FunctionParam], body: &[Stmt]) {
        // Declare the function symbol in the enclosing scope before its body.
        let scope = self.current_scope();
        if self.symbols.declared_locally(scope, name) {
            self.emit(
                "duplicate-declaration",
                None,
                format!("`{name}` is already declared in this scope"),
            );
        }
        self.record(
            Symbol::new(name, SymbolKind::Function, loc.position(), scope)
                .with_params(params.iter().map(|p| p.name.clone()).collect()),
        );
        // A parameter with a default may be omitted, so it is not required.
        let required = params.iter().filter(|p| p.default_value.is_none()).count();
        self.functions
            .insert(name.to_string(), (required, params.len()));
        for param in params {
            self.check_type_annotation(param.type_annotation.as_ref());
        }
        self.fn_stack.push(name.to_string());
        self.function_body(
            params.iter().map(|p| {
                (
                    p.name.as_str(),
                    p.default_value.as_ref(),
                    p.loc,
                    p.type_annotation.as_ref(),
                )
            }),
            body,
        );
        self.fn_stack.pop();
    }

    /// Declare `name` in the current scope, reporting a duplicate if it already
    /// exists there. Pine has no hoisting — names become visible in source
    /// order — so this is called at each declaration's position.
    fn declare(&mut self, name: &str, kind: SymbolKind, loc: Loc) -> SymbolId {
        let scope = self.current_scope();
        if self.symbols.declared_locally(scope, name) {
            self.emit(
                "duplicate-declaration",
                None,
                format!("`{name}` is already declared in this scope"),
            );
        }
        self.record(Symbol::new(name, kind, loc.position(), scope))
    }

    /// Visit a non-loop nested block (an `if`/`else` branch) in its own scope.
    fn block(&mut self, body: &[Stmt]) {
        self.enter_scope(ScopeKind::Block);
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.exit_scope();
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
        params: impl Iterator<Item = (&'p str, Option<&'p Expr>, Loc, Option<&'p String>)>,
        body: &[Stmt],
    ) {
        self.enter_scope(ScopeKind::Function);
        let saved_loop_depth = self.loop_depth;
        self.loop_depth = 0;
        let scope = self.current_scope();
        for (name, default, loc, type_annotation) in params {
            if let Some(default) = default {
                self.check_expr(default);
            }
            self.check_shadow(name);
            if self.symbols.declared_locally(scope, name) {
                self.emit(
                    "duplicate-parameter",
                    None,
                    format!("parameter `{name}` is declared more than once"),
                );
            }
            self.record(
                Symbol::new(name, SymbolKind::Var, loc.position(), scope)
                    .with_type(type_annotation.cloned()),
            );
        }
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.loop_depth = saved_loop_depth;
        self.exit_scope();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                initializer,
                type_annotation,
                loc,
                ..
            } => {
                self.check_type_annotation(type_annotation.as_ref());
                self.check_shadow(name);
                if let Some(Expr::Function { params, body }) = initializer {
                    // A named function definition `f(x) => …`, which the parser
                    // lowers to a lambda-valued variable. `analyze_function`
                    // records it before its body, so a call to it inside the body
                    // reads as recursion and calls resolve and are arity-checked.
                    self.analyze_function(name, *loc, params, body);
                } else {
                    // Check the initializer *before* declaring the name, so a
                    // self-reference (`x = x`) resolves against the outer scope.
                    if let Some(init) = initializer {
                        self.check_expr(init);
                    }
                    let scope = self.current_scope();
                    if self.symbols.declared_locally(scope, name) {
                        self.emit(
                            "duplicate-declaration",
                            None,
                            format!(
                                "`{name}` is already declared in this scope (use `:=` to reassign)"
                            ),
                        );
                    }
                    self.record(
                        Symbol::new(name, SymbolKind::Var, loc.position(), scope)
                            .with_type(type_annotation.clone()),
                    );
                }
            }
            Stmt::Assignment { target, value } => {
                self.check_expr(value);
                self.check_assign_target(target);
            }
            Stmt::TupleAssignment {
                names, value, loc, ..
            } => {
                self.check_expr(value);
                let scope = self.current_scope();
                for name in names {
                    self.check_shadow(name);
                    if self.symbols.declared_locally(scope, name) {
                        self.emit(
                            "duplicate-declaration",
                            None,
                            format!("`{name}` is already declared in this scope"),
                        );
                    }
                    self.record(Symbol::new(name, SymbolKind::Var, loc.position(), scope));
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
                loc,
            } => {
                self.check_expr(from);
                self.check_expr(to);
                self.enter_scope(ScopeKind::Block);
                self.check_shadow(var_name);
                let scope = self.current_scope();
                self.record(Symbol::new(var_name, SymbolKind::Var, loc.position(), scope));
                self.loop_body(body);
                self.exit_scope();
            }
            Stmt::ForIn {
                index_var,
                item_var,
                collection,
                body,
                loc,
            } => {
                self.check_expr(collection);
                self.enter_scope(ScopeKind::Block);
                let scope = self.current_scope();
                if let Some(idx) = index_var {
                    self.check_shadow(idx);
                    self.record(Symbol::new(idx, SymbolKind::Var, loc.position(), scope));
                }
                self.check_shadow(item_var);
                self.record(Symbol::new(item_var, SymbolKind::Var, loc.position(), scope));
                self.loop_body(body);
                self.exit_scope();
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition);
                self.enter_scope(ScopeKind::Block);
                self.loop_body(body);
                self.exit_scope();
            }
            Stmt::Break => self.check_loop_keyword("break"),
            Stmt::Continue => self.check_loop_keyword("continue"),
            Stmt::FunctionDecl {
                name,
                params,
                body,
                loc,
                ..
            } => {
                self.check_shadow(name);
                self.analyze_function(name, *loc, params, body);
            }
            Stmt::MethodDecl {
                name,
                params,
                body,
                loc,
                ..
            } => {
                // Methods may share a name (overload by receiver type), so the
                // name is not declared/duplicate-checked; just check the body.
                let scope = self.current_scope();
                self.record(
                    Symbol::new(name, SymbolKind::Function, loc.position(), scope)
                        .with_params(params.iter().map(|p| p.name.clone()).collect()),
                );
                for param in params {
                    self.check_type_annotation(param.type_annotation.as_ref());
                }
                self.function_body(
                    params.iter().map(|p| {
                        (
                            p.name.as_str(),
                            p.default_value.as_ref(),
                            p.loc,
                            p.type_annotation.as_ref(),
                        )
                    }),
                    body,
                );
            }
            Stmt::TypeDecl {
                name, fields, loc, ..
            } => {
                let owner = self.declare(name, SymbolKind::Type, *loc);
                for field in fields {
                    self.check_type_annotation(Some(&field.type_annotation));
                    self.symbols.declare_member(
                        owner,
                        &field.name,
                        field.loc.position(),
                        Some(field.type_annotation.clone()),
                    );
                }
            }
            Stmt::EnumDecl {
                name, fields, loc, ..
            } => {
                let owner = self.declare(name, SymbolKind::Enum, *loc);
                for field in fields {
                    self.symbols
                        .declare_member(owner, &field.name, field.loc.position(), None);
                }
            }
            Stmt::Import { alias, loc, .. } => {
                self.declare(alias, SymbolKind::Import, *loc);
            }
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
            Expr::Variable { name, loc } => match self.resolve(name) {
                Some(SymbolKind::Var) => self.record_use(name, *loc),
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
            Expr::Variable { name, loc } => {
                if self.resolve(name).is_none() && !self.is_builtin(name) {
                    self.emit(
                        "undeclared-variable",
                        None,
                        format!("undeclared variable `{name}`"),
                    );
                } else {
                    self.record_use(name, *loc);
                }
            }
            Expr::Call {
                callee, args, loc, ..
            } => {
                if let Expr::Variable {
                    name: fname,
                    loc: fname_loc,
                } = callee.as_ref()
                {
                    let pos = loc.position();
                    self.record_use(fname, *fname_loc);
                    if is_global_only(fname) && self.current_scope() != SymbolTable::GLOBAL {
                        self.emit(
                            "global-scope-required",
                            pos,
                            format!("`{fname}` may only be called in the global scope"),
                        );
                    }
                    if SCRIPT_DECLARATIONS.contains(&fname.as_str()) {
                        self.declarations += 1;
                        if self.declarations > 1 {
                            self.emit(
                                "duplicate-declaration",
                                pos,
                                "a script may only have one indicator/strategy/library declaration",
                            );
                        }
                    }
                    match self.resolve(fname) {
                        Some(SymbolKind::Function) => {
                            // Record the call as an edge out of the enclosing
                            // function; cycles are found once the walk finishes.
                            if let Some(caller) = self.fn_stack.last() {
                                self.call_edges.push((caller.clone(), fname.clone(), pos));
                            }
                            if let Some(&(required, total)) = self.functions.get(fname) {
                                self.check_call_arity(fname, args.len(), required, total, pos);
                            }
                        }
                        // A value, type or enum is not callable.
                        Some(kind @ (SymbolKind::Var | SymbolKind::Type | SymbolKind::Enum)) => {
                            self.emit(
                                "not-callable",
                                pos,
                                format!("`{fname}` is a {}, not a function", kind.noun()),
                            );
                        }
                        // An import alias is called through its members, not directly.
                        Some(SymbolKind::Import) => {}
                        None => {
                            if !self.is_builtin(fname) {
                                self.emit(
                                    "unknown-function",
                                    pos,
                                    format!("unknown function `{fname}`"),
                                );
                            }
                        }
                    }
                } else {
                    self.check_expr(callee);
                }
                if let Some(signature) = self.builtin_signature(callee) {
                    let name = callee_name(callee);
                    self.check_builtin_args(&name, &signature, args, loc.position());
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
                    params.iter().map(|p| {
                        (
                            p.name.as_str(),
                            p.default_value.as_ref(),
                            p.loc,
                            p.type_annotation.as_ref(),
                        )
                    }),
                    body,
                );
            }
            Expr::Literal(_) => {}
        }
    }
}
