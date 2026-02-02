use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// year(time) - Returns year for given UNIX time in milliseconds
#[derive(BuiltinFunction)]
#[builtin(name = "year")]
struct Year {
    time: f64,
}

impl Year {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let secs = (self.time / 1000.0) as i64;
        let second = secs % 60;

        Ok(Value::Number(second as f64))
    }
}

pub fn register_time_functions() -> Vec<(String, Value)> {
    use std::rc::Rc;

    vec![
        ("year".to_string(), Value::BuiltinFunction(Rc::new(Year::builtin_fn))),
        ("month".to_string(), Value::BuiltinFunction(Rc::new(Month::builtin_fn))),
        ("dayofmonth".to_string(), Value::BuiltinFunction(Rc::new(DayOfMonth::builtin_fn))),
        ("dayofweek".to_string(), Value::BuiltinFunction(Rc::new(DayOfWeek::builtin_fn))),
        ("hour".to_string(), Value::BuiltinFunction(Rc::new(Hour::builtin_fn))),
        ("minute".to_string(), Value::BuiltinFunction(Rc::new(Minute::builtin_fn))),
        ("second".to_string(), Value::BuiltinFunction(Rc::new(Second::builtin_fn))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::EvaluatedArg;

    #[test]
    fn test_hour() {
        let mut ctx = Interpreter::new();

        // Test 1 hour in milliseconds (3600000 ms)
        let result = Hour::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(3600000.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test 25 hours (should wrap to 1)
        let result = Hour::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(25.0 * 3600000.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_minute() {
        let mut ctx = Interpreter::new();

        // Test 30 minutes
        let result = Minute::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(30.0 * 60000.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(30.0));
    }

    #[test]
    fn test_second() {
        let mut ctx = Interpreter::new();

        // Test 45 seconds
        let result = Second::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(45000.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(45.0));
    }

    #[test]
    fn test_dayofweek() {
        let mut ctx = Interpreter::new();

        // Test epoch (Jan 1, 1970 was a Thursday = 5)
        let result = DayOfWeek::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(0.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test one day later (Friday = 6)
        let result = DayOfWeek::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(24.0 * 60.0 * 60.0 * 1000.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_dayofmonth() {
        let mut ctx = Interpreter::new();

        // Test epoch (should be day 1)
        let result = DayOfMonth::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(0.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test 5 days later (should be day 6)
        let result = DayOfMonth::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(5.0 * 24.0 * 60.0 * 60.0 * 1000.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_year() {
        let mut ctx = Interpreter::new();

        // Test epoch (should be 1970)
        let result = Year::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(0.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(1970.0));
    }

    #[test]
    fn test_month() {
        let mut ctx = Interpreter::new();

        // Test epoch (should be January = 1)
        let result = Month::builtin_fn(&mut ctx, vec![
            EvaluatedArg::Positional(Value::Number(0.0))
        ]).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }
}
