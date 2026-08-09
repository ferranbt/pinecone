use pine_core::{PineOutput, PineVersion, MAX_LOOKBACK};
use pine_interpreter::{Interpreter, PerBarAdvance, Series, Value};
use std::cell::{Cell, RefCell};
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
pub fn register<O: PineOutput>(
    version: PineVersion,
) -> (HashMap<String, Value<O>>, PerBarAdvance<O>) {
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

    for &(name, seed, _) in ACCUMULATORS {
        ta_ns.insert(
            name.to_string(),
            Value::Series(Series {
                id: format!("ta.{name}"),
                current: Box::new(Value::Number(seed)),
                history: Some(Rc::new(RefCell::new(Vec::new()))),
            }),
        );
    }

    if matches!(version, PineVersion::V5 | PineVersion::V6) {
        let fields = Rc::new(RefCell::new(ta_ns));
        let advance = advance_accumulators(Rc::clone(&fields));
        let mut obj: HashMap<String, Value<O>> = HashMap::new();
        obj.insert(
            "ta".to_string(),
            Value::Object {
                type_name: "ta".to_string(),
                fields,
                call: None,
                value: None,
            },
        );
        (obj, advance)
    } else {
        (ta_ns, Rc::new(|_| {}))
    }
}

fn series_now<O: PineOutput>(ctx: &Interpreter<O>, name: &str) -> Option<f64> {
    match ctx.get_variable(name)? {
        Value::Series(s) => s.current.as_number().ok(),
        v => v.as_number().ok(),
    }
}

fn series_prev<O: PineOutput>(ctx: &Interpreter<O>, name: &str) -> Option<f64> {
    ctx.user_series_history.get(name)?.last()?.as_number().ok()
}

/// The current bar's OHLCV plus the previous bar's `close`/`volume` — everything
/// the accumulators read.
struct Bars {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    close1: Option<f64>,
    volume1: Option<f64>,
}

fn read_bars<O: PineOutput>(ctx: &Interpreter<O>) -> Option<Bars> {
    Some(Bars {
        open: series_now(ctx, "open")?,
        high: series_now(ctx, "high")?,
        low: series_now(ctx, "low")?,
        close: series_now(ctx, "close")?,
        volume: series_now(ctx, "volume")?,
        close1: series_prev(ctx, "close"),
        volume1: series_prev(ctx, "volume"),
    })
}

type Formula = fn(f64, &Bars) -> f64;

/// `(name, seed, per-bar step)`. `obv`…`wad` accumulate from 0; `nvi`/`pvi` index
/// from 1; `wvad`/`iii` are pure per-bar and ignore the previous value.
const ACCUMULATORS: &[(&str, f64, Formula)] = &[
    ("obv", 0.0, obv_next),
    ("accdist", 0.0, accdist_next),
    ("pvt", 0.0, pvt_next),
    ("wad", 0.0, wad_next),
    ("nvi", 1.0, nvi_next),
    ("pvi", 1.0, pvi_next),
    ("wvad", 0.0, wvad_next),
    ("iii", 0.0, iii_next),
];

fn obv_next(prev: f64, b: &Bars) -> f64 {
    match b.close1 {
        Some(c1) if b.close > c1 => prev + b.volume,
        Some(c1) if b.close < c1 => prev - b.volume,
        _ => prev,
    }
}

fn accdist_next(prev: f64, b: &Bars) -> f64 {
    let range = b.high - b.low;
    if range == 0.0 {
        prev
    } else {
        prev + ((b.close - b.low) - (b.high - b.close)) / range * b.volume
    }
}

fn pvt_next(prev: f64, b: &Bars) -> f64 {
    match b.close1 {
        Some(c1) if c1 != 0.0 => prev + (b.close - c1) / c1 * b.volume,
        _ => prev,
    }
}

fn wad_next(prev: f64, b: &Bars) -> f64 {
    match b.close1 {
        Some(c1) if b.close > c1 => prev + b.close - b.low.min(c1),
        Some(c1) if b.close < c1 => prev + b.close - b.high.max(c1),
        _ => prev,
    }
}

fn nvi_next(prev: f64, b: &Bars) -> f64 {
    match (b.close1, b.volume1) {
        (Some(c1), Some(v1)) if c1 != 0.0 && b.close != 0.0 && b.volume < v1 => {
            prev + (b.close - c1) / c1 * prev
        }
        _ => prev,
    }
}

fn pvi_next(prev: f64, b: &Bars) -> f64 {
    match (b.close1, b.volume1) {
        (Some(c1), Some(v1)) if c1 != 0.0 && b.close != 0.0 && b.volume > v1 => {
            prev + (b.close - c1) / c1 * prev
        }
        _ => prev,
    }
}

fn wvad_next(_prev: f64, b: &Bars) -> f64 {
    let range = b.high - b.low;
    if range == 0.0 {
        0.0
    } else {
        (b.close - b.open) / range * b.volume
    }
}

fn iii_next(_prev: f64, b: &Bars) -> f64 {
    let denom = (b.high - b.low) * b.volume;
    if denom == 0.0 {
        0.0
    } else {
        (2.0 * b.close - b.high - b.low) / denom
    }
}

fn advance_accumulators<O: PineOutput>(
    fields: Rc<RefCell<HashMap<String, Value<O>>>>,
) -> PerBarAdvance<O> {
    let advanced = Cell::new(false);
    Rc::new(move |ctx: &mut Interpreter<O>| {
        let Some(bars) = read_bars(ctx) else {
            return;
        };
        // The previous bar's value only exists to be pushed from the second bar on.
        let push = advanced.replace(true);
        for &(name, seed, formula) in ACCUMULATORS {
            let (previous, history) = match fields.borrow().get(name) {
                Some(Value::Series(s)) => ((*s.current).clone(), s.history.clone()),
                _ => (Value::Na, None),
            };
            let prev = previous
                .as_number()
                .ok()
                .filter(|n| !n.is_nan())
                .unwrap_or(seed);
            let next = formula(prev, &bars);
            if push {
                if let Some(history) = &history {
                    let mut history = history.borrow_mut();
                    history.push(previous);
                    if history.len() > MAX_LOOKBACK {
                        let excess = history.len() - MAX_LOOKBACK;
                        history.drain(..excess);
                    }
                }
            }
            if let Some(Value::Series(s)) = fields.borrow_mut().get_mut(name) {
                *s.current = Value::Number(next);
            }
        }
    })
}
