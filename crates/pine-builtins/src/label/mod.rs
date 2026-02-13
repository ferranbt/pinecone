use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Color, Interpreter, Label, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// label.new() - Creates a new label object
#[derive(BuiltinFunction)]
#[builtin(name = "label.new")]
struct LabelNew {
    x: Value,
    y: Value,
    #[arg(default = "")]
    text: String,
    #[arg(default = "bar_index")]
    xloc: String,
    #[arg(default = "price")]
    yloc: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = "style_label_down")]
    style: String,
    #[arg(default = None)]
    textcolor: Option<Color>,
    #[arg(default = "normal")]
    size: String,
    #[arg(default = "center")]
    textalign: String,
    #[arg(default = None)]
    tooltip: Option<String>,
    #[arg(default = "default")]
    text_font_family: String,
}

impl LabelNew {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        // Create a label struct
        let label = Label {
            x: self.x.clone(),
            y: self.y.clone(),
            text: self.text.clone(),
            xloc: self.xloc.clone(),
            yloc: self.yloc.clone(),
            color: self.color.clone(),
            style: self.style.clone(),
            textcolor: self.textcolor.clone(),
            size: self.size.clone(),
            textalign: self.textalign.clone(),
            tooltip: self.tooltip.clone(),
            text_font_family: self.text_font_family.clone(),
        };

        // Add to interpreter and get ID
        let id = ctx.output.add_label(label);

        // Return the ID as a number
        Ok(Value::Number(id as f64))
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.x = self.x.clone();
        Ok(Value::Na)
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.y = self.y.clone();
        Ok(Value::Na)
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.x = self.x.clone();
        label.y = self.y.clone();
        Ok(Value::Na)
    }
}

/// label.set_xloc() - Sets the x location mode and coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_xloc")]
struct LabelSetXloc {
    id: Value,
    x: Value,
    xloc: String,
}

impl LabelSetXloc {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.x = self.x.clone();
        label.xloc = self.xloc.clone();
        Ok(Value::Na)
    }
}

/// label.set_yloc() - Sets the y location mode
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_yloc")]
struct LabelSetYloc {
    id: Value,
    yloc: String,
}

impl LabelSetYloc {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.yloc = self.yloc.clone();
        Ok(Value::Na)
    }
}

/// label.set_color() - Sets the label color
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_color")]
struct LabelSetColor {
    id: Value,
    color: Color,
}

impl LabelSetColor {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.color = Some(self.color.clone());
        Ok(Value::Na)
    }
}

/// label.set_style() - Sets the label style
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_style")]
struct LabelSetStyle {
    id: Value,
    style: String,
}

impl LabelSetStyle {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.style = self.style.clone();
        Ok(Value::Na)
    }
}

/// label.set_text() - Sets the label text
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_text")]
struct LabelSetText {
    id: Value,
    text: String,
}

impl LabelSetText {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.text = self.text.clone();
        Ok(Value::Na)
    }
}

/// label.set_textcolor() - Sets the label text color
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_textcolor")]
struct LabelSetTextcolor {
    id: Value,
    textcolor: Color,
}

impl LabelSetTextcolor {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.textcolor = Some(self.textcolor.clone());
        Ok(Value::Na)
    }
}

/// label.set_size() - Sets the label size
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_size")]
struct LabelSetSize {
    id: Value,
    size: String,
}

impl LabelSetSize {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.size = self.size.clone();
        Ok(Value::Na)
    }
}

/// label.set_textalign() - Sets the label text alignment
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_textalign")]
struct LabelSetTextalign {
    id: Value,
    textalign: String,
}

impl LabelSetTextalign {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.textalign = self.textalign.clone();
        Ok(Value::Na)
    }
}

/// label.set_tooltip() - Sets the label tooltip
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_tooltip")]
struct LabelSetTooltip {
    id: Value,
    tooltip: String,
}

impl LabelSetTooltip {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.tooltip = Some(self.tooltip.clone());
        Ok(Value::Na)
    }
}

/// label.set_text_font_family() - Sets the label text font family
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_text_font_family")]
struct LabelSetTextFontFamily {
    id: Value,
    text_font_family: String,
}

impl LabelSetTextFontFamily {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.text_font_family = self.text_font_family.clone();
        Ok(Value::Na)
    }
}

/// label.get_x() - Gets the x coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_x")]
struct LabelGetX {
    id: Value,
}

impl LabelGetX {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        Ok(label.x.clone())
    }
}

/// label.get_y() - Gets the y coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_y")]
struct LabelGetY {
    id: Value,
}

impl LabelGetY {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        Ok(label.y.clone())
    }
}

/// label.get_text() - Gets the text of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_text")]
struct LabelGetText {
    id: Value,
}

impl LabelGetText {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        Ok(Value::String(label.text.clone()))
    }
}

/// label.delete() - Deletes a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.delete")]
struct LabelDelete {
    id: Value,
}

impl LabelDelete {
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        ctx.output.delete_label(id);
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let id = self.id.as_number()? as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        let copied_label = label.clone();
        let new_id = ctx.output.add_label(copied_label);
        Ok(Value::Number(new_id as f64))
    }
}

/// Register the label namespace with all functions
pub fn register() -> Value {
    let mut members = HashMap::new();

    // Add style constants
    members.insert(
        "style_arrowdown".to_string(),
        Value::String("style_arrowdown".to_string()),
    );
    members.insert(
        "style_arrowup".to_string(),
        Value::String("style_arrowup".to_string()),
    );
    members.insert(
        "style_circle".to_string(),
        Value::String("style_circle".to_string()),
    );
    members.insert(
        "style_cross".to_string(),
        Value::String("style_cross".to_string()),
    );
    members.insert(
        "style_diamond".to_string(),
        Value::String("style_diamond".to_string()),
    );
    members.insert(
        "style_flag".to_string(),
        Value::String("style_flag".to_string()),
    );
    members.insert(
        "style_label_center".to_string(),
        Value::String("style_label_center".to_string()),
    );
    members.insert(
        "style_label_down".to_string(),
        Value::String("style_label_down".to_string()),
    );
    members.insert(
        "style_label_left".to_string(),
        Value::String("style_label_left".to_string()),
    );
    members.insert(
        "style_label_lower_left".to_string(),
        Value::String("style_label_lower_left".to_string()),
    );
    members.insert(
        "style_label_lower_right".to_string(),
        Value::String("style_label_lower_right".to_string()),
    );
    members.insert(
        "style_label_right".to_string(),
        Value::String("style_label_right".to_string()),
    );
    members.insert(
        "style_label_up".to_string(),
        Value::String("style_label_up".to_string()),
    );
    members.insert(
        "style_label_upper_left".to_string(),
        Value::String("style_label_upper_left".to_string()),
    );
    members.insert(
        "style_label_upper_right".to_string(),
        Value::String("style_label_upper_right".to_string()),
    );
    members.insert(
        "style_none".to_string(),
        Value::String("style_none".to_string()),
    );
    members.insert(
        "style_square".to_string(),
        Value::String("style_square".to_string()),
    );
    members.insert(
        "style_text_outline".to_string(),
        Value::String("style_text_outline".to_string()),
    );
    members.insert(
        "style_triangledown".to_string(),
        Value::String("style_triangledown".to_string()),
    );
    members.insert(
        "style_triangleup".to_string(),
        Value::String("style_triangleup".to_string()),
    );
    members.insert(
        "style_xcross".to_string(),
        Value::String("style_xcross".to_string()),
    );

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
