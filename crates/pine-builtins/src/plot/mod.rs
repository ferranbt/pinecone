use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{
    Color, Interpreter, Plot as PlotStruct, PlotOutput, Plotarrow as PlotarrowStruct,
    Plotbar as PlotbarStruct, Plotcandle as PlotcandleStruct, Plotchar as PlotcharStruct,
    Plotshape as PlotshapeStruct, RuntimeError, Value,
};
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let plot = PlotStruct {
            series: self.series.as_number()?,
            title: self.title.clone(),
            color: self.color.clone(),
            linewidth: self.linewidth,
            style: self.style.clone(),
            trackprice: self.trackprice,
            histbase: self.histbase,
            offset: self.offset,
            join: self.join,
            editable: self.editable,
            show_last: self.show_last,
            display: self.display.clone(),
            format: self.format.clone(),
            precision: self.precision,
            force_overlay: self.force_overlay,
            linestyle: self.linestyle.clone(),
        };

        ctx.output.add_plot(plot);
        Ok(Value::Na)
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let plotarrow = PlotarrowStruct {
            series: self.series.as_number()?,
            title: self.title.clone(),
            colorup: self.colorup.clone(),
            colordown: self.colordown.clone(),
            offset: self.offset,
            minheight: self.minheight,
            maxheight: self.maxheight,
            editable: self.editable,
            show_last: self.show_last,
            display: self.display.clone(),
            format: self.format.clone(),
            precision: self.precision,
            force_overlay: self.force_overlay,
        };

        ctx.output.add_plotarrow(plotarrow);
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let plotbar = PlotbarStruct {
            open: self.open.as_number()?,
            high: self.high.as_number()?,
            low: self.low.as_number()?,
            close: self.close.as_number()?,
            title: self.title.clone(),
            color: self.color.clone(),
            editable: self.editable,
            show_last: self.show_last,
            display: self.display.clone(),
            format: self.format.clone(),
            precision: self.precision,
            force_overlay: self.force_overlay,
        };

        ctx.output.add_plotbar(plotbar);
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let plotcandle = PlotcandleStruct {
            open: self.open.as_number()?,
            high: self.high.as_number()?,
            low: self.low.as_number()?,
            close: self.close.as_number()?,
            title: self.title.clone(),
            color: self.color.clone(),
            wickcolor: self.wickcolor.clone(),
            editable: self.editable,
            show_last: self.show_last,
            bordercolor: self.bordercolor.clone(),
            display: self.display.clone(),
            format: self.format.clone(),
            precision: self.precision,
            force_overlay: self.force_overlay,
        };

        ctx.output.add_plotcandle(plotcandle);
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let plotchar = PlotcharStruct {
            series: self.series.as_number()?,
            title: self.title.clone(),
            char: self.char.clone(),
            location: self.location.clone(),
            color: self.color.clone(),
            offset: self.offset,
            text: self.text.clone(),
            textcolor: self.textcolor.clone(),
            editable: self.editable,
            size: self.size.clone(),
            show_last: self.show_last,
            display: self.display.clone(),
            format: self.format.clone(),
            precision: self.precision,
            force_overlay: self.force_overlay,
        };

        ctx.output.add_plotchar(plotchar);
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
    fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let plotshape = PlotshapeStruct {
            series: self.series.as_number()?,
            title: self.title.clone(),
            style: self.style.clone(),
            location: self.location.clone(),
            color: self.color.clone(),
            offset: self.offset,
            text: self.text.clone(),
            textcolor: self.textcolor.clone(),
            editable: self.editable,
            size: self.size.clone(),
            show_last: self.show_last,
            display: self.display.clone(),
            format: self.format.clone(),
            precision: self.precision,
            force_overlay: self.force_overlay,
        };

        ctx.output.add_plotshape(plotshape);
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
