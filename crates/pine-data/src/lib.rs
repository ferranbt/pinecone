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

// `::csv` is the crate; the module below shadows the bare name here.
mod csv;

pub use csv::CsvSource;

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
}

/// Somewhere [`Data`] can be loaded from — a file, an exchange, a fixture.
pub trait DataSource {
    fn load(&self) -> Result<Data, DataError>;
}
