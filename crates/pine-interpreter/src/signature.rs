//! What a builtin accepts, so a wrong argument can be reported before the
//! script runs.
//!
//! [`ParamType`] mirrors the field type a builtin declares — `f64`, `String`,
//! `bool` and so on — not Pine's own type system. A parameter therefore rejects
//! exactly what the runtime's conversion would reject, which is what makes the
//! check safe: it can only turn a guaranteed runtime error into a compile-time
//! one, never reject a call that would have worked.

use pine_ast::Literal;

/// The kind of value a parameter accepts, taken from the builtin's field type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    /// `f64`, `i64` or `Num`.
    Number,
    String,
    Bool,
    Color,
    /// `Value<O>` — the builtin inspects the value itself, so anything goes.
    Any,
}

impl ParamType {
    /// Whether a literal argument can be passed here, mirroring the conversion
    /// the builtin will apply at runtime.
    pub fn accepts(self, literal: &Literal) -> bool {
        match self {
            ParamType::Any => true,
            // `na` stands in for any type.
            _ if matches!(literal, Literal::Na) => true,
            // The numeric conversion takes numbers and bools; a string is a
            // type error.
            ParamType::Number | ParamType::Bool => {
                !matches!(literal, Literal::String(_) | Literal::HexColor(_))
            }
            // Any scalar renders as a string.
            ParamType::String => true,
            ParamType::Color => matches!(literal, Literal::HexColor(_) | Literal::String(_)),
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            ParamType::Number => "a number",
            ParamType::String => "a string",
            ParamType::Bool => "a bool",
            ParamType::Color => "a color",
            ParamType::Any => "a value",
        }
    }
}

/// One parameter of a builtin.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: ParamType,
    /// False when the parameter has a default and may be omitted.
    pub required: bool,
    /// True for a trailing parameter that soaks up any number of arguments.
    pub variadic: bool,
}

/// The parameters a builtin accepts, in positional order.
#[derive(Debug, Clone, Default)]
pub struct BuiltinSignature {
    pub params: Vec<Param>,
}

impl BuiltinSignature {
    /// The parameter an argument at `index` binds to; a trailing variadic
    /// parameter takes every argument past it. `None` means the call passed
    /// more arguments than the builtin accepts.
    pub fn positional(&self, index: usize) -> Option<&Param> {
        match self.params.get(index) {
            Some(param) => Some(param),
            None => self.params.last().filter(|last| last.variadic),
        }
    }

    pub fn named(&self, name: &str) -> Option<&Param> {
        self.params.iter().find(|param| param.name == name)
    }

    /// The most positional arguments accepted, or `None` when variadic.
    pub fn max_positional(&self) -> Option<usize> {
        if self.params.last().is_some_and(|last| last.variadic) {
            None
        } else {
            Some(self.params.len())
        }
    }
}
