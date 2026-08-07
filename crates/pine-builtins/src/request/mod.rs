//! The `request` namespace.
//!
//! `request.security` evaluates an `expression` on another symbol/timeframe.
//! The expression is a lazy argument — captured unevaluated — and replayed in a
//! secondary interpreter over the requested feed's bars, seeded with a snapshot
//! of the chart's variables so its namespaces and builtins are already in place.
//! The resulting series is merged back non-repainting: each bar sees the last
//! *closed* requested bar.
//!
//! The feed comes from `ctx.request_provider` and the chart's bar spacing from
//! `ctx.chart_period`, set by the host — the same way `strategy.*` reaches the
//! broker through `ctx`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pine_ast::{Expr, Program, Stmt, VarKind};
use pine_builtin_macro::BuiltinFunction;
use pine_core::{Data, PineOutput, Timeframe};
use pine_interpreter::{Interpreter, RuntimeError, Series, Value};

/// One requested series, cached per call site so the secondary run happens once.
type SecondarySeries<O> = Rc<Vec<(i64, Value<O>)>>;

/// request.financial(symbol, financial_id, period, ...) - A fundamental metric
/// from the host feed; `na` when it has none.
#[derive(BuiltinFunction)]
#[builtin(name = "request.financial")]
struct RequestFinancial<O: PineOutput> {
    symbol: String,
    financial_id: String,
    period: String,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestFinancial<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.options;
        Ok(ctx
            .request_provider
            .as_ref()
            .and_then(|p| p.financial(&self.symbol, &self.financial_id, &self.period))
            .map_or(Value::Na, Value::Number))
    }
}

/// request.dividends(ticker, field, ...) - A dividend field; `na` without a feed.
#[derive(BuiltinFunction)]
#[builtin(name = "request.dividends")]
struct RequestDividends<O: PineOutput> {
    ticker: String,
    #[arg(default = "gross")]
    field: String,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestDividends<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.options;
        Ok(ctx
            .request_provider
            .as_ref()
            .and_then(|p| p.dividends(&self.ticker, &self.field))
            .map_or(Value::Na, Value::Number))
    }
}

/// request.earnings(ticker, field, ...) - An earnings field; `na` without a feed.
#[derive(BuiltinFunction)]
#[builtin(name = "request.earnings")]
struct RequestEarnings<O: PineOutput> {
    ticker: String,
    #[arg(default = "actual")]
    field: String,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestEarnings<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.options;
        Ok(ctx
            .request_provider
            .as_ref()
            .and_then(|p| p.earnings(&self.ticker, &self.field))
            .map_or(Value::Na, Value::Number))
    }
}

/// request.splits(ticker, field, ...) - A splits field; `na` without a feed.
#[derive(BuiltinFunction)]
#[builtin(name = "request.splits")]
struct RequestSplits<O: PineOutput> {
    ticker: String,
    #[arg(default = "numerator")]
    field: String,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestSplits<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.options;
        Ok(ctx
            .request_provider
            .as_ref()
            .and_then(|p| p.splits(&self.ticker, &self.field))
            .map_or(Value::Na, Value::Number))
    }
}

/// request.economic(country_code, field, ...) - An economic series; `na` without
/// a feed.
#[derive(BuiltinFunction)]
#[builtin(name = "request.economic")]
struct RequestEconomic<O: PineOutput> {
    country_code: String,
    field: String,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestEconomic<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.options;
        Ok(ctx
            .request_provider
            .as_ref()
            .and_then(|p| p.economic(&self.country_code, &self.field))
            .map_or(Value::Na, Value::Number))
    }
}

/// request.currency_rate(from, to, ...) - The `from`→`to` rate; `1.0` for a
/// same-currency pair, otherwise the host feed's rate or `na`.
#[derive(BuiltinFunction)]
#[builtin(name = "request.currency_rate")]
struct RequestCurrencyRate<O: PineOutput> {
    from: String,
    to: String,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestCurrencyRate<O> {
    fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.options;
        if self.from == self.to {
            return Ok(Value::Number(1.0));
        }
        Ok(ctx
            .request_provider
            .as_ref()
            .and_then(|p| p.currency_rate(&self.from, &self.to))
            .map_or(Value::Na, Value::Number))
    }
}

/// request.quandl(ticker, ...) - Deprecated Nasdaq Data Link feed; `na`.
#[derive(BuiltinFunction)]
#[builtin(name = "request.quandl")]
struct RequestQuandl<O: PineOutput> {
    ticker: String,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestQuandl<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = (&self.ticker, &self.options);
        Ok(Value::Na)
    }
}

/// request.seed(source, symbol, expression, ...) - A community seed feed; `na`
/// (no seed data channel), so the expression is not evaluated.
#[derive(BuiltinFunction)]
#[builtin(name = "request.seed")]
struct RequestSeed<O: PineOutput> {
    source: String,
    symbol: String,
    #[arg(lazy)]
    expression: Value<O>,
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestSeed<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = (&self.source, &self.symbol, &self.expression, &self.options);
        Ok(Value::Na)
    }
}

/// request.footprint(...) - Footprint/volume-profile rows; `na` (not modeled).
#[derive(BuiltinFunction)]
#[builtin(name = "request.footprint")]
struct RequestFootprint<O: PineOutput> {
    #[arg(variadic)]
    options: Vec<Value<O>>,
}

impl<O: PineOutput> RequestFootprint<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.options;
        Ok(Value::Na)
    }
}

pub fn register<O: PineOutput>() -> Value<O> {
    let mut fields: HashMap<String, Value<O>> = HashMap::new();
    fields.insert(
        "security".to_string(),
        RequestSecurity::<O>::builtin_value(),
    );
    fields.insert(
        "security_lower_tf".to_string(),
        RequestSecurityLowerTf::<O>::builtin_value(),
    );
    fields.insert("financial".to_string(), RequestFinancial::<O>::builtin_value());
    fields.insert("dividends".to_string(), RequestDividends::<O>::builtin_value());
    fields.insert("earnings".to_string(), RequestEarnings::<O>::builtin_value());
    fields.insert("splits".to_string(), RequestSplits::<O>::builtin_value());
    fields.insert("economic".to_string(), RequestEconomic::<O>::builtin_value());
    fields.insert("currency_rate".to_string(), RequestCurrencyRate::<O>::builtin_value());
    fields.insert("quandl".to_string(), RequestQuandl::<O>::builtin_value());
    fields.insert("seed".to_string(), RequestSeed::<O>::builtin_value());
    fields.insert("footprint".to_string(), RequestFootprint::<O>::builtin_value());
    Value::Object {
        type_name: "request".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
    }
}

/// `request.security(symbol, timeframe, expression, …)` — the last confirmed
/// value of `expression` on the requested feed, merged back non-repainting.
#[derive(BuiltinFunction)]
#[builtin(name = "request.security", stateful)]
struct RequestSecurity<O: PineOutput> {
    symbol: String,
    timeframe: String,
    #[arg(lazy)]
    expression: Value<O>,
    #[arg(default = None)]
    gaps: Option<Value<O>>,
    #[arg(default = None)]
    lookahead: Option<Value<O>>,
    #[arg(default = None)]
    ignore_invalid_symbol: Option<bool>,
    #[arg(default = None)]
    currency: Option<String>,
    #[arg(default = None)]
    calc_bars_count: Option<f64>,
    #[state]
    series: Option<SecondarySeries<O>>,
}

impl<O: PineOutput> RequestSecurity<O> {
    fn execute(&mut self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // Accepted, but their repainting/currency behaviour is not applied yet.
        let _ = (
            &self.gaps,
            &self.lookahead,
            &self.ignore_invalid_symbol,
            &self.currency,
            &self.calc_bars_count,
        );
        let (Value::Expr(expr), Ok(timeframe)) =
            (&self.expression, self.timeframe.parse::<Timeframe>())
        else {
            return Ok(Value::Na);
        };
        let expr = Rc::clone(expr);

        // Build the requested series once, then reuse it every bar.
        if self.series.is_none() {
            self.series = Some(Rc::new(request_series(ctx, &self.symbol, timeframe, &expr)));
        }
        let series = self.series.as_ref().expect("series built above");
        Ok(aligned(series, current_time(ctx)))
    }
}

/// `request.security_lower_tf(symbol, timeframe, expression, …)` — each bar, the
/// array of `expression`'s values across the intrabars of the current chart bar.
/// This engine's chart is already the finest feed, so a same-timeframe request
/// yields a one-element array; a request coarser than the chart is rejected.
#[derive(BuiltinFunction)]
#[builtin(name = "request.security_lower_tf", stateful)]
struct RequestSecurityLowerTf<O: PineOutput> {
    symbol: String,
    timeframe: String,
    #[arg(lazy)]
    expression: Value<O>,
    #[arg(default = None)]
    ignore_invalid_symbol: Option<bool>,
    #[arg(default = None)]
    currency: Option<String>,
    #[arg(default = None)]
    calc_bars_count: Option<f64>,
    #[state]
    series: Option<SecondarySeries<O>>,
}

impl<O: PineOutput> RequestSecurityLowerTf<O> {
    fn execute(&mut self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = (
            &self.ignore_invalid_symbol,
            &self.currency,
            &self.calc_bars_count,
        );
        let (Value::Expr(expr), Ok(tf)) = (&self.expression, self.timeframe.parse::<Timeframe>())
        else {
            return Ok(Value::Na);
        };
        let expr = Rc::clone(expr);

        // A "lower" timeframe must not be coarser than the chart's.
        if let (Some(tf_ms), Some(chart)) = (tf.to_millis(), ctx.chart_period) {
            if tf_ms > chart {
                return Err(RuntimeError::TypeError(format!(
                    "request.security_lower_tf: timeframe \"{}\" is not lower than the chart timeframe",
                    self.timeframe
                )));
            }
        }

        if self.series.is_none() {
            self.series = Some(Rc::new(request_series(ctx, &self.symbol, tf, &expr)));
        }
        let series = self.series.as_ref().expect("series built above");

        // The intrabars of the current chart bar: those in `[time, time + period)`.
        let now = current_time(ctx);
        let end = ctx.chart_period.map_or(i64::MAX, |period| now + period);
        let values: Vec<Value<O>> = series
            .iter()
            .filter(|(time, _)| *time >= now && *time < end)
            .map(|(_, value)| value.clone())
            .collect();
        Ok(Value::Array(Rc::new(RefCell::new(values))))
    }
}

/// Fetch `symbol` at `timeframe` from the host provider and replay `expr` over
/// its bars, or an empty series when there is no provider or the symbol is
/// unavailable.
fn request_series<O: PineOutput>(
    ctx: &Interpreter<O>,
    symbol: &str,
    timeframe: Timeframe,
    expr: &Expr,
) -> Vec<(i64, Value<O>)> {
    let data = ctx
        .request_provider
        .clone()
        .and_then(|provider| provider.request(symbol, timeframe).ok());
    data.map_or_else(Vec::new, |data| {
        secondary_series(&ctx.snapshot(), expr, data)
    })
}

/// Replay `expr` over `data`'s bars in an interpreter seeded from `base_vars`
/// (the chart's namespaces and builtins), taking its value at each bar's close.
fn secondary_series<O: PineOutput>(
    base_vars: &HashMap<String, Value<O>>,
    expr: &Expr,
    data: Data,
) -> Vec<(i64, Value<O>)> {
    let mut interp = Interpreter::<O>::new();
    for (name, value) in base_vars {
        interp.set_variable(name, value.clone());
    }

    // A one-statement program that computes the captured expression each bar.
    let program = Program::new(vec![Stmt::VarDecl {
        name: "__req".to_string(),
        type_qualifier: None,
        type_annotation: None,
        initializer: Some(expr.clone()),
        var_kind: VarKind::Plain,
        loc: Default::default(),
    }]);

    let mut series = Vec::with_capacity(data.bars.len());
    for bar in &data.bars {
        bind_bar(&mut interp, bar);
        if interp.execute(&program).is_err() {
            break;
        }
        // A bare series (`close`) yields the wrapper; take its scalar value.
        let value = match interp.get_variable("__req") {
            Some(Value::Series(series)) => (*series.current).clone(),
            Some(value) => value.clone(),
            None => Value::Na,
        };
        series.push((bar.time, value));
    }
    series
}

/// Bind the secondary run's per-bar OHLCV series and `bar_index`/`time`.
fn bind_bar<O: PineOutput>(interp: &mut Interpreter<O>, bar: &pine_core::Bar) {
    for (id, value) in [
        ("open", bar.open),
        ("high", bar.high),
        ("low", bar.low),
        ("close", bar.close),
        ("volume", bar.volume),
        ("hl2", (bar.high + bar.low) / 2.0),
        ("hlc3", (bar.high + bar.low + bar.close) / 3.0),
        ("hlcc4", (bar.high + bar.low + bar.close * 2.0) / 4.0),
        ("ohlc4", (bar.open + bar.high + bar.low + bar.close) / 4.0),
    ] {
        interp.advance_series(
            id,
            Value::Series(Series {
                id: id.to_string(),
                current: Box::new(Value::Number(value)),
            }),
        );
    }
    interp.set_variable("bar_index", Value::Number(bar.index as f64));
    for (name, value) in crate::register_per_bar(bar) {
        interp.set_variable(&name, value);
    }
}

/// Non-repainting merge: a requested bar's `time` is when it is confirmed (its
/// last constituent bar), so the value is the last one confirmed at or before
/// `now`.
fn aligned<O: PineOutput>(series: &[(i64, Value<O>)], now: i64) -> Value<O> {
    let confirmed = series.partition_point(|(time, _)| *time <= now);
    confirmed
        .checked_sub(1)
        .and_then(|i| series.get(i))
        .map_or(Value::Na, |(_, value)| value.clone())
}

fn current_time<O: PineOutput>(ctx: &Interpreter<O>) -> i64 {
    match ctx.get_variable("time") {
        Some(Value::Number(ms)) => *ms as i64,
        Some(Value::Int(ms)) => *ms,
        _ => 0,
    }
}
