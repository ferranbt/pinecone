//! Builtin identifiers implemented for a Pine version, sorted, one per line.
//! `cargo run -p pine-builtins --bin symbols -- [version]` (default latest).
//! Diff against `pine-reference symbols` to find what is missing.

use pine_builtins::{per_bar_variables, register_namespace_objects};
use pine_core::{Bar, DefaultPineOutput, PineVersion};
use pine_interpreter::Value;

fn walk(name: &str, value: &Value<DefaultPineOutput>, out: &mut Vec<String>) {
    match value {
        Value::Object {
            fields,
            call,
            value,
            ..
        } => {
            if call.is_some() || value.is_some() {
                // A namespace usable bare too: callable (`strategy`) or
                // value-carrying (`dayofweek`, `strategy.closedtrades`).
                out.push(name.to_string());
            }
            for (field, v) in fields.borrow().iter() {
                walk(&format!("{name}.{field}"), v, out);
            }
        }
        _ => out.push(name.to_string()),
    }
}

fn main() {
    let version = std::env::args()
        .nth(1)
        .and_then(|a| a.parse::<u8>().ok())
        .and_then(PineVersion::from_number)
        .unwrap_or(PineVersion::LATEST);

    let (mut env, _) = register_namespace_objects::<DefaultPineOutput>(version, None, None);
    for (name, value) in per_bar_variables::<DefaultPineOutput>(&Bar::default()) {
        env.insert(name, value);
    }

    let mut names = Vec::new();
    for (name, value) in &env {
        walk(name, value, &mut names);
    }
    names.sort();
    names.dedup();
    for name in names {
        println!("{name}");
    }
}
