//! Bars from Yahoo Finance's chart endpoint — the one `yfinance` uses.

use crate::{fetch, DataError};
use pine_core::{Data, DataProvider, Ohlcv, ProviderError, SymInfo, Timeframe};

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

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| bad(format!("{e}: {body:.200}")))?;
        let chart = json
            .get("chart")
            .ok_or_else(|| bad("no chart".to_string()))?;

        // Yahoo reports failures in the body rather than by status.
        if let Some(error) = chart.get("error").filter(|e| !e.is_null()) {
            return Err(bad(error.to_string()).into());
        }

        let result = chart
            .get("result")
            .and_then(|r| r.get(0))
            .ok_or_else(|| bad(format!("no data for {symbol}")))?;

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

        let data = Data::from_ohlcv(rows).with_syminfo(SymInfo {
            ticker: symbol.to_string(),
            tickerid: format!("{exchange}:{symbol}"),
            prefix: exchange,
            currency,
            ..SymInfo::default()
        });

        // The requested timeframe is authoritative. Inference would be wrong
        // here: an equity session leaves a short last bar and uneven gaps.
        Ok(data.with_timeframe(timeframe))
    }
}
