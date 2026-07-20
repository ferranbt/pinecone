//! Constant-only namespaces: objects whose members are fixed string tags used
//! as arguments elsewhere (e.g. `plotshape(..., style=shape.circle)`).
//!
//! Each follows the same shape as `currency` — a `Value::Object` of string
//! constants. Only `size`, `shape`, and `location` live here for now; the other
//! constant families remain in their own modules.

pub mod display;
pub mod format;
pub mod location;
pub mod position;
pub mod shape;
pub mod size;
