use pine_interpreter::{EvaluatedArg, LogLevel, LogOutput, PineOutput, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Create the log namespace with functions that write to interpreter output
pub fn register<O: PineOutput + LogOutput>() -> Value<O> {
    use std::collections::HashMap;

    let mut log_ns = HashMap::new();

    // Define all log levels and their corresponding function names
    let levels = [
        ("info", LogLevel::Info),
        ("warning", LogLevel::Warning),
        ("error", LogLevel::Error),
    ];

    for (name, level) in levels {
        let log_fn: pine_interpreter::BuiltinFn<O> = Rc::new(move |ctx, func_call| {
            let msg = format_message(name, func_call.args.first())?;
            ctx.output.add_log(level, msg);
            Ok(Value::Na)
        });
        log_ns.insert(name.to_string(), Value::BuiltinFunction(log_fn));
    }

    Value::Object {
        type_name: "log".to_string(),
        fields: Rc::new(RefCell::new(log_ns)),
        call: None,
    }
}

/// The log message. Pine types this argument as `string`, so a non-string is a
/// type error — a number is printed with `str.tostring`, not logged directly.
fn format_message<O: PineOutput>(
    name: &str,
    first: Option<&EvaluatedArg<O>>,
) -> Result<String, RuntimeError> {
    match first {
        Some(EvaluatedArg::Positional(Value::String(s)))
        | Some(EvaluatedArg::Named {
            value: Value::String(s),
            ..
        }) => Ok(s.clone()),
        Some(EvaluatedArg::Positional(Value::Na))
        | Some(EvaluatedArg::Named {
            value: Value::Na, ..
        }) => Ok(String::new()),
        Some(EvaluatedArg::Positional(other)) | Some(EvaluatedArg::Named { value: other, .. }) => {
            Err(RuntimeError::TypeError(format!(
                "log.{name} expects a string message, got {other:?}"
            )))
        }
        None => Ok(String::new()),
    }
}
