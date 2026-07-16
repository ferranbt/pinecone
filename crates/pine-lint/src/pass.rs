//! The lint-pass abstraction and the driver that runs every registered pass.

use pine_ast::Program;

use crate::diagnostic::Diagnostic;
use crate::passes;
use crate::visitor::Visitor;

/// A single check.
///
/// A pass is a [`Visitor`] (so it can walk the tree and collect findings as it
/// goes) plus a way to hand back what it found. Most passes accumulate into a
/// `Vec<Diagnostic>` field and return it from [`finish`](LintPass::finish); a
/// pass that needs a whole-program view can instead ignore the visitor methods
/// and do its work in `finish`.
pub trait LintPass: Visitor {
    /// The rule identifier, matching the `rule` field of the diagnostics it
    /// emits (e.g. `"eq-na"`).
    fn name(&self) -> &'static str;

    /// Consume everything collected during the walk. Called once, after the
    /// driver has run this pass over the program.
    fn finish(&mut self) -> Vec<Diagnostic>;
}

/// Every built-in pass, freshly constructed. Add new checks here.
fn all_passes() -> Vec<Box<dyn LintPass>> {
    vec![
        Box::new(passes::EqNa::default()),
        Box::new(passes::ConstantCondition::default()),
    ]
}

/// Run every built-in lint pass over `program` and return all findings,
/// sorted by line for stable, readable output.
pub fn lint(program: &Program) -> Vec<Diagnostic> {
    lint_with(program, all_passes())
}

/// Run a specific set of passes. Useful for tests that want to exercise one
/// check in isolation.
pub fn lint_with(program: &Program, mut passes: Vec<Box<dyn LintPass>>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for pass in &mut passes {
        pass.visit_program(program);
        diagnostics.extend(pass.finish());
    }
    // Stable ordering: located findings by position, then unlocated, preserving
    // the pass registration order within a position.
    diagnostics.sort_by_key(|d| d.pos.unwrap_or((u32::MAX, u32::MAX)));
    diagnostics
}
