use pine_interpreter::{LogLevel, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Create the log namespace with functions that write to interpreter output
pub fn register() -> Value {
    use std::collections::HashMap;

    let mut log_ns = HashMap::new();

    // Define all log levels and their corresponding function names
    let levels = [
        ("info", LogLevel::Info),
        ("warning", LogLevel::Warning),
        ("error", LogLevel::Error),
    ];

    for (name, level) in levels {
        let log_fn: pine_interpreter::BuiltinFn = Rc::new(move |ctx, func_call| {
            let msg = match func_call.args.first() {
                Some(pine_interpreter::EvaluatedArg::Positional(v)) => value_to_string(v),
                _ => String::new(),
            };
            ctx.output.add_log(level, msg);
            Ok(Value::Na)
        });
        log_ns.insert(name.to_string(), Value::BuiltinFunction(log_fn));
    }

    Value::Object {
        type_name: "log".to_string(),
        fields: Rc::new(RefCell::new(log_ns)),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Na => "na".to_string(),
        other => format!("{:?}", other),
    }
}
