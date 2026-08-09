use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};
use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{
    Builtin, BuiltinFn, BuiltinSignature, EvaluatedArg, Interpreter, RuntimeError, Value,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

/// The `timenow` variable: the current UTC time in milliseconds. Unlike `time`
/// this is wall-clock rather than bar data, so it is re-read for every bar.
pub fn register_timenow<O: PineOutput>() -> Value<O> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(since_epoch) => Value::Number(since_epoch.as_millis() as f64),
        // A clock set before 1970: report "unknown" rather than claim 1970.
        Err(_) => Value::Na,
    }
}

/// UNIX milliseconds (UTC) for the given date parts, or `None` if out of range.
fn ymd_to_millis(y: i64, mo: i64, d: i64, h: i64, mi: i64, s: i64) -> Option<i64> {
    let date = NaiveDate::from_ymd_opt(y as i32, mo as u32, d as u32)?;
    let dt = date.and_hms_opt(h as u32, mi as u32, s as u32)?;
    Some(dt.and_utc().timestamp_millis())
}

/// Parse a date string (the `timestamp("01 Jan 2019 00:00")` form) to UNIX ms.
fn parse_date_string(s: &str) -> Option<i64> {
    let s = s.trim();
    const DATETIME_FORMATS: &[&str] = &[
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d %b %Y %H:%M:%S",
        "%d %b %Y %H:%M",
    ];
    for fmt in DATETIME_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt.and_utc().timestamp_millis());
        }
    }
    const DATE_FORMATS: &[&str] = &["%Y-%m-%d", "%d %b %Y", "%d %B %Y"];
    for fmt in DATE_FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis());
        }
    }
    None
}

/// `timestamp(...)` — build a UNIX timestamp (ms) from date parts.
///
/// Handles the numeric form `timestamp(year, month, day, hour, minute, second)`,
/// the timezone form `timestamp(tz, year, month, ...)` (the timezone string is
/// ignored and the remaining numbers read as `[year, month, day, hour?, minute?,
/// second?]`), and the single date-string form `timestamp("01 Jan 2019 00:00")`.
fn timestamp_fn<O: PineOutput>() -> BuiltinFn<O> {
    Rc::new(|_ctx, call_args| {
        let values: Vec<&Value<O>> = call_args
            .args
            .iter()
            .map(|arg| match arg {
                EvaluatedArg::Positional(v) => v,
                EvaluatedArg::Named { value, .. } => value,
            })
            .collect();

        // Single date-string form.
        if let [Value::String(s)] = values.as_slice() {
            return Ok(parse_date_string(s)
                .map(|ms| Value::Number(ms as f64))
                .unwrap_or(Value::Na));
        }

        // Numeric (optionally timezone-prefixed) form: drop any non-numeric
        // argument (the timezone string) and read the remaining numbers.
        let nums: Vec<i64> = values
            .iter()
            .filter_map(|v| v.as_number().ok().map(|n| n as i64))
            .collect();
        if nums.len() < 3 {
            return Ok(Value::Na);
        }
        let get = |i: usize| nums.get(i).copied().unwrap_or(0);
        Ok(
            ymd_to_millis(get(0), get(1), get(2), get(3), get(4), get(5))
                .map(|ms| Value::Number(ms as f64))
                .unwrap_or(Value::Na),
        )
    })
}

/// The UTC datetime for a UNIX-ms timestamp, or `None` if out of range.
fn datetime_of(millis: f64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_millis(millis as i64)
}

// The date-part extractors, shared by the `x(time)` functions and the bare-value
// forms so the two can never disagree. `sunday = 1 … saturday = 7`, matching Pine.
fn year_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.year() as i64)
}
fn month_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.month() as i64)
}
fn dayofmonth_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.day() as i64)
}
fn dayofweek_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.weekday().num_days_from_sunday() as i64 + 1)
}
fn hour_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.hour() as i64)
}
fn minute_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.minute() as i64)
}
fn second_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.second() as i64)
}
fn weekofyear_of(ms: f64) -> Option<i64> {
    datetime_of(ms).map(|d| d.iso_week().week() as i64)
}

/// Defines a `name(time)` date function returning an integer part (or `na`).
macro_rules! date_fn {
    ($ident:ident, $name:literal, $extract:ident) => {
        #[derive(BuiltinFunction)]
        #[builtin(name = $name)]
        struct $ident {
            time: f64,
        }

        impl $ident {
            fn execute<O: PineOutput>(
                &self,
                _ctx: &mut Interpreter<O>,
            ) -> Result<Value<O>, RuntimeError> {
                Ok($extract(self.time).map(Value::Int).unwrap_or(Value::Na))
            }
        }
    };
}

date_fn!(Year, "year", year_of);
date_fn!(Month, "month", month_of);
date_fn!(DayOfMonth, "dayofmonth", dayofmonth_of);
date_fn!(DayOfWeek, "dayofweek", dayofweek_of);
date_fn!(Hour, "hour", hour_of);
date_fn!(Minute, "minute", minute_of);
date_fn!(Second, "second", second_of);
date_fn!(Weekofyear, "weekofyear", weekofyear_of);

/// A date name that is both a function (`year(t)`) and a bare value (the current
/// bar's year, from the interpreter's `current_time`).
fn date_dual<O: PineOutput>(
    name: &str,
    call: BuiltinFn<O>,
    signature: &'static BuiltinSignature,
    extract: fn(f64) -> Option<i64>,
) -> Value<O> {
    Value::Object {
        type_name: name.to_string(),
        fields: Rc::new(RefCell::new(HashMap::new())),
        call: Some(Builtin { call, signature }),
        value: Some(Rc::new(move |ctx: &mut Interpreter<O>| {
            Ok(ctx
                .current_time
                .and_then(|ms| extract(ms as f64))
                .map(Value::Int)
                .unwrap_or(Value::Na))
        })),
    }
}

/// Builds one `(name, date_dual)` entry from its function struct and extractor.
macro_rules! dual {
    ($name:literal, $struct:ident, $extract:ident) => {
        (
            $name.to_string(),
            date_dual(
                $name,
                Rc::new($struct::builtin_fn::<O>) as BuiltinFn<O>,
                $struct::signature(),
                $extract,
            ),
        )
    };
}

/// `time(timeframe, session, timezone)` — the bar's time. Timeframe/session
/// resolution is not modelled, so it returns the current bar's time regardless.
fn time_fn<O: PineOutput>() -> BuiltinFn<O> {
    Rc::new(|ctx: &mut Interpreter<O>, _call_args| {
        Ok(ctx
            .current_time
            .map(|ms| Value::Number(ms as f64))
            .unwrap_or(Value::Na))
    })
}

/// The `time` name: a value (the bar's opening UNIX ms) and a function
/// (`time(timeframe, ...)`) at once.
pub fn register_time<O: PineOutput>() -> Value<O> {
    Value::Object {
        type_name: "time".to_string(),
        fields: Rc::new(RefCell::new(HashMap::new())),
        call: Some(Builtin::untyped(time_fn::<O>())),
        value: Some(Rc::new(|ctx: &mut Interpreter<O>| {
            Ok(ctx
                .current_time
                .map(|ms| Value::Number(ms as f64))
                .unwrap_or(Value::Na))
        })),
    }
}

/// The bar's closing UNIX ms: the last millisecond of the bar, i.e. its open time
/// plus the chart period, minus one.
fn close_time<O: PineOutput>(ctx: &Interpreter<O>) -> Value<O> {
    match (ctx.current_time, ctx.chart_period) {
        (Some(time), Some(period)) => Value::Number((time + period - 1) as f64),
        _ => Value::Na,
    }
}

/// The `time_close` name: a value (the bar's closing UNIX ms) and a function.
pub fn register_time_close<O: PineOutput>() -> Value<O> {
    Value::Object {
        type_name: "time_close".to_string(),
        fields: Rc::new(RefCell::new(HashMap::new())),
        call: Some(Builtin::untyped(Rc::new(
            |ctx: &mut Interpreter<O>, _args| Ok(close_time(ctx)),
        ))),
        value: Some(Rc::new(|ctx: &mut Interpreter<O>| Ok(close_time(ctx)))),
    }
}

/// The `time_tradingday` value: the start of the bar's trading day (UTC midnight).
pub fn register_time_tradingday<O: PineOutput>() -> Value<O> {
    Value::Object {
        type_name: "time_tradingday".to_string(),
        fields: Rc::new(RefCell::new(HashMap::new())),
        call: None,
        value: Some(Rc::new(|ctx: &mut Interpreter<O>| {
            const DAY_MS: i64 = 86_400_000;
            Ok(ctx
                .current_time
                .map(|ms| Value::Number((ms.div_euclid(DAY_MS) * DAY_MS) as f64))
                .unwrap_or(Value::Na))
        })),
    }
}

pub fn register_time_functions<O: PineOutput>() -> Vec<(String, Value<O>)> {
    vec![
        (
            "timestamp".to_string(),
            Value::BuiltinFunction(Builtin::untyped(timestamp_fn::<O>())),
        ),
        ("time".to_string(), register_time()),
        dual!("year", Year, year_of),
        dual!("month", Month, month_of),
        dual!("dayofmonth", DayOfMonth, dayofmonth_of),
        dual!("hour", Hour, hour_of),
        dual!("minute", Minute, minute_of),
        dual!("second", Second, second_of),
        dual!("weekofyear", Weekofyear, weekofyear_of),
    ]
}

/// The `dayofweek` name: a value (the current bar's day), a function
/// (`dayofweek(time)`), and a namespace (`dayofweek.monday` … constants) at once.
pub fn register_dayofweek<O: PineOutput>() -> Value<O> {
    let mut fields: HashMap<String, Value<O>> = HashMap::new();
    for (name, number) in [
        ("sunday", 1),
        ("monday", 2),
        ("tuesday", 3),
        ("wednesday", 4),
        ("thursday", 5),
        ("friday", 6),
        ("saturday", 7),
    ] {
        fields.insert(name.to_string(), Value::Int(number));
    }
    Value::Object {
        type_name: "dayofweek".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: Some(Builtin {
            call: Rc::new(DayOfWeek::builtin_fn::<O>) as BuiltinFn<O>,
            signature: DayOfWeek::signature(),
        }),
        value: Some(Rc::new(|ctx: &mut Interpreter<O>| {
            Ok(ctx
                .current_time
                .and_then(|ms| dayofweek_of(ms as f64))
                .map(Value::Int)
                .unwrap_or(Value::Na))
        })),
    }
}
