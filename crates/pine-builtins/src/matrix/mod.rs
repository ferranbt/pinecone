use nalgebra::DMatrix;
use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// matrix.new<type>() - Creates a new typed matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.new", type_params = 1)]
struct MatrixNew<O: PineOutput> {
    #[type_param]
    element_type: String,
    #[arg(default = 0.0)]
    rows: f64,
    #[arg(default = 0.0)]
    columns: f64,
    #[arg(default = Value::Na)]
    initial_value: Value<O>,
}

impl<O: PineOutput> MatrixNew<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let rows = self.rows as usize;
        let columns = self.columns as usize;

        // A built-in element type, or any declared user-defined type.
        let is_builtin = matches!(
            self.element_type.as_str(),
            "int" | "float" | "string" | "bool"
        );
        if !is_builtin && !ctx.is_user_type(&self.element_type) {
            return Err(RuntimeError::TypeError(format!(
                "Invalid matrix element type '{}'. Must be a built-in type or a user-defined type",
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
struct MatrixGet<O: PineOutput> {
    id: Value<O>,
    row: f64,
    column: f64,
}

impl<O: PineOutput> MatrixGet<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let row_idx = self.row as usize;
        let col_idx = self.column as usize;

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
struct MatrixSet<O: PineOutput> {
    id: Value<O>,
    row: f64,
    column: f64,
    value: Value<O>,
}

impl<O: PineOutput> MatrixSet<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let row_idx = self.row as usize;
        let col_idx = self.column as usize;

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
struct MatrixRows<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixRows<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct MatrixColumns<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixColumns<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct MatrixElementsCount<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixElementsCount<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct MatrixFill<O: PineOutput> {
    id: Value<O>,
    value: Value<O>,
}

impl<O: PineOutput> MatrixFill<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct MatrixCopy<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixCopy<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let (matrix, element_type) = match &self.id {
            Value::Matrix { data, element_type } => (data, element_type.clone()),
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let matrix_ref = matrix.borrow();
        let copied_data = matrix_ref.clone();
        Ok(Value::Matrix {
            element_type,
            data: Rc::new(RefCell::new(copied_data)),
        })
    }
}

/// matrix.add_row() - Adds a row to the matrix
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.add_row")]
struct MatrixAddRow<O: PineOutput> {
    id: Value<O>,
    row: f64,
    #[arg(default = Value::Na)]
    array_id: Value<O>,
}

impl<O: PineOutput> MatrixAddRow<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let row_idx = self.row as usize;

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
struct MatrixAddCol<O: PineOutput> {
    id: Value<O>,
    column: f64,
    #[arg(default = Value::Na)]
    array_id: Value<O>,
}

impl<O: PineOutput> MatrixAddCol<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let matrix = match &self.id {
            Value::Matrix { data, .. } => data,
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let col_idx = self.column as usize;

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
struct MatrixTranspose<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixTranspose<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let (matrix, element_type) = match &self.id {
            Value::Matrix { data, element_type } => (data, element_type.clone()),
            _ => return Err(RuntimeError::TypeError("Expected matrix".to_string())),
        };

        let matrix_ref = matrix.borrow();
        if matrix_ref.is_empty() {
            return Ok(Value::Matrix {
                element_type,
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
            element_type,
            data: Rc::new(RefCell::new(transposed)),
        })
    }
}

type MatrixData<O> = Rc<RefCell<Vec<Vec<Value<O>>>>>;

fn as_matrix<O: PineOutput>(v: &Value<O>) -> Result<&MatrixData<O>, RuntimeError> {
    match v {
        Value::Matrix { data, .. } => Ok(data),
        _ => Err(RuntimeError::TypeError("Expected matrix".to_string())),
    }
}

fn numeric<O: PineOutput>(v: &Value<O>) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Number(n) if n.is_finite() => Some(*n),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// The finite elements of a matrix, row-major.
fn all_numbers<O: PineOutput>(m: &Value<O>) -> Result<Vec<f64>, RuntimeError> {
    Ok(as_matrix(m)?
        .borrow()
        .iter()
        .flatten()
        .filter_map(numeric)
        .collect())
}

/// A matrix as an `f64` grid (`na` → `NaN`), for the shape predicates.
fn grid<O: PineOutput>(m: &[Vec<Value<O>>]) -> Vec<Vec<f64>> {
    m.iter()
        .map(|r| r.iter().map(|v| numeric(v).unwrap_or(f64::NAN)).collect())
        .collect()
}

fn mean(ns: &[f64]) -> Option<f64> {
    (!ns.is_empty()).then(|| ns.iter().sum::<f64>() / ns.len() as f64)
}

fn median(mut ns: Vec<f64>) -> Option<f64> {
    if ns.is_empty() {
        return None;
    }
    ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = ns.len() / 2;
    Some(if ns.len().is_multiple_of(2) {
        (ns[m - 1] + ns[m]) / 2.0
    } else {
        ns[m]
    })
}

fn mode(mut ns: Vec<f64>) -> Option<f64> {
    if ns.is_empty() {
        return None;
    }
    ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (mut best, mut best_count) = (ns[0], 0usize);
    let mut i = 0;
    while i < ns.len() {
        let j = ns[i..].partition_point(|&x| x == ns[i]) + i;
        if j - i > best_count {
            best_count = j - i;
            best = ns[i];
        }
        i = j;
    }
    Some(best)
}

fn is_square(g: &[Vec<f64>]) -> bool {
    let r = g.len();
    r > 0 && g.iter().all(|row| row.len() == r)
}
fn is_zero(g: &[Vec<f64>]) -> bool {
    g.iter().flatten().all(|&x| x == 0.0)
}
fn is_binary(g: &[Vec<f64>]) -> bool {
    g.iter().flatten().all(|&x| x == 0.0 || x == 1.0)
}
fn is_diagonal(g: &[Vec<f64>]) -> bool {
    is_square(g) && (0..g.len()).all(|i| (0..g.len()).all(|j| i == j || g[i][j] == 0.0))
}
fn is_identity(g: &[Vec<f64>]) -> bool {
    is_square(g)
        && (0..g.len()).all(|i| (0..g.len()).all(|j| g[i][j] == if i == j { 1.0 } else { 0.0 }))
}
fn is_symmetric(g: &[Vec<f64>]) -> bool {
    is_square(g) && (0..g.len()).all(|i| (0..g.len()).all(|j| g[i][j] == g[j][i]))
}
fn is_antisymmetric(g: &[Vec<f64>]) -> bool {
    is_square(g) && (0..g.len()).all(|i| (0..g.len()).all(|j| g[i][j] == -g[j][i]))
}
fn is_antidiagonal(g: &[Vec<f64>]) -> bool {
    let n = g.len();
    is_square(g) && (0..n).all(|i| (0..n).all(|j| j == n - 1 - i || g[i][j] == 0.0))
}
fn is_triangular(g: &[Vec<f64>]) -> bool {
    let n = g.len();
    let upper = (0..n).all(|i| (0..n).all(|j| i <= j || g[i][j] == 0.0));
    let lower = (0..n).all(|i| (0..n).all(|j| i >= j || g[i][j] == 0.0));
    is_square(g) && (upper || lower)
}
fn is_stochastic(g: &[Vec<f64>]) -> bool {
    is_square(g)
        && g.iter().all(|row| {
            row.iter().all(|&x| x >= 0.0) && (row.iter().sum::<f64>() - 1.0).abs() < 1e-9
        })
}

macro_rules! matrix_reduce {
    ($name:literal, $ident:ident, $reduce:expr) => {
        #[derive(BuiltinFunction)]
        #[builtin(name = $name)]
        struct $ident<O: PineOutput> {
            id: Value<O>,
        }
        impl<O: PineOutput> $ident<O> {
            fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
                Ok(($reduce)(all_numbers(&self.id)?).map_or(Value::Na, Value::Number))
            }
        }
    };
}

matrix_reduce!("matrix.avg", MatrixAvg, |ns: Vec<f64>| mean(&ns));
matrix_reduce!("matrix.min", MatrixMin, |ns: Vec<f64>| ns
    .into_iter()
    .reduce(f64::min));
matrix_reduce!("matrix.max", MatrixMax, |ns: Vec<f64>| ns
    .into_iter()
    .reduce(f64::max));
matrix_reduce!("matrix.median", MatrixMedian, median);
matrix_reduce!("matrix.mode", MatrixMode, mode);

macro_rules! matrix_predicate {
    ($name:literal, $ident:ident, $check:ident) => {
        #[derive(BuiltinFunction)]
        #[builtin(name = $name)]
        struct $ident<O: PineOutput> {
            id: Value<O>,
        }
        impl<O: PineOutput> $ident<O> {
            fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
                Ok(Value::Bool($check(&grid(&as_matrix(&self.id)?.borrow()))))
            }
        }
    };
}

matrix_predicate!("matrix.is_square", MatrixIsSquare, is_square);
matrix_predicate!("matrix.is_zero", MatrixIsZero, is_zero);
matrix_predicate!("matrix.is_binary", MatrixIsBinary, is_binary);
matrix_predicate!("matrix.is_diagonal", MatrixIsDiagonal, is_diagonal);
matrix_predicate!("matrix.is_identity", MatrixIsIdentity, is_identity);
matrix_predicate!("matrix.is_symmetric", MatrixIsSymmetric, is_symmetric);
matrix_predicate!(
    "matrix.is_antisymmetric",
    MatrixIsAntisymmetric,
    is_antisymmetric
);
matrix_predicate!(
    "matrix.is_antidiagonal",
    MatrixIsAntidiagonal,
    is_antidiagonal
);
matrix_predicate!("matrix.is_triangular", MatrixIsTriangular, is_triangular);
matrix_predicate!("matrix.is_stochastic", MatrixIsStochastic, is_stochastic);

/// matrix.trace(id) - Sum of the main-diagonal elements.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.trace")]
struct MatrixTrace<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixTrace<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let m = as_matrix(&self.id)?.borrow();
        let n = m.len().min(m.first().map_or(0, |r| r.len()));
        let trace: f64 = (0..n).filter_map(|i| numeric(&m[i][i])).sum();
        Ok(Value::Number(trace))
    }
}

/// matrix.row(id, row) - The row as an array.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.row")]
struct MatrixRow<O: PineOutput> {
    id: Value<O>,
    row: f64,
}

impl<O: PineOutput> MatrixRow<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let m = as_matrix(&self.id)?.borrow();
        let row = self.row as usize;
        let out = m
            .get(row)
            .cloned()
            .ok_or(RuntimeError::IndexOutOfBounds(row))?;
        Ok(Value::Array(Rc::new(RefCell::new(out))))
    }
}

/// matrix.col(id, column) - The column as an array.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.col")]
struct MatrixCol<O: PineOutput> {
    id: Value<O>,
    column: f64,
}

impl<O: PineOutput> MatrixCol<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let m = as_matrix(&self.id)?.borrow();
        let col = self.column as usize;
        let out: Vec<Value<O>> = m
            .iter()
            .map(|r| {
                r.get(col)
                    .cloned()
                    .ok_or(RuntimeError::IndexOutOfBounds(col))
            })
            .collect::<Result<_, _>>()?;
        Ok(Value::Array(Rc::new(RefCell::new(out))))
    }
}

/// matrix.reverse(id) - Reverse the order of both rows and columns, in place.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.reverse")]
struct MatrixReverse<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixReverse<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut m = as_matrix(&self.id)?.borrow_mut();
        m.reverse();
        for row in m.iter_mut() {
            row.reverse();
        }
        Ok(Value::Na)
    }
}

/// matrix.swap_rows(id, row1, row2) - Swap two rows, in place.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.swap_rows")]
struct MatrixSwapRows<O: PineOutput> {
    id: Value<O>,
    row1: f64,
    row2: f64,
}

impl<O: PineOutput> MatrixSwapRows<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut m = as_matrix(&self.id)?.borrow_mut();
        let (r1, r2) = (self.row1 as usize, self.row2 as usize);
        if r1 >= m.len() || r2 >= m.len() {
            return Err(RuntimeError::IndexOutOfBounds(r1.max(r2)));
        }
        m.swap(r1, r2);
        Ok(Value::Na)
    }
}

/// matrix.swap_columns(id, column1, column2) - Swap two columns, in place.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.swap_columns")]
struct MatrixSwapColumns<O: PineOutput> {
    id: Value<O>,
    column1: f64,
    column2: f64,
}

impl<O: PineOutput> MatrixSwapColumns<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut m = as_matrix(&self.id)?.borrow_mut();
        let (c1, c2) = (self.column1 as usize, self.column2 as usize);
        for row in m.iter_mut() {
            if c1 >= row.len() || c2 >= row.len() {
                return Err(RuntimeError::IndexOutOfBounds(c1.max(c2)));
            }
            row.swap(c1, c2);
        }
        Ok(Value::Na)
    }
}

/// matrix.remove_row(id, row) - Remove and return the row as an array.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.remove_row")]
struct MatrixRemoveRow<O: PineOutput> {
    id: Value<O>,
    row: f64,
}

impl<O: PineOutput> MatrixRemoveRow<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut m = as_matrix(&self.id)?.borrow_mut();
        let row = self.row as usize;
        if row >= m.len() {
            return Err(RuntimeError::IndexOutOfBounds(row));
        }
        Ok(Value::Array(Rc::new(RefCell::new(m.remove(row)))))
    }
}

/// matrix.remove_col(id, column) - Remove and return the column as an array.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.remove_col")]
struct MatrixRemoveCol<O: PineOutput> {
    id: Value<O>,
    column: f64,
}

impl<O: PineOutput> MatrixRemoveCol<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut m = as_matrix(&self.id)?.borrow_mut();
        let col = self.column as usize;
        let mut removed = Vec::with_capacity(m.len());
        for row in m.iter_mut() {
            if col >= row.len() {
                return Err(RuntimeError::IndexOutOfBounds(col));
            }
            removed.push(row.remove(col));
        }
        Ok(Value::Array(Rc::new(RefCell::new(removed))))
    }
}

fn element_type<O: PineOutput>(v: &Value<O>) -> String {
    match v {
        Value::Matrix { element_type, .. } => element_type.clone(),
        _ => "float".to_string(),
    }
}

/// Build a matrix value from an `f64` grid.
fn from_grid<O: PineOutput>(g: Vec<Vec<f64>>, element_type: &str) -> Value<O> {
    let data = g
        .into_iter()
        .map(|row| row.into_iter().map(Value::Number).collect())
        .collect();
    Value::Matrix {
        element_type: element_type.to_string(),
        data: Rc::new(RefCell::new(data)),
    }
}

/// Element-wise `op` of a matrix with another matrix or a scalar.
fn broadcast<O: PineOutput>(
    a_id: &Value<O>,
    b: &Value<O>,
    op: fn(f64, f64) -> f64,
) -> Result<Value<O>, RuntimeError> {
    let a = as_matrix(a_id)?.borrow();
    let cell = |p: &Value<O>, q: &Value<O>| match (numeric(p), numeric(q)) {
        (Some(x), Some(y)) => Value::Number(op(x, y)),
        _ => Value::Na,
    };
    let out: Vec<Vec<Value<O>>> = match b {
        Value::Matrix { data, .. } => {
            let b = data.borrow();
            if a.len() != b.len() || a.iter().zip(b.iter()).any(|(x, y)| x.len() != y.len()) {
                return Err(RuntimeError::TypeError(
                    "matrix op: dimensions do not match".to_string(),
                ));
            }
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| x.iter().zip(y.iter()).map(|(p, q)| cell(p, q)).collect())
                .collect()
        }
        scalar => {
            let s = numeric(scalar).ok_or_else(|| {
                RuntimeError::TypeError("matrix op: expected a matrix or number".to_string())
            })?;
            let s = Value::Number(s);
            a.iter()
                .map(|row| row.iter().map(|p| cell(p, &s)).collect())
                .collect()
        }
    };
    Ok(Value::Matrix {
        element_type: element_type(a_id),
        data: Rc::new(RefCell::new(out)),
    })
}

/// Determinant by Gaussian elimination with partial pivoting.
#[allow(clippy::needless_range_loop)]
fn determinant(mut a: Vec<Vec<f64>>) -> f64 {
    let n = a.len();
    let mut det = 1.0;
    for i in 0..n {
        let pivot = (i..n)
            .max_by(|&x, &y| a[x][i].abs().total_cmp(&a[y][i].abs()))
            .unwrap();
        if a[pivot][i] == 0.0 {
            return 0.0;
        }
        if pivot != i {
            a.swap(i, pivot);
            det = -det;
        }
        det *= a[i][i];
        for r in i + 1..n {
            let factor = a[r][i] / a[i][i];
            for c in i..n {
                a[r][c] -= factor * a[i][c];
            }
        }
    }
    det
}

/// Inverse of a square matrix, or `None` when singular.
fn inverse(a: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    to_dmatrix(a).try_inverse().map(|m| from_dmatrix(&m))
}

/// Rank of the matrix.
fn matrix_rank(a: &[Vec<f64>]) -> usize {
    if a.is_empty() {
        return 0;
    }
    to_dmatrix(a).rank(1e-12)
}

/// Convert an `f64` grid into an `nalgebra` dynamic matrix.
fn to_dmatrix(g: &[Vec<f64>]) -> DMatrix<f64> {
    let (r, c) = (g.len(), g.first().map_or(0, |x| x.len()));
    DMatrix::from_fn(r, c, |i, j| g[i][j])
}

/// Convert an `nalgebra` dynamic matrix back into an `f64` grid.
fn from_dmatrix(m: &DMatrix<f64>) -> Vec<Vec<f64>> {
    (0..m.nrows())
        .map(|i| (0..m.ncols()).map(|j| m[(i, j)]).collect())
        .collect()
}

/// matrix.sum(id1, id2) - Matrix addition; `id2` may be a matrix or a scalar.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.sum")]
struct MatrixSum<O: PineOutput> {
    id1: Value<O>,
    id2: Value<O>,
}

impl<O: PineOutput> MatrixSum<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        broadcast(&self.id1, &self.id2, |a, b| a + b)
    }
}

/// matrix.diff(id1, id2) - Matrix subtraction; `id2` may be a matrix or a scalar.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.diff")]
struct MatrixDiff<O: PineOutput> {
    id1: Value<O>,
    id2: Value<O>,
}

impl<O: PineOutput> MatrixDiff<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        broadcast(&self.id1, &self.id2, |a, b| a - b)
    }
}

/// matrix.mult(id1, id2) - Product with a matrix, a scalar, or a vector (array).
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.mult")]
struct MatrixMult<O: PineOutput> {
    id1: Value<O>,
    id2: Value<O>,
}

impl<O: PineOutput> MatrixMult<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id1)?.borrow());
        let (m, n) = (a.len(), a.first().map_or(0, |r| r.len()));
        match &self.id2 {
            Value::Matrix { data, .. } => {
                let b = grid(&data.borrow());
                let (p, q) = (b.len(), b.first().map_or(0, |r| r.len()));
                if n != p {
                    return Err(RuntimeError::TypeError(
                        "matrix.mult: inner dimensions do not match".to_string(),
                    ));
                }
                let out = (0..m)
                    .map(|i| {
                        (0..q)
                            .map(|j| (0..n).map(|k| a[i][k] * b[k][j]).sum())
                            .collect()
                    })
                    .collect();
                Ok(from_grid(out, "float"))
            }
            Value::Array(arr) => {
                let v: Vec<f64> = arr.borrow().iter().filter_map(numeric).collect();
                if v.len() != n {
                    return Err(RuntimeError::TypeError(
                        "matrix.mult: vector length does not match".to_string(),
                    ));
                }
                let out = a
                    .iter()
                    .map(|row| Value::Number(row.iter().zip(&v).map(|(x, y)| x * y).sum()))
                    .collect();
                Ok(Value::Array(Rc::new(RefCell::new(out))))
            }
            scalar => {
                let s = numeric(scalar).ok_or_else(|| {
                    RuntimeError::TypeError(
                        "matrix.mult: expected a matrix, number or array".into(),
                    )
                })?;
                let out = a
                    .iter()
                    .map(|row| row.iter().map(|x| x * s).collect())
                    .collect();
                Ok(from_grid(out, "float"))
            }
        }
    }
}

/// matrix.kron(id1, id2) - Kronecker product.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.kron")]
struct MatrixKron<O: PineOutput> {
    id1: Value<O>,
    id2: Value<O>,
}

impl<O: PineOutput> MatrixKron<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id1)?.borrow());
        let b = grid(&as_matrix(&self.id2)?.borrow());
        let (ar, ac) = (a.len(), a.first().map_or(0, |r| r.len()));
        let (br, bc) = (b.len(), b.first().map_or(0, |r| r.len()));
        let mut out = vec![vec![0.0; ac * bc]; ar * br];
        for i in 0..ar {
            for j in 0..ac {
                for k in 0..br {
                    for l in 0..bc {
                        out[i * br + k][j * bc + l] = a[i][j] * b[k][l];
                    }
                }
            }
        }
        Ok(from_grid(out, "float"))
    }
}

/// matrix.pow(id, power) - A square matrix raised to a non-negative power.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.pow")]
struct MatrixPow<O: PineOutput> {
    id: Value<O>,
    power: f64,
}

impl<O: PineOutput> MatrixPow<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id)?.borrow());
        let n = a.len();
        if !is_square(&a) {
            return Err(RuntimeError::TypeError(
                "matrix.pow: matrix is not square".into(),
            ));
        }
        // Start from the identity and multiply `power` times.
        let mut result: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();
        for _ in 0..self.power.max(0.0) as usize {
            result = mat_mul(&result, &a);
        }
        Ok(from_grid(result, "float"))
    }
}

/// Square matrix product `a * b`.
fn mat_mul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| (0..n).map(|k| a[i][k] * b[k][j]).sum())
                .collect()
        })
        .collect()
}

/// matrix.det(id) - Determinant of a square matrix.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.det")]
struct MatrixDet<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixDet<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id)?.borrow());
        if !is_square(&a) {
            return Ok(Value::Na);
        }
        Ok(Value::Number(determinant(a)))
    }
}

/// matrix.inv(id) - Inverse of a square matrix, na-filled when singular.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.inv")]
struct MatrixInv<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixInv<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id)?.borrow());
        if !is_square(&a) {
            return Err(RuntimeError::TypeError(
                "matrix.inv: matrix is not square".into(),
            ));
        }
        let n = a.len();
        match inverse(&a) {
            Some(inv) => Ok(from_grid(inv, "float")),
            None => Ok(from_grid(vec![vec![f64::NAN; n]; n], "float")),
        }
    }
}

/// matrix.rank(id) - The rank of the matrix.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.rank")]
struct MatrixRank<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixRank<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Int(
            matrix_rank(&grid(&as_matrix(&self.id)?.borrow())) as i64,
        ))
    }
}

/// matrix.eigenvalues(id) - Eigenvalues of a symmetric matrix (Implicit QL).
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.eigenvalues")]
struct MatrixEigenvalues<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixEigenvalues<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id)?.borrow());
        let out = if is_square(&a) && !a.is_empty() {
            to_dmatrix(&a)
                .symmetric_eigen()
                .eigenvalues
                .iter()
                .copied()
                .map(Value::Number)
                .collect()
        } else {
            vec![]
        };
        Ok(Value::Array(Rc::new(RefCell::new(out))))
    }
}

/// matrix.eigenvectors(id) - Eigenvectors of a symmetric matrix, one per column.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.eigenvectors")]
struct MatrixEigenvectors<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixEigenvectors<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id)?.borrow());
        let out = if is_square(&a) && !a.is_empty() {
            from_dmatrix(&to_dmatrix(&a).symmetric_eigen().eigenvectors)
        } else {
            vec![]
        };
        Ok(from_grid(out, "float"))
    }
}

/// matrix.pinv(id) - Moore-Penrose pseudoinverse; `na`-filled when it fails.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.pinv")]
struct MatrixPinv<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MatrixPinv<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = grid(&as_matrix(&self.id)?.borrow());
        let (rows, cols) = (a.len(), a.first().map_or(0, |r| r.len()));
        let out = to_dmatrix(&a)
            .pseudo_inverse(1e-15)
            .map(|m| from_dmatrix(&m))
            .unwrap_or_else(|_| vec![vec![f64::NAN; rows]; cols]);
        Ok(from_grid(out, "float"))
    }
}

/// matrix.concat(id1, id2) - Append `id2`'s rows to `id1`, returning `id1`.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.concat")]
struct MatrixConcat<O: PineOutput> {
    id1: Value<O>,
    id2: Value<O>,
}

impl<O: PineOutput> MatrixConcat<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let tail = as_matrix(&self.id2)?.borrow().clone();
        let mut a = as_matrix(&self.id1)?.borrow_mut();
        let cols = a.first().map_or(0, |r| r.len());
        if tail.iter().any(|r| r.len() != cols) {
            return Err(RuntimeError::TypeError(
                "matrix.concat: column counts differ".to_string(),
            ));
        }
        a.extend(tail);
        drop(a);
        Ok(self.id1.clone())
    }
}

/// matrix.reshape(id, rows, columns) - Rearrange the elements row-major, in place.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.reshape")]
struct MatrixReshape<O: PineOutput> {
    id: Value<O>,
    rows: f64,
    columns: f64,
}

impl<O: PineOutput> MatrixReshape<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut m = as_matrix(&self.id)?.borrow_mut();
        let (rows, cols) = (self.rows as usize, self.columns as usize);
        let flat: Vec<Value<O>> = m.iter().flatten().cloned().collect();
        if flat.len() != rows * cols {
            return Err(RuntimeError::TypeError(
                "matrix.reshape: element count does not match".to_string(),
            ));
        }
        *m = flat.chunks(cols).map(<[Value<O>]>::to_vec).collect();
        Ok(Value::Na)
    }
}

/// matrix.submatrix(id, from_row, to_row, from_column, to_column) - Extract a
/// range of rows and columns (`to` exclusive).
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.submatrix")]
struct MatrixSubmatrix<O: PineOutput> {
    id: Value<O>,
    from_row: f64,
    to_row: f64,
    from_column: f64,
    to_column: f64,
}

impl<O: PineOutput> MatrixSubmatrix<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let m = as_matrix(&self.id)?.borrow();
        let (fr, tr) = (self.from_row as usize, self.to_row as usize);
        let (fc, tc) = (self.from_column as usize, self.to_column as usize);
        let out: Vec<Vec<Value<O>>> = m
            .get(fr..tr.min(m.len()))
            .unwrap_or(&[])
            .iter()
            .map(|row| row[fc.min(row.len())..tc.min(row.len())].to_vec())
            .collect();
        Ok(Value::Matrix {
            element_type: element_type(&self.id),
            data: Rc::new(RefCell::new(out)),
        })
    }
}

/// matrix.sort(id, column, order) - Sort rows by a column's values, in place.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.sort")]
struct MatrixSort<O: PineOutput> {
    id: Value<O>,
    #[arg(default = 0.0)]
    column: f64,
    #[arg(default = "ascending")]
    order: String,
}

impl<O: PineOutput> MatrixSort<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut m = as_matrix(&self.id)?.borrow_mut();
        let col = self.column as usize;
        let key = |row: &Vec<Value<O>>| row.get(col).and_then(numeric).unwrap_or(f64::NAN);
        m.sort_by(|a, b| key(a).total_cmp(&key(b)));
        if self.order == "descending" {
            m.reverse();
        }
        Ok(Value::Na)
    }
}

/// Register the matrix namespace with all functions
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert("new".to_string(), MatrixNew::<O>::builtin_value());
    members.insert("get".to_string(), MatrixGet::<O>::builtin_value());
    members.insert("set".to_string(), MatrixSet::<O>::builtin_value());
    members.insert("rows".to_string(), MatrixRows::<O>::builtin_value());
    members.insert("columns".to_string(), MatrixColumns::<O>::builtin_value());
    members.insert(
        "elements_count".to_string(),
        MatrixElementsCount::<O>::builtin_value(),
    );
    members.insert("fill".to_string(), MatrixFill::<O>::builtin_value());
    members.insert("copy".to_string(), MatrixCopy::<O>::builtin_value());
    members.insert("add_row".to_string(), MatrixAddRow::<O>::builtin_value());
    members.insert("add_col".to_string(), MatrixAddCol::<O>::builtin_value());
    members.insert(
        "transpose".to_string(),
        MatrixTranspose::<O>::builtin_value(),
    );
    members.insert("avg".to_string(), MatrixAvg::<O>::builtin_value());
    members.insert("min".to_string(), MatrixMin::<O>::builtin_value());
    members.insert("max".to_string(), MatrixMax::<O>::builtin_value());
    members.insert("median".to_string(), MatrixMedian::<O>::builtin_value());
    members.insert("mode".to_string(), MatrixMode::<O>::builtin_value());
    members.insert("trace".to_string(), MatrixTrace::<O>::builtin_value());
    members.insert("sum".to_string(), MatrixSum::<O>::builtin_value());
    members.insert("row".to_string(), MatrixRow::<O>::builtin_value());
    members.insert("col".to_string(), MatrixCol::<O>::builtin_value());
    members.insert("reverse".to_string(), MatrixReverse::<O>::builtin_value());
    members.insert(
        "swap_rows".to_string(),
        MatrixSwapRows::<O>::builtin_value(),
    );
    members.insert(
        "swap_columns".to_string(),
        MatrixSwapColumns::<O>::builtin_value(),
    );
    members.insert(
        "remove_row".to_string(),
        MatrixRemoveRow::<O>::builtin_value(),
    );
    members.insert(
        "remove_col".to_string(),
        MatrixRemoveCol::<O>::builtin_value(),
    );
    members.insert(
        "is_square".to_string(),
        MatrixIsSquare::<O>::builtin_value(),
    );
    members.insert("is_zero".to_string(), MatrixIsZero::<O>::builtin_value());
    members.insert(
        "is_binary".to_string(),
        MatrixIsBinary::<O>::builtin_value(),
    );
    members.insert(
        "is_diagonal".to_string(),
        MatrixIsDiagonal::<O>::builtin_value(),
    );
    members.insert(
        "is_identity".to_string(),
        MatrixIsIdentity::<O>::builtin_value(),
    );
    members.insert(
        "is_symmetric".to_string(),
        MatrixIsSymmetric::<O>::builtin_value(),
    );
    members.insert(
        "is_antisymmetric".to_string(),
        MatrixIsAntisymmetric::<O>::builtin_value(),
    );
    members.insert(
        "is_antidiagonal".to_string(),
        MatrixIsAntidiagonal::<O>::builtin_value(),
    );
    members.insert(
        "is_triangular".to_string(),
        MatrixIsTriangular::<O>::builtin_value(),
    );
    members.insert(
        "is_stochastic".to_string(),
        MatrixIsStochastic::<O>::builtin_value(),
    );
    members.insert("diff".to_string(), MatrixDiff::<O>::builtin_value());
    members.insert("mult".to_string(), MatrixMult::<O>::builtin_value());
    members.insert("kron".to_string(), MatrixKron::<O>::builtin_value());
    members.insert("pow".to_string(), MatrixPow::<O>::builtin_value());
    members.insert("det".to_string(), MatrixDet::<O>::builtin_value());
    members.insert("inv".to_string(), MatrixInv::<O>::builtin_value());
    members.insert("rank".to_string(), MatrixRank::<O>::builtin_value());
    members.insert(
        "eigenvalues".to_string(),
        MatrixEigenvalues::<O>::builtin_value(),
    );
    members.insert(
        "eigenvectors".to_string(),
        MatrixEigenvectors::<O>::builtin_value(),
    );
    members.insert("pinv".to_string(), MatrixPinv::<O>::builtin_value());
    members.insert("concat".to_string(), MatrixConcat::<O>::builtin_value());
    members.insert("reshape".to_string(), MatrixReshape::<O>::builtin_value());
    members.insert(
        "submatrix".to_string(),
        MatrixSubmatrix::<O>::builtin_value(),
    );
    members.insert("sort".to_string(), MatrixSort::<O>::builtin_value());

    Value::Object {
        type_name: "matrix".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
