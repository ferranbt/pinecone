//! Bars from Kraken's public OHLC endpoint.

use crate::{fetch, DataError};
use pine_core::{Data, DataProvider, Ohlcv, ProviderError, SymInfo, Timeframe};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;

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

#[derive(Debug, Deserialize)]
struct HttpResult {
    #[serde(default)]
    error: Vec<String>,
    result: Option<OhlcResult>,
}

// The candles sit under Kraken's own name for the pair — "XBTUSD" comes back as
// "XXBTZUSD" — so they are whichever key `last` is not.
#[derive(Debug, Deserialize)]
struct OhlcResult {
    #[allow(dead_code)]
    last: i64,
    #[serde(flatten)]
    pairs: HashMap<String, Vec<Candle>>,
}

// [time, open, high, low, close, vwap, volume, count], the prices as strings.
#[derive(Debug, Deserialize)]
struct Candle(
    i64,
    #[serde(deserialize_with = "quoted")] f64,
    #[serde(deserialize_with = "quoted")] f64,
    #[serde(deserialize_with = "quoted")] f64,
    #[serde(deserialize_with = "quoted")] f64,
    #[serde(deserialize_with = "quoted")] f64,
    #[serde(deserialize_with = "quoted")] f64,
    i64,
);

impl From<Candle> for Ohlcv {
    fn from(candle: Candle) -> Self {
        let Candle(time, open, high, low, close, _vwap, volume, _count) = candle;
        Ohlcv {
            // Kraken timestamps are seconds; a bar's time is in ms.
            time: time * 1000,
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

fn quoted<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    <&str>::deserialize(deserializer)?
        .parse()
        .map_err(de::Error::custom)
}

impl DataProvider for KrakenSource {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError> {
        let pair = symbol.to_uppercase();
        // Kraken asks for the interval as a number of minutes; a sub-minute or
        // month timeframe has none, and Kraken serves neither, so it falls back
        // to the hour its API defaults to.
        let minutes = timeframe.as_minutes().unwrap_or(60);

        let url = format!("https://api.kraken.com/0/public/OHLC?pair={pair}&interval={minutes}");
        let body = fetch(&url)?;

        let bad = |message: String| DataError::Provider {
            provider: "kraken",
            message,
        };

        let response: HttpResult =
            serde_json::from_str(&body).map_err(|e| bad(format!("{e}: {body:.200}")))?;

        if !response.error.is_empty() {
            return Err(bad(response.error.join(", ")).into());
        }

        let candles = response
            .result
            .and_then(|result| result.pairs.into_values().next())
            .ok_or_else(|| bad(format!("no candles for {pair}")))?;

        let rows = candles.into_iter().map(Ohlcv::from);

        Ok(Data::from_ohlcv(rows).with_syminfo(SymInfo {
            ticker: pair.clone(),
            tickerid: format!("KRAKEN:{pair}"),
            prefix: "KRAKEN".to_string(),
            type_: "crypto".to_string(),
            ..SymInfo::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kraken() {
        let data = KrakenSource::new()
            .request("XBTUSD", "60".parse().unwrap())
            .unwrap();

        assert_ne!(data.bars.len(), 0);
    }
}
