use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Re-export for convenience
pub use pine_interpreter::Bar;

/// Register all builtin namespaces as objects
/// Returns namespace objects to be loaded as variables (e.g., "array", "str", "ta")
/// Each member stores the builtin function pointer as Value::BuiltinFunction
pub fn register_namespace_objects() -> HashMap<String, Value> {
    let mut namespaces = HashMap::new();

    // Create 'array' namespace object with builtin functions
    let mut array_ns = HashMap::new();
    array_ns.insert("new_float".to_string(), Value::BuiltinFunction(ArrayNewFloat::builtin_fn));
    array_ns.insert("clear".to_string(), Value::BuiltinFunction(ArrayClear::builtin_fn));
    array_ns.insert("push".to_string(), Value::BuiltinFunction(ArrayPush::builtin_fn));
    array_ns.insert("get".to_string(), Value::BuiltinFunction(ArrayGet::builtin_fn));
    array_ns.insert("size".to_string(), Value::BuiltinFunction(ArraySize::builtin_fn));

    namespaces.insert("array".to_string(), Value::Object(Rc::new(RefCell::new(array_ns))));

    namespaces
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.new_float")]
struct ArrayNewFloat {
    size: f64,
    initial_value: Value,
}

impl ArrayNewFloat {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let size = self.size as usize;
        let arr = vec![self.initial_value.clone(); size];
        Ok(Value::Array(Rc::new(RefCell::new(arr))))
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.clear")]
struct ArrayClear {
    array: Value,
}

impl ArrayClear {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let arr = self.array.as_array()?;
        arr.borrow_mut().clear();
        Ok(Value::Na)
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.push")]
struct ArrayPush {
    array: Value,
    value: Value,
}

impl ArrayPush {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let arr = self.array.as_array()?;
        arr.borrow_mut().push(self.value.clone());
        Ok(Value::Na)
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.get")]
struct ArrayGet {
    array: Value,
    index: f64,
}

impl ArrayGet {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
struct ArraySize {
    array: Value,
}

impl ArraySize {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let arr = self.array.as_array()?;
        let size = arr.borrow().len();
        Ok(Value::Number(size as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::EvaluatedArg;

    fn create_mock_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_array_new_float() {
        let mut ctx = create_mock_interpreter();
        let args = vec![
            EvaluatedArg::Positional(Value::Number(3.0)),
            EvaluatedArg::Positional(Value::Number(5.5)),
        ];

        let result = ArrayNewFloat::builtin_fn(&mut ctx, args).unwrap();

        if let Value::Array(arr_ref) = result {
            let arr = arr_ref.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(5.5));
            assert_eq!(arr[1], Value::Number(5.5));
            assert_eq!(arr[2], Value::Number(5.5));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_array_operations() {
        let mut ctx = create_mock_interpreter();

        // Create array
        let create_args = vec![
            EvaluatedArg::Positional(Value::Number(2.0)),
            EvaluatedArg::Positional(Value::Number(10.0)),
        ];
        let array = ArrayNewFloat::builtin_fn(&mut ctx, create_args).unwrap();

        // Clear array
        let clear_args = vec![EvaluatedArg::Positional(array.clone())];
        ArrayClear::builtin_fn(&mut ctx, clear_args).unwrap();

        // Check size after clear
        let size_args = vec![EvaluatedArg::Positional(array.clone())];
        let size = ArraySize::builtin_fn(&mut ctx, size_args).unwrap();
        assert_eq!(size, Value::Number(0.0));

        // Push element
        let push_args = vec![
            EvaluatedArg::Positional(array.clone()),
            EvaluatedArg::Positional(Value::Number(42.0)),
        ];
        ArrayPush::builtin_fn(&mut ctx, push_args).unwrap();

        // Check size after push
        let size_args = vec![EvaluatedArg::Positional(array.clone())];
        let size = ArraySize::builtin_fn(&mut ctx, size_args).unwrap();
        assert_eq!(size, Value::Number(1.0));

        // Get element
        let get_args = vec![
            EvaluatedArg::Positional(array.clone()),
            EvaluatedArg::Positional(Value::Number(0.0)),
        ];
        let value = ArrayGet::builtin_fn(&mut ctx, get_args).unwrap();
        assert_eq!(value, Value::Number(42.0));
    }
}
