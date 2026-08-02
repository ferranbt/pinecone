//! A static [`Data`] served as a [`DataProvider`]: bars in memory or read from a
//! CSV, answering `request` for their own symbol and resampling up to a coarser
//! timeframe.

use crate::DataError;
use pine_core::{Bar, Data, DataProvider, Ohlcv, ProviderError, SymInfo, Timeframe};
use std::io::Read;
use std::path::Path;

/// A fixed dataset served as a provider.
pub struct StaticProvider {
    data: Data,
}

impl StaticProvider {
    pub fn new(data: Data) -> Self {
        Self { data }
    }

    /// Serve the bars of a `time,open,high,low,close,volume` CSV. A header naming
    /// the columns is required; `#` comment lines are ignored. The symbol
    /// defaults to a placeholder — set it with [`with_syminfo`](Self::with_syminfo).
    pub fn from_csv(path: impl AsRef<Path>) -> Result<Self, DataError> {
        Ok(Self::new(Data::from_ohlcv(read_csv(path.as_ref())?)))
    }

    /// The symbol these bars belong to, exposed to scripts as `syminfo.*`.
    pub fn with_syminfo(mut self, syminfo: SymInfo) -> Self {
        self.data.syminfo = syminfo;
        self
    }

    /// The bars and symbol this provider serves.
    pub fn data(&self) -> &Data {
        &self.data
    }
}

impl DataProvider for StaticProvider {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError> {
        // One dataset only knows its own symbol; an empty symbol means "this".
        let syminfo = &self.data.syminfo;
        if !symbol.is_empty() && symbol != syminfo.tickerid && symbol != syminfo.ticker {
            return Err(format!("no data for symbol {symbol:?}").into());
        }
        let tf_ms = timeframe
            .to_millis()
            .ok_or_else(|| format!("cannot resample to timeframe {:?}", timeframe.period()))?;
        // Native spacing from the bars themselves; can't go below it.
        let native = self
            .data
            .bars
            .windows(2)
            .next()
            .map_or(tf_ms, |w| w[1].time - w[0].time);
        if tf_ms <= native {
            return Ok(self.data.clone());
        }
        Ok(Data {
            syminfo: self.data.syminfo.clone(),
            bars: resample(&self.data.bars, tf_ms),
        })
    }
}

/// Aggregate `bars` into `tf_ms` buckets (open of the first, high/low over all,
/// close of the last, summed volume). A bucket's `time` is its **last**
/// constituent bar's time — the bar it is confirmed on — so `request.security`
/// can align it non-repainting.
pub fn resample(bars: &[Bar], tf_ms: i64) -> Vec<Bar> {
    let mut out: Vec<Bar> = Vec::new();
    let mut current = None;
    for bar in bars {
        let key = bar.time.div_euclid(tf_ms);
        if current == Some(key) {
            let bucket = out.last_mut().expect("a bucket exists once current is set");
            bucket.high = bucket.high.max(bar.high);
            bucket.low = bucket.low.min(bar.low);
            bucket.close = bar.close;
            bucket.volume += bar.volume;
            bucket.time = bar.time;
        } else {
            let mut bucket = bar.clone();
            bucket.index = out.len() as u64;
            out.push(bucket);
            current = Some(key);
        }
    }
    out
}

#[derive(serde::Deserialize)]
struct Row {
    /// Opening time as a UNIX timestamp in milliseconds.
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn read_csv(path: &Path) -> Result<Vec<Ohlcv>, DataError> {
    let file = std::fs::File::open(path).map_err(|source| DataError::Read {
        path: path.display().to_string(),
        source: source.into(),
    })?;
    read(file).map_err(|source| DataError::Read {
        path: path.display().to_string(),
        source,
    })
}

fn read(source: impl Read) -> Result<Vec<Ohlcv>, csv::Error> {
    csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .trim(csv::Trim::All)
        .from_reader(source)
        .deserialize()
        .map(|row| {
            let row: Row = row?;
            Ok(Ohlcv {
                time: row.time,
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::read;

    #[test]
    fn reads_rows_skipping_comments() {
        let rows = read(
            "time,open,high,low,close,volume\n\
             # first bar\n\
             0,100,105,95,102,1000\n\
             60000,101,106,96,103,1010\n"
                .as_bytes(),
        )
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].time, 0);
        assert_eq!(rows[0].close, 102.0);
        assert_eq!(rows[1].time, 60000);
        assert_eq!(rows[1].volume, 1010.0);
    }

    #[test]
    fn columns_are_matched_by_header_not_position() {
        let rows =
            read("volume,close,low,high,open,time\n1000,102,95,105,100,0\n".as_bytes()).unwrap();

        assert_eq!(rows[0].open, 100.0);
        assert_eq!(rows[0].close, 102.0);
        assert_eq!(rows[0].volume, 1000.0);
    }

    #[test]
    fn reports_where_a_bad_row_is() {
        let error = read(
            "time,open,high,low,close,volume\n\
             0,100,105,95,102,1000\n\
             60000,101,106,96,oops,1010\n"
                .as_bytes(),
        )
        .unwrap_err();

        assert!(error.to_string().contains("line: 3"), "{error}");
    }

    #[test]
    fn reports_a_missing_column() {
        let error = read("time,open,high,low,close\n0,100,105,95,102\n".as_bytes()).unwrap_err();

        assert!(error.to_string().contains("volume"), "{error}");
    }
}
