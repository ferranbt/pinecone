//! The outcome of replaying a `strategy`: its equity curve and trade log.

use pine_broker::Trade;

/// What a `strategy` produced over a run: the account's equity at each bar and
/// every trade it took. Present on a [`RunResult`](crate::RunResult) only when
/// the script declared a `strategy`; an indicator trades nothing.
#[derive(Debug, Clone, Default)]
pub struct Backtest {
    /// The capital the account started with, before any trade or commission.
    pub initial_capital: f64,
    /// Account value at each bar's close, from the strategy's declaration
    /// onward (normally one per bar, since a `strategy` is declared up front).
    pub equity: Vec<f64>,
    /// Every trade, closed ones first (in the order they closed) then those
    /// still open. A trade's `exit_price` is `None` while open; `profit(price)`
    /// values it at a given price.
    pub trades: Vec<Trade>,
    /// Realised profit of the closed trades (Pine's `strategy.netprofit`).
    pub net_profit: f64,
    /// Unrealised profit of the open position (Pine's `strategy.openprofit`).
    pub open_profit: f64,
    /// Total profit of the winning closed trades (`strategy.grossprofit`).
    pub gross_profit: f64,
    /// Total loss of the losing closed trades, as a positive magnitude
    /// (`strategy.grossloss`).
    pub gross_loss: f64,
    /// Largest peak-to-trough equity drop (`strategy.max_drawdown`).
    pub max_drawdown: f64,
    /// Largest trough-to-peak equity rise (`strategy.max_runup`).
    pub max_runup: f64,
    /// Closed-trade counts by outcome (`strategy.wintrades`/`losstrades`/
    /// `eventrades`).
    pub win_trades: usize,
    pub loss_trades: usize,
    pub even_trades: usize,
    /// Signed size of the final position: positive long, negative short.
    pub position_size: f64,
    /// The last bar's close, at which open trades are valued: pass it to
    /// [`Trade::profit`](pine_broker::Trade::profit).
    pub mark_price: f64,
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
