//! The `linefill.*` namespace: a colored fill between two lines. Id-based
//! create / read / mutate / delete over the shared [`DrawingOutput`] sink.

use pine_builtin_macro::BuiltinFunction;
use pine_core::{Color, DrawingOutput, LinefillObject, PineOutput};
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// linefill.new(line1, line2, color) - Fill the area between two lines.
#[derive(BuiltinFunction)]
#[builtin(name = "linefill.new", output = DrawingOutput)]
struct LinefillNew {
    line1: f64,
    line2: f64,
    color: Color,
}

impl LinefillNew {
    fn execute<O: PineOutput + DrawingOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let id = ctx.output.add_linefill(LinefillObject {
            line1: self.line1.max(0.0) as usize,
            line2: self.line2.max(0.0) as usize,
            color: Some(self.color.clone()),
        });
        Ok(Value::Number(id as f64))
    }
}

/// linefill.get_line1(id) - The first line's id.
#[derive(BuiltinFunction)]
#[builtin(name = "linefill.get_line1", output = DrawingOutput)]
struct LinefillGetLine1 {
    id: f64,
}

impl LinefillGetLine1 {
    fn execute<O: PineOutput + DrawingOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(linefill(ctx, self.id)?.line1 as f64))
    }
}

/// linefill.get_line2(id) - The second line's id.
#[derive(BuiltinFunction)]
#[builtin(name = "linefill.get_line2", output = DrawingOutput)]
struct LinefillGetLine2 {
    id: f64,
}

impl LinefillGetLine2 {
    fn execute<O: PineOutput + DrawingOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(linefill(ctx, self.id)?.line2 as f64))
    }
}

/// linefill.set_color(id, color) - Recolor the fill.
#[derive(BuiltinFunction)]
#[builtin(name = "linefill.set_color", output = DrawingOutput)]
struct LinefillSetColor {
    id: f64,
    color: Color,
}

impl LinefillSetColor {
    fn execute<O: PineOutput + DrawingOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let color = self.color.clone();
        linefill_mut(ctx, self.id)?.color = Some(color);
        Ok(Value::Na)
    }
}

/// linefill.delete(id) - Remove the fill.
#[derive(BuiltinFunction)]
#[builtin(name = "linefill.delete", output = DrawingOutput)]
struct LinefillDelete {
    id: f64,
}

impl LinefillDelete {
    fn execute<O: PineOutput + DrawingOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        ctx.output.delete_linefill(self.id as usize);
        Ok(Value::Na)
    }
}

/// A linefill by id, or a "not found" error.
fn linefill<O: PineOutput + DrawingOutput>(
    ctx: &Interpreter<O>,
    id: f64,
) -> Result<&LinefillObject, RuntimeError> {
    let id = id as usize;
    ctx.output
        .get_linefill(id)
        .ok_or_else(|| RuntimeError::TypeError(format!("Linefill with id {} not found", id)))
}

/// A mutable linefill by id, or a "not found" error.
fn linefill_mut<O: PineOutput + DrawingOutput>(
    ctx: &mut Interpreter<O>,
    id: f64,
) -> Result<&mut LinefillObject, RuntimeError> {
    let id = id as usize;
    ctx.output
        .get_linefill_mut(id)
        .ok_or_else(|| RuntimeError::TypeError(format!("Linefill with id {} not found", id)))
}

/// Register the `linefill.*` namespace object.
pub fn register<O: PineOutput + DrawingOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    members.insert("new".to_string(), LinefillNew::builtin_value::<O>());
    members.insert(
        "get_line1".to_string(),
        LinefillGetLine1::builtin_value::<O>(),
    );
    members.insert(
        "get_line2".to_string(),
        LinefillGetLine2::builtin_value::<O>(),
    );
    members.insert(
        "set_color".to_string(),
        LinefillSetColor::builtin_value::<O>(),
    );
    members.insert("delete".to_string(), LinefillDelete::builtin_value::<O>());
    members.insert(
        "all".to_string(),
        Value::Array(Rc::new(RefCell::new(Vec::new()))),
    );
    Value::Object {
        type_name: "linefill".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
