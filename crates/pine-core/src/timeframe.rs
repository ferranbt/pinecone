use std::cmp::Ordering;

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

    /// The unit a Pine suffix names
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        Some(match suffix {
            "T" => TimeframeUnit::Ticks,
            "S" => TimeframeUnit::Seconds,
            "" => TimeframeUnit::Minutes,
            "D" => TimeframeUnit::Daily,
            "W" => TimeframeUnit::Weekly,
            "M" => TimeframeUnit::Monthly,
            _ => return None,
        })
    }

    /// The length of one of this unit in milliseconds
    pub fn millis(self) -> Option<i64> {
        Some(match self {
            TimeframeUnit::Seconds => 1_000,
            TimeframeUnit::Minutes => 60_000,
            TimeframeUnit::Daily => 86_400_000,
            TimeframeUnit::Weekly => 604_800_000,
            TimeframeUnit::Ticks | TimeframeUnit::Monthly => return None,
        })
    }
}

/// The chart timeframe a script runs on, exposed as `timeframe.*`.
///
/// A multiplier plus a unit, e.g. `{ 3, Daily }` → `timeframe.period == "3D"`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timeframe {
    pub multiplier: u32,
    pub unit: TimeframeUnit,
}

impl Ord for Timeframe {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl PartialOrd for Timeframe {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl serde::Serialize for Timeframe {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.period())
    }
}

impl<'de> serde::Deserialize<'de> for Timeframe {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let period = <String as serde::Deserialize>::deserialize(deserializer)?;
        period.parse().map_err(serde::de::Error::custom)
    }
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

    /// Parse a Pine period string (`"5"`, `"60"`, `"1D"`, `"30S"`, `"1W"`).
    fn from_str(period: &str) -> Result<Self, Self::Err> {
        let bad = || TimeframeError(period.to_string());
        let split = period
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(period.len());
        let (count, suffix) = period.split_at(split);
        Ok(Self {
            multiplier: count.parse().ok().filter(|&m| m > 0).ok_or_else(bad)?,
            unit: TimeframeUnit::from_suffix(suffix).ok_or_else(bad)?,
        })
    }
}

/// The fixed-length units, coarsest first: a weekly gap is also a whole number
/// of days and of minutes, and the coarsest unit that divides it is the one Pine
/// would name.
const REGULAR_UNITS: [TimeframeUnit; 4] = [
    TimeframeUnit::Weekly,
    TimeframeUnit::Daily,
    TimeframeUnit::Minutes,
    TimeframeUnit::Seconds,
];

impl Timeframe {
    fn sort_key(&self) -> (i128, u8, u32) {
        let (millis_per_unit, rank): (i128, u8) = match self.unit {
            TimeframeUnit::Ticks => (0, 0),
            TimeframeUnit::Seconds => (1_000, 1),
            TimeframeUnit::Minutes => (60_000, 2),
            TimeframeUnit::Daily => (86_400_000, 3),
            TimeframeUnit::Weekly => (604_800_000, 4),
            TimeframeUnit::Monthly => (30 * 86_400_000, 5),
        };
        (
            millis_per_unit * self.multiplier as i128,
            rank,
            self.multiplier,
        )
    }

    /// The whole timeframe expressed in minutes. `None` for sub-minute and month
    /// periods, which have no whole-minute length.
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

        REGULAR_UNITS
            .into_iter()
            .filter_map(|unit| Some((unit, unit.millis()?)))
            .find(|(_, size)| millis % size == 0)
            .map(|(unit, size)| Self {
                multiplier: (millis / size) as u32,
                unit,
            })
    }

    /// The Pine period string, e.g. `"3D"`, `"60"`, `"5S"`.
    pub fn period(&self) -> String {
        format!("{}{}", self.multiplier, self.unit.suffix())
    }

    pub fn to_millis(&self) -> Option<i64> {
        Some(self.unit.millis()? * i64::from(self.multiplier))
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
    fn parses_pine_period_notation() {
        // Round-trips through `period`: a Pine string parses to the same string.
        for period in ["30S", "5", "60", "240", "1D", "1W", "1M"] {
            assert_eq!(Timeframe::from_str(period).unwrap().period(), period);
        }
        // Minutes have no suffix, so a bare number is minutes.
        assert!(Timeframe::from_str("5").unwrap().is_minutes());
    }

    #[test]
    fn parses_millisecond_lengths() {
        assert_eq!(
            Timeframe::from_str("30S").unwrap().to_millis(),
            Some(30_000)
        );
        assert_eq!(Timeframe::from_str("5").unwrap().to_millis(), Some(300_000));
        assert_eq!(
            Timeframe::from_str("1D").unwrap().to_millis(),
            Some(86_400_000)
        );
        // Months and ticks have no fixed length.
        assert_eq!(Timeframe::from_str("1M").unwrap().to_millis(), None);
    }

    #[test]
    fn orders_by_real_duration() {
        let tf = |s: &str| Timeframe::from_str(s).unwrap();
        assert!(tf("1T") < tf("30S"));
        assert!(tf("30S") < tf("1"));
        assert!(tf("1") < tf("60"));
        assert!(tf("60") < tf("1D"));
        assert!(tf("1D") < tf("1W"));
        assert!(tf("1W") < tf("1M"));
        // Equal duration, different unit: ordered deterministically, not equal.
        assert!(tf("1440") < tf("1D"));
        assert_ne!(tf("1440"), tf("1D"));
    }

    #[test]
    fn an_unreadable_interval_is_an_error() {
        assert!(Timeframe::from_str("").is_err());
        assert!(Timeframe::from_str("hourly").is_err());
        assert!(Timeframe::from_str("1y").is_err()); // provider "1y" is not Pine
        assert!(Timeframe::from_str("5m").is_err()); // provider spelling, not Pine
        assert!(Timeframe::from_str("d1").is_err());
    }
}
