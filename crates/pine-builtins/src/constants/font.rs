use pine_core::PineOutput;
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `font.*` constants (font families for label/table/box text).
const FONTS: &[&str] = &["family_default", "family_monospace"];

/// Register the font namespace with all font constants.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    for font in FONTS {
        members.insert(font.to_string(), Value::String(font.to_string()));
    }

    Value::Object {
        type_name: "font".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
