use serde::{Deserialize, Serialize};

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
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Argument>,
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
        params: Vec<String>,
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
        else_expr: Option<Box<Expr>>, // None means return na if no branch matches
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
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
        type_annotation: Option<String>,
        initializer: Option<Expr>,
        is_varip: bool, // true for varip, false for var
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
    },
    MethodDecl {
        name: String,
        params: Vec<MethodParam>,
        body: Vec<Stmt>,
    },
    EnumDecl {
        name: String,
        fields: Vec<EnumField>,
    },
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
    pub type_annotation: Option<String>, // e.g., "InfoLabel"
    pub name: String,
    pub default_value: Option<Expr>,
}

/// A field in a user-defined type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeField {
    pub name: String,
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
