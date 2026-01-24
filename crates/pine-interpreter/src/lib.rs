use pine_ast::{BinOp, Expr, Literal, Program, Stmt, UnOp};
use std::collections::HashMap;
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
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Na, // PineScript's N/A value
    Array(Vec<Value>),
}

impl Value {
    fn as_number(&self) -> Result<f64, RuntimeError> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected number, got {:?}",
                self
            ))),
        }
    }

    fn as_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::Number(n) => Ok(*n != 0.0),
            _ => Err(RuntimeError::TypeError(format!(
                "Expected bool, got {:?}",
                self
            ))),
        }
    }

    fn as_string(&self) -> Result<String, RuntimeError> {
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
}

/// The interpreter executes a program with a given bar
pub struct Interpreter {
    /// Local variables in the current scope
    variables: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Execute a program with a single bar
    pub fn execute(&mut self, program: &Program, _bar: &Bar) -> Result<(), RuntimeError> {
        for stmt in &program.statements {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation: _,
                initializer,
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
                if let Value::Array(elements) = val {
                    for (i, name) in names.iter().enumerate() {
                        let element_val = elements
                            .get(i)
                            .cloned()
                            .unwrap_or(Value::Na);
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
                else_branch,
            } => {
                let cond_value = self.eval_expr(condition)?;
                if cond_value.as_bool()? {
                    for stmt in then_branch {
                        self.execute_stmt(stmt)?;
                    }
                } else if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.execute_stmt(stmt)?;
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

                    for stmt in body {
                        self.execute_stmt(stmt)?;
                    }

                    i += 1;
                }

                Ok(None)
            }
        }
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

            Expr::Array(elements) => {
                let values: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::Array(values?))
            }

            Expr::Index { expr, index } => {
                let array_val = self.eval_expr(expr)?;
                let index_val = self.eval_expr(index)?.as_number()? as usize;

                if let Value::Array(elements) = array_val {
                    elements
                        .get(index_val)
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

            // Not yet implemented - will be added later
            Expr::Call { .. } => Err(RuntimeError::TypeError(
                "Function calls not yet implemented".to_string(),
            )),

            Expr::MemberAccess { .. } => Err(RuntimeError::TypeError(
                "Member access not yet implemented".to_string(),
            )),

            Expr::Function { .. } => Err(RuntimeError::TypeError(
                "Function definitions not yet implemented".to_string(),
            )),
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
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
