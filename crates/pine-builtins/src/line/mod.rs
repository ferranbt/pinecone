//! The `line.*` namespace: trend lines drawn between two `(x, y)` points.
//!
//! Mirrors the `box` namespace — id-based create / mutate / read / delete over
//! the [`LineOutput`] sink, plus the `line.style_*` constants.

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineVersion;
use pine_interpreter::{
    BuiltinFn, Color, Interpreter, LineObject, LineOutput, PineOutput, RuntimeError, Value,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// line.new(x1, y1, x2, y2, xloc, extend, color, style, width) - Creates a line
#[derive(BuiltinFunction)]
#[builtin(name = "line.new")]
struct LineNew<O: PineOutput + LineOutput> {
    x1: Value<O>,
    y1: Value<O>,
    x2: Value<O>,
    y2: Value<O>,
    #[arg(default = "bar_index")]
    xloc: String,
    #[arg(default = "none")]
    extend: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = "solid")]
    style: String,
    #[arg(default = 1.0)]
    width: f64,
}

impl<O: PineOutput + LineOutput> LineNew<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let line = LineObject {
            x1: self.x1.as_number()?,
            y1: self.y1.as_number()?,
            x2: self.x2.as_number()?,
            y2: self.y2.as_number()?,
            xloc: self.xloc.clone(),
            extend: self.extend.clone(),
            color: self.color.clone(),
            style: self.style.clone(),
            width: self.width,
        };
        let id = ctx.output.add_line(line);
        Ok(Value::Number(id as f64))
    }
}

/// line.set_x1() - Sets the first point's x coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_x1")]
struct LineSetX1<O: PineOutput + LineOutput> {
    id: f64,
    x: Value<O>,
}

impl<O: PineOutput + LineOutput> LineSetX1<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.x1 = self.x.as_number()?;
        Ok(Value::Na)
    }
}

/// line.set_y1() - Sets the first point's y coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_y1")]
struct LineSetY1<O: PineOutput + LineOutput> {
    id: f64,
    y: Value<O>,
}

impl<O: PineOutput + LineOutput> LineSetY1<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.y1 = self.y.as_number()?;
        Ok(Value::Na)
    }
}

/// line.set_x2() - Sets the second point's x coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_x2")]
struct LineSetX2<O: PineOutput + LineOutput> {
    id: f64,
    x: Value<O>,
}

impl<O: PineOutput + LineOutput> LineSetX2<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.x2 = self.x.as_number()?;
        Ok(Value::Na)
    }
}

/// line.set_y2() - Sets the second point's y coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_y2")]
struct LineSetY2<O: PineOutput + LineOutput> {
    id: f64,
    y: Value<O>,
}

impl<O: PineOutput + LineOutput> LineSetY2<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.y2 = self.y.as_number()?;
        Ok(Value::Na)
    }
}

/// line.set_xy1() - Sets the first point
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_xy1")]
struct LineSetXy1<O: PineOutput + LineOutput> {
    id: f64,
    x: Value<O>,
    y: Value<O>,
}

impl<O: PineOutput + LineOutput> LineSetXy1<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let (x, y) = (self.x.as_number()?, self.y.as_number()?);
        let line = get_line_mut(ctx, self.id)?;
        line.x1 = x;
        line.y1 = y;
        Ok(Value::Na)
    }
}

/// line.set_xy2() - Sets the second point
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_xy2")]
struct LineSetXy2<O: PineOutput + LineOutput> {
    id: f64,
    x: Value<O>,
    y: Value<O>,
}

impl<O: PineOutput + LineOutput> LineSetXy2<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let (x, y) = (self.x.as_number()?, self.y.as_number()?);
        let line = get_line_mut(ctx, self.id)?;
        line.x2 = x;
        line.y2 = y;
        Ok(Value::Na)
    }
}

/// line.set_color() - Sets the line color
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_color", output = LineOutput)]
struct LineSetColor {
    id: f64,
    color: Color,
}

impl LineSetColor {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.color = Some(self.color.clone());
        Ok(Value::Na)
    }
}

/// line.set_width() - Sets the line width
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_width", output = LineOutput)]
struct LineSetWidth {
    id: f64,
    width: f64,
}

impl LineSetWidth {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.width = self.width;
        Ok(Value::Na)
    }
}

/// line.set_style() - Sets the line style
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_style", output = LineOutput)]
struct LineSetStyle {
    id: f64,
    style: String,
}

impl LineSetStyle {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.style = self.style.clone();
        Ok(Value::Na)
    }
}

/// line.set_extend() - Sets how the line extends past its endpoints
#[derive(BuiltinFunction)]
#[builtin(name = "line.set_extend", output = LineOutput)]
struct LineSetExtend {
    id: f64,
    extend: String,
}

impl LineSetExtend {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        line.extend = self.extend.clone();
        Ok(Value::Na)
    }
}

/// line.get_x1() - Gets the first point's x coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.get_x1", output = LineOutput)]
struct LineGetX1 {
    id: f64,
}

impl LineGetX1 {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(get_line_mut(ctx, self.id)?.x1))
    }
}

/// line.get_y1() - Gets the first point's y coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.get_y1", output = LineOutput)]
struct LineGetY1 {
    id: f64,
}

impl LineGetY1 {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(get_line_mut(ctx, self.id)?.y1))
    }
}

/// line.get_x2() - Gets the second point's x coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.get_x2", output = LineOutput)]
struct LineGetX2 {
    id: f64,
}

impl LineGetX2 {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(get_line_mut(ctx, self.id)?.x2))
    }
}

/// line.get_y2() - Gets the second point's y coordinate
#[derive(BuiltinFunction)]
#[builtin(name = "line.get_y2", output = LineOutput)]
struct LineGetY2 {
    id: f64,
}

impl LineGetY2 {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(get_line_mut(ctx, self.id)?.y2))
    }
}

/// line.get_price(id, x) - The line's y value at bar `x` (linear extrapolation)
#[derive(BuiltinFunction)]
#[builtin(name = "line.get_price", output = LineOutput)]
struct LineGetPrice {
    id: f64,
    x: f64,
}

impl LineGetPrice {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let line = get_line_mut(ctx, self.id)?;
        // A vertical line has no single price; Pine returns the endpoint value.
        let price = if line.x2 == line.x1 {
            line.y1
        } else {
            line.y1 + (line.y2 - line.y1) * (self.x - line.x1) / (line.x2 - line.x1)
        };
        Ok(Value::Number(price))
    }
}

/// line.delete() - Deletes a line
#[derive(BuiltinFunction)]
#[builtin(name = "line.delete", output = LineOutput)]
struct LineDelete {
    id: f64,
}

impl LineDelete {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        ctx.output.delete_line(self.id as usize);
        Ok(Value::Na)
    }
}

/// line.copy() - Copies a line and returns the new id
#[derive(BuiltinFunction)]
#[builtin(name = "line.copy", output = LineOutput)]
struct LineCopy {
    id: f64,
}

impl LineCopy {
    fn execute<O: PineOutput + LineOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let copied = get_line_mut(ctx, self.id)?.clone();
        let new_id = ctx.output.add_line(copied);
        Ok(Value::Number(new_id as f64))
    }
}

/// hline(price, title, color, linestyle, linewidth) - A horizontal line at a
/// fixed price.
///
/// `hline` is a global (not a `line.` member), but a horizontal line is just a
/// line with `y1 == y2`, so it reuses the same [`LineOutput`] sink — the id it
/// returns works with the `line.get_*` getters.
#[derive(BuiltinFunction)]
#[builtin(name = "hline")]
struct Hline<O: PineOutput + LineOutput> {
    price: Value<O>,
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = "solid")]
    linestyle: String,
    #[arg(default = 1.0)]
    linewidth: f64,
}

impl<O: PineOutput + LineOutput> Hline<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.title;
        let price = self.price.as_number()?;
        let line = LineObject {
            x1: 0.0,
            y1: price,
            x2: 0.0,
            y2: price,
            xloc: "bar_index".to_string(),
            extend: "both".to_string(),
            color: self.color.clone(),
            style: self.linestyle.clone(),
            width: self.linewidth,
        };
        let id = ctx.output.add_line(line);
        Ok(Value::Number(id as f64))
    }
}

/// The `hline` global: a callable object (`hline(...)`) that also carries the
/// `hline.style_*` constants.
fn hline_object<O: PineOutput + LineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    for style in ["style_solid", "style_dotted", "style_dashed"] {
        let tag = style.strip_prefix("style_").unwrap_or(style);
        members.insert(style.to_string(), Value::String(tag.to_string()));
    }
    Value::Object {
        type_name: "hline".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: Some(Rc::new(Hline::<O>::builtin_fn) as BuiltinFn<O>),
    }
}

/// Shared lookup: a mutable line by id, or a "not found" error.
fn get_line_mut<O: PineOutput + LineOutput>(
    ctx: &mut Interpreter<O>,
    id: f64,
) -> Result<&mut LineObject, RuntimeError> {
    let id = id as usize;
    ctx.output
        .get_line_mut(id)
        .ok_or_else(|| RuntimeError::TypeError(format!("Line with id {} not found", id)))
}

/// Every name this module contributes: the `line` namespace, the `hline` global,
/// and (v3/v4 only) the bare `linestyle=dotted` constants.
pub fn register<O: PineOutput + LineOutput>(version: PineVersion) -> Vec<(String, Value<O>)> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(LineNew::<O>::builtin_fn)),
    );
    members.insert(
        "set_x1".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetX1::<O>::builtin_fn)),
    );
    members.insert(
        "set_y1".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetY1::<O>::builtin_fn)),
    );
    members.insert(
        "set_x2".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetX2::<O>::builtin_fn)),
    );
    members.insert(
        "set_y2".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetY2::<O>::builtin_fn)),
    );
    members.insert(
        "set_xy1".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetXy1::<O>::builtin_fn)),
    );
    members.insert(
        "set_xy2".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetXy2::<O>::builtin_fn)),
    );
    members.insert(
        "set_color".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetColor::builtin_fn::<O>)),
    );
    members.insert(
        "set_width".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetWidth::builtin_fn::<O>)),
    );
    members.insert(
        "set_style".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetStyle::builtin_fn::<O>)),
    );
    members.insert(
        "set_extend".to_string(),
        Value::BuiltinFunction(Rc::new(LineSetExtend::builtin_fn::<O>)),
    );
    members.insert(
        "get_x1".to_string(),
        Value::BuiltinFunction(Rc::new(LineGetX1::builtin_fn::<O>)),
    );
    members.insert(
        "get_y1".to_string(),
        Value::BuiltinFunction(Rc::new(LineGetY1::builtin_fn::<O>)),
    );
    members.insert(
        "get_x2".to_string(),
        Value::BuiltinFunction(Rc::new(LineGetX2::builtin_fn::<O>)),
    );
    members.insert(
        "get_y2".to_string(),
        Value::BuiltinFunction(Rc::new(LineGetY2::builtin_fn::<O>)),
    );
    members.insert(
        "get_price".to_string(),
        Value::BuiltinFunction(Rc::new(LineGetPrice::builtin_fn::<O>)),
    );
    members.insert(
        "delete".to_string(),
        Value::BuiltinFunction(Rc::new(LineDelete::builtin_fn::<O>)),
    );
    members.insert(
        "copy".to_string(),
        Value::BuiltinFunction(Rc::new(LineCopy::builtin_fn::<O>)),
    );

    // Style constants.
    for style in [
        "style_solid",
        "style_dotted",
        "style_dashed",
        "style_arrow_left",
        "style_arrow_right",
        "style_arrow_both",
    ] {
        let tag = style.strip_prefix("style_").unwrap_or(style);
        members.insert(style.to_string(), Value::String(tag.to_string()));
    }

    let line_object = Value::Object {
        type_name: "line".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    };

    let mut entries = vec![
        ("line".to_string(), line_object),
        ("hline".to_string(), hline_object()),
    ];
    // v3/v4 pass the linestyle as a bare constant, e.g. `hline(0, linestyle=dotted)`;
    // v5/v6 use `line.style_dotted` / `hline.style_dotted` instead.
    if version < PineVersion::V5 {
        for tag in ["solid", "dotted", "dashed"] {
            entries.push((tag.to_string(), Value::String(tag.to_string())));
        }
    }
    entries
}
