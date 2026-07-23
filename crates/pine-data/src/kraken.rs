//! Bars from Kraken's public OHLC endpoint.

use crate::{fetch, quoted, DataError, DataSource};
use pine_core::{Data, Ohlcv, SymInfo, Timeframe};

/// Candles for a Kraken pair.
///
/// ```no_run
/// # use pine_data::{DataSource, KrakenSource};
/// let data = KrakenSource::new("XBTUSD", "1h".parse()?).load()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct KrakenSource {
    pair: String,
    timeframe: Timeframe,
}

impl KrakenSource {
    /// `pair` is a Kraken pair such as `"XBTUSD"`.
    pub fn new(pair: &str, timeframe: Timeframe) -> Self {
        Self {
            pair: pair.to_uppercase(),
            timeframe,
        }
    }

    /// Kraken asks for the interval as a number of minutes. A timeframe with no
    /// whole-minute length (sub-minute, or a month) has none, and Kraken serves
    /// neither, so it falls back to the hour its API defaults to.
    fn minutes(&self) -> u32 {
        self.timeframe.as_minutes().unwrap_or(60)
    }
}

impl DataSource for KrakenSource {
    fn load(&self) -> Result<Data, DataError> {
        let url = format!(
            "https://api.kraken.com/0/public/OHLC?pair={}&interval={}",
            self.pair,
            self.minutes()
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

        Ok(Data::from_ohlcv(rows)
            .with_syminfo(SymInfo {
                ticker: self.pair.clone(),
                tickerid: format!("KRAKEN:{}", self.pair),
                prefix: "KRAKEN".to_string(),
                type_: "crypto".to_string(),
                ..SymInfo::default()
            })
            .with_timeframe(self.timeframe.clone()))
    }
}
