//! The v3 / v4 `input` model.
//!
//! Unlike v5/v6's namespaced functions ([`super`]), v3/v4 expose a single
//! overloaded `input(defval, title, type, ...)` where the type is an *argument*
//! (`type=integer` in v3, `type=input.integer` in v4). So here `input` is a
//! callable object whose members (`input.integer`, `input.bool`, ...) are type
//! **constants**, not functions.
//!
//! In a headless run the type/minval/options only shape a settings UI, so
//! `input(...)` simply returns its default and records the declaration.

use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{
    BuiltinFn, Input, InputOutput, InputValue, Interpreter, PineOutput, RuntimeError, Value,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The type tags accepted as `input(..., type=<tag>)`, exposed as `input.<tag>`.
const TYPE_TAGS: &[&str] = &[
    "integer",
    "float",
    "bool",
    "string",
    "source",
    "color",
    "symbol",
    "session",
    "resolution",
    "time",
    "price",
];

/// v3's *unqualified* type tags (`type=integer`). Registered as globals, but
/// only names that do not collide with an existing builtin (`bool`/`int`/`float`
/// are already cast functions, so they are left alone and simply ignored when
/// they reach `input` as the `type` argument).
const GLOBAL_TYPE_TAGS: &[&str] = &["integer", "source", "symbol", "resolution", "price"];

/// The overloaded v3/v4 `input(defval, title, type, ...)`.
///
/// The type/constraint/options arguments only shape a settings UI, so they are
/// accepted and ignored; the default is returned and the declaration recorded.
#[derive(BuiltinFunction)]
#[builtin(name = "input")]
struct InputLegacy<O: PineOutput + InputOutput> {
    defval: Value<O>,
    #[arg(default = "")]
    title: String,
    // `type=integer`/`type=input.integer` arrive as a string tag, but v3's
    // `type=float`/`type=bool` resolve to the cast *functions*, so accept any
    // value and read the tag only when it is a string.
    #[arg(default = Value::Na)]
    r#type: Value<O>,
    #[arg(default = None)]
    minval: Option<f64>,
    #[arg(default = None)]
    maxval: Option<f64>,
    #[arg(default = None)]
    step: Option<f64>,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
    #[arg(default = Value::Na)]
    options: Value<O>,
    #[arg(default = false)]
    confirm: bool,
}

impl<O: PineOutput + InputOutput> InputLegacy<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // Settings-UI only; do not affect the value handed to the script.
        let _ = (
            self.minval,
            self.maxval,
            self.step,
            &self.tooltip,
            &self.options,
            self.confirm,
        );
        let kind = match &self.r#type {
            Value::String(tag) if !tag.is_empty() => tag.clone(),
            _ => infer_kind(&self.defval),
        };
        ctx.output.add_input(Input {
            title: self.title.clone(),
            group: self.group.clone(),
            default: to_input_value(&kind, &self.defval),
            kind,
        });
        Ok(self.defval.clone())
    }
}

/// Kind inferred from the default when no `type=` was given.
fn infer_kind<O: PineOutput>(defval: &Value<O>) -> String {
    match defval {
        Value::Bool(_) => "bool",
        Value::String(_) => "string",
        Value::Color(_) => "color",
        Value::Series(_) => "source",
        _ => "float",
    }
    .to_string()
}

fn to_input_value<O: PineOutput>(kind: &str, defval: &Value<O>) -> InputValue {
    match defval {
        Value::Number(n) if kind == "integer" || kind == "int" => InputValue::Int(*n as i64),
        Value::Number(n) => InputValue::Float(*n),
        Value::Bool(b) => InputValue::Bool(*b),
        Value::String(s) => InputValue::Str(s.clone()),
        Value::Color(c) => InputValue::Color(c.clone()),
        Value::Series(series) => InputValue::Str(series.id.clone()),
        other => InputValue::Str(format!("{:?}", other)),
    }
}

/// Every name the v3/v4 `input` model contributes: the callable `input` object
/// (with type-tag members) plus v3's unqualified `type=integer` globals.
pub fn register<O: PineOutput + InputOutput>() -> Vec<(String, Value<O>)> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    for tag in TYPE_TAGS {
        members.insert(tag.to_string(), Value::String(tag.to_string()));
    }
    let input_object = Value::Object {
        type_name: "input".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: Some(Rc::new(InputLegacy::<O>::builtin_fn) as BuiltinFn<O>),
    };

    let mut entries = vec![("input".to_string(), input_object)];
    for tag in GLOBAL_TYPE_TAGS {
        entries.push((tag.to_string(), Value::String(tag.to_string())));
    }
    entries
}
