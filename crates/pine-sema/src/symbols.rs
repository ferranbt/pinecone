//! A durable symbol table.
//!
//! Every declaration a script makes — its kind, where it is written, its
//! parameters or type — arranged in the scope tree they belong to. That is what
//! a tool queries: go-to-definition (a symbol's `decl`), hover (its kind +
//! signature), completion (the symbols visible in a scope, or the members of a
//! type).
//!
//! This is the analyzer's *only* scope structure: [`Analyzer`](crate::Analyzer)
//! both resolves names against it and records declarations into it as it walks,
//! so name resolution and the durable table are one and the same. The AST is
//! visited once.
//!
//! It records **declarations** only. Mapping a *use* back to its declaration
//! (find-references, rename) additionally needs a span on every identifier
//! occurrence — a later layer on top of this one, not a change to it.

pub use crate::scope::SymbolKind;

/// A scope's index within a [`SymbolTable`]. `0` is always the global scope.
pub type ScopeId = usize;

/// A symbol's index within a [`SymbolTable`].
pub type SymbolId = usize;

/// What opened a scope — for display and for scope-aware queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// The whole script.
    Global,
    /// A function/method body; its parameters are declared here.
    Function,
    /// A loop or `if`/`else` block.
    Block,
}

/// A declared name and what is known about it at its declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    /// Where the name is written, `(line, column)`; `None` when unknown.
    pub decl: Option<(u32, u32)>,
    /// The scope the name is declared in.
    pub scope: ScopeId,
    /// Parameter names in order — set for functions and methods (arity, and a
    /// first cut at signature help).
    pub params: Vec<String>,
    /// The declared type annotation, if written (`float`, `InfoLabel`, …).
    pub type_annotation: Option<String>,
    /// The type/enum this symbol is a member of, for a field or enum case.
    /// `None` for a top-level declaration. A member is *not* reachable by bare
    /// name (it is accessed as `owner.member`), so it never enters a scope's
    /// resolution list — `container` is how it is found instead.
    pub container: Option<SymbolId>,
}

impl Symbol {
    /// A plain declaration reachable by bare name in `scope`.
    pub(crate) fn new(name: &str, kind: SymbolKind, decl: Option<(u32, u32)>, scope: ScopeId) -> Self {
        Symbol {
            name: name.to_string(),
            kind,
            decl,
            scope,
            params: Vec::new(),
            type_annotation: None,
            container: None,
        }
    }

    pub(crate) fn with_params(mut self, params: Vec<String>) -> Self {
        self.params = params;
        self
    }

    pub(crate) fn with_type(mut self, type_annotation: Option<String>) -> Self {
        self.type_annotation = type_annotation;
        self
    }
}

#[derive(Debug, Clone)]
struct ScopeData {
    parent: Option<ScopeId>,
    kind: ScopeKind,
    /// Symbols reachable by bare name in this scope, in source order.
    symbols: Vec<SymbolId>,
}

/// A *use* of a symbol: where a name resolves to a declaration. This is what
/// turns the table from declaration-only into find-references / rename capable.
#[derive(Debug, Clone)]
struct Occurrence {
    line: u32,
    column: u32,
    symbol: SymbolId,
}

/// The scope tree of a program, the symbols each scope declares, and every use
/// of those symbols.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    scopes: Vec<ScopeData>,
    occurrences: Vec<Occurrence>,
}

impl SymbolTable {
    /// The global scope, always present.
    pub const GLOBAL: ScopeId = 0;

    // --- Building (driven by the analyzer's walk) ---

    pub(crate) fn new() -> Self {
        SymbolTable {
            symbols: Vec::new(),
            scopes: vec![ScopeData {
                parent: None,
                kind: ScopeKind::Global,
                symbols: Vec::new(),
            }],
            occurrences: Vec::new(),
        }
    }

    /// Open a child scope of `parent` and return its id.
    pub(crate) fn open_scope(&mut self, parent: ScopeId, kind: ScopeKind) -> ScopeId {
        let id = self.scopes.len();
        self.scopes.push(ScopeData {
            parent: Some(parent),
            kind,
            symbols: Vec::new(),
        });
        id
    }

    /// Record a symbol reachable by bare name in its scope.
    pub(crate) fn declare(&mut self, symbol: Symbol) -> SymbolId {
        let scope = symbol.scope;
        let id = self.symbols.len();
        self.symbols.push(symbol);
        self.scopes[scope].symbols.push(id);
        id
    }

    /// Whether `name` is already declared directly in `scope` (ignoring parents)
    /// — a redeclaration in the same scope. Drives the duplicate-declaration
    /// diagnostic now that the analyzer resolves against this table.
    pub(crate) fn declared_locally(&self, scope: ScopeId, name: &str) -> bool {
        self.scopes[scope]
            .symbols
            .iter()
            .any(|&id| self.symbols[id].name == name)
    }

    /// Record that the name at `pos` refers to `symbol` — one entry in the
    /// occurrence index. `None` positions (unknown) are dropped.
    pub(crate) fn record_use(&mut self, pos: Option<(u32, u32)>, symbol: SymbolId) {
        if let Some((line, column)) = pos {
            self.occurrences.push(Occurrence {
                line,
                column,
                symbol,
            });
        }
    }

    /// Record a member (field or enum case) of `owner`. Not reachable by bare
    /// name, so not added to any scope's resolution list.
    pub(crate) fn declare_member(
        &mut self,
        owner: SymbolId,
        name: &str,
        decl: Option<(u32, u32)>,
        type_annotation: Option<String>,
    ) {
        let scope = self.symbols[owner].scope;
        self.symbols.push(Symbol {
            name: name.to_string(),
            kind: SymbolKind::Var,
            decl,
            scope,
            params: Vec::new(),
            type_annotation,
            container: Some(owner),
        });
    }

    // --- Queries ---

    /// Every symbol declared anywhere in the program, in declaration order.
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// The kind of a scope.
    pub fn scope_kind(&self, scope: ScopeId) -> ScopeKind {
        self.scopes[scope].kind
    }

    /// The symbols reachable by bare name directly in `scope`.
    pub fn symbols_in(&self, scope: ScopeId) -> impl Iterator<Item = &Symbol> {
        self.scopes[scope].symbols.iter().map(|&id| &self.symbols[id])
    }

    /// The members (fields, enum cases) of a type or enum symbol — the
    /// candidates for completion after `owner.`.
    pub fn members_of(&self, owner: SymbolId) -> impl Iterator<Item = &Symbol> {
        self.symbols
            .iter()
            .filter(move |s| s.container == Some(owner))
    }

    /// The symbol at index `id`.
    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id]
    }

    /// Resolve `name` from `scope` outward, innermost first — Pine's own lookup
    /// order. This powers hover / go-to-definition once a cursor has been mapped
    /// to its scope.
    pub fn resolve(&self, scope: ScopeId, name: &str) -> Option<&Symbol> {
        self.resolve_id(scope, name).map(|id| &self.symbols[id])
    }

    /// Like [`resolve`](Self::resolve) but returns the symbol's id, so a use can
    /// be recorded against it.
    pub fn resolve_id(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        let mut current = Some(scope);
        while let Some(id) = current {
            let data = &self.scopes[id];
            if let Some(&sym) = data
                .symbols
                .iter()
                .rev()
                .find(|&&s| self.symbols[s].name == name)
            {
                return Some(sym);
            }
            current = data.parent;
        }
        None
    }

    /// Every position where `symbol` is *used* (its declaration is not included
    /// — that is [`Symbol::decl`]). This is find-references, and the set rename
    /// would rewrite.
    pub fn references(&self, symbol: SymbolId) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.occurrences
            .iter()
            .filter(move |o| o.symbol == symbol)
            .map(|o| (o.line, o.column))
    }

    /// The symbol referenced or declared at `(line, column)` — hover and
    /// go-to-definition, whether the cursor sits on a use or the declaration.
    pub fn symbol_at(&self, line: u32, column: u32) -> Option<SymbolId> {
        if let Some(occ) = self
            .occurrences
            .iter()
            .find(|o| o.line == line && o.column == column)
        {
            return Some(occ.symbol);
        }
        self.symbols
            .iter()
            .position(|s| s.decl == Some((line, column)))
    }

    /// The symbol *declared* at `(line, column)`, if any. Enough for hover and
    /// go-to-definition when the cursor sits on the declaration itself; resolving
    /// a cursor on a *use* needs the occurrence layer.
    pub fn find_declaration_at(&self, line: u32, column: u32) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|s| s.decl == Some((line, column)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{analyze_with_symbols, SymbolKind, SymbolTable};
    use pine_ast::Program;
    use pine_core::DefaultPineOutput;
    use pine_interpreter::Value;
    use pine_lexer::Lexer;
    use pine_parser::Parser;
    use std::collections::HashMap;

    /// Build the symbol table via the analyzer's single walk.
    fn table(source: &str) -> SymbolTable {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Program::new(Parser::new(tokens).parse().unwrap());
        let builtins: HashMap<String, Value<DefaultPineOutput>> = HashMap::new();
        analyze_with_symbols(&program, &builtins).1
    }

    #[test]
    fn records_kinds_locations_and_params() {
        // `sum` is a function (line 3), its parameters live in a child scope,
        // and `total` is a global variable (line 4).
        let table = table("//@version=5\nindicator(\"t\")\nsum(a, b) => a + b\ntotal = sum(1, 2)\n");

        let sum = table.resolve(SymbolTable::GLOBAL, "sum").unwrap();
        assert_eq!(sum.kind, SymbolKind::Function);
        assert_eq!(sum.params, vec!["a", "b"]);
        assert_eq!(sum.decl, Some((3, 1)));

        let total = table.resolve(SymbolTable::GLOBAL, "total").unwrap();
        assert_eq!(total.kind, SymbolKind::Var);
        assert_eq!(total.decl, Some((4, 1)));

        // The parameters are not visible at global scope, only inside the body.
        assert!(table.resolve(SymbolTable::GLOBAL, "a").is_none());
        assert!(table.symbols().iter().any(|s| s.name == "a"));
    }

    #[test]
    fn records_type_members_and_lookups() {
        let table = table(
            "//@version=5\nindicator(\"t\")\ntype Point\n    float x\n    float y = 0.0\np = Point.new()\n",
        );

        let point = table.resolve(SymbolTable::GLOBAL, "Point").unwrap();
        assert_eq!(point.kind, SymbolKind::Type);

        // Members are reached through the owner, not by bare name.
        assert!(table.resolve(SymbolTable::GLOBAL, "x").is_none());
        let point_id = table.symbols().iter().position(|s| s.name == "Point").unwrap();
        let members: Vec<_> = table.members_of(point_id).map(|m| m.name.as_str()).collect();
        assert_eq!(members, vec!["x", "y"]);

        // `find_declaration_at` maps a declaration position back to its symbol.
        let x = table.find_declaration_at(4, 11).unwrap();
        assert_eq!(x.name, "x");
        assert_eq!(x.type_annotation.as_deref(), Some("float"));
    }

    #[test]
    fn records_uses_for_references_and_hover() {
        // `x` is declared on line 3 and used twice on line 4 (`y = x + x`).
        let table = table("//@version=5\nindicator(\"t\")\nx = 1\ny = x + x\n");
        let x_id = table.symbols().iter().position(|s| s.name == "x").unwrap();

        // find-references: both uses, not the declaration.
        let mut refs: Vec<_> = table.references(x_id).collect();
        refs.sort();
        assert_eq!(refs, vec![(4, 5), (4, 9)]);

        // hover / go-to-definition maps a cursor on a use back to the symbol…
        assert_eq!(table.symbol_at(4, 5), Some(x_id));
        assert_eq!(table.symbol_at(4, 9), Some(x_id));
        // …and a cursor on the declaration resolves to it too.
        assert_eq!(table.symbol_at(3, 1), Some(x_id));

        // A builtin use (`indicator`) is not a user symbol, so no occurrence.
        assert!(table.symbol_at(2, 1).is_none());
    }
}
