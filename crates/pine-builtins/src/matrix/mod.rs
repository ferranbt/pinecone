use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// matrix.new<type>() - Creates a new typed matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.new", type_params = 1)]
struct MatrixNew {
    #[type_param]
    element_type: String,
    #[arg(default = Value::Number(0.0))]
    rows: Value,
    #[arg(default = Value::Number(0.0))]
    columns: Value,
    #[arg(default = Value::Na)]
    initial_value: Value,
}

impl MatrixNew {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let rows = match &self.rows {
            Value::Number(n) => *n as usize,
            _ => return Err(RuntimeError::TypeError("rows must be a number".to_string())),
        };

        let columns = match &self.columns {
            Value::Number(n) => *n as usize,
            _ => {
                return Err(RuntimeError::TypeError(
                    "columns must be a number".to_string(),
                ))
            }
        };

        // Validate element type
        if !matches!(
            self.element_type.as_str(),
            "int" | "float" | "string" | "bool"
        ) {
            return Err(RuntimeError::TypeError(format!(
                "Invalid matrix element type '{}'. Must be int, float, string, or bool",
                self.element_type
            )));
        }

        // Create a matrix filled with the initial value
        let mut matrix_data = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(columns);
            for _ in 0..columns {
                row.push(self.initial_value.clone());
            }
            matrix_data.push(row);
        }

        Ok(Value::Matrix {
            element_type: self.element_type.clone(),
            data: Rc::new(RefCell::new(matrix_data)),
        })
    }
}

/// matrix.get() - Gets an element from the matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.get")]
struct MatrixGet {
    id: Value,
    row: Value,
    column: Value,
}

impl MatrixGet {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let row_idx = match &self.row {
            Value::Number(n) => *n as usize,
            _ => return Err(RuntimeError::TypeError("row must be a number".to_string())),
        };

        let col_idx = match &self.column {
            Value::Number(n) => *n as usize,
            _ => {
                return Err(RuntimeError::TypeError(
                    "column must be a number".to_string(),
                ))
            }
        };

        let matrix_ref = matrix.borrow();
        if row_idx >= matrix_ref.len() {
            return Err(RuntimeError::IndexOutOfBounds(row_idx));
        }
        if col_idx >= matrix_ref[row_idx].len() {
            return Err(RuntimeError::IndexOutOfBounds(col_idx));
        }

        Ok(matrix_ref[row_idx][col_idx].clone())
    }
}

/// matrix.set() - Sets an element in the matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.set")]
struct MatrixSet {
    id: Value,
    row: Value,
    column: Value,
    value: Value,
}

impl MatrixSet {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let row_idx = match &self.row {
            Value::Number(n) => *n as usize,
            _ => return Err(RuntimeError::TypeError("row must be a number".to_string())),
        };

        let col_idx = match &self.column {
            Value::Number(n) => *n as usize,
            _ => {
                return Err(RuntimeError::TypeError(
                    "column must be a number".to_string(),
                ))
            }
        };

        let mut matrix_ref = matrix.borrow_mut();
        if row_idx >= matrix_ref.len() {
            return Err(RuntimeError::IndexOutOfBounds(row_idx));
        }
        if col_idx >= matrix_ref[row_idx].len() {
            return Err(RuntimeError::IndexOutOfBounds(col_idx));
        }

        matrix_ref[row_idx][col_idx] = self.value.clone();
        Ok(Value::Na)
    }
}

/// matrix.rows() - Returns the number of rows
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.rows")]
struct MatrixRows {
    id: Value,
}

impl MatrixRows {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let count = matrix.borrow().len();
        Ok(Value::Number(count as f64))
    }
}

/// matrix.columns() - Returns the number of columns
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.columns")]
struct MatrixColumns {
    id: Value,
}

impl MatrixColumns {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let matrix_ref = matrix.borrow();
        let count = if matrix_ref.is_empty() {
            0
        } else {
            matrix_ref[0].len()
        };
        Ok(Value::Number(count as f64))
    }
}

/// matrix.elements_count() - Returns total number of elements
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.elements_count")]
struct MatrixElementsCount {
    id: Value,
}

impl MatrixElementsCount {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let matrix_ref = matrix.borrow();
        let count: usize = matrix_ref.iter().map(|row| row.len()).sum();
        Ok(Value::Number(count as f64))
    }
}

/// matrix.fill() - Fills the matrix with a value
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.fill")]
struct MatrixFill {
    id: Value,
    value: Value,
}

impl MatrixFill {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let mut matrix_ref = matrix.borrow_mut();
        for row in matrix_ref.iter_mut() {
            for cell in row.iter_mut() {
                *cell = self.value.clone();
            }
        }

        Ok(Value::Na)
    }
}

/// matrix.copy() - Creates a copy of the matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.copy")]
struct MatrixCopy {
    id: Value,
}

impl MatrixCopy {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let matrix_ref = matrix.borrow();
        let copied_data = matrix_ref.clone();
        Ok(Value::Matrix {
            element_type: "float".to_string(),
            data: Rc::new(RefCell::new(copied_data)),
        })
    }
}

/// matrix.add_row() - Adds a row to the matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.add_row")]
struct MatrixAddRow {
    id: Value,
    row: Value,
    #[arg(default = Value::Na)]
    array_id: Value,
}

impl MatrixAddRow {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let row_idx = match &self.row {
            Value::Number(n) => *n as usize,
            _ => return Err(RuntimeError::TypeError("row must be a number".to_string())),
        };

        let mut matrix_ref = matrix.borrow_mut();
        let cols = if matrix_ref.is_empty() {
            0
        } else {
            matrix_ref[0].len()
        };

        // If array_id is provided, use its values, otherwise use na
        let new_row = match &self.array_id {
            Value::Array(arr) => arr.borrow().clone(),
            Value::Na => vec![Value::Na; cols],
            _ => {
                return Err(RuntimeError::TypeError(
                    "array_id must be an array".to_string(),
                ))
            }
        };

        if row_idx > matrix_ref.len() {
            return Err(RuntimeError::IndexOutOfBounds(row_idx));
        }

        matrix_ref.insert(row_idx, new_row);
        Ok(Value::Na)
    }
}

/// matrix.add_col() - Adds a column to the matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.add_col")]
struct MatrixAddCol {
    id: Value,
    column: Value,
    #[arg(default = Value::Na)]
    array_id: Value,
}

impl MatrixAddCol {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let col_idx = match &self.column {
            Value::Number(n) => *n as usize,
            _ => {
                return Err(RuntimeError::TypeError(
                    "column must be a number".to_string(),
                ))
            }
        };

        let mut matrix_ref = matrix.borrow_mut();

        // Get column values from array or use na
        let col_values = match &self.array_id {
            Value::Array(arr) => arr.borrow().clone(),
            Value::Na => vec![Value::Na; matrix_ref.len()],
            _ => {
                return Err(RuntimeError::TypeError(
                    "array_id must be an array".to_string(),
                ))
            }
        };

        for (i, row) in matrix_ref.iter_mut().enumerate() {
            if col_idx > row.len() {
                return Err(RuntimeError::IndexOutOfBounds(col_idx));
            }
            let val = col_values.get(i).cloned().unwrap_or(Value::Na);
            row.insert(col_idx, val);
        }

        Ok(Value::Na)
    }
}

/// matrix.transpose() - Transposes the matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.transpose")]
struct MatrixTranspose {
    id: Value,
}

impl MatrixTranspose {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let matrix_ref = matrix.borrow();
        if matrix_ref.is_empty() {
            return Ok(Value::Matrix {
                element_type: "float".to_string(),
                data: Rc::new(RefCell::new(vec![])),
            });
        }

        let rows = matrix_ref.len();
        let cols = matrix_ref[0].len();
        let mut transposed = vec![vec![Value::Na; rows]; cols];

        for i in 0..rows {
            for j in 0..cols {
                transposed[j][i] = matrix_ref[i][j].clone();
            }
        }

        Ok(Value::Matrix {
            element_type: "float".to_string(),
            data: Rc::new(RefCell::new(transposed)),
        })
    }
}

/// Register the matrix namespace with all functions
pub fn register() -> Value {
    let mut members = HashMap::new();

    members.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixNew::builtin_fn)),
    );
    members.insert(
        "get".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixGet::builtin_fn)),
    );
    members.insert(
        "set".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixSet::builtin_fn)),
    );
    members.insert(
        "rows".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixRows::builtin_fn)),
    );
    members.insert(
        "columns".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixColumns::builtin_fn)),
    );
    members.insert(
        "elements_count".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixElementsCount::builtin_fn)),
    );
    members.insert(
        "fill".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixFill::builtin_fn)),
    );
    members.insert(
        "copy".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixCopy::builtin_fn)),
    );
    members.insert(
        "add_row".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixAddRow::builtin_fn)),
    );
    members.insert(
        "add_col".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixAddCol::builtin_fn)),
    );
    members.insert(
        "transpose".to_string(),
        Value::BuiltinFunction(Rc::new(MatrixTranspose::builtin_fn)),
    );

    Value::Object {
        type_name: "matrix".to_string(),
        fields: Rc::new(RefCell::new(members)),
    }
}
