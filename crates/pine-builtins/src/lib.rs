use pine_builtin_macro::BuiltinFunction;
use pine_core::{PineVersion, SymInfo, Timeframe};
use pine_interpreter::{
    BoxOutput, GlobalOutput, IndicatorOutput, InputOutput, LabelOutput, LineOutput, LogOutput,
    PineOutput, PlotOutput, TableOutput,
};
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::collections::HashMap;
use std::rc::Rc;

// Re-export for convenience
pub use pine_interpreter::Bar;
pub use pine_interpreter::BuiltinFn;
pub use pine_interpreter::DefaultPineOutput;
pub use pine_interpreter::EvaluatedArg;
pub use pine_interpreter::LogLevel;

// Namespace modules
mod array;
mod barstate;
mod r#box;
mod color;
mod constants;
mod currency;
mod globals;
mod indicator;
mod input;
mod label;
mod line;
mod log;
mod math;
mod matrix;
mod plot;
mod str;
mod syminfo;
mod ta;
mod table;
mod time;
mod timeframe;

// Global utility functions - defined first so they can be referenced in register function

/// na(value) - Returns true if the value is na, false otherwise
#[derive(BuiltinFunction)]
#[builtin(name = "na")]
struct Na<O: PineOutput> {
    value: Value<O>,
}

impl<O: PineOutput> Na<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Bool(matches!(self.value, Value::Na)))
    }
}

/// bool(x) - Converts value to bool
#[derive(BuiltinFunction)]
#[builtin(name = "bool")]
struct Bool<O: PineOutput> {
    x: Value<O>,
}

impl<O: PineOutput> Bool<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct Int<O: PineOutput> {
    x: Value<O>,
}

impl<O: PineOutput> Int<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        match &self.x {
            Value::Number(n) => Ok(Value::Number(n.trunc())),
            Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
            Value::Na => Ok(Value::Na),
            _ => Err(RuntimeError::TypeError(format!(
                "Cannot convert {:?} to int",
                self.x
            ))),
        }
    }
}

/// float(x) - Converts value to float
#[derive(BuiltinFunction)]
#[builtin(name = "float")]
struct Float<O: PineOutput> {
    x: Value<O>,
}

impl<O: PineOutput> Float<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        match &self.x {
            Value::Number(n) => Ok(Value::Number(*n)),
            Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
            Value::Na => Ok(Value::Na),
            _ => Err(RuntimeError::TypeError(format!(
                "Cannot convert {:?} to float",
                self.x
            ))),
        }
    }
}

/// nz(source, replacement) - Replaces na values with default or replacement value
#[derive(BuiltinFunction)]
#[builtin(name = "nz")]
struct Nz<O: PineOutput> {
    source: Value<O>,
    #[arg(default = Value::Number(0.0))]
    replacement: Value<O>,
}

impl<O: PineOutput> Nz<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct Fixnan<O: PineOutput> {
    source: Value<O>,
}

impl<O: PineOutput> Fixnan<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // This is a simplified implementation
        // A full implementation would need to track previous values across bar evaluations
        match &self.source {
            Value::Na => {
                // Try to get the last non-na value from context
                // For now, just return 0.0 as a placeholder
                Ok(Value::Number(0.0))
            }
            Value::Number(n) if n.is_nan() => Ok(Value::Number(0.0)),
            _ => Ok(self.source.clone()),
        }
    }
}

/// Register all builtin namespaces as objects and global functions
/// Returns namespace objects to be loaded as variables (e.g., "array", "str", "ta")
/// and global builtin functions (e.g., "na")
/// Each member stores the builtin function pointer as Value::BuiltinFunction
///
/// This uses DefaultPineOutput for now. Full generic support will be added when the
/// BuiltinFunction macro is updated to support generic output types.
pub fn register_namespace_objects<
    O: PineOutput
        + LogOutput
        + PlotOutput
        + LabelOutput
        + BoxOutput
        + InputOutput
        + LineOutput
        + TableOutput
        + IndicatorOutput
        + GlobalOutput,
>(
    version: PineVersion,
    syminfo: Option<SymInfo>,
    timeframe: Option<Timeframe>,
) -> HashMap<String, Value<O>> {
    let mut namespaces = HashMap::new();

    // `syminfo` and `timeframe` are always present in Pine, so an absent one
    // falls back to defaults.
    namespaces.insert(
        "syminfo".to_string(),
        syminfo::create_syminfo(syminfo.unwrap_or_default()),
    );
    namespaces.insert(
        "timeframe".to_string(),
        timeframe::register(timeframe.unwrap_or_default()),
    );

    // Register namespace objects
    namespaces.insert("array".to_string(), array::register());
    namespaces.insert("box".to_string(), r#box::register());
    namespaces.insert("color".to_string(), color::register());
    namespaces.insert("currency".to_string(), currency::register());
    for (name, value) in input::register(version) {
        namespaces.insert(name, value);
    }
    namespaces.insert("label".to_string(), label::register());
    for (name, value) in line::register(version) {
        namespaces.insert(name, value);
    }
    namespaces.insert("table".to_string(), table::register());
    namespaces.insert("indicator".to_string(), indicator::register());
    for (name, value) in globals::register() {
        namespaces.insert(name, value);
    }

    // Constant-only namespaces (string tags used as arguments elsewhere).
    namespaces.insert("size".to_string(), constants::size::register());
    namespaces.insert("shape".to_string(), constants::shape::register());
    namespaces.insert("location".to_string(), constants::location::register());
    namespaces.insert("position".to_string(), constants::position::register());
    namespaces.insert("display".to_string(), constants::display::register());
    namespaces.insert("format".to_string(), constants::format::register());
    namespaces.insert("log".to_string(), log::register());
    namespaces.insert("math".to_string(), math::register());
    namespaces.insert("matrix".to_string(), matrix::register());
    namespaces.insert("str".to_string(), str::register());
    namespaces.insert("ta".to_string(), ta::register());

    // Register global builtin functions
    namespaces.insert(
        "na".to_string(),
        Value::BuiltinFunction(Rc::new(Na::<O>::builtin_fn)),
    );
    namespaces.insert(
        "bool".to_string(),
        Value::BuiltinFunction(Rc::new(Bool::<O>::builtin_fn)),
    );
    namespaces.insert(
        "int".to_string(),
        Value::BuiltinFunction(Rc::new(Int::<O>::builtin_fn)),
    );
    namespaces.insert(
        "float".to_string(),
        Value::BuiltinFunction(Rc::new(Float::<O>::builtin_fn)),
    );
    namespaces.insert(
        "nz".to_string(),
        Value::BuiltinFunction(Rc::new(Nz::<O>::builtin_fn)),
    );
    namespaces.insert(
        "fixnan".to_string(),
        Value::BuiltinFunction(Rc::new(Fixnan::<O>::builtin_fn)),
    );

    // Register time/date functions
    for (name, func) in time::register_time_functions() {
        namespaces.insert(name, func);
    }

    // Register plot functions
    for (name, func) in plot::register_plot_functions() {
        namespaces.insert(name, func);
    }

    namespaces
}

/// Per-bar variables, rebuilt for each [`Bar`] and registered before it executes.
///
/// The compile-time counterpart is [`register_namespace_objects`]; this holds the
/// namespaces whose values change every bar. For now that is only `barstate`.
pub fn register_per_bar<O: PineOutput>(bar: &Bar) -> Vec<(String, Value<O>)> {
    vec![("barstate".to_string(), barstate::register(bar))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::{EvaluatedArg, FunctionCallArgs};

    #[test]
    fn test_na() {
        let mut ctx = Interpreter::<DefaultPineOutput>::new();

        // Test with na value
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Na::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Test with number
        let args = vec![EvaluatedArg::Positional(Value::Number(42.0))];
        let result = Na::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test with string
        let args = vec![EvaluatedArg::Positional(Value::String("hello".to_string()))];
        let result = Na::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test with bool
        let args = vec![EvaluatedArg::Positional(Value::Bool(true))];
        let result = Na::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_bool() {
        let mut ctx = Interpreter::<DefaultPineOutput>::new();

        // Test number to bool
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Bool::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Bool(true));

        let args = vec![EvaluatedArg::Positional(Value::Number(0.0))];
        let result = Bool::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test na to bool
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Bool::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_int() {
        let mut ctx = Interpreter::<DefaultPineOutput>::new();

        // Test float to int (truncate)
        let args = vec![EvaluatedArg::Positional(Value::Number(5.7))];
        let result = Int::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(5.0));

        let args = vec![EvaluatedArg::Positional(Value::Number(-5.7))];
        let result = Int::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(-5.0));

        // Test bool to int
        let args = vec![EvaluatedArg::Positional(Value::Bool(true))];
        let result = Int::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test na to int
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Int::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Na);
    }

    #[test]
    fn test_float() {
        let mut ctx = Interpreter::<DefaultPineOutput>::new();

        // Test number to float
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Float::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test bool to float
        let args = vec![EvaluatedArg::Positional(Value::Bool(true))];
        let result = Float::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test na to float
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Float::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Na);
    }

    #[test]
    fn test_nz() {
        let mut ctx = Interpreter::<DefaultPineOutput>::new();

        // Test na value without replacement (should return 0.0)
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Nz::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(0.0));

        // Test na value with replacement
        let args = vec![
            EvaluatedArg::Positional(Value::Na),
            EvaluatedArg::Positional(Value::Number(42.0)),
        ];
        let result = Nz::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(42.0));

        // Test non-na value (should return source)
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Nz::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_fixnan() {
        let mut ctx = Interpreter::<DefaultPineOutput>::new();

        // Test na value
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Fixnan::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(0.0));

        // Test normal value
        let args = vec![EvaluatedArg::Positional(Value::Number(5.0))];
        let result = Fixnan::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test NaN value
        let args = vec![EvaluatedArg::Positional(Value::Number(f64::NAN))];
        let result = Fixnan::builtin_fn(&mut ctx, FunctionCallArgs::without_types(args)).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }
}
