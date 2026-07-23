//! Bars from Binance's public klines endpoint.

use crate::{fetch, quoted, DataError, DataSource};
use pine_core::{Data, Ohlcv, SymInfo};

/// Candles for a Binance symbol.
///
/// ```no_run
/// # use pine_data::{BinanceSource, DataSource};
/// let data = BinanceSource::new("BTCUSDT", "1h").limit(500).load()?;
/// # Ok::<(), pine_data::DataError>(())
/// ```
#[derive(Debug, Clone)]
pub struct BinanceSource {
    symbol: String,
    interval: String,
    limit: usize,
}

impl BinanceSource {
    /// `symbol` is a Binance pair such as `"BTCUSDT"`; `interval` one of its
    /// kline intervals (`"1m"`, `"1h"`, `"1d"`, …).
    pub fn new(symbol: &str, interval: &str) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            interval: interval.to_string(),
            limit: 500,
        }
    }

    /// How many of the most recent candles to ask for. Binance caps this at 1000.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl DataSource for BinanceSource {
    fn load(&self) -> Result<Data, DataError> {
        let url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
            self.symbol, self.interval, self.limit
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

        Ok(Data::from_ohlcv(rows).with_syminfo(SymInfo {
            ticker: self.symbol.clone(),
            tickerid: format!("BINANCE:{}", self.symbol),
            prefix: "BINANCE".to_string(),
            type_: "crypto".to_string(),
            ..SymInfo::default()
        }))
    }
}
