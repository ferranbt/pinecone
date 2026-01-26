use pine_ast::{Argument, BinOp, Expr, Literal, Program, Stmt, UnOp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

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
}

/// Control flow signals for loops
#[derive(Debug, Clone, PartialEq)]
enum LoopControl {
    None,
    Break,
    Continue,
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

/// Value types in the interpreter
#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Na,                             // PineScript's N/A value
    Array(Rc<RefCell<Vec<Value>>>), // Mutable shared array reference
    Object(Rc<RefCell<HashMap<String, Value>>>), // Dictionary/Object with string keys
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    BuiltinFunction(BuiltinFn), // Builtin function pointer
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
            Value::Object(o) => write!(f, "Object({:?})", o),
            Value::Function { params, .. } => write!(f, "Function({} params)", params.len()),
            Value::BuiltinFunction(_) => write!(f, "BuiltinFunction"),
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
            // Arrays and Objects compare by reference (Rc pointer equality)
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b),
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b),
            // Functions never equal (can't compare closures or function pointers)
            (Value::Function { .. }, Value::Function { .. }) => false,
            (Value::BuiltinFunction(_), Value::BuiltinFunction(_)) => false,
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

/// Type signature for builtin functions
pub type BuiltinFn = fn(&mut Interpreter, Vec<EvaluatedArg>) -> Result<Value, RuntimeError>;

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
}

/// The interpreter executes a program with a given bar
pub struct Interpreter {
    /// Local variables in the current scope
    variables: HashMap<String, Value>,
    /// Builtin function registry
    builtins: HashMap<String, BuiltinFn>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            builtins: HashMap::new(),
        }
    }

    /// Create interpreter with custom builtins
    pub fn with_builtins(builtins: HashMap<String, BuiltinFn>) -> Self {
        Self {
            variables: HashMap::new(),
            builtins,
        }
    }

    /// Execute a program with a single bar
    pub fn execute(&mut self, program: &Program, bar: &Bar) -> Result<(), RuntimeError> {
        // Initialize builtin variables from bar
        self.variables
            .insert("open".to_string(), Value::Number(bar.open));
        self.variables
            .insert("high".to_string(), Value::Number(bar.high));
        self.variables
            .insert("low".to_string(), Value::Number(bar.low));
        self.variables
            .insert("close".to_string(), Value::Number(bar.close));
        self.variables
            .insert("volume".to_string(), Value::Number(bar.volume));

        for stmt in &program.statements {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Set a variable value (useful for loading objects and test setup)
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation: _,
                initializer,
                is_varip: _,  // TODO: implement varip behavior (requires stateful execution)
            } => {
                let value = if let Some(init_expr) = initializer {
                    self.eval_expr(init_expr)?
                } else {
                    Value::Na
                };
                self.variables.insert(name.clone(), value);
                Ok(None)
            }

            Stmt::Assignment { name, value } => {
                let val = self.eval_expr(value)?;
                self.variables.insert(name.clone(), val);
                Ok(None)
            }

            Stmt::TupleAssignment { names, value } => {
                let val = self.eval_expr(value)?;
                if let Value::Array(arr_ref) = val {
                    let arr = arr_ref.borrow();
                    for (i, name) in names.iter().enumerate() {
                        let element_val = arr.get(i).cloned().unwrap_or(Value::Na);
                        self.variables.insert(name.clone(), element_val);
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
                    self.variables
                        .insert(var_name.clone(), Value::Number(i as f64));

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
                        self.variables
                            .insert(idx_var.clone(), Value::Number(index as f64));
                    }

                    // Set item variable
                    self.variables.insert(item_var.clone(), item.clone());

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

            Expr::Variable(name) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone())),

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
                    // No else if matched, evaluate else branch
                    self.eval_expr(else_expr)
                }
            }

            Expr::Array(elements) => {
                let values: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::Array(Rc::new(RefCell::new(values?))))
            }

            Expr::Index { expr, index } => {
                let array_val = self.eval_expr(expr)?;
                let index_val = self.eval_expr(index)?.as_number()? as usize;

                if let Value::Array(arr_ref) = array_val {
                    let arr = arr_ref.borrow();
                    arr.get(index_val)
                        .cloned()
                        .ok_or(RuntimeError::IndexOutOfBounds(index_val))
                } else {
                    Err(RuntimeError::TypeError(
                        "Cannot index non-array value".to_string(),
                    ))
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

            Expr::Call { callee, args } => {
                // Evaluate arguments and validate positional-before-named rule
                let mut evaluated_args = Vec::new();
                let mut seen_named = false;

                for arg in args {
                    match arg {
                        Argument::Positional(expr) => {
                            if seen_named {
                                return Err(RuntimeError::TypeError(
                                    "Positional arguments cannot follow named arguments"
                                        .to_string(),
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

                // Look up builtin function first
                if let Some(&builtin_fn) = self.builtins.get(callee) {
                    builtin_fn(self, evaluated_args)
                } else if let Some(func_value) = self.variables.get(callee).cloned() {
                    // Check what type of function it is
                    match func_value {
                        Value::Function { params, body } => {
                            self.call_user_function(&params, &body, evaluated_args)
                        }
                        Value::BuiltinFunction(builtin_fn) => {
                            builtin_fn(self, evaluated_args)
                        }
                        _ => Err(RuntimeError::TypeError(format!(
                            "'{}' is not a function",
                            callee
                        )))
                    }
                } else {
                    Err(RuntimeError::UndefinedVariable(format!(
                        "Unknown function: {}",
                        callee
                    )))
                }
            }

            Expr::MemberAccess { object, member } => {
                let obj_value = self.eval_expr(object)?;
                match obj_value {
                    Value::Object(obj_ref) => {
                        let obj = obj_ref.borrow();
                        obj.get(member)
                            .cloned()
                            .ok_or_else(|| RuntimeError::TypeError(format!(
                                "Object has no member '{}'",
                                member
                            )))
                    }
                    _ => Err(RuntimeError::TypeError(format!(
                        "Cannot access member '{}' on non-object value",
                        member
                    ))),
                }
            }

            Expr::Function { params, body } => {
                // Create a function value
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
            _ => Ok(false),
        }
    }

    fn call_user_function(
        &mut self,
        params: &[String],
        body: &[Stmt],
        args: Vec<EvaluatedArg>,
    ) -> Result<Value, RuntimeError> {
        // Extract positional arguments (user functions don't support named args yet)
        let mut positional_values = Vec::new();
        for arg in args {
            match arg {
                EvaluatedArg::Positional(value) => positional_values.push(value),
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

        // Save current variable state (for function scope)
        let saved_vars = self.variables.clone();

        // Bind parameters to arguments
        for (param, value) in params.iter().zip(positional_values.iter()) {
            self.variables.insert(param.clone(), value.clone());
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
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Helper function to parse PineScript source code into a Program
    fn parse_str(source: &str) -> eyre::Result<Program> {
        use pine_lexer::Lexer;
        use pine_parser::Parser;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        Ok(Program::new(statements))
    }

    /// Test builtin function: greet(name, greeting="Hello")
    fn test_greet(
        _interp: &mut Interpreter,
        args: Vec<EvaluatedArg>,
    ) -> Result<Value, RuntimeError> {
        let mut name: Option<String> = None;
        let mut greeting = "Hello".to_string();

        let mut positional_idx = 0;
        for arg in args {
            match arg {
                EvaluatedArg::Positional(value) => {
                    if positional_idx == 0 {
                        name = Some(value.as_string()?);
                    } else if positional_idx == 1 {
                        greeting = value.as_string()?;
                    }
                    positional_idx += 1;
                }
                EvaluatedArg::Named {
                    name: param_name,
                    value,
                } => {
                    if param_name == "greeting" {
                        greeting = value.as_string()?;
                    }
                }
            }
        }

        let name = name.ok_or_else(|| RuntimeError::TypeError("Missing name".to_string()))?;
        Ok(Value::String(format!("{}, {}", greeting, name)))
    }

    #[test]
    fn test_variable_declaration() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str("var x = 42")?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("x"), Some(&Value::Number(42.0)));
        Ok(())
    }

    #[test]
    fn test_arithmetic() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var a = 10
            var b = 5
            var result = a + b
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("a"), Some(&Value::Number(10.0)));
        assert_eq!(interp.get_variable("b"), Some(&Value::Number(5.0)));
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(15.0)));
        Ok(())
    }

    #[test]
    fn test_if_statement() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var x = 10
            if x > 5
                var result = "greater"
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("x"), Some(&Value::Number(10.0)));
        assert_eq!(
            interp.get_variable("result"),
            Some(&Value::String("greater".to_string()))
        );
        Ok(())
    }

    #[test]
    fn test_for_loop() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var sum = 0
            for i = 1 to 5
                sum := sum + i
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // Sum of 1+2+3+4+5 = 15
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(15.0)));
        Ok(())
    }

    #[test]
    fn test_named_arguments() -> eyre::Result<()> {
        let mut builtins = HashMap::new();
        builtins.insert("greet".to_string(), test_greet as BuiltinFn);
        let mut interp = Interpreter::with_builtins(builtins);

        // Test positional arguments only (uses default greeting)
        let program1 = parse_str(r#"var msg1 = greet("Alice")"#)?;
        interp.execute(&program1, &Bar::default())?;
        assert_eq!(
            interp.get_variable("msg1"),
            Some(&Value::String("Hello, Alice".to_string()))
        );

        // Test positional argument with named parameter
        let program2 = parse_str(r#"var msg2 = greet("Bob", greeting="Hi")"#)?;
        interp.execute(&program2, &Bar::default())?;
        assert_eq!(
            interp.get_variable("msg2"),
            Some(&Value::String("Hi, Bob".to_string()))
        );

        // Test both positional arguments
        let program3 = parse_str(r#"var msg3 = greet("Charlie", "Hey")"#)?;
        interp.execute(&program3, &Bar::default())?;
        assert_eq!(
            interp.get_variable("msg3"),
            Some(&Value::String("Hey, Charlie".to_string()))
        );

        Ok(())
    }

    #[test]
    fn test_invalid_named_argument_order() -> eyre::Result<()> {
        let mut builtins = HashMap::new();
        builtins.insert("greet".to_string(), test_greet as BuiltinFn);
        let mut interp = Interpreter::with_builtins(builtins);

        // This should fail: positional arg after named arg
        let program = parse_str(r#"var msg = greet(greeting="Hi", "Alice")"#)?;
        let result = interp.execute(&program, &Bar::default());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Positional arguments cannot follow named arguments"));

        Ok(())
    }

    #[test]
    fn test_switch_expression() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var x = 2
            var result = switch x
                1 => "one"
                2 => "two"
                3 => "three"
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(
            interp.get_variable("result"),
            Some(&Value::String("two".to_string()))
        );
        Ok(())
    }

    #[test]
    fn test_user_defined_function() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            add(a, b) => a + b
            var result = add(3, 5)
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(8.0)));
        Ok(())
    }

    #[test]
    fn test_user_function_with_variables() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            double(x) => x * 2
            var value = 10
            var result = double(value)
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(20.0)));
        Ok(())
    }

    #[test]
    fn test_while_simple() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var i = 0
            while i < 5
                i := i + 1
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("i"), Some(&Value::Number(5.0)));
        Ok(())
    }

    #[test]
    fn test_while_break() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var i = 0
            while i < 10
                if i == 5
                    break
                i := i + 1
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("i"), Some(&Value::Number(5.0)));
        Ok(())
    }

    #[test]
    fn test_while_continue() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var i = 0
            var sum = 0
            while i < 10
                i := i + 1
                if i % 2 == 0
                    continue
                sum := sum + i
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // sum = 1 + 3 + 5 + 7 + 9 = 25
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(25.0)));
        assert_eq!(interp.get_variable("i"), Some(&Value::Number(10.0)));
        Ok(())
    }

    #[test]
    fn test_while_break_continue() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var i = 0
            var sum = 0
            while i < 20
                i := i + 1
                if i > 10
                    break
                if i % 2 == 0
                    continue
                sum := sum + i
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // sum = 1 + 3 + 5 + 7 + 9 = 25 (stops at i=11)
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(25.0)));
        assert_eq!(interp.get_variable("i"), Some(&Value::Number(11.0)));
        Ok(())
    }

    #[test]
    fn test_while_nested() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var i = 0
            var sum = 0
            while i < 3
                var j = 0
                while j < 2
                    sum := sum + i * j
                    j := j + 1
                i := i + 1
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // sum = (0*0 + 0*1) + (1*0 + 1*1) + (2*0 + 2*1) = 0 + 1 + 2 = 3
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(3.0)));
        Ok(())
    }

    #[test]
    fn test_break_outside_loop() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str("break")?;
        let result = interp.execute(&program, &Bar::default());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Break statement outside of loop"));
        Ok(())
    }

    #[test]
    fn test_continue_outside_loop() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str("continue")?;
        let result = interp.execute(&program, &Bar::default());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Continue statement outside of loop"));
        Ok(())
    }

    #[test]
    fn test_compound_assignment() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var x = 10
            x += 5
            var y = 20
            y -= 3
            var z = 4
            z *= 2
            var w = 20
            w /= 4
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("x"), Some(&Value::Number(15.0)));
        assert_eq!(interp.get_variable("y"), Some(&Value::Number(17.0)));
        assert_eq!(interp.get_variable("z"), Some(&Value::Number(8.0)));
        assert_eq!(interp.get_variable("w"), Some(&Value::Number(5.0)));
        Ok(())
    }

    #[test]
    fn test_for_in_simple() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var arr = [1, 2, 3, 4, 5]
            var sum = 0
            for item in arr
                sum := sum + item
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // sum = 1 + 2 + 3 + 4 + 5 = 15
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(15.0)));
        Ok(())
    }

    #[test]
    fn test_for_in_tuple() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var arr = [10, 20, 30]
            var sum = 0
            for [i, value] in arr
                sum := sum + i * value
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // sum = 0*10 + 1*20 + 2*30 = 0 + 20 + 60 = 80
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(80.0)));
        Ok(())
    }

    #[test]
    fn test_for_in_break() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var arr = [1, 2, 3, 4, 5]
            var sum = 0
            for item in arr
                if item == 3
                    break
                sum := sum + item
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // sum = 1 + 2 = 3 (stops at item=3)
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(3.0)));
        Ok(())
    }

    #[test]
    fn test_for_in_continue() -> eyre::Result<()> {
        let mut interp = Interpreter::new();
        let program = parse_str(
            r#"
            var arr = [1, 2, 3, 4, 5]
            var sum = 0
            for item in arr
                if item % 2 == 0
                    continue
                sum := sum + item
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        // sum = 1 + 3 + 5 = 9 (skips even numbers)
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(9.0)));
        Ok(())
    }

    #[test]
    fn test_member_access() -> eyre::Result<()> {
        let mut interp = Interpreter::new();

        // Create an object with some properties
        let mut color_obj = HashMap::new();
        color_obj.insert("red".to_string(), Value::String("#FF0000".to_string()));
        color_obj.insert("blue".to_string(), Value::String("#0000FF".to_string()));
        color_obj.insert("green".to_string(), Value::String("#00FF00".to_string()));

        // Load the object into the interpreter
        interp.set_variable("color", Value::Object(Rc::new(RefCell::new(color_obj))));

        // Test member access
        let program = parse_str(
            r#"
            var myColor = color.red
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(
            interp.get_variable("myColor"),
            Some(&Value::String("#FF0000".to_string()))
        );
        Ok(())
    }

    #[test]
    fn test_member_access_nested() -> eyre::Result<()> {
        let mut interp = Interpreter::new();

        // Create a nested object structure
        let mut barstate_obj = HashMap::new();
        barstate_obj.insert("islast".to_string(), Value::Bool(true));
        barstate_obj.insert("isrealtime".to_string(), Value::Bool(false));

        interp.set_variable("barstate", Value::Object(Rc::new(RefCell::new(barstate_obj))));

        let program = parse_str(
            r#"
            var isLast = barstate.islast
            var isRealtime = barstate.isrealtime
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("isLast"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("isRealtime"), Some(&Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_builtin_function_in_object() -> eyre::Result<()> {
        let mut interp = Interpreter::new();

        // Create a test builtin function
        fn test_add(_ctx: &mut Interpreter, args: Vec<EvaluatedArg>) -> Result<Value, RuntimeError> {
            let mut sum = 0.0;
            for arg in args {
                if let EvaluatedArg::Positional(Value::Number(n)) = arg {
                    sum += n;
                }
            }
            Ok(Value::Number(sum))
        }

        // Create a namespace object with a builtin function
        let mut math_ns = HashMap::new();
        math_ns.insert("add".to_string(), Value::BuiltinFunction(test_add));

        interp.set_variable("math", Value::Object(Rc::new(RefCell::new(math_ns))));

        // Access the function and call it
        let program = parse_str(
            r#"
            var addFunc = math.add
            var result = addFunc(1, 2, 3)
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(6.0)));
        Ok(())
    }

    #[test]
    fn test_member_access_nonexistent() -> eyre::Result<()> {
        let mut interp = Interpreter::new();

        let mut obj = HashMap::new();
        obj.insert("foo".to_string(), Value::Number(42.0));

        interp.set_variable("obj", Value::Object(Rc::new(RefCell::new(obj))));

        let program = parse_str(
            r#"
            var x = obj.bar
            "#,
        )?;

        let result = interp.execute(&program, &Bar::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("has no member 'bar'"));
        Ok(())
    }

    #[test]
    fn test_else_if() -> eyre::Result<()> {
        let mut interp = Interpreter::new();

        // Test first else if branch matches
        let program = parse_str(
            r#"
            var x = 5
            var result = 0

            if x < 0
                result := -1
            else if x == 0
                result := 0
            else if x < 10
                result := 1
            else
                result := 2
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(1.0)));

        // Test else branch when no condition matches
        let mut interp2 = Interpreter::new();
        let program2 = parse_str(
            r#"
            var x = 100
            var result = 0

            if x < 0
                result := -1
            else if x == 0
                result := 0
            else if x < 10
                result := 1
            else
                result := 2
            "#,
        )?;

        interp2.execute(&program2, &Bar::default())?;
        assert_eq!(interp2.get_variable("result"), Some(&Value::Number(2.0)));

        // Test first condition matches (skips else if)
        let mut interp3 = Interpreter::new();
        let program3 = parse_str(
            r#"
            var x = -5
            var result = 0

            if x < 0
                result := -1
            else if x == 0
                result := 0
            else if x < 10
                result := 1
            else
                result := 2
            "#,
        )?;

        interp3.execute(&program3, &Bar::default())?;
        assert_eq!(interp3.get_variable("result"), Some(&Value::Number(-1.0)));

        Ok(())
    }

    #[test]
    fn test_if_expression() -> eyre::Result<()> {
        let mut interp = Interpreter::new();

        // Test if expression with else if
        let program = parse_str(
            r#"
            var x = 5
            string result = if x < 0
                "negative"
            else if x == 0
                "zero"
            else if x < 10
                "small"
            else
                "large"
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("result"), Some(&Value::String("small".to_string())));

        // Test with different values
        let mut interp2 = Interpreter::new();
        let program2 = parse_str(
            r#"
            var x = 100
            string result = if x < 0
                "negative"
            else if x == 0
                "zero"
            else if x < 10
                "small"
            else
                "large"
            "#,
        )?;

        interp2.execute(&program2, &Bar::default())?;
        assert_eq!(interp2.get_variable("result"), Some(&Value::String("large".to_string())));

        Ok(())
    }

    #[test]
    fn test_if_expression_simple() -> eyre::Result<()> {
        let mut interp = Interpreter::new();

        // Test simple if-else expression without else-if
        let program = parse_str(
            r#"
            var x = 5
            string result = if x > 0
                "positive"
            else
                "non-positive"
            "#,
        )?;

        interp.execute(&program, &Bar::default())?;
        assert_eq!(interp.get_variable("result"), Some(&Value::String("positive".to_string())));

        // Test else branch
        let mut interp2 = Interpreter::new();
        let program2 = parse_str(
            r#"
            var x = -5
            string result = if x > 0
                "positive"
            else
                "non-positive"
            "#,
        )?;

        interp2.execute(&program2, &Bar::default())?;
        assert_eq!(interp2.get_variable("result"), Some(&Value::String("non-positive".to_string())));

        Ok(())
    }
}
