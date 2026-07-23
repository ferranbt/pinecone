use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{
    Color, Interpreter, PineOutput, Plot as PlotStruct, PlotOutput, Plotarrow as PlotarrowStruct,
    Plotbar as PlotbarStruct, Plotcandle as PlotcandleStruct, Plotchar as PlotcharStruct,
    Plotshape as PlotshapeStruct, RuntimeError, Value,
};
use std::collections::HashMap;

/// plot() - Plots a series of data on the chart
#[derive(BuiltinFunction)]
#[builtin(name = "plot")]
struct Plot<O: PineOutput + PlotOutput> {
    series: Value<O>,
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

impl<O: PineOutput + PlotOutput> Plot<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct Plotarrow<O: PineOutput + PlotOutput> {
    series: Value<O>,
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

impl<O: PineOutput + PlotOutput> Plotarrow<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct Plotbar<O: PineOutput + PlotOutput> {
    open: Value<O>,
    high: Value<O>,
    low: Value<O>,
    close: Value<O>,
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

impl<O: PineOutput + PlotOutput> Plotbar<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct Plotcandle<O: PineOutput + PlotOutput> {
    open: Value<O>,
    high: Value<O>,
    low: Value<O>,
    close: Value<O>,
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

impl<O: PineOutput + PlotOutput> Plotcandle<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct Plotchar<O: PineOutput + PlotOutput> {
    series: Value<O>,
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

impl<O: PineOutput + PlotOutput> Plotchar<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
struct Plotshape<O: PineOutput + PlotOutput> {
    series: Value<O>,
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

impl<O: PineOutput + PlotOutput> Plotshape<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
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
pub fn register_plot_functions<O: PineOutput + PlotOutput>() -> HashMap<String, Value<O>> {
    let mut functions: HashMap<String, Value<O>> = HashMap::new();

    functions.insert("plot".to_string(), Plot::<O>::builtin_value());
    functions.insert("plotarrow".to_string(), Plotarrow::<O>::builtin_value());
    functions.insert("plotbar".to_string(), Plotbar::<O>::builtin_value());
    functions.insert("plotcandle".to_string(), Plotcandle::<O>::builtin_value());
    functions.insert("plotchar".to_string(), Plotchar::<O>::builtin_value());
    functions.insert("plotshape".to_string(), Plotshape::<O>::builtin_value());

    functions
}
