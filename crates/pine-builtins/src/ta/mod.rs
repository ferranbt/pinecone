use pine_core::{PineOutput, PineVersion};
use pine_interpreter::Value;
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
    ta_ns.insert("sma".to_string(), TaSma::builtin_value::<O>());
    ta_ns.insert("ema".to_string(), TaEma::builtin_value::<O>());
    ta_ns.insert("rma".to_string(), TaRma::builtin_value::<O>());
    ta_ns.insert("wma".to_string(), TaWma::builtin_value::<O>());
    ta_ns.insert("vwma".to_string(), TaVwma::builtin_value::<O>());
    ta_ns.insert("hma".to_string(), TaHma::builtin_value::<O>());
    ta_ns.insert("swma".to_string(), TaSwma::builtin_value::<O>());

    // Statistics
    ta_ns.insert("stdev".to_string(), TaStdev::builtin_value::<O>());
    ta_ns.insert("variance".to_string(), TaVariance::builtin_value::<O>());
    ta_ns.insert("median".to_string(), TaMedian::builtin_value::<O>());
    ta_ns.insert("dev".to_string(), TaDev::builtin_value::<O>());
    ta_ns.insert(
        "percentile_nearest_rank".to_string(),
        TaPercentileNearestRank::builtin_value::<O>(),
    );
    ta_ns.insert(
        "cum".to_string(),
        // Stateful: the closure owns this script's per-call-site running totals.
        TaCum::builtin_value::<O>(),
    );

    // Volatility
    ta_ns.insert("tr".to_string(), TaTr::builtin_value::<O>());
    ta_ns.insert("atr".to_string(), TaAtr::builtin_value::<O>());
    ta_ns.insert("bb".to_string(), TaBb::builtin_value::<O>());
    ta_ns.insert("bbw".to_string(), TaBbw::builtin_value::<O>());
    ta_ns.insert("kc".to_string(), TaKc::builtin_value::<O>());
    ta_ns.insert("kcw".to_string(), TaKcw::builtin_value::<O>());
    ta_ns.insert("macd".to_string(), TaMacd::builtin_value::<O>());
    ta_ns.insert("dmi".to_string(), TaDmi::builtin_value::<O>());
    ta_ns.insert("supertrend".to_string(), TaSupertrend::builtin_value::<O>());
    ta_ns.insert("alma".to_string(), TaAlma::builtin_value::<O>());
    ta_ns.insert("sar".to_string(), TaSar::builtin_value::<O>());

    // Comparison & Signals
    ta_ns.insert("change".to_string(), TaChange::builtin_value::<O>());
    ta_ns.insert("highest".to_string(), TaHighest::builtin_value::<O>());
    ta_ns.insert("lowest".to_string(), TaLowest::builtin_value::<O>());
    ta_ns.insert(
        "highestbars".to_string(),
        TaHighestbars::builtin_value::<O>(),
    );
    ta_ns.insert("lowestbars".to_string(), TaLowestbars::builtin_value::<O>());
    ta_ns.insert("rising".to_string(), TaRising::builtin_value::<O>());
    ta_ns.insert("falling".to_string(), TaFalling::builtin_value::<O>());
    ta_ns.insert("cross".to_string(), TaCross::builtin_value::<O>());
    ta_ns.insert("crossover".to_string(), TaCrossover::builtin_value::<O>());
    ta_ns.insert("crossunder".to_string(), TaCrossunder::builtin_value::<O>());
    ta_ns.insert("barssince".to_string(), TaBarssince::builtin_value::<O>());
    ta_ns.insert("valuewhen".to_string(), TaValuewhen::builtin_value::<O>());
    ta_ns.insert("pivothigh".to_string(), TaPivothigh::builtin_value::<O>());
    ta_ns.insert("pivotlow".to_string(), TaPivotlow::builtin_value::<O>());

    // Oscillators & Indicators
    ta_ns.insert("rsi".to_string(), TaRsi::builtin_value::<O>());
    ta_ns.insert("cci".to_string(), TaCci::builtin_value::<O>());
    ta_ns.insert("mom".to_string(), TaMom::builtin_value::<O>());
    ta_ns.insert("roc".to_string(), TaRoc::builtin_value::<O>());
    ta_ns.insert("cmo".to_string(), TaCmo::builtin_value::<O>());
    ta_ns.insert("linreg".to_string(), TaLinreg::builtin_value::<O>());
    ta_ns.insert("stoch".to_string(), TaStoch::builtin_value::<O>());
    ta_ns.insert("mfi".to_string(), TaMfi::builtin_value::<O>());
    ta_ns.insert("wpr".to_string(), TaWpr::builtin_value::<O>());
    ta_ns.insert("tsi".to_string(), TaTsi::builtin_value::<O>());
    ta_ns.insert("cog".to_string(), TaCog::builtin_value::<O>());
    ta_ns.insert("rci".to_string(), TaRci::builtin_value::<O>());
    ta_ns.insert("max".to_string(), TaMax::builtin_value::<O>());
    ta_ns.insert("min".to_string(), TaMin::builtin_value::<O>());
    ta_ns.insert("range".to_string(), TaRange::builtin_value::<O>());
    ta_ns.insert("mode".to_string(), TaMode::builtin_value::<O>());
    ta_ns.insert(
        "correlation".to_string(),
        TaCorrelation::builtin_value::<O>(),
    );
    ta_ns.insert(
        "percentrank".to_string(),
        TaPercentrank::builtin_value::<O>(),
    );
    ta_ns.insert(
        "percentile_linear_interpolation".to_string(),
        TaPercentileLinearInterpolation::builtin_value::<O>(),
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
