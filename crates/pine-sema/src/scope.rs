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
        }
    }
}

/// A stack of scopes; the last element is the innermost (current) scope.
pub struct ScopeStack {
    scopes: Vec<HashMap<String, SymbolKind>>,
}

impl ScopeStack {
    /// Create a stack with a single (global) scope already open.
    pub fn new() -> Self {
        Self {
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

    /// Resolve `name` against all enclosing scopes, innermost first.
    pub fn resolve(&self, name: &str) -> Option<SymbolKind> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}
