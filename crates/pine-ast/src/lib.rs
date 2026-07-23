use serde::{Deserialize, Serialize};

pub mod visitor;
pub use visitor::{walk_block, walk_expr, walk_program, walk_stmt, Visitor};

// Helper function for serde to skip false values
fn is_false(b: &bool) -> bool {
    !b
}

// Helper function for serde to skip None values
fn skip_none<T>(opt: &Option<T>) -> bool {
    opt.is_none()
}

// Helper function for serde to skip unassigned call-site ids
fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

/// Source location (1-based line and column) attached to select AST nodes for
/// diagnostics.
///
/// `Loc` is intentionally transparent to equality and serialization: two nodes
/// that differ only in location compare **equal**, and the position is **never**
/// written to the serialized AST (the field carries `#[serde(skip)]`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Loc {
    pub line: u32,
    pub column: u32,
}

impl Loc {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    /// The tracked `(line, column)`, or `None` when unknown (line `0`).
    pub fn position(&self) -> Option<(u32, u32)> {
        (self.line != 0).then_some((self.line, self.column))
    }

    /// The tracked line, or `None` when unknown (line `0`).
    pub fn line(&self) -> Option<u32> {
        (self.line != 0).then_some(self.line)
    }
}

// Location must not participate in structural equality: an AST compared against
// a snapshot (which never stores a line) must still match.
impl PartialEq for Loc {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Type qualifier for variables and parameters
/// Hierarchy: const < input < simple < series (const is the weakest)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeQualifier {
    Const,
    Input,
    Simple,
    Series,
}

/// How a variable declaration behaves across bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VarKind {
    /// `x = expr` — the initializer is re-evaluated on every bar.
    #[default]
    Plain,
    /// `var x = expr` — the initializer runs once; the value persists across bars.
    Var,
    /// `varip x = expr` — like `Var`, but also updates intrabar in realtime.
    Varip,
}

impl VarKind {
    /// `var`/`varip`: initialize once and retain the value across bars.
    pub fn is_persistent(self) -> bool {
        !matches!(self, VarKind::Plain)
    }

    /// Used by serde to omit the field for plain declarations.
    fn is_plain(&self) -> bool {
        matches!(self, VarKind::Plain)
    }
}

/// Function argument - can be positional or named
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Argument {
    Positional(Expr),
    Named { name: String, value: Expr },
}

// AST nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        #[serde(skip)]
        loc: Loc,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_args: Vec<String>, // Type arguments like <int>, <float>
        args: Vec<Argument>,
        #[serde(default, skip_serializing_if = "is_zero_u32")]
        id: u32,
        #[serde(skip)]
        loc: Loc,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Function {
        params: Vec<FunctionParam>,
        body: Vec<Stmt>,
    },
    Array(Vec<Expr>),
    Switch {
        value: Box<Expr>,
        cases: Vec<(Expr, Expr)>, // (pattern, result)
    },
    IfExpr {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_if_branches: Vec<(Expr, Expr)>, // Vec of (condition, expression) for else if
        else_expr: Option<Box<Expr>>,        // None means return na if no branch matches
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Number(f64),
    String(String),
    Bool(bool),
    Na,               // PineScript's N/A value
    HexColor(String), // Hex color: #RRGGBB or #RRGGBBAA
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    VarDecl {
        name: String,
        #[serde(skip_serializing_if = "skip_none")]
        type_qualifier: Option<TypeQualifier>,
        type_annotation: Option<String>,
        initializer: Option<Expr>,
        #[serde(default, skip_serializing_if = "VarKind::is_plain")]
        var_kind: VarKind,
    },
    Assignment {
        target: Expr, // Can be Variable or MemberAccess
        value: Expr,
    },
    TupleAssignment {
        names: Vec<String>,
        value: Expr,
    },
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_if_branches: Vec<(Expr, Vec<Stmt>)>, // Vec of (condition, statements) for else if
        else_branch: Option<Vec<Stmt>>,
    },
    For {
        var_name: String,
        from: Expr,
        to: Expr,
        body: Vec<Stmt>,
    },
    ForIn {
        // For single item: for item in collection
        // For tuple: for [index, item] in collection
        index_var: Option<String>, // None for simple form, Some(name) for tuple form
        item_var: String,
        collection: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    TypeDecl {
        name: String,
        fields: Vec<TypeField>,
        #[serde(default, skip_serializing_if = "is_false")]
        export: bool,
    },
    MethodDecl {
        name: String,
        params: Vec<MethodParam>,
        body: Vec<Stmt>,
        #[serde(default, skip_serializing_if = "is_false")]
        export: bool,
    },
    EnumDecl {
        name: String,
        fields: Vec<EnumField>,
        #[serde(default, skip_serializing_if = "is_false")]
        export: bool,
    },
    FunctionDecl {
        name: String,
        params: Vec<FunctionParam>,
        body: Vec<Stmt>,
        #[serde(default, skip_serializing_if = "is_false")]
        export: bool,
    },
    Export {
        item: ExportItem,
    },
    Import {
        path: String,  // e.g., "userName/Point/1"
        alias: String, // e.g., "pt"
    },
}

/// An item that can be exported from a library
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportItem {
    Type(String),     // export type typename
    Function(String), // export functionname
}

/// A field in an enum declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumField {
    pub name: String,
    pub title: Option<String>, // Optional title for the enum field
}

/// A parameter in a method declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodParam {
    #[serde(skip_serializing_if = "skip_none")]
    pub type_qualifier: Option<TypeQualifier>,
    pub type_annotation: Option<String>, // e.g., "InfoLabel"
    pub name: String,
    pub default_value: Option<Expr>,
}

/// A parameter in a function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParam {
    #[serde(skip_serializing_if = "skip_none")]
    pub type_qualifier: Option<TypeQualifier>,
    #[serde(skip_serializing_if = "skip_none")]
    pub type_annotation: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "skip_none")]
    pub default_value: Option<Expr>,
}

/// A field in a user-defined type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeField {
    pub name: String,
    #[serde(skip_serializing_if = "skip_none")]
    pub type_qualifier: Option<TypeQualifier>,
    pub type_annotation: String,
    pub default_value: Option<Expr>,
}

/// A program is a collection of statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}
