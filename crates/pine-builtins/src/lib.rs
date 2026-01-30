use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Re-export for convenience
pub use pine_interpreter::Bar;
pub use pine_interpreter::BuiltinFn;
pub use pine_interpreter::EvaluatedArg;

// Module for math namespace
mod math;

/// Register all builtin namespaces as objects and global functions
/// Returns namespace objects to be loaded as variables (e.g., "array", "str", "ta")
/// and global builtin functions (e.g., "na")
/// Each member stores the builtin function pointer as Value::BuiltinFunction
pub fn register_namespace_objects() -> HashMap<String, Value> {
    let mut namespaces = HashMap::new();

    // Create 'array' namespace object with builtin functions
    let mut array_ns = HashMap::new();
    array_ns.insert("new_float".to_string(), Value::BuiltinFunction(Rc::new(ArrayNewFloat::builtin_fn)));
    array_ns.insert("clear".to_string(), Value::BuiltinFunction(Rc::new(ArrayClear::builtin_fn)));
    array_ns.insert("push".to_string(), Value::BuiltinFunction(Rc::new(ArrayPush::builtin_fn)));
    array_ns.insert("get".to_string(), Value::BuiltinFunction(Rc::new(ArrayGet::builtin_fn)));
    array_ns.insert("size".to_string(), Value::BuiltinFunction(Rc::new(ArraySize::builtin_fn)));

    namespaces.insert("array".to_string(), Value::Object {
        type_name: "array".to_string(),
        fields: Rc::new(RefCell::new(array_ns)),
    });

    // Create 'color' namespace object with builtin functions
    let mut color_ns = HashMap::new();
    color_ns.insert("new".to_string(), Value::BuiltinFunction(Rc::new(ColorNew::builtin_fn)));
    color_ns.insert("rgb".to_string(), Value::BuiltinFunction(Rc::new(ColorRgb::builtin_fn)));
    color_ns.insert("r".to_string(), Value::BuiltinFunction(Rc::new(ColorR::builtin_fn)));
    color_ns.insert("g".to_string(), Value::BuiltinFunction(Rc::new(ColorG::builtin_fn)));
    color_ns.insert("b".to_string(), Value::BuiltinFunction(Rc::new(ColorB::builtin_fn)));
    color_ns.insert("t".to_string(), Value::BuiltinFunction(Rc::new(ColorT::builtin_fn)));
    color_ns.insert("from_gradient".to_string(), Value::BuiltinFunction(Rc::new(ColorFromGradient::builtin_fn)));

    namespaces.insert("color".to_string(), Value::Object {
        type_name: "color".to_string(),
        fields: Rc::new(RefCell::new(color_ns)),
    });

    // Register global builtin functions
    namespaces.insert("na".to_string(), Value::BuiltinFunction(Rc::new(Na::builtin_fn) as BuiltinFn));

    namespaces
}

/// na(value) - Returns true if the value is na, false otherwise
#[derive(BuiltinFunction)]
#[builtin(name = "na")]
struct Na {
    value: Value,
}

impl Na {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(matches!(self.value, Value::Na)))
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.new_float")]
struct ArrayNewFloat {
    size: f64,
    initial_value: Value,
}

impl ArrayNewFloat {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let size = self.size as usize;
        let arr = vec![self.initial_value.clone(); size];
        Ok(Value::Array(Rc::new(RefCell::new(arr))))
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.clear")]
struct ArrayClear {
    array: Value,
}

impl ArrayClear {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let arr = self.array.as_array()?;
        arr.borrow_mut().clear();
        Ok(Value::Na)
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.push")]
struct ArrayPush {
    array: Value,
    value: Value,
}

impl ArrayPush {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let arr = self.array.as_array()?;
        arr.borrow_mut().push(self.value.clone());
        Ok(Value::Na)
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.get")]
struct ArrayGet {
    array: Value,
    index: f64,
}

impl ArrayGet {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let arr = self.array.as_array()?;
        let index = self.index as usize;
        arr.borrow()
            .get(index)
            .cloned()
            .ok_or(RuntimeError::IndexOutOfBounds(index))
    }
}

#[derive(BuiltinFunction)]
#[builtin(name = "array.size")]
struct ArraySize {
    array: Value,
}

impl ArraySize {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let arr = self.array.as_array()?;
        let size = arr.borrow().len();
        Ok(Value::Number(size as f64))
    }
}

// Color functions

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

#[cfg(test)]
mod tests {
    use super::*;
    use pine_interpreter::EvaluatedArg;

    fn create_mock_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_array_new_float() {
        let mut ctx = create_mock_interpreter();
        let args = vec![
            EvaluatedArg::Positional(Value::Number(3.0)),
            EvaluatedArg::Positional(Value::Number(5.5)),
        ];

        let result = ArrayNewFloat::builtin_fn(&mut ctx, args).unwrap();

        if let Value::Array(arr_ref) = result {
            let arr = arr_ref.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(5.5));
            assert_eq!(arr[1], Value::Number(5.5));
            assert_eq!(arr[2], Value::Number(5.5));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_array_operations() {
        let mut ctx = create_mock_interpreter();

        // Create array
        let create_args = vec![
            EvaluatedArg::Positional(Value::Number(2.0)),
            EvaluatedArg::Positional(Value::Number(10.0)),
        ];
        let array = ArrayNewFloat::builtin_fn(&mut ctx, create_args).unwrap();

        // Clear array
        let clear_args = vec![EvaluatedArg::Positional(array.clone())];
        ArrayClear::builtin_fn(&mut ctx, clear_args).unwrap();

        // Check size after clear
        let size_args = vec![EvaluatedArg::Positional(array.clone())];
        let size = ArraySize::builtin_fn(&mut ctx, size_args).unwrap();
        assert_eq!(size, Value::Number(0.0));

        // Push element
        let push_args = vec![
            EvaluatedArg::Positional(array.clone()),
            EvaluatedArg::Positional(Value::Number(42.0)),
        ];
        ArrayPush::builtin_fn(&mut ctx, push_args).unwrap();

        // Check size after push
        let size_args = vec![EvaluatedArg::Positional(array.clone())];
        let size = ArraySize::builtin_fn(&mut ctx, size_args).unwrap();
        assert_eq!(size, Value::Number(1.0));

        // Get element
        let get_args = vec![
            EvaluatedArg::Positional(array.clone()),
            EvaluatedArg::Positional(Value::Number(0.0)),
        ];
        let value = ArrayGet::builtin_fn(&mut ctx, get_args).unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn test_na() {
        let mut ctx = Interpreter::new();

        // Test with na value
        let args = vec![EvaluatedArg::Positional(Value::Na)];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Test with number
        let args = vec![EvaluatedArg::Positional(Value::Number(42.0))];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test with string
        let args = vec![EvaluatedArg::Positional(Value::String("hello".to_string()))];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test with bool
        let args = vec![EvaluatedArg::Positional(Value::Bool(true))];
        let result = Na::builtin_fn(&mut ctx, args).unwrap();
        assert_eq!(result, Value::Bool(false));
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
