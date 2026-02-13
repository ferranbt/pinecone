/// Example: Custom Builtin Functions with Custom Output
///
/// This example demonstrates how to extend Pinecone with:
/// 1. A custom output type that stores additional data (alerts)
/// 2. A custom builtin function that works with the custom output
use pine::Script;
use pine_interpreter::{
    impl_output_traits_delegate, DefaultPineOutput, EvaluatedArg, FunctionCallArgs, Interpreter,
    PineOutput, RuntimeError, Value,
};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Clone, Debug)]
pub struct CustomOutput {
    /// Embed the default output - delegate all base traits to this field
    base: DefaultPineOutput,
    alerts: Vec<String>,
}

impl PineOutput for CustomOutput {
    fn clear(&mut self) {
        self.base.clear();
        self.alerts.clear();
    }
}

// This single macro call implements LogOutput, PlotOutput, LabelOutput, and BoxOutput
// by delegating all methods to self.base - no boilerplate needed!
impl_output_traits_delegate!(CustomOutput, base);

pub trait AlertOutput: PineOutput {
    fn add_alert(&mut self, message: String);
    fn get_alerts(&self) -> &[String];
}

impl AlertOutput for CustomOutput {
    fn add_alert(&mut self, message: String) {
        self.alerts.push(message);
    }

    fn get_alerts(&self) -> &[String] {
        &self.alerts
    }
}

struct AlertFunc;

impl AlertFunc {
    fn execute<O: PineOutput + AlertOutput>(
        ctx: &mut Interpreter<O>,
        args: FunctionCallArgs<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let message = match args.args.first() {
            Some(EvaluatedArg::Positional(Value::String(s))) => s.clone(),
            Some(EvaluatedArg::Positional(v)) => format!("{:?}", v),
            _ => "Alert!".to_string(),
        };

        ctx.output.add_alert(message);

        Ok(Value::Na)
    }
}

fn create_alert_builtin<O: PineOutput + AlertOutput + 'static>() -> Value<O> {
    Value::BuiltinFunction(Rc::new(AlertFunc::execute))
}

fn main() {
    let script_source = r#"
        price = close

        // Trigger alert when price is high
        if price > 100
            alert("Price is high!")
    "#;

    let mut custom_variables: HashMap<String, Value<CustomOutput>> = HashMap::new();
    custom_variables.insert("alert".to_string(), create_alert_builtin());

    let mut script =
        Script::<CustomOutput>::compile_with_variables(script_source, custom_variables)
            .expect("Compilation failed");

    let bar = pine_interpreter::Bar {
        open: 101.0,
        high: 105.0,
        low: 100.0,
        close: 103.0,
        volume: 1500.0,
    };

    let output = script.execute(&bar).expect("Execution failed");

    for alert in output.get_alerts() {
        println!("{}", alert);
    }
}
