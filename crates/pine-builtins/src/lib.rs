use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::collections::HashMap;
use std::rc::Rc;

// Re-export for convenience
pub use pine_interpreter::Bar;
pub use pine_interpreter::BuiltinFn;
pub use pine_interpreter::EvaluatedArg;

// Namespace modules
mod array;
mod color;
mod log;
mod math;
mod str;
mod time;

// Global utility functions - defined first so they can be referenced in register function

/// na(value) - Returns true if the value is na, false otherwise
#[derive(BuiltinFunction)]
#[builtin(name = "na")]
struct Na {
    value: Value,
}

impl Na {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(matches!(self.value, Value::Na)))
    }
}

/// bool(x) - Converts value to bool
#[derive(BuiltinFunction)]
#[builtin(name = "bool")]
struct Bool {
    x: Value,
}

impl Bool {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        match &self.x {
            Value::Bool(b) => Ok(Value::Bool(*b)),
            Value::Number(n) => Ok(Value::Bool(*n != 0.0)),
            Value::Na => Ok(Value::Bool(false)),
            _ => Ok(Value::Bool(true)),
        }
    }
}

/// int(x) - Converts value to int (truncates float)
#[derive(BuiltinFunction)]
#[builtin(name = "int")]
struct Int {
    x: Value,
}

impl Int {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        match &self.x {
            Value::Number(n) => Ok(Value::Number(n.trunc())),
            Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
            Value::Na => Ok(Value::Na),
            _ => Err(RuntimeError::TypeError(format!("Cannot convert {:?} to int", self.x))),
        }
    }
}

/// float(x) - Converts value to float
#[derive(BuiltinFunction)]
#[builtin(name = "float")]
struct Float {
    x: Value,
}

impl Float {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        match &self.x {
            Value::Number(n) => Ok(Value::Number(*n)),
            Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
            Value::Na => Ok(Value::Na),
            _ => Err(RuntimeError::TypeError(format!("Cannot convert {:?} to float", self.x))),
        }
    }
}

/// nz(source, replacement) - Replaces na values with default or replacement value
#[derive(BuiltinFunction)]
#[builtin(name = "nz")]
struct Nz {
    source: Value,
    #[arg(default = Value::Number(0.0))]
    replacement: Value,
}

impl Nz {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        match &self.source {
            Value::Na => {
                // If replacement is not provided (default), use type-specific defaults
                match &self.replacement {
                    Value::Number(_) => Ok(self.replacement.clone()),
                    _ => Ok(Value::Number(0.0)),
                }
            }
            _ => Ok(self.source.clone()),
        }
    }
}

/// fixnan(source) - Replaces NaN values with previous nearest non-NaN value
#[derive(BuiltinFunction)]
#[builtin(name = "fixnan")]
struct Fixnan {
    source: Value,
}

impl Fixnan {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // This is a simplified implementation
        // A full implementation would need to track previous values across bar evaluations
        match &self.source {
            Value::Na => {
                // Try to get the last non-na value from context
                // For now, just return 0.0 as a placeholder
                Ok(Value::Number(0.0))
            }
            Value::Number(n) if n.is_nan() => {
                Ok(Value::Number(0.0))
            }
            _ => Ok(self.source.clone()),
        }
    }
}

/// Register all builtin namespaces as objects and global functions
/// Returns namespace objects to be loaded as variables (e.g., "array", "str", "ta")
/// and global builtin functions (e.g., "na")
/// Each member stores the builtin function pointer as Value::BuiltinFunction
pub fn register_namespace_objects() -> HashMap<String, Value> {
    let mut namespaces = HashMap::new();

    // Register namespace objects
    namespaces.insert("array".to_string(), array::register());
    namespaces.insert("color".to_string(), color::register());
    namespaces.insert("log".to_string(), log::register());
    namespaces.insert("math".to_string(), math::register());
    namespaces.insert("str".to_string(), str::register());

    // Register global builtin functions
    namespaces.insert("na".to_string(), Value::BuiltinFunction(Rc::new(Na::builtin_fn) as BuiltinFn));
    namespaces.insert("bool".to_string(), Value::BuiltinFunction(Rc::new(Bool::builtin_fn) as BuiltinFn));
    namespaces.insert("int".to_string(), Value::BuiltinFunction(Rc::new(Int::builtin_fn) as BuiltinFn));
    namespaces.insert("float".to_string(), Value::BuiltinFunction(Rc::new(Float::builtin_fn) as BuiltinFn));
    namespaces.insert("nz".to_string(), Value::BuiltinFunction(Rc::new(Nz::builtin_fn) as BuiltinFn));
    namespaces.insert("fixnan".to_string(), Value::BuiltinFunction(Rc::new(Fixnan::builtin_fn) as BuiltinFn));

    // Register time/date functions
    for (name, func) in time::register_time_functions() {
        namespaces.insert(name, func);
    }

    namespaces
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::EvaluatedArg;

    #[test]
    fn test_na() {
        let mut ctx = Interpreter::new();

        // Test with na value
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Test with number
        let args = vec![EvaluatedArg::Positional(Value::Number(42.0))];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test with string
        let args = vec![EvaluatedArg::Positional(Value::String("hello".to_string()))];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test with bool
        let args = vec![EvaluatedArg::Positional(Value::Bool(true))];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_bool() {
        let mut ctx = Interpreter::new();

        // Test number to bool
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Bool::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(true));

        let args = vec![EvaluatedArg::Positional(Value::Number(0.0))];
        let result = Bool::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test na to bool
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Bool::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_int() {
        let mut ctx = Interpreter::new();

        // Test float to int (truncate)
        let args = vec![EvaluatedArg::Positional(Value::Number(5.7))];
        let result = Int::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(5.0));

        let args = vec![EvaluatedArg::Positional(Value::Number(-5.7))];
        let result = Int::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(-5.0));

        // Test bool to int
        let args = vec![EvaluatedArg::Positional(Value::Bool(true))];
        let result = Int::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test na to int
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Int::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Na);
    }

    #[test]
    fn test_float() {
        let mut ctx = Interpreter::new();

        // Test number to float
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Float::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test bool to float
        let args = vec![EvaluatedArg::Positional(Value::Bool(true))];
        let result = Float::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test na to float
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Float::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Na);
    }

    #[test]
    fn test_nz() {
        let mut ctx = Interpreter::new();

        // Test na value without replacement (should return 0.0)
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Nz::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(0.0));

        // Test na value with replacement
        let args = vec![
            EvaluatedArg::Positional(Value::Na),
            EvaluatedArg::Positional(Value::Number(42.0)),
        ];
        let result = Nz::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(42.0));

        // Test non-na value (should return source)
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Nz::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_fixnan() {
        let mut ctx = Interpreter::new();

        // Test na value
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Fixnan::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(0.0));

        // Test normal value
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Fixnan::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test NaN value
        let args = vec![EvaluatedArg::Positional(Value::Number(f64::NAN))];
        let result = Fixnan::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }
}
