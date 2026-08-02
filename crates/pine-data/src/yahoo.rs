//! Bars from Yahoo Finance's chart endpoint

use crate::{fetch, DataError};
use pine_core::{Data, DataProvider, Ohlcv, ProviderError, SymInfo, Timeframe};
use serde::Deserialize;

/// Yahoo Finance's chart endpoint (the one `yfinance` uses) as a [`DataProvider`]
/// for equities, ETFs, indices, FX and crypto: it fetches whatever symbol and
/// timeframe are asked for.
///
/// Yahoo limits how far back the finer intervals reach — minute data only goes
/// back days — so a range it will not serve comes back empty. Widen it with
/// [`range`](Self::range).
///
/// ```no_run
/// # use pine_data::YahooSource;
/// # use pine_core::DataProvider;
/// let data = YahooSource::new().range("6mo").request("AAPL", "1D".parse()?)?;
/// # Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
/// ```
#[derive(Debug, Clone)]
pub struct YahooSource {
    range: String,
}

impl Default for YahooSource {
    fn default() -> Self {
        Self::new()
    }
}

impl YahooSource {
    pub fn new() -> Self {
        Self {
            range: "1mo".to_string(),
        }
    }

    /// How far back to fetch: `"1d"`, `"5d"`, `"1mo"`, `"1y"`, `"max"`, …
    pub fn range(mut self, range: &str) -> Self {
        self.range = range.to_string();
        self
    }

    /// A timeframe as Yahoo spells its intervals: whole hours as `"1h"`, and
    /// `"1wk"` / `"1mo"` for the longer periods.
    fn interval(tf: &Timeframe) -> String {
        match tf.as_minutes() {
            Some(minutes) if tf.is_minutes() && minutes % 60 == 0 => format!("{}h", minutes / 60),
            _ if tf.is_minutes() => format!("{}m", tf.multiplier),
            _ if tf.is_daily() => format!("{}d", tf.multiplier),
            _ if tf.is_weekly() => format!("{}wk", tf.multiplier),
            _ if tf.is_monthly() => format!("{}mo", tf.multiplier),
            _ => format!("{}m", tf.multiplier),
        }
    }
}

#[derive(Debug, Deserialize)]
struct HttpResult {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Option<Vec<Res>>,
    error: Option<ChartError>,
}

#[derive(Debug, Deserialize)]
struct ChartError {
    code: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct Res {
    meta: Metadata,
    #[serde(default)]
    timestamp: Vec<i64>,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    exchange_name: Option<String>,
    currency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    #[serde(default)]
    quote: Vec<Quote>,
}

#[derive(Debug, Default, Deserialize)]
struct Quote {
    #[serde(default)]
    open: Vec<Option<f64>>,
    #[serde(default)]
    high: Vec<Option<f64>>,
    #[serde(default)]
    low: Vec<Option<f64>>,
    #[serde(default)]
    close: Vec<Option<f64>>,
    #[serde(default)]
    volume: Vec<Option<f64>>,
}

impl DataProvider for YahooSource {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval={}&range={}",
            symbol,
            Self::interval(&timeframe),
            self.range
        );
        let body = fetch(&url)?;

        let bad = |message: String| DataError::Provider {
            provider: "yahoo",
            message,
        };

        let response: HttpResult =
            serde_json::from_str(&body).map_err(|e| bad(format!("{e}: {body:.200}")))?;

        if let Some(error) = response.chart.error {
            return Err(bad(format!("{}: {}", error.code, error.description)).into());
        }

        let result = response
            .chart
            .result
            .and_then(|results| results.into_iter().next())
            .ok_or_else(|| bad(format!("no data for {symbol}")))?;
        let quote = result
            .indicators
            .quote
            .into_iter()
            .next()
            .unwrap_or_default();

        let rows = (0..result.timestamp.len())
            .filter_map(|i| {
                let at = |column: &[Option<f64>]| column.get(i).copied().flatten();
                Some(Ohlcv {
                    // Yahoo timestamps are seconds; a bar's time is in ms.
                    time: result.timestamp.get(i)? * 1000,
                    open: at(&quote.open)?,
                    high: at(&quote.high)?,
                    low: at(&quote.low)?,
                    close: at(&quote.close)?,
                    volume: at(&quote.volume).unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>();

        let exchange = result.meta.exchange_name.unwrap_or("YAHOO".to_string());
        let currency = result.meta.currency.unwrap_or_default();

        let data = Data::from_ohlcv(rows).with_syminfo(SymInfo {
            ticker: symbol.to_string(),
            tickerid: format!("{exchange}:{symbol}"),
            prefix: exchange,
            currency,
            ..SymInfo::default()
        });

        // The requested timeframe is authoritative. Inference would be wrong
        // here: an equity session leaves a short last bar and uneven gaps.
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yahoo() {
        let data = YahooSource::new()
            .range("6mo")
            .request("AAPL", "1D".parse().unwrap())
            .unwrap();

        assert_ne!(data.bars.len(), 0);
    }
}
