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
        && g.iter()
            .all(|row| row.iter().all(|&x| x >= 0.0) && (row.iter().sum::<f64>() - 1.0).abs() < 1e-9)
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
        let out = m.get(row).cloned().ok_or(RuntimeError::IndexOutOfBounds(row))?;
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
            .map(|r| r.get(col).cloned().ok_or(RuntimeError::IndexOutOfBounds(col)))
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

/// matrix.sum(id1, id2) - Matrix addition; `id2` may be a matrix or a scalar.
#[derive(BuiltinFunction)]
#[builtin(name = "matrix.sum")]
struct MatrixSum<O: PineOutput> {
    id1: Value<O>,
    id2: Value<O>,
}

impl<O: PineOutput> MatrixSum<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a = as_matrix(&self.id1)?.borrow();
        let element_type = match &self.id1 {
            Value::Matrix { element_type, .. } => element_type.clone(),
            _ => "float".to_string(),
        };
        let out: Vec<Vec<Value<O>>> = match &self.id2 {
            Value::Matrix { data, .. } => {
                let b = data.borrow();
                if a.len() != b.len() || a.iter().zip(b.iter()).any(|(x, y)| x.len() != y.len()) {
                    return Err(RuntimeError::TypeError(
                        "matrix.sum: dimensions do not match".to_string(),
                    ));
                }
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| {
                        x.iter()
                            .zip(y.iter())
                            .map(|(p, q)| add(p, q))
                            .collect()
                    })
                    .collect()
            }
            scalar => {
                let s = numeric(scalar).ok_or_else(|| {
                    RuntimeError::TypeError("matrix.sum: expected a matrix or number".to_string())
                })?;
                a.iter()
                    .map(|row| row.iter().map(|p| add(p, &Value::Number(s))).collect())
                    .collect()
            }
        };
        Ok(Value::Matrix {
            element_type,
            data: Rc::new(RefCell::new(out)),
        })
    }
}

/// Element-wise sum of two values, `na` when either is non-numeric.
fn add<O: PineOutput>(a: &Value<O>, b: &Value<O>) -> Value<O> {
    match (numeric(a), numeric(b)) {
        (Some(x), Some(y)) => Value::Number(x + y),
        _ => Value::Na,
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
    members.insert("is_square".to_string(), MatrixIsSquare::<O>::builtin_value());
    members.insert("is_zero".to_string(), MatrixIsZero::<O>::builtin_value());
    members.insert("is_binary".to_string(), MatrixIsBinary::<O>::builtin_value());
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

    Value::Object {
        type_name: "matrix".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
