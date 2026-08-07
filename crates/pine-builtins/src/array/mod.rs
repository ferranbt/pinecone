use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// array.new<type>() - Creates a new typed array (generic version)
#[derive(BuiltinFunction)]
#[builtin(name = "array.new", type_params = 1)]
struct ArrayNew<O: PineOutput> {
    #[type_param]
    element_type: String,
    #[arg(default = 0.0)]
    size: f64,
    #[arg(default = Value::Na)]
    initial_value: Value<O>,
}

impl<O: PineOutput> ArrayNew<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // A built-in element type, or any declared user-defined type.
        let is_builtin = matches!(
            self.element_type.as_str(),
            "int" | "float" | "string" | "bool" | "color"
        );
        if !is_builtin && !ctx.is_user_type(&self.element_type) {
            return Err(RuntimeError::TypeError(format!(
                "Invalid array element type '{}'. Must be a built-in type or a user-defined type",
                self.element_type
            )));
        }

        let size = self.size as usize;
        let arr = vec![self.initial_value.clone(); size];
        Ok(Value::Array(Rc::new(RefCell::new(arr))))
    }
}

/// array.new_float() - Creates a new float array (backward compatibility)
#[derive(BuiltinFunction)]
#[builtin(name = "array.new_float")]
struct ArrayNewFloat<O: PineOutput> {
    size: f64,
    initial_value: Value<O>,
}

impl<O: PineOutput> ArrayNewFloat<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let size = self.size as usize;
        let arr = vec![self.initial_value.clone(); size];
        Ok(Value::Array(Rc::new(RefCell::new(arr))))
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.clear")]
struct ArrayClear<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayClear<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?;
        arr.borrow_mut().clear();
        Ok(Value::Na)
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.push")]
struct ArrayPush<O: PineOutput> {
    array: Value<O>,
    value: Value<O>,
}

impl<O: PineOutput> ArrayPush<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?;
        arr.borrow_mut().push(self.value.clone());
        Ok(Value::Na)
    }
}

/// array.unshift() - Inserts a value at the front of the array.
#[derive(BuiltinFunction)]
#[builtin(name = "array.unshift")]
struct ArrayUnshift<O: PineOutput> {
    array: Value<O>,
    value: Value<O>,
}

impl<O: PineOutput> ArrayUnshift<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?;
        arr.borrow_mut().insert(0, self.value.clone());
        Ok(Value::Na)
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.get")]
struct ArrayGet<O: PineOutput> {
    array: Value<O>,
    index: f64,
}

impl<O: PineOutput> ArrayGet<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?;
        let index = self.index as usize;
        arr.borrow()
            .get(index)
            .cloned()
            .ok_or(RuntimeError::IndexOutOfBounds(index))
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.size")]
struct ArraySize<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArraySize<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?;
        let size = arr.borrow().len();
        Ok(Value::Number(size as f64))
    }
}

/// array.new_int(size, initial_value) - Creates a new int array.
#[derive(BuiltinFunction)]
#[builtin(name = "array.new_int")]
struct ArrayNewInt<O: PineOutput> {
    #[arg(default = 0.0)]
    size: f64,
    #[arg(default = Value::Na)]
    initial_value: Value<O>,
}

impl<O: PineOutput> ArrayNewInt<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = vec![self.initial_value.clone(); self.size as usize];
        Ok(Value::Array(Rc::new(RefCell::new(arr))))
    }
}

/// array.new_string(size, initial_value) - Creates a new string array.
#[derive(BuiltinFunction)]
#[builtin(name = "array.new_string")]
struct ArrayNewString<O: PineOutput> {
    #[arg(default = 0.0)]
    size: f64,
    #[arg(default = Value::Na)]
    initial_value: Value<O>,
}

impl<O: PineOutput> ArrayNewString<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = vec![self.initial_value.clone(); self.size as usize];
        Ok(Value::Array(Rc::new(RefCell::new(arr))))
    }
}

/// array.from(...) - Creates an array from the given arguments.
#[derive(BuiltinFunction)]
#[builtin(name = "array.from")]
struct ArrayFrom<O: PineOutput> {
    #[arg(variadic)]
    values: Vec<Value<O>>,
}

impl<O: PineOutput> ArrayFrom<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Array(Rc::new(RefCell::new(self.values.clone()))))
    }
}

/// array.set(id, index, value) - Overwrites the element at `index`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.set")]
struct ArraySet<O: PineOutput> {
    array: Value<O>,
    index: f64,
    value: Value<O>,
}

impl<O: PineOutput> ArraySet<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?;
        let index = self.index as usize;
        let mut arr = arr.borrow_mut();
        if index >= arr.len() {
            return Err(RuntimeError::IndexOutOfBounds(index));
        }
        arr[index] = self.value.clone();
        Ok(Value::Na)
    }
}

/// array.copy(id) - Returns an independent shallow copy of the array.
#[derive(BuiltinFunction)]
#[builtin(name = "array.copy")]
struct ArrayCopy<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayCopy<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let copied = self.array.as_array()?.borrow().clone();
        Ok(Value::Array(Rc::new(RefCell::new(copied))))
    }
}

/// array.concat(id1, id2) - Appends `id2`'s elements onto `id1`, returning `id1`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.concat")]
struct ArrayConcat<O: PineOutput> {
    array1: Value<O>,
    array2: Value<O>,
}

impl<O: PineOutput> ArrayConcat<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let a1 = self.array1.as_array()?;
        // Snapshot `id2` before borrowing `id1` mutably, so `array.concat(a, a)`
        // (both arguments the same array) can't double-borrow.
        let tail = self.array2.as_array()?.borrow().clone();
        a1.borrow_mut().extend(tail);
        Ok(self.array1.clone())
    }
}

/// array.sort(id, order) - Sorts the array in place, ascending by default.
#[derive(BuiltinFunction)]
#[builtin(name = "array.sort")]
struct ArraySort<O: PineOutput> {
    array: Value<O>,
    #[arg(default = "ascending")]
    order: String,
}

impl<O: PineOutput> ArraySort<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?;
        let mut arr = arr.borrow_mut();
        arr.sort_by(compare_values);
        if self.order == "descending" {
            arr.reverse();
        }
        Ok(Value::Na)
    }
}

/// array.join(id, separator) - Concatenates the elements into a string.
#[derive(BuiltinFunction)]
#[builtin(name = "array.join")]
struct ArrayJoin<O: PineOutput> {
    array: Value<O>,
    #[arg(default = "")]
    separator: String,
}

impl<O: PineOutput> ArrayJoin<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let joined = self
            .array
            .as_array()?
            .borrow()
            .iter()
            .map(join_string)
            .collect::<Vec<_>>()
            .join(&self.separator);
        Ok(Value::String(joined))
    }
}

/// array.indexof(id, value) - Index of the first matching element, or -1.
#[derive(BuiltinFunction)]
#[builtin(name = "array.indexof")]
struct ArrayIndexOf<O: PineOutput> {
    array: Value<O>,
    value: Value<O>,
}

impl<O: PineOutput> ArrayIndexOf<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let index = self
            .array
            .as_array()?
            .borrow()
            .iter()
            .position(|v| *v == self.value);
        Ok(Value::Int(index.map_or(-1, |i| i as i64)))
    }
}

/// array.includes(id, value) - Whether the array contains `value`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.includes")]
struct ArrayIncludes<O: PineOutput> {
    array: Value<O>,
    value: Value<O>,
}

impl<O: PineOutput> ArrayIncludes<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let found = self.array.as_array()?.borrow().contains(&self.value);
        Ok(Value::Bool(found))
    }
}

/// Orders two array elements: numerically when both read as numbers, else
/// lexicographically for strings; anything else compares equal (stable).
fn compare_values<O: PineOutput>(a: &Value<O>, b: &Value<O>) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let num = |v: &Value<O>| match v {
        Value::Int(n) => Some(*n as f64),
        Value::Number(n) => Some(*n),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    };
    if let (Some(x), Some(y)) = (num(a), num(b)) {
        return x.partial_cmp(&y).unwrap_or(Ordering::Equal);
    }
    match (a, b) {
        (Value::String(x), Value::String(y)) => x.cmp(y),
        _ => Ordering::Equal,
    }
}

/// Renders one element for `array.join`.
fn join_string<O: PineOutput>(v: &Value<O>) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Number(n) if n.fract() == 0.0 && n.is_finite() => (*n as i64).to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Na => "NaN".to_string(),
        other => format!("{:?}", other),
    }
}

/// A finite number from a value, or `None` for `na` / non-numeric — so the
/// reductions ignore `na`, as Pine does.
fn numeric<O: PineOutput>(v: &Value<O>) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Number(n) if n.is_finite() => Some(*n),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// The finite numbers of an array, in order.
fn numbers<O: PineOutput>(array: &Value<O>) -> Result<Vec<f64>, RuntimeError> {
    Ok(array
        .as_array()?
        .borrow()
        .iter()
        .filter_map(numeric)
        .collect())
}

/// array.first(id) - The first element (errors when empty).
#[derive(BuiltinFunction)]
#[builtin(name = "array.first")]
struct ArrayFirst<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayFirst<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        self.array
            .as_array()?
            .borrow()
            .first()
            .cloned()
            .ok_or(RuntimeError::IndexOutOfBounds(0))
    }
}

/// array.last(id) - The last element (errors when empty).
#[derive(BuiltinFunction)]
#[builtin(name = "array.last")]
struct ArrayLast<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayLast<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        self.array
            .as_array()?
            .borrow()
            .last()
            .cloned()
            .ok_or(RuntimeError::IndexOutOfBounds(0))
    }
}

/// array.pop(id) - Remove and return the last element.
#[derive(BuiltinFunction)]
#[builtin(name = "array.pop")]
struct ArrayPop<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayPop<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        self.array
            .as_array()?
            .borrow_mut()
            .pop()
            .ok_or(RuntimeError::IndexOutOfBounds(0))
    }
}

/// array.shift(id) - Remove and return the first element.
#[derive(BuiltinFunction)]
#[builtin(name = "array.shift")]
struct ArrayShift<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayShift<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut arr = self.array.as_array()?.borrow_mut();
        if arr.is_empty() {
            return Err(RuntimeError::IndexOutOfBounds(0));
        }
        Ok(arr.remove(0))
    }
}

/// array.reverse(id) - Reverse the array in place.
#[derive(BuiltinFunction)]
#[builtin(name = "array.reverse")]
struct ArrayReverse<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayReverse<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        self.array.as_array()?.borrow_mut().reverse();
        Ok(Value::Na)
    }
}

/// array.insert(id, index, value) - Insert `value` before `index`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.insert")]
struct ArrayInsert<O: PineOutput> {
    array: Value<O>,
    index: f64,
    value: Value<O>,
}

impl<O: PineOutput> ArrayInsert<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut arr = self.array.as_array()?.borrow_mut();
        let index = self.index as usize;
        if index > arr.len() {
            return Err(RuntimeError::IndexOutOfBounds(index));
        }
        arr.insert(index, self.value.clone());
        Ok(Value::Na)
    }
}

/// array.remove(id, index) - Remove and return the element at `index`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.remove")]
struct ArrayRemove<O: PineOutput> {
    array: Value<O>,
    index: f64,
}

impl<O: PineOutput> ArrayRemove<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut arr = self.array.as_array()?.borrow_mut();
        let index = self.index as usize;
        if index >= arr.len() {
            return Err(RuntimeError::IndexOutOfBounds(index));
        }
        Ok(arr.remove(index))
    }
}

/// array.fill(id, value, index_from, index_to) - Set a range to `value`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.fill")]
struct ArrayFill<O: PineOutput> {
    array: Value<O>,
    value: Value<O>,
    #[arg(default = 0.0)]
    index_from: f64,
    #[arg(default = None)]
    index_to: Option<f64>,
}

impl<O: PineOutput> ArrayFill<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut arr = self.array.as_array()?.borrow_mut();
        let from = self.index_from as usize;
        let to = self
            .index_to
            .map_or(arr.len(), |n| n as usize)
            .min(arr.len());
        for slot in arr.iter_mut().take(to).skip(from) {
            *slot = self.value.clone();
        }
        Ok(Value::Na)
    }
}

/// array.sum(id) - Sum of the finite elements.
#[derive(BuiltinFunction)]
#[builtin(name = "array.sum")]
struct ArraySum<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArraySum<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(numbers(&self.array)?.iter().sum()))
    }
}

/// array.avg(id) - Mean of the finite elements, na when there are none.
#[derive(BuiltinFunction)]
#[builtin(name = "array.avg")]
struct ArrayAvg<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayAvg<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        Ok(mean(&ns).map_or(Value::Na, Value::Number))
    }
}

/// array.min(id) - Smallest finite element, na when there are none.
#[derive(BuiltinFunction)]
#[builtin(name = "array.min")]
struct ArrayMin<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayMin<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        Ok(ns
            .iter()
            .copied()
            .fold(None, |acc: Option<f64>, x| {
                Some(acc.map_or(x, |a| a.min(x)))
            })
            .map_or(Value::Na, Value::Number))
    }
}

/// array.max(id) - Largest finite element, na when there are none.
#[derive(BuiltinFunction)]
#[builtin(name = "array.max")]
struct ArrayMax<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayMax<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        Ok(ns
            .iter()
            .copied()
            .fold(None, |acc: Option<f64>, x| {
                Some(acc.map_or(x, |a| a.max(x)))
            })
            .map_or(Value::Na, Value::Number))
    }
}

/// array.range(id) - Largest minus smallest, na when empty.
#[derive(BuiltinFunction)]
#[builtin(name = "array.range")]
struct ArrayRange<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayRange<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        if ns.is_empty() {
            return Ok(Value::Na);
        }
        let lo = ns.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = ns.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        Ok(Value::Number(hi - lo))
    }
}

/// array.median(id) - Median of the finite elements, na when empty.
#[derive(BuiltinFunction)]
#[builtin(name = "array.median")]
struct ArrayMedian<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayMedian<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut ns = numbers(&self.array)?;
        if ns.is_empty() {
            return Ok(Value::Na);
        }
        ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = ns.len() / 2;
        let median = if ns.len() % 2 == 0 {
            (ns[mid - 1] + ns[mid]) / 2.0
        } else {
            ns[mid]
        };
        Ok(Value::Number(median))
    }
}

/// array.variance(id, biased) - Variance of the finite elements. `biased` (the
/// default) divides by n; otherwise by n-1.
#[derive(BuiltinFunction)]
#[builtin(name = "array.variance")]
struct ArrayVariance<O: PineOutput> {
    array: Value<O>,
    #[arg(default = true)]
    biased: bool,
}

impl<O: PineOutput> ArrayVariance<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(variance(&numbers(&self.array)?, self.biased).map_or(Value::Na, Value::Number))
    }
}

/// array.stdev(id, biased) - Standard deviation of the finite elements.
#[derive(BuiltinFunction)]
#[builtin(name = "array.stdev")]
struct ArrayStdev<O: PineOutput> {
    array: Value<O>,
    #[arg(default = true)]
    biased: bool,
}

impl<O: PineOutput> ArrayStdev<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(variance(&numbers(&self.array)?, self.biased)
            .map_or(Value::Na, |v| Value::Number(v.sqrt())))
    }
}

/// array.mode(id) - Most frequent element (the smallest on a tie), na when empty.
#[derive(BuiltinFunction)]
#[builtin(name = "array.mode")]
struct ArrayMode<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayMode<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut ns = numbers(&self.array)?;
        if ns.is_empty() {
            return Ok(Value::Na);
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
        Ok(Value::Number(best))
    }
}

/// Mean of `ns`, or `None` when empty.
fn mean(ns: &[f64]) -> Option<f64> {
    (!ns.is_empty()).then(|| ns.iter().sum::<f64>() / ns.len() as f64)
}

/// Variance of `ns` (population when `biased`), or `None` when too few elements.
fn variance(ns: &[f64], biased: bool) -> Option<f64> {
    let mean = mean(ns)?;
    let denominator = if biased {
        ns.len()
    } else {
        ns.len().checked_sub(1)?
    };
    if denominator == 0 {
        return None;
    }
    Some(ns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / denominator as f64)
}

/// Whether a value counts as `true` for `array.every`/`array.some`.
fn truthy<O: PineOutput>(v: &Value<O>) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Number(n) => *n != 0.0 && !n.is_nan(),
        _ => false,
    }
}

macro_rules! array_new {
    ($name:literal, $ident:ident) => {
        #[derive(BuiltinFunction)]
        #[builtin(name = $name)]
        struct $ident<O: PineOutput> {
            #[arg(default = 0.0)]
            size: f64,
            #[arg(default = Value::Na)]
            initial_value: Value<O>,
        }

        impl<O: PineOutput> $ident<O> {
            fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
                let arr = vec![self.initial_value.clone(); self.size as usize];
                Ok(Value::Array(Rc::new(RefCell::new(arr))))
            }
        }
    };
}

array_new!("array.new_bool", ArrayNewBool);
array_new!("array.new_color", ArrayNewColor);
array_new!("array.new_box", ArrayNewBox);
array_new!("array.new_label", ArrayNewLabel);
array_new!("array.new_line", ArrayNewLine);
array_new!("array.new_linefill", ArrayNewLinefill);
array_new!("array.new_table", ArrayNewTable);

/// array.abs(id) - A new array of the absolute values.
#[derive(BuiltinFunction)]
#[builtin(name = "array.abs")]
struct ArrayAbs<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayAbs<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let out: Vec<Value<O>> = self
            .array
            .as_array()?
            .borrow()
            .iter()
            .map(|v| numeric(v).map_or_else(|| v.clone(), |n| Value::Number(n.abs())))
            .collect();
        Ok(Value::Array(Rc::new(RefCell::new(out))))
    }
}

/// array.every(id) - Whether every element is true (vacuously true when empty).
#[derive(BuiltinFunction)]
#[builtin(name = "array.every")]
struct ArrayEvery<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayEvery<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Bool(
            self.array.as_array()?.borrow().iter().all(truthy),
        ))
    }
}

/// array.some(id) - Whether at least one element is true.
#[derive(BuiltinFunction)]
#[builtin(name = "array.some")]
struct ArraySome<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArraySome<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Bool(
            self.array.as_array()?.borrow().iter().any(truthy),
        ))
    }
}

/// array.lastindexof(id, value) - Index of the last matching element, or -1.
#[derive(BuiltinFunction)]
#[builtin(name = "array.lastindexof")]
struct ArrayLastIndexOf<O: PineOutput> {
    array: Value<O>,
    value: Value<O>,
}

impl<O: PineOutput> ArrayLastIndexOf<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let index = self
            .array
            .as_array()?
            .borrow()
            .iter()
            .rposition(|v| *v == self.value);
        Ok(Value::Int(index.map_or(-1, |i| i as i64)))
    }
}

/// array.standardize(id) - A new array of z-scores `(x - mean) / stdev`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.standardize")]
struct ArrayStandardize<O: PineOutput> {
    array: Value<O>,
}

impl<O: PineOutput> ArrayStandardize<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        let mean = mean(&ns).unwrap_or(0.0);
        let stdev = variance(&ns, true).unwrap_or(0.0).sqrt();
        let out: Vec<Value<O>> = ns
            .iter()
            .map(|x| Value::Number(if stdev > 0.0 { (x - mean) / stdev } else { 0.0 }))
            .collect();
        Ok(Value::Array(Rc::new(RefCell::new(out))))
    }
}

/// array.covariance(id1, id2, biased) - Covariance of two arrays over their
/// paired finite elements.
#[derive(BuiltinFunction)]
#[builtin(name = "array.covariance")]
struct ArrayCovariance<O: PineOutput> {
    array1: Value<O>,
    array2: Value<O>,
    #[arg(default = true)]
    biased: bool,
}

impl<O: PineOutput> ArrayCovariance<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let xs = numbers(&self.array1)?;
        let ys = numbers(&self.array2)?;
        let n = xs.len().min(ys.len());
        let denominator = if self.biased { n } else { n.saturating_sub(1) };
        if denominator == 0 {
            return Ok(Value::Na);
        }
        let mx = xs[..n].iter().sum::<f64>() / n as f64;
        let my = ys[..n].iter().sum::<f64>() / n as f64;
        let cov = (0..n).map(|i| (xs[i] - mx) * (ys[i] - my)).sum::<f64>() / denominator as f64;
        Ok(Value::Number(cov))
    }
}

/// array.sort_indices(id, order) - Indices that would sort the array.
#[derive(BuiltinFunction)]
#[builtin(name = "array.sort_indices")]
struct ArraySortIndices<O: PineOutput> {
    array: Value<O>,
    #[arg(default = "ascending")]
    order: String,
}

impl<O: PineOutput> ArraySortIndices<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?.borrow();
        let mut indices: Vec<usize> = (0..arr.len()).collect();
        indices.sort_by(|&a, &b| compare_values(&arr[a], &arr[b]));
        if self.order == "descending" {
            indices.reverse();
        }
        let out = indices.into_iter().map(|i| Value::Int(i as i64)).collect();
        Ok(Value::Array(Rc::new(RefCell::new(out))))
    }
}

/// array.binary_search(id, val) - Index of `val` in a sorted array, or -1.
#[derive(BuiltinFunction)]
#[builtin(name = "array.binary_search")]
struct ArrayBinarySearch<O: PineOutput> {
    array: Value<O>,
    val: f64,
}

impl<O: PineOutput> ArrayBinarySearch<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        let found = ns
            .binary_search_by(|x| x.partial_cmp(&self.val).unwrap())
            .ok();
        Ok(Value::Int(found.map_or(-1, |i| i as i64)))
    }
}

/// array.slice(id, index_from, index_to) - A copy of the `[from, to)` range.
/// (Pine returns a live view; this returns a shallow copy.)
#[derive(BuiltinFunction)]
#[builtin(name = "array.slice")]
struct ArraySlice<O: PineOutput> {
    array: Value<O>,
    index_from: f64,
    index_to: f64,
}

impl<O: PineOutput> ArraySlice<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?.borrow();
        let from = (self.index_from as usize).min(arr.len());
        let to = (self.index_to as usize).clamp(from, arr.len());
        Ok(Value::Array(Rc::new(RefCell::new(arr[from..to].to_vec()))))
    }
}

/// array.binary_search_leftmost(id, val) - Index of `val`, else the index just
/// left of where it would lie (-1 before the start). Array must be sorted.
#[derive(BuiltinFunction)]
#[builtin(name = "array.binary_search_leftmost")]
struct ArrayBinarySearchLeftmost<O: PineOutput> {
    array: Value<O>,
    val: f64,
}

impl<O: PineOutput> ArrayBinarySearchLeftmost<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        let lower = ns.partition_point(|x| *x < self.val);
        let found = ns.get(lower) == Some(&self.val);
        Ok(Value::Int(if found {
            lower as i64
        } else {
            lower as i64 - 1
        }))
    }
}

/// array.binary_search_rightmost(id, val) - Index of `val`, else the index just
/// right of where it would lie. Array must be sorted ascending.
#[derive(BuiltinFunction)]
#[builtin(name = "array.binary_search_rightmost")]
struct ArrayBinarySearchRightmost<O: PineOutput> {
    array: Value<O>,
    val: f64,
}

impl<O: PineOutput> ArrayBinarySearchRightmost<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let ns = numbers(&self.array)?;
        let upper = ns.partition_point(|x| *x <= self.val);
        let found = upper > 0 && ns[upper - 1] == self.val;
        Ok(Value::Int(if found {
            upper as i64 - 1
        } else {
            upper as i64
        }))
    }
}

/// array.percentrank(id, index) - Percentage of elements below the element at
/// `index`.
#[derive(BuiltinFunction)]
#[builtin(name = "array.percentrank")]
struct ArrayPercentRank<O: PineOutput> {
    array: Value<O>,
    index: f64,
}

impl<O: PineOutput> ArrayPercentRank<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let arr = self.array.as_array()?.borrow();
        let index = self.index as usize;
        let Some(target) = arr.get(index).and_then(numeric) else {
            return Ok(Value::Na);
        };
        if arr.len() < 2 {
            return Ok(Value::Number(0.0));
        }
        let below = arr
            .iter()
            .filter_map(numeric)
            .filter(|&x| x < target)
            .count();
        Ok(Value::Number(below as f64 / (arr.len() - 1) as f64 * 100.0))
    }
}

/// array.percentile_nearest_rank(id, percentage) - The value at `percentage`
/// using the nearest-rank method.
#[derive(BuiltinFunction)]
#[builtin(name = "array.percentile_nearest_rank")]
struct ArrayPercentileNearestRank<O: PineOutput> {
    array: Value<O>,
    percentage: f64,
}

impl<O: PineOutput> ArrayPercentileNearestRank<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut ns = numbers(&self.array)?;
        if ns.is_empty() {
            return Ok(Value::Na);
        }
        ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let rank = (self.percentage / 100.0 * ns.len() as f64).ceil() as usize;
        Ok(Value::Number(ns[rank.clamp(1, ns.len()) - 1]))
    }
}

/// array.percentile_linear_interpolation(id, percentage) - The value at
/// `percentage`, interpolating between ranks.
#[derive(BuiltinFunction)]
#[builtin(name = "array.percentile_linear_interpolation")]
struct ArrayPercentileLinear<O: PineOutput> {
    array: Value<O>,
    percentage: f64,
}

impl<O: PineOutput> ArrayPercentileLinear<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut ns = numbers(&self.array)?;
        if ns.is_empty() {
            return Ok(Value::Na);
        }
        ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let pos = self.percentage / 100.0 * (ns.len() - 1) as f64;
        let lo = pos.floor() as usize;
        let value = if lo + 1 < ns.len() {
            ns[lo] + (pos - lo as f64) * (ns[lo + 1] - ns[lo])
        } else {
            ns[lo]
        };
        Ok(Value::Number(value))
    }
}

/// Register all array namespace functions and return the namespace object
pub fn register<O: PineOutput>() -> Value<O> {
    let mut array_ns: std::collections::HashMap<String, Value<O>> =
        std::collections::HashMap::new();

    // Generic typed array.new<type>()
    array_ns.insert("new".to_string(), ArrayNew::<O>::builtin_value());
    // Backward compatible typed constructors.
    array_ns.insert("new_float".to_string(), ArrayNewFloat::<O>::builtin_value());
    array_ns.insert("new_int".to_string(), ArrayNewInt::<O>::builtin_value());
    array_ns.insert(
        "new_string".to_string(),
        ArrayNewString::<O>::builtin_value(),
    );
    array_ns.insert("from".to_string(), ArrayFrom::<O>::builtin_value());
    array_ns.insert("clear".to_string(), ArrayClear::<O>::builtin_value());
    array_ns.insert("push".to_string(), ArrayPush::<O>::builtin_value());
    array_ns.insert("unshift".to_string(), ArrayUnshift::<O>::builtin_value());
    array_ns.insert("get".to_string(), ArrayGet::<O>::builtin_value());
    array_ns.insert("set".to_string(), ArraySet::<O>::builtin_value());
    array_ns.insert("size".to_string(), ArraySize::<O>::builtin_value());
    array_ns.insert("copy".to_string(), ArrayCopy::<O>::builtin_value());
    array_ns.insert("concat".to_string(), ArrayConcat::<O>::builtin_value());
    array_ns.insert("sort".to_string(), ArraySort::<O>::builtin_value());
    array_ns.insert("join".to_string(), ArrayJoin::<O>::builtin_value());
    array_ns.insert("indexof".to_string(), ArrayIndexOf::<O>::builtin_value());
    array_ns.insert("includes".to_string(), ArrayIncludes::<O>::builtin_value());
    array_ns.insert("first".to_string(), ArrayFirst::<O>::builtin_value());
    array_ns.insert("last".to_string(), ArrayLast::<O>::builtin_value());
    array_ns.insert("pop".to_string(), ArrayPop::<O>::builtin_value());
    array_ns.insert("shift".to_string(), ArrayShift::<O>::builtin_value());
    array_ns.insert("reverse".to_string(), ArrayReverse::<O>::builtin_value());
    array_ns.insert("insert".to_string(), ArrayInsert::<O>::builtin_value());
    array_ns.insert("remove".to_string(), ArrayRemove::<O>::builtin_value());
    array_ns.insert("fill".to_string(), ArrayFill::<O>::builtin_value());
    array_ns.insert("sum".to_string(), ArraySum::<O>::builtin_value());
    array_ns.insert("avg".to_string(), ArrayAvg::<O>::builtin_value());
    array_ns.insert("min".to_string(), ArrayMin::<O>::builtin_value());
    array_ns.insert("max".to_string(), ArrayMax::<O>::builtin_value());
    array_ns.insert("range".to_string(), ArrayRange::<O>::builtin_value());
    array_ns.insert("median".to_string(), ArrayMedian::<O>::builtin_value());
    array_ns.insert("variance".to_string(), ArrayVariance::<O>::builtin_value());
    array_ns.insert("stdev".to_string(), ArrayStdev::<O>::builtin_value());
    array_ns.insert("mode".to_string(), ArrayMode::<O>::builtin_value());
    array_ns.insert("new_bool".to_string(), ArrayNewBool::<O>::builtin_value());
    array_ns.insert("new_color".to_string(), ArrayNewColor::<O>::builtin_value());
    array_ns.insert("new_box".to_string(), ArrayNewBox::<O>::builtin_value());
    array_ns.insert("new_label".to_string(), ArrayNewLabel::<O>::builtin_value());
    array_ns.insert("new_line".to_string(), ArrayNewLine::<O>::builtin_value());
    array_ns.insert(
        "new_linefill".to_string(),
        ArrayNewLinefill::<O>::builtin_value(),
    );
    array_ns.insert("new_table".to_string(), ArrayNewTable::<O>::builtin_value());
    array_ns.insert("abs".to_string(), ArrayAbs::<O>::builtin_value());
    array_ns.insert("every".to_string(), ArrayEvery::<O>::builtin_value());
    array_ns.insert("some".to_string(), ArraySome::<O>::builtin_value());
    array_ns.insert(
        "lastindexof".to_string(),
        ArrayLastIndexOf::<O>::builtin_value(),
    );
    array_ns.insert(
        "standardize".to_string(),
        ArrayStandardize::<O>::builtin_value(),
    );
    array_ns.insert(
        "covariance".to_string(),
        ArrayCovariance::<O>::builtin_value(),
    );
    array_ns.insert(
        "sort_indices".to_string(),
        ArraySortIndices::<O>::builtin_value(),
    );
    array_ns.insert(
        "binary_search".to_string(),
        ArrayBinarySearch::<O>::builtin_value(),
    );
    array_ns.insert("slice".to_string(), ArraySlice::<O>::builtin_value());
    array_ns.insert(
        "binary_search_leftmost".to_string(),
        ArrayBinarySearchLeftmost::<O>::builtin_value(),
    );
    array_ns.insert(
        "binary_search_rightmost".to_string(),
        ArrayBinarySearchRightmost::<O>::builtin_value(),
    );
    array_ns.insert(
        "percentrank".to_string(),
        ArrayPercentRank::<O>::builtin_value(),
    );
    array_ns.insert(
        "percentile_nearest_rank".to_string(),
        ArrayPercentileNearestRank::<O>::builtin_value(),
    );
    array_ns.insert(
        "percentile_linear_interpolation".to_string(),
        ArrayPercentileLinear::<O>::builtin_value(),
    );

    Value::Object {
        type_name: "array".to_string(),
        fields: Rc::new(RefCell::new(array_ns)),
        call: None,
        value: None,
    }
}
