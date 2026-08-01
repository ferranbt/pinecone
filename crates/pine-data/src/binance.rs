//! Bars from Binance's public klines endpoint.

use crate::{fetch, quoted, DataError};
use pine_core::{Data, DataProvider, Ohlcv, ProviderError, SymInfo, Timeframe};

/// Binance's public klines endpoint, as a [`DataProvider`]: it fetches whatever
/// symbol and timeframe are asked for.
///
/// ```no_run
/// # use pine_data::BinanceSource;
/// # use pine_core::DataProvider;
/// let data = BinanceSource::new().limit(500).request("BTCUSDT", "60".parse()?)?;
/// # Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
/// ```
#[derive(Debug, Clone)]
pub struct BinanceSource {
    limit: usize,
}

impl Default for BinanceSource {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceSource {
    pub fn new() -> Self {
        Self { limit: 500 }
    }

    /// How many of the most recent candles to ask for. Binance caps this at 1000.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// A timeframe as Binance spells its kline intervals. Binance takes whole
    /// hours as `"1h"` rather than `"60m"`, and writes a month `"1M"`.
    fn interval(tf: &Timeframe) -> String {
        match tf.as_minutes() {
            Some(minutes) if tf.is_minutes() && minutes % 60 == 0 => format!("{}h", minutes / 60),
            _ if tf.is_minutes() => format!("{}m", tf.multiplier),
            _ if tf.is_daily() => format!("{}d", tf.multiplier),
            _ if tf.is_weekly() => format!("{}w", tf.multiplier),
            _ if tf.is_monthly() => format!("{}M", tf.multiplier),
            _ => format!("{}m", tf.multiplier),
        }
    }
}

impl DataProvider for BinanceSource {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError> {
        let symbol = symbol.to_uppercase();

        let url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
            symbol,
            Self::interval(&timeframe),
            self.limit
        );
        let body = fetch(&url)?;

        let bad = |message: String| DataError::Provider {
            provider: "binance",
            message,
        };

        // Each kline is an array: [openTime, open, high, low, close, volume, …]
        // with the prices sent as strings.
        let klines: Vec<serde_json::Value> =
            serde_json::from_str(&body).map_err(|e| bad(format!("{e}: {body:.200}")))?;

        let rows = klines
            .iter()
            .map(|k| {
                Some(Ohlcv {
                    time: k.get(0)?.as_i64()?,
                    open: quoted(k.get(1)?)?,
                    high: quoted(k.get(2)?)?,
                    low: quoted(k.get(3)?)?,
                    close: quoted(k.get(4)?)?,
                    volume: quoted(k.get(5)?)?,
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| bad("unexpected kline shape".to_string()))?;

        let data = Data::from_ohlcv(rows).with_syminfo(SymInfo {
            ticker: symbol.clone(),
            tickerid: format!("BINANCE:{symbol}"),
            prefix: "BINANCE".to_string(),
            type_: "crypto".to_string(),
            ..SymInfo::default()
        });

        // The requested timeframe is authoritative, not one guessed from the
        // spacing between bars.
        Ok(data.with_timeframe(timeframe))
    }
}
