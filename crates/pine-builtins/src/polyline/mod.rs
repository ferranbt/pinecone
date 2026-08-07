//! The `polyline.*` namespace: a multi-point line built from an array of
//! `chart.point`s. Id-based over the shared [`DrawingOutput`] sink.

use pine_builtin_macro::BuiltinFunction;
use pine_core::{Color, DrawingOutput, PineOutput, PolylineObject};
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// polyline.new(points, curved, closed, xloc, line_color, fill_color,
/// line_style, line_width) - A polyline through an array of `chart.point`s.
#[derive(BuiltinFunction)]
#[builtin(name = "polyline.new")]
struct PolylineNew<O: PineOutput + DrawingOutput> {
    points: Value<O>,
    #[arg(default = false)]
    curved: bool,
    #[arg(default = false)]
    closed: bool,
    #[arg(default = "bar_index")]
    xloc: String,
    #[arg(default = None)]
    line_color: Option<Color>,
    #[arg(default = None)]
    fill_color: Option<Color>,
    #[arg(default = "solid")]
    line_style: String,
    #[arg(default = 1.0)]
    line_width: f64,
}

impl<O: PineOutput + DrawingOutput> PolylineNew<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let Value::Array(points) = &self.points else {
            return Err(RuntimeError::TypeError(
                "polyline.new: expected an array of points".into(),
            ));
        };
        let points: Vec<(f64, f64)> = points
            .borrow()
            .iter()
            .map(crate::chart::point_coords)
            .collect::<Result<_, _>>()?;
        let id = ctx.output.add_polyline(PolylineObject {
            points,
            curved: self.curved,
            closed: self.closed,
            xloc: self.xloc.clone(),
            line_color: self.line_color.clone(),
            fill_color: self.fill_color.clone(),
            line_style: self.line_style.clone(),
            line_width: self.line_width,
        });
        Ok(Value::Number(id as f64))
    }
}

/// polyline.delete(id) - Remove the polyline.
#[derive(BuiltinFunction)]
#[builtin(name = "polyline.delete", output = DrawingOutput)]
struct PolylineDelete {
    id: f64,
}

impl PolylineDelete {
    fn execute<O: PineOutput + DrawingOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        ctx.output.delete_polyline(self.id as usize);
        Ok(Value::Na)
    }
}

/// Register the `polyline.*` namespace object.
pub fn register<O: PineOutput + DrawingOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    members.insert("new".to_string(), PolylineNew::<O>::builtin_value());
    members.insert("delete".to_string(), PolylineDelete::builtin_value::<O>());
    members.insert(
        "all".to_string(),
        Value::Array(Rc::new(RefCell::new(Vec::new()))),
    );
    Value::Object {
        type_name: "polyline".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
        value: None,
    }
}
