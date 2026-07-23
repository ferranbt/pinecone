use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Color, Interpreter, Label, LabelOutput, PineOutput, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// label.new() - Creates a new label object
#[derive(BuiltinFunction)]
#[builtin(name = "label.new")]
struct LabelNew<O: PineOutput + LabelOutput> {
    x: Value<O>,
    y: Value<O>,
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

impl<O: PineOutput + LabelOutput> LabelNew<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // Create a label struct
        let label = Label {
            x: self.x.as_number()?,
            y: self.y.as_number()?,
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
struct LabelSetX<O: PineOutput + LabelOutput> {
    id: f64,
    x: Value<O>,
}

impl<O: PineOutput + LabelOutput> LabelSetX<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.x = self.x.as_number()?;
        Ok(Value::Na)
    }
}

/// label.set_y() - Sets the y coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_y")]
struct LabelSetY<O: PineOutput + LabelOutput> {
    id: f64,
    y: Value<O>,
}

impl<O: PineOutput + LabelOutput> LabelSetY<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.y = self.y.as_number()?;
        Ok(Value::Na)
    }
}

/// label.set_xy() - Sets the x and y coordinates of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_xy")]
struct LabelSetXy<O: PineOutput + LabelOutput> {
    id: f64,
    x: Value<O>,
    y: Value<O>,
}

impl<O: PineOutput + LabelOutput> LabelSetXy<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.x = self.x.as_number()?;
        label.y = self.y.as_number()?;
        Ok(Value::Na)
    }
}

/// label.set_xloc() - Sets the x location mode and coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_xloc")]
struct LabelSetXloc<O: PineOutput + LabelOutput> {
    id: f64,
    x: Value<O>,
    xloc: String,
}

impl<O: PineOutput + LabelOutput> LabelSetXloc<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        label.x = self.x.as_number()?;
        label.xloc = self.xloc.clone();
        Ok(Value::Na)
    }
}

/// label.set_yloc() - Sets the y location mode
#[derive(BuiltinFunction)]
#[builtin(name = "label.set_yloc", output = LabelOutput)]
struct LabelSetYloc {
    id: f64,
    yloc: String,
}

impl LabelSetYloc {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_color", output = LabelOutput)]
struct LabelSetColor {
    id: f64,
    color: Color,
}

impl LabelSetColor {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_style", output = LabelOutput)]
struct LabelSetStyle {
    id: f64,
    style: String,
}

impl LabelSetStyle {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_text", output = LabelOutput)]
struct LabelSetText {
    id: f64,
    text: String,
}

impl LabelSetText {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_textcolor", output = LabelOutput)]
struct LabelSetTextcolor {
    id: f64,
    textcolor: Color,
}

impl LabelSetTextcolor {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_size", output = LabelOutput)]
struct LabelSetSize {
    id: f64,
    size: String,
}

impl LabelSetSize {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_textalign", output = LabelOutput)]
struct LabelSetTextalign {
    id: f64,
    textalign: String,
}

impl LabelSetTextalign {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_tooltip", output = LabelOutput)]
struct LabelSetTooltip {
    id: f64,
    tooltip: String,
}

impl LabelSetTooltip {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.set_text_font_family", output = LabelOutput)]
struct LabelSetTextFontFamily {
    id: f64,
    text_font_family: String,
}

impl LabelSetTextFontFamily {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
#[builtin(name = "label.get_x", output = LabelOutput)]
struct LabelGetX {
    id: f64,
}

impl LabelGetX {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        Ok(Value::Number(label.x))
    }
}

/// label.get_y() - Gets the y coordinate of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_y", output = LabelOutput)]
struct LabelGetY {
    id: f64,
}

impl LabelGetY {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        Ok(Value::Number(label.y))
    }
}

/// label.get_text() - Gets the text of a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.get_text", output = LabelOutput)]
struct LabelGetText {
    id: f64,
}

impl LabelGetText {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let label = ctx
            .output
            .get_label_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Label with id {} not found", id)))?;
        Ok(Value::String(label.text.clone()))
    }
}

/// label.delete() - Deletes a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.delete", output = LabelOutput)]
struct LabelDelete {
    id: f64,
}

impl LabelDelete {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        ctx.output.delete_label(id);
        Ok(Value::Na)
    }
}

/// label.copy() - Copies a label
#[derive(BuiltinFunction)]
#[builtin(name = "label.copy", output = LabelOutput)]
struct LabelCopy {
    id: f64,
}

impl LabelCopy {
    fn execute<O: PineOutput + LabelOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
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
pub fn register<O: PineOutput + LabelOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

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

    members.insert("new".to_string(), LabelNew::<O>::builtin_value());
    members.insert("set_x".to_string(), LabelSetX::<O>::builtin_value());
    members.insert("set_y".to_string(), LabelSetY::<O>::builtin_value());
    members.insert("set_xy".to_string(), LabelSetXy::<O>::builtin_value());
    members.insert("set_xloc".to_string(), LabelSetXloc::<O>::builtin_value());
    members.insert("set_yloc".to_string(), LabelSetYloc::builtin_value::<O>());
    members.insert("set_color".to_string(), LabelSetColor::builtin_value::<O>());
    members.insert("set_style".to_string(), LabelSetStyle::builtin_value::<O>());
    members.insert("set_text".to_string(), LabelSetText::builtin_value::<O>());
    members.insert(
        "set_textcolor".to_string(),
        LabelSetTextcolor::builtin_value::<O>(),
    );
    members.insert("set_size".to_string(), LabelSetSize::builtin_value::<O>());
    members.insert(
        "set_textalign".to_string(),
        LabelSetTextalign::builtin_value::<O>(),
    );
    members.insert(
        "set_tooltip".to_string(),
        LabelSetTooltip::builtin_value::<O>(),
    );
    members.insert(
        "set_text_font_family".to_string(),
        LabelSetTextFontFamily::builtin_value::<O>(),
    );
    members.insert("get_x".to_string(), LabelGetX::builtin_value::<O>());
    members.insert("get_y".to_string(), LabelGetY::builtin_value::<O>());
    members.insert("get_text".to_string(), LabelGetText::builtin_value::<O>());
    members.insert("delete".to_string(), LabelDelete::builtin_value::<O>());
    members.insert("copy".to_string(), LabelCopy::builtin_value::<O>());

    Value::Object {
        type_name: "label".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
