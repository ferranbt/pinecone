use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// plot() - Plots a series of data on the chart
#[derive(BuiltinFunction)]
#[builtin(name = "plot")]
struct Plot {
    series: Value,
    #[arg(default = Value::String(String::new()))]
    title: Value,
    #[arg(default = Value::Na)]
    color: Value,
    #[arg(default = Value::Number(1.0))]
    linewidth: Value,
    #[arg(default = Value::String("line".to_string()))]
    style: Value,
    #[arg(default = Value::Bool(false))]
    trackprice: Value,
    #[arg(default = Value::Number(0.0))]
    histbase: Value,
    #[arg(default = Value::Number(0.0))]
    offset: Value,
    #[arg(default = Value::Bool(false))]
    join: Value,
    #[arg(default = Value::Bool(true))]
    editable: Value,
    #[arg(default = Value::Na)]
    show_last: Value,
    #[arg(default = Value::String("all".to_string()))]
    display: Value,
    #[arg(default = Value::Na)]
    format: Value,
    #[arg(default = Value::Na)]
    precision: Value,
    #[arg(default = Value::Bool(false))]
    force_overlay: Value,
    #[arg(default = Value::String("solid".to_string()))]
    linestyle: Value,
}

impl Plot {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), self.title.clone());
        fields.insert("color".to_string(), self.color.clone());
        fields.insert("linewidth".to_string(), self.linewidth.clone());
        fields.insert("style".to_string(), self.style.clone());
        fields.insert("trackprice".to_string(), self.trackprice.clone());
        fields.insert("histbase".to_string(), self.histbase.clone());
        fields.insert("offset".to_string(), self.offset.clone());
        fields.insert("join".to_string(), self.join.clone());
        fields.insert("editable".to_string(), self.editable.clone());
        fields.insert("show_last".to_string(), self.show_last.clone());
        fields.insert("display".to_string(), self.display.clone());
        fields.insert("format".to_string(), self.format.clone());
        fields.insert("precision".to_string(), self.precision.clone());
        fields.insert("force_overlay".to_string(), self.force_overlay.clone());
        fields.insert("linestyle".to_string(), self.linestyle.clone());

        Ok(Value::Object {
            type_name: "plot".to_string(),
            fields: Rc::new(RefCell::new(fields)),
        })
    }
}

/// plotarrow() - Plots up and down arrows on the chart
#[derive(BuiltinFunction)]
#[builtin(name = "plotarrow")]
struct Plotarrow {
    series: Value,
    #[arg(default = Value::String(String::new()))]
    title: Value,
    #[arg(default = Value::Na)]
    colorup: Value,
    #[arg(default = Value::Na)]
    colordown: Value,
    #[arg(default = Value::Number(0.0))]
    offset: Value,
    #[arg(default = Value::Number(5.0))]
    minheight: Value,
    #[arg(default = Value::Number(100.0))]
    maxheight: Value,
    #[arg(default = Value::Bool(true))]
    editable: Value,
    #[arg(default = Value::Na)]
    show_last: Value,
    #[arg(default = Value::String("all".to_string()))]
    display: Value,
    #[arg(default = Value::Na)]
    format: Value,
    #[arg(default = Value::Na)]
    precision: Value,
    #[arg(default = Value::Bool(false))]
    force_overlay: Value,
}

impl Plotarrow {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), self.title.clone());
        fields.insert("colorup".to_string(), self.colorup.clone());
        fields.insert("colordown".to_string(), self.colordown.clone());
        fields.insert("offset".to_string(), self.offset.clone());
        fields.insert("minheight".to_string(), self.minheight.clone());
        fields.insert("maxheight".to_string(), self.maxheight.clone());
        fields.insert("editable".to_string(), self.editable.clone());
        fields.insert("show_last".to_string(), self.show_last.clone());
        fields.insert("display".to_string(), self.display.clone());
        fields.insert("format".to_string(), self.format.clone());
        fields.insert("precision".to_string(), self.precision.clone());
        fields.insert("force_overlay".to_string(), self.force_overlay.clone());

        Ok(Value::Na)
    }
}

/// plotbar() - Plots OHLC bars on the chart
#[derive(BuiltinFunction)]
#[builtin(name = "plotbar")]
struct Plotbar {
    open: Value,
    high: Value,
    low: Value,
    close: Value,
    #[arg(default = Value::String(String::new()))]
    title: Value,
    #[arg(default = Value::Na)]
    color: Value,
    #[arg(default = Value::Bool(true))]
    editable: Value,
    #[arg(default = Value::Na)]
    show_last: Value,
    #[arg(default = Value::String("all".to_string()))]
    display: Value,
    #[arg(default = Value::Na)]
    format: Value,
    #[arg(default = Value::Na)]
    precision: Value,
    #[arg(default = Value::Bool(false))]
    force_overlay: Value,
}

impl Plotbar {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("open".to_string(), self.open.clone());
        fields.insert("high".to_string(), self.high.clone());
        fields.insert("low".to_string(), self.low.clone());
        fields.insert("close".to_string(), self.close.clone());
        fields.insert("title".to_string(), self.title.clone());
        fields.insert("color".to_string(), self.color.clone());
        fields.insert("editable".to_string(), self.editable.clone());
        fields.insert("show_last".to_string(), self.show_last.clone());
        fields.insert("display".to_string(), self.display.clone());
        fields.insert("format".to_string(), self.format.clone());
        fields.insert("precision".to_string(), self.precision.clone());
        fields.insert("force_overlay".to_string(), self.force_overlay.clone());

        Ok(Value::Na)
    }
}

/// plotcandle() - Plots candlestick chart
#[derive(BuiltinFunction)]
#[builtin(name = "plotcandle")]
struct Plotcandle {
    open: Value,
    high: Value,
    low: Value,
    close: Value,
    #[arg(default = Value::String(String::new()))]
    title: Value,
    #[arg(default = Value::Na)]
    color: Value,
    #[arg(default = Value::Na)]
    wickcolor: Value,
    #[arg(default = Value::Bool(true))]
    editable: Value,
    #[arg(default = Value::Na)]
    show_last: Value,
    #[arg(default = Value::Na)]
    bordercolor: Value,
    #[arg(default = Value::String("all".to_string()))]
    display: Value,
    #[arg(default = Value::Na)]
    format: Value,
    #[arg(default = Value::Na)]
    precision: Value,
    #[arg(default = Value::Bool(false))]
    force_overlay: Value,
}

impl Plotcandle {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("open".to_string(), self.open.clone());
        fields.insert("high".to_string(), self.high.clone());
        fields.insert("low".to_string(), self.low.clone());
        fields.insert("close".to_string(), self.close.clone());
        fields.insert("title".to_string(), self.title.clone());
        fields.insert("color".to_string(), self.color.clone());
        fields.insert("wickcolor".to_string(), self.wickcolor.clone());
        fields.insert("editable".to_string(), self.editable.clone());
        fields.insert("show_last".to_string(), self.show_last.clone());
        fields.insert("bordercolor".to_string(), self.bordercolor.clone());
        fields.insert("display".to_string(), self.display.clone());
        fields.insert("format".to_string(), self.format.clone());
        fields.insert("precision".to_string(), self.precision.clone());
        fields.insert("force_overlay".to_string(), self.force_overlay.clone());

        Ok(Value::Na)
    }
}

/// plotchar() - Plots visual shapes on the chart using ASCII characters
#[derive(BuiltinFunction)]
#[builtin(name = "plotchar")]
struct Plotchar {
    series: Value,
    #[arg(default = Value::String(String::new()))]
    title: Value,
    #[arg(default = Value::String("★".to_string()))]
    char: Value,
    #[arg(default = Value::String("bottom".to_string()))]
    location: Value,
    #[arg(default = Value::Na)]
    color: Value,
    #[arg(default = Value::Number(0.0))]
    offset: Value,
    #[arg(default = Value::String(String::new()))]
    text: Value,
    #[arg(default = Value::Na)]
    textcolor: Value,
    #[arg(default = Value::Bool(true))]
    editable: Value,
    #[arg(default = Value::String("auto".to_string()))]
    size: Value,
    #[arg(default = Value::Na)]
    show_last: Value,
    #[arg(default = Value::String("all".to_string()))]
    display: Value,
    #[arg(default = Value::Na)]
    format: Value,
    #[arg(default = Value::Na)]
    precision: Value,
    #[arg(default = Value::Bool(false))]
    force_overlay: Value,
}

impl Plotchar {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), self.title.clone());
        fields.insert("char".to_string(), self.char.clone());
        fields.insert("location".to_string(), self.location.clone());
        fields.insert("color".to_string(), self.color.clone());
        fields.insert("offset".to_string(), self.offset.clone());
        fields.insert("text".to_string(), self.text.clone());
        fields.insert("textcolor".to_string(), self.textcolor.clone());
        fields.insert("editable".to_string(), self.editable.clone());
        fields.insert("size".to_string(), self.size.clone());
        fields.insert("show_last".to_string(), self.show_last.clone());
        fields.insert("display".to_string(), self.display.clone());
        fields.insert("format".to_string(), self.format.clone());
        fields.insert("precision".to_string(), self.precision.clone());
        fields.insert("force_overlay".to_string(), self.force_overlay.clone());

        Ok(Value::Na)
    }
}

/// plotshape() - Plots visual shapes on the chart
#[derive(BuiltinFunction)]
#[builtin(name = "plotshape")]
struct Plotshape {
    series: Value,
    #[arg(default = Value::String(String::new()))]
    title: Value,
    #[arg(default = Value::String("circle".to_string()))]
    style: Value,
    #[arg(default = Value::String("bottom".to_string()))]
    location: Value,
    #[arg(default = Value::Na)]
    color: Value,
    #[arg(default = Value::Number(0.0))]
    offset: Value,
    #[arg(default = Value::String(String::new()))]
    text: Value,
    #[arg(default = Value::Na)]
    textcolor: Value,
    #[arg(default = Value::Bool(true))]
    editable: Value,
    #[arg(default = Value::String("auto".to_string()))]
    size: Value,
    #[arg(default = Value::Na)]
    show_last: Value,
    #[arg(default = Value::String("all".to_string()))]
    display: Value,
    #[arg(default = Value::Na)]
    format: Value,
    #[arg(default = Value::Na)]
    precision: Value,
    #[arg(default = Value::Bool(false))]
    force_overlay: Value,
}

impl Plotshape {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), self.title.clone());
        fields.insert("style".to_string(), self.style.clone());
        fields.insert("location".to_string(), self.location.clone());
        fields.insert("color".to_string(), self.color.clone());
        fields.insert("offset".to_string(), self.offset.clone());
        fields.insert("text".to_string(), self.text.clone());
        fields.insert("textcolor".to_string(), self.textcolor.clone());
        fields.insert("editable".to_string(), self.editable.clone());
        fields.insert("size".to_string(), self.size.clone());
        fields.insert("show_last".to_string(), self.show_last.clone());
        fields.insert("display".to_string(), self.display.clone());
        fields.insert("format".to_string(), self.format.clone());
        fields.insert("precision".to_string(), self.precision.clone());
        fields.insert("force_overlay".to_string(), self.force_overlay.clone());

        Ok(Value::Na)
    }
}

/// Register plot functions as global functions
pub fn register_plot_functions() -> HashMap<String, Value> {
    let mut functions = HashMap::new();

    functions.insert(
        "plot".to_string(),
        Value::BuiltinFunction(Rc::new(Plot::builtin_fn)),
    );
    functions.insert(
        "plotarrow".to_string(),
        Value::BuiltinFunction(Rc::new(Plotarrow::builtin_fn)),
    );
    functions.insert(
        "plotbar".to_string(),
        Value::BuiltinFunction(Rc::new(Plotbar::builtin_fn)),
    );
    functions.insert(
        "plotcandle".to_string(),
        Value::BuiltinFunction(Rc::new(Plotcandle::builtin_fn)),
    );
    functions.insert(
        "plotchar".to_string(),
        Value::BuiltinFunction(Rc::new(Plotchar::builtin_fn)),
    );
    functions.insert(
        "plotshape".to_string(),
        Value::BuiltinFunction(Rc::new(Plotshape::builtin_fn)),
    );

    functions
}
