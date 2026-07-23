//! Bars from Binance's public klines endpoint.

use crate::{fetch, quoted, DataError, DataSource};
use pine_core::{Data, Ohlcv, SymInfo, Timeframe};

/// Candles for a Binance symbol.
///
/// ```no_run
/// # use pine_data::{BinanceSource, DataSource};
/// let data = BinanceSource::new("BTCUSDT", "1h".parse()?).limit(500).load()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct BinanceSource {
    symbol: String,
    timeframe: Timeframe,
    limit: usize,
}

impl BinanceSource {
    /// `symbol` is a Binance pair such as `"BTCUSDT"`.
    pub fn new(symbol: &str, timeframe: Timeframe) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            timeframe,
            limit: 500,
        }
    }

    /// The timeframe as Binance spells its kline intervals. Binance takes whole
    /// hours as `"1h"` rather than `"60m"`, and writes a month `"1M"`.
    fn interval(&self) -> String {
        let tf = &self.timeframe;
        match tf.as_minutes() {
            Some(minutes) if tf.is_minutes() && minutes % 60 == 0 => format!("{}h", minutes / 60),
            _ if tf.is_minutes() => format!("{}m", tf.multiplier),
            _ if tf.is_daily() => format!("{}d", tf.multiplier),
            _ if tf.is_weekly() => format!("{}w", tf.multiplier),
            _ if tf.is_monthly() => format!("{}M", tf.multiplier),
            _ => format!("{}m", tf.multiplier),
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
            self.symbol,
            self.interval(),
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
            ticker: self.symbol.clone(),
            tickerid: format!("BINANCE:{}", self.symbol),
            prefix: "BINANCE".to_string(),
            type_: "crypto".to_string(),
            ..SymInfo::default()
        });

        // The requested timeframe is authoritative, not one guessed from the
        // spacing between bars.
        Ok(data.with_timeframe(self.timeframe.clone()))
    }
}
