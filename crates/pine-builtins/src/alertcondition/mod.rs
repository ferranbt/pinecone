//! The `alertcondition(condition, title, message)` declaration.
//!
//! A global function that declares a named alert. The condition is a per-bar
//! boolean series; in a headless run there is no alerting engine, so the
//! declaration (title + message) is simply recorded via [`AlertConditionOutput`].

use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{
    AlertCondition, AlertConditionOutput, Interpreter, PineOutput, RuntimeError, Value,
};
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
            });
        }
        Ok(Value::Na)
    }
}

/// The `alertcondition` global function value.
pub fn register<O: PineOutput + AlertConditionOutput>() -> Value<O> {
    Value::BuiltinFunction(Rc::new(Alertcondition::<O>::builtin_fn))
}
