//! The `input.*` namespace.
//!
//! `input` changed shape between major versions, so [`register`] dispatches on
//! it: v5/v6 use the namespaced functions in this module; v3/v4 use the single
//! overloaded `input(...)` in [`legacy`].
//!
//! In a headless interpreter there is no settings UI, so each function returns
//! its default value so the script can run. It also records the declaration into
//! the output (via [`InputOutput`]) so a host can enumerate a script's inputs
//! without executing it.

mod legacy;

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineVersion;
use pine_core::{Color, Input, InputOutput, InputValue, PineOutput};
use pine_interpreter::{Builtin, BuiltinFn, Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The effective numeric value: a host override for `title`, clamped to
/// `[minval, maxval]`, if present and numeric; otherwise `default`.
fn num_input<O: PineOutput>(
    ctx: &Interpreter<O>,
    title: &str,
    default: f64,
    minval: Option<f64>,
    maxval: Option<f64>,
) -> f64 {
    let mut value = match ctx.input(title) {
        Some(InputValue::Int(n)) => *n as f64,
        Some(InputValue::Float(f)) => *f,
        _ => return default,
    };
    if let Some(lo) = minval {
        value = value.max(lo);
    }
    if let Some(hi) = maxval {
        value = value.min(hi);
    }
    value
}

/// The effective boolean value: a boolean override for `title`, else `default`.
fn bool_input<O: PineOutput>(ctx: &Interpreter<O>, title: &str, default: bool) -> bool {
    match ctx.input(title) {
        Some(InputValue::Bool(b)) => *b,
        _ => default,
    }
}

/// The effective string value: a string override for `title` if present and, when
/// `options` is a list, one of its members; otherwise `default`.
fn string_input<O: PineOutput>(
    ctx: &Interpreter<O>,
    title: &str,
    default: &str,
    options: &Value<O>,
) -> String {
    let InputValue::Str(value) = (match ctx.input(title) {
        Some(v) => v,
        None => return default.to_string(),
    }) else {
        return default.to_string();
    };
    if let Value::Array(list) = options {
        let allowed = list
            .borrow()
            .iter()
            .any(|o| matches!(o, Value::String(s) if s == value));
        if !allowed {
            return default.to_string();
        }
    }
    value.clone()
}

/// Every name the `input` namespace contributes, chosen by version: the v5/v6
/// namespaced functions, or the v3/v4 overloaded `input()` plus its type
/// constants.
pub fn register<O: PineOutput + InputOutput>(version: PineVersion) -> Vec<(String, Value<O>)> {
    if version >= PineVersion::V5 {
        vec![("input".to_string(), register_v56())]
    } else {
        legacy::register()
    }
}

/// input.int(defval, title, minval, maxval, step, group, tooltip, options)
#[derive(BuiltinFunction)]
#[builtin(name = "input.int")]
struct InputInt<O: PineOutput + InputOutput> {
    defval: f64,
    #[arg(default = "")]
    title: String,
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
}

impl<O: PineOutput + InputOutput> InputInt<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = (&self.tooltip, &self.options);
        let value = num_input(ctx, &self.title, self.defval, self.minval, self.maxval).trunc();
        ctx.output.add_input(Input {
            kind: "int".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Int(self.defval as i64),
            value: InputValue::Int(value as i64),
            min_val: self.minval,
            max_val: self.maxval,
            step: self.step,
        });
        Ok(Value::Number(value))
    }
}

/// input.float(defval, title, minval, maxval, step, group, tooltip, options)
#[derive(BuiltinFunction)]
#[builtin(name = "input.float")]
struct InputFloat<O: PineOutput + InputOutput> {
    defval: f64,
    #[arg(default = "")]
    title: String,
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
}

impl<O: PineOutput + InputOutput> InputFloat<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = (&self.tooltip, &self.options);
        let value = num_input(ctx, &self.title, self.defval, self.minval, self.maxval);
        ctx.output.add_input(Input {
            kind: "float".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Float(self.defval),
            value: InputValue::Float(value),
            min_val: self.minval,
            max_val: self.maxval,
            step: self.step,
        });
        Ok(Value::Number(value))
    }
}

/// input.bool(defval, title, group, tooltip)
#[derive(BuiltinFunction)]
#[builtin(name = "input.bool", output = InputOutput)]
struct InputBool {
    defval: bool,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
}

impl InputBool {
    fn execute<O: PineOutput + InputOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        let value = bool_input(ctx, &self.title, self.defval);
        ctx.output.add_input(Input {
            kind: "bool".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Bool(self.defval),
            value: InputValue::Bool(value),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(Value::Bool(value))
    }
}

/// input.string(defval, title, group, tooltip, options)
#[derive(BuiltinFunction)]
#[builtin(name = "input.string")]
struct InputString<O: PineOutput + InputOutput> {
    defval: String,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
    #[arg(default = Value::Na)]
    options: Value<O>,
}

impl<O: PineOutput + InputOutput> InputString<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        let value = string_input(ctx, &self.title, &self.defval, &self.options);
        ctx.output.add_input(Input {
            kind: "string".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Str(self.defval.clone()),
            value: InputValue::Str(value.clone()),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(Value::String(value))
    }
}

/// input.session(defval, title, group, tooltip, options)
#[derive(BuiltinFunction)]
#[builtin(name = "input.session")]
struct InputSession<O: PineOutput + InputOutput> {
    defval: String,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
    #[arg(default = Value::Na)]
    options: Value<O>,
}

impl<O: PineOutput + InputOutput> InputSession<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        let value = string_input(ctx, &self.title, &self.defval, &self.options);
        ctx.output.add_input(Input {
            kind: "session".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Str(self.defval.clone()),
            value: InputValue::Str(value.clone()),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(Value::String(value))
    }
}

/// input.color(defval, title, group, tooltip)
#[derive(BuiltinFunction)]
#[builtin(name = "input.color", output = InputOutput)]
struct InputColor {
    defval: Color,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
}

impl InputColor {
    fn execute<O: PineOutput + InputOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        ctx.output.add_input(Input {
            kind: "color".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Color(self.defval.clone()),
            value: InputValue::Color(self.defval.clone()),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(Value::Color(self.defval.clone()))
    }
}

/// input.time(defval, title, group, tooltip)
#[derive(BuiltinFunction)]
#[builtin(name = "input.time", output = InputOutput)]
struct InputTime {
    defval: f64,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
}

impl InputTime {
    fn execute<O: PineOutput + InputOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        let value = num_input(ctx, &self.title, self.defval, None, None);
        ctx.output.add_input(Input {
            kind: "time".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Int(self.defval as i64),
            value: InputValue::Int(value as i64),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(Value::Number(value))
    }
}

/// input.source(defval, title, group, tooltip)
#[derive(BuiltinFunction)]
#[builtin(name = "input.source")]
struct InputSource<O: PineOutput + InputOutput> {
    defval: Value<O>,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
}

impl<O: PineOutput + InputOutput> InputSource<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        // The default is a series (e.g. `close`); record its id for reference.
        let series_id = match &self.defval {
            Value::Series(series) => series.id.clone(),
            _ => "source".to_string(),
        };
        ctx.output.add_input(Input {
            kind: "source".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Str(series_id.clone()),
            value: InputValue::Str(series_id),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(self.defval.clone())
    }
}

/// input.price(defval, ...) - A price input; returns its default.
#[derive(BuiltinFunction)]
#[builtin(name = "input.price")]
struct InputPrice<O: PineOutput + InputOutput> {
    defval: f64,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
    #[arg(default = Value::Na)]
    options: Value<O>,
}

impl<O: PineOutput + InputOutput> InputPrice<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = (&self.tooltip, &self.options);
        let value = num_input(ctx, &self.title, self.defval, None, None);
        ctx.output.add_input(Input {
            kind: "price".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Float(self.defval),
            value: InputValue::Float(value),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(Value::Number(value))
    }
}

/// Defines a string-valued input (`input.symbol`/`input.timeframe`/
/// `input.text_area`) that records a `$kind` widget and returns its default.
macro_rules! input_string_like {
    ($ident:ident, $name:literal, $kind:literal) => {
        #[derive(BuiltinFunction)]
        #[builtin(name = $name, output = InputOutput)]
        struct $ident {
            defval: String,
            #[arg(default = "")]
            title: String,
            #[arg(default = "")]
            group: String,
            #[arg(default = "")]
            tooltip: String,
        }

        impl $ident {
            fn execute<O: PineOutput + InputOutput>(
                &self,
                ctx: &mut Interpreter<O>,
            ) -> Result<Value<O>, RuntimeError> {
                let _ = &self.tooltip;
                let value = string_input(ctx, &self.title, &self.defval, &Value::Na);
                ctx.output.add_input(Input {
                    kind: $kind.to_string(),
                    title: self.title.clone(),
                    group: self.group.clone(),
                    default: InputValue::Str(self.defval.clone()),
                    value: InputValue::Str(value.clone()),
                    min_val: None,
                    max_val: None,
                    step: None,
                });
                Ok(Value::String(value))
            }
        }
    };
}

input_string_like!(InputSymbol, "input.symbol", "symbol");
input_string_like!(InputTimeframe, "input.timeframe", "timeframe");
input_string_like!(InputTextArea, "input.text_area", "text_area");

/// input.enum(defval, ...) - An enum input; returns its default member.
#[derive(BuiltinFunction)]
#[builtin(name = "input.enum")]
struct InputEnum<O: PineOutput + InputOutput> {
    defval: Value<O>,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
}

impl<O: PineOutput + InputOutput> InputEnum<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        let member = match &self.defval {
            Value::Enum { field_name, .. } => field_name.clone(),
            _ => String::new(),
        };
        ctx.output.add_input(Input {
            kind: "enum".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Str(member.clone()),
            value: InputValue::Str(member),
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(self.defval.clone())
    }
}

/// input(defval, ...) - The type-inferring shorthand; records the input and
/// returns `defval` unchanged, its type taken from the default's.
#[derive(BuiltinFunction)]
#[builtin(name = "input")]
struct InputAuto<O: PineOutput + InputOutput> {
    defval: Value<O>,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    group: String,
    #[arg(default = "")]
    tooltip: String,
}

impl<O: PineOutput + InputOutput> InputAuto<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        // The type is inferred from the default, and the override coerced to it.
        let (kind, default, effective, effective_value) = match &self.defval {
            Value::Bool(b) => {
                let v = bool_input(ctx, &self.title, *b);
                (
                    "bool",
                    InputValue::Bool(*b),
                    Value::Bool(v),
                    InputValue::Bool(v),
                )
            }
            Value::Int(n) => {
                let v = num_input(ctx, &self.title, *n as f64, None, None).trunc();
                (
                    "int",
                    InputValue::Int(*n),
                    Value::Number(v),
                    InputValue::Int(v as i64),
                )
            }
            Value::Number(n) => {
                let v = num_input(ctx, &self.title, *n, None, None);
                (
                    "float",
                    InputValue::Float(*n),
                    Value::Number(v),
                    InputValue::Float(v),
                )
            }
            Value::String(s) => {
                let v = string_input(ctx, &self.title, s, &Value::Na);
                (
                    "string",
                    InputValue::Str(s.clone()),
                    Value::String(v.clone()),
                    InputValue::Str(v),
                )
            }
            Value::Color(c) => (
                "color",
                InputValue::Color(c.clone()),
                Value::Color(c.clone()),
                InputValue::Color(c.clone()),
            ),
            other => (
                "source",
                InputValue::Str(String::new()),
                other.clone(),
                InputValue::Str(String::new()),
            ),
        };
        ctx.output.add_input(Input {
            kind: kind.to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default,
            value: effective_value,
            min_val: None,
            max_val: None,
            step: None,
        });
        Ok(effective)
    }
}

/// Build the `input` namespace object.
///
/// `input.integer` is v4's spelling of `input.int`; both share one
/// implementation.
/// The v5/v6 `input` object: type-specific member functions (`input.int(...)`),
/// and callable itself as the type-inferring `input(...)` shorthand.
fn register_v56<O: PineOutput + InputOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert("int".to_string(), InputInt::<O>::builtin_value());
    members.insert("integer".to_string(), InputInt::<O>::builtin_value());
    members.insert("float".to_string(), InputFloat::<O>::builtin_value());
    members.insert("bool".to_string(), InputBool::builtin_value::<O>());
    members.insert("string".to_string(), InputString::<O>::builtin_value());
    members.insert("session".to_string(), InputSession::<O>::builtin_value());
    members.insert("color".to_string(), InputColor::builtin_value::<O>());
    members.insert("time".to_string(), InputTime::builtin_value::<O>());
    members.insert("source".to_string(), InputSource::<O>::builtin_value());
    members.insert("price".to_string(), InputPrice::<O>::builtin_value());
    members.insert("symbol".to_string(), InputSymbol::builtin_value::<O>());
    members.insert(
        "timeframe".to_string(),
        InputTimeframe::builtin_value::<O>(),
    );
    members.insert("text_area".to_string(), InputTextArea::builtin_value::<O>());
    members.insert("enum".to_string(), InputEnum::<O>::builtin_value());

    Value::Object {
        type_name: "input".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: Some(Builtin {
            call: Rc::new(InputAuto::<O>::builtin_fn) as BuiltinFn<O>,
            signature: InputAuto::<O>::signature(),
        }),
        value: None,
    }
}
