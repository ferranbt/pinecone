use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// str.length(string) - Returns the length of a string
#[derive(BuiltinFunction)]
#[builtin(name = "str.length")]
struct StrLength {
    string: String,
}

impl StrLength {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Number(self.string.len() as f64))
    }
}

/// str.lower(source) - Converts string to lowercase
#[derive(BuiltinFunction)]
#[builtin(name = "str.lower")]
struct StrLower {
    source: String,
}

impl StrLower {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::String(self.source.to_lowercase()))
    }
}

/// str.upper(source) - Converts string to uppercase
#[derive(BuiltinFunction)]
#[builtin(name = "str.upper")]
struct StrUpper {
    source: String,
}

impl StrUpper {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::String(self.source.to_uppercase()))
    }
}

/// str.contains(source, str) - Checks if source contains substring
#[derive(BuiltinFunction)]
#[builtin(name = "str.contains")]
struct StrContains {
    source: String,
    str: String,
}

impl StrContains {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self.source.contains(&self.str)))
    }
}

/// str.startswith(source, str) - Checks if source starts with substring
#[derive(BuiltinFunction)]
#[builtin(name = "str.startswith")]
struct StrStartsWith {
    source: String,
    str: String,
}

impl StrStartsWith {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self.source.starts_with(&self.str)))
    }
}

/// str.endswith(source, str) - Checks if source ends with substring
#[derive(BuiltinFunction)]
#[builtin(name = "str.endswith")]
struct StrEndsWith {
    source: String,
    str: String,
}

impl StrEndsWith {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self.source.ends_with(&self.str)))
    }
}

/// str.substring(source, begin_pos, end_pos) - Extracts substring
#[derive(BuiltinFunction)]
#[builtin(name = "str.substring")]
struct StrSubstring {
    source: String,
    begin_pos: f64,
    #[arg(default = -1.0)]
    end_pos: f64,
}

impl StrSubstring {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let begin = self.begin_pos as usize;
        let end = if self.end_pos < 0.0 {
            self.source.len()
        } else {
            (self.end_pos as usize).min(self.source.len())
        };

        if begin >= self.source.len() || begin >= end {
            return Ok(Value::String(String::new()));
        }

        // Handle UTF-8 correctly by using char indices
        let chars: Vec<char> = self.source.chars().collect();
        let result: String = chars[begin..end.min(chars.len())].iter().collect();
        Ok(Value::String(result))
    }
}

/// str.replace(source, target, replacement, occurrence) - Replaces Nth occurrence
#[derive(BuiltinFunction)]
#[builtin(name = "str.replace")]
struct StrReplace {
    source: String,
    target: String,
    replacement: String,
    #[arg(default = 0.0)]
    occurrence: f64,
}

impl StrReplace {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let occurrence = self.occurrence as usize;
        let mut result = self.source.clone();

        if let Some(pos) = self
            .source
            .match_indices(&self.target)
            .nth(occurrence)
            .map(|(i, _)| i)
        {
            result.replace_range(pos..pos + self.target.len(), &self.replacement);
        }

        Ok(Value::String(result))
    }
}

/// str.replace_all(source, target, replacement) - Replaces all occurrences
#[derive(BuiltinFunction)]
#[builtin(name = "str.replace_all")]
struct StrReplaceAll {
    source: String,
    target: String,
    replacement: String,
}

impl StrReplaceAll {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        Ok(Value::String(
            self.source.replace(&self.target, &self.replacement),
        ))
    }
}

/// str.split(string, separator) - Splits string into array
#[derive(BuiltinFunction)]
#[builtin(name = "str.split")]
struct StrSplit {
    string: String,
    separator: String,
}

impl StrSplit {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let parts: Vec<Value> = self
            .string
            .split(&self.separator)
            .map(|s| Value::String(s.to_string()))
            .collect();
        Ok(Value::Array(Rc::new(RefCell::new(parts))))
    }
}

/// str.tonumber(string) - Converts string to number
#[derive(BuiltinFunction)]
#[builtin(name = "str.tonumber")]
struct StrToNumber {
    string: String,
}

impl StrToNumber {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        match self.string.trim().parse::<f64>() {
            Ok(num) => Ok(Value::Number(num)),
            Err(_) => Ok(Value::Na),
        }
    }
}

/// str.tostring(value) - Converts value to string
#[derive(BuiltinFunction)]
#[builtin(name = "str.tostring")]
struct StrToString {
    value: Value,
}

impl StrToString {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let result = match &self.value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Na => "NaN".to_string(),
            Value::Color { r, g, b, t } => format!("rgba({}, {}, {}, {})", r, g, b, t),
            Value::Array(_) => "[Array]".to_string(),
            Value::Series(series) => format!("[Series:{}]", series.id),
            Value::Object { type_name, .. } => format!("[Object:{}]", type_name),
            Value::Function { .. } => "[Function]".to_string(),
            Value::BuiltinFunction(_) => "[BuiltinFunction]".to_string(),
            Value::Type { name, .. } => format!("[Type:{}]", name),
            Value::Enum {
                enum_name,
                field_name,
                ..
            } => format!("{}::{}", enum_name, field_name),
        };
        Ok(Value::String(result))
    }
}

/// str.pos(source, str) - Returns position of substring
#[derive(BuiltinFunction)]
#[builtin(name = "str.pos")]
struct StrPos {
    source: String,
    str: String,
}

impl StrPos {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        match self.source.find(&self.str) {
            Some(pos) => Ok(Value::Number(pos as f64)),
            None => Ok(Value::Number(-1.0)),
        }
    }
}

/// str.repeat(source, count) - Repeats string count times
#[derive(BuiltinFunction)]
#[builtin(name = "str.repeat")]
struct StrRepeat {
    source: String,
    count: f64,
}

impl StrRepeat {
    fn execute(&self, _ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
        let count = self.count.max(0.0) as usize;
        Ok(Value::String(self.source.repeat(count)))
    }
}

/// Register all str namespace functions and return the namespace object
pub fn register() -> Value {
    use std::cell::RefCell;

    let mut str_ns = std::collections::HashMap::new();

    str_ns.insert(
        "length".to_string(),
        Value::BuiltinFunction(Rc::new(StrLength::builtin_fn)),
    );
    str_ns.insert(
        "lower".to_string(),
        Value::BuiltinFunction(Rc::new(StrLower::builtin_fn)),
    );
    str_ns.insert(
        "upper".to_string(),
        Value::BuiltinFunction(Rc::new(StrUpper::builtin_fn)),
    );
    str_ns.insert(
        "contains".to_string(),
        Value::BuiltinFunction(Rc::new(StrContains::builtin_fn)),
    );
    str_ns.insert(
        "startswith".to_string(),
        Value::BuiltinFunction(Rc::new(StrStartsWith::builtin_fn)),
    );
    str_ns.insert(
        "endswith".to_string(),
        Value::BuiltinFunction(Rc::new(StrEndsWith::builtin_fn)),
    );
    str_ns.insert(
        "substring".to_string(),
        Value::BuiltinFunction(Rc::new(StrSubstring::builtin_fn)),
    );
    str_ns.insert(
        "replace".to_string(),
        Value::BuiltinFunction(Rc::new(StrReplace::builtin_fn)),
    );
    str_ns.insert(
        "replace_all".to_string(),
        Value::BuiltinFunction(Rc::new(StrReplaceAll::builtin_fn)),
    );
    str_ns.insert(
        "split".to_string(),
        Value::BuiltinFunction(Rc::new(StrSplit::builtin_fn)),
    );
    str_ns.insert(
        "tonumber".to_string(),
        Value::BuiltinFunction(Rc::new(StrToNumber::builtin_fn)),
    );
    str_ns.insert(
        "tostring".to_string(),
        Value::BuiltinFunction(Rc::new(StrToString::builtin_fn)),
    );
    str_ns.insert(
        "pos".to_string(),
        Value::BuiltinFunction(Rc::new(StrPos::builtin_fn)),
    );
    str_ns.insert(
        "repeat".to_string(),
        Value::BuiltinFunction(Rc::new(StrRepeat::builtin_fn)),
    );

    Value::Object {
        type_name: "str".to_string(),
        fields: Rc::new(RefCell::new(str_ns)),
    }
}
