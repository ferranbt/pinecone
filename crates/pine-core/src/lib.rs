mod bar;
mod syminfo;
mod timeframe;
mod version;

pub use bar::{Bar, Data, Ohlcv};
pub use syminfo::SymInfo;
pub use timeframe::{Timeframe, TimeframeError, TimeframeUnit};
pub use version::{PineVersion, VersionError};

/// The error a [`DataProvider`] fails with.
pub type ProviderError = Box<dyn std::error::Error + Send + Sync>;

/// A source of market data: given a symbol and a Pine timeframe, produce its
/// bars.
pub trait DataProvider {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError>;
}
