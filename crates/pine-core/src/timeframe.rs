//! The timeframe a script's bars represent.

use std::fmt;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid timeframe `{0}`")]
pub struct TimeframeError(pub String);

/// The unit of a timeframe. Minutes have no unit letter in Pine, and hours are
/// expressed as minutes (`"60"` is one hour).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeframeUnit {
    Tick,
    Second,
    Minute,
    Day,
    Week,
    Month,
}

impl TimeframeUnit {
    /// The letter Pine writes after the quantity.
    pub fn suffix(self) -> &'static str {
        match self {
            TimeframeUnit::Tick => "T",
            TimeframeUnit::Second => "S",
            TimeframeUnit::Minute => "",
            TimeframeUnit::Day => "D",
            TimeframeUnit::Week => "W",
            TimeframeUnit::Month => "M",
        }
    }

    fn from_suffix(suffix: &str) -> Option<Self> {
        match suffix {
            "T" => Some(TimeframeUnit::Tick),
            "S" => Some(TimeframeUnit::Second),
            "" => Some(TimeframeUnit::Minute),
            "D" => Some(TimeframeUnit::Day),
            "W" => Some(TimeframeUnit::Week),
            "M" => Some(TimeframeUnit::Month),
            _ => None,
        }
    }

    /// How long one unit lasts, in seconds.
    ///
    /// Months use a 30-day approximation, as calendar months vary; ticks have
    /// no duration.
    pub fn seconds(self) -> i64 {
        match self {
            TimeframeUnit::Tick => 0,
            TimeframeUnit::Second => 1,
            TimeframeUnit::Minute => 60,
            TimeframeUnit::Day => 86_400,
            TimeframeUnit::Week => 604_800,
            TimeframeUnit::Month => 2_592_000,
        }
    }
}

/// A timeframe, e.g. `"60"` (an hour), `"D"`, `"5D"`, `"12M"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timeframe {
    pub multiplier: u32,
    pub unit: TimeframeUnit,
}

impl Default for Timeframe {
    /// One day, Pine's `"1D"`.
    fn default() -> Self {
        Self {
            multiplier: 1,
            unit: TimeframeUnit::Day,
        }
    }
}

impl Timeframe {
    pub fn new(multiplier: u32, unit: TimeframeUnit) -> Self {
        Self { multiplier, unit }
    }

    /// Parse Pine's `"<quantity><unit>"` format. A missing quantity means 1 and
    /// a missing unit means minutes, so `"D"` is one day and `"60"` is an hour.
    pub fn parse(period: &str) -> Result<Self, TimeframeError> {
        let trimmed = period.trim();
        let digits = trimmed.len() - trimmed.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        let (quantity, suffix) = trimmed.split_at(digits);

        let err = || TimeframeError(period.to_string());
        let multiplier = if quantity.is_empty() {
            1
        } else {
            quantity.parse().map_err(|_| err())?
        };
        let unit = TimeframeUnit::from_suffix(&suffix.to_ascii_uppercase()).ok_or_else(err)?;

        Ok(Self { multiplier, unit })
    }

    /// How long one bar of this timeframe lasts, in seconds.
    pub fn in_seconds(&self) -> i64 {
        self.multiplier as i64 * self.unit.seconds()
    }

    /// The largest timeframe whose bars last `seconds`, preferring the coarsest
    /// unit that divides evenly. Inverse of [`Timeframe::in_seconds`].
    pub fn from_seconds(seconds: i64) -> Self {
        for unit in [
            TimeframeUnit::Month,
            TimeframeUnit::Week,
            TimeframeUnit::Day,
            TimeframeUnit::Minute,
        ] {
            let len = unit.seconds();
            if seconds >= len && seconds % len == 0 {
                return Self::new((seconds / len) as u32, unit);
            }
        }
        Self::new(seconds.max(0) as u32, TimeframeUnit::Second)
    }

    pub fn is_intraday(&self) -> bool {
        matches!(self.unit, TimeframeUnit::Second | TimeframeUnit::Minute)
    }

    /// Daily, weekly or monthly.
    pub fn is_dwm(&self) -> bool {
        matches!(
            self.unit,
            TimeframeUnit::Day | TimeframeUnit::Week | TimeframeUnit::Month
        )
    }

    /// Which period of this timeframe `timestamp` (UNIX ms) falls in.
    ///
    /// Two timestamps in the same period share a bucket, so a change of bucket
    /// between consecutive bars marks the start of a new period.
    pub fn bucket(&self, timestamp_ms: i64) -> i64 {
        match self.in_seconds() {
            0 => timestamp_ms,
            seconds => timestamp_ms.div_euclid(seconds * 1_000),
        }
    }
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.multiplier, self.unit.suffix())
    }
}

#[cfg(test)]
mod tests {
    use super::{Timeframe, TimeframeUnit};

    #[test]
    fn parses_pine_period_strings() {
        // No unit means minutes; no quantity means 1.
        assert_eq!(
            Timeframe::parse("60").unwrap(),
            Timeframe::new(60, TimeframeUnit::Minute)
        );
        assert_eq!(
            Timeframe::parse("D").unwrap(),
            Timeframe::new(1, TimeframeUnit::Day)
        );
        assert_eq!(
            Timeframe::parse("5D").unwrap(),
            Timeframe::new(5, TimeframeUnit::Day)
        );
        assert!(Timeframe::parse("5X").is_err());
    }

    #[test]
    fn display_round_trips_through_parse() {
        for period in ["60", "1D", "5D", "12M", "30S"] {
            assert_eq!(Timeframe::parse(period).unwrap().to_string(), period);
        }
    }

    #[test]
    fn converts_to_and_from_seconds() {
        assert_eq!(Timeframe::parse("1D").unwrap().in_seconds(), 86_400);
        assert_eq!(Timeframe::parse("60").unwrap().in_seconds(), 3_600);
        assert_eq!(Timeframe::from_seconds(86_400).to_string(), "1D");
        assert_eq!(Timeframe::from_seconds(3_600).to_string(), "60");
    }

    #[test]
    fn bucket_marks_period_boundaries() {
        let daily = Timeframe::parse("1D").unwrap();
        let day = 86_400_000;
        // Same day shares a bucket; the next day does not.
        assert_eq!(daily.bucket(0), daily.bucket(day - 1));
        assert_ne!(daily.bucket(day - 1), daily.bucket(day));
    }
}
