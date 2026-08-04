//! A durable symbol table.
//!
//! Every declaration a script makes — its kind, where it is written, its
//! parameters or type — arranged in the scope tree they belong to. That is what
//! a tool queries: go-to-definition (a symbol's `decl`), hover (its kind +
//! signature), completion (the symbols visible in a scope, or the members of a
//! type).
//!
//! This is the analyzer's *only* scope structure: [`Analyzer`](crate::Analyzer)
//! both resolves names against it and records declarations into it as it walks.
//! Imported libraries are analyzed into the same table under their own
//! [`FileId`] and their own global (root) scope, so `alias.export` resolves
//! cross-file while names never leak between files.

use crate::scope::Namespace;
pub use crate::scope::SymbolKind;

/// A scope's index within a [`SymbolTable`]. `0` is always the main file's
/// global scope.
pub type ScopeId = usize;

/// A symbol's index within a [`SymbolTable`].
pub type SymbolId = usize;

/// A source file's index within a [`SymbolTable`]. `0` is always the main
/// script; imported libraries get the ids that follow.
pub type FileId = usize;

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
    pub file: FileId,
    pub decl: Option<(u32, u32)>,
    pub scope: ScopeId,
    pub params: Vec<String>,
    pub type_annotation: Option<String>,
    /// The type/enum this symbol is a member (field or enum case) of. A member
    /// is reached as `owner.member`, so it is not in any scope's resolution list.
    pub container: Option<SymbolId>,
    /// For a variable of a user-defined type, that type's symbol, so `v.field`
    /// resolves. Set from an annotation or a `Type.new()` initializer.
    pub type_ref: Option<SymbolId>,
    pub exported: bool,
    /// For an import alias, the imported library's global scope.
    pub module: Option<ScopeId>,
}

impl Symbol {
    pub(crate) fn new(
        name: &str,
        kind: SymbolKind,
        decl: Option<(u32, u32)>,
        scope: ScopeId,
    ) -> Self {
        Symbol {
            name: name.to_string(),
            kind,
            file: 0,
            decl,
            scope,
            params: Vec::new(),
            type_annotation: None,
            container: None,
            type_ref: None,
            exported: false,
            module: None,
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

    pub(crate) fn with_type_ref(mut self, type_ref: Option<SymbolId>) -> Self {
        self.type_ref = type_ref;
        self
    }
}

#[derive(Debug, Clone)]
struct ScopeData {
    parent: Option<ScopeId>,
    kind: ScopeKind,
    file: FileId,
    symbols: Vec<SymbolId>,
}

/// A use of a symbol — the index behind find-references / rename.
#[derive(Debug, Clone)]
struct Occurrence {
    file: FileId,
    line: u32,
    column: u32,
    symbol: SymbolId,
}

/// The scope tree of a program, the symbols each scope declares, every use of
/// those symbols, and the files they live in.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    scopes: Vec<ScopeData>,
    occurrences: Vec<Occurrence>,
    files: Vec<String>,
    file_roots: Vec<ScopeId>,
}

impl SymbolTable {
    /// The main file's global scope, always present.
    pub const GLOBAL: ScopeId = 0;
    /// The main file's id, always present.
    pub const MAIN: FileId = 0;

    pub(crate) fn new() -> Self {
        SymbolTable {
            symbols: Vec::new(),
            scopes: vec![ScopeData {
                parent: None,
                kind: ScopeKind::Global,
                file: Self::MAIN,
                symbols: Vec::new(),
            }],
            occurrences: Vec::new(),
            files: vec![String::new()],
            file_roots: vec![Self::GLOBAL],
        }
    }

    pub(crate) fn add_file(&mut self, path: &str) -> (FileId, ScopeId) {
        let file = self.files.len();
        self.files.push(path.to_string());
        let root = self.scopes.len();
        self.scopes.push(ScopeData {
            parent: None,
            kind: ScopeKind::Global,
            file,
            symbols: Vec::new(),
        });
        self.file_roots.push(root);
        (file, root)
    }

    pub(crate) fn file_by_path(&self, path: &str) -> Option<FileId> {
        self.files.iter().position(|p| p == path)
    }

    pub fn file_path(&self, file: FileId) -> &str {
        &self.files[file]
    }

    pub fn file_root(&self, file: FileId) -> ScopeId {
        self.file_roots[file]
    }

    pub fn files(&self) -> impl Iterator<Item = (FileId, &str)> {
        self.files
            .iter()
            .enumerate()
            .map(|(id, path)| (id, path.as_str()))
    }

    pub fn scope_file(&self, scope: ScopeId) -> FileId {
        self.scopes[scope].file
    }

    pub(crate) fn open_scope(&mut self, parent: ScopeId, kind: ScopeKind) -> ScopeId {
        let id = self.scopes.len();
        let file = self.scopes[parent].file;
        self.scopes.push(ScopeData {
            parent: Some(parent),
            kind,
            file,
            symbols: Vec::new(),
        });
        id
    }

    pub(crate) fn set_module(&mut self, id: SymbolId, scope: ScopeId) {
        self.symbols[id].module = Some(scope);
    }

    pub(crate) fn mark_exported(&mut self, id: SymbolId) {
        self.symbols[id].exported = true;
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
    /// — a redeclaration in the same scope.
    pub(crate) fn declared_locally(&self, scope: ScopeId, name: &str) -> bool {
        self.scopes[scope]
            .symbols
            .iter()
            .any(|&id| self.symbols[id].name == name)
    }

    /// Like [`Self::declared_locally`], but only counts a symbol in the same
    /// namespace — so a type and a value may share a name.
    pub(crate) fn declared_locally_in(
        &self,
        scope: ScopeId,
        name: &str,
        namespace: Namespace,
    ) -> bool {
        self.scopes[scope].symbols.iter().any(|&id| {
            let symbol = &self.symbols[id];
            symbol.name == name && symbol.kind.namespace() == namespace
        })
    }

    /// Record that the name at `(file, pos)` refers to `symbol` — one entry in
    /// the occurrence index. `None` positions (unknown) are dropped.
    pub(crate) fn record_use(&mut self, file: FileId, pos: Option<(u32, u32)>, symbol: SymbolId) {
        if let Some((line, column)) = pos {
            self.occurrences.push(Occurrence {
                file,
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
        let file = self.symbols[owner].file;
        let mut member = Symbol::new(name, SymbolKind::Var, decl, scope);
        member.file = file;
        member.type_annotation = type_annotation;
        member.container = Some(owner);
        self.symbols.push(member);
    }

    // --- Queries ---

    /// Every symbol declared anywhere, in declaration order.
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// The kind of a scope.
    pub fn scope_kind(&self, scope: ScopeId) -> ScopeKind {
        self.scopes[scope].kind
    }

    /// The symbols reachable by bare name directly in `scope`.
    pub fn symbols_in(&self, scope: ScopeId) -> impl Iterator<Item = &Symbol> {
        self.scopes[scope]
            .symbols
            .iter()
            .map(|&id| &self.symbols[id])
    }

    /// The members (fields, enum cases) of a type or enum symbol — the
    /// candidates for completion after `owner.`.
    pub fn members_of(&self, owner: SymbolId) -> impl Iterator<Item = &Symbol> {
        self.symbols
            .iter()
            .filter(move |s| s.container == Some(owner))
    }

    /// The member of `owner` named `name`, if any — resolves `owner.name`.
    pub fn member_id(&self, owner: SymbolId, name: &str) -> Option<SymbolId> {
        self.symbols
            .iter()
            .position(|s| s.container == Some(owner) && s.name == name)
    }

    /// An `export`ed symbol named `name` declared directly in `scope` — how a
    /// member access resolves against an imported library's global scope.
    pub fn exported_id(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.scopes[scope]
            .symbols
            .iter()
            .rev()
            .find(|&&id| self.symbols[id].exported && self.symbols[id].name == name)
            .copied()
    }

    /// A symbol's declaration location as `(file, line, column)` — go-to-def,
    /// including into an imported library file.
    pub fn declaration_location(&self, id: SymbolId) -> Option<(FileId, u32, u32)> {
        let symbol = &self.symbols[id];
        symbol.decl.map(|(line, col)| (symbol.file, line, col))
    }

    /// The symbol at index `id`.
    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id]
    }

    /// Resolve `name` from `scope` outward, innermost first.
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

    pub fn references(&self, symbol: SymbolId) -> impl Iterator<Item = (FileId, u32, u32)> + '_ {
        self.occurrences
            .iter()
            .filter(move |o| o.symbol == symbol)
            .map(|o| (o.file, o.line, o.column))
    }

    pub fn occurrences_in_file(
        &self,
        file: FileId,
    ) -> impl Iterator<Item = (u32, u32, SymbolId)> + '_ {
        self.occurrences
            .iter()
            .filter(move |o| o.file == file)
            .map(|o| (o.line, o.column, o.symbol))
    }

    /// The symbol referenced or declared at `(file, line, column)` — hover and
    /// go-to-definition, whether the cursor sits on a use or the declaration.
    pub fn symbol_at(&self, file: FileId, line: u32, column: u32) -> Option<SymbolId> {
        if let Some(occ) = self
            .occurrences
            .iter()
            .find(|o| o.file == file && o.line == line && o.column == column)
        {
            return Some(occ.symbol);
        }
        self.symbols
            .iter()
            .position(|s| s.file == file && s.decl == Some((line, column)))
    }

    /// The symbol *declared* at `(file, line, column)`, if any.
    pub fn find_declaration_at(&self, file: FileId, line: u32, column: u32) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|s| s.file == file && s.decl == Some((line, column)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{analyze_with_symbols, Diagnostic, SymbolKind, SymbolTable};
    use pine_core::{DefaultPineOutput, FileResolver};
    use pine_interpreter::Value;
    use pine_parser::Parser;
    use std::collections::HashMap;

    const MAIN: crate::FileId = SymbolTable::MAIN;

    fn table(source: &str) -> SymbolTable {
        analyze(source, None).1
    }

    /// Analyze `source` (the main file) with an optional in-memory loader, and
    /// return diagnostics + the symbol table.
    fn analyze(source: &str, loader: Option<&FileResolver>) -> (Vec<Diagnostic>, SymbolTable) {
        let program = Parser::parse_source(source).unwrap();
        let builtins: HashMap<String, Value<DefaultPineOutput>> = HashMap::new();
        analyze_with_symbols(
            &program,
            &builtins,
            loader.map(|l| l as &dyn crate::LibraryLoader),
        )
    }

    /// An in-memory loader over `(path, source)` library files.
    fn libs(files: &[(&str, &str)]) -> FileResolver {
        let mut resolver = FileResolver::new();
        for (path, source) in files {
            resolver.add(path, source);
        }
        resolver
    }

    #[test]
    fn records_kinds_locations_and_params() {
        // `sum` is a function (line 3), its parameters live in a child scope,
        // and `total` is a global variable (line 4).
        let table =
            table("//@version=5\nindicator(\"t\")\nsum(a, b) => a + b\ntotal = sum(1, 2)\n");

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
        let point_id = table
            .symbols()
            .iter()
            .position(|s| s.name == "Point")
            .unwrap();
        let members: Vec<_> = table
            .members_of(point_id)
            .map(|m| m.name.as_str())
            .collect();
        assert_eq!(members, vec!["x", "y"]);

        // `find_declaration_at` maps a declaration position back to its symbol.
        let x = table.find_declaration_at(MAIN, 4, 11).unwrap();
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
        assert_eq!(refs, vec![(MAIN, 4, 5), (MAIN, 4, 9)]);

        // hover / go-to-definition maps a cursor on a use back to the symbol…
        assert_eq!(table.symbol_at(MAIN, 4, 5), Some(x_id));
        assert_eq!(table.symbol_at(MAIN, 4, 9), Some(x_id));
        // …and a cursor on the declaration resolves to it too.
        assert_eq!(table.symbol_at(MAIN, 3, 1), Some(x_id));

        // A builtin use (`indicator`) is not a user symbol, so no occurrence.
        assert!(table.symbol_at(MAIN, 2, 1).is_none());
    }

    #[test]
    fn resolves_members_of_a_typed_variable() {
        // `p` is a Point (via the `.new()` constructor); `p.x` on line 6 refers
        // to the field `x` declared on line 4.
        let table = table(
            "//@version=5\nindicator(\"t\")\ntype Point\n    float x\n    float y\np = Point.new()\nv = p.x\n",
        );

        let point_id = table
            .symbols()
            .iter()
            .position(|s| s.name == "Point")
            .unwrap();
        let x_id = table.member_id(point_id, "x").unwrap();

        // The `x` in `p.x` (line 7) resolves to the field, not a bare name.
        assert!(table.resolve(SymbolTable::GLOBAL, "x").is_none());
        let refs: Vec<_> = table.references(x_id).collect();
        assert_eq!(refs, vec![(MAIN, 7, 7)]);
        assert_eq!(table.symbol_at(MAIN, 7, 7), Some(x_id));
    }

    #[test]
    fn resolves_enum_cases() {
        // `Signal.buy` refers to the enum case declared on line 4.
        let table = table(
            "//@version=5\nindicator(\"t\")\nenum Signal\n    buy\n    sell\ns = Signal.buy\n",
        );

        let signal_id = table
            .symbols()
            .iter()
            .position(|s| s.name == "Signal")
            .unwrap();
        let buy_id = table.member_id(signal_id, "buy").unwrap();

        assert_eq!(table.symbol_at(MAIN, 6, 12), Some(buy_id));
    }

    #[test]
    fn imports_record_the_alias_but_not_its_members_without_a_loader() {
        // Without a loader, `import foo/bar/1 as lib` then `x = lib.calc(1)`.
        let table =
            table("//@version=5\nindicator(\"t\")\nimport foo/bar/1 as lib\nx = lib.calc(1)\n");

        // The alias is a symbol, so go-to-definition / find-references on `lib`
        // work: it is declared on line 3 and used once (line 4, `lib.calc`).
        let lib = table.resolve(SymbolTable::GLOBAL, "lib").unwrap();
        assert_eq!(lib.kind, SymbolKind::Import);
        let lib_id = table
            .symbols()
            .iter()
            .position(|s| s.name == "lib")
            .unwrap();
        assert_eq!(
            table.references(lib_id).collect::<Vec<_>>(),
            vec![(MAIN, 4, 5)]
        );

        // The member `calc` is a library export — with no loader the library is
        // not analyzed, so it does not resolve. No occurrence, no guess.
        assert_eq!(table.symbol_at(MAIN, 4, 9), None);
    }

    #[test]
    fn resolves_a_library_export_across_files() {
        let loader = libs(&[("lib", "//@version=5\nexport add(a, b) => a + b\n")]);
        let (_diags, table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport lib as l\nx = l.add(1, 2)\n",
            Some(&loader),
        );

        // `l.add` resolves to the library's exported function.
        let add_id = table
            .symbols()
            .iter()
            .position(|s| s.name == "add" && s.exported)
            .unwrap();
        // Its declaration is in the library file, go-to-def lands there.
        let (file, _, _) = table.declaration_location(add_id).unwrap();
        assert_ne!(file, MAIN);
        assert_eq!(table.file_path(file), "lib");
        // Find-references reports the single main-file use, which maps back.
        let refs: Vec<_> = table.references(add_id).collect();
        assert_eq!(refs.len(), 1);
        let (rf, rl, rc) = refs[0];
        assert_eq!(rf, MAIN);
        assert_eq!(table.symbol_at(rf, rl, rc), Some(add_id));
    }

    #[test]
    fn resolves_a_field_of_a_variable_typed_from_a_library() {
        let loader = libs(&[(
            "geo",
            "//@version=5\nexport type Point\n    float x\n    float y\n",
        )]);
        let (_diags, table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport geo as g\np = g.Point.new()\nv = p.x\n",
            Some(&loader),
        );

        let point_id = table
            .symbols()
            .iter()
            .position(|s| s.name == "Point" && s.exported)
            .unwrap();
        let x_id = table.member_id(point_id, "x").unwrap();
        // `p.x` (main) resolves to the library field — one use, in the main file.
        let refs: Vec<_> = table.references(x_id).collect();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0, MAIN);
    }

    #[test]
    fn resolves_an_enum_case_from_a_library() {
        let loader = libs(&[(
            "sig",
            "//@version=5\nexport enum Direction\n    up\n    down\n",
        )]);
        let (_diags, table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport sig as s\nd = s.Direction.up\n",
            Some(&loader),
        );

        let dir_id = table
            .symbols()
            .iter()
            .position(|s| s.name == "Direction" && s.exported)
            .unwrap();
        let up_id = table.member_id(dir_id, "up").unwrap();
        assert_eq!(table.references(up_id).count(), 1);
    }

    #[test]
    fn a_library_use_of_its_own_import_resolves() {
        // main -> a -> b. `a`'s exported `compute` body calls `bb.helper()`
        // (b's export, reached through a's own import). That inner use resolves
        // to b's `helper`, recorded in file `a`.
        let loader = libs(&[
            ("b", "//@version=5\nexport helper() => 42\n"),
            (
                "a",
                "//@version=5\nimport b as bb\nexport compute() => bb.helper()\n",
            ),
        ]);
        let (_diags, table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport a as x\ny = x.compute()\n",
            Some(&loader),
        );

        // main's `x.compute` resolves to a's export.
        let compute = table
            .symbols()
            .iter()
            .position(|s| s.name == "compute")
            .unwrap();
        assert_eq!(table.references(compute).count(), 1);

        // b's `helper` is used inside library `a`, not the main file.
        let helper = table
            .symbols()
            .iter()
            .position(|s| s.name == "helper")
            .unwrap();
        let (decl_file, _, _) = table.declaration_location(helper).unwrap();
        let refs: Vec<_> = table.references(helper).collect();
        assert_eq!(refs.len(), 1);
        let (use_file, _, _) = refs[0];
        assert_ne!(use_file, MAIN);
        assert_eq!(table.file_path(use_file), "a");
        assert_eq!(table.file_path(decl_file), "b");
    }

    #[test]
    fn an_unknown_library_member_resolves_to_nothing() {
        let loader = libs(&[("lib", "//@version=5\nexport add(a, b) => a + b\n")]);
        let (diags, table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport lib as l\nx = l.nope\n",
            Some(&loader),
        );
        // `nope` is not an export: no occurrence, and no invented diagnostic.
        assert_eq!(table.symbol_at(MAIN, 4, 7), None);
        assert!(diags.iter().all(|d| d.rule != "unknown-member"));
    }

    #[test]
    fn library_diagnostics_are_tagged_with_the_file() {
        let loader = libs(&[("broken", "//@version=5\nexport bad() => undeclared_thing\n")]);
        let (diags, _table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport broken as b\n",
            Some(&loader),
        );
        let lib_err = diags
            .iter()
            .find(|d| d.rule == "undeclared-variable")
            .expect("library's undeclared-variable error is surfaced");
        assert_eq!(lib_err.file.as_deref(), Some("broken"));
        assert!(format!("{lib_err}").contains("broken"));

        // The same error in the main script stays untagged and byte-identical.
        let (main_diags, _) = analyze(
            "//@version=5\nindicator(\"t\")\ny = undeclared_thing\n",
            None,
        );
        let main_err = main_diags
            .iter()
            .find(|d| d.rule == "undeclared-variable")
            .unwrap();
        assert_eq!(main_err.file, None);
        assert_eq!(
            format!("{main_err}"),
            "error [undeclared-variable]: undeclared variable `undeclared_thing`"
        );
    }

    #[test]
    fn import_cycles_terminate() {
        // Mutual: a imports b, b imports a. Terminates via the cache; a's export
        // still resolves from the main file.
        let loader = libs(&[
            ("a", "//@version=5\nimport b as bb\nexport fa() => 1\n"),
            ("b", "//@version=5\nimport a as aa\nexport fb() => 2\n"),
        ]);
        let (_diags, table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport a as x\ny = x.fa()\n",
            Some(&loader),
        );
        let fa_id = table.symbols().iter().position(|s| s.name == "fa").unwrap();
        assert_eq!(table.references(fa_id).count(), 1);

        // Self-import terminates too.
        let loader = libs(&[(
            "selfimp",
            "//@version=5\nimport selfimp as me\nexport g() => 1\n",
        )]);
        let (_diags, table) = analyze(
            "//@version=5\nindicator(\"t\")\nimport selfimp as s\ny = s.g()\n",
            Some(&loader),
        );
        let g_id = table.symbols().iter().position(|s| s.name == "g").unwrap();
        assert_eq!(table.references(g_id).count(), 1);
    }

    #[test]
    fn a_use_binds_to_the_innermost_shadowing_declaration() {
        // A global `x` (line 3) and a parameter `x` (line 4) that shadows it. The
        // use of `x` inside the function body must bind to the parameter, not the
        // global.
        let table = table("//@version=5\nindicator(\"t\")\nx = 1\nf(x) => x + 1\ny = x + 1\n");

        let symbols = table.symbols();
        // Two distinct `x` symbols: the global var and the parameter.
        let global_x = symbols
            .iter()
            .position(|s| s.name == "x" && s.scope == SymbolTable::GLOBAL)
            .unwrap();
        let param_x = symbols
            .iter()
            .position(|s| s.name == "x" && s.scope != SymbolTable::GLOBAL)
            .unwrap();
        assert_ne!(global_x, param_x);

        // The `x` in the body (line 4, `f(x) => x + 1`) binds to the parameter;
        // the `x` in `y = x + 1` (line 5) binds to the global.
        assert_eq!(table.symbol_at(MAIN, 4, 9), Some(param_x));
        assert_eq!(table.symbol_at(MAIN, 5, 5), Some(global_x));

        // Find-references keeps them apart.
        assert_eq!(
            table.references(param_x).collect::<Vec<_>>(),
            vec![(MAIN, 4, 9)]
        );
        assert_eq!(
            table.references(global_x).collect::<Vec<_>>(),
            vec![(MAIN, 5, 5)]
        );
    }
}
