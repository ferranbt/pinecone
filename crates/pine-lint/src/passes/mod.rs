//! Built-in lint passes. One check per submodule.
//!
//! To add a check: create a module here, implement [`Visitor`](crate::Visitor)
//! and [`LintPass`](crate::LintPass) for a struct that collects
//! [`Diagnostic`](crate::Diagnostic)s, re-export it below, and register it in
//! `all_passes` in [`crate::pass`].

mod calc_on_every_tick;
mod constant_condition;
mod eq_na;
mod request_lookahead;
mod security_repaint;

pub use calc_on_every_tick::CalcOnEveryTick;
pub use constant_condition::ConstantCondition;
pub use eq_na::EqNa;
pub use request_lookahead::RequestLookahead;
pub use security_repaint::SecurityRepaint;
