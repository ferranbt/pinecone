//! The outcome of replaying a `strategy`: its equity curve and trade log.

use pine_broker::Trade;
use pine_core::Timeframe;
use serde::{Deserialize, Serialize};

/// Milliseconds in a 365-day year, for annualising a per-bar figure.
const MS_PER_YEAR: f64 = 365.0 * 24.0 * 60.0 * 60.0 * 1000.0;

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
    /// The chart timeframe, so per-bar figures can be annualised.
    pub timeframe: Timeframe,
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

    /// Standard summary metrics derived from the equity curve and trade log.
    pub fn generate_metrics(&self) -> Metrics {
        let trades = self.win_trades + self.loss_trades + self.even_trades;
        let final_equity = self.final_equity();
        let max_drawdown = max_drawdown_percent(&self.equity);

        // Annualisation from the bar length: how many bars a year holds, and how
        // many years this run's window spans.
        let bars_per_year = bars_per_year(&self.timeframe);
        let years = ratio(self.equity.len() as f64, bars_per_year);

        // Compounding back out of the window. Meaningless over a sub-bar window
        // or once equity has reached zero.
        let annual_return = if years > 0.0 && final_equity > 0.0 && self.initial_capital > 0.0 {
            (final_equity / self.initial_capital).powf(1.0 / years) - 1.0
        } else {
            0.0
        };

        let returns = bar_returns(&self.equity);
        let (mean, deviation) = mean_and_deviation(&returns);
        let annualise = bars_per_year.sqrt();

        Metrics {
            bars: self.equity.len(),
            initial_capital: self.initial_capital,
            final_equity,
            net_profit: self.net_profit,
            total_return: ratio(final_equity - self.initial_capital, self.initial_capital),
            annual_return,
            max_drawdown,
            sharpe: ratio(mean * annualise, deviation),
            sortino: ratio(mean * annualise, downside_deviation(&returns)),
            calmar: ratio(annual_return, max_drawdown),
            trades,
            wins: self.win_trades,
            losses: self.loss_trades,
            win_rate: ratio(self.win_trades as f64, trades as f64),
            profit_factor: ratio(self.gross_profit, self.gross_loss),
            avg_trade: ratio(self.net_profit, trades as f64),
            exposure: exposure(&self.trades, self.equity.len()),
        }
    }
}

/// Standard summary metrics of a run, from [`Backtest::generate_metrics`]. Every
/// figure is reported with the context that makes it comparable — returns beside
/// drawdown, wins beside profit factor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Bars the strategy ran over.
    pub bars: usize,
    pub initial_capital: f64,
    pub final_equity: f64,
    pub net_profit: f64,
    /// Total return over the run, as a fraction of starting capital.
    pub total_return: f64,
    /// Return annualised from the window's length.
    pub annual_return: f64,
    /// Largest peak-to-trough fall in close equity, as a fraction of the peak.
    pub max_drawdown: f64,
    /// Annualised mean return over its standard deviation.
    pub sharpe: f64,
    /// As Sharpe, but penalising only downside deviation.
    pub sortino: f64,
    /// Annual return over max drawdown: profit per unit of worst loss.
    pub calmar: f64,
    /// Closed trades — winners, losers and breakevens.
    pub trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    /// Gross profit over gross loss. Below 1.0 loses money.
    pub profit_factor: f64,
    /// Average profit per closed trade.
    pub avg_trade: f64,
    /// Fraction of bars holding a position; can exceed 1.0 with pyramiding.
    pub exposure: f64,
}

/// `numerator / denominator`, or 0 when the denominator can't divide. A summary
/// of nothing is zero, not an infinity that later sorts to the top of a ranking.
fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator > 0.0 && denominator.is_finite() {
        numerator / denominator
    } else {
        0.0
    }
}

/// Largest peak-to-trough fall in the equity curve, as a fraction of the peak it
/// fell from — the drawdown percentage TradingView reports. Measured on
/// bar-close equity; the cash `max_drawdown` field is the intrabar figure the
/// risk engine enforces against.
fn max_drawdown_percent(equity: &[f64]) -> f64 {
    let mut peak = f64::NEG_INFINITY;
    let mut worst = 0.0f64;
    for &value in equity {
        peak = peak.max(value);
        if peak > 0.0 {
            worst = worst.max((peak - value) / peak);
        }
    }
    worst
}

/// Fraction of `bars` spent holding a position, summing each trade's span.
fn exposure(trades: &[Trade], bars: usize) -> f64 {
    if bars == 0 {
        return 0.0;
    }
    let last = bars.saturating_sub(1) as u64;
    let held: u64 = trades
        .iter()
        .map(|t| t.exit_bar.unwrap_or(last).saturating_sub(t.entry_bar))
        .sum();
    held as f64 / bars as f64
}

/// Bars a year holds at `tf`'s length, or 0 for a timeframe with no fixed
/// duration — which zeroes the annualised figures rather than inventing one.
fn bars_per_year(tf: &Timeframe) -> f64 {
    match tf.to_millis() {
        Some(ms) if ms > 0 => MS_PER_YEAR / ms as f64,
        _ => 0.0,
    }
}

/// Simple return from each bar's close to the next.
fn bar_returns(equity: &[f64]) -> Vec<f64> {
    equity
        .windows(2)
        .filter(|pair| pair[0] > 0.0)
        .map(|pair| pair[1] / pair[0] - 1.0)
        .collect()
}

/// Mean and (population) standard deviation of `returns`.
fn mean_and_deviation(returns: &[f64]) -> (f64, f64) {
    if returns.is_empty() {
        return (0.0, 0.0);
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
    (mean, variance.sqrt())
}

/// Standard deviation of the losing bars only — upside volatility is not risk.
fn downside_deviation(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let sum: f64 = returns
        .iter()
        .filter(|r| **r < 0.0)
        .map(|r| r.powi(2))
        .sum();
    (sum / returns.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratios_over_nothing_are_zero() {
        // A flat run would otherwise sort to the top on an infinite profit factor.
        assert_eq!(ratio(1.0, 0.0), 0.0);
        assert_eq!(ratio(0.0, 0.0), 0.0);
    }

    #[test]
    fn max_drawdown_percent_is_measured_from_the_peak() {
        // Up to 200, down to 100: half the peak, not half the start.
        assert_eq!(max_drawdown_percent(&[100.0, 200.0, 100.0, 150.0]), 0.5);
        // A curve that only rises never draws down.
        assert_eq!(max_drawdown_percent(&[100.0, 110.0, 120.0]), 0.0);
    }

    #[test]
    fn generate_metrics_derives_the_summary() {
        let b = Backtest {
            initial_capital: 1000.0,
            equity: vec![1000.0, 1100.0, 1200.0],
            gross_profit: 200.0,
            gross_loss: 100.0,
            net_profit: 100.0,
            win_trades: 3,
            loss_trades: 1,
            even_trades: 0,
            ..Default::default()
        };
        let m = b.generate_metrics();

        assert_eq!(m.bars, 3);
        assert!((m.total_return - 0.2).abs() < 1e-12); // 1200 / 1000 - 1
        assert_eq!(m.profit_factor, 2.0);
        assert_eq!(m.win_rate, 0.75); // 3 of 4 closed
        assert_eq!(m.avg_trade, 25.0); // 100 over 4
        assert_eq!(m.trades, 4);
    }

    #[test]
    fn annualises_from_the_timeframe() {
        // 365 daily bars (the default timeframe) doubling equity = one year, so
        // roughly a 100% annual return, with a finite, positive Sharpe.
        let equity: Vec<f64> = (0..365)
            .map(|i| 1000.0 + 1000.0 * i as f64 / 364.0)
            .collect();
        let m = Backtest {
            initial_capital: 1000.0,
            equity,
            ..Default::default()
        }
        .generate_metrics();

        assert!((m.annual_return - 1.0).abs() < 1e-9);
        assert!(m.sharpe.is_finite() && m.sharpe > 0.0);
    }
}
