//! The `alertcondition(...)` declaration and the `alert(...)` fire, which share
//! the [`AlertConditionOutput`] sink.

use pine_builtin_macro::BuiltinFunction;
use pine_core::{AlertCondition, AlertConditionOutput, Frequency, PineOutput};
use pine_interpreter::{Builtin, BuiltinFn, Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// alertcondition(condition, title, message)
#[derive(BuiltinFunction)]
#[builtin(name = "alertcondition")]
struct Alertcondition<O: PineOutput + AlertConditionOutput> {
    condition: Value<O>,
    #[arg(default = "")]
    title: String,
    #[arg(default = "")]
    message: String,
}

impl<O: PineOutput + AlertConditionOutput> Alertcondition<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // Record (fire) the alert only on bars where the condition is true.
        if self.condition.truthy_for_condition()? {
            ctx.output.add_alertcondition(AlertCondition {
                title: self.title.clone(),
                message: self.message.clone(),
                frequency: None,
            });
        }
        Ok(Value::Na)
    }
}

/// The `alertcondition` global function value.
pub fn register<O: PineOutput + AlertConditionOutput>() -> Value<O> {
    Alertcondition::<O>::builtin_value()
}

/// alert(message, freq) - Fire an alert, recording it via the same sink.
#[derive(BuiltinFunction)]
#[builtin(name = "alert", output = AlertConditionOutput)]
struct Alert {
    message: String,
    #[arg(default = "freq_once_per_bar")]
    freq: String,
}

impl Alert {
    fn execute<O: PineOutput + AlertConditionOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        ctx.output.add_alertcondition(AlertCondition {
            title: String::new(),
            message: self.message.clone(),
            frequency: Some(Frequency::from_const(&self.freq)),
        });
        Ok(Value::Na)
    }
}

/// The callable `alert` namespace: `alert(...)` plus the `freq_*` constants.
pub fn register_alert<O: PineOutput + AlertConditionOutput>() -> Value<O> {
    let mut fields: HashMap<String, Value<O>> = HashMap::new();
    for freq in ["freq_all", "freq_once_per_bar", "freq_once_per_bar_close"] {
        fields.insert(freq.to_string(), Value::String(freq.to_string()));
    }
    Value::Object {
        type_name: "alert".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: Some(Builtin {
            call: Rc::new(Alert::builtin_fn::<O>) as BuiltinFn<O>,
            signature: Alert::signature(),
        }),
        value: None,
    }
}
