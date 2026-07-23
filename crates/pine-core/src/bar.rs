//! A bar of market data, and the series a script runs over.

use crate::{SymInfo, Timeframe};

/// The period a series of rows is sampled at, read off the gaps between them.
///
/// The *smallest* gap is the period: a real series has holes — weekends, halts,
/// missing candles — and a hole only ever makes a gap wider, never narrower.
/// Falls back to the default when the rows are too few or too irregular to say.
fn infer_timeframe(rows: &[Ohlcv]) -> Timeframe {
    rows.windows(2)
        .map(|pair| pair[1].time - pair[0].time)
        .filter(|gap| *gap > 0)
        .min()
        .and_then(Timeframe::from_millis)
        .unwrap_or_default()
}

/// Represents a single bar/candle of market data
#[derive(Debug, Clone, Default)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub index: u64,
    /// The bar's opening time as a UNIX timestamp in milliseconds, exposed to
    /// scripts as the `time` variable.
    pub time: i64,
    /// Barstate flags the host supplies, exposed to scripts as `barstate.*`.
    /// The first bar of the dataset (`barstate.isfirst`).
    pub is_first: bool,
    /// The last bar of the dataset (`barstate.islast`).
    pub is_last: bool,
    /// A new bar has just opened (`barstate.isnew`).
    pub is_new: bool,
    /// The bar is closed/confirmed (`barstate.isconfirmed`).
    pub is_confirmed: bool,
    /// A historical bar (`barstate.ishistory`).
    pub is_history: bool,
    /// A real-time bar (`barstate.isrealtime`).
    pub is_realtime: bool,
    /// The last historical bar before real-time (`barstate.islastconfirmedhistory`).
    pub is_last_confirmed_history: bool,
}

/// One row of raw market data, before it is placed in a series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ohlcv {
    /// Opening time as a UNIX timestamp in milliseconds.
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Everything a script needs to know about the market it is running on: the
/// bars themselves, and the symbol and timeframe they belong to.
///
/// These travel together because they come from the same place — whatever hands
/// you BTCUSD hourly candles also knows it is BTCUSD and that it is hourly.
/// Splitting them would let a caller describe bars as something they are not.
#[derive(Debug, Clone, Default)]
pub struct Data {
    /// Exposed to the script as `syminfo.*`.
    pub syminfo: SymInfo,
    /// Exposed to the script as `timeframe.*`.
    pub timeframe: Timeframe,
    /// Oldest first. A script is replayed over all of them.
    pub bars: Vec<Bar>,
}

impl Data {
    /// Bars for an unnamed symbol on a default timeframe. Use the builders to
    /// say what they actually are.
    pub fn new(bars: Vec<Bar>) -> Self {
        Self {
            syminfo: SymInfo::default(),
            timeframe: Timeframe::default(),
            bars,
        }
    }

    /// Build a series from raw rows, stamping on the positional metadata: the
    /// bar index, and the barstate flags that follow from where a bar sits.
    ///
    /// Every bar of a completed series is closed, so they are all confirmed
    /// history; only the first and last are distinguished. The timeframe is
    /// inferred from how far apart the bars are — override it with
    /// [`Data::with_timeframe`] when the rows do not say.
    pub fn from_ohlcv(rows: impl IntoIterator<Item = Ohlcv>) -> Self {
        let rows: Vec<Ohlcv> = rows.into_iter().collect();
        let last = rows.len().saturating_sub(1);
        let timeframe = infer_timeframe(&rows);

        let bars = rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| Bar {
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
                index: index as u64,
                time: row.time,
                is_first: index == 0,
                is_last: index == last,
                is_new: true,
                is_confirmed: true,
                is_history: true,
                is_realtime: false,
                is_last_confirmed_history: index == last,
            })
            .collect();

        Self {
            timeframe,
            ..Self::new(bars)
        }
    }

    pub fn with_syminfo(mut self, syminfo: SymInfo) -> Self {
        self.syminfo = syminfo;
        self
    }

    pub fn with_timeframe(mut self, timeframe: Timeframe) -> Self {
        self.timeframe = timeframe;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(time: i64) -> Ohlcv {
        Ohlcv {
            time,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
        }
    }

    #[test]
    fn flags_mark_the_ends_of_the_series() {
        let data = Data::from_ohlcv([row(0), row(1), row(2)]);

        assert_eq!(data.bars.len(), 3);
        assert!(data.bars[0].is_first && !data.bars[0].is_last);
        assert!(!data.bars[1].is_first && !data.bars[1].is_last);
        assert!(!data.bars[2].is_first && data.bars[2].is_last);
        assert_eq!(data.bars[2].index, 2);
        assert!(data
            .bars
            .iter()
            .all(|bar| bar.is_history && bar.is_confirmed));
    }

    #[test]
    fn a_single_bar_is_both_ends() {
        let data = Data::from_ohlcv([row(0)]);
        assert!(data.bars[0].is_first && data.bars[0].is_last);
    }

    #[test]
    fn an_empty_series_has_no_bars() {
        assert!(Data::from_ohlcv([]).bars.is_empty());
    }
}

#[cfg(test)]
mod timeframe_tests {
    use super::*;

    fn spaced(times: &[i64]) -> Data {
        Data::from_ohlcv(times.iter().map(|&time| Ohlcv {
            time,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        }))
    }

    const MINUTE: i64 = 60_000;
    const DAY: i64 = 24 * 60 * MINUTE;

    #[test]
    fn reads_the_period_off_the_gaps() {
        assert_eq!(spaced(&[0, MINUTE, 2 * MINUTE]).timeframe.period(), "1");
        assert_eq!(spaced(&[0, 60 * MINUTE]).timeframe.period(), "60");
        assert_eq!(spaced(&[0, DAY, 2 * DAY]).timeframe.period(), "1D");
        assert_eq!(spaced(&[0, 7 * DAY]).timeframe.period(), "1W");
        assert_eq!(spaced(&[0, 30_000]).timeframe.period(), "30S");
    }

    #[test]
    fn a_gap_in_the_series_does_not_widen_the_period() {
        // Friday, Monday, Tuesday: the weekend hole is 3 days wide, but the
        // series is still daily.
        let bars = spaced(&[0, 3 * DAY, 4 * DAY]);
        assert_eq!(bars.timeframe.period(), "1D");
    }

    #[test]
    fn too_few_bars_to_tell_falls_back_to_the_default() {
        assert_eq!(spaced(&[0]).timeframe.unit, Timeframe::default().unit);
        assert_eq!(spaced(&[]).timeframe.unit, Timeframe::default().unit);
    }

    #[test]
    fn an_indivisible_gap_falls_back_to_the_default() {
        // 1500ms is not a whole number of seconds' worth of any unit above it,
        // but it is 1500 milliseconds — no unit divides it, so we do not guess.
        assert_eq!(spaced(&[0, 1501]).timeframe.unit, Timeframe::default().unit);
    }
}
