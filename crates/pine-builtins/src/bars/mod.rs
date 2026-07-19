use pine_core::BarState;
use pine_interpreter::{Bar, PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The `barstate` namespace for `state`.
///
/// Every member is a plain value, since scripts read them without parentheses
/// (`barstate.islast`).
fn bar_state_namespace<O: PineOutput>(state: BarState) -> Value<O> {
    let fields: HashMap<String, Value<O>> = [
        ("isfirst", state.is_first()),
        ("islast", state.is_last()),
        ("ishistory", state.is_history()),
        ("isrealtime", state.is_realtime()),
        ("isconfirmed", state.is_confirmed()),
        (
            "islastconfirmedhistory",
            state.is_last_confirmed_history(),
        ),
        // Only a realtime bar is recalculated, so any other state is being
        // calculated for the first time.
        ("isnew", state.is_confirmed()),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_string(), Value::Bool(value)))
    .collect();

    Value::Object {
        type_name: "barstate".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}

/// The `barstate` namespace describing `bar`.
pub fn register_bar_state<O: PineOutput>(bar: &Bar) -> (&'static str, Value<O>) {
    ("barstate", bar_state_namespace(bar.state))
}
