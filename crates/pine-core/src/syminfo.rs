/// Symbol information the host supplies for a script run.
///
/// Mirrors PineScript's `syminfo.*` namespace: the instrument's identity and
/// trading conventions. A script may read any of these; the host fills in what
/// it knows and leaves the rest at their defaults.
#[derive(Clone, Debug, Default)]
pub struct SymInfo {
    /// Symbol without exchange prefix, e.g. `"AAPL"` (`syminfo.ticker`).
    pub ticker: String,
    /// Fully qualified symbol including exchange, e.g. `"NASDAQ:AAPL"`
    /// (`syminfo.tickerid`).
    pub tickerid: String,
    /// Human-readable description of the symbol (`syminfo.description`).
    pub description: String,
    /// Exchange/data-source prefix, e.g. `"NASDAQ"` (`syminfo.prefix`).
    pub prefix: String,
    /// Currency the symbol is quoted in, e.g. `"USD"` (`syminfo.currency`).
    pub currency: String,
    /// Base currency for forex pairs, e.g. `"EUR"` in `EURUSD`
    /// (`syminfo.basecurrency`).
    pub basecurrency: String,
    /// Instrument type, e.g. `"stock"`, `"forex"`, `"crypto"` (`syminfo.type`).
    pub type_: String,
    /// Smallest price increment, e.g. `0.01` (`syminfo.mintick`).
    pub mintick: f64,
    /// Currency value of one point of price movement (`syminfo.pointvalue`).
    pub pointvalue: f64,
    /// Exchange timezone, e.g. `"America/New_York"` (`syminfo.timezone`).
    pub timezone: String,
    /// Trading session specification (`syminfo.session`).
    pub session: String,
}
