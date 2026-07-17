use pine_interpreter::{BuiltinFn, DefaultPineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const STYLES: &[&str] = &["style_dashed", "style_dotted", "style_solid"];

/// `hline` is both callable (`hline(0)`) and a namespace (`hline.style_dotted`).
///
/// The call does not draw anything yet — it accepts its arguments and returns
/// `na` so scripts using it compile and run.
pub fn register() -> Value<DefaultPineOutput> {
    let fields: HashMap<String, Value<DefaultPineOutput>> = STYLES
        .iter()
        .map(|s| (s.to_string(), Value::String(s.to_string())))
        .collect();

    let hline_fn: BuiltinFn<DefaultPineOutput> = Rc::new(|_ctx, _args| Ok(Value::Na));

    Value::Object {
        type_name: "hline".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: Some(hline_fn),
    }
}
