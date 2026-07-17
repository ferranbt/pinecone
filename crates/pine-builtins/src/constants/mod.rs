use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Namespaces that are nothing but named constants, e.g. `size.small`.
///
/// A member's value is its own name, which is the vocabulary the plot builtins
/// already expect (`plotshape(style = shape.circle)` → `"circle"`).
const CONSTANT_NAMESPACES: &[(&str, &[&str])] = &[
    (
        "size",
        &["auto", "huge", "large", "normal", "small", "tiny"],
    ),
    (
        "shape",
        &[
            "arrowdown",
            "arrowup",
            "circle",
            "cross",
            "diamond",
            "flag",
            "labeldown",
            "labelup",
            "square",
            "triangledown",
            "triangleup",
            "xcross",
        ],
    ),
    (
        "location",
        &["abovebar", "absolute", "belowbar", "bottom", "top"],
    ),
    (
        "display",
        &[
            "all",
            "data_window",
            "none",
            "pane",
            "pine_screener",
            "price_scale",
            "status_line",
        ],
    ),
    (
        "format",
        &["inherit", "mintick", "percent", "price", "volume"],
    ),
];

/// Build one namespace of string constants.
pub fn namespace<O: PineOutput>(name: &str, members: &[&str]) -> Value<O> {
    let fields: HashMap<String, Value<O>> = members
        .iter()
        .map(|m| (m.to_string(), Value::String(m.to_string())))
        .collect();

    Value::Object {
        type_name: name.to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}

/// Every constant-only namespace, ready to register.
pub fn register<O: PineOutput>() -> Vec<(String, Value<O>)> {
    CONSTANT_NAMESPACES
        .iter()
        .map(|(name, members)| (name.to_string(), namespace(name, members)))
        .collect()
}

/// A namespace whose members exist but hold `na`, for chart context we do not
/// model yet (`timeframe.period`, `barstate.islast`).
pub fn stub_namespace<O: PineOutput>(name: &str, members: &[&str]) -> Value<O> {
    let fields: HashMap<String, Value<O>> = members
        .iter()
        .map(|m| (m.to_string(), Value::Na))
        .collect();

    Value::Object {
        type_name: name.to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}
