use serde::{Deserialize, Serialize};

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

/// Type qualifier for variables and parameters
/// Hierarchy: const < input < simple < series (const is the weakest)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeQualifier {
    Const,
    Input,
    Simple,
    Series,
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
        /// Stable lexical call-site identity, assigned by `Program::new` in
        /// AST-walk order starting at 1 (0 = unassigned). Built-in functions
        /// that keep per-call-site state (e.g. crossover history) need this:
        /// with lazy `and`/`or` and untaken `if`/ternary branches, a call
        /// site may not execute on every bar, so execution-order counting
        /// cannot identify call sites.
        #[serde(default, skip_serializing_if = "is_zero_u32")]
        id: u32,
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
    pub fn new(mut statements: Vec<Stmt>) -> Self {
        let mut next_id: u32 = 1;
        for stmt in &mut statements {
            assign_call_ids_stmt(stmt, &mut next_id);
        }
        Self { statements }
    }
}

/// Assign stable lexical ids to every `Expr::Call` node, in AST-walk order.
/// See the `Expr::Call::id` field docs for why call sites need identity.
fn assign_call_ids_stmt(stmt: &mut Stmt, next_id: &mut u32) {
    let walk_body = |body: &mut Vec<Stmt>, next_id: &mut u32| {
        for s in body {
            assign_call_ids_stmt(s, next_id);
        }
    };
    match stmt {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(e) = initializer {
                assign_call_ids_expr(e, next_id);
            }
        }
        Stmt::Assignment { target, value } => {
            assign_call_ids_expr(target, next_id);
            assign_call_ids_expr(value, next_id);
        }
        Stmt::TupleAssignment { value, .. } => assign_call_ids_expr(value, next_id),
        Stmt::Expression(e) => assign_call_ids_expr(e, next_id),
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            assign_call_ids_expr(condition, next_id);
            walk_body(then_branch, next_id);
            for (c, b) in else_if_branches {
                assign_call_ids_expr(c, next_id);
                walk_body(b, next_id);
            }
            if let Some(b) = else_branch {
                walk_body(b, next_id);
            }
        }
        Stmt::For { from, to, body, .. } => {
            assign_call_ids_expr(from, next_id);
            assign_call_ids_expr(to, next_id);
            walk_body(body, next_id);
        }
        Stmt::ForIn {
            collection, body, ..
        } => {
            assign_call_ids_expr(collection, next_id);
            walk_body(body, next_id);
        }
        Stmt::While { condition, body } => {
            assign_call_ids_expr(condition, next_id);
            walk_body(body, next_id);
        }
        Stmt::TypeDecl { fields, .. } => {
            for f in fields {
                if let Some(e) = &mut f.default_value {
                    assign_call_ids_expr(e, next_id);
                }
            }
        }
        Stmt::MethodDecl { params, body, .. } => {
            for p in params {
                if let Some(e) = &mut p.default_value {
                    assign_call_ids_expr(e, next_id);
                }
            }
            walk_body(body, next_id);
        }
        Stmt::FunctionDecl { params, body, .. } => {
            for p in params {
                if let Some(e) = &mut p.default_value {
                    assign_call_ids_expr(e, next_id);
                }
            }
            walk_body(body, next_id);
        }
        Stmt::Break
        | Stmt::Continue
        | Stmt::EnumDecl { .. }
        | Stmt::Export { .. }
        | Stmt::Import { .. } => {}
    }
}

fn assign_call_ids_expr(expr: &mut Expr, next_id: &mut u32) {
    match expr {
        Expr::Literal(_) | Expr::Variable(_) => {}
        Expr::Binary { left, right, .. } => {
            assign_call_ids_expr(left, next_id);
            assign_call_ids_expr(right, next_id);
        }
        Expr::Unary { expr, .. } => assign_call_ids_expr(expr, next_id),
        Expr::Call {
            callee, args, id, ..
        } => {
            *id = *next_id;
            *next_id += 1;
            assign_call_ids_expr(callee, next_id);
            for arg in args {
                match arg {
                    Argument::Positional(e) => assign_call_ids_expr(e, next_id),
                    Argument::Named { value, .. } => assign_call_ids_expr(value, next_id),
                }
            }
        }
        Expr::Index { expr, index } => {
            assign_call_ids_expr(expr, next_id);
            assign_call_ids_expr(index, next_id);
        }
        Expr::MemberAccess { object, .. } => assign_call_ids_expr(object, next_id),
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            assign_call_ids_expr(condition, next_id);
            assign_call_ids_expr(then_expr, next_id);
            assign_call_ids_expr(else_expr, next_id);
        }
        Expr::Function { params, body } => {
            for p in params {
                if let Some(e) = &mut p.default_value {
                    assign_call_ids_expr(e, next_id);
                }
            }
            for s in body {
                assign_call_ids_stmt(s, next_id);
            }
        }
        Expr::Array(items) => {
            for e in items {
                assign_call_ids_expr(e, next_id);
            }
        }
        Expr::Switch { value, cases } => {
            assign_call_ids_expr(value, next_id);
            for (pat, res) in cases {
                assign_call_ids_expr(pat, next_id);
                assign_call_ids_expr(res, next_id);
            }
        }
        Expr::IfExpr {
            condition,
            then_expr,
            else_if_branches,
            else_expr,
        } => {
            assign_call_ids_expr(condition, next_id);
            assign_call_ids_expr(then_expr, next_id);
            for (c, e) in else_if_branches {
                assign_call_ids_expr(c, next_id);
                assign_call_ids_expr(e, next_id);
            }
            if let Some(e) = else_expr {
                assign_call_ids_expr(e, next_id);
            }
        }
    }
}
