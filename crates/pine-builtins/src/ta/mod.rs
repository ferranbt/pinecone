use pine_core::PineVersion;
use pine_interpreter::{PineOutput, Value};
use std::cell::RefCell;
use std::collections::HashMap;
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
pub fn register<O: PineOutput>(version: PineVersion) -> HashMap<String, Value<O>> {
    let mut ta_ns: HashMap<String, Value<O>> = HashMap::new();

    // Moving averages
    ta_ns.insert(
        "sma".to_string(),
        Value::BuiltinFunction(TaSma::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "ema".to_string(),
        Value::BuiltinFunction(TaEma::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "rma".to_string(),
        Value::BuiltinFunction(TaRma::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "wma".to_string(),
        Value::BuiltinFunction(TaWma::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "vwma".to_string(),
        Value::BuiltinFunction(TaVwma::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "hma".to_string(),
        Value::BuiltinFunction(TaHma::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "swma".to_string(),
        Value::BuiltinFunction(TaSwma::builtin_fn::<O>()),
    );

    // Statistics
    ta_ns.insert(
        "stdev".to_string(),
        Value::BuiltinFunction(TaStdev::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "variance".to_string(),
        Value::BuiltinFunction(TaVariance::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "median".to_string(),
        Value::BuiltinFunction(TaMedian::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "dev".to_string(),
        Value::BuiltinFunction(TaDev::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "percentile_nearest_rank".to_string(),
        Value::BuiltinFunction(TaPercentileNearestRank::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "cum".to_string(),
        // Stateful: the closure owns this script's per-call-site running totals.
        Value::BuiltinFunction(TaCum::builtin_fn::<O>()),
    );

    // Volatility
    ta_ns.insert(
        "tr".to_string(),
        Value::BuiltinFunction(TaTr::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "atr".to_string(),
        Value::BuiltinFunction(TaAtr::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "bb".to_string(),
        Value::BuiltinFunction(TaBb::builtin_fn::<O>()),
    );

    // Comparison & Signals
    ta_ns.insert(
        "change".to_string(),
        Value::BuiltinFunction(TaChange::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "highest".to_string(),
        Value::BuiltinFunction(TaHighest::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "lowest".to_string(),
        Value::BuiltinFunction(TaLowest::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "highestbars".to_string(),
        Value::BuiltinFunction(TaHighestbars::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "lowestbars".to_string(),
        Value::BuiltinFunction(TaLowestbars::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "rising".to_string(),
        Value::BuiltinFunction(TaRising::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "falling".to_string(),
        Value::BuiltinFunction(TaFalling::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "cross".to_string(),
        Value::BuiltinFunction(TaCross::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "crossover".to_string(),
        Value::BuiltinFunction(TaCrossover::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "crossunder".to_string(),
        Value::BuiltinFunction(TaCrossunder::builtin_fn::<O>()),
    );

    // Oscillators & Indicators
    ta_ns.insert(
        "rsi".to_string(),
        Value::BuiltinFunction(TaRsi::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "cci".to_string(),
        Value::BuiltinFunction(TaCci::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "mom".to_string(),
        Value::BuiltinFunction(TaMom::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "roc".to_string(),
        Value::BuiltinFunction(TaRoc::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "cmo".to_string(),
        Value::BuiltinFunction(TaCmo::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "linreg".to_string(),
        Value::BuiltinFunction(TaLinreg::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "stoch".to_string(),
        Value::BuiltinFunction(TaStoch::builtin_fn::<O>()),
    );
    ta_ns.insert(
        "mfi".to_string(),
        Value::BuiltinFunction(TaMfi::builtin_fn::<O>()),
    );

    if matches!(version, PineVersion::V5 | PineVersion::V6) {
        let mut obj: HashMap<String, Value<O>> = HashMap::new();
        obj.insert(
            "ta".to_string(),
            Value::Object {
                type_name: "ta".to_string(),
                fields: Rc::new(RefCell::new(ta_ns)),
                call: None,
            },
        );
        obj
    } else {
        ta_ns
    }
}
