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
