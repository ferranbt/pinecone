use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// The developing period's high/low/close, and the last completed one — the basis
/// for the pivot levels. A period runs between `anchor` resets.
#[derive(Default, Clone)]
struct PivotState {
    dev_high: f64,
    dev_low: f64,
    dev_close: f64,
    prev_high: f64,
    prev_low: f64,
    prev_close: f64,
    has_prev: bool,
    started: bool,
}

/// ta.pivot_point_levels(type, anchor, developing) - The pivot levels for the
/// period delimited by `anchor` resets, as `[P, R1, S1, R2, S2, R3, S3, R4, S4,
/// R5, S5]`. Levels a type does not define are `na`.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.pivot_point_levels", stateful)]
pub struct TaPivotPointLevels {
    #[arg(default = "Traditional")]
    kind: String,
    anchor: bool,
    #[arg(default = false)]
    developing: bool,
    #[state]
    state: PivotState,
}

impl TaPivotPointLevels {
    fn execute<O: PineOutput>(
        &mut self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let (high, low, close) = (
            bar_value(ctx, "high"),
            bar_value(ctx, "low"),
            bar_value(ctx, "close"),
        );
        let state = &mut self.state;
        if !state.started {
            state.dev_high = high;
            state.dev_low = low;
            state.dev_close = close;
            state.started = true;
        } else if self.anchor {
            // The developing period just ended; it becomes the basis, and a new
            // one starts at this bar.
            state.prev_high = state.dev_high;
            state.prev_low = state.dev_low;
            state.prev_close = state.dev_close;
            state.has_prev = true;
            state.dev_high = high;
            state.dev_low = low;
            state.dev_close = close;
        } else {
            state.dev_high = state.dev_high.max(high);
            state.dev_low = state.dev_low.min(low);
            state.dev_close = close;
        }

        let basis = if self.developing {
            Some((state.dev_high, state.dev_low, state.dev_close))
        } else if state.has_prev {
            Some((state.prev_high, state.prev_low, state.prev_close))
        } else {
            None
        };
        let levels = match basis {
            Some((h, l, c)) if self.kind == "Traditional" => traditional(h, l, c),
            // Other types (Fibonacci/Woodie/Classic/DM/Camarilla) are a follow-up.
            _ => [f64::NAN; 11],
        };
        let array = levels.iter().map(|v| Value::Number(*v)).collect();
        Ok(Value::Array(Rc::new(RefCell::new(array))))
    }
}

/// Traditional pivots `[P, R1, S1, R2, S2, R3, S3, R4, S4, R5, S5]`.
fn traditional(h: f64, l: f64, c: f64) -> [f64; 11] {
    let p = (h + l + c) / 3.0;
    let range = h - l;
    [
        p,
        p * 2.0 - l,
        p * 2.0 - h,
        p + range,
        p - range,
        p * 2.0 + (h - 2.0 * l),
        p * 2.0 - (2.0 * h - l),
        p * 3.0 + (h - 3.0 * l),
        p * 3.0 - (3.0 * h - l),
        p * 4.0 + (h - 4.0 * l),
        p * 4.0 - (4.0 * h - l),
    ]
}

fn bar_value<O: PineOutput>(ctx: &Interpreter<O>, name: &str) -> f64 {
    match ctx.get_variable(name) {
        Some(Value::Series(s)) => s.current.as_number().unwrap_or(f64::NAN),
        Some(v) => v.as_number().unwrap_or(f64::NAN),
        None => f64::NAN,
    }
}
