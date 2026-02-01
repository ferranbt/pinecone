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

/// math.min(...) - Returns minimum of all arguments
#[derive(BuiltinFunction)]
#[builtin(name = "math.min")]
struct MathMin {
    number0: f64,
    #[arg(default = f64::INFINITY)]
    number1: f64,
}

impl MathMin {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Number(self.number0.min(self.number1)))
    }
}

/// math.max(...) - Returns maximum of all arguments
#[derive(BuiltinFunction)]
#[builtin(name = "math.max")]
struct MathMax {
    number0: f64,
    #[arg(default = f64::NEG_INFINITY)]
    number1: f64,
}

impl MathMax {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Number(self.number0.max(self.number1)))
    }
}

/// math.avg(...) - Returns average of all arguments
#[derive(BuiltinFunction)]
#[builtin(name = "math.avg")]
struct MathAvg {
    number0: f64,
    #[arg(default = 0.0)]
    number1: f64,
}

impl MathAvg {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        if self.number1 == 0.0 {
            Ok(Value::Number(self.number0))
        } else {
            Ok(Value::Number((self.number0 + self.number1) / 2.0))
        }
    }
}

/// math.sum(...) - Returns sum of all arguments
#[derive(BuiltinFunction)]
#[builtin(name = "math.sum")]
struct MathSum {
    number0: f64,
    #[arg(default = 0.0)]
    number1: f64,
}

impl MathSum {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Number(self.number0 + self.number1))
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
            static SEED: Cell<u64> = Cell::new(0x123456789abcdef0);
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
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::collections::HashMap;

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

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::EvaluatedArg;
    use std::f64::consts::PI;

    fn create_mock_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_math_abs() {
        let mut ctx = create_mock_interpreter();

        let result = MathAbs::builtin_fn(
            &mut ctx,
            vec![EvaluatedArg::Positional(Value::Number(-5.5))],
        )
        .unwrap();
        assert_eq!(result, Value::Number(5.5));

        let result =
            MathAbs::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(3.0))])
                .unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_math_ceil_floor() {
        let mut ctx = create_mock_interpreter();

        let result =
            MathCeil::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(4.2))])
                .unwrap();
        assert_eq!(result, Value::Number(5.0));

        let result =
            MathFloor::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(4.8))])
                .unwrap();
        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_math_round() {
        let mut ctx = create_mock_interpreter();

        let result =
            MathRound::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(4.5))])
                .unwrap();
        assert_eq!(result, Value::Number(5.0));

        let result =
            MathRound::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(4.4))])
                .unwrap();
        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_math_sign() {
        let mut ctx = create_mock_interpreter();

        let result =
            MathSign::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(5.0))])
                .unwrap();
        assert_eq!(result, Value::Number(1.0));

        let result = MathSign::builtin_fn(
            &mut ctx,
            vec![EvaluatedArg::Positional(Value::Number(-5.0))],
        )
        .unwrap();
        assert_eq!(result, Value::Number(-1.0));

        let result =
            MathSign::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(0.0))])
                .unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_math_sqrt() {
        let mut ctx = create_mock_interpreter();

        let result = MathSqrt::builtin_fn(
            &mut ctx,
            vec![EvaluatedArg::Positional(Value::Number(16.0))],
        )
        .unwrap();
        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_math_pow() {
        let mut ctx = create_mock_interpreter();

        let result = MathPow::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(Value::Number(2.0)),
                EvaluatedArg::Positional(Value::Number(3.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_math_min_max() {
        let mut ctx = create_mock_interpreter();

        let result = MathMin::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(Value::Number(5.0)),
                EvaluatedArg::Positional(Value::Number(3.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(3.0));

        let result = MathMax::builtin_fn(
            &mut ctx,
            vec![
                EvaluatedArg::Positional(Value::Number(5.0)),
                EvaluatedArg::Positional(Value::Number(3.0)),
            ],
        )
        .unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_math_trig() {
        let mut ctx = create_mock_interpreter();

        // Test sin(0) = 0
        let result =
            MathSin::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(0.0))])
                .unwrap();
        assert_eq!(result, Value::Number(0.0));

        // Test cos(0) = 1
        let result =
            MathCos::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(0.0))])
                .unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_math_degrees_radians() {
        let mut ctx = create_mock_interpreter();

        let result = MathToRadians::builtin_fn(
            &mut ctx,
            vec![EvaluatedArg::Positional(Value::Number(180.0))],
        )
        .unwrap();
        if let Value::Number(n) = result {
            assert!((n - PI).abs() < 0.0001);
        }

        let result =
            MathToDegrees::builtin_fn(&mut ctx, vec![EvaluatedArg::Positional(Value::Number(PI))])
                .unwrap();
        if let Value::Number(n) = result {
            assert!((n - 180.0).abs() < 0.0001);
        }
    }
}
