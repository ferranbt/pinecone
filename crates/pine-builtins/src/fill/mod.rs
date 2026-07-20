//! The `fill(id1, id2, color, title, transp)` global function.
//!
//! Fills the area between two plots or hlines. The ids come from `plot`/`hline`
//! (which may be `na`); the fill is recorded via [`FillOutput`].

use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{
    Color, FillObject, FillOutput, Interpreter, PineOutput, RuntimeError, Value,
};
use std::rc::Rc;

/// fill(id1, id2, color, title, transp, ...)
#[derive(BuiltinFunction)]
#[builtin(name = "fill", output = FillOutput)]
struct Fill {
    #[arg(default = None)]
    id1: Option<f64>,
    #[arg(default = None)]
    id2: Option<f64>,
    #[arg(default = None)]
    color: Option<Color>,
    #[arg(default = "")]
    title: String,
    #[arg(default = None)]
    transp: Option<f64>,
}

impl Fill {
    fn execute<O: PineOutput + FillOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = self.transp;
        ctx.output.add_fill(FillObject {
            id1: self.id1.map(|x| x as usize),
            id2: self.id2.map(|x| x as usize),
            color: self.color.clone(),
            title: self.title.clone(),
        });
        Ok(Value::Na)
    }
}

/// The `fill` global function value.
pub fn register<O: PineOutput + FillOutput>() -> Value<O> {
    Value::BuiltinFunction(Rc::new(Fill::builtin_fn::<O>))
}
