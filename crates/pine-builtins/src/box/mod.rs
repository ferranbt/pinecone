use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Color, Interpreter, PineBox, RuntimeError, Value};
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // Create a box struct
        let box_obj = PineBox {
            left: self.left.clone(),
            top: self.top.clone(),
            right: self.right.clone(),
            bottom: self.bottom.clone(),
            border_color: self.border_color.clone(),
            border_width: self.border_width,
            border_style: self.border_style.clone(),
            extend: self.extend.clone(),
            xloc: self.xloc.clone(),
            bgcolor: self.bgcolor.clone(),
            text: self.text.clone(),
            text_size: self.text_size,
            text_color: self.text_color.clone(),
            text_halign: self.text_halign.clone(),
            text_valign: self.text_valign.clone(),
            text_wrap: self.text_wrap.clone(),
            text_font_family: self.text_font_family.clone(),
        };

        // Add to interpreter and get ID
        let id = ctx.output.add_box(box_obj);

        // Return the ID as a number
        Ok(Value::Number(id as f64))
    }
}

/// box.set_left() - Sets the left coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_left")]
struct BoxSetLeft {
    id: f64,
    left: Value,
}

impl BoxSetLeft {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.left = self.left.clone();
        Ok(Value::Na)
    }
}

/// box.set_top() - Sets the top coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_top")]
struct BoxSetTop {
    id: f64,
    top: Value,
}

impl BoxSetTop {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.top = self.top.clone();
        Ok(Value::Na)
    }
}

/// box.set_right() - Sets the right coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_right")]
struct BoxSetRight {
    id: f64,
    right: Value,
}

impl BoxSetRight {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.right = self.right.clone();
        Ok(Value::Na)
    }
}

/// box.set_bottom() - Sets the bottom coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_bottom")]
struct BoxSetBottom {
    id: f64,
    bottom: Value,
}

impl BoxSetBottom {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.bottom = self.bottom.clone();
        Ok(Value::Na)
    }
}

/// box.set_lefttop() - Sets the left and top coordinates
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_lefttop")]
struct BoxSetLefttop {
    id: f64,
    left: Value,
    top: Value,
}

impl BoxSetLefttop {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.left = self.left.clone();
        box_obj.top = self.top.clone();
        Ok(Value::Na)
    }
}

/// box.set_rightbottom() - Sets the right and bottom coordinates
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_rightbottom")]
struct BoxSetRightbottom {
    id: f64,
    right: Value,
    bottom: Value,
}

impl BoxSetRightbottom {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.right = self.right.clone();
        box_obj.bottom = self.bottom.clone();
        Ok(Value::Na)
    }
}

/// box.set_border_color() - Sets the border color
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_border_color")]
struct BoxSetBorderColor {
    id: f64,
    color: Color,
}

impl BoxSetBorderColor {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.border_color = Some(self.color.clone());
        Ok(Value::Na)
    }
}

/// box.set_border_width() - Sets the border width
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_border_width")]
struct BoxSetBorderWidth {
    id: f64,
    width: f64,
}

impl BoxSetBorderWidth {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.border_width = self.width;
        Ok(Value::Na)
    }
}

/// box.set_border_style() - Sets the border style
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_border_style")]
struct BoxSetBorderStyle {
    id: f64,
    style: String,
}

impl BoxSetBorderStyle {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.border_style = self.style.clone();
        Ok(Value::Na)
    }
}

/// box.set_extend() - Sets the extend property
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_extend")]
struct BoxSetExtend {
    id: f64,
    extend: String,
}

impl BoxSetExtend {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.extend = self.extend.clone();
        Ok(Value::Na)
    }
}

/// box.set_bgcolor() - Sets the background color
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_bgcolor")]
struct BoxSetBgcolor {
    id: f64,
    color: Color,
}

impl BoxSetBgcolor {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.bgcolor = Some(self.color.clone());
        Ok(Value::Na)
    }
}

/// box.set_text() - Sets the text
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text")]
struct BoxSetText {
    id: f64,
    text: String,
}

impl BoxSetText {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.text = self.text.clone();
        Ok(Value::Na)
    }
}

/// box.set_text_color() - Sets the text color
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_color")]
struct BoxSetTextColor {
    id: f64,
    color: Color,
}

impl BoxSetTextColor {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.text_color = Some(self.color.clone());
        Ok(Value::Na)
    }
}

/// box.set_text_size() - Sets the text size
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_size")]
struct BoxSetTextSize {
    id: f64,
    size: f64,
}

impl BoxSetTextSize {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.text_size = self.size;
        Ok(Value::Na)
    }
}

/// box.set_text_halign() - Sets the text horizontal alignment
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_halign")]
struct BoxSetTextHalign {
    id: f64,
    halign: String,
}

impl BoxSetTextHalign {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.text_halign = self.halign.clone();
        Ok(Value::Na)
    }
}

/// box.set_text_valign() - Sets the text vertical alignment
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_valign")]
struct BoxSetTextValign {
    id: f64,
    valign: String,
}

impl BoxSetTextValign {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.text_valign = self.valign.clone();
        Ok(Value::Na)
    }
}

/// box.set_text_wrap() - Sets the text wrap mode
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_wrap")]
struct BoxSetTextWrap {
    id: f64,
    wrap: String,
}

impl BoxSetTextWrap {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.text_wrap = self.wrap.clone();
        Ok(Value::Na)
    }
}

/// box.set_text_font_family() - Sets the text font family
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_text_font_family")]
struct BoxSetTextFontFamily {
    id: f64,
    font_family: String,
}

impl BoxSetTextFontFamily {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.text_font_family = self.font_family.clone();
        Ok(Value::Na)
    }
}

/// box.set_xloc() - Sets the xloc mode
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_xloc")]
struct BoxSetXloc {
    id: f64,
    left: Value,
    right: Value,
    xloc: String,
}

impl BoxSetXloc {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.left = self.left.clone();
        box_obj.right = self.right.clone();
        box_obj.xloc = self.xloc.clone();
        Ok(Value::Na)
    }
}

/// box.get_left() - Gets the left coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_left")]
struct BoxGetLeft {
    id: f64,
}

impl BoxGetLeft {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(box_obj.left.clone())
    }
}

/// box.get_top() - Gets the top coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_top")]
struct BoxGetTop {
    id: f64,
}

impl BoxGetTop {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(box_obj.top.clone())
    }
}

/// box.get_right() - Gets the right coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_right")]
struct BoxGetRight {
    id: f64,
}

impl BoxGetRight {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(box_obj.right.clone())
    }
}

/// box.get_bottom() - Gets the bottom coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_bottom")]
struct BoxGetBottom {
    id: f64,
}

impl BoxGetBottom {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(box_obj.bottom.clone())
    }
}

/// box.delete() - Deletes a box
#[derive(BuiltinFunction)]
#[builtin(name = "box.delete")]
struct BoxDelete {
    id: f64,
}

impl BoxDelete {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        ctx.output.delete_box(id);
        Ok(Value::Na)
    }
}

/// box.copy() - Copies a box
#[derive(BuiltinFunction)]
#[builtin(name = "box.copy")]
struct BoxCopy {
    id: f64,
}

impl BoxCopy {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        let copied_box = box_obj.clone();
        let new_id = ctx.output.add_box(copied_box);
        Ok(Value::Number(new_id as f64))
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
