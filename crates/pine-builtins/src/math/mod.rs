use crate::ta::checked_length;
use pine_builtin_macro::BuiltinFunction;
use pine_core::PineVersion;
use pine_interpreter::{Interpreter, Num, PineOutput, RuntimeError, SeriesBuffer, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Fold the arguments with `pick`, skipping na. The spec gives math.max and
/// math.min both an int and a float overload, and [`Num`] keeps whichever type
/// the arguments imply. All-na yields na.
fn extremum<O: PineOutput>(values: &[Value<O>], pick: impl Fn(Num, Num) -> Num) -> Value<O> {
    values
        .iter()
        .filter_map(|value| value.as_num())
        .reduce(pick)
        .map_or(Value::Na, Value::from)
}

/// math.abs(number) - Returns absolute value
#[derive(BuiltinFunction)]
#[builtin(name = "math.abs")]
struct MathAbs {
    number: f64,
}

impl MathAbs {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.abs()))
    }
}

/// math.ceil(number) - Rounds up to nearest integer
#[derive(BuiltinFunction)]
#[builtin(name = "math.ceil")]
struct MathCeil {
    number: f64,
}

impl MathCeil {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.ceil()))
    }
}

/// math.floor(number) - Rounds down to nearest integer
#[derive(BuiltinFunction)]
#[builtin(name = "math.floor")]
struct MathFloor {
    number: f64,
}

impl MathFloor {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.floor()))
    }
}

/// math.round(number) - Rounds to nearest integer
#[derive(BuiltinFunction)]
#[builtin(name = "math.round")]
struct MathRound {
    number: f64,
    #[arg(default = 0.0)]
    precision: f64,
}

impl MathRound {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if self.precision == 0.0 {
            Ok(Value::Number(self.number.round()))
        } else {
            let multiplier = 10_f64.powf(self.precision);
            Ok(Value::Number(
                (self.number * multiplier).round() / multiplier,
            ))
        }
    }
}

/// math.sign(number) - Returns -1, 0, or 1
#[derive(BuiltinFunction)]
#[builtin(name = "math.sign")]
struct MathSign {
    number: f64,
}

impl MathSign {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let result = if self.number > 0.0 {
            1.0
        } else if self.number < 0.0 {
            -1.0
        } else {
            0.0
        };
        Ok(Value::Number(result))
    }
}

/// math.sqrt(number) - Returns square root
#[derive(BuiltinFunction)]
#[builtin(name = "math.sqrt")]
struct MathSqrt {
    number: f64,
}

impl MathSqrt {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.sqrt()))
    }
}

/// math.exp(number) - Returns e^number
#[derive(BuiltinFunction)]
#[builtin(name = "math.exp")]
struct MathExp {
    number: f64,
}

impl MathExp {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.exp()))
    }
}

/// math.log(number) - Returns natural logarithm
#[derive(BuiltinFunction)]
#[builtin(name = "math.log")]
struct MathLog {
    number: f64,
}

impl MathLog {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.ln()))
    }
}

/// math.log10(number) - Returns base-10 logarithm
#[derive(BuiltinFunction)]
#[builtin(name = "math.log10")]
struct MathLog10 {
    number: f64,
}

impl MathLog10 {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.log10()))
    }
}

// Trigonometric functions

/// math.sin(number) - Returns sine
#[derive(BuiltinFunction)]
#[builtin(name = "math.sin")]
struct MathSin {
    number: f64,
}

impl MathSin {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.sin()))
    }
}

/// math.cos(number) - Returns cosine
#[derive(BuiltinFunction)]
#[builtin(name = "math.cos")]
struct MathCos {
    number: f64,
}

impl MathCos {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.cos()))
    }
}

/// math.tan(number) - Returns tangent
#[derive(BuiltinFunction)]
#[builtin(name = "math.tan")]
struct MathTan {
    number: f64,
}

impl MathTan {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.tan()))
    }
}

/// math.asin(number) - Returns arcsine
#[derive(BuiltinFunction)]
#[builtin(name = "math.asin")]
struct MathAsin {
    number: f64,
}

impl MathAsin {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.asin()))
    }
}

/// math.acos(number) - Returns arccosine
#[derive(BuiltinFunction)]
#[builtin(name = "math.acos")]
struct MathAcos {
    number: f64,
}

impl MathAcos {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.acos()))
    }
}

/// math.atan(number) - Returns arctangent
#[derive(BuiltinFunction)]
#[builtin(name = "math.atan")]
struct MathAtan {
    number: f64,
}

impl MathAtan {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.atan()))
    }
}

/// math.toradians(number) - Converts degrees to radians
#[derive(BuiltinFunction)]
#[builtin(name = "math.toradians")]
struct MathToRadians {
    number: f64,
}

impl MathToRadians {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.to_radians()))
    }
}

/// math.todegrees(number) - Converts radians to degrees
#[derive(BuiltinFunction)]
#[builtin(name = "math.todegrees")]
struct MathToDegrees {
    number: f64,
}

impl MathToDegrees {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.number.to_degrees()))
    }
}

// Two-argument functions

/// math.pow(base, exponent) - Returns base^exponent
#[derive(BuiltinFunction)]
#[builtin(name = "math.pow")]
struct MathPow {
    base: f64,
    exponent: f64,
}

impl MathPow {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Number(self.base.powf(self.exponent)))
    }
}

// Variadic functions - these need special handling

/// math.min(...) - Returns minimum of all arguments (requires at least 2)
#[derive(BuiltinFunction)]
#[builtin(name = "math.min")]
struct MathMin<O: PineOutput> {
    #[arg(variadic)]
    values: Vec<Value<O>>,
}

impl<O: PineOutput> MathMin<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if self.values.len() < 2 {
            return Err(RuntimeError::TypeError(
                "math.min requires at least 2 arguments".to_string(),
            ));
        }

        Ok(extremum(&self.values, Num::min))
    }
}

/// math.max(...) - Returns maximum of all arguments (requires at least 2)
#[derive(BuiltinFunction)]
#[builtin(name = "math.max")]
struct MathMax<O: PineOutput> {
    #[arg(variadic)]
    values: Vec<Value<O>>,
}

impl<O: PineOutput> MathMax<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if self.values.len() < 2 {
            return Err(RuntimeError::TypeError(
                "math.max requires at least 2 arguments".to_string(),
            ));
        }

        Ok(extremum(&self.values, Num::max))
    }
}

/// math.avg(...) - Returns average of all arguments (requires at least 1)
#[derive(BuiltinFunction)]
#[builtin(name = "math.avg")]
struct MathAvg<O: PineOutput> {
    #[arg(variadic)]
    values: Vec<Value<O>>,
}

impl<O: PineOutput> MathAvg<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if self.values.is_empty() {
            return Err(RuntimeError::TypeError(
                "math.avg requires at least 1 argument".to_string(),
            ));
        }

        // The spec gives math.avg only a float return, so it averages as float
        // however its arguments were typed.
        let values: Vec<f64> = self
            .values
            .iter()
            .filter_map(|v| v.as_num())
            .map(Num::as_f64)
            .collect();

        if values.is_empty() {
            Ok(Value::Na)
        } else {
            Ok(Value::Number(
                values.iter().sum::<f64>() / values.len() as f64,
            ))
        }
    }
}

/// math.sum(source, length) - The sliding sum of the last `length` values.
///
/// Per the spec, na values in `source` are ignored: an na bar does not take a
/// slot, so the sum is always over `length` non-na values.
#[derive(BuiltinFunction)]
#[builtin(name = "math.sum", stateful)]
struct MathSum {
    source: f64,
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl MathSum {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = checked_length(self.length)?;

        // na arrives as NaN and is skipped rather than entering the window.
        if !self.source.is_nan() {
            self.window.push(self.source, length);
        }

        if self.window.len() < length {
            return Ok(Value::Na);
        }

        Ok(Value::Number(self.window.window(length).iter().sum()))
    }
}

/// math.random() - Returns random number between 0 and 1
#[derive(BuiltinFunction)]
#[builtin(name = "math.random")]
struct MathRandom {
    #[arg(default = 0.0)]
    _seed: f64,
}

impl MathRandom {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        use std::cell::Cell;
        thread_local! {
            static SEED: Cell<u64> = const { Cell::new(0x123456789abcdef0) };
        }

        // Simple LCG random number generator
        let random = SEED.with(|seed| {
            let current = seed.get();
            let next = current.wrapping_mul(6364136223846793005).wrapping_add(1);
            seed.set(next);
            ((next >> 32) as f64) / (u32::MAX as f64)
        });

        Ok(Value::Number(random))
    }
}

/// Register all math namespace functions and return the namespace object
pub fn register<O: PineOutput>(version: PineVersion) -> HashMap<String, Value<O>> {
    let mut math_ns: HashMap<String, Value<O>> = HashMap::new();

    // Single-argument functions
    math_ns.insert("abs".to_string(), MathAbs::builtin_value::<O>());
    math_ns.insert("ceil".to_string(), MathCeil::builtin_value::<O>());
    math_ns.insert("floor".to_string(), MathFloor::builtin_value::<O>());
    math_ns.insert("round".to_string(), MathRound::builtin_value::<O>());
    math_ns.insert("sign".to_string(), MathSign::builtin_value::<O>());
    math_ns.insert("sqrt".to_string(), MathSqrt::builtin_value::<O>());
    math_ns.insert("exp".to_string(), MathExp::builtin_value::<O>());
    math_ns.insert("log".to_string(), MathLog::builtin_value::<O>());
    math_ns.insert("log10".to_string(), MathLog10::builtin_value::<O>());

    // Trigonometric functions
    math_ns.insert("sin".to_string(), MathSin::builtin_value::<O>());
    math_ns.insert("cos".to_string(), MathCos::builtin_value::<O>());
    math_ns.insert("tan".to_string(), MathTan::builtin_value::<O>());
    math_ns.insert("asin".to_string(), MathAsin::builtin_value::<O>());
    math_ns.insert("acos".to_string(), MathAcos::builtin_value::<O>());
    math_ns.insert("atan".to_string(), MathAtan::builtin_value::<O>());
    math_ns.insert("toradians".to_string(), MathToRadians::builtin_value::<O>());
    math_ns.insert("todegrees".to_string(), MathToDegrees::builtin_value::<O>());

    // Two-argument functions
    math_ns.insert("pow".to_string(), MathPow::builtin_value::<O>());

    // Variadic functions
    math_ns.insert("min".to_string(), MathMin::<O>::builtin_value());
    math_ns.insert("max".to_string(), MathMax::<O>::builtin_value());
    math_ns.insert("avg".to_string(), MathAvg::<O>::builtin_value());
    math_ns.insert("sum".to_string(), MathSum::builtin_value::<O>());

    // Special functions
    math_ns.insert("random".to_string(), MathRandom::builtin_value::<O>());

    if matches!(version, PineVersion::V5 | PineVersion::V6) {
        let mut obj: HashMap<String, Value<O>> = HashMap::new();
        obj.insert(
            "math".to_string(),
            Value::Object {
                type_name: "math".to_string(),
                fields: Rc::new(RefCell::new(math_ns)),
                call: None,
            },
        );
        obj
    } else {
        math_ns
    }
}
