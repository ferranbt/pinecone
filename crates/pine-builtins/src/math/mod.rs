use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// math.abs(number) - Returns absolute value
#[derive(BuiltinFunction)]
#[builtin(name = "math.abs")]
struct MathAbs {
    number: f64,
}

impl MathAbs {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Number(self.base.powf(self.exponent)))
    }
}

// Variadic functions - these need special handling

/// math.min(...) - Returns minimum of all arguments (requires at least 2)
#[derive(BuiltinFunction)]
#[builtin(name = "math.min")]
struct MathMin {
    #[arg(variadic)]
    values: Vec<Value>,
}

impl MathMin {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if self.values.len() < 2 {
            return Err(RuntimeError::TypeError(
                "math.min requires at least 2 arguments".to_string(),
            ));
        }

        let mut min = f64::INFINITY;
        for val in &self.values {
            match val {
                Value::Number(n) => {
                    min = min.min(*n);
                }
                Value::Na => continue, // Skip Na values
                _ => {
                    return Err(RuntimeError::TypeError(
                        "math.min requires number arguments".to_string(),
                    ))
                }
            }
        }

        Ok(Value::Number(min))
    }
}

/// math.max(...) - Returns maximum of all arguments (requires at least 2)
#[derive(BuiltinFunction)]
#[builtin(name = "math.max")]
struct MathMax {
    #[arg(variadic)]
    values: Vec<Value>,
}

impl MathMax {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if self.values.len() < 2 {
            return Err(RuntimeError::TypeError(
                "math.max requires at least 2 arguments".to_string(),
            ));
        }

        let mut max = f64::NEG_INFINITY;
        for val in &self.values {
            match val {
                Value::Number(n) => {
                    max = max.max(*n);
                }
                Value::Na => continue, // Skip Na values
                _ => {
                    return Err(RuntimeError::TypeError(
                        "math.max requires number arguments".to_string(),
                    ))
                }
            }
        }

        Ok(Value::Number(max))
    }
}

/// math.avg(...) - Returns average of all arguments (requires at least 1)
#[derive(BuiltinFunction)]
#[builtin(name = "math.avg")]
struct MathAvg {
    #[arg(variadic)]
    values: Vec<Value>,
}

impl MathAvg {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if self.values.is_empty() {
            return Err(RuntimeError::TypeError(
                "math.avg requires at least 1 argument".to_string(),
            ));
        }

        let mut sum = 0.0;
        let mut count = 0;

        for val in &self.values {
            match val {
                Value::Number(n) => {
                    sum += n;
                    count += 1;
                }
                Value::Na => continue, // Skip Na values
                _ => {
                    return Err(RuntimeError::TypeError(
                        "math.avg requires number arguments".to_string(),
                    ))
                }
            }
        }

        if count == 0 {
            Ok(Value::Na)
        } else {
            Ok(Value::Number(sum / count as f64))
        }
    }
}

/// math.sum(...) - Returns sum of all arguments (requires at least 1)
#[derive(BuiltinFunction)]
#[builtin(name = "math.sum")]
struct MathSum {
    #[arg(variadic)]
    values: Vec<Value>,
}

impl MathSum {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if self.values.is_empty() {
            return Err(RuntimeError::TypeError(
                "math.sum requires at least 1 argument".to_string(),
            ));
        }

        let mut sum = 0.0;

        for val in &self.values {
            match val {
                Value::Number(n) => {
                    sum += n;
                }
                Value::Na => continue, // Skip Na values
                _ => {
                    return Err(RuntimeError::TypeError(
                        "math.sum requires number arguments".to_string(),
                    ))
                }
            }
        }

        Ok(Value::Number(sum))
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
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
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
pub fn register() -> Value {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    let mut math_ns = HashMap::new();

    // Single-argument functions
    math_ns.insert(
        "abs".to_string(),
        Value::BuiltinFunction(Rc::new(MathAbs::builtin_fn)),
    );
    math_ns.insert(
        "ceil".to_string(),
        Value::BuiltinFunction(Rc::new(MathCeil::builtin_fn)),
    );
    math_ns.insert(
        "floor".to_string(),
        Value::BuiltinFunction(Rc::new(MathFloor::builtin_fn)),
    );
    math_ns.insert(
        "round".to_string(),
        Value::BuiltinFunction(Rc::new(MathRound::builtin_fn)),
    );
    math_ns.insert(
        "sign".to_string(),
        Value::BuiltinFunction(Rc::new(MathSign::builtin_fn)),
    );
    math_ns.insert(
        "sqrt".to_string(),
        Value::BuiltinFunction(Rc::new(MathSqrt::builtin_fn)),
    );
    math_ns.insert(
        "exp".to_string(),
        Value::BuiltinFunction(Rc::new(MathExp::builtin_fn)),
    );
    math_ns.insert(
        "log".to_string(),
        Value::BuiltinFunction(Rc::new(MathLog::builtin_fn)),
    );
    math_ns.insert(
        "log10".to_string(),
        Value::BuiltinFunction(Rc::new(MathLog10::builtin_fn)),
    );

    // Trigonometric functions
    math_ns.insert(
        "sin".to_string(),
        Value::BuiltinFunction(Rc::new(MathSin::builtin_fn)),
    );
    math_ns.insert(
        "cos".to_string(),
        Value::BuiltinFunction(Rc::new(MathCos::builtin_fn)),
    );
    math_ns.insert(
        "tan".to_string(),
        Value::BuiltinFunction(Rc::new(MathTan::builtin_fn)),
    );
    math_ns.insert(
        "asin".to_string(),
        Value::BuiltinFunction(Rc::new(MathAsin::builtin_fn)),
    );
    math_ns.insert(
        "acos".to_string(),
        Value::BuiltinFunction(Rc::new(MathAcos::builtin_fn)),
    );
    math_ns.insert(
        "atan".to_string(),
        Value::BuiltinFunction(Rc::new(MathAtan::builtin_fn)),
    );
    math_ns.insert(
        "toradians".to_string(),
        Value::BuiltinFunction(Rc::new(MathToRadians::builtin_fn)),
    );
    math_ns.insert(
        "todegrees".to_string(),
        Value::BuiltinFunction(Rc::new(MathToDegrees::builtin_fn)),
    );

    // Two-argument functions
    math_ns.insert(
        "pow".to_string(),
        Value::BuiltinFunction(Rc::new(MathPow::builtin_fn)),
    );

    // Variadic functions
    math_ns.insert(
        "min".to_string(),
        Value::BuiltinFunction(Rc::new(MathMin::builtin_fn)),
    );
    math_ns.insert(
        "max".to_string(),
        Value::BuiltinFunction(Rc::new(MathMax::builtin_fn)),
    );
    math_ns.insert(
        "avg".to_string(),
        Value::BuiltinFunction(Rc::new(MathAvg::builtin_fn)),
    );
    math_ns.insert(
        "sum".to_string(),
        Value::BuiltinFunction(Rc::new(MathSum::builtin_fn)),
    );

    // Special functions
    math_ns.insert(
        "random".to_string(),
        Value::BuiltinFunction(Rc::new(MathRandom::builtin_fn)),
    );

    Value::Object {
        type_name: "math".to_string(),
        fields: Rc::new(RefCell::new(math_ns)),
    }
}
