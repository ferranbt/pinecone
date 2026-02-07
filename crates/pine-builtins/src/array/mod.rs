use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// array.new<type>() - Creates a new typed array (generic version)
#[derive(BuiltinFunction)]
#[builtin(name = "array.new", type_params = 1)]
struct ArrayNew {
    #[type_param]
    element_type: String,
    size: f64,
    #[arg(default = Value::Na)]
    initial_value: Value,
}

impl ArrayNew {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // Validate element type
        if !matches!(
            self.element_type.as_str(),
            "int" | "float" | "string" | "bool" | "color"
        ) {
            return Err(RuntimeError::TypeError(format!(
                "Invalid array element type '{}'. Must be int, float, string, bool, or color",
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

/// Register all array namespace functions and return the namespace object
pub fn register() -> Value {
    let mut array_ns = std::collections::HashMap::new();

    // Generic typed array.new<type>()
    array_ns.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayNew::builtin_fn)),
    );
    // Backward compatible array.new_float()
    array_ns.insert(
        "new_float".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayNewFloat::builtin_fn)),
    );
    array_ns.insert(
        "clear".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayClear::builtin_fn)),
    );
    array_ns.insert(
        "push".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayPush::builtin_fn)),
    );
    array_ns.insert(
        "get".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayGet::builtin_fn)),
    );
    array_ns.insert(
        "size".to_string(),
        Value::BuiltinFunction(Rc::new(ArraySize::builtin_fn)),
    );

    Value::Object {
        type_name: "array".to_string(),
        fields: Rc::new(RefCell::new(array_ns)),
    }
}
