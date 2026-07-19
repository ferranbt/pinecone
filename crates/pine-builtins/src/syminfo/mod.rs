use pine_core::SymInfo;
use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Build the `syminfo` namespace object from host-supplied symbol information.
pub fn create_syminfo<O: PineOutput>(info: SymInfo) -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert("ticker".to_string(), Value::String(info.ticker));
    members.insert("tickerid".to_string(), Value::String(info.tickerid));
    members.insert("description".to_string(), Value::String(info.description));
    members.insert("prefix".to_string(), Value::String(info.prefix));
    members.insert("currency".to_string(), Value::String(info.currency));
    members.insert("basecurrency".to_string(), Value::String(info.basecurrency));
    members.insert("type".to_string(), Value::String(info.type_));
    members.insert("mintick".to_string(), Value::Number(info.mintick));
    members.insert("pointvalue".to_string(), Value::Number(info.pointvalue));
    members.insert("timezone".to_string(), Value::String(info.timezone));
    members.insert("session".to_string(), Value::String(info.session));

    Value::Object {
        type_name: "syminfo".to_string(),
        fields: Rc::new(RefCell::new(members)),
    }
}
