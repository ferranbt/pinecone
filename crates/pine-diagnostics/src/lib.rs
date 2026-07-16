//! A single diagnostic type shared across the Pine toolchain.
//!
//! Both `pine-sema` (which reports errors) and `pine-lint` (which reports
//! warnings) emit this same [`Diagnostic`], so a consumer can collect, sort,
//! and render findings from every phase through one code path. `Severity`
//! — not the crate that produced it — decides how a finding is treated.

use std::fmt;

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The program is invalid; it should not be executed.
    Error,
    /// Legal but suspect or non-idiomatic; does not block execution.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => f.write_str("error"),
            Severity::Warning => f.write_str("warning"),
        }
    }
}

/// A single finding from any analysis phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable identifier of the rule/check that produced this, e.g. `"eq-na"`
    /// or `"undeclared-variable"`.
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    /// 1-based `(line, column)` of the offending node, when it carries a
    /// location. `None` for nodes that don't track one yet.
    pub pos: Option<(u32, u32)>,
}

impl Diagnostic {
    pub fn new(
        rule: &'static str,
        severity: Severity,
        pos: Option<(u32, u32)>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule,
            severity,
            message: message.into(),
            pos,
        }
    }

    pub fn error(rule: &'static str, pos: Option<(u32, u32)>, message: impl Into<String>) -> Self {
        Self::new(rule, Severity::Error, pos, message)
    }

    pub fn warning(rule: &'static str, pos: Option<(u32, u32)>, message: impl Into<String>) -> Self {
        Self::new(rule, Severity::Warning, pos, message)
    }

    /// 1-based line, if located.
    pub fn line(&self) -> Option<u32> {
        self.pos.map(|(line, _)| line)
    }

    /// 1-based column, if located.
    pub fn column(&self) -> Option<u32> {
        self.pos.map(|(_, col)| col)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.pos {
            Some((line, col)) => write!(
                f,
                "{sev} [{rule}] {line}:{col}: {msg}",
                sev = self.severity,
                rule = self.rule,
                msg = self.message,
            ),
            None => write!(
                f,
                "{sev} [{rule}]: {msg}",
                sev = self.severity,
                rule = self.rule,
                msg = self.message,
            ),
        }
    }
}
