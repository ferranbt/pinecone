use chrono::{NaiveDate, NaiveDateTime};
use pine_builtin_macro::BuiltinFunction;
use pine_core::Bar;
use pine_interpreter::{
    Builtin, BuiltinFn, EvaluatedArg, Interpreter, PineOutput, RuntimeError, Value,
};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

/// The per-bar `time` variable: the bar's opening UNIX timestamp (milliseconds).
pub fn register_bar_time<O: PineOutput>(bar: &Bar) -> Value<O> {
    Value::Number(bar.time as f64)
}

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

/// year(time) - Returns year for given UNIX time in milliseconds
#[derive(BuiltinFunction)]
#[builtin(name = "year")]
struct Year {
    time: f64,
}

impl Year {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // Convert milliseconds to seconds
        let secs = (self.time / 1000.0) as i64;

        // For now, just use UTC (proper timezone support would require chrono)
        let year = 1970 + (secs / (365 * 24 * 60 * 60));

        Ok(Value::Number(year as f64))
    }
}

/// month(time) - Returns month (1-12) for given UNIX time
#[derive(BuiltinFunction)]
#[builtin(name = "month")]
struct Month {
    time: f64,
}

impl Month {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        // Simplified implementation - proper implementation needs date/time library
        let secs = (self.time / 1000.0) as i64;
        let days_since_epoch = secs / (24 * 60 * 60);

        // Rough approximation
        let month = ((days_since_epoch % 365) / 30) + 1;
        let month = month.min(12);

        Ok(Value::Number(month as f64))
    }
}

/// dayofmonth(time) - Returns day of month (1-31)
#[derive(BuiltinFunction)]
#[builtin(name = "dayofmonth")]
struct DayOfMonth {
    time: f64,
}

impl DayOfMonth {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let secs = (self.time / 1000.0) as i64;
        let days_since_epoch = secs / (24 * 60 * 60);

        // Simplified - assumes 30 day months
        let day = (days_since_epoch % 30) + 1;

        Ok(Value::Number(day as f64))
    }
}

/// dayofweek(time) - Returns day of week (1=Sunday, 7=Saturday)
#[derive(BuiltinFunction)]
#[builtin(name = "dayofweek")]
struct DayOfWeek {
    time: f64,
}

impl DayOfWeek {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let secs = (self.time / 1000.0) as i64;
        let days_since_epoch = secs / (24 * 60 * 60);

        // Jan 1, 1970 was a Thursday (5)
        // Sunday = 1, Monday = 2, ..., Saturday = 7
        let day = ((days_since_epoch + 4) % 7) + 1;

        Ok(Value::Number(day as f64))
    }
}

/// hour(time) - Returns hour (0-23)
#[derive(BuiltinFunction)]
#[builtin(name = "hour")]
struct Hour {
    time: f64,
}

impl Hour {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let secs = (self.time / 1000.0) as i64;
        let hour = (secs / 3600) % 24;

        Ok(Value::Number(hour as f64))
    }
}

/// minute(time) - Returns minute (0-59)
#[derive(BuiltinFunction)]
#[builtin(name = "minute")]
struct Minute {
    time: f64,
}

impl Minute {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let secs = (self.time / 1000.0) as i64;
        let minute = (secs / 60) % 60;

        Ok(Value::Number(minute as f64))
    }
}

/// second(time) - Returns second (0-59)
#[derive(BuiltinFunction)]
#[builtin(name = "second")]
struct Second {
    time: f64,
}

impl Second {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let secs = (self.time / 1000.0) as i64;
        let second = secs % 60;

        Ok(Value::Number(second as f64))
    }
}

pub fn register_time_functions<O: PineOutput>() -> Vec<(String, Value<O>)> {
    vec![
        (
            "timestamp".to_string(),
            Value::BuiltinFunction(Builtin::untyped(timestamp_fn::<O>())),
        ),
        ("year".to_string(), Year::builtin_value::<O>()),
        ("month".to_string(), Month::builtin_value::<O>()),
        ("dayofmonth".to_string(), DayOfMonth::builtin_value::<O>()),
        ("dayofweek".to_string(), DayOfWeek::builtin_value::<O>()),
        ("hour".to_string(), Hour::builtin_value::<O>()),
        ("minute".to_string(), Minute::builtin_value::<O>()),
        ("second".to_string(), Second::builtin_value::<O>()),
    ]
}
