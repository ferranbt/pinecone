//! Name-kind classification and the global-only builtin list.
//!
//! Scopes themselves live in [`SymbolTable`](crate::SymbolTable): the analyzer
//! resolves names against it directly, so there is no separate scope stack.

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

/// Functions Pine only permits at **global** scope (never inside `if`, loops, or
/// function bodies).
const GLOBAL_ONLY_FUNCTIONS: &[&str] = &[
    "plot",
    "plotshape",
    "plotchar",
    "plotcandle",
    "plotbar",
    "plotarrow",
    "fill",
];

/// May `name` only be called at global scope?
pub fn is_global_only(name: &str) -> bool {
    GLOBAL_ONLY_FUNCTIONS.contains(&name)
}
