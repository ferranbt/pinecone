//! The Pine Script language version a script targets.

use std::fmt;
use thiserror::Error;

/// A *missing* annotation is deliberately not represented here — that is
/// `Ok(None)` from [`PineVersion::detect`], because choosing what to assume is
/// the caller's policy, not a failure to resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum VersionError {
    /// The annotation names a version this toolchain does not support.
    #[error("unsupported Pine version {0}")]
    Unsupported(u8),
}

/// The language version a script targets, declared by the `//@version=N`
/// annotation at the top of a script.
///
/// Variants are ordered oldest → newest, so version gates read naturally and
/// stay readable as versions are added:
///
/// ```
/// use pine_core::PineVersion;
///
/// let version = PineVersion::V6;
/// // "namespaced builtins (`ta.sma`) exist from v5 onwards"
/// assert!(version >= PineVersion::V5);
/// ```
///
/// [`Default`] is [`PineVersion::LATEST`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum PineVersion {
    V3,
    V4,
    V5,
    #[default]
    V6,
}

impl PineVersion {
    /// The newest version this toolchain supports.
    pub const LATEST: PineVersion = PineVersion::V6;

    /// The number as written in `//@version=N`.
    pub fn number(self) -> u8 {
        match self {
            PineVersion::V3 => 3,
            PineVersion::V4 => 4,
            PineVersion::V5 => 5,
            PineVersion::V6 => 6,
        }
    }

    /// The version for a `//@version=N` number, or `None` if unsupported.
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            3 => Some(PineVersion::V3),
            4 => Some(PineVersion::V4),
            5 => Some(PineVersion::V5),
            6 => Some(PineVersion::V6),
            _ => None,
        }
    }

    /// Resolve the version a script targets from its `//@version=N` annotation.
    ///
    /// - `Ok(Some(version))` — a supported annotation was found.
    /// - `Ok(None)` — the script has no annotation. What to assume is the
    ///   caller's policy; note that real Pine assumes v1 here.
    /// - `Err(VersionError::Unsupported)` — the annotation names a version this
    ///   toolchain cannot compile.
    pub fn detect(source: &str) -> Result<Option<Self>, VersionError> {
        let number = source.lines().find_map(|line| {
            // Tolerate the spacing variants:
            // `//@version=6`, `// @version = 6`.
            let rest = line.trim().strip_prefix("//")?;
            let rest = rest.trim_start().strip_prefix("@version")?;
            let rest = rest.trim_start().strip_prefix('=')?;
            let digits: String = rest
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse::<u8>().ok()
        });

        match number {
            None => Ok(None),
            Some(number) => Self::from_number(number)
                .map(Some)
                .ok_or(VersionError::Unsupported(number)),
        }
    }
}

impl fmt::Display for PineVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::{PineVersion, VersionError};

    #[test]
    fn detects_the_version_annotation() {
        assert_eq!(
            PineVersion::detect("//@version=5\nx = 1\n"),
            Ok(Some(PineVersion::V5))
        );
        // Spacing variants and a non-first line.
        assert_eq!(
            PineVersion::detect("// a comment\n// @version = 4\n"),
            Ok(Some(PineVersion::V4))
        );
    }

    #[test]
    fn distinguishes_missing_from_unsupported() {
        assert_eq!(PineVersion::detect("x = 1\n"), Ok(None));
        assert_eq!(
            PineVersion::detect("//@version=2\n"),
            Err(VersionError::Unsupported(2))
        );
    }

    #[test]
    fn versions_order_oldest_to_newest() {
        assert!(PineVersion::V4 < PineVersion::V5);
        assert!(PineVersion::V5 < PineVersion::V6);
        assert_eq!(PineVersion::default(), PineVersion::LATEST);
    }
}
