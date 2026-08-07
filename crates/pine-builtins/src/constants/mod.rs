//! Constant-only namespaces: objects whose members are fixed string tags used
//! as arguments elsewhere (e.g. `plotshape(..., style=shape.circle)`).
//!
//! Each follows the same shape as `currency` — a `Value::Object` of string
//! constants. Only `size`, `shape`, and `location` live here for now; the other
//! constant families remain in their own modules.

pub mod adjustment;
pub mod backadjustment;
pub mod barmerge;
pub mod display;
pub mod extend;
pub mod font;
pub mod format;
pub mod location;
pub mod order;
pub mod position;
pub mod scale;
pub mod settlement_as_close;
pub mod shape;
pub mod size;
pub mod splits;
pub mod text;
pub mod xloc;
pub mod yloc;
