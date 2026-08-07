//! The `runtime.*` namespace: control over script execution.

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// runtime.error(message) - Halt the script with a custom error message.
#[derive(BuiltinFunction)]
#[builtin(name = "runtime.error")]
struct RuntimeErrorFn {
    message: String,
}

impl RuntimeErrorFn {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Err(RuntimeError::UserError(self.message.clone()))
    }
}

/// Register the `runtime.*` namespace object.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    members.insert("error".to_string(), RuntimeErrorFn::builtin_value::<O>());
    Value::Object {
        type_name: "runtime".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
