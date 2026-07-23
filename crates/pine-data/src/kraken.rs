//! Bars from Kraken's public OHLC endpoint.

use crate::{fetch, quoted, DataError, DataSource};
use pine_core::{Data, Ohlcv, SymInfo};

/// Candles for a Kraken pair.
///
/// ```no_run
/// # use pine_data::{DataSource, KrakenSource};
/// let data = KrakenSource::new("XBTUSD", "1h").load()?;
/// # Ok::<(), pine_data::DataError>(())
/// ```
#[derive(Debug, Clone)]
pub struct KrakenSource {
    pair: String,
    minutes: u32,
}

impl KrakenSource {
    /// `pair` is a Kraken pair such as `"XBTUSD"`. `interval` is the same
    /// spelling the other sources take (`"1m"`, `"1h"`, `"1d"`); Kraken counts
    /// in minutes, so it is converted. An unrecognised interval falls back to
    /// one hour.
    pub fn new(pair: &str, interval: &str) -> Self {
        Self {
            pair: pair.to_uppercase(),
            minutes: minutes_of(interval).unwrap_or(60),
        }
    }
}

/// Kraken takes the interval as a number of minutes, and accepts only these.
fn minutes_of(interval: &str) -> Option<u32> {
    let (count, unit) = interval.split_at(interval.len().checked_sub(1)?);
    let count: u32 = count.parse().ok()?;
    match unit {
        "m" => Some(count),
        "h" => Some(count * 60),
        "d" => Some(count * 60 * 24),
        "w" => Some(count * 60 * 24 * 7),
        _ => None,
    }
}

impl DataSource for KrakenSource {
    fn load(&self) -> Result<Data, DataError> {
        let url = format!(
            "https://api.kraken.com/0/public/OHLC?pair={}&interval={}",
            self.pair, self.minutes
        );
        let body = fetch(&url)?;

        let bad = |message: String| DataError::Provider {
            provider: "kraken",
            message,
        };

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| bad(format!("{e}: {body:.200}")))?;

        // Kraken reports failures in an `error` array rather than by status.
        if let Some(errors) = json.get("error").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                return Err(bad(errors
                    .iter()
                    .filter_map(|e| e.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")));
            }
        }

        // `result` holds the candles under Kraken's own name for the pair —
        // "XBTUSD" comes back as "XXBTZUSD" — alongside a `last` cursor, so the
        // candles are whichever other key is there.
        let result = json
            .get("result")
            .and_then(|r| r.as_object())
            .ok_or_else(|| bad("no result".to_string()))?;
        let candles = result
            .iter()
            .find(|(key, _)| key.as_str() != "last")
            .and_then(|(_, value)| value.as_array())
            .ok_or_else(|| bad(format!("no candles for {}", self.pair)))?;

        let rows = candles
            .iter()
            .map(|c| {
                Some(Ohlcv {
                    // Kraken timestamps are seconds; a bar's time is in ms.
                    time: c.get(0)?.as_i64()? * 1000,
                    open: quoted(c.get(1)?)?,
                    high: quoted(c.get(2)?)?,
                    low: quoted(c.get(3)?)?,
                    close: quoted(c.get(4)?)?,
                    // [5] is vwap; volume is [6].
                    volume: quoted(c.get(6)?)?,
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| bad("unexpected candle shape".to_string()))?;

        Ok(Data::from_ohlcv(rows).with_syminfo(SymInfo {
            ticker: self.pair.clone(),
            tickerid: format!("KRAKEN:{}", self.pair),
            prefix: "KRAKEN".to_string(),
            type_: "crypto".to_string(),
            ..SymInfo::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::minutes_of;

    #[test]
    fn intervals_convert_to_kraken_minutes() {
        assert_eq!(minutes_of("1m"), Some(1));
        assert_eq!(minutes_of("15m"), Some(15));
        assert_eq!(minutes_of("1h"), Some(60));
        assert_eq!(minutes_of("4h"), Some(240));
        assert_eq!(minutes_of("1d"), Some(1440));
        assert_eq!(minutes_of("1w"), Some(10080));
        assert_eq!(minutes_of("bogus"), None);
        assert_eq!(minutes_of(""), None);
    }
}
