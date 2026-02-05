use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Register the label namespace with all label style constants
pub fn register() -> Value {
    let mut members = HashMap::new();

    // All label style constants as string values
    let styles = [
        "style_arrowdown",
        "style_arrowup",
        "style_circle",
        "style_cross",
        "style_diamond",
        "style_flag",
        "style_label_center",
        "style_label_down",
        "style_label_left",
        "style_label_lower_left",
        "style_label_lower_right",
        "style_label_right",
        "style_label_up",
        "style_label_upper_left",
        "style_label_upper_right",
        "style_none",
        "style_square",
        "style_text_outline",
        "style_triangledown",
        "style_triangleup",
        "style_xcross",
    ];

    for style in styles {
        members.insert(style.to_string(), Value::String(style.to_string()));
    }

    Value::Object {
        type_name: "label".to_string(),
        fields: Rc::new(RefCell::new(members)),
    }
}
