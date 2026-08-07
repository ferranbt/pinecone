//! The `ticker.*` namespace: builds the ticker-id strings `request.security`
//! consumes. The exact encoding is ours, not TradingView's — `prefix:ticker`
//! with a `_Modifier` suffix per non-standard chart type — since nothing here
//! needs to match Pine's opaque format, only round-trip through `ticker.standard`.

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The chart-type suffixes appended by the modifier constructors.
const MODIFIERS: &[&str] = &["_HeikinAshi", "_Renko", "_Kagi", "_PnF", "_LineBreak"];

/// The base symbol with any chart-type modifier removed.
fn base(symbol: &str) -> &str {
    for modifier in MODIFIERS {
        if let Some(stripped) = symbol.strip_suffix(modifier) {
            return stripped;
        }
    }
    symbol
}

/// Defines a chart-type modifier constructor: `symbol` plus any extra styling
/// arguments (absorbed and ignored), returning `symbol` with `$suffix`.
macro_rules! ticker_modifier {
    ($ident:ident, $name:literal, $suffix:literal) => {
        #[derive(BuiltinFunction)]
        #[builtin(name = $name)]
        struct $ident<O: PineOutput> {
            symbol: String,
            #[arg(variadic)]
            styling: Vec<Value<O>>,
        }

        impl<O: PineOutput> $ident<O> {
            fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
                let _ = &self.styling;
                Ok(Value::String(format!("{}{}", base(&self.symbol), $suffix)))
            }
        }
    };
}

ticker_modifier!(TickerHeikinashi, "ticker.heikinashi", "_HeikinAshi");
ticker_modifier!(TickerRenko, "ticker.renko", "_Renko");
ticker_modifier!(TickerKagi, "ticker.kagi", "_Kagi");
ticker_modifier!(TickerPointfigure, "ticker.pointfigure", "_PnF");
ticker_modifier!(TickerLinebreak, "ticker.linebreak", "_LineBreak");

/// ticker.new(prefix, ticker, ...) - Build `prefix:ticker`.
#[derive(BuiltinFunction)]
#[builtin(name = "ticker.new")]
struct TickerNew<O: PineOutput> {
    prefix: String,
    ticker: String,
    #[arg(variadic)]
    modifiers: Vec<Value<O>>,
}

impl<O: PineOutput> TickerNew<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.modifiers;
        Ok(Value::String(format!("{}:{}", self.prefix, self.ticker)))
    }
}

/// ticker.modify(tickerid, ...) - Change session/adjustment on a ticker id. Those
/// modifiers are not part of our encoding, so the id is returned unchanged.
#[derive(BuiltinFunction)]
#[builtin(name = "ticker.modify")]
struct TickerModify<O: PineOutput> {
    tickerid: String,
    #[arg(variadic)]
    modifiers: Vec<Value<O>>,
}

impl<O: PineOutput> TickerModify<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.modifiers;
        Ok(Value::String(self.tickerid.clone()))
    }
}

/// ticker.standard(symbol) - The symbol with any chart-type modifier removed.
#[derive(BuiltinFunction)]
#[builtin(name = "ticker.standard")]
struct TickerStandard {
    #[arg(default = "")]
    symbol: String,
}

impl TickerStandard {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::String(base(&self.symbol).to_string()))
    }
}

/// ticker.inherit(from_tickerid, symbol) - Apply `from_tickerid`'s chart-type
/// modifier to `symbol`.
#[derive(BuiltinFunction)]
#[builtin(name = "ticker.inherit")]
struct TickerInherit {
    from_tickerid: String,
    symbol: String,
}

impl TickerInherit {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let modifier = MODIFIERS
            .iter()
            .find(|m| self.from_tickerid.ends_with(**m))
            .copied()
            .unwrap_or("");
        Ok(Value::String(format!("{}{}", base(&self.symbol), modifier)))
    }
}

/// Register the `ticker.*` namespace object.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    members.insert("new".to_string(), TickerNew::<O>::builtin_value());
    members.insert("modify".to_string(), TickerModify::<O>::builtin_value());
    members.insert(
        "heikinashi".to_string(),
        TickerHeikinashi::<O>::builtin_value(),
    );
    members.insert("renko".to_string(), TickerRenko::<O>::builtin_value());
    members.insert("kagi".to_string(), TickerKagi::<O>::builtin_value());
    members.insert(
        "pointfigure".to_string(),
        TickerPointfigure::<O>::builtin_value(),
    );
    members.insert(
        "linebreak".to_string(),
        TickerLinebreak::<O>::builtin_value(),
    );
    members.insert("standard".to_string(), TickerStandard::builtin_value::<O>());
    members.insert("inherit".to_string(), TickerInherit::builtin_value::<O>());
    Value::Object {
        type_name: "ticker".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
