//! The `indicator(...)` declaration.
//!
//! A global function (not a namespace) that declares the script's identity and
//! display settings. It records the declaration into the output via
//! [`MetadataOutput`]; a script may have at most one (enforced by sema).

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineVersion;
use pine_core::{Indicator, MetadataOutput, PineOutput};
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// indicator(title, shorttitle, overlay, format, precision, ...)
#[derive(BuiltinFunction)]
#[builtin(name = "indicator", output = MetadataOutput)]
struct IndicatorFn {
    title: String,
    #[arg(default = "")]
    shorttitle: String,
    #[arg(default = false)]
    overlay: bool,
    #[arg(default = "")]
    format: String,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = "")]
    timeframe: String,
    // Accepted and ignored: chart-capacity and display hints.
    #[arg(default = None)]
    max_bars_back: Option<f64>,
    #[arg(default = None)]
    max_lines_count: Option<f64>,
    #[arg(default = None)]
    max_labels_count: Option<f64>,
    #[arg(default = None)]
    max_boxes_count: Option<f64>,
    #[arg(default = None)]
    max_polylines_count: Option<f64>,
    #[arg(default = None)]
    calc_bars_count: Option<f64>,
    #[arg(default = "")]
    scale: String,
    #[arg(default = None)]
    timeframe_gaps: Option<bool>,
    #[arg(default = None)]
    explicit_plot_zorder: Option<bool>,
    #[arg(default = None)]
    dynamic_requests: Option<bool>,
    #[arg(default = None)]
    behind_chart: Option<bool>,
}

impl IndicatorFn {
    fn execute<O: PineOutput + MetadataOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = (
            self.max_bars_back,
            self.max_lines_count,
            self.max_labels_count,
            self.max_boxes_count,
            self.max_polylines_count,
            self.calc_bars_count,
            &self.scale,
            self.timeframe_gaps,
            self.explicit_plot_zorder,
            self.dynamic_requests,
            self.behind_chart,
        );
        ctx.output.set_indicator(Indicator {
            title: self.title.clone(),
            shorttitle: self.shorttitle.clone(),
            overlay: self.overlay,
            format: self.format.clone(),
            precision: self.precision.map(|p| p as i64),
            timeframe: self.timeframe.clone(),
        });
        Ok(Value::Na)
    }
}

pub fn register<O: PineOutput + MetadataOutput>(version: PineVersion) -> Vec<(String, Value<O>)> {
    let name = if version < PineVersion::V5 {
        "study"
    } else {
        "indicator"
    };
    vec![(name.to_string(), IndicatorFn::builtin_value::<O>())]
}
