use pine_interpreter::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Trait for logging output
pub trait Logger {
    fn log(&self, level: LogLevel, msg: &str);
}

/// Default logger implementation that outputs to screen
pub struct DefaultLogger;

impl Logger for DefaultLogger {
    fn log(&self, level: LogLevel, msg: &str) {
        eprintln!("[{}] {}", level, msg);
    }
}

pub struct Log<T: Logger> {
    pub logger: T,
}

impl<T: Logger + 'static> Log<T> {
    pub fn new(logger: T) -> Self {
        Log { logger }
    }

    pub fn register(self) -> Value {
        use std::collections::HashMap;

        let logger = Rc::new(self.logger);
        let mut log_ns = HashMap::new();

        // Define all log levels and their corresponding function names
        let levels = [
            ("info", LogLevel::Info),
            ("warning", LogLevel::Warning),
            ("error", LogLevel::Error),
        ];

        for (name, level) in levels {
            let logger_clone = logger.clone();
            let log_fn: pine_interpreter::BuiltinFn = Rc::new(move |_ctx, args| {
                let msg = match args.get(0) {
                    Some(pine_interpreter::EvaluatedArg::Positional(v)) => value_to_string(v),
                    _ => String::new(),
                };
                logger_clone.log(level, &msg);
                Ok(Value::Na)
            });
            log_ns.insert(name.to_string(), Value::BuiltinFunction(log_fn));
        }

        Value::Object {
            type_name: "log".to_string(),
            fields: Rc::new(RefCell::new(log_ns)),
        }
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
