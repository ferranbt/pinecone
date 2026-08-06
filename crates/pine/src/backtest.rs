//! The outcome of replaying a `strategy`: its equity curve and trade log.

use pine_broker::Trade;

/// What a `strategy` produced over a run: the equity curve, the trade log, and
/// the summary values Pine exposes as `strategy.*`. Field names follow Pine's.
#[derive(Debug, Clone, Default)]
pub struct Backtest {
    pub initial_capital: f64,
    /// Account value at each bar's close.
    pub equity: Vec<f64>,
    /// Every trade, closed ones (in the order they closed) before still-open
    /// ones. `exit_price` is `None` while open; `profit(price)` values it.
    pub trades: Vec<Trade>,
    pub net_profit: f64,
    pub open_profit: f64,
    pub gross_profit: f64,
    /// Total loss of the losing trades, as a positive magnitude.
    pub gross_loss: f64,
    pub max_drawdown: f64,
    pub max_runup: f64,
    pub win_trades: usize,
    pub loss_trades: usize,
    pub even_trades: usize,
    /// Signed: positive long, negative short.
    pub position_size: f64,
    /// The last bar's close, at which open trades are valued.
    pub mark_price: f64,
    /// The bar the run halted on if a rest-of-run risk rule fired
    /// (`strategy.risk.max_drawdown` / `max_cons_loss_days`)
    pub halted: Option<u64>,
}

impl Backtest {
    /// The final account value, or the initial capital if no bar ran.
    pub fn final_equity(&self) -> f64 {
        self.equity.last().copied().unwrap_or(self.initial_capital)
    }

    /// The trades already closed, in the order they closed.
    pub fn closed_trades(&self) -> impl Iterator<Item = &Trade> {
        self.trades.iter().filter(|t| !t.is_open())
    }

    /// The trades still open at the end of the run.
    pub fn open_trades(&self) -> impl Iterator<Item = &Trade> {
        self.trades.iter().filter(|t| t.is_open())
    }
}
