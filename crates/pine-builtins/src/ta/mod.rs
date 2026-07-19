use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::rc::Rc;

mod comparison;
mod moving_averages;
mod oscillators;
mod statistics;
mod volatility;

pub use comparison::*;
pub use moving_averages::*;
pub use oscillators::*;
pub use statistics::*;
pub use volatility::*;

/// Register all ta namespace functions and return the namespace object
pub fn register<O: PineOutput>() -> Value<O> {
    let mut ta_ns: std::collections::HashMap<String, Value<O>> = std::collections::HashMap::new();

    // Moving averages
    ta_ns.insert(
        "sma".to_string(),
        Value::BuiltinFunction(Rc::new(TaSma::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "ema".to_string(),
        Value::BuiltinFunction(Rc::new(TaEma::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "rma".to_string(),
        Value::BuiltinFunction(Rc::new(TaRma::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "wma".to_string(),
        Value::BuiltinFunction(Rc::new(TaWma::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "vwma".to_string(),
        Value::BuiltinFunction(Rc::new(TaVwma::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "hma".to_string(),
        Value::BuiltinFunction(Rc::new(TaHma::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "swma".to_string(),
        Value::BuiltinFunction(Rc::new(TaSwma::<O>::builtin_fn)),
    );

    // Statistics
    ta_ns.insert(
        "stdev".to_string(),
        Value::BuiltinFunction(Rc::new(TaStdev::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "variance".to_string(),
        Value::BuiltinFunction(Rc::new(TaVariance::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "median".to_string(),
        Value::BuiltinFunction(Rc::new(TaMedian::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "dev".to_string(),
        Value::BuiltinFunction(Rc::new(TaDev::<O>::builtin_fn)),
    );

    // Volatility
    ta_ns.insert(
        "tr".to_string(),
        Value::BuiltinFunction(Rc::new(TaTr::builtin_fn::<O>)),
    );
    ta_ns.insert(
        "atr".to_string(),
        Value::BuiltinFunction(Rc::new(TaAtr::builtin_fn::<O>)),
    );
    ta_ns.insert(
        "bb".to_string(),
        Value::BuiltinFunction(Rc::new(TaBb::<O>::builtin_fn)),
    );

    // Comparison & Signals
    ta_ns.insert(
        "change".to_string(),
        Value::BuiltinFunction(Rc::new(TaChange::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "highest".to_string(),
        Value::BuiltinFunction(Rc::new(TaHighest::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "lowest".to_string(),
        Value::BuiltinFunction(Rc::new(TaLowest::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "highestbars".to_string(),
        Value::BuiltinFunction(Rc::new(TaHighestbars::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "lowestbars".to_string(),
        Value::BuiltinFunction(Rc::new(TaLowestbars::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "rising".to_string(),
        Value::BuiltinFunction(Rc::new(TaRising::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "falling".to_string(),
        Value::BuiltinFunction(Rc::new(TaFalling::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "cross".to_string(),
        Value::BuiltinFunction(Rc::new(TaCross::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "crossover".to_string(),
        Value::BuiltinFunction(Rc::new(TaCrossover::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "crossunder".to_string(),
        Value::BuiltinFunction(Rc::new(TaCrossunder::<O>::builtin_fn)),
    );

    // Oscillators & Indicators
    ta_ns.insert(
        "rsi".to_string(),
        Value::BuiltinFunction(Rc::new(TaRsi::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "cci".to_string(),
        Value::BuiltinFunction(Rc::new(TaCci::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "mom".to_string(),
        Value::BuiltinFunction(Rc::new(TaMom::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "roc".to_string(),
        Value::BuiltinFunction(Rc::new(TaRoc::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "cmo".to_string(),
        Value::BuiltinFunction(Rc::new(TaCmo::<O>::builtin_fn)),
    );
    ta_ns.insert(
        "linreg".to_string(),
        Value::BuiltinFunction(Rc::new(TaLinreg::<O>::builtin_fn)),
    );

    Value::Object {
        type_name: "ta".to_string(),
        fields: Rc::new(RefCell::new(ta_ns)),
    }
}
