//! The `chart.*` namespace: chart-state variables and the `chart.point` type.
//!
//! The chart-state variables report the current chart style and visible range.
//! Headless, there is no viewport or non-standard chart type, so the styles are
//! fixed (`is_standard` true, the rest false) and the visible-range times are
//! `na`. `chart.point` builds the point objects `line`/`box`/`polyline` accept.

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// `na` for a missing coordinate, a plain number otherwise.
fn coord<O: PineOutput>(value: f64) -> Value<O> {
    if value.is_nan() {
        Value::Na
    } else {
        Value::Number(value)
    }
}

/// A `chart.point` object with `time`/`index`/`price` fields.
fn point<O: PineOutput>(time: Value<O>, index: Value<O>, price: Value<O>) -> Value<O> {
    let mut fields = HashMap::new();
    fields.insert("time".to_string(), time);
    fields.insert("index".to_string(), index);
    fields.insert("price".to_string(), price);
    Value::Object {
        type_name: "chart.point".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}

/// chart.point.new(time, index, price) - A point from an explicit time, index
/// and price.
#[derive(BuiltinFunction)]
#[builtin(name = "chart.point.new")]
struct ChartPointNew {
    #[arg(default = f64::NAN)]
    time: f64,
    #[arg(default = f64::NAN)]
    index: f64,
    #[arg(default = f64::NAN)]
    price: f64,
}

impl ChartPointNew {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(point(
            coord(self.time),
            coord(self.index),
            coord(self.price),
        ))
    }
}

/// chart.point.now(price) - A point at the current bar's index and time.
#[derive(BuiltinFunction)]
#[builtin(name = "chart.point.now")]
struct ChartPointNow {
    #[arg(default = f64::NAN)]
    price: f64,
}

impl ChartPointNow {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let index = ctx.get_variable("bar_index").cloned().unwrap_or(Value::Na);
        let time = ctx.get_variable("time").cloned().unwrap_or(Value::Na);
        Ok(point(time, index, coord(self.price)))
    }
}

/// chart.point.from_index(index, price) - A point from a bar index (time `na`).
#[derive(BuiltinFunction)]
#[builtin(name = "chart.point.from_index")]
struct ChartPointFromIndex {
    index: f64,
    price: f64,
}

impl ChartPointFromIndex {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(point(Value::Na, coord(self.index), coord(self.price)))
    }
}

/// chart.point.from_time(time, price) - A point from a time (index `na`).
#[derive(BuiltinFunction)]
#[builtin(name = "chart.point.from_time")]
struct ChartPointFromTime {
    time: f64,
    price: f64,
}

impl ChartPointFromTime {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(point(coord(self.time), Value::Na, coord(self.price)))
    }
}

/// chart.point.copy(id) - A copy of an existing point.
#[derive(BuiltinFunction)]
#[builtin(name = "chart.point.copy")]
struct ChartPointCopy<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> ChartPointCopy<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let Value::Object { fields, .. } = &self.id else {
            return Err(RuntimeError::TypeError(
                "chart.point.copy: expected a chart.point".into(),
            ));
        };
        let fields = fields.borrow();
        let read = |name: &str| fields.get(name).cloned().unwrap_or(Value::Na);
        Ok(point(read("time"), read("index"), read("price")))
    }
}

/// Register the `chart.*` namespace object.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut point_ns: HashMap<String, Value<O>> = HashMap::new();
    point_ns.insert("new".to_string(), ChartPointNew::builtin_value::<O>());
    point_ns.insert("now".to_string(), ChartPointNow::builtin_value::<O>());
    point_ns.insert(
        "from_index".to_string(),
        ChartPointFromIndex::builtin_value::<O>(),
    );
    point_ns.insert(
        "from_time".to_string(),
        ChartPointFromTime::builtin_value::<O>(),
    );
    point_ns.insert("copy".to_string(), ChartPointCopy::<O>::builtin_value());
    let point_obj = Value::Object {
        type_name: "chart.point".to_string(),
        fields: Rc::new(RefCell::new(point_ns)),
        call: None,
    };

    let mut fields: HashMap<String, Value<O>> = HashMap::new();
    fields.insert("bg_color".to_string(), Value::Na);
    fields.insert("fg_color".to_string(), Value::Na);
    fields.insert("is_standard".to_string(), Value::Bool(true));
    for style in [
        "is_heikinashi",
        "is_kagi",
        "is_linebreak",
        "is_pnf",
        "is_range",
        "is_renko",
    ] {
        fields.insert(style.to_string(), Value::Bool(false));
    }
    fields.insert("left_visible_bar_time".to_string(), Value::Na);
    fields.insert("right_visible_bar_time".to_string(), Value::Na);
    fields.insert("point".to_string(), point_obj);

    Value::Object {
        type_name: "chart".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}
