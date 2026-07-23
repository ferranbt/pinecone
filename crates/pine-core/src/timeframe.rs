/// The unit of a [`Timeframe`]. Pine writes each as a suffix on the multiplier
/// (`"3D"`, `"5S"`, `"1W"`); minutes have no suffix (`"60"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeframeUnit {
    Ticks,
    Seconds,
    Minutes,
    Daily,
    Weekly,
    Monthly,
}

impl TimeframeUnit {
    /// The suffix Pine writes for this unit (minutes have none).
    pub fn suffix(self) -> &'static str {
        match self {
            TimeframeUnit::Ticks => "T",
            TimeframeUnit::Seconds => "S",
            TimeframeUnit::Minutes => "",
            TimeframeUnit::Daily => "D",
            TimeframeUnit::Weekly => "W",
            TimeframeUnit::Monthly => "M",
        }
    }
}

/// The chart timeframe a script runs on, exposed as `timeframe.*`.
///
/// A multiplier plus a unit, e.g. `{ 3, Daily }` → `timeframe.period == "3D"`.
#[derive(Clone, Debug)]
pub struct Timeframe {
    pub multiplier: u32,
    pub unit: TimeframeUnit,
}

impl Default for Timeframe {
    fn default() -> Self {
        Self {
            multiplier: 1,
            unit: TimeframeUnit::Daily,
        }
    }
}

/// Why an interval string could not be read as a timeframe.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("unrecognised timeframe {0:?}")]
pub struct TimeframeError(pub String);

impl std::str::FromStr for Timeframe {
    type Err = TimeframeError;

    /// Read an interval as a data provider spells it: `"5m"`, `"1h"`, `"1d"`,
    /// `"1wk"`, `"1mo"`.
    ///
    /// Providers disagree on spelling — Binance writes a month `"1M"` and a
    /// minute `"1m"`, Yahoo writes `"1mo"` and `"1m"` — so a bare `M` is
    /// case-sensitive. Pine counts hours in minutes, so `"1h"` is `60`.
    fn from_str(interval: &str) -> Result<Self, Self::Err> {
        let bad = || TimeframeError(interval.to_string());
        let split = interval
            .find(|c: char| !c.is_ascii_digit())
            .ok_or_else(bad)?;
        let (count, unit) = interval.split_at(split);
        let multiplier: u32 = count.parse().map_err(|_| bad())?;

        let (unit, multiplier) = match unit {
            "s" => (TimeframeUnit::Seconds, multiplier),
            "m" | "min" => (TimeframeUnit::Minutes, multiplier),
            "h" => (
                TimeframeUnit::Minutes,
                multiplier.checked_mul(60).ok_or_else(bad)?,
            ),
            "d" => (TimeframeUnit::Daily, multiplier),
            "w" | "wk" => (TimeframeUnit::Weekly, multiplier),
            "M" | "mo" => (TimeframeUnit::Monthly, multiplier),
            _ => return Err(bad()),
        };
        Ok(Self { multiplier, unit })
    }
}

/// Milliseconds in each unit Pine can express a regular period in. Ordered
/// coarsest first, because a weekly gap is also a whole number of days and of
/// minutes — the coarsest unit that divides it is the one Pine would name.
const UNIT_MILLIS: [(TimeframeUnit, i64); 4] = [
    (TimeframeUnit::Weekly, 7 * 24 * 60 * 60 * 1000),
    (TimeframeUnit::Daily, 24 * 60 * 60 * 1000),
    (TimeframeUnit::Minutes, 60 * 1000),
    (TimeframeUnit::Seconds, 1000),
];

impl Timeframe {
    /// The whole timeframe expressed in minutes, for a provider that counts
    /// that way. `None` for sub-minute and month periods, which do not convert.
    pub fn as_minutes(&self) -> Option<u32> {
        let per_unit = match self.unit {
            TimeframeUnit::Minutes => 1,
            TimeframeUnit::Daily => 60 * 24,
            TimeframeUnit::Weekly => 60 * 24 * 7,
            TimeframeUnit::Ticks | TimeframeUnit::Seconds | TimeframeUnit::Monthly => return None,
        };
        self.multiplier.checked_mul(per_unit)
    }

    /// The period covered by a gap of `millis` between two bars, or `None` if
    /// no whole unit divides it.
    ///
    /// Months are never inferred: their length varies, so a monthly series has
    /// no single gap to recognise.
    pub fn from_millis(millis: i64) -> Option<Self> {
        if millis <= 0 {
            return None;
        }

        UNIT_MILLIS
            .iter()
            .find(|(_, size)| millis % size == 0)
            .map(|&(unit, size)| Self {
                multiplier: (millis / size) as u32,
                unit,
            })
    }

    /// The Pine period string, e.g. `"3D"`, `"60"`, `"5S"`.
    pub fn period(&self) -> String {
        format!("{}{}", self.multiplier, self.unit.suffix())
    }

    pub fn is_seconds(&self) -> bool {
        self.unit == TimeframeUnit::Seconds
    }

    pub fn is_minutes(&self) -> bool {
        self.unit == TimeframeUnit::Minutes
    }

    pub fn is_daily(&self) -> bool {
        self.unit == TimeframeUnit::Daily
    }

    pub fn is_weekly(&self) -> bool {
        self.unit == TimeframeUnit::Weekly
    }

    pub fn is_monthly(&self) -> bool {
        self.unit == TimeframeUnit::Monthly
    }

    pub fn is_ticks(&self) -> bool {
        self.unit == TimeframeUnit::Ticks
    }

    /// Intraday timeframes are seconds or minutes.
    pub fn is_intraday(&self) -> bool {
        self.is_seconds() || self.is_minutes()
    }

    /// Day/week/month timeframes.
    pub fn is_dwm(&self) -> bool {
        self.is_daily() || self.is_weekly() || self.is_monthly()
    }
}

#[cfg(test)]
mod tests {
    use super::Timeframe;
    use std::str::FromStr;

    #[test]
    fn parses_the_common_interval_spellings() {
        assert_eq!(Timeframe::from_str("30s").unwrap().period(), "30S");
        assert_eq!(Timeframe::from_str("5m").unwrap().period(), "5");
        assert_eq!(Timeframe::from_str("1h").unwrap().period(), "60");
        assert_eq!(Timeframe::from_str("4h").unwrap().period(), "240");
        assert_eq!(Timeframe::from_str("1d").unwrap().period(), "1D");
    }

    #[test]
    fn a_bare_m_is_a_month_only_in_upper_case() {
        assert_eq!(Timeframe::from_str("1M").unwrap().period(), "1M");
        assert_eq!(Timeframe::from_str("1mo").unwrap().period(), "1M");
        assert_eq!(Timeframe::from_str("1m").unwrap().period(), "1");
    }

    #[test]
    fn weeks_take_either_spelling() {
        assert_eq!(Timeframe::from_str("1w").unwrap().period(), "1W");
        assert_eq!(Timeframe::from_str("1wk").unwrap().period(), "1W");
    }

    #[test]
    fn an_unreadable_interval_is_an_error() {
        assert!(Timeframe::from_str("").is_err());
        assert!(Timeframe::from_str("hourly").is_err());
        assert!(Timeframe::from_str("1y").is_err());
        assert!(Timeframe::from_str("d1").is_err());
    }
}
