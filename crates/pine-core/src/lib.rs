mod bar;
mod library;
mod output;
mod series_buffer;
mod syminfo;
mod timeframe;
mod version;

pub use bar::{Bar, Data, Ohlcv};
pub use library::{DirLoader, FileResolver, LibraryLoader};
pub use output::{
    AlertCondition, AlertConditionOutput, BoxOutput, Color, DefaultPineOutput, DrawingOutput,
    FillObject, FillOutput, GlobalContext, GlobalOutput, Indicator, Input, InputOutput, InputValue,
    Label, LabelOutput, Library, LineObject, LineOutput, LinefillObject, LogEntry, LogLevel,
    LogOutput, MetadataOutput, PineBox, PineOutput, Plot, PlotOutput, Plotarrow, Plotbar,
    Plotcandle, Plotchar, Plotshape, PolylineObject, Table, TableCell, TableOutput,
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
