//! The `barstate.*` namespace: booleans describing the bar being executed.
//!
//! Unlike the compile-time namespaces, these change every bar, so they are built
//! per bar from the [`Bar`]'s flags (see `register_per_bar` in the crate root).

use pine_core::Bar;
use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Build the `barstate` object for `bar`.
pub fn register<O: PineOutput>(bar: &Bar) -> Value<O> {
    let members: HashMap<String, Value<O>> = [
        ("isfirst", bar.is_first),
        ("islast", bar.is_last),
        ("isnew", bar.is_new),
        ("isconfirmed", bar.is_confirmed),
        ("ishistory", bar.is_history),
        ("isrealtime", bar.is_realtime),
        ("islastconfirmedhistory", bar.is_last_confirmed_history),
    ]
    .into_iter()
    .map(|(name, flag)| (name.to_string(), Value::Bool(flag)))
    .collect();

    Value::Object {
        type_name: "barstate".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
