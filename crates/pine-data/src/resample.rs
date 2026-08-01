//! Aggregating bars into a higher timeframe, and serving a static [`Data`] as a
//! [`DataProvider`] so `request.security` can resample the chart's own bars.

use pine_core::{Bar, Data, DataProvider, ProviderError, Timeframe};

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

/// A single [`Data`] served as a [`DataProvider`]: it answers for its own symbol
/// at its own timeframe (or resamples up to a higher one), which is how
/// `request.security(syminfo.tickerid, …)` works without a separate feed.
pub struct StaticProvider {
    data: Data,
}

impl StaticProvider {
    pub fn new(data: Data) -> Self {
        Self { data }
    }
}

impl DataProvider for StaticProvider {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError> {
        // One dataset only knows its own symbol; an empty symbol means "this".
        if !symbol.is_empty() && symbol != self.data.syminfo.tickerid {
            return Err(format!("no data for symbol {symbol:?}").into());
        }
        let tf_ms = timeframe.to_millis().ok_or_else(|| {
            format!("cannot resample to timeframe {:?}", timeframe.period())
        })?;
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
            timeframe: self.data.timeframe.clone(),
            bars: resample(&self.data.bars, tf_ms),
        })
    }
}
