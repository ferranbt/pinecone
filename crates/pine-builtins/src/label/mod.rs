use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// label.new() - Creates a new label object
#[derive(BuiltinFunction)]
#[builtin(name = "label.new")]
struct LabelNew {
    x: Value,
    y: Value,
    #[arg(default = Value::String(String::new()))]
    text: Value,
    #[arg(default = Value::String("bar_index".to_string()))]
    xloc: Value,
    #[arg(default = Value::String("price".to_string()))]
    yloc: Value,
    #[arg(default = Value::Na)]
    color: Value,
    #[arg(default = Value::String("style_label_down".to_string()))]
    style: Value,
    #[arg(default = Value::Na)]
    textcolor: Value,
    #[arg(default = Value::String("normal".to_string()))]
    size: Value,
    #[arg(default = Value::String("center".to_string()))]
    textalign: Value,
    #[arg(default = Value::Na)]
    tooltip: Value,
    #[arg(default = Value::String("default".to_string()))]
    text_font_family: Value,
}

impl LabelNew {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // Create a label object with all the properties
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), self.x.clone());
        fields.insert("y".to_string(), self.y.clone());
        fields.insert("text".to_string(), self.text.clone());
        fields.insert("xloc".to_string(), self.xloc.clone());
        fields.insert("yloc".to_string(), self.yloc.clone());
        fields.insert("color".to_string(), self.color.clone());
        fields.insert("style".to_string(), self.style.clone());
        fields.insert("textcolor".to_string(), self.textcolor.clone());
        fields.insert("size".to_string(), self.size.clone());
        fields.insert("textalign".to_string(), self.textalign.clone());
        fields.insert("tooltip".to_string(), self.tooltip.clone());
        fields.insert(
            "text_font_family".to_string(),
            self.text_font_family.clone(),
        );

        Ok(Value::Object {
            type_name: "label".to_string(),
            fields: Rc::new(RefCell::new(fields)),
        })
    }
}

/// label.set_x() - Sets the x coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_x")]
struct LabelSetX {
    id: Value,
    x: Value,
}

impl LabelSetX {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields.borrow_mut().insert("x".to_string(), self.x.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_y() - Sets the y coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_y")]
struct LabelSetY {
    id: Value,
    y: Value,
}

impl LabelSetY {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields.borrow_mut().insert("y".to_string(), self.y.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_xy() - Sets the x and y coordinates of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_xy")]
struct LabelSetXy {
    id: Value,
    x: Value,
    y: Value,
}

impl LabelSetXy {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            let mut fields_mut = fields.borrow_mut();
            fields_mut.insert("x".to_string(), self.x.clone());
            fields_mut.insert("y".to_string(), self.y.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_xloc() - Sets the x location mode of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_xloc")]
struct LabelSetXloc {
    id: Value,
    x: Value,
    xloc: Value,
}

impl LabelSetXloc {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            let mut fields_mut = fields.borrow_mut();
            fields_mut.insert("x".to_string(), self.x.clone());
            fields_mut.insert("xloc".to_string(), self.xloc.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_yloc() - Sets the y location mode of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_yloc")]
struct LabelSetYloc {
    id: Value,
    yloc: Value,
}

impl LabelSetYloc {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("yloc".to_string(), self.yloc.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_color() - Sets the color of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_color")]
struct LabelSetColor {
    id: Value,
    color: Value,
}

impl LabelSetColor {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("color".to_string(), self.color.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_style() - Sets the style of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_style")]
struct LabelSetStyle {
    id: Value,
    style: Value,
}

impl LabelSetStyle {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("style".to_string(), self.style.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_text() - Sets the text of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_text")]
struct LabelSetText {
    id: Value,
    text: Value,
}

impl LabelSetText {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text".to_string(), self.text.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_textcolor() - Sets the text color of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_textcolor")]
struct LabelSetTextcolor {
    id: Value,
    textcolor: Value,
}

impl LabelSetTextcolor {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("textcolor".to_string(), self.textcolor.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_size() - Sets the size of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_size")]
struct LabelSetSize {
    id: Value,
    size: Value,
}

impl LabelSetSize {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("size".to_string(), self.size.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_textalign() - Sets the text alignment of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_textalign")]
struct LabelSetTextalign {
    id: Value,
    textalign: Value,
}

impl LabelSetTextalign {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("textalign".to_string(), self.textalign.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_tooltip() - Sets the tooltip of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_tooltip")]
struct LabelSetTooltip {
    id: Value,
    tooltip: Value,
}

impl LabelSetTooltip {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("tooltip".to_string(), self.tooltip.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.set_text_font_family() - Sets the font family of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_text_font_family")]
struct LabelSetTextFontFamily {
    id: Value,
    text_font_family: Value,
}

impl LabelSetTextFontFamily {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields.borrow_mut().insert(
                "text_font_family".to_string(),
                self.text_font_family.clone(),
            );
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.get_x() - Gets the x coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_x")]
struct LabelGetX {
    id: Value,
}

impl LabelGetX {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            Ok(fields.borrow().get("x").cloned().unwrap_or(Value::Na))
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.get_y() - Gets the y coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_y")]
struct LabelGetY {
    id: Value,
}

impl LabelGetY {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            Ok(fields.borrow().get("y").cloned().unwrap_or(Value::Na))
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.get_text() - Gets the text of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_text")]
struct LabelGetText {
    id: Value,
}

impl LabelGetText {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            Ok(fields.borrow().get("text").cloned().unwrap_or(Value::Na))
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// label.delete() - Deletes a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.delete")]
struct LabelDelete {
    _id: Value,
}

impl LabelDelete {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // For now, just return na - actual deletion would be handled by state management
        Ok(Value::Na)
    }
}

/// label.copy() - Copies a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.copy")]
struct LabelCopy {
    id: Value,
}

impl LabelCopy {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { type_name, fields } = &self.id {
            // Create a deep copy of the fields
            let copied_fields = fields.borrow().clone();
            Ok(Value::Object {
                type_name: type_name.clone(),
                fields: Rc::new(RefCell::new(copied_fields)),
            })
        } else {
            Err(RuntimeError::TypeError("Expected label object".to_string()))
        }
    }
}

/// Register the label namespace with all label style constants and functions
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

    // Register functions
    members.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(LabelNew::builtin_fn)),
    );
    members.insert(
        "set_x".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetX::builtin_fn)),
    );
    members.insert(
        "set_y".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetY::builtin_fn)),
    );
    members.insert(
        "set_xy".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetXy::builtin_fn)),
    );
    members.insert(
        "set_xloc".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetXloc::builtin_fn)),
    );
    members.insert(
        "set_yloc".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetYloc::builtin_fn)),
    );
    members.insert(
        "set_color".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetColor::builtin_fn)),
    );
    members.insert(
        "set_style".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetStyle::builtin_fn)),
    );
    members.insert(
        "set_text".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetText::builtin_fn)),
    );
    members.insert(
        "set_textcolor".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetTextcolor::builtin_fn)),
    );
    members.insert(
        "set_size".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetSize::builtin_fn)),
    );
    members.insert(
        "set_textalign".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetTextalign::builtin_fn)),
    );
    members.insert(
        "set_tooltip".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetTooltip::builtin_fn)),
    );
    members.insert(
        "set_text_font_family".to_string(),
        Value::BuiltinFunction(Rc::new(LabelSetTextFontFamily::builtin_fn)),
    );
    members.insert(
        "get_x".to_string(),
        Value::BuiltinFunction(Rc::new(LabelGetX::builtin_fn)),
    );
    members.insert(
        "get_y".to_string(),
        Value::BuiltinFunction(Rc::new(LabelGetY::builtin_fn)),
    );
    members.insert(
        "get_text".to_string(),
        Value::BuiltinFunction(Rc::new(LabelGetText::builtin_fn)),
    );
    members.insert(
        "delete".to_string(),
        Value::BuiltinFunction(Rc::new(LabelDelete::builtin_fn)),
    );
    members.insert(
        "copy".to_string(),
        Value::BuiltinFunction(Rc::new(LabelCopy::builtin_fn)),
    );

    Value::Object {
        type_name: "label".to_string(),
        fields: Rc::new(RefCell::new(members)),
    }
}
