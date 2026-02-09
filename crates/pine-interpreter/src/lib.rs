use pine_ast::{Argument, BinOp, Expr, Literal, MethodParam, Program, Stmt, TypeField, UnOp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

/// Trait for loading external libraries
pub trait LibraryLoader {
    /// Load a library from the given path and return its Program AST
    fn load_library(&self, path: &str) -> Result<Program, String>;
}

/// Trait for providing historical data for series
pub trait HistoricalDataProvider {
    /// Get historical value for a series at a given offset (0 = current, 1 = previous bar, etc.)
    fn get_historical(&self, series_id: &str, offset: usize) -> Option<Value>;
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Variable '{0}' not found")]
    UndefinedVariable(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    #[error("Cannot iterate: from={0}, to={1}")]
    InvalidForLoop(f64, f64),

    #[error("Break statement outside of loop")]
    BreakOutsideLoop,

    #[error("Continue statement outside of loop")]
    ContinueOutsideLoop,

    #[error("Library error: {0}")]
    LibraryError(String),

    #[error("Cannot reassign const variable '{0}'")]
    ConstReassignment(String),
}

/// Control flow signals for loops
#[derive(Debug, Clone, PartialEq)]
enum LoopControl {
    None,
    Break,
    Continue,
}

/// Variable storage with const qualifier tracking
#[derive(Clone)]
struct Variable {
    value: Value,
    is_const: bool,
}

/// Represents a single bar/candle of market data
#[derive(Debug, Clone, Default)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Represents a time series with an identifier and current value
#[derive(Clone, Debug)]
pub struct Series {
    pub id: String,
    pub current: Box<Value>,
}

/// Value types in the interpreter
#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Na,                             // PineScript's N/A value
    Array(Rc<RefCell<Vec<Value>>>), // Mutable shared array reference
    Series(Series),                 // Time series - ID and current value only
    Object {
        type_name: String, // The type name of this object (e.g., "InfoLabel")
        fields: Rc<RefCell<HashMap<String, Value>>>, // Dictionary/Object with string keys
    },
    Function {
        params: Vec<pine_ast::FunctionParam>,
        body: Vec<Stmt>,
    },
    BuiltinFunction(BuiltinFn), // Builtin function pointer
    Type {
        name: String,
        fields: Vec<TypeField>,
    }, // User-defined type
    Enum {
        enum_name: String,  // The enum type name (e.g., "Signal")
        field_name: String, // The specific field/member name (e.g., "buy")
        title: String,      // The title of this enum member
    }, // Enum member value
    Color {
        r: u8, // Red component (0-255)
        g: u8, // Green component (0-255)
        b: u8, // Blue component (0-255)
        t: u8, // Transparency (0-100)
    }, // Color value
    Matrix {
        element_type: String, // Type of elements: "int", "float", "string", "bool"
        data: Rc<RefCell<Vec<Vec<Value>>>>, // 2D matrix - mutable shared reference to rows of columns
    },
}

impl Value {
    pub fn new_color(r: u8, g: u8, b: u8, t: u8) -> Value {
        Value::Color { r, g, b, t }
    }
}

// Manual Debug impl since function pointers don't implement Debug
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({:?})", n),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Bool(b) => write!(f, "Bool({:?})", b),
            Value::Na => write!(f, "Na"),
            Value::Array(a) => write!(f, "Array({:?})", a),
            Value::Series(s) => write!(f, "Series({:?})", s),
            Value::Object { type_name, fields } => write!(f, "Object({}:{:?})", type_name, fields),
            Value::Function { params, .. } => write!(f, "Function({} params)", params.len()),
            Value::BuiltinFunction(_) => write!(f, "BuiltinFunction"),
            Value::Type { name, .. } => write!(f, "Type({})", name),
            Value::Enum {
                enum_name,
                field_name,
                ..
            } => write!(f, "Enum({}::{})", enum_name, field_name),
            Value::Color { r, g, b, t } => write!(f, "Color(rgba({}, {}, {}, {}))", r, g, b, t),
            Value::Matrix { element_type, data } => {
                write!(f, "Matrix<{}>({:?})", element_type, data)
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Na, Value::Na) => true,
            // Arrays compare by reference (Rc pointer equality)
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b),
            // Series compare by ID and current value
            (Value::Series(a), Value::Series(b)) => a.id == b.id && *a.current == *b.current,
            (Value::Object { fields: a, .. }, Value::Object { fields: b, .. }) => Rc::ptr_eq(a, b),
            // Functions never equal (can't compare closures or function pointers)
            (Value::Function { .. }, Value::Function { .. }) => false,
            (Value::BuiltinFunction(_), Value::BuiltinFunction(_)) => false,
            // Types compare by name
            (Value::Type { name: a, .. }, Value::Type { name: b, .. }) => a == b,
            // Enums compare by enum name and field name (ensuring type safety)
            (
                Value::Enum {
                    enum_name: a_enum,
                    field_name: a_field,
                    ..
                },
                Value::Enum {
                    enum_name: b_enum,
                    field_name: b_field,
                    ..
                },
            ) => a_enum == b_enum && a_field == b_field,
            // Colors compare by all components
            (
                Value::Color {
                    r: r1,
                    g: g1,
                    b: b1,
                    t: t1,
                },
                Value::Color {
                    r: r2,
                    g: g2,
                    b: b2,
                    t: t2,
                },
            ) => r1 == r2 && g1 == g2 && b1 == b2 && t1 == t2,
            // Matrices compare by reference (Rc pointer equality)
            (Value::Matrix { data: a, .. }, Value::Matrix { data: b, .. }) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// Evaluated function argument
#[derive(Debug, Clone)]
pub enum EvaluatedArg {
    Positional(Value),
    Named { name: String, value: Value },
}

/// Container for function call arguments including type parameters
#[derive(Debug, Clone)]
pub struct FunctionCallArgs {
    pub type_args: Vec<String>,
    pub args: Vec<EvaluatedArg>,
}

impl FunctionCallArgs {
    pub fn new(type_args: Vec<String>, args: Vec<EvaluatedArg>) -> Self {
        Self { type_args, args }
    }

    pub fn without_types(args: Vec<EvaluatedArg>) -> Self {
        Self {
            type_args: vec![],
            args,
        }
    }
}

/// Type signature for builtin functions (can be function pointers or closures)
pub type BuiltinFn = Rc<dyn Fn(&mut Interpreter, FunctionCallArgs) -> Result<Value, RuntimeError>>;

impl Value {
    pub fn as_number(&self) -> Result<f64, RuntimeError> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected number, got {:?}",
                self
            ))),
        }
    }

    pub fn as_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::Number(n) => Ok(*n != 0.0),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected bool, got {:?}",
                self
            ))),
        }
    }

    pub fn as_string(&self) -> Result<String, RuntimeError> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Na => Ok("na".to_string()),
            _ => Err(RuntimeError::TypeError(format!(
                "Cannot convert {:?} to string",
                self
            ))),
        }
    }

    pub fn as_array(&self) -> Result<&Rc<RefCell<Vec<Value>>>, RuntimeError> {
        match self {
            Value::Array(arr) => Ok(arr),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected array, got {:?}",
                self
            ))),
        }
    }

    pub fn as_color(&self) -> Result<(u8, u8, u8, u8), RuntimeError> {
        match self {
            Value::Color { r, g, b, t } => Ok((*r, *g, *b, *t)),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected color, got {:?}",
                self
            ))),
        }
    }
}

/// Method definition stored in the interpreter
#[derive(Clone)]
struct MethodDef {
    type_name: String, // The type this method belongs to (from first param's type annotation)
    params: Vec<pine_ast::MethodParam>,
    body: Vec<Stmt>,
}

/// The interpreter executes a program with a given bar
pub struct Interpreter {
    /// Local variables in the current scope
    variables: HashMap<String, Variable>,
    /// Builtin function registry
    builtins: HashMap<String, BuiltinFn>,
    /// Method registry (method_name -> Vec<MethodDef>) - can have multiple methods with same name for different types
    methods: HashMap<String, Vec<MethodDef>>,
    /// Library loader for importing external libraries
    library_loader: Option<Box<dyn LibraryLoader>>,
    /// Historical data provider for series lookback
    pub historical_provider: Option<Box<dyn HistoricalDataProvider>>,
    /// Exported items from this module (for library mode)
    exports: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            methods: HashMap::new(),
            builtins: HashMap::new(),
            library_loader: None,
            historical_provider: None,
            exports: HashMap::new(),
        }
    }

    /// Create interpreter with custom builtins
    pub fn with_builtins(builtins: HashMap<String, BuiltinFn>) -> Self {
        Self {
            variables: HashMap::new(),
            methods: HashMap::new(),
            builtins,
            library_loader: None,
            historical_provider: None,
            exports: HashMap::new(),
        }
    }

    /// Create interpreter with a library loader
    pub fn with_loader(loader: Box<dyn LibraryLoader>) -> Self {
        Self {
            variables: HashMap::new(),
            methods: HashMap::new(),
            builtins: HashMap::new(),
            library_loader: Some(loader),
            historical_provider: None,
            exports: HashMap::new(),
        }
    }

    /// Create interpreter with custom builtins and library loader
    pub fn with_builtins_and_loader(
        builtins: HashMap<String, BuiltinFn>,
        loader: Box<dyn LibraryLoader>,
    ) -> Self {
        Self {
            variables: HashMap::new(),
            methods: HashMap::new(),
            builtins,
            library_loader: Some(loader),
            historical_provider: None,
            exports: HashMap::new(),
        }
    }

    /// Set the historical data provider
    pub fn set_historical_provider(&mut self, provider: Box<dyn HistoricalDataProvider>) {
        self.historical_provider = Some(provider);
    }

    /// Set the library loader
    pub fn set_library_loader(&mut self, library_loader: Box<dyn LibraryLoader>) {
        self.library_loader = Some(library_loader);
    }

    /// Get the exported items from this interpreter (for library mode)
    pub fn exports(&self) -> &HashMap<String, Value> {
        &self.exports
    }

    /// Execute a program with a single bar
    pub fn execute(&mut self, program: &Program) -> Result<(), RuntimeError> {
        for stmt in &program.statements {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name).map(|var| &var.value)
    }

    /// Set a variable value (useful for loading objects and test setup)
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(
            name.to_string(),
            Variable {
                value,
                is_const: false,
            },
        );
    }

    /// Set a const variable (cannot be reassigned)
    pub fn set_const_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(
            name.to_string(),
            Variable {
                value,
                is_const: true,
            },
        );
    }

    /// Helper to get series values as a Vec<f64> for the given length
    /// Returns current value + historical values up to length-1
    pub fn get_series_values(
        &self,
        source: &Value,
        length: usize,
    ) -> Result<Vec<f64>, RuntimeError> {
        let mut values = Vec::new();

        match source {
            Value::Series(series) => {
                // Get current value
                if let Value::Number(n) = *series.current {
                    values.push(n);
                } else {
                    return Err(RuntimeError::TypeError(
                        "Series must contain numbers".to_string(),
                    ));
                }

                // Get historical values
                if let Some(provider) = &self.historical_provider {
                    for i in 1..length {
                        if let Some(Value::Number(n)) = provider.get_historical(&series.id, i) {
                            values.push(n);
                        } else {
                            break;
                        }
                    }
                }
            }
            Value::Number(n) => {
                values.push(*n);
            }
            _ => {
                return Err(RuntimeError::TypeError(
                    "source must be a number or series".to_string(),
                ));
            }
        }

        Ok(values)
    }

    /// Helper to evaluate arguments and validate positional-before-named rule
    fn evaluate_arguments(&mut self, args: &[Argument]) -> Result<Vec<EvaluatedArg>, RuntimeError> {
        let mut evaluated_args = Vec::new();
        let mut seen_named = false;

        for arg in args {
            match arg {
                Argument::Positional(expr) => {
                    if seen_named {
                        return Err(RuntimeError::TypeError(
                            "Positional arguments cannot follow named arguments".to_string(),
                        ));
                    }
                    let value = self.eval_expr(expr)?;
                    evaluated_args.push(EvaluatedArg::Positional(value));
                }
                Argument::Named { name, value: expr } => {
                    seen_named = true;
                    let value = self.eval_expr(expr)?;
                    evaluated_args.push(EvaluatedArg::Named {
                        name: name.clone(),
                        value,
                    });
                }
            }
        }

        Ok(evaluated_args)
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Stmt::VarDecl {
                name,
                type_qualifier,
                type_annotation: _,
                initializer,
                is_varip: _, // TODO: implement varip behavior (requires stateful execution)
            } => {
                let value = if let Some(init_expr) = initializer {
                    self.eval_expr(init_expr)?
                } else {
                    Value::Na
                };
                let is_const = matches!(type_qualifier, Some(pine_ast::TypeQualifier::Const));
                self.variables
                    .insert(name.clone(), Variable { value, is_const });
                Ok(None)
            }

            Stmt::Assignment { target, value } => {
                let val = self.eval_expr(value)?;

                match target {
                    Expr::Variable(name) => {
                        // Check if variable is const
                        if let Some(var) = self.variables.get(name) {
                            if var.is_const {
                                return Err(RuntimeError::ConstReassignment(name.clone()));
                            }
                        }

                        self.variables.insert(
                            name.clone(),
                            Variable {
                                value: val,
                                is_const: false,
                            },
                        );
                        Ok(None)
                    }
                    Expr::MemberAccess { object, member } => {
                        // Check if we're trying to modify a member of a const variable
                        if let Expr::Variable(var_name) = object.as_ref() {
                            if let Some(var) = self.variables.get(var_name) {
                                if var.is_const {
                                    return Err(RuntimeError::ConstReassignment(format!(
                                        "{}.{}",
                                        var_name, member
                                    )));
                                }
                            }
                        }

                        // Get the object
                        let obj_value = self.eval_expr(object)?;

                        if let Value::Object { fields, .. } = obj_value {
                            let mut obj = fields.borrow_mut();
                            obj.insert(member.clone(), val);
                            Ok(None)
                        } else {
                            Err(RuntimeError::TypeError(
                                "Cannot assign to member of non-object value".to_string(),
                            ))
                        }
                    }
                    _ => Err(RuntimeError::TypeError(
                        "Invalid assignment target".to_string(),
                    )),
                }
            }

            Stmt::TupleAssignment { names, value } => {
                let val = self.eval_expr(value)?;
                if let Value::Array(arr_ref) = val {
                    let arr = arr_ref.borrow();
                    for (i, name) in names.iter().enumerate() {
                        let element_val = arr.get(i).cloned().unwrap_or(Value::Na);
                        self.variables.insert(
                            name.clone(),
                            Variable {
                                value: element_val,
                                is_const: false,
                            },
                        );
                    }
                    Ok(None)
                } else {
                    Err(RuntimeError::TypeError(
                        "Expected array for tuple destructuring".to_string(),
                    ))
                }
            }

            Stmt::Expression(expr) => {
                self.eval_expr(expr)?;
                Ok(None)
            }

            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                let cond_value = self.eval_expr(condition)?;
                if cond_value.as_bool()? {
                    for stmt in then_branch {
                        self.execute_stmt(stmt)?;
                    }
                } else {
                    // Try each else if branch in order
                    let mut executed = false;
                    for (else_if_cond, else_if_body) in else_if_branches {
                        let else_if_value = self.eval_expr(else_if_cond)?;
                        if else_if_value.as_bool()? {
                            for stmt in else_if_body {
                                self.execute_stmt(stmt)?;
                            }
                            executed = true;
                            break;
                        }
                    }

                    // If no else if matched, try else branch
                    if !executed {
                        if let Some(else_stmts) = else_branch {
                            for stmt in else_stmts {
                                self.execute_stmt(stmt)?;
                            }
                        }
                    }
                }
                Ok(None)
            }

            Stmt::For {
                var_name,
                from,
                to,
                body,
            } => {
                let from_val = self.eval_expr(from)?.as_number()?;
                let to_val = self.eval_expr(to)?.as_number()?;

                if from_val > to_val {
                    return Err(RuntimeError::InvalidForLoop(from_val, to_val));
                }

                let mut i = from_val as i64;
                let end = to_val as i64;

                while i <= end {
                    self.variables.insert(
                        var_name.clone(),
                        Variable {
                            value: Value::Number(i as f64),
                            is_const: false,
                        },
                    );

                    let control = self.execute_loop_body(body)?;
                    if control == LoopControl::Break {
                        break;
                    }

                    i += 1;
                }

                Ok(None)
            }

            Stmt::ForIn {
                index_var,
                item_var,
                collection,
                body,
            } => {
                let collection_value = self.eval_expr(collection)?;
                let arr = collection_value.as_array()?;
                let arr_borrowed = arr.borrow();

                for (index, item) in arr_borrowed.iter().enumerate() {
                    // Set index variable if tuple form
                    if let Some(idx_var) = index_var {
                        self.variables.insert(
                            idx_var.clone(),
                            Variable {
                                value: Value::Number(index as f64),
                                is_const: false,
                            },
                        );
                    }

                    // Set item variable
                    self.variables.insert(
                        item_var.clone(),
                        Variable {
                            value: item.clone(),
                            is_const: false,
                        },
                    );

                    let control = self.execute_loop_body(body)?;
                    if control == LoopControl::Break {
                        break;
                    }
                }

                Ok(None)
            }

            Stmt::While { condition, body } => {
                loop {
                    let cond_value = self.eval_expr(condition)?;
                    if !cond_value.as_bool()? {
                        break;
                    }

                    let control = self.execute_loop_body(body)?;
                    if control == LoopControl::Break {
                        break;
                    }
                }
                Ok(None)
            }

            Stmt::Break => Err(RuntimeError::BreakOutsideLoop),
            Stmt::Continue => Err(RuntimeError::ContinueOutsideLoop),

            Stmt::TypeDecl {
                name,
                fields,
                export,
            } => {
                // Create a Type value and store it as a variable
                let type_value = Value::Type {
                    name: name.clone(),
                    fields: fields.clone(),
                };
                self.variables.insert(
                    name.clone(),
                    Variable {
                        value: type_value.clone(),
                        is_const: false,
                    },
                );

                // If exported, also store in exports
                if *export {
                    self.exports.insert(name.clone(), type_value);
                }
                Ok(None)
            }

            Stmt::EnumDecl {
                name,
                fields,
                export,
            } => {
                // Create an Object that contains all enum members as fields
                let mut enum_fields = HashMap::new();

                for field in fields {
                    let title = field.title.clone().unwrap_or_else(|| field.name.clone());
                    let enum_value = Value::Enum {
                        enum_name: name.clone(),
                        field_name: field.name.clone(),
                        title,
                    };
                    enum_fields.insert(field.name.clone(), enum_value);
                }

                let enum_object = Value::Object {
                    type_name: name.clone(),
                    fields: Rc::new(RefCell::new(enum_fields)),
                };
                self.variables.insert(
                    name.clone(),
                    Variable {
                        value: enum_object.clone(),
                        is_const: false,
                    },
                );

                // If exported, also store in exports
                if *export {
                    self.exports.insert(name.clone(), enum_object);
                }
                Ok(None)
            }

            Stmt::Export { item } => {
                // Mark the item for export
                match item {
                    pine_ast::ExportItem::Type(type_name) => {
                        // Export the type - it should already be in variables
                        if let Some(var) = self.variables.get(type_name) {
                            self.exports.insert(type_name.clone(), var.value.clone());
                        }
                    }
                    pine_ast::ExportItem::Function(func_name) => {
                        // Export the function - it should already be in variables
                        if let Some(var) = self.variables.get(func_name) {
                            self.exports.insert(func_name.clone(), var.value.clone());
                        }
                    }
                }
                Ok(None)
            }

            Stmt::Import { path, alias } => {
                // Try to load the library - fail if no loader is available
                if let Some(ref loader) = self.library_loader {
                    match loader.load_library(path) {
                        Ok(library_program) => {
                            // Create a new interpreter for the library
                            let mut library_interp = Interpreter::new();

                            // Execute the library program (without a bar context for simplicity)
                            library_interp.execute(&library_program)?;

                            // Get the exports from the library
                            let library_exports = library_interp.exports();

                            // Import methods from the library
                            for (method_name, method_defs) in &library_interp.methods {
                                for method_def in method_defs {
                                    self.methods
                                        .entry(method_name.clone())
                                        .or_default()
                                        .push(method_def.clone());
                                }
                            }

                            // Create a namespace object containing the exported items
                            let namespace = Value::Object {
                                type_name: alias.clone(),
                                fields: Rc::new(RefCell::new(library_exports.clone())),
                            };
                            self.variables.insert(
                                alias.clone(),
                                Variable {
                                    value: namespace,
                                    is_const: false,
                                },
                            );
                        }
                        Err(e) => {
                            return Err(RuntimeError::LibraryError(format!(
                                "Failed to load library '{}': {}",
                                path, e
                            )));
                        }
                    }
                } else {
                    return Err(RuntimeError::LibraryError(
                        "Cannot import library: no library loader configured".to_string(),
                    ));
                }
                Ok(None)
            }

            Stmt::MethodDecl {
                name,
                params,
                body,
                export,
            } => {
                // Extract the type name from the first parameter's type annotation
                let type_name = if let Some(first_param) = params.first() {
                    first_param.type_annotation.clone().ok_or_else(|| {
                        RuntimeError::TypeError(
                            "Method's first parameter must have a type annotation".to_string(),
                        )
                    })?
                } else {
                    return Err(RuntimeError::TypeError(
                        "Method must have at least one parameter (this)".to_string(),
                    ));
                };

                // Store the method definition
                let method_def = MethodDef {
                    type_name,
                    params: params.clone(),
                    body: body.clone(),
                };

                self.methods
                    .entry(name.clone())
                    .or_default()
                    .push(method_def);

                // If exported, store the method in exports
                // Methods are exported as part of their type, so we may need to handle this differently
                // For now, just mark it as exported (this might need more work)
                if *export {
                    // TODO: Handle method exports properly
                }

                Ok(None)
            }

            Stmt::FunctionDecl {
                name,
                params,
                body,
                export,
            } => {
                // Create a function value
                let func_value = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.variables.insert(
                    name.clone(),
                    Variable {
                        value: func_value.clone(),
                        is_const: false,
                    },
                );

                // If exported, also store in exports
                if *export {
                    self.exports.insert(name.clone(), func_value);
                }

                Ok(None)
            }
        }
    }

    /// Execute loop body, handling break/continue
    fn execute_loop_body(&mut self, body: &[Stmt]) -> Result<LoopControl, RuntimeError> {
        for stmt in body {
            match stmt {
                Stmt::Break => return Ok(LoopControl::Break),
                Stmt::Continue => return Ok(LoopControl::Continue),
                Stmt::If {
                    condition,
                    then_branch,
                    else_if_branches,
                    else_branch,
                } => {
                    let cond_value = self.eval_expr(condition)?;
                    let branch = if cond_value.as_bool()? {
                        then_branch
                    } else {
                        // Try each else if branch
                        let mut matched_branch = None;
                        for (else_if_cond, else_if_body) in else_if_branches {
                            let else_if_value = self.eval_expr(else_if_cond)?;
                            if else_if_value.as_bool()? {
                                matched_branch = Some(else_if_body);
                                break;
                            }
                        }

                        if let Some(branch) = matched_branch {
                            branch
                        } else if let Some(else_stmts) = else_branch {
                            else_stmts
                        } else {
                            continue;
                        }
                    };

                    let control = self.execute_loop_body(branch)?;
                    if control != LoopControl::None {
                        return Ok(control);
                    }
                }
                Stmt::For { .. } | Stmt::ForIn { .. } | Stmt::While { .. } => {
                    // Nested loops handle their own break/continue
                    self.execute_stmt(stmt)?;
                }
                _ => {
                    self.execute_stmt(stmt)?;
                }
            }
        }
        Ok(LoopControl::None)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(self.eval_literal(lit)),

            Expr::Variable(name) => {
                // Check builtins first, then variables
                if let Some(builtin_fn) = self.builtins.get(name).cloned() {
                    Ok(Value::BuiltinFunction(builtin_fn))
                } else {
                    self.variables
                        .get(name)
                        .map(|var| var.value.clone())
                        .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))
                }
            }

            Expr::Binary { left, op, right } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary_op(&left_val, op, &right_val)
            }

            Expr::Unary { op, expr } => {
                let val = self.eval_expr(expr)?;
                self.eval_unary_op(op, &val)
            }

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.as_bool()? {
                    self.eval_expr(then_expr)
                } else {
                    self.eval_expr(else_expr)
                }
            }

            Expr::IfExpr {
                condition,
                then_expr,
                else_if_branches,
                else_expr,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.as_bool()? {
                    self.eval_expr(then_expr)
                } else {
                    // Try each else if branch
                    for (else_if_cond, else_if_expr) in else_if_branches {
                        let else_if_val = self.eval_expr(else_if_cond)?;
                        if else_if_val.as_bool()? {
                            return self.eval_expr(else_if_expr);
                        }
                    }
                    // No else if matched, evaluate else branch or return na
                    if let Some(expr) = else_expr {
                        self.eval_expr(expr)
                    } else {
                        Ok(Value::Na)
                    }
                }
            }

            Expr::Array(elements) => {
                let values: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::Array(Rc::new(RefCell::new(values?))))
            }

            Expr::Index { expr, index } => {
                let val = self.eval_expr(expr)?;
                let index_val = self.eval_expr(index)?.as_number()? as usize;

                match val {
                    Value::Array(arr_ref) => {
                        let arr = arr_ref.borrow();
                        arr.get(index_val)
                            .cloned()
                            .ok_or(RuntimeError::IndexOutOfBounds(index_val))
                    }
                    Value::Series(series) => {
                        // For series, index 0 is current value, index > 0 queries historical provider
                        if index_val == 0 {
                            Ok((*series.current).clone())
                        } else {
                            self.historical_provider
                                .as_ref()
                                .and_then(|p| p.get_historical(&series.id, index_val))
                                .ok_or(RuntimeError::IndexOutOfBounds(index_val))
                        }
                    }
                    ref v => Err(RuntimeError::TypeError(format!(
                        "Cannot index non-array/non-series value: {:?}",
                        v
                    ))),
                }
            }

            Expr::Switch { value, cases } => {
                let switch_val = self.eval_expr(value)?;

                for (pattern, result) in cases {
                    // Check if pattern matches
                    let pattern_val = self.eval_expr(pattern)?;

                    // Special case: default pattern (true literal)
                    if pattern_val == Value::Bool(true)
                        && matches!(pattern, Expr::Literal(Literal::Bool(true)))
                    {
                        return self.eval_expr(result);
                    }

                    // Check equality
                    if self.values_equal(&switch_val, &pattern_val)? {
                        return self.eval_expr(result);
                    }
                }

                // No match found
                Ok(Value::Na)
            }

            Expr::Call {
                callee,
                type_args,
                args,
            } => {
                // Check if this is a method call (object.method())
                if let Expr::MemberAccess { object, member } = callee.as_ref() {
                    // Try to find a method with this name
                    if let Some(method_defs) = self.methods.get(member).cloned() {
                        // Evaluate the object (this will be the first parameter)
                        let obj_value = self.eval_expr(object)?;

                        // Find the method that matches the object's type
                        let obj_type = self.get_object_type_name(&obj_value)?;

                        if let Some(method_def) =
                            method_defs.iter().find(|m| m.type_name == obj_type)
                        {
                            // Evaluate the other arguments
                            let mut evaluated_args = vec![EvaluatedArg::Positional(obj_value)];
                            evaluated_args.extend(self.evaluate_arguments(args)?);

                            // Call the method (treating it like a function)
                            return self.call_method(
                                &method_def.params,
                                &method_def.body,
                                evaluated_args,
                            );
                        }
                    }
                }

                // Not a method call, proceed with regular function call
                // Evaluate arguments and validate positional-before-named rule
                let evaluated_args = self.evaluate_arguments(args)?;

                // Evaluate the callee expression to get the function
                let callee_value = self.eval_expr(callee)?;

                // Call the function based on its type
                match callee_value {
                    Value::Function { params, body } => {
                        self.call_user_function(&params, &body, args, evaluated_args)
                    }
                    Value::BuiltinFunction(builtin_fn) => {
                        // Pass type_args from the parsed call expression
                        let call_args = FunctionCallArgs::new(type_args.clone(), evaluated_args);
                        (builtin_fn)(self, call_args)
                    }
                    _ => Err(RuntimeError::TypeError(
                        "Attempted to call a non-function value".to_string(),
                    )),
                }
            }

            Expr::MemberAccess { object, member } => {
                let obj_value = self.eval_expr(object)?;
                match obj_value {
                    Value::Object { fields, .. } => {
                        let obj = fields.borrow();
                        obj.get(member).cloned().ok_or_else(|| {
                            RuntimeError::TypeError(format!("Object has no member '{}'", member))
                        })
                    }
                    Value::Type { name, fields } => {
                        // Types have 'new' and 'copy' methods
                        if member == "new" {
                            // Return a constructor function
                            Ok(Value::BuiltinFunction(Self::create_constructor(
                                name, fields,
                            )))
                        } else if member == "copy" {
                            // Return a copy function
                            Ok(Value::BuiltinFunction(Self::create_copy_function()))
                        } else {
                            Err(RuntimeError::TypeError(format!(
                                "Type '{}' has no member '{}' (only 'new' and 'copy' are supported)",
                                name, member
                            )))
                        }
                    }
                    _ => Err(RuntimeError::TypeError(format!(
                        "Cannot access member '{}' on non-object value",
                        member
                    ))),
                }
            }

            Expr::Function { params, body } => {
                // params is already Vec<FunctionParam> from the AST
                Ok(Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                })
            }
        }
    }

    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Na => Value::Na,
            Literal::HexColor(hex) => Value::String(hex.clone()),
        }
    }

    fn eval_binary_op(
        &self,
        left: &Value,
        op: &BinOp,
        right: &Value,
    ) -> Result<Value, RuntimeError> {
        match op {
            BinOp::Add => {
                // String concatenation or numeric addition
                if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
                    Ok(Value::String(format!(
                        "{}{}",
                        left.as_string()?,
                        right.as_string()?
                    )))
                } else {
                    Ok(Value::Number(left.as_number()? + right.as_number()?))
                }
            }

            BinOp::Sub => Ok(Value::Number(left.as_number()? - right.as_number()?)),

            BinOp::Mul => Ok(Value::Number(left.as_number()? * right.as_number()?)),

            BinOp::Div => {
                let divisor = right.as_number()?;
                if divisor == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Value::Number(left.as_number()? / divisor))
            }

            BinOp::Mod => {
                let divisor = right.as_number()?;
                if divisor == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Value::Number(left.as_number()? % divisor))
            }

            BinOp::Eq => Ok(Value::Bool(self.values_equal(left, right)?)),

            BinOp::NotEq => Ok(Value::Bool(!self.values_equal(left, right)?)),

            BinOp::Less => Ok(Value::Bool(left.as_number()? < right.as_number()?)),

            BinOp::Greater => Ok(Value::Bool(left.as_number()? > right.as_number()?)),

            BinOp::LessEq => Ok(Value::Bool(left.as_number()? <= right.as_number()?)),

            BinOp::GreaterEq => Ok(Value::Bool(left.as_number()? >= right.as_number()?)),

            BinOp::And => Ok(Value::Bool(left.as_bool()? && right.as_bool()?)),

            BinOp::Or => Ok(Value::Bool(left.as_bool()? || right.as_bool()?)),
        }
    }

    fn eval_unary_op(&self, op: &UnOp, val: &Value) -> Result<Value, RuntimeError> {
        match op {
            UnOp::Neg => Ok(Value::Number(-val.as_number()?)),
            UnOp::Not => Ok(Value::Bool(!val.as_bool()?)),
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> Result<bool, RuntimeError> {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok((l - r).abs() < f64::EPSILON),
            (Value::String(l), Value::String(r)) => Ok(l == r),
            (Value::Bool(l), Value::Bool(r)) => Ok(l == r),
            (Value::Na, Value::Na) => Ok(true),
            (
                Value::Enum {
                    enum_name: a_enum,
                    field_name: a_field,
                    ..
                },
                Value::Enum {
                    enum_name: b_enum,
                    field_name: b_field,
                    ..
                },
            ) => Ok(a_enum == b_enum && a_field == b_field),
            _ => Ok(false),
        }
    }

    /// Check if an expression evaluates to a const value
    fn is_const_expr(&self, expr: &Expr) -> bool {
        match expr {
            // Literals are always const
            Expr::Literal(_) => true,
            // Variable is const if it's stored as const
            Expr::Variable(name) => self
                .variables
                .get(name)
                .map(|var| var.is_const)
                .unwrap_or(false),
            // Member access is const if the base object is const
            Expr::MemberAccess { object, .. } => self.is_const_expr(object),
            // All other expressions are not const
            _ => false,
        }
    }

    fn call_user_function(
        &mut self,
        params: &[pine_ast::FunctionParam],
        body: &[Stmt],
        arg_exprs: &[Argument],
        args: Vec<EvaluatedArg>,
    ) -> Result<Value, RuntimeError> {
        // Extract positional arguments (user functions don't support named args yet)
        let mut positional_values = Vec::new();
        let mut positional_exprs = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            match arg {
                EvaluatedArg::Positional(value) => {
                    positional_values.push(value.clone());
                    if let Some(Argument::Positional(expr)) = arg_exprs.get(i) {
                        positional_exprs.push(expr);
                    }
                }
                EvaluatedArg::Named { .. } => {
                    return Err(RuntimeError::TypeError(
                        "User-defined functions do not support named arguments yet".to_string(),
                    ))
                }
            }
        }

        // Check argument count
        if positional_values.len() != params.len() {
            return Err(RuntimeError::TypeError(format!(
                "Expected {} arguments, got {}",
                params.len(),
                positional_values.len()
            )));
        }

        // Validate const parameters receive const arguments
        for (i, param) in params.iter().enumerate() {
            if matches!(param.type_qualifier, Some(pine_ast::TypeQualifier::Const)) {
                if let Some(arg_expr) = positional_exprs.get(i) {
                    if !self.is_const_expr(arg_expr) {
                        return Err(RuntimeError::TypeError(format!(
                            "Parameter '{}' requires a const argument, but received a non-const value",
                            param.name
                        )));
                    }
                }
            }
        }

        // Save current variable state (for function scope)
        let saved_vars = self.variables.clone();

        // Bind parameters to arguments with appropriate const flag
        for (param, value) in params.iter().zip(positional_values.iter()) {
            let is_const = matches!(param.type_qualifier, Some(pine_ast::TypeQualifier::Const));
            self.variables.insert(
                param.name.clone(),
                Variable {
                    value: value.clone(),
                    is_const,
                },
            );
        }

        // Execute function body
        let mut result = Value::Na;
        for stmt in body {
            if let Some(return_value) = self.execute_stmt(stmt)? {
                result = return_value;
            } else if let Stmt::Expression(expr) = stmt {
                // Last expression is the return value
                result = self.eval_expr(expr)?;
            }
        }

        // Restore variable state
        self.variables = saved_vars;

        Ok(result)
    }

    /// Get the type name for an object value
    fn get_object_type_name(&self, value: &Value) -> Result<String, RuntimeError> {
        match value {
            Value::Object { type_name, .. } => Ok(type_name.clone()),
            _ => Err(RuntimeError::TypeError(
                "Cannot determine type of non-object value".to_string(),
            )),
        }
    }

    /// Call a method (similar to call_user_function but handles MethodParam with defaults)
    fn call_method(
        &mut self,
        params: &[MethodParam],
        body: &[Stmt],
        args: Vec<EvaluatedArg>,
    ) -> Result<Value, RuntimeError> {
        // Save current variable state (for method scope)
        let saved_vars = self.variables.clone();

        // Bind parameters to arguments
        let mut positional_idx = 0;

        for param in params {
            let param_value = if positional_idx < args.len() {
                match &args[positional_idx] {
                    EvaluatedArg::Positional(value) => {
                        positional_idx += 1;
                        value.clone()
                    }
                    EvaluatedArg::Named { name, value } => {
                        if name == &param.name {
                            positional_idx += 1;
                            value.clone()
                        } else if let Some(default_expr) = &param.default_value {
                            self.eval_expr(default_expr)?
                        } else {
                            Value::Na
                        }
                    }
                }
            } else if let Some(default_expr) = &param.default_value {
                self.eval_expr(default_expr)?
            } else {
                Value::Na
            };

            self.variables.insert(
                param.name.clone(),
                Variable {
                    value: param_value,
                    is_const: false,
                },
            );
        }

        // Execute method body
        let mut result = Value::Na;
        for stmt in body {
            if let Some(return_value) = self.execute_stmt(stmt)? {
                result = return_value;
            } else if let Stmt::Expression(expr) = stmt {
                // Last expression is the return value
                result = self.eval_expr(expr)?;
            }
        }

        // Restore variable state
        self.variables = saved_vars;

        Ok(result)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a constructor function for a user-defined type
    fn create_constructor(type_name: String, fields: Vec<TypeField>) -> BuiltinFn {
        Rc::new(
            move |interp: &mut Interpreter, call_args: FunctionCallArgs| {
                let mut instance_fields = HashMap::new();

                // Match arguments to fields
                let mut positional_idx = 0;

                for arg in &call_args.args {
                    match arg {
                        EvaluatedArg::Positional(value) => {
                            // Assign to field by position
                            if positional_idx < fields.len() {
                                let field = &fields[positional_idx];
                                instance_fields.insert(field.name.clone(), value.clone());
                                positional_idx += 1;
                            } else {
                                return Err(RuntimeError::TypeError(format!(
                                    "Too many arguments for type '{}' (expected {} fields)",
                                    type_name,
                                    fields.len()
                                )));
                            }
                        }
                        EvaluatedArg::Named { name, value } => {
                            // Find field by name
                            if let Some(field) = fields.iter().find(|f| f.name == *name) {
                                instance_fields.insert(field.name.clone(), value.clone());
                            } else {
                                return Err(RuntimeError::TypeError(format!(
                                    "Type '{}' has no field '{}'",
                                    type_name, name
                                )));
                            }
                        }
                    }
                }

                // Fill in defaults for missing fields
                for field in &fields {
                    if !instance_fields.contains_key(&field.name) {
                        if let Some(default_expr) = &field.default_value {
                            let default_val = interp.eval_expr(default_expr)?;
                            instance_fields.insert(field.name.clone(), default_val);
                        } else {
                            // Field has no default and wasn't provided
                            instance_fields.insert(field.name.clone(), Value::Na);
                        }
                    }
                }

                Ok(Value::Object {
                    type_name: type_name.clone(),
                    fields: Rc::new(RefCell::new(instance_fields)),
                })
            },
        )
    }

    /// Creates a copy function for types that takes an object and returns a shallow copy
    fn create_copy_function() -> BuiltinFn {
        Rc::new(|_interp: &mut Interpreter, call_args: FunctionCallArgs| {
            // Expect exactly one positional argument (the object to copy)
            if call_args.args.len() != 1 {
                return Err(RuntimeError::TypeError(
                    "copy() expects exactly one argument".to_string(),
                ));
            }

            match &call_args.args[0] {
                EvaluatedArg::Positional(value) => {
                    if let Value::Object { type_name, fields } = value {
                        // Create a shallow copy of the object's fields
                        let obj = fields.borrow();
                        let copied_fields = obj.clone();
                        Ok(Value::Object {
                            type_name: type_name.clone(),
                            fields: Rc::new(RefCell::new(copied_fields)),
                        })
                    } else {
                        Err(RuntimeError::TypeError(
                            "copy() expects an object argument".to_string(),
                        ))
                    }
                }
                EvaluatedArg::Named { .. } => Err(RuntimeError::TypeError(
                    "copy() does not accept named arguments".to_string(),
                )),
            }
        })
    }
}
