mod bar;
mod library;
mod output;
mod series_buffer;
mod syminfo;
mod timeframe;
mod version;

pub use bar::{Bar, Data, Ohlcv};
pub use library::{FileResolver, LibraryLoader};
pub use output::{
    AlertCondition, AlertConditionOutput, BoxOutput, Color, DefaultPineOutput, FillObject,
    FillOutput, GlobalContext, GlobalOutput, Indicator, IndicatorOutput, Input, InputOutput,
    InputValue, Label, LabelOutput, LineObject, LineOutput, LogEntry, LogLevel, LogOutput, PineBox,
    PineOutput, Plot, PlotOutput, Plotarrow, Plotbar, Plotcandle, Plotchar, Plotshape, Table,
    TableCell, TableOutput,
};
pub use series_buffer::{SeriesBuffer, MAX_LOOKBACK};
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
