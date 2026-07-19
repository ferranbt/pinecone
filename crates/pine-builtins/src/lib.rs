use pine_builtin_macro::BuiltinFunction;
use pine_core::Timeframe;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::collections::HashMap;
use std::rc::Rc;

// Re-export for convenience
pub use pine_interpreter::Bar;
pub use pine_interpreter::BuiltinFn;
pub use pine_interpreter::DefaultPineOutput;
pub use pine_interpreter::EvaluatedArg;
pub use pine_interpreter::LogLevel;

pub use bars::register_bar_state;

// Namespace modules
mod array;
mod bars;
mod r#box;
mod color;
mod constants;
mod currency;
mod hline;
mod input;
mod label;
mod log;
mod math;
mod matrix;
mod plot;
mod str;
mod ta;
mod time;
mod timeframe;

// Global utility functions - defined first so they can be referenced in register function

/// na(value) - Returns true if the value is na, false otherwise
#[derive(BuiltinFunction)]
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
struct Int {
    x: Value,
}

impl Int {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
struct Float {
    x: Value,
}

impl Float {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
            Value::Number(n) if n.is_nan() => Ok(Value::Number(0.0)),
            _ => Ok(self.source.clone()),
        }
    }
}

/// Functions v4 exposes bare that v5 moved into a namespace, as
/// `(namespace, member)`. Both spellings share one implementation.
const V4_FLAT_ALIASES: &[(&str, &str)] = &[
    ("math", "abs"),
    ("math", "asin"),
    ("math", "cos"),
    ("math", "exp"),
    ("math", "floor"),
    ("math", "max"),
    ("math", "min"),
    ("math", "pow"),
    ("math", "round"),
    ("math", "sin"),
    ("math", "sqrt"),
    ("math", "sum"),
    ("str", "tonumber"),
    ("str", "tostring"),
    ("ta", "change"),
    ("ta", "crossover"),
    ("ta", "crossunder"),
    ("ta", "dev"),
    ("ta", "ema"),
    ("ta", "highest"),
    ("ta", "lowest"),
    ("ta", "roc"),
    ("ta", "rsi"),
    ("ta", "sma"),
    ("ta", "variance"),
];

/// Register all builtin namespaces as objects and global functions
/// Returns namespace objects to be loaded as variables (e.g., "array", "str", "ta")
/// and global builtin functions (e.g., "na")
/// Each member stores the builtin function pointer as Value::BuiltinFunction
///
/// This uses DefaultPineOutput for now. Full generic support will be added when the
/// BuiltinFunction macro is updated to support generic output types.
pub fn register_namespace_objects(timeframe: Timeframe) -> HashMap<String, Value<DefaultPineOutput>> {
    let mut namespaces = HashMap::new();

    // Register namespace objects
    namespaces.insert("array".to_string(), array::register());
    namespaces.insert("box".to_string(), r#box::register());
    namespaces.insert("color".to_string(), color::register());
    namespaces.insert("currency".to_string(), currency::register());
    namespaces.insert("label".to_string(), label::register());
    namespaces.insert("log".to_string(), log::register::<DefaultPineOutput>());
    namespaces.insert("input".to_string(), input::register());
    namespaces.insert("math".to_string(), math::register());
    namespaces.insert("matrix".to_string(), matrix::register());
    namespaces.insert("str".to_string(), str::register());
    namespaces.insert("ta".to_string(), ta::register());

    // Register global builtin functions
    namespaces.insert(
        "na".to_string(),
        Value::BuiltinFunction(Rc::new(Na::builtin_fn) as BuiltinFn<DefaultPineOutput>),
    );
    namespaces.insert(
        "bool".to_string(),
        Value::BuiltinFunction(Rc::new(Bool::builtin_fn) as BuiltinFn<DefaultPineOutput>),
    );
    namespaces.insert(
        "int".to_string(),
        Value::BuiltinFunction(Rc::new(Int::builtin_fn) as BuiltinFn<DefaultPineOutput>),
    );
    namespaces.insert(
        "float".to_string(),
        Value::BuiltinFunction(Rc::new(Float::builtin_fn) as BuiltinFn<DefaultPineOutput>),
    );
    namespaces.insert(
        "nz".to_string(),
        Value::BuiltinFunction(Rc::new(Nz::builtin_fn) as BuiltinFn<DefaultPineOutput>),
    );
    namespaces.insert(
        "fixnan".to_string(),
        Value::BuiltinFunction(Rc::new(Fixnan::builtin_fn) as BuiltinFn<DefaultPineOutput>),
    );

    // Register time/date functions
    for (name, func) in time::register_time_functions() {
        namespaces.insert(name, func);
    }

    // Register plot functions
    for (name, func) in plot::register_plot_functions() {
        namespaces.insert(name, func);
    }

    // Constant-only namespaces (size.small, shape.circle, ...)
    for (name, value) in constants::register::<DefaultPineOutput>() {
        namespaces.insert(name, value);
    }

    namespaces.insert("hline".to_string(), hline::register());
    namespaces.insert(
        "timeframe".to_string(),
        timeframe::register(timeframe),
    );

    // `line` is a namespace (`line.new()`) that is also callable in v4.
    // Drawing is not implemented, so the members return na.
    namespaces.insert(
        "line".to_string(),
        callable_stub_namespace(
            "line",
            &[
                "delete", "get_price", "get_x1", "get_x2", "get_y1", "get_y2", "new", "set_color",
                "set_extend", "set_style", "set_width", "set_x1", "set_x2", "set_xloc", "set_xy1",
                "set_xy2", "set_y1", "set_y2",
            ],
        ),
    );

    // `bar_index` is the bar counter; we do not track it yet.
    namespaces.insert("bar_index".to_string(), Value::Na);

    // `barstate` describes the bar being executed, so it is registered per bar
    // by `register_bar_state` rather than here.
    let (name, value) = bars::register_bar_state(&Bar::default());
    namespaces.insert(name.to_string(), value);

    // v5 moved these into namespaces (`sma` -> `ta.sma`). v4 scripts call them
    // bare, so alias them to the very same implementation.
    let aliases: Vec<(String, Value<DefaultPineOutput>)> = V4_FLAT_ALIASES
        .iter()
        .filter_map(|(namespace, member)| {
            let Some(Value::Object { fields, .. }) = namespaces.get(*namespace) else {
                return None;
            };
            let value = fields.borrow().get(*member)?.clone();
            Some((member.to_string(), value))
        })
        .collect();
    for (name, value) in aliases {
        namespaces.insert(name, value);
    }

    // `time` is both the bar's timestamp and the `time(timeframe)` function.
    // `Bar` carries no timestamp yet, so it is na for now.
    namespaces.insert("time".to_string(), Value::Na);

    // Symbol information; not modelled, so members are na.
    namespaces.insert(
        "syminfo".to_string(),
        constants::stub_namespace(
            "syminfo",
            &[
                "basecurrency",
                "currency",
                "description",
                "mintick",
                "pointvalue",
                "prefix",
                "root",
                "session",
                "ticker",
                "tickerid",
                "timezone",
                "type",
            ],
        ),
    );

    // Accepted but not producing chart output yet. `study` is v4's `indicator`,
    // `security` is v4's `request.security`; the rest are v4 `ta.*` functions we
    // have not implemented.
    for name in [
        "indicator",
        "study",
        "alertcondition",
        "fill",
        "bgcolor",
        "barcolor",
        "timestamp",
        "timenow",
        "security",
        "cum",
        "mfi",
        "stoch",
        "pvt",
        "percentile_nearest_rank",
        "valuewhen",
    ] {
        let stub: BuiltinFn<DefaultPineOutput> = Rc::new(|_ctx, _args| Ok(Value::Na));
        namespaces.insert(name.to_string(), Value::BuiltinFunction(stub));
    }

    namespaces
}

/// A namespace whose members are functions returning na, and which is itself
/// callable. For built-ins we accept but have not implemented.
fn callable_stub_namespace(name: &str, members: &[&str]) -> Value<DefaultPineOutput> {
    fn stub() -> BuiltinFn<DefaultPineOutput> {
        Rc::new(|_ctx, _args| Ok(Value::Na))
    }

    let fields = members
        .iter()
        .map(|m| (m.to_string(), Value::BuiltinFunction(stub())))
        .collect();

    Value::Object {
        type_name: name.to_string(),
        fields: std::rc::Rc::new(std::cell::RefCell::new(fields)),
        call: Some(stub()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::{EvaluatedArg, FunctionCallArgs};

    #[test]
    fn test_na() {
        let mut ctx = Interpreter::new();

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
        let mut ctx = Interpreter::new();

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
        let mut ctx = Interpreter::new();

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
        let mut ctx = Interpreter::new();

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
        let mut ctx = Interpreter::new();

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
        let mut ctx = Interpreter::new();

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

