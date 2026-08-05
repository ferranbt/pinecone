//! The `library(...)` declaration.
//!
//! Marks a script as a reusable library (v5+). Like [`indicator`](crate::indicator),
//! it is a global declaration function; a script may have at most one (enforced
//! by sema). It records the declaration via [`MetadataOutput`].

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineVersion;
use pine_core::{Library, MetadataOutput, PineOutput};
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// library(title, overlay, dynamic_requests)
#[derive(BuiltinFunction)]
#[builtin(name = "library", output = MetadataOutput)]
struct LibraryFn {
    title: String,
    #[arg(default = false)]
    overlay: bool,
    #[arg(default = None)]
    dynamic_requests: Option<bool>,
}

impl LibraryFn {
    fn execute<O: PineOutput + MetadataOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        ctx.output.set_library(Library {
            title: self.title.clone(),
            overlay: self.overlay,
            dynamic_requests: self.dynamic_requests,
        });
        Ok(Value::Na)
    }
}

pub fn register<O: PineOutput + MetadataOutput>(
    _version: PineVersion,
) -> Vec<(String, Value<O>)> {
    vec![("library".to_string(), LibraryFn::builtin_value::<O>())]
}
