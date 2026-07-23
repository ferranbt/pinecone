//! Pine's two numeric types, and the rule that decides which one an operation
//! produces.
//!
//! Pine overloads on `int` vs `float`: `math.max(int, int)` returns an int,
//! `math.max(int, float)` a float, and `15 / 2` is `7` where `15 / 2.0` is
//! `7.5`. That rule is written down once, here, so the interpreter's operators
//! and the builtins cannot drift apart.
//!
//! A builtin opts into the rule by declaring a field as [`Num`]; one that always
//! returns a float regardless of its input (`ta.sma`, `math.avg`) declares `f64`
//! instead and never sees an int. The field type *is* the spec's return type.

use std::ops::{Add, Div, Mul, Rem, Sub};

/// A Pine number: either an `int` or a `float`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Num {
    Int(i64),
    Float(f64),
}

impl Num {
    /// The value as a float, for computations that do not care about the type.
    pub fn as_f64(self) -> f64 {
        match self {
            Num::Int(n) => n as f64,
            Num::Float(n) => n,
        }
    }

    pub fn is_int(self) -> bool {
        matches!(self, Num::Int(_))
    }

    /// Combine two numbers under Pine's rule: two ints stay an int, anything
    /// else widens to float. Either closure returns `None` where Pine has no
    /// result (a zero divisor), which callers turn into `na`.
    fn combine(
        self,
        other: Num,
        int_op: impl Fn(i64, i64) -> Option<i64>,
        float_op: impl Fn(f64, f64) -> Option<f64>,
    ) -> Option<Num> {
        match (self, other) {
            (Num::Int(a), Num::Int(b)) => int_op(a, b).map(Num::Int),
            _ => float_op(self.as_f64(), other.as_f64()).map(Num::Float),
        }
    }

    /// The larger of two numbers, keeping the type Pine's overloads specify.
    pub fn max(self, other: Num) -> Num {
        self.combine(other, |a, b| Some(a.max(b)), |a, b| Some(a.max(b)))
            .expect("max is total")
    }

    /// The smaller of two numbers, keeping the type Pine's overloads specify.
    pub fn min(self, other: Num) -> Num {
        self.combine(other, |a, b| Some(a.min(b)), |a, b| Some(a.min(b)))
            .expect("min is total")
    }

    /// The magnitude, keeping the type.
    pub fn abs(self) -> Num {
        match self {
            Num::Int(n) => Num::Int(n.abs()),
            Num::Float(n) => Num::Float(n.abs()),
        }
    }

    /// Division, or `None` for a zero divisor — Pine yields `na` rather than
    /// erroring. Two ints divide as ints, so `15 / 2` is `7`.
    pub fn checked_div(self, other: Num) -> Option<Num> {
        self.combine(
            other,
            |a, b| (b != 0).then(|| a / b),
            |a, b| (b != 0.0).then(|| a / b),
        )
    }

    /// Remainder, or `None` for a zero divisor.
    pub fn checked_rem(self, other: Num) -> Option<Num> {
        self.combine(
            other,
            |a, b| (b != 0).then(|| a % b),
            |a, b| (b != 0.0).then(|| a % b),
        )
    }
}

impl Add for Num {
    type Output = Num;
    fn add(self, other: Num) -> Num {
        self.combine(other, |a, b| Some(a + b), |a, b| Some(a + b))
            .expect("addition is total")
    }
}

impl Sub for Num {
    type Output = Num;
    fn sub(self, other: Num) -> Num {
        self.combine(other, |a, b| Some(a - b), |a, b| Some(a - b))
            .expect("subtraction is total")
    }
}

impl Mul for Num {
    type Output = Num;
    fn mul(self, other: Num) -> Num {
        self.combine(other, |a, b| Some(a * b), |a, b| Some(a * b))
            .expect("multiplication is total")
    }
}

impl Div for Num {
    type Output = Num;
    /// Prefer [`Num::checked_div`]; this yields NaN on a zero divisor.
    fn div(self, other: Num) -> Num {
        self.checked_div(other).unwrap_or(Num::Float(f64::NAN))
    }
}

impl Rem for Num {
    type Output = Num;
    /// Prefer [`Num::checked_rem`]; this yields NaN on a zero divisor.
    fn rem(self, other: Num) -> Num {
        self.checked_rem(other).unwrap_or(Num::Float(f64::NAN))
    }
}

impl From<i64> for Num {
    fn from(n: i64) -> Self {
        Num::Int(n)
    }
}

impl From<f64> for Num {
    fn from(n: f64) -> Self {
        Num::Float(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_ints_stay_an_int() {
        assert_eq!(Num::Int(3) + Num::Int(4), Num::Int(7));
        assert_eq!(Num::Int(3) * Num::Int(4), Num::Int(12));
        // The case that started this: Pine's `15 / 2` is 7, not 7.5.
        assert_eq!(Num::Int(15).checked_div(Num::Int(2)), Some(Num::Int(7)));
    }

    #[test]
    fn a_float_operand_widens_the_result() {
        assert_eq!(
            Num::Int(15).checked_div(Num::Float(2.0)),
            Some(Num::Float(7.5))
        );
        assert_eq!(
            Num::Float(15.0).checked_div(Num::Int(2)),
            Some(Num::Float(7.5))
        );
        assert_eq!(Num::Int(3) + Num::Float(0.5), Num::Float(3.5));
    }

    #[test]
    fn a_zero_divisor_has_no_result() {
        assert_eq!(Num::Int(1).checked_div(Num::Int(0)), None);
        assert_eq!(Num::Float(1.0).checked_div(Num::Float(0.0)), None);
        assert_eq!(Num::Int(1).checked_rem(Num::Int(0)), None);
    }

    #[test]
    fn max_and_min_keep_the_type() {
        assert_eq!(Num::Int(3).max(Num::Int(4)), Num::Int(4));
        assert_eq!(Num::Int(3).max(Num::Float(4.0)), Num::Float(4.0));
        assert_eq!(Num::Int(3).min(Num::Int(4)), Num::Int(3));
        assert_eq!(Num::Int(-3).abs(), Num::Int(3));
    }
}
