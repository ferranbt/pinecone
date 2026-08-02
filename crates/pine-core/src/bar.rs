//! A bar of market data, and the series a script runs over.

use crate::SymInfo;

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
/// bars themselves, and the symbol they belong to.
///
/// These travel together because they come from the same place — whatever hands
/// you BTCUSD candles also knows it is BTCUSD. Splitting them would let a caller
/// describe bars as something they are not.
#[derive(Debug, Clone, Default)]
pub struct Data {
    /// Exposed to the script as `syminfo.*`.
    pub syminfo: SymInfo,
    /// Oldest first. A script is replayed over all of them.
    pub bars: Vec<Bar>,
}

impl Data {
    /// Bars for an unnamed symbol. Use [`with_syminfo`](Self::with_syminfo) to
    /// say what they actually are.
    pub fn new(bars: Vec<Bar>) -> Self {
        Self {
            syminfo: SymInfo::default(),
            bars,
        }
    }

    /// Build a series from raw rows, stamping on the positional metadata: the
    /// bar index, and the barstate flags that follow from where a bar sits.
    ///
    /// Every bar of a completed series is closed, so they are all confirmed
    /// history; only the first and last are distinguished.
    pub fn from_ohlcv(rows: impl IntoIterator<Item = Ohlcv>) -> Self {
        let rows: Vec<Ohlcv> = rows.into_iter().collect();
        let last = rows.len().saturating_sub(1);

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

        Self { ..Self::new(bars) }
    }

    pub fn with_syminfo(mut self, syminfo: SymInfo) -> Self {
        self.syminfo = syminfo;
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
