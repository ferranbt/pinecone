//! Chart-wide global functions that write to the output's [`GlobalContext`]:
//! `bgcolor` (background color) and `barcolor` (price-bar color).
//!
//! Both take a color (often conditional, so `na` when it should not paint) plus
//! display arguments that are accepted and ignored in a headless run.

use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Color, GlobalOutput, Interpreter, PineOutput, RuntimeError, Value};
use std::collections::HashMap;

/// bgcolor(color, offset, editable, show_last, title, transp, ...)
#[derive(BuiltinFunction)]
#[builtin(name = "bgcolor", output = GlobalOutput)]
struct Bgcolor {
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    transp: Option<f64>,
    #[arg(default = None)]
    offset: Option<f64>,
    #[arg(default = false)]
    editable: bool,
    #[arg(default = "")]
    display: String,
}

impl Bgcolor {
    fn execute<O: PineOutput + GlobalOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = (
            &self.title,
            self.transp,
            self.offset,
            self.editable,
            &self.display,
        );
        ctx.output.set_bgcolor(self.color.clone());
        Ok(Value::Na)
    }
}

/// barcolor(color, offset, editable, show_last, title, ...)
#[derive(BuiltinFunction)]
#[builtin(name = "barcolor", output = GlobalOutput)]
struct Barcolor {
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    offset: Option<f64>,
    #[arg(default = false)]
    editable: bool,
    #[arg(default = "")]
    display: String,
}

impl Barcolor {
    fn execute<O: PineOutput + GlobalOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = (&self.title, self.offset, self.editable, &self.display);
        ctx.output.set_barcolor(self.color.clone());
        Ok(Value::Na)
    }
}

/// The `bgcolor` and `barcolor` global functions.
pub fn register<O: PineOutput + GlobalOutput>() -> HashMap<String, Value<O>> {
    let mut globals: HashMap<String, Value<O>> = HashMap::new();
    globals.insert("bgcolor".to_string(), Bgcolor::builtin_value::<O>());
    globals.insert("barcolor".to_string(), Barcolor::builtin_value::<O>());
    globals
}
