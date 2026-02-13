use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Color, Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// plot() - Plots a series of data on the chart
#[derive(BuiltinFunction)]
#[builtin(name = "plot")]
struct Plot {
    series: Value,
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = 1.0)]
    linewidth: f64,
    #[arg(default = "line")]
    style: String,
    #[arg(default = false)]
    trackprice: bool,
    #[arg(default = 0.0)]
    histbase: f64,
    #[arg(default = 0.0)]
    offset: f64,
    #[arg(default = false)]
    join: bool,
    #[arg(default = true)]
    editable: bool,
    #[arg(default = None)]
    show_last: Option<f64>,
    #[arg(default = "all")]
    display: String,
    #[arg(default = None)]
    format: Option<String>,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = false)]
    force_overlay: bool,
    #[arg(default = "solid")]
    linestyle: String,
}

impl Plot {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), Value::String(self.title.clone()));
        fields.insert("color".to_string(), self.color.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("linewidth".to_string(), Value::Number(self.linewidth));
        fields.insert("style".to_string(), Value::String(self.style.clone()));
        fields.insert("trackprice".to_string(), Value::Bool(self.trackprice));
        fields.insert("histbase".to_string(), Value::Number(self.histbase));
        fields.insert("offset".to_string(), Value::Number(self.offset));
        fields.insert("join".to_string(), Value::Bool(self.join));
        fields.insert("editable".to_string(), Value::Bool(self.editable));
        fields.insert("show_last".to_string(), self.show_last.map_or(Value::Na, Value::Number));
        fields.insert("display".to_string(), Value::String(self.display.clone()));
        fields.insert("format".to_string(), self.format.as_ref().map_or(Value::Na, |s| Value::String(s.clone())));
        fields.insert("precision".to_string(), self.precision.map_or(Value::Na, Value::Number));
        fields.insert("force_overlay".to_string(), Value::Bool(self.force_overlay));
        fields.insert(
            "linestyle".to_string(),
            Value::String(self.linestyle.clone()),
        );

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
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    colorup: Option<Color>,
    #[arg(default = None)]
    colordown: Option<Color>,
    #[arg(default = 0.0)]
    offset: f64,
    #[arg(default = 5.0)]
    minheight: f64,
    #[arg(default = 100.0)]
    maxheight: f64,
    #[arg(default = true)]
    editable: bool,
    #[arg(default = None)]
    show_last: Option<f64>,
    #[arg(default = "all")]
    display: String,
    #[arg(default = None)]
    format: Option<String>,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = false)]
    force_overlay: bool,
}

impl Plotarrow {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), Value::String(self.title.clone()));
        fields.insert("colorup".to_string(), self.colorup.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("colordown".to_string(), self.colordown.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("offset".to_string(), Value::Number(self.offset));
        fields.insert("minheight".to_string(), Value::Number(self.minheight));
        fields.insert("maxheight".to_string(), Value::Number(self.maxheight));
        fields.insert("editable".to_string(), Value::Bool(self.editable));
        fields.insert("show_last".to_string(), self.show_last.map_or(Value::Na, Value::Number));
        fields.insert("display".to_string(), Value::String(self.display.clone()));
        fields.insert("format".to_string(), self.format.as_ref().map_or(Value::Na, |s| Value::String(s.clone())));
        fields.insert("precision".to_string(), self.precision.map_or(Value::Na, Value::Number));
        fields.insert("force_overlay".to_string(), Value::Bool(self.force_overlay));

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
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = true)]
    editable: bool,
    #[arg(default = None)]
    show_last: Option<f64>,
    #[arg(default = "all")]
    display: String,
    #[arg(default = None)]
    format: Option<String>,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = false)]
    force_overlay: bool,
}

impl Plotbar {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("open".to_string(), self.open.clone());
        fields.insert("high".to_string(), self.high.clone());
        fields.insert("low".to_string(), self.low.clone());
        fields.insert("close".to_string(), self.close.clone());
        fields.insert("title".to_string(), Value::String(self.title.clone()));
        fields.insert("color".to_string(), self.color.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("editable".to_string(), Value::Bool(self.editable));
        fields.insert("show_last".to_string(), self.show_last.map_or(Value::Na, Value::Number));
        fields.insert("display".to_string(), Value::String(self.display.clone()));
        fields.insert("format".to_string(), self.format.as_ref().map_or(Value::Na, |s| Value::String(s.clone())));
        fields.insert("precision".to_string(), self.precision.map_or(Value::Na, Value::Number));
        fields.insert("force_overlay".to_string(), Value::Bool(self.force_overlay));

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
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = None)]
    wickcolor: Option<Color>,
    #[arg(default = true)]
    editable: bool,
    #[arg(default = None)]
    show_last: Option<f64>,
    #[arg(default = None)]
    bordercolor: Option<Color>,
    #[arg(default = "all")]
    display: String,
    #[arg(default = None)]
    format: Option<String>,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = false)]
    force_overlay: bool,
}

impl Plotcandle {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("open".to_string(), self.open.clone());
        fields.insert("high".to_string(), self.high.clone());
        fields.insert("low".to_string(), self.low.clone());
        fields.insert("close".to_string(), self.close.clone());
        fields.insert("title".to_string(), Value::String(self.title.clone()));
        fields.insert("color".to_string(), self.color.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("wickcolor".to_string(), self.wickcolor.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("editable".to_string(), Value::Bool(self.editable));
        fields.insert("show_last".to_string(), self.show_last.map_or(Value::Na, Value::Number));
        fields.insert("bordercolor".to_string(), self.bordercolor.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("display".to_string(), Value::String(self.display.clone()));
        fields.insert("format".to_string(), self.format.as_ref().map_or(Value::Na, |s| Value::String(s.clone())));
        fields.insert("precision".to_string(), self.precision.map_or(Value::Na, Value::Number));
        fields.insert("force_overlay".to_string(), Value::Bool(self.force_overlay));

        Ok(Value::Na)
    }
}

/// plotchar() - Plots visual shapes on the chart using ASCII characters
#[derive(BuiltinFunction)]
#[builtin(name = "plotchar")]
struct Plotchar {
    series: Value,
    #[arg(default = "")]
    title: String,
    #[arg(default = "★")]
    char: String,
    #[arg(default = "bottom")]
    location: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = 0.0)]
    offset: f64,
    #[arg(default = "")]
    text: String,
    #[arg(default = None)]
    textcolor: Option<Color>,
    #[arg(default = true)]
    editable: bool,
    #[arg(default = "auto")]
    size: String,
    #[arg(default = None)]
    show_last: Option<f64>,
    #[arg(default = "all")]
    display: String,
    #[arg(default = None)]
    format: Option<String>,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = false)]
    force_overlay: bool,
}

impl Plotchar {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), Value::String(self.title.clone()));
        fields.insert("char".to_string(), Value::String(self.char.clone()));
        fields.insert("location".to_string(), Value::String(self.location.clone()));
        fields.insert("color".to_string(), self.color.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("offset".to_string(), Value::Number(self.offset));
        fields.insert("text".to_string(), Value::String(self.text.clone()));
        fields.insert("textcolor".to_string(), self.textcolor.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("editable".to_string(), Value::Bool(self.editable));
        fields.insert("size".to_string(), Value::String(self.size.clone()));
        fields.insert("show_last".to_string(), self.show_last.map_or(Value::Na, Value::Number));
        fields.insert("display".to_string(), Value::String(self.display.clone()));
        fields.insert("format".to_string(), self.format.as_ref().map_or(Value::Na, |s| Value::String(s.clone())));
        fields.insert("precision".to_string(), self.precision.map_or(Value::Na, Value::Number));
        fields.insert("force_overlay".to_string(), Value::Bool(self.force_overlay));

        Ok(Value::Na)
    }
}

/// plotshape() - Plots visual shapes on the chart
#[derive(BuiltinFunction)]
#[builtin(name = "plotshape")]
struct Plotshape {
    series: Value,
    #[arg(default = "")]
    title: String,
    #[arg(default = "circle")]
    style: String,
    #[arg(default = "bottom")]
    location: String,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = 0.0)]
    offset: f64,
    #[arg(default = "")]
    text: String,
    #[arg(default = None)]
    textcolor: Option<Color>,
    #[arg(default = true)]
    editable: bool,
    #[arg(default = "auto")]
    size: String,
    #[arg(default = None)]
    show_last: Option<f64>,
    #[arg(default = "all")]
    display: String,
    #[arg(default = None)]
    format: Option<String>,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = false)]
    force_overlay: bool,
}

impl Plotshape {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let mut fields = HashMap::new();
        fields.insert("series".to_string(), self.series.clone());
        fields.insert("title".to_string(), Value::String(self.title.clone()));
        fields.insert("style".to_string(), Value::String(self.style.clone()));
        fields.insert("location".to_string(), Value::String(self.location.clone()));
        fields.insert("color".to_string(), self.color.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("offset".to_string(), Value::Number(self.offset));
        fields.insert("text".to_string(), Value::String(self.text.clone()));
        fields.insert("textcolor".to_string(), self.textcolor.clone().map(|c| Value::Color(c)).unwrap_or(Value::Na));
        fields.insert("editable".to_string(), Value::Bool(self.editable));
        fields.insert("size".to_string(), Value::String(self.size.clone()));
        fields.insert("show_last".to_string(), self.show_last.map_or(Value::Na, Value::Number));
        fields.insert("display".to_string(), Value::String(self.display.clone()));
        fields.insert("format".to_string(), self.format.as_ref().map_or(Value::Na, |s| Value::String(s.clone())));
        fields.insert("precision".to_string(), self.precision.map_or(Value::Na, Value::Number));
        fields.insert("force_overlay".to_string(), Value::Bool(self.force_overlay));

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
