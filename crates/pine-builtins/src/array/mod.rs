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

/// Register all array namespace functions and return the namespace object
pub fn register<O: PineOutput>() -> Value<O> {
    let mut array_ns: std::collections::HashMap<String, Value<O>> =
        std::collections::HashMap::new();

    // Generic typed array.new<type>()
    array_ns.insert("new".to_string(), ArrayNew::<O>::builtin_value());
    // Backward compatible typed constructors.
    array_ns.insert("new_float".to_string(), ArrayNewFloat::<O>::builtin_value());
    array_ns.insert("new_int".to_string(), ArrayNewInt::<O>::builtin_value());
    array_ns.insert("new_string".to_string(), ArrayNewString::<O>::builtin_value());
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

    Value::Object {
        type_name: "array".to_string(),
        fields: Rc::new(RefCell::new(array_ns)),
        call: None,
    }
}
