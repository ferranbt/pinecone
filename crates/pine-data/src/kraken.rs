//! Bars from Kraken's public OHLC endpoint.

use crate::{fetch, quoted, DataError};
use pine_core::{Data, DataProvider, Ohlcv, ProviderError, SymInfo, Timeframe};

/// Kraken's public OHLC endpoint, as a [`DataProvider`]: it fetches whatever pair
/// and timeframe are asked for.
///
/// ```no_run
/// # use pine_data::KrakenSource;
/// # use pine_core::DataProvider;
/// let data = KrakenSource::new().request("XBTUSD", "60".parse()?)?;
/// # Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct KrakenSource;

impl KrakenSource {
    pub fn new() -> Self {
        Self
    }
}

impl DataProvider for KrakenSource {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError> {
        let pair = symbol.to_uppercase();
        // Kraken asks for the interval as a number of minutes; a sub-minute or
        // month timeframe has none, and Kraken serves neither, so it falls back
        // to the hour its API defaults to.
        let minutes = timeframe.as_minutes().unwrap_or(60);

        let url =
            format!("https://api.kraken.com/0/public/OHLC?pair={pair}&interval={minutes}");
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
                    .join(", "))
                .into());
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
            .ok_or_else(|| bad(format!("no candles for {pair}")))?;

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
                ticker: pair.clone(),
                tickerid: format!("KRAKEN:{pair}"),
                prefix: "KRAKEN".to_string(),
                type_: "crypto".to_string(),
                ..SymInfo::default()
            })
            .with_timeframe(timeframe))
    }
}
