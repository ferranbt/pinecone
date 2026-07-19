use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{BuiltinFn, DefaultPineOutput, Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `input.*` members. They all share one implementation because every
/// input returns its `defval` until a user overrides it in the Settings tab,
/// which we do not model.
const MEMBERS: &[&str] = &[
    "bool", "color", "enum", "float", "int", "price", "session", "source", "string", "symbol",
    "text_area", "time", "timeframe",
];

/// input(defval, ...) — returns the default value.
///
/// The fields past `defval` exist so calls naming them parse; only `defval`
/// affects the result.
#[allow(dead_code)]
#[derive(BuiltinFunction)]
struct Input {
    defval: Value,
    #[arg(default = Value::Na)]
    title: Value,
    #[arg(default = Value::Na)]
    group: Value,
    #[arg(default = Value::Na)]
    tooltip: Value,
    #[arg(default = Value::Na)]
    inline: Value,
    #[arg(default = Value::Na)]
    minval: Value,
    #[arg(default = Value::Na)]
    maxval: Value,
    #[arg(default = Value::Na)]
    step: Value,
    #[arg(default = Value::Na)]
    options: Value,
    #[arg(default = Value::Na)]
    display: Value,
    #[arg(default = Value::Na)]
    confirm: Value,
}

impl Input {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(self.defval.clone())
    }
}

/// `input` is both callable (`input(true)`) and a namespace (`input.int(2)`).
pub fn register() -> Value<DefaultPineOutput> {
    let input_fn = || Rc::new(Input::builtin_fn) as BuiltinFn<DefaultPineOutput>;

    let mut input_ns = HashMap::new();
    for name in MEMBERS {
        input_ns.insert(name.to_string(), Value::BuiltinFunction(input_fn()));
    }

    Value::Object {
        type_name: "input".to_string(),
        fields: Rc::new(RefCell::new(input_ns)),
        call: Some(input_fn()),
    }
}
