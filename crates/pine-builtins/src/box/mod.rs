use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{BoxOutput, Color, Interpreter, PineBox, PineOutput, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// box.new() - Creates a new box object
#[derive(BuiltinFunction)]
#[builtin(name = "box.new")]
struct BoxNew<O: PineOutput + BoxOutput> {
    left: Value<O>,
    top: Value<O>,
    right: Value<O>,
    bottom: Value<O>,
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

impl<O: PineOutput + BoxOutput> BoxNew<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // Create a box struct
        let box_obj = PineBox {
            left: self.left.as_number()?,
            top: self.top.as_number()?,
            right: self.right.as_number()?,
            bottom: self.bottom.as_number()?,
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
struct BoxSetLeft<O: PineOutput + BoxOutput> {
    id: f64,
    left: Value<O>,
}

impl<O: PineOutput + BoxOutput> BoxSetLeft<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.left = self.left.as_number()?;
        Ok(Value::Na)
    }
}

/// box.set_top() - Sets the top coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_top")]
struct BoxSetTop<O: PineOutput + BoxOutput> {
    id: f64,
    top: Value<O>,
}

impl<O: PineOutput + BoxOutput> BoxSetTop<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.top = self.top.as_number()?;
        Ok(Value::Na)
    }
}

/// box.set_right() - Sets the right coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_right")]
struct BoxSetRight<O: PineOutput + BoxOutput> {
    id: f64,
    right: Value<O>,
}

impl<O: PineOutput + BoxOutput> BoxSetRight<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.right = self.right.as_number()?;
        Ok(Value::Na)
    }
}

/// box.set_bottom() - Sets the bottom coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_bottom")]
struct BoxSetBottom<O: PineOutput + BoxOutput> {
    id: f64,
    bottom: Value<O>,
}

impl<O: PineOutput + BoxOutput> BoxSetBottom<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.bottom = self.bottom.as_number()?;
        Ok(Value::Na)
    }
}

/// box.set_lefttop() - Sets the left and top coordinates
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_lefttop")]
struct BoxSetLefttop<O: PineOutput + BoxOutput> {
    id: f64,
    left: Value<O>,
    top: Value<O>,
}

impl<O: PineOutput + BoxOutput> BoxSetLefttop<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.left = self.left.as_number()?;
        box_obj.top = self.top.as_number()?;
        Ok(Value::Na)
    }
}

/// box.set_rightbottom() - Sets the right and bottom coordinates
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_rightbottom")]
struct BoxSetRightbottom<O: PineOutput + BoxOutput> {
    id: f64,
    right: Value<O>,
    bottom: Value<O>,
}

impl<O: PineOutput + BoxOutput> BoxSetRightbottom<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.right = self.right.as_number()?;
        box_obj.bottom = self.bottom.as_number()?;
        Ok(Value::Na)
    }
}

/// box.set_border_color() - Sets the border color
#[derive(BuiltinFunction)]
#[builtin(name = "box.set_border_color", output = BoxOutput)]
struct BoxSetBorderColor {
    id: f64,
    color: Color,
}

impl BoxSetBorderColor {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_border_width", output = BoxOutput)]
struct BoxSetBorderWidth {
    id: f64,
    width: f64,
}

impl BoxSetBorderWidth {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_border_style", output = BoxOutput)]
struct BoxSetBorderStyle {
    id: f64,
    style: String,
}

impl BoxSetBorderStyle {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_extend", output = BoxOutput)]
struct BoxSetExtend {
    id: f64,
    extend: String,
}

impl BoxSetExtend {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_bgcolor", output = BoxOutput)]
struct BoxSetBgcolor {
    id: f64,
    color: Color,
}

impl BoxSetBgcolor {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_text", output = BoxOutput)]
struct BoxSetText {
    id: f64,
    text: String,
}

impl BoxSetText {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_text_color", output = BoxOutput)]
struct BoxSetTextColor {
    id: f64,
    color: Color,
}

impl BoxSetTextColor {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_text_size", output = BoxOutput)]
struct BoxSetTextSize {
    id: f64,
    size: f64,
}

impl BoxSetTextSize {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_text_halign", output = BoxOutput)]
struct BoxSetTextHalign {
    id: f64,
    halign: String,
}

impl BoxSetTextHalign {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_text_valign", output = BoxOutput)]
struct BoxSetTextValign {
    id: f64,
    valign: String,
}

impl BoxSetTextValign {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_text_wrap", output = BoxOutput)]
struct BoxSetTextWrap {
    id: f64,
    wrap: String,
}

impl BoxSetTextWrap {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
#[builtin(name = "box.set_text_font_family", output = BoxOutput)]
struct BoxSetTextFontFamily {
    id: f64,
    font_family: String,
}

impl BoxSetTextFontFamily {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
struct BoxSetXloc<O: PineOutput + BoxOutput> {
    id: f64,
    left: Value<O>,
    right: Value<O>,
    xloc: String,
}

impl<O: PineOutput + BoxOutput> BoxSetXloc<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        box_obj.left = self.left.as_number()?;
        box_obj.right = self.right.as_number()?;
        box_obj.xloc = self.xloc.clone();
        Ok(Value::Na)
    }
}

/// box.get_left() - Gets the left coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_left", output = BoxOutput)]
struct BoxGetLeft {
    id: f64,
}

impl BoxGetLeft {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(Value::Number(box_obj.left))
    }
}

/// box.get_top() - Gets the top coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_top", output = BoxOutput)]
struct BoxGetTop {
    id: f64,
}

impl BoxGetTop {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(Value::Number(box_obj.top))
    }
}

/// box.get_right() - Gets the right coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_right", output = BoxOutput)]
struct BoxGetRight {
    id: f64,
}

impl BoxGetRight {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(Value::Number(box_obj.right))
    }
}

/// box.get_bottom() - Gets the bottom coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "box.get_bottom", output = BoxOutput)]
struct BoxGetBottom {
    id: f64,
}

impl BoxGetBottom {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        let box_obj = ctx
            .output
            .get_box_mut(id)
            .ok_or_else(|| RuntimeError::TypeError(format!("Box with id {} not found", id)))?;
        Ok(Value::Number(box_obj.bottom))
    }
}

/// box.delete() - Deletes a box
#[derive(BuiltinFunction)]
#[builtin(name = "box.delete", output = BoxOutput)]
struct BoxDelete {
    id: f64,
}

impl BoxDelete {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = self.id as usize;
        ctx.output.delete_box(id);
        Ok(Value::Na)
    }
}

/// box.copy() - Copies a box
#[derive(BuiltinFunction)]
#[builtin(name = "box.copy", output = BoxOutput)]
struct BoxCopy {
    id: f64,
}

impl BoxCopy {
    fn execute<O: PineOutput + BoxOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
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
pub fn register<O: PineOutput + BoxOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(BoxNew::<O>::builtin_fn)),
    );
    members.insert(
        "set_left".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetLeft::<O>::builtin_fn)),
    );
    members.insert(
        "set_top".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTop::<O>::builtin_fn)),
    );
    members.insert(
        "set_right".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetRight::<O>::builtin_fn)),
    );
    members.insert(
        "set_bottom".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBottom::<O>::builtin_fn)),
    );
    members.insert(
        "set_lefttop".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetLefttop::<O>::builtin_fn)),
    );
    members.insert(
        "set_rightbottom".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetRightbottom::<O>::builtin_fn)),
    );
    members.insert(
        "set_border_color".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBorderColor::builtin_fn::<O>)),
    );
    members.insert(
        "set_border_width".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBorderWidth::builtin_fn::<O>)),
    );
    members.insert(
        "set_border_style".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBorderStyle::builtin_fn::<O>)),
    );
    members.insert(
        "set_extend".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetExtend::builtin_fn::<O>)),
    );
    members.insert(
        "set_bgcolor".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetBgcolor::builtin_fn::<O>)),
    );
    members.insert(
        "set_text".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetText::builtin_fn::<O>)),
    );
    members.insert(
        "set_text_color".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextColor::builtin_fn::<O>)),
    );
    members.insert(
        "set_text_size".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextSize::builtin_fn::<O>)),
    );
    members.insert(
        "set_text_halign".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextHalign::builtin_fn::<O>)),
    );
    members.insert(
        "set_text_valign".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextValign::builtin_fn::<O>)),
    );
    members.insert(
        "set_text_wrap".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextWrap::builtin_fn::<O>)),
    );
    members.insert(
        "set_text_font_family".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetTextFontFamily::builtin_fn::<O>)),
    );
    members.insert(
        "set_xloc".to_string(),
        Value::BuiltinFunction(Rc::new(BoxSetXloc::<O>::builtin_fn)),
    );
    members.insert(
        "get_left".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetLeft::builtin_fn::<O>)),
    );
    members.insert(
        "get_top".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetTop::builtin_fn::<O>)),
    );
    members.insert(
        "get_right".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetRight::builtin_fn::<O>)),
    );
    members.insert(
        "get_bottom".to_string(),
        Value::BuiltinFunction(Rc::new(BoxGetBottom::builtin_fn::<O>)),
    );
    members.insert(
        "delete".to_string(),
        Value::BuiltinFunction(Rc::new(BoxDelete::builtin_fn::<O>)),
    );
    members.insert(
        "copy".to_string(),
        Value::BuiltinFunction(Rc::new(BoxCopy::builtin_fn::<O>)),
    );

    Value::Object {
        type_name: "box".to_string(),
        fields: Rc::new(RefCell::new(members)),
    }
}
