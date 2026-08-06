//! The semantic analyzer: a scope-aware walk that emits Tier 1 (name
//! resolution) and Tier 4 (structural) errors.
//!
//! This intentionally does **not** use the shared [`pine_ast::Visitor`]. That
//! traversal is for observational passes; sema needs to push/pop a scope at
//! every block boundary, hoist declarations, and track context (loop depth,
//! global-vs-local), which the default recurse-everything walk doesn't express.
//! So we hand-write the recursion and interleave the scope bookkeeping.

use std::collections::{HashMap, HashSet};

use pine_ast::{Argument, ExportItem, Expr, FunctionParam, Literal, Loc, Program, Stmt};
use pine_core::{LibraryLoader, PineOutput};
use pine_interpreter::{BuiltinSignature, Value};
use pine_parser::Parser;

use crate::scope::{is_global_only, Namespace, SymbolKind};
use crate::symbols::{FileId, ScopeId, ScopeKind, Symbol, SymbolId, SymbolTable};
use pine_diagnostics::Diagnostic;

pub struct Analyzer<'a, O: PineOutput> {
    diagnostics: Vec<Diagnostic>,
    /// Enclosing loops in the current function (reset at function boundaries).
    loop_depth: u32,
    /// The runtime's registered built-ins (namespaces, globals, per-bar variables).
    builtins: &'a HashMap<String, Value<O>>,
    /// Script declarations seen (indicator/strategy/library); at most one allowed.
    declarations: u32,
    /// Whether the current file declared `library(...)` — required of an import.
    library_declared: bool,
    /// Free functions, `name -> (required, total)` param counts, for arity checks.
    functions: HashMap<String, (usize, usize)>,
    /// User type/enum names, collected up front for forward-referencing annotations.
    user_types: HashSet<String>,
    /// Functions enclosing the current point; the last is the caller of any call.
    fn_stack: Vec<String>,
    /// The call graph `(caller, callee, call-site)`, scanned afterwards for cycles.
    call_edges: Vec<CallEdge>,
    /// The durable symbol table; `scope_ids` tracks the current (innermost-last) scope.
    symbols: SymbolTable,
    scope_ids: Vec<ScopeId>,
    /// Resolves `import` paths to source; absent means no cross-file resolution.
    loader: Option<&'a dyn LibraryLoader>,
}

/// Per-file state saved and restored around analyzing a library.
struct FileState {
    scope_ids: Vec<ScopeId>,
    loop_depth: u32,
    functions: HashMap<String, (usize, usize)>,
    user_types: HashSet<String>,
    declarations: u32,
    library_declared: bool,
    fn_stack: Vec<String>,
    call_edges: Vec<CallEdge>,
}

/// One call-graph edge: `(caller, callee, call-site location)`.
type CallEdge = (String, String, Loc);

/// The script-declaration functions — a script must have exactly one.
const SCRIPT_DECLARATIONS: &[&str] = &["study", "indicator", "strategy", "library"];

/// Built-in type names an annotation may use without a user declaration.
const BUILTIN_TYPES: &[&str] = &[
    "int", "float", "bool", "string", "color", "line", "linefill", "label", "box", "table",
    "polyline", "array", "matrix", "map",
];

/// The called name as written, for diagnostics: `plot` or `ta.sma`.
fn callee_name(callee: &Expr) -> String {
    match callee {
        Expr::Variable { name, .. } => name.clone(),
        Expr::MemberAccess { object, member, .. } => match object.as_ref() {
            Expr::Variable {
                name: namespace, ..
            } => format!("{namespace}.{member}"),
            _ => member.clone(),
        },
        _ => String::new(),
    }
}

/// Whether `start` reaches `target` in the call graph (a self-call counts).
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

/// The type names within an annotation: the base and every generic argument,
/// with `[]`/`<>`/`,` stripped (`map<string, Point>` -> `map`, `string`, `Point`).
fn type_names(annotation: &str) -> impl Iterator<Item = &str> {
    annotation
        .split(['<', '>', ',', '[', ']', ' '])
        .filter(|name| !name.is_empty())
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
    pub fn new(
        builtins: &'a HashMap<String, Value<O>>,
        loader: Option<&'a dyn LibraryLoader>,
    ) -> Self {
        Self {
            diagnostics: Vec::new(),
            loop_depth: 0,
            builtins,
            declarations: 0,
            library_declared: false,
            functions: HashMap::new(),
            user_types: HashSet::new(),
            fn_stack: Vec::new(),
            call_edges: Vec::new(),
            symbols: SymbolTable::new(),
            scope_ids: vec![SymbolTable::GLOBAL],
            loader,
        }
    }

    /// The innermost open scope — where names resolve from and declarations are
    /// recorded into.
    fn current_scope(&self) -> ScopeId {
        *self.scope_ids.last().expect("scope stack is never empty")
    }

    fn current_file(&self) -> FileId {
        self.symbols.scope_file(self.current_scope())
    }

    fn current_lib(&self) -> Option<String> {
        let file = self.current_file();
        (file != SymbolTable::MAIN).then(|| self.symbols.file_path(file).to_string())
    }

    /// Open a nested scope in the symbol tree and make it current.
    fn enter_scope(&mut self, kind: ScopeKind) {
        let child = self.symbols.open_scope(self.current_scope(), kind);
        self.scope_ids.push(child);
    }

    /// Close the current scope; its symbols stay in the table as a child scope.
    fn exit_scope(&mut self) {
        self.scope_ids.pop();
    }

    /// Declare a symbol, stamped with the current file.
    fn record(&mut self, mut symbol: Symbol) -> SymbolId {
        symbol.file = self.current_file();
        self.symbols.declare(symbol)
    }

    /// Resolve `name` from the current scope outward.
    fn resolve(&self, name: &str) -> Option<SymbolKind> {
        self.symbols
            .resolve(self.current_scope(), name)
            .map(|symbol| symbol.kind)
    }

    /// Record a use of `name`, if it resolves to a user symbol.
    fn record_use(&mut self, name: &str, loc: Loc) {
        let scope = self.current_scope();
        if let Some(id) = self.symbols.resolve_id(scope, name) {
            let file = self.current_file();
            self.symbols.record_use(file, loc.position(), id);
        }
    }

    /// A declaration's user type, from an annotation or a `Type.new()` initializer.
    fn infer_var_type(
        &self,
        type_annotation: Option<&String>,
        initializer: Option<&Expr>,
    ) -> Option<SymbolId> {
        if let Some(annotation) = type_annotation {
            let base = annotation.trim_end_matches("[]");
            if let Some(id) = self.symbols.resolve_id(self.current_scope(), base) {
                if matches!(
                    self.symbols.symbol(id).kind,
                    SymbolKind::Type | SymbolKind::Enum
                ) {
                    return Some(id);
                }
            }
        }
        // A `Type.new(...)` (or `lib.Type.new(...)`) constructor initializer.
        if let Some(Expr::Call { callee, .. }) = initializer {
            if let Expr::MemberAccess { object, member, .. } = callee.as_ref() {
                if member == "new" {
                    if let Some(id) = self.expr_type(object) {
                        if self.symbols.symbol(id).kind == SymbolKind::Type {
                            return Some(id);
                        }
                    }
                }
            }
        }
        None
    }

    /// The type/enum a symbol denotes: itself, or a variable's `type_ref`.
    fn owner_type(&self, id: SymbolId) -> Option<SymbolId> {
        let symbol = self.symbols.symbol(id);
        match symbol.kind {
            SymbolKind::Type | SymbolKind::Enum => Some(id),
            SymbolKind::Var => symbol.type_ref,
            _ => None,
        }
    }

    /// The type/enum an expression's member access reads from (`Enum.Case`,
    /// `v.field`, `lib.Point`), or `None` when the type is unknown.
    fn expr_type(&self, expr: &Expr) -> Option<SymbolId> {
        let id = match expr {
            Expr::Variable { name, .. } => self.symbols.resolve_id(self.current_scope(), name)?,
            Expr::MemberAccess { object, member, .. } => self.resolve_member(object, member)?,
            _ => return None,
        };
        self.owner_type(id)
    }

    /// The declaration `object.member` points at: a library export, or a member
    /// of the object's user type.
    fn resolve_member(&self, object: &Expr, member: &str) -> Option<SymbolId> {
        if let Some(module) = self.alias_module(object) {
            return self.symbols.exported_id(module, member);
        }
        let owner = self.expr_type(object)?;
        self.symbols.member_id(owner, member)
    }

    /// `object.member` where the object's full member set is known — a builtin
    /// namespace or a resolved import — yet `member` is not among them. False
    /// whenever the members can't be enumerated (a user variable, an unloaded
    /// import), so no diagnostic is invented on incomplete information.
    fn unknown_member(&self, object: &Expr, member: &str) -> bool {
        // A resolved import alias: its export set is fully known.
        if let Some(module) = self.alias_module(object) {
            return self.symbols.exported_id(module, member).is_none();
        }
        // A builtin namespace object, not shadowed by a user declaration. Sema
        // reads the same registry the interpreter calls into, so an absent
        // field is exactly one a run would reject.
        if let Expr::Variable { name, .. } = object {
            if self.resolve(name).is_none() {
                if let Some(Value::Object { fields, .. }) = self.builtins.get(name) {
                    return !fields.borrow().contains_key(member);
                }
            }
        }
        false
    }

    /// The imported library's global scope, if `object` is an import alias.
    fn alias_module(&self, object: &Expr) -> Option<ScopeId> {
        let Expr::Variable { name, .. } = object else {
            return None;
        };
        let id = self.symbols.resolve_id(self.current_scope(), name)?;
        let symbol = self.symbols.symbol(id);
        (symbol.kind == SymbolKind::Import)
            .then_some(symbol.module)
            .flatten()
    }

    fn is_builtin(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    /// The signature of the builtin the callee names (`plot` or `ta.sma`), or
    /// `None` — a user-shadowed name or a builtin with no declared parameters.
    fn builtin_signature(&self, callee: &Expr) -> Option<BuiltinSignature> {
        let value = match callee {
            Expr::Variable { name, .. } => {
                if self.resolve(name).is_some() {
                    return None;
                }
                self.builtins.get(name)?.clone()
            }
            Expr::MemberAccess { object, member, .. } => {
                let Expr::Variable {
                    name: namespace, ..
                } = object.as_ref()
                else {
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

    /// Check a call against the builtin's parameters: too many/few arguments, an
    /// unknown named argument, and a literal of the wrong type.
    fn check_builtin_args(
        &mut self,
        name: &str,
        signature: &BuiltinSignature,
        args: &[Argument],
        loc: Loc,
    ) {
        let positional = args
            .iter()
            .filter(|arg| matches!(arg, Argument::Positional(_)))
            .count();

        if let Some(max) = signature.max_positional() {
            if positional > max {
                self.emit(
                    "too-many-arguments",
                    loc,
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
                            loc,
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
                    loc,
                    format!("`{name}` expects {expected} for `{label}`, found {found}"),
                );
            }
        }

        // Counting (not position-matching) required params stays sound for
        // leading-optional overloads like `ta.highest(length)`.
        let required = signature
            .params
            .iter()
            .filter(|param| param.required)
            .count();
        if args.len() < required {
            self.emit(
                "too-few-arguments",
                loc,
                format!(
                    "`{name}` requires at least {required} arguments, found {}",
                    args.len()
                ),
            );
        }
    }

    /// Analyze one file in its own scope, type set, and call graph.
    fn run_file(&mut self, program: &Program) {
        // Types may be referenced before their declaration, so collect them first.
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

    /// Swap in fresh state for a library, returning the caller's to restore.
    fn enter_file(&mut self, root: ScopeId) -> FileState {
        FileState {
            scope_ids: std::mem::replace(&mut self.scope_ids, vec![root]),
            loop_depth: std::mem::take(&mut self.loop_depth),
            functions: std::mem::take(&mut self.functions),
            user_types: std::mem::take(&mut self.user_types),
            declarations: std::mem::take(&mut self.declarations),
            library_declared: std::mem::take(&mut self.library_declared),
            fn_stack: std::mem::take(&mut self.fn_stack),
            call_edges: std::mem::take(&mut self.call_edges),
        }
    }

    fn exit_file(&mut self, saved: FileState) {
        self.scope_ids = saved.scope_ids;
        self.loop_depth = saved.loop_depth;
        self.functions = saved.functions;
        self.user_types = saved.user_types;
        self.declarations = saved.declarations;
        self.library_declared = saved.library_declared;
        self.fn_stack = saved.fn_stack;
        self.call_edges = saved.call_edges;
    }

    /// Analyze the library at `path` once. Registering it before the walk lets a
    /// re-entrant import find it, which breaks cycles.
    fn resolve_import(&mut self, path: &str, loc: Loc) -> Option<(FileId, ScopeId)> {
        if let Some(file) = self.symbols.file_by_path(path) {
            return Some((file, self.symbols.file_root(file)));
        }
        let loader = self.loader?;
        let source = match loader.load_library(path) {
            Ok(source) => source,
            Err(err) => {
                self.emit(
                    "import-error",
                    loc,
                    format!("cannot load library `{path}`: {err}"),
                );
                return None;
            }
        };
        let program = match Parser::parse_source(&source) {
            Ok(program) => program,
            Err(err) => {
                self.emit(
                    "import-parse-error",
                    loc,
                    format!("cannot parse library `{path}`: {err}"),
                );
                return None;
            }
        };
        let (file, root) = self.symbols.add_file(path);
        let saved = self.enter_file(root);
        self.run_file(&program);
        let is_library = self.library_declared;
        self.exit_file(saved);
        // Reported back in the importing file, at the `import` statement.
        if !is_library {
            self.emit(
                "not-a-library",
                loc,
                format!("imported script `{path}` has no `library()` declaration"),
            );
        }
        Some((file, root))
    }

    /// Analyze a whole program, returning the errors found.
    pub fn analyze(mut self, program: &Program) -> Vec<Diagnostic> {
        self.run_file(program);
        self.diagnostics
    }

    /// Analyze a whole program, returning both the errors and the symbol table
    /// reconstructed from the same walk.
    pub fn into_analysis(mut self, program: &Program) -> (Vec<Diagnostic>, SymbolTable) {
        self.run_file(program);
        (self.diagnostics, self.symbols)
    }

    /// Report call-graph cycles as recursion (Pine forbids it), at the call site.
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

    fn emit(&mut self, rule: &'static str, loc: Loc, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(rule, loc.position(), message).in_file(self.current_lib()));
    }

    fn warn(&mut self, rule: &'static str, loc: Loc, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::warning(rule, loc.position(), message).in_file(self.current_lib()));
    }

    /// Warn that a declaration shadows a built-in (Pine allows it, but warns).
    fn check_shadow(&mut self, name: &str, loc: Loc) {
        if self.is_builtin(name) {
            self.warn(
                "shadows-builtin",
                loc,
                format!("declaration of `{name}` shadows a built-in"),
            );
        }
    }

    /// Reject a type annotation naming a type that is neither a built-in nor a
    /// declared type — including every name inside a generic like `array<Foo>`.
    fn check_type_annotation(&mut self, annotation: Option<&String>, loc: Loc) {
        let Some(annotation) = annotation else {
            return;
        };
        for name in type_names(annotation) {
            if !BUILTIN_TYPES.contains(&name) && !self.user_types.contains(name) {
                self.emit("unknown-type", loc, format!("unknown type `{name}`"));
                return;
            }
        }
    }

    /// Check a user-function call's argument count against its parameters.
    fn check_call_arity(
        &mut self,
        name: &str,
        supplied: usize,
        required: usize,
        total: usize,
        loc: Loc,
    ) {
        if supplied < required {
            self.emit(
                "too-few-arguments",
                loc,
                format!("`{name}` requires at least {required} arguments, found {supplied}"),
            );
        } else if supplied > total {
            self.emit(
                "too-many-arguments",
                loc,
                format!("`{name}` takes at most {total} arguments, found {supplied}"),
            );
        }
    }

    /// Record a user function and walk its body. Declared before the body so a
    /// self-call inside reads as recursion.
    fn analyze_function(
        &mut self,
        name: &str,
        loc: Loc,
        params: &[FunctionParam],
        body: &[Stmt],
    ) -> SymbolId {
        let scope = self.current_scope();
        if self
            .symbols
            .declared_locally_in(scope, name, Namespace::Value)
        {
            self.emit(
                "duplicate-declaration",
                loc,
                format!("`{name}` is already declared in this scope"),
            );
        }
        let id = self.record(
            Symbol::new(name, SymbolKind::Function, loc.position(), scope)
                .with_params(params.iter().map(|p| p.name.clone()).collect()),
        );
        // A parameter with a default may be omitted, so it is not required.
        let required = params.iter().filter(|p| p.default_value.is_none()).count();
        self.functions
            .insert(name.to_string(), (required, params.len()));
        for param in params {
            self.check_type_annotation(param.type_annotation.as_ref(), param.loc);
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
        id
    }

    /// Declare `name` in the current scope, reporting a same-scope duplicate.
    fn declare(&mut self, name: &str, kind: SymbolKind, loc: Loc) -> SymbolId {
        let scope = self.current_scope();
        if self
            .symbols
            .declared_locally_in(scope, name, kind.namespace())
        {
            self.emit(
                "duplicate-declaration",
                loc,
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

    /// Visit a loop body with `loop_depth` raised so `break`/`continue` are legal.
    fn loop_body(&mut self, body: &[Stmt]) {
        self.loop_depth += 1;
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.loop_depth -= 1;
    }

    /// Visit a function body in a fresh scope with `params` bound.
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
            self.check_shadow(name, loc);
            if self.symbols.declared_locally(scope, name) {
                self.emit(
                    "duplicate-parameter",
                    loc,
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
                self.check_type_annotation(type_annotation.as_ref(), *loc);
                self.check_shadow(name, *loc);
                if let Some(Expr::Function { params, body }) = initializer {
                    // A named function `f(x) => …`, lowered to a lambda-valued var.
                    self.analyze_function(name, *loc, params, body);
                } else {
                    // Check the initializer *before* declaring the name, so a
                    // self-reference (`x = x`) resolves against the outer scope.
                    if let Some(init) = initializer {
                        self.check_expr(init);
                    }
                    let scope = self.current_scope();
                    if self
                        .symbols
                        .declared_locally_in(scope, name, Namespace::Value)
                    {
                        self.emit(
                            "duplicate-declaration",
                            *loc,
                            format!(
                                "`{name}` is already declared in this scope (use `:=` to reassign)"
                            ),
                        );
                    }
                    let type_ref =
                        self.infer_var_type(type_annotation.as_ref(), initializer.as_ref());
                    self.record(
                        Symbol::new(name, SymbolKind::Var, loc.position(), scope)
                            .with_type(type_annotation.clone())
                            .with_type_ref(type_ref),
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
                    // `_` is a discard, not a binding: it never collides and is
                    // not recorded.
                    if name == "_" {
                        continue;
                    }
                    self.check_shadow(name, *loc);
                    if self
                        .symbols
                        .declared_locally_in(scope, name, Namespace::Value)
                    {
                        self.emit(
                            "duplicate-declaration",
                            *loc,
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
                step,
                body,
                loc,
            } => {
                self.check_expr(from);
                self.check_expr(to);
                if let Some(step) = step {
                    self.check_expr(step);
                }
                self.enter_scope(ScopeKind::Block);
                self.check_shadow(var_name, *loc);
                let scope = self.current_scope();
                self.record(Symbol::new(
                    var_name,
                    SymbolKind::Var,
                    loc.position(),
                    scope,
                ));
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
                    self.check_shadow(idx, *loc);
                    self.record(Symbol::new(idx, SymbolKind::Var, loc.position(), scope));
                }
                self.check_shadow(item_var, *loc);
                self.record(Symbol::new(
                    item_var,
                    SymbolKind::Var,
                    loc.position(),
                    scope,
                ));
                self.loop_body(body);
                self.exit_scope();
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition);
                self.enter_scope(ScopeKind::Block);
                self.loop_body(body);
                self.exit_scope();
            }
            Stmt::Break { loc } => self.check_loop_keyword("break", *loc),
            Stmt::Continue { loc } => self.check_loop_keyword("continue", *loc),
            Stmt::FunctionDecl {
                name,
                params,
                body,
                export,
                loc,
            } => {
                self.check_shadow(name, *loc);
                let id = self.analyze_function(name, *loc, params, body);
                if *export {
                    self.symbols.mark_exported(id);
                }
            }
            Stmt::MethodDecl {
                name,
                params,
                body,
                export,
                loc,
            } => {
                // Methods overload by receiver type, so the name is not duplicate-checked.
                let scope = self.current_scope();
                let id = self.record(
                    Symbol::new(name, SymbolKind::Function, loc.position(), scope)
                        .with_params(params.iter().map(|p| p.name.clone()).collect()),
                );
                if *export {
                    self.symbols.mark_exported(id);
                }
                for param in params {
                    self.check_type_annotation(param.type_annotation.as_ref(), param.loc);
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
                name,
                fields,
                export,
                loc,
            } => {
                let owner = self.declare(name, SymbolKind::Type, *loc);
                if *export {
                    self.symbols.mark_exported(owner);
                }
                for field in fields {
                    self.check_type_annotation(Some(&field.type_annotation), field.loc);
                    self.symbols.declare_member(
                        owner,
                        &field.name,
                        field.loc.position(),
                        Some(field.type_annotation.clone()),
                    );
                }
            }
            Stmt::EnumDecl {
                name,
                fields,
                export,
                loc,
            } => {
                let owner = self.declare(name, SymbolKind::Enum, *loc);
                if *export {
                    self.symbols.mark_exported(owner);
                }
                for field in fields {
                    self.symbols
                        .declare_member(owner, &field.name, field.loc.position(), None);
                }
            }
            Stmt::Import { path, alias, loc } => {
                let id = self.declare(alias, SymbolKind::Import, *loc);
                if let Some((_, root)) = self.resolve_import(path, *loc) {
                    self.symbols.set_module(id, root);
                }
            }
            // `export name` re-exports an already-declared item: mark it exported.
            Stmt::Export { item } => {
                let name = match item {
                    ExportItem::Function(name) | ExportItem::Type(name) => name,
                };
                if let Some(id) = self.symbols.resolve_id(self.current_scope(), name) {
                    self.symbols.mark_exported(id);
                }
            }
        }
    }

    fn check_loop_keyword(&mut self, keyword: &str, loc: Loc) {
        if self.loop_depth == 0 {
            self.emit(
                "break-outside-loop",
                loc,
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
                    *loc,
                    format!("cannot assign to `{name}`, it is a {}", other.noun()),
                ),
                None if self.is_builtin(name) => self.emit(
                    "reassign-builtin",
                    *loc,
                    format!("cannot reassign built-in `{name}`"),
                ),
                None => self.emit(
                    "invalid-assignment",
                    *loc,
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
                        *loc,
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
                    self.record_use(fname, *fname_loc);
                    if is_global_only(fname) && self.current_scope() != SymbolTable::GLOBAL {
                        self.emit(
                            "global-scope-required",
                            *loc,
                            format!("`{fname}` may only be called in the global scope"),
                        );
                    }
                    if SCRIPT_DECLARATIONS.contains(&fname.as_str()) {
                        self.declarations += 1;
                        if fname == "library" {
                            self.library_declared = true;
                        }
                        if self.declarations > 1 {
                            self.emit(
                                "duplicate-declaration",
                                *loc,
                                "a script may only have one indicator/strategy/library declaration",
                            );
                        }
                    }
                    match self.resolve(fname) {
                        Some(SymbolKind::Function) => {
                            // Record the call as an edge out of the enclosing
                            // function; cycles are found once the walk finishes.
                            if let Some(caller) = self.fn_stack.last() {
                                self.call_edges.push((caller.clone(), fname.clone(), *loc));
                            }
                            if let Some(&(required, total)) = self.functions.get(fname) {
                                self.check_call_arity(fname, args.len(), required, total, *loc);
                            }
                        }
                        // A value, type or enum is not callable.
                        Some(kind @ (SymbolKind::Var | SymbolKind::Type | SymbolKind::Enum)) => {
                            self.emit(
                                "not-callable",
                                *loc,
                                format!("`{fname}` is a {}, not a function", kind.noun()),
                            );
                        }
                        // An import alias is called through its members, not directly.
                        Some(SymbolKind::Import) => {}
                        None => {
                            if !self.is_builtin(fname) {
                                self.emit(
                                    "unknown-function",
                                    *loc,
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
                    self.check_builtin_args(&name, &signature, args, *loc);
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
            Expr::Index { expr, index, .. } => {
                self.check_expr(expr);
                self.check_expr(index);
            }
            // When the object's type is known, record the member's occurrence.
            Expr::MemberAccess {
                object,
                member,
                member_loc,
            } => {
                self.check_expr(object);
                if let Some(id) = self.resolve_member(object, member) {
                    let file = self.current_file();
                    self.symbols.record_use(file, member_loc.position(), id);
                } else if self.unknown_member(object, member) {
                    if let Expr::Variable { name, .. } = object.as_ref() {
                        self.emit(
                            "unknown-member",
                            *member_loc,
                            format!("`{name}` has no member `{member}`"),
                        );
                    }
                }
            }
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
