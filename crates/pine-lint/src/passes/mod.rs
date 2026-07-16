//! Built-in lint passes. One check per submodule.
//!
//! To add a check: create a module here, implement [`Visitor`](crate::Visitor)
//! and [`LintPass`](crate::LintPass) for a struct that collects
//! [`Diagnostic`](crate::Diagnostic)s, re-export it below, and register it in
//! `all_passes` in [`crate::pass`].

mod constant_condition;
mod eq_na;

pub use constant_condition::ConstantCondition;
pub use eq_na::EqNa;
