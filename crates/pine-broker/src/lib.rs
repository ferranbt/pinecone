//! A simulated broker: the emulator a Pine `strategy` trades against.
//!
//! There is no order book and no exchange. A backtest replays historical bars
//! and asks, bar by bar, what *would* have happened — so filling an order is a
//! modelling assumption ([`FillModel`]), not a match against a real resting
//! order. Everything else — position, average price, commission, the trade log,
//! equity — is plain accounting that does not depend on the venue, so there is
//! one [`BarBroker`], not one per exchange.
//!
//! Placing real orders is deliberately out of scope: in Pine that happens
//! outside the strategy, when an alert is delivered to an external system. This
//! crate only simulates.

use pine_core::{Bar, Timeframe};

mod backtest;
mod broker;
mod fill;

pub use backtest::{Backtest, Metrics};
pub use broker::BarBroker;
pub use fill::{FillModel, PineFills};

/// Long or short.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Long,
    Short,
}

impl Direction {
    /// +1 for long, -1 for short — the sign a position of this direction has.
    pub fn sign(self) -> f64 {
        match self {
            Direction::Long => 1.0,
            Direction::Short => -1.0,
        }
    }
}

impl From<&str> for Direction {
    /// From the `strategy.long`/`strategy.short` constants; anything else long.
    fn from(tag: &str) -> Self {
        if tag == "short" {
            Direction::Short
        } else {
            Direction::Long
        }
    }
}

/// The price condition that decides when an order fills.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderKind {
    /// Fills at the next bar's open (or this bar's close under
    /// `process_orders_on_close`).
    Market,
    /// Fills when price reaches `price` or better.
    Limit(f64),
    /// Fills when price reaches `price` or worse.
    Stop(f64),
    /// A limit at `limit`, armed once price reaches `stop`.
    StopLimit { stop: f64, limit: f64 },
}

/// How a `strategy` declaration charges commission.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Commission {
    /// A percentage of the traded value.
    Percent(f64),
    /// A fixed amount per contract traded.
    CashPerContract(f64),
    /// A fixed amount per order.
    CashPerOrder(f64),
}

impl Commission {
    /// The commission on filling `qty` contracts at `price`.
    fn charge(self, qty: f64, price: f64) -> f64 {
        match self {
            Commission::Percent(pct) => qty.abs() * price * pct / 100.0,
            Commission::CashPerContract(cash) => qty.abs() * cash,
            Commission::CashPerOrder(cash) => cash,
        }
    }
}

/// How an order without an explicit `qty` is sized, from the `strategy`
/// declaration's `default_qty_type`/`default_qty_value`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sizing {
    /// A fixed number of contracts (`strategy.fixed`).
    Contracts(f64),
    /// A fixed amount of cash, converted to contracts at the fill price
    /// (`strategy.cash`).
    Cash(f64),
    /// A percentage of current equity, converted at the fill price
    /// (`strategy.percent_of_equity`).
    PercentOfEquity(f64),
}

/// How a risk threshold's value is measured (`strategy.risk.max_drawdown` and
/// `strategy.risk.max_intraday_loss`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskType {
    /// A percentage of the reference (peak) equity.
    Percent(f64),
    /// A fixed cash amount.
    Cash(f64),
}

impl RiskType {
    /// The loss threshold in cash, given the `reference` equity a percentage is
    /// taken against.
    fn threshold(self, reference: f64) -> f64 {
        match self {
            RiskType::Percent(pct) => reference.abs() * pct / 100.0,
            RiskType::Cash(cash) => cash,
        }
    }
}

/// Which entry directions `strategy.risk.allow_entry_in` permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryFilter {
    #[default]
    All,
    LongOnly,
    ShortOnly,
}

impl EntryFilter {
    /// Whether an entry in `direction` is allowed.
    fn allows(self, direction: Direction) -> bool {
        match self {
            EntryFilter::All => true,
            EntryFilter::LongOnly => direction == Direction::Long,
            EntryFilter::ShortOnly => direction == Direction::Short,
        }
    }
}

/// A risk-management rule set by a `strategy.risk.*` call, applied to the broker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskRule {
    /// Restrict entries to one direction (`allow_entry_in`).
    AllowEntryIn(EntryFilter),
    /// Cap the absolute position size in contracts (`max_position_size`).
    MaxPositionSize(f64),
    /// Halt the strategy once equity falls this far from its peak (`max_drawdown`).
    MaxDrawdown(RiskType),
    /// Halt for the rest of the day once equity falls this far from the day's
    /// peak (`max_intraday_loss`).
    MaxIntradayLoss(RiskType),
    /// Halt after this many consecutive losing days (`max_cons_loss_days`).
    MaxConsLossDays(u32),
    /// Block new orders after this many fills in a day (`max_intraday_filled_orders`).
    MaxIntradayFilledOrders(u32),
}

impl Sizing {
    /// The contract count this sizing buys at `price`, given current `equity`.
    fn contracts(self, price: f64, equity: f64) -> f64 {
        match self {
            Sizing::Contracts(c) => c,
            Sizing::Cash(cash) if price > 0.0 => cash / price,
            Sizing::PercentOfEquity(pct) if price > 0.0 => (pct / 100.0 * equity) / price,
            _ => 0.0,
        }
    }
}

/// What happens to the other orders in a One-Cancels-All group when one of them
/// fills, from `strategy.oca.*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OcaType {
    /// Not in an OCA group.
    #[default]
    None,
    /// Cancel the group's other unfilled orders.
    Cancel,
    /// Shrink the group's other unfilled orders by the filled size.
    Reduce,
}

impl From<&str> for OcaType {
    /// From the `strategy.oca.*` constants; anything else is no group.
    fn from(tag: &str) -> Self {
        match tag {
            "cancel" => OcaType::Cancel,
            "reduce" => OcaType::Reduce,
            _ => OcaType::None,
        }
    }
}

/// A submitted order, before it fills. Replaces any pending order with the same
/// `id`, as Pine's order commands do.
#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub direction: Direction,
    /// Contracts to trade; `None` means the strategy's default quantity, which
    /// the broker supplies.
    pub qty: Option<f64>,
    /// For a reducing order, the percentage of the position to close when `qty`
    /// is absent (`strategy.close`'s `qty_percent`).
    pub qty_percent: Option<f64>,
    /// The price used to size a `cash`/`percent_of_equity` order without an
    /// explicit `qty` — Pine sizes from the close of the bar the order is
    /// generated on, not the later fill price. `None` falls back to the fill.
    pub sizing_price: Option<f64>,
    pub kind: OrderKind,
    /// True for `strategy.close`/`close_all`: only ever shrinks the position.
    pub reduce_only: bool,
    /// `strategy.entry` reverses an opposite position and obeys pyramiding;
    /// `strategy.order` does neither.
    pub reverses: bool,
    /// For a reducing order, the entry id whose lots it closes; `None` closes
    /// the whole position (`strategy.close_all`).
    pub close_target: Option<String>,
    /// The One-Cancels-All group this order belongs to, if any.
    pub oca_name: Option<String>,
    pub oca_type: OcaType,
    pub comment: String,
}

impl Order {
    /// A market order to trade `qty` in `direction`, as `strategy.entry` makes.
    pub fn market(id: impl Into<String>, direction: Direction, qty: Option<f64>) -> Self {
        Self {
            id: id.into(),
            direction,
            qty,
            qty_percent: None,
            sizing_price: None,
            kind: OrderKind::Market,
            reduce_only: false,
            reverses: true,
            close_target: None,
            oca_name: None,
            oca_type: OcaType::None,
            comment: String::new(),
        }
    }
}

/// A stop-loss / take-profit bracket attached to a position, from
/// `strategy.exit`. Its legs are evaluated each bar once the position exists;
/// whichever fills first closes it and cancels the other (one-cancels-all).
#[derive(Debug, Clone)]
pub struct Exit {
    pub id: String,
    /// The entry whose position this exits; `None` exits the whole position.
    pub from_entry: Option<String>,
    /// Contracts to exit; `None` exits the whole matched position (or
    /// `qty_percent` of it).
    pub qty: Option<f64>,
    /// Percentage of the matched position to exit when `qty` is absent.
    pub qty_percent: Option<f64>,
    /// Take-profit as a price (`limit`) or a distance in ticks from the entry
    /// (`profit`). A price wins if both are given.
    pub limit: Option<f64>,
    pub profit_ticks: Option<f64>,
    /// Stop-loss as a price (`stop`) or a distance in ticks (`loss`).
    pub stop: Option<f64>,
    pub loss_ticks: Option<f64>,
    /// Trailing stop: it activates once price moves `trail_points` ticks past
    /// the entry favourably, or touches `trail_price`, then trails
    /// `trail_offset` ticks behind the best price reached.
    pub trail_price: Option<f64>,
    pub trail_points: Option<f64>,
    pub trail_offset: Option<f64>,
    /// Runtime state of the trailing stop, carried across bars: whether it has
    /// activated and the best price seen since.
    pub activated: bool,
    pub peak: Option<f64>,
}

impl Exit {
    /// A bracket with no trailing stop and no runtime state yet.
    pub fn resting(
        id: impl Into<String>,
        from_entry: Option<String>,
        qty: Option<f64>,
        qty_percent: Option<f64>,
    ) -> Self {
        Self {
            id: id.into(),
            from_entry,
            qty,
            qty_percent,
            limit: None,
            profit_ticks: None,
            stop: None,
            loss_ticks: None,
            trail_price: None,
            trail_points: None,
            trail_offset: None,
            activated: false,
            peak: None,
        }
    }
}

/// One trade: an entry, and its exit once closed. `size` is signed — positive is
/// long, negative short — matching `strategy.*trades.size`.
#[derive(Debug, Clone)]
pub struct Trade {
    pub entry_id: String,
    pub size: f64,
    pub entry_price: f64,
    pub entry_bar: u64,
    pub exit_price: Option<f64>,
    pub exit_bar: Option<u64>,
    /// Commission on entry, plus exit once closed.
    pub commission: f64,
}

impl Trade {
    /// Realised profit once closed, or profit at `price` while open.
    pub fn profit(&self, price: f64) -> f64 {
        let exit = self.exit_price.unwrap_or(price);
        (exit - self.entry_price) * self.size - self.commission
    }

    pub fn is_open(&self) -> bool {
        self.exit_price.is_none()
    }
}

/// The current net position: signed size and the average price it was opened at.
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    /// Signed: positive long, negative short, zero flat.
    pub size: f64,
    pub avg_price: f64,
}

impl Position {
    pub fn is_flat(&self) -> bool {
        self.size == 0.0
    }
}

/// The simulated broker a strategy trades against.
///
/// Driven one bar at a time: submit orders from the script body, then
/// [`advance`](Broker::advance) to fill whatever the bar allows.
pub trait Broker {
    /// Submit an order, replacing any pending one with the same id.
    fn submit(&mut self, order: Order);

    /// Submit a stop-loss / take-profit bracket, replacing any with the same id.
    fn submit_exit(&mut self, exit: Exit);

    /// Cancel a pending order by id; a filled order is unaffected.
    fn cancel(&mut self, id: &str);

    /// Cancel every pending order.
    fn cancel_all(&mut self);

    /// Apply a risk-management rule (from a `strategy.risk.*` call).
    fn set_risk(&mut self, rule: RiskRule);

    /// Fill whatever `bar` allows, updating the position and trade log.
    fn advance(&mut self, bar: &Bar);

    /// The current net position.
    fn position(&self) -> Position;

    /// The capital the account started with, before any trade or commission.
    fn initial_capital(&self) -> f64;

    /// Account value: capital plus realised and unrealised profit, marked at
    /// `price` (typically the latest close).
    fn equity(&self, price: f64) -> f64;

    /// Trades still open, oldest first.
    fn open_trades(&self) -> Vec<&Trade>;

    /// Trades already closed, in the order they closed.
    fn closed_trades(&self) -> &[Trade];

    /// Running totals over the closed trades.
    fn stats(&self) -> &Stats;

    /// The bar the run halted on if a rest-of-run risk rule fired
    /// (`max_drawdown`, `max_cons_loss_days`), else `None`.
    fn halted_bar(&self) -> Option<u64>;

    /// Runs once the bar's series are set, before the script body: the place
    /// to fill what the previous bar left pending. Defaults to [`advance`].
    ///
    /// [`advance`]: Broker::advance
    fn pre_hook(&mut self, bar: &Bar) {
        self.advance(bar);
    }

    /// Runs after the script body, with what the bar left. Defaults to nothing.
    fn post_hook(&mut self, _bar: &Bar) {}

    /// What the run produced so far — the trade log, equity curve and summary
    /// figures — with `timeframe` carried along so the metrics can annualise.
    fn backtest(&self, timeframe: Timeframe) -> Backtest {
        let stats = self.stats();
        let close = stats.mark_price;

        // Closed trades first, then those still open.
        let mut trades = self.closed_trades().to_vec();
        trades.extend(self.open_trades().into_iter().cloned());

        let open_profit = self
            .open_trades()
            .iter()
            .fold(0.0, |acc, t| acc + t.profit(close));
        let initial_capital = self.initial_capital();
        let equity = stats.equity.clone();
        let final_equity = equity.last().copied().unwrap_or(initial_capital);

        Backtest {
            initial_capital,
            net_profit: final_equity - initial_capital - open_profit,
            open_profit,
            gross_profit: stats.gross_profit,
            gross_loss: stats.gross_loss,
            max_drawdown: stats.max_drawdown,
            max_runup: stats.max_runup,
            win_trades: stats.wins,
            loss_trades: stats.losses,
            even_trades: stats.evens,
            position_size: self.position().size,
            mark_price: close,
            equity,
            trades,
            halted: self.halted_bar(),
            timeframe,
        }
    }
}

/// The account settings a `strategy()` declaration configures its broker with,
/// so a custom [`BrokerFactory`] can honour the script's parameters rather than
/// inventing its own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrokerConfig {
    /// Starting capital (`strategy.initial_capital`).
    pub initial_capital: f64,
    /// The symbol's tick size, or 0 when unknown.
    pub mintick: f64,
    /// How an order's absent `qty` is sized.
    pub sizing: Sizing,
    /// How many entries in the same direction may stack (`pyramiding`).
    pub pyramiding: usize,
    /// Per-trade commission, or `None` when the script sets none.
    pub commission: Option<Commission>,
    /// Slippage applied to fills, in ticks.
    pub slippage: f64,
}

/// Running totals over the trades a broker has closed, folded in as each one
/// closes so reading them costs nothing per bar.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub gross_profit: f64,
    /// Total loss of the losing trades, as a positive magnitude.
    pub gross_loss: f64,
    pub wins: usize,
    pub losses: usize,
    pub evens: usize,
    /// Sums of each trade's percent return — over all trades, the winners and
    /// the losers — behind the average-trade-percent figures.
    pub pct_sum_all: f64,
    pub pct_sum_wins: f64,
    pub pct_sum_losses: f64,
    /// Largest peak-to-trough equity drop and trough-to-peak rise, in cash and
    /// as a percentage of the peak/trough — tracked separately, since the
    /// percentage extreme need not coincide with the cash one.
    pub max_drawdown: f64,
    pub max_runup: f64,
    pub max_drawdown_percent: f64,
    pub max_runup_percent: f64,
    /// Largest position (in contracts) ever held, overall and per side.
    pub max_contracts_all: f64,
    pub max_contracts_long: f64,
    pub max_contracts_short: f64,
    /// Account value at each bar's close, in bar order.
    pub equity: Vec<f64>,
    /// The last bar's close, at which open trades are valued.
    pub mark_price: f64,
}

impl Stats {
    pub fn record_close(&mut self, trade: &Trade) {
        let profit = trade.profit(0.0); // closed, so the price is ignored
        let basis = trade.entry_price * trade.size.abs();
        let ret = if basis != 0.0 {
            profit / basis * 100.0
        } else {
            0.0
        };
        self.pct_sum_all += ret;
        if profit > 0.0 {
            self.gross_profit += profit;
            self.wins += 1;
            self.pct_sum_wins += ret;
        } else if profit < 0.0 {
            self.gross_loss -= profit;
            self.losses += 1;
            self.pct_sum_losses += ret;
        } else {
            self.evens += 1;
        }
    }
}

/// Builds the [`Broker`] a `strategy` trades against. The default,
/// [`DefaultBrokerFactory`], produces the built-in bar-fill broker; a host can
/// supply its own to simulate against a different engine while still honouring
/// the script's [`BrokerConfig`].
pub trait BrokerFactory {
    fn build(&self, config: &BrokerConfig) -> Box<dyn Broker>;
}

/// The built-in factory: a [`BarBroker`] with [`PineFills`], reproducing Pine's
/// default fill model.
pub struct DefaultBrokerFactory;

impl BrokerFactory for DefaultBrokerFactory {
    fn build(&self, config: &BrokerConfig) -> Box<dyn Broker> {
        let fills = PineFills {
            slippage: config.slippage,
            mintick: config.mintick,
        };
        let mut broker = BarBroker::new(fills, config.initial_capital)
            .with_mintick(config.mintick)
            .with_sizing(config.sizing)
            .with_pyramiding(config.pyramiding);
        if let Some(commission) = config.commission {
            broker = broker.with_commission(commission);
        }
        Box::new(broker)
    }
}
