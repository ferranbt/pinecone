//! Loading market data for a script to run over.
//!
//! The data itself is [`pine_core::Data`] — bars plus the symbol and timeframe
//! they belong to. This crate is the ways of getting one: a CSV file today, an
//! exchange later.
//!
//! Series history and stateful builtins accumulate as bars execute, so a script
//! must be replayed from its first bar. Handing over the whole series at once is
//! what makes that the only thing a caller *can* do.

use pine_core::{Data, Ohlcv};

mod static_provider;

mod binance;
mod kraken;
mod yahoo;

pub use binance::BinanceSource;
pub use kraken::KrakenSource;
pub use static_provider::{resample, StaticProvider};
pub use yahoo::YahooSource;

pub(crate) fn fetch(url: &str) -> Result<String, DataError> {
    // This call blocks on purpose such that a Script execution
    // stays sync all the way down
    ureq::get(url)
        .set("User-Agent", "pinecone/0.1")
        .call()
        .map_err(|source| DataError::Http {
            url: url.to_string(),
            message: source.to_string(),
        })?
        .into_string()
        .map_err(|source| DataError::Http {
            url: url.to_string(),
            message: source.to_string(),
        })
}

/// Read a price that a provider sends as a JSON string rather than a number.
pub(crate) fn quoted(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::String(text) => text.parse().ok(),
        other => other.as_f64(),
    }
}

pub fn synthetic(count: usize) -> Data {
    Data::from_ohlcv((0..count).map(|i| {
        let close = 100.0 + i as f64;
        Ohlcv {
            time: i as i64 * 60_000,
            open: close - 1.0,
            high: close + 1.0,
            low: close - 2.0,
            close,
            volume: 1000.0,
        }
    }))
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    /// A file could not be opened, or its contents did not parse. The inner
    /// error carries the row and line a bad record was on.
    #[error("{path}: {source}")]
    Read {
        path: String,
        #[source]
        source: ::csv::Error,
    },

    /// The request itself failed — unreachable host, non-2xx status, bad body.
    #[error("{url}: {message}")]
    Http { url: String, message: String },

    /// The provider answered, but not with the bars we asked for: an in-band
    /// error, an unknown symbol, or a shape we cannot read.
    #[error("{provider}: {message}")]
    Provider {
        provider: &'static str,
        message: String,
    },
}
