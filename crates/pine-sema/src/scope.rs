//! The scope/symbol table — Tier 0.
//!
//! A stack of lexical scopes. The global program is the bottom scope; every
//! function body and every `if`/`for`/`while` block pushes a new scope (Pine
//! locals are visible only within their block). Name resolution walks the stack
//! from innermost to outermost.

use std::collections::HashMap;

/// What a declared name refers to. This drives rules like "you can't reassign a
/// function" — only [`SymbolKind::Var`] is a reassignable value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A variable (`x = …`, loop variable, tuple binding, parameter).
    Var,
    Function,
    Type,
    Enum,
    /// An import alias (`import foo/bar/1 as alias`).
    Import,
    /// A predefined name supplied by the host (`close`, `ta`, `plot`).
    Builtin,
}

impl SymbolKind {
    /// A human-readable noun for diagnostics.
    pub fn noun(self) -> &'static str {
        match self {
            SymbolKind::Var => "variable",
            SymbolKind::Function => "function",
            SymbolKind::Type => "type",
            SymbolKind::Enum => "enum",
            SymbolKind::Import => "import",
            SymbolKind::Builtin => "built-in",
        }
    }
}

/// A stack of scopes; the last element is the innermost (current) scope.
pub struct ScopeStack {
    /// Predefined names supplied by the host. Outside every scope, so a script
    /// may shadow them.
    builtins: HashMap<String, SymbolKind>,
    scopes: Vec<HashMap<String, SymbolKind>>,
}

impl ScopeStack {
    /// Create a stack with `builtins` as the outermost names and an empty
    /// global scope open.
    ///
    /// Built-ins deliberately sit *outside* the global scope so a script can
    /// shadow one (`var close = 10`) without it counting as a redeclaration.
    pub fn new<'a>(builtins: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            builtins: builtins
                .into_iter()
                .map(|name| (name.to_string(), SymbolKind::Builtin))
                .collect(),
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        // The global scope is never popped.
        debug_assert!(self.scopes.len() > 1, "attempted to pop the global scope");
        self.scopes.pop();
    }

    /// True when the current scope is the global one.
    pub fn at_global(&self) -> bool {
        self.scopes.len() == 1
    }

    /// Declare `name` in the current scope. Returns the previously declared
    /// kind if `name` already exists *in this same scope* (a redeclaration),
    /// otherwise `None`.
    pub fn declare(&mut self, name: &str, kind: SymbolKind) -> Option<SymbolKind> {
        let scope = self
            .scopes
            .last_mut()
            .expect("scope stack always has the global scope");
        scope.insert(name.to_string(), kind)
    }

    /// Resolve `name` against all enclosing scopes, innermost first, falling
    /// back to the built-ins.
    pub fn resolve(&self, name: &str) -> Option<SymbolKind> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .or_else(|| self.builtins.get(name).copied())
    }
}
