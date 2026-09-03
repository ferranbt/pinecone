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
    Extend, FillObject, FillOutput, Frequency, FullPineOutput, GlobalContext, GlobalOutput, HAlign,
    Indicator, Input, InputOutput, InputValue, Label, LabelOutput, LabelStyle, Library, LineObject,
    LineOutput, LineStyle, LinefillObject, LogEntry, LogLevel, LogOutput, MetadataOutput, PineBox,
    PineOutput, Plot, PlotOutput, Plotarrow, Plotbar, Plotcandle, Plotchar, Plotshape,
    PolylineObject, Size, Table, TableCell, TableOutput, XLoc, YLoc,
};
pub use series_buffer::{SeriesBuffer, MAX_LOOKBACK};
pub use syminfo::SymInfo;
pub use timeframe::{Timeframe, TimeframeError, TimeframeUnit};
pub use version::{PineVersion, VersionError};

/// The error a [`DataProvider`] fails with.
pub type ProviderError = Box<dyn std::error::Error + Send + Sync>;

/// A source of market data: given a symbol and a Pine timeframe, produce its
/// bars.
/// One price row of a volume footprint: the price range it spans and the volume
/// traded into the bid (`sell`) and ask (`buy`) at that level.
#[derive(Debug, Clone)]
pub struct FootprintRow {
    pub down_price: f64,
    pub up_price: f64,
    pub buy_volume: f64,
    pub sell_volume: f64,
}

pub trait DataProvider {
    fn request(&self, symbol: &str, timeframe: Timeframe) -> Result<Data, ProviderError>;

    /// The volume footprint rows for the current bar (`request.footprint`),
    /// lowest price first. `None` — the default — means the host has no order-flow
    /// feed, so the script reads `na`.
    fn footprint(
        &self,
        _ticks_per_row: f64,
        _va_percent: f64,
        _imbalance_percent: f64,
    ) -> Option<Vec<FootprintRow>> {
        None
    }

    /// A fundamental financial metric (`request.financial`), e.g. `id =
    /// "TOTAL_REVENUE"`, `period = "FY"`/`"FQ"`. `None` — the default — means the
    /// host has no such feed, so the script reads `na`.
    fn financial(&self, _symbol: &str, _id: &str, _period: &str) -> Option<f64> {
        None
    }
    /// A dividend field (`request.dividends`), e.g. `"gross"`/`"net"`.
    fn dividends(&self, _ticker: &str, _field: &str) -> Option<f64> {
        None
    }
    /// An earnings field (`request.earnings`), e.g. `"actual"`/`"estimate"`.
    fn earnings(&self, _ticker: &str, _field: &str) -> Option<f64> {
        None
    }
    /// A splits field (`request.splits`), e.g. `"numerator"`/`"denominator"`.
    fn splits(&self, _ticker: &str, _field: &str) -> Option<f64> {
        None
    }
    /// An economic series (`request.economic`), e.g. `country = "US"`, `field =
    /// "GDP"`.
    fn economic(&self, _country: &str, _field: &str) -> Option<f64> {
        None
    }
    /// The exchange rate `from`→`to` (`request.currency_rate`). Same-currency
    /// pairs are answered as `1.0` by the builtin without consulting the feed.
    fn currency_rate(&self, _from: &str, _to: &str) -> Option<f64> {
        None
    }
}
