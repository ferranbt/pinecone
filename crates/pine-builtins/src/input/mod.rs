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
        // Constraints and dropdown options refine the settings UI only; they do
        // not change the value handed to the script.
        let _ = (
            self.minval,
            self.maxval,
            self.step,
            &self.tooltip,
            &self.options,
        );
        ctx.output.add_input(Input {
            kind: "int".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Int(self.defval as i64),
        });
        Ok(Value::Number(self.defval.trunc()))
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
        let _ = (
            self.minval,
            self.maxval,
            self.step,
            &self.tooltip,
            &self.options,
        );
        ctx.output.add_input(Input {
            kind: "float".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Float(self.defval),
        });
        Ok(Value::Number(self.defval))
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
        ctx.output.add_input(Input {
            kind: "bool".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Bool(self.defval),
        });
        Ok(Value::Bool(self.defval))
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
        let _ = (&self.tooltip, &self.options);
        ctx.output.add_input(Input {
            kind: "string".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Str(self.defval.clone()),
        });
        Ok(Value::String(self.defval.clone()))
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
        let _ = (&self.tooltip, &self.options);
        ctx.output.add_input(Input {
            kind: "session".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Str(self.defval.clone()),
        });
        Ok(Value::String(self.defval.clone()))
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
        ctx.output.add_input(Input {
            kind: "time".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Int(self.defval as i64),
        });
        Ok(Value::Number(self.defval))
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
            default: InputValue::Str(series_id),
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
        ctx.output.add_input(Input {
            kind: "price".to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default: InputValue::Float(self.defval),
        });
        Ok(Value::Number(self.defval))
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
                ctx.output.add_input(Input {
                    kind: $kind.to_string(),
                    title: self.title.clone(),
                    group: self.group.clone(),
                    default: InputValue::Str(self.defval.clone()),
                });
                Ok(Value::String(self.defval.clone()))
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
            default: InputValue::Str(member),
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
        let (kind, default) = match &self.defval {
            Value::Bool(b) => ("bool", InputValue::Bool(*b)),
            Value::Int(n) => ("int", InputValue::Int(*n)),
            Value::Number(n) => ("float", InputValue::Float(*n)),
            Value::String(s) => ("string", InputValue::Str(s.clone())),
            Value::Color(c) => ("color", InputValue::Color(c.clone())),
            _ => ("source", InputValue::Str(String::new())),
        };
        ctx.output.add_input(Input {
            kind: kind.to_string(),
            title: self.title.clone(),
            group: self.group.clone(),
            default,
        });
        Ok(self.defval.clone())
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
