use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Color, Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// box.new() - Creates a new box object
#[derive(BuiltinFunction)]
#[builtin(name = "box.new")]
struct BoxNew {
    left: Value,
    top: Value,
    right: Value,
    bottom: Value,
    #[arg(default = None)]
    border_color: Option<Color>,
    #[arg(default = 1.0)]
    border_width: f64,
    #[arg(default = "solid")]
    border_style: String,
    #[arg(default = "none")]
    extend: String,
    #[arg(default = "bar_index")]
    xloc: String,
    #[arg(default = None)]
    bgcolor: Option<Color>,
    #[arg(default = "")]
    text: String,
    #[arg(default = 0.0)]
    text_size: f64,
    #[arg(default = None)]
    text_color: Option<Color>,
    #[arg(default = "center")]
    text_halign: String,
    #[arg(default = "center")]
    text_valign: String,
    #[arg(default = "none")]
    text_wrap: String,
    #[arg(default = "default")]
    text_font_family: String,
}

impl BoxNew {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("left".to_string(), self.left.clone());
        fields.insert("top".to_string(), self.top.clone());
        fields.insert("right".to_string(), self.right.clone());
        fields.insert("bottom".to_string(), self.bottom.clone());
        fields.insert("border_color".to_string(), match &self.border_color {
            Some(c) => Value::Color(c.clone()),
            None => Value::Na,
        });
        fields.insert("border_width".to_string(), Value::Number(self.border_width));
        fields.insert("border_style".to_string(), Value::String(self.border_style.clone()));
        fields.insert("extend".to_string(), Value::String(self.extend.clone()));
        fields.insert("xloc".to_string(), Value::String(self.xloc.clone()));
        fields.insert("bgcolor".to_string(), match &self.bgcolor {
            Some(c) => Value::Color(c.clone()),
            None => Value::Na,
        });
        fields.insert("text".to_string(), Value::String(self.text.clone()));
        fields.insert("text_size".to_string(), Value::Number(self.text_size));
        fields.insert("text_color".to_string(), match &self.text_color {
            Some(c) => Value::Color(c.clone()),
            None => Value::Na,
        });
        fields.insert("text_halign".to_string(), Value::String(self.text_halign.clone()));
        fields.insert("text_valign".to_string(), Value::String(self.text_valign.clone()));
        fields.insert("text_wrap".to_string(), Value::String(self.text_wrap.clone()));
        fields.insert("text_font_family".to_string(), Value::String(self.text_font_family.clone()));

        Ok(Value::Object {
            type_name: "box".to_string(),
            fields: Rc::new(RefCell::new(fields)),
        })
    }
}

/// box.set_left() - Sets the left coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_left")]
struct BoxSetLeft {
    id: Value,
    left: Value,
}

impl BoxSetLeft {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("left".to_string(), self.left.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_top() - Sets the top coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_top")]
struct BoxSetTop {
    id: Value,
    top: Value,
}

impl BoxSetTop {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("top".to_string(), self.top.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_right() - Sets the right coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_right")]
struct BoxSetRight {
    id: Value,
    right: Value,
}

impl BoxSetRight {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("right".to_string(), self.right.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_bottom() - Sets the bottom coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_bottom")]
struct BoxSetBottom {
    id: Value,
    bottom: Value,
}

impl BoxSetBottom {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("bottom".to_string(), self.bottom.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_lefttop() - Sets the left and top coordinates
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_lefttop")]
struct BoxSetLefttop {
    id: Value,
    left: Value,
    top: Value,
}

impl BoxSetLefttop {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            let mut fields_mut = fields.borrow_mut();
            fields_mut.insert("left".to_string(), self.left.clone());
            fields_mut.insert("top".to_string(), self.top.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_rightbottom() - Sets the right and bottom coordinates
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_rightbottom")]
struct BoxSetRightbottom {
    id: Value,
    right: Value,
    bottom: Value,
}

impl BoxSetRightbottom {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            let mut fields_mut = fields.borrow_mut();
            fields_mut.insert("right".to_string(), self.right.clone());
            fields_mut.insert("bottom".to_string(), self.bottom.clone());
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_border_color() - Sets the border color
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_border_color")]
struct BoxSetBorderColor {
    id: Value,
    color: Color,
}

impl BoxSetBorderColor {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("border_color".to_string(), Value::Color(self.color.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_border_width() - Sets the border width
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_border_width")]
struct BoxSetBorderWidth {
    id: Value,
    width: f64,
}

impl BoxSetBorderWidth {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("border_width".to_string(), Value::Number(self.width));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_border_style() - Sets the border style
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_border_style")]
struct BoxSetBorderStyle {
    id: Value,
    style: String,
}

impl BoxSetBorderStyle {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("border_style".to_string(), Value::String(self.style.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_extend() - Sets the extend property
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_extend")]
struct BoxSetExtend {
    id: Value,
    extend: String,
}

impl BoxSetExtend {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("extend".to_string(), Value::String(self.extend.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_bgcolor() - Sets the background color
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_bgcolor")]
struct BoxSetBgcolor {
    id: Value,
    color: Color,
}

impl BoxSetBgcolor {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("bgcolor".to_string(), Value::Color(self.color.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_text() - Sets the text
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text")]
struct BoxSetText {
    id: Value,
    text: String,
}

impl BoxSetText {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text".to_string(), Value::String(self.text.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_text_color() - Sets the text color
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_color")]
struct BoxSetTextColor {
    id: Value,
    color: Color,
}

impl BoxSetTextColor {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text_color".to_string(), Value::Color(self.color.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_text_size() - Sets the text size
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_size")]
struct BoxSetTextSize {
    id: Value,
    size: f64,
}

impl BoxSetTextSize {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text_size".to_string(), Value::Number(self.size));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_text_halign() - Sets the text horizontal alignment
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_halign")]
struct BoxSetTextHalign {
    id: Value,
    halign: String,
}

impl BoxSetTextHalign {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text_halign".to_string(), Value::String(self.halign.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_text_valign() - Sets the text vertical alignment
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_valign")]
struct BoxSetTextValign {
    id: Value,
    valign: String,
}

impl BoxSetTextValign {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text_valign".to_string(), Value::String(self.valign.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_text_wrap() - Sets the text wrap mode
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_wrap")]
struct BoxSetTextWrap {
    id: Value,
    wrap: String,
}

impl BoxSetTextWrap {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text_wrap".to_string(), Value::String(self.wrap.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_text_font_family() - Sets the text font family
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_font_family")]
struct BoxSetTextFontFamily {
    id: Value,
    font_family: String,
}

impl BoxSetTextFontFamily {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            fields
                .borrow_mut()
                .insert("text_font_family".to_string(), Value::String(self.font_family.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.set_xloc() - Sets the xloc mode
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_xloc")]
struct BoxSetXloc {
    id: Value,
    left: Value,
    right: Value,
    xloc: String,
}

impl BoxSetXloc {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            let mut fields_mut = fields.borrow_mut();
            fields_mut.insert("left".to_string(), self.left.clone());
            fields_mut.insert("right".to_string(), self.right.clone());
            fields_mut.insert("xloc".to_string(), Value::String(self.xloc.clone()));
            Ok(Value::Na)
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.get_left() - Gets the left coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_left")]
struct BoxGetLeft {
    id: Value,
}

impl BoxGetLeft {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            Ok(fields.borrow().get("left").cloned().unwrap_or(Value::Na))
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.get_top() - Gets the top coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_top")]
struct BoxGetTop {
    id: Value,
}

impl BoxGetTop {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            Ok(fields.borrow().get("top").cloned().unwrap_or(Value::Na))
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.get_right() - Gets the right coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_right")]
struct BoxGetRight {
    id: Value,
}

impl BoxGetRight {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            Ok(fields.borrow().get("right").cloned().unwrap_or(Value::Na))
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.get_bottom() - Gets the bottom coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_bottom")]
struct BoxGetBottom {
    id: Value,
}

impl BoxGetBottom {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { fields, .. } = &self.id {
            Ok(fields.borrow().get("bottom").cloned().unwrap_or(Value::Na))
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// box.delete() - Deletes a box
#[derive(BuiltinFunction)]
#[builtin(name = "box.delete")]
struct BoxDelete {
    _id: Value,
}

impl BoxDelete {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // For now, just return na - actual deletion would be handled by state management
        Ok(Value::Na)
    }
}

/// box.copy() - Copies a box
#[derive(BuiltinFunction)]
#[builtin(name = "box.copy")]
struct BoxCopy {
    id: Value,
}

impl BoxCopy {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if let Value::Object { type_name, fields } = &self.id {
            let copied_fields = fields.borrow().clone();
            Ok(Value::Object {
                type_name: type_name.clone(),
                fields: Rc::new(RefCell::new(copied_fields)),
            })
        } else {
            Err(RuntimeError::TypeError("Expected box object".to_string()))
        }
    }
}

/// Register the box namespace with all functions
pub fn register() -> Value {
    let mut members = HashMap::new();

    members.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(BoxNew::builtin_fn)),
    );
    members.insert(
        "set_left".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetLeft::builtin_fn)),
    );
    members.insert(
        "set_top".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTop::builtin_fn)),
    );
    members.insert(
        "set_right".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetRight::builtin_fn)),
    );
    members.insert(
        "set_bottom".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBottom::builtin_fn)),
    );
    members.insert(
        "set_lefttop".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetLefttop::builtin_fn)),
    );
    members.insert(
        "set_rightbottom".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetRightbottom::builtin_fn)),
    );
    members.insert(
        "set_border_color".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBorderColor::builtin_fn)),
    );
    members.insert(
        "set_border_width".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBorderWidth::builtin_fn)),
    );
    members.insert(
        "set_border_style".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBorderStyle::builtin_fn)),
    );
    members.insert(
        "set_extend".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetExtend::builtin_fn)),
    );
    members.insert(
        "set_bgcolor".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBgcolor::builtin_fn)),
    );
    members.insert(
        "set_text".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetText::builtin_fn)),
    );
    members.insert(
        "set_text_color".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextColor::builtin_fn)),
    );
    members.insert(
        "set_text_size".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextSize::builtin_fn)),
    );
    members.insert(
        "set_text_halign".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextHalign::builtin_fn)),
    );
    members.insert(
        "set_text_valign".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextValign::builtin_fn)),
    );
    members.insert(
        "set_text_wrap".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextWrap::builtin_fn)),
    );
    members.insert(
        "set_text_font_family".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextFontFamily::builtin_fn)),
    );
    members.insert(
        "set_xloc".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetXloc::builtin_fn)),
    );
    members.insert(
        "get_left".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetLeft::builtin_fn)),
    );
    members.insert(
        "get_top".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetTop::builtin_fn)),
    );
    members.insert(
        "get_right".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetRight::builtin_fn)),
    );
    members.insert(
        "get_bottom".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetBottom::builtin_fn)),
    );
    members.insert(
        "delete".to_string(),
        Value::BuiltinFunction(Rc::new(BoxDelete::builtin_fn)),
    );
    members.insert(
        "copy".to_string(),
        Value::BuiltinFunction(Rc::new(BoxCopy::builtin_fn)),
    );

    Value::Object {
        type_name: "box".to_string(),
        fields: Rc::new(RefCell::new(members)),
    }
}
