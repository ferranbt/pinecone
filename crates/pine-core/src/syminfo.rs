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
    /// Root of a futures contract, e.g. `"ES"` (`syminfo.root`).
    pub root: String,
    /// The current standard contract of a continuous futures symbol
    /// (`syminfo.current_contract`).
    pub current_contract: String,
    /// Symbol of the main pair for a spread/derived symbol
    /// (`syminfo.main_tickerid`).
    pub main_tickerid: String,
    /// ISIN of the symbol (`syminfo.isin`).
    pub isin: String,
    /// Country the symbol is traded in, e.g. `"US"` (`syminfo.country`).
    pub country: String,
    /// Economic sector, e.g. `"Technology"` (`syminfo.sector`).
    pub sector: String,
    /// Industry, e.g. `"Semiconductors"` (`syminfo.industry`).
    pub industry: String,
    /// How volume is reported, e.g. `"base"`/`"quote"` (`syminfo.volumetype`).
    pub volumetype: String,
    /// Number of mintick increments in the minimum price move
    /// (`syminfo.minmove`).
    pub minmove: f64,
    /// Price scale, the denominator of a fractional price (`syminfo.pricescale`).
    pub pricescale: f64,
    /// Minimum tradable contract size (`syminfo.mincontract`).
    pub mincontract: f64,
    /// Expiration date of a derivative, as a UNIX timestamp
    /// (`syminfo.expiration_date`).
    pub expiration_date: f64,
    /// Number of employees (`syminfo.employees`).
    pub employees: f64,
    /// Number of shareholders (`syminfo.shareholders`).
    pub shareholders: f64,
    /// Total shares outstanding (`syminfo.shares_outstanding_total`).
    pub shares_outstanding_total: f64,
    /// Float shares outstanding (`syminfo.shares_outstanding_float`).
    pub shares_outstanding_float: f64,
    /// Analyst recommendation totals (`syminfo.recommendations_*`).
    pub recommendations_buy: f64,
    pub recommendations_buy_strong: f64,
    pub recommendations_hold: f64,
    pub recommendations_sell: f64,
    pub recommendations_sell_strong: f64,
    pub recommendations_total: f64,
    /// Date the recommendations were issued, as a UNIX timestamp
    /// (`syminfo.recommendations_date`).
    pub recommendations_date: f64,
    /// Analyst price targets (`syminfo.target_price_*`).
    pub target_price_average: f64,
    pub target_price_high: f64,
    pub target_price_low: f64,
    pub target_price_median: f64,
    pub target_price_estimates: f64,
    /// Date the price targets were issued, as a UNIX timestamp
    /// (`syminfo.target_price_date`).
    pub target_price_date: f64,
}
