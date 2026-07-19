use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, PineOutput, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// array.new<type>() - Creates a new typed array (generic version)
#[derive(BuiltinFunction)]
#[builtin(name = "array.new", type_params = 1)]
struct ArrayNew<O: PineOutput> {
    #[type_param]
    element_type: String,
    size: f64,
    #[arg(default = Value::Na)]
    initial_value: Value<O>,
}

impl<O: PineOutput> ArrayNew<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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

/// Register all array namespace functions and return the namespace object
pub fn register<O: PineOutput>() -> Value<O> {
    let mut array_ns: std::collections::HashMap<String, Value<O>> =
        std::collections::HashMap::new();

    // Generic typed array.new<type>()
    array_ns.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayNew::<O>::builtin_fn)),
    );
    // Backward compatible array.new_float()
    array_ns.insert(
        "new_float".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayNewFloat::<O>::builtin_fn)),
    );
    array_ns.insert(
        "clear".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayClear::<O>::builtin_fn)),
    );
    array_ns.insert(
        "push".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayPush::<O>::builtin_fn)),
    );
    array_ns.insert(
        "get".to_string(),
        Value::BuiltinFunction(Rc::new(ArrayGet::<O>::builtin_fn)),
    );
    array_ns.insert(
        "size".to_string(),
        Value::BuiltinFunction(Rc::new(ArraySize::<O>::builtin_fn)),
    );

    Value::Object {
        type_name: "array".to_string(),
        fields: Rc::new(RefCell::new(array_ns)),
        call: None,
    }
}
