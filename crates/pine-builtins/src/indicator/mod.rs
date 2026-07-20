//! The `indicator(...)` declaration.
//!
//! A global function (not a namespace) that declares the script's identity and
//! display settings. It records the declaration into the output via
//! [`IndicatorOutput`]; a script may have at most one (enforced by sema).

use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Indicator, IndicatorOutput, Interpreter, PineOutput, RuntimeError, Value};
use std::rc::Rc;

/// indicator(title, shorttitle, overlay, format, precision, ...)
#[derive(BuiltinFunction)]
#[builtin(name = "indicator", output = IndicatorOutput)]
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
    #[arg(default = "")]
    scale: String,
}

impl IndicatorFn {
    fn execute<O: PineOutput + IndicatorOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = (
            self.max_bars_back,
            self.max_lines_count,
            self.max_labels_count,
            self.max_boxes_count,
            &self.scale,
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

/// The `indicator` global function value.
pub fn register<O: PineOutput + IndicatorOutput>() -> Value<O> {
    Value::BuiltinFunction(Rc::new(IndicatorFn::builtin_fn::<O>))
}
