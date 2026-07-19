use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, PineOutput, RuntimeError, Value};

/// color.new(color, transp) - Applies transparency to a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.new")]
struct ColorNew<O: PineOutput> {
    color: Value<O>,
    transp: f64,
}

impl<O: PineOutput> ColorNew<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut color = self.color.as_color()?;
        color.t = self.transp.clamp(0.0, 100.0) as u8;
        Ok(Value::Color(color))
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
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let r = self.red.clamp(0.0, 255.0) as u8;
        let g = self.green.clamp(0.0, 255.0) as u8;
        let b = self.blue.clamp(0.0, 255.0) as u8;
        let t = self.transp.clamp(0.0, 100.0) as u8;
        Ok(Value::Color(pine_interpreter::Color::new(r, g, b, t)))
    }
}

/// color.r(color) - Retrieves the red component of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.r")]
struct ColorR<O: PineOutput> {
    color: Value<O>,
}

impl<O: PineOutput> ColorR<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let color = self.color.as_color()?;
        Ok(Value::Number(color.r as f64))
    }
}

/// color.g(color) - Retrieves the green component of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.g")]
struct ColorG<O: PineOutput> {
    color: Value<O>,
}

impl<O: PineOutput> ColorG<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let color = self.color.as_color()?;
        Ok(Value::Number(color.g as f64))
    }
}

/// color.b(color) - Retrieves the blue component of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.b")]
struct ColorB<O: PineOutput> {
    color: Value<O>,
}

impl<O: PineOutput> ColorB<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let color = self.color.as_color()?;
        Ok(Value::Number(color.b as f64))
    }
}

/// color.t(color) - Retrieves the transparency of a color
#[derive(BuiltinFunction)]
#[builtin(name = "color.t")]
struct ColorT<O: PineOutput> {
    color: Value<O>,
}

impl<O: PineOutput> ColorT<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let color = self.color.as_color()?;
        Ok(Value::Number(color.t as f64))
    }
}

/// color.from_gradient(value, bottom_value, top_value, bottom_color, top_color)
/// - Creates a gradient color based on value position
#[derive(BuiltinFunction)]
#[builtin(name = "color.from_gradient")]
struct ColorFromGradient<O: PineOutput> {
    value: f64,
    bottom_value: f64,
    top_value: f64,
    bottom_color: Value<O>,
    top_color: Value<O>,
}

impl<O: PineOutput> ColorFromGradient<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let c1 = self.bottom_color.as_color()?;
        let c2 = self.top_color.as_color()?;

        // Calculate the position ratio (0.0 to 1.0)
        let ratio = if (self.top_value - self.bottom_value).abs() < f64::EPSILON {
            0.5 // If range is zero, use middle
        } else {
            ((self.value - self.bottom_value) / (self.top_value - self.bottom_value))
                .clamp(0.0, 1.0)
        };

        // Interpolate each component
        let r = (c1.r as f64 + (c2.r as f64 - c1.r as f64) * ratio) as u8;
        let g = (c1.g as f64 + (c2.g as f64 - c1.g as f64) * ratio) as u8;
        let b = (c1.b as f64 + (c2.b as f64 - c1.b as f64) * ratio) as u8;
        let t = (c1.t as f64 + (c2.t as f64 - c1.t as f64) * ratio) as u8;

        Ok(Value::Color(pine_interpreter::Color::new(r, g, b, t)))
    }
}

/// Register all color namespace functions and return the namespace object
pub fn register<O: PineOutput>() -> Value<O> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut color_ns: std::collections::HashMap<String, Value<O>> =
        std::collections::HashMap::new();

    color_ns.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(ColorNew::<O>::builtin_fn)),
    );
    color_ns.insert(
        "rgb".to_string(),
        Value::BuiltinFunction(Rc::new(ColorRgb::builtin_fn::<O>)),
    );
    color_ns.insert(
        "r".to_string(),
        Value::BuiltinFunction(Rc::new(ColorR::<O>::builtin_fn)),
    );
    color_ns.insert(
        "g".to_string(),
        Value::BuiltinFunction(Rc::new(ColorG::<O>::builtin_fn)),
    );
    color_ns.insert(
        "b".to_string(),
        Value::BuiltinFunction(Rc::new(ColorB::<O>::builtin_fn)),
    );
    color_ns.insert(
        "t".to_string(),
        Value::BuiltinFunction(Rc::new(ColorT::<O>::builtin_fn)),
    );
    color_ns.insert(
        "from_gradient".to_string(),
        Value::BuiltinFunction(Rc::new(ColorFromGradient::<O>::builtin_fn)),
    );

    color_ns.insert("aqua".to_string(), Value::new_color(0, 255, 255, 0));
    color_ns.insert("black".to_string(), Value::new_color(0, 0, 0, 0));
    color_ns.insert("blue".to_string(), Value::new_color(0, 0, 255, 0));
    color_ns.insert("fuchsia".to_string(), Value::new_color(255, 0, 255, 0));
    color_ns.insert("gray".to_string(), Value::new_color(128, 128, 128, 0));
    color_ns.insert("green".to_string(), Value::new_color(0, 128, 0, 0));
    color_ns.insert("lime".to_string(), Value::new_color(0, 255, 0, 0));
    color_ns.insert("maroon".to_string(), Value::new_color(128, 0, 0, 0));
    color_ns.insert("navy".to_string(), Value::new_color(0, 0, 128, 0));
    color_ns.insert("olive".to_string(), Value::new_color(128, 128, 0, 0));
    color_ns.insert("orange".to_string(), Value::new_color(255, 165, 0, 0));
    color_ns.insert("purple".to_string(), Value::new_color(128, 0, 128, 0));
    color_ns.insert("red".to_string(), Value::new_color(255, 0, 0, 0));
    color_ns.insert("silver".to_string(), Value::new_color(192, 192, 192, 0));
    color_ns.insert("teal".to_string(), Value::new_color(0, 128, 128, 0));
    color_ns.insert("white".to_string(), Value::new_color(255, 255, 255, 0));
    color_ns.insert("yellow".to_string(), Value::new_color(255, 255, 0, 0));

    Value::Object {
        type_name: "color".to_string(),
        fields: Rc::new(RefCell::new(color_ns)),
        call: None,
    }
}
