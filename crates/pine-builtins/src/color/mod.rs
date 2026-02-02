use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// color.new(color, transp) - Applies transparency to a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.new")]
struct ColorNew {
    color: Value,
    transp: f64,
}

impl ColorNew {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let (r, g, b, _) = self.color.as_color()?;
        let transp = self.transp.clamp(0.0, 100.0) as u8;
        Ok(Value::Color { r, g, b, t: transp })
    }
}

/// color.rgb(red, green, blue, transp) - Creates a color from RGB components
#[derive(BuiltinFunction)]
#[builtin(name = "color.rgb")]
struct ColorRgb {
    red: f64,
    green: f64,
    blue: f64,
    #[arg(default = 0.0)]
    transp: f64,
}

impl ColorRgb {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let r = self.red.clamp(0.0, 255.0) as u8;
        let g = self.green.clamp(0.0, 255.0) as u8;
        let b = self.blue.clamp(0.0, 255.0) as u8;
        let t = self.transp.clamp(0.0, 100.0) as u8;
        Ok(Value::Color { r, g, b, t })
    }
}

/// color.r(color) - Retrieves the red component of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.r")]
struct ColorR {
    color: Value,
}

impl ColorR {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let (r, _, _, _) = self.color.as_color()?;
        Ok(Value::Number(r as f64))
    }
}

/// color.g(color) - Retrieves the green component of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.g")]
struct ColorG {
    color: Value,
}

impl ColorG {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let (_, g, _, _) = self.color.as_color()?;
        Ok(Value::Number(g as f64))
    }
}

/// color.b(color) - Retrieves the blue component of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.b")]
struct ColorB {
    color: Value,
}

impl ColorB {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let (_, _, b, _) = self.color.as_color()?;
        Ok(Value::Number(b as f64))
    }
}

/// color.t(color) - Retrieves the transparency of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.t")]
struct ColorT {
    color: Value,
}

impl ColorT {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let (_, _, _, t) = self.color.as_color()?;
        Ok(Value::Number(t as f64))
    }
}

/// color.from_gradient(value, bottom_value, top_value, bottom_color, top_color)
/// - Creates a gradient color based on value position
#[derive(BuiltinFunction)]
#[builtin(name = "color.from_gradient")]
struct ColorFromGradient {
    value: f64,
    bottom_value: f64,
    top_value: f64,
    bottom_color: Value,
    top_color: Value,
}

impl ColorFromGradient {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let (r1, g1, b1, t1) = self.bottom_color.as_color()?;
        let (r2, g2, b2, t2) = self.top_color.as_color()?;

        // Calculate the position ratio (0.0 to 1.0)
        let ratio = if (self.top_value - self.bottom_value).abs() < f64::EPSILON {
            0.5 // If range is zero, use middle
        } else {
            ((self.value - self.bottom_value) / (self.top_value - self.bottom_value))
                .clamp(0.0, 1.0)
        };

        // Interpolate each component
        let r = (r1 as f64 + (r2 as f64 - r1 as f64) * ratio) as u8;
        let g = (g1 as f64 + (g2 as f64 - g1 as f64) * ratio) as u8;
        let b = (b1 as f64 + (b2 as f64 - b1 as f64) * ratio) as u8;
        let t = (t1 as f64 + (t2 as f64 - t1 as f64) * ratio) as u8;

        Ok(Value::Color { r, g, b, t })
    }
}

/// Register all color namespace functions and return the namespace object
pub fn register() -> Value {
    use std::rc::Rc;
    use std::cell::RefCell;

    let mut color_ns = std::collections::HashMap::new();

    color_ns.insert("new".to_string(), Value::BuiltinFunction(Rc::new(ColorNew::builtin_fn)));
    color_ns.insert("rgb".to_string(), Value::BuiltinFunction(Rc::new(ColorRgb::builtin_fn)));
    color_ns.insert("r".to_string(), Value::BuiltinFunction(Rc::new(ColorR::builtin_fn)));
    color_ns.insert("g".to_string(), Value::BuiltinFunction(Rc::new(ColorG::builtin_fn)));
    color_ns.insert("b".to_string(), Value::BuiltinFunction(Rc::new(ColorB::builtin_fn)));
    color_ns.insert("t".to_string(), Value::BuiltinFunction(Rc::new(ColorT::builtin_fn)));
    color_ns.insert("from_gradient".to_string(), Value::BuiltinFunction(Rc::new(ColorFromGradient::builtin_fn)));

    Value::Object {
        type_name: "color".to_string(),
        fields: Rc::new(RefCell::new(color_ns)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::EvaluatedArg;

    fn create_mock_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_color_rgb() {
        let mut ctx = create_mock_interpreter();

        // Test basic RGB creation
        let args = vec![
            EvaluatedArg::Positional(Value::Number(255.0)),
            EvaluatedArg::Positional(Value::Number(128.0)),
            EvaluatedArg::Positional(Value::Number(64.0)),
        ];
        let result = ColorRgb::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Color { r: 255, g: 128, b: 64, t: 0 });

        // Test with transparency
        let args = vec![
            EvaluatedArg::Positional(Value::Number(100.0)),
            EvaluatedArg::Positional(Value::Number(200.0)),
            EvaluatedArg::Positional(Value::Number(50.0)),
            EvaluatedArg::Positional(Value::Number(50.0)),
        ];
        let result = ColorRgb::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Color { r: 100, g: 200, b: 50, t: 50 });
    }

    #[test]
    fn test_color_new() {
        let mut ctx = create_mock_interpreter();

        let base_color = Value::Color { r: 255, g: 0, b: 0, t: 0 };
        let args = vec![
            EvaluatedArg::Positional(base_color),
            EvaluatedArg::Positional(Value::Number(50.0)),
        ];

        let result = ColorNew::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Color { r: 255, g: 0, b: 0, t: 50 });
    }

    #[test]
    fn test_color_components() {
        let mut ctx = create_mock_interpreter();
        let color = Value::Color { r: 123, g: 45, b: 67, t: 89 };

        // Test r component
        let args = vec![EvaluatedArg::Positional(color.clone())];
        let result = ColorR::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(123.0));

        // Test g component
        let args = vec![EvaluatedArg::Positional(color.clone())];
        let result = ColorG::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(45.0));

        // Test b component
        let args = vec![EvaluatedArg::Positional(color.clone())];
        let result = ColorB::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(67.0));

        // Test t component
        let args = vec![EvaluatedArg::Positional(color.clone())];
        let result = ColorT::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Number(89.0));
    }

    #[test]
    fn test_color_from_gradient() {
        let mut ctx = create_mock_interpreter();

        let bottom_color = Value::Color { r: 0, g: 0, b: 0, t: 0 };
        let top_color = Value::Color { r: 100, g: 100, b: 100, t: 100 };

        // Test middle value (should be halfway)
        let args = vec![
            EvaluatedArg::Positional(Value::Number(50.0)),
            EvaluatedArg::Positional(Value::Number(0.0)),
            EvaluatedArg::Positional(Value::Number(100.0)),
            EvaluatedArg::Positional(bottom_color.clone()),
            EvaluatedArg::Positional(top_color.clone()),
        ];
        let result = ColorFromGradient::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Color { r: 50, g: 50, b: 50, t: 50 });

        // Test bottom value (should be bottom color)
        let args = vec![
            EvaluatedArg::Positional(Value::Number(0.0)),
            EvaluatedArg::Positional(Value::Number(0.0)),
            EvaluatedArg::Positional(Value::Number(100.0)),
            EvaluatedArg::Positional(bottom_color.clone()),
            EvaluatedArg::Positional(top_color.clone()),
        ];
        let result = ColorFromGradient::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Color { r: 0, g: 0, b: 0, t: 0 });

        // Test top value (should be top color)
        let args = vec![
            EvaluatedArg::Positional(Value::Number(100.0)),
            EvaluatedArg::Positional(Value::Number(0.0)),
            EvaluatedArg::Positional(Value::Number(100.0)),
            EvaluatedArg::Positional(bottom_color),
            EvaluatedArg::Positional(top_color),
        ];
        let result = ColorFromGradient::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Color { r: 100, g: 100, b: 100, t: 100 });
    }
}
