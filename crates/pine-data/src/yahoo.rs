//! Bars from Yahoo Finance's chart endpoint — the one `yfinance` uses.

use crate::{fetch, DataError, DataSource};
use pine_core::{Data, Ohlcv, SymInfo};

/// Candles for a Yahoo Finance symbol: equities, ETFs, indices, FX and crypto.
///
/// ```no_run
/// # use pine_data::{DataSource, YahooSource};
/// let data = YahooSource::new("AAPL", "1d").range("6mo").load()?;
/// # Ok::<(), pine_data::DataError>(())
/// ```
#[derive(Debug, Clone)]
pub struct YahooSource {
    symbol: String,
    interval: String,
    range: String,
}

impl YahooSource {
    /// `symbol` is a Yahoo ticker such as `"AAPL"`, `"BTC-USD"` or `"^GSPC"`.
    /// `interval` is one of Yahoo's (`"1m"`, `"1h"`, `"1d"`, `"1wk"`, `"1mo"`).
    ///
    /// Yahoo limits how far back the finer intervals reach — minute data only
    /// goes back days — so a range it will not serve comes back empty.
    pub fn new(symbol: &str, interval: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            range: "1mo".to_string(),
        }
    }

    /// How far back to fetch: `"1d"`, `"5d"`, `"1mo"`, `"1y"`, `"max"`, …
    pub fn range(mut self, range: &str) -> Self {
        self.range = range.to_string();
        self
    }
}

impl DataSource for YahooSource {
    fn load(&self) -> Result<Data, DataError> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval={}&range={}",
            self.symbol, self.interval, self.range
        );
        let body = fetch(&url)?;

        let bad = |message: String| DataError::Provider {
            provider: "yahoo",
            message,
        };

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| bad(format!("{e}: {body:.200}")))?;
        let chart = json
            .get("chart")
            .ok_or_else(|| bad("no chart".to_string()))?;

        // Yahoo reports failures in the body rather than by status.
        if let Some(error) = chart.get("error").filter(|e| !e.is_null()) {
            return Err(bad(error.to_string()));
        }

        let result = chart
            .get("result")
            .and_then(|r| r.get(0))
            .ok_or_else(|| bad(format!("no data for {}", self.symbol)))?;

        let times = result
            .get("timestamp")
            .and_then(|t| t.as_array())
            .ok_or_else(|| bad("no timestamps".to_string()))?;
        let quote = result
            .get("indicators")
            .and_then(|i| i.get("quote"))
            .and_then(|q| q.get(0))
            .ok_or_else(|| bad("no quotes".to_string()))?;

        // The prices come back as parallel columns rather than one array per
        // candle, and a gap in the data is a null in every column.
        let column = |name: &str| quote.get(name).and_then(|c| c.as_array());
        let (opens, highs, lows, closes, volumes) = (
            column("open"),
            column("high"),
            column("low"),
            column("close"),
            column("volume"),
        );

        let rows = (0..times.len())
            .filter_map(|i| {
                let at = |c: Option<&Vec<serde_json::Value>>| c?.get(i)?.as_f64();
                Some(Ohlcv {
                    // Yahoo timestamps are seconds; a bar's time is in ms.
                    time: times.get(i)?.as_i64()? * 1000,
                    open: at(opens)?,
                    high: at(highs)?,
                    low: at(lows)?,
                    close: at(closes)?,
                    volume: at(volumes).unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>();

        let exchange = result
            .get("meta")
            .and_then(|m| m.get("exchangeName"))
            .and_then(|e| e.as_str())
            .unwrap_or("YAHOO")
            .to_string();
        let currency = result
            .get("meta")
            .and_then(|m| m.get("currency"))
            .and_then(|c| c.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(Data::from_ohlcv(rows).with_syminfo(SymInfo {
            ticker: self.symbol.clone(),
            tickerid: format!("{exchange}:{}", self.symbol),
            prefix: exchange,
            currency,
            ..SymInfo::default()
        }))
    }
}
