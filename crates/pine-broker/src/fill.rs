//! When a pending order fills against a bar, and at what price.
//!
//! This is the one modelling choice a backtest makes, so it is behind a trait:
//! a more pessimistic model (a limit fills only if price traded *through* it,
//! not merely touched) is a swap here, leaving the accounting untouched.

use crate::{Direction, Order, OrderKind};
use pine_core::Bar;

/// Decides whether `order` fills against `bar`, returning the fill price.
pub trait FillModel {
    fn fill(&self, order: &Order, bar: &Bar) -> Option<f64>;
}

/// TradingView's default assumptions:
///
/// - a market order fills at the bar's open;
/// - a limit fills if the bar's range reaches its price, at the better of the
///   limit and the open (a gap through the limit fills at the open);
/// - a stop fills if the bar's range reaches its price, at the worse of the stop
///   and the open (a gap through the stop fills at the open);
/// - slippage moves the fill `slippage` ticks against the order.
#[derive(Debug, Clone, Copy, Default)]
pub struct PineFills {
    /// Ticks of slippage applied against the order's direction.
    pub slippage: f64,
    /// Tick size, so slippage is a price. Zero disables slippage.
    pub mintick: f64,
}

impl PineFills {
    fn slip(&self, direction: Direction, price: f64) -> f64 {
        price + direction.sign() * self.slippage * self.mintick
    }
}

impl FillModel for PineFills {
    fn fill(&self, order: &Order, bar: &Bar) -> Option<f64> {
        let raw = match order.kind {
            OrderKind::Market => bar.open,

            // A buy limit sits at or below price and fills on a dip to it; a
            // sell limit sits above and fills on a rise. A gap past it fills at
            // the open, which is the better price.
            OrderKind::Limit(price) => {
                let reached = match order.direction {
                    Direction::Long => bar.low <= price,
                    Direction::Short => bar.high >= price,
                };
                if !reached {
                    return None;
                }
                match order.direction {
                    Direction::Long => bar.open.min(price),
                    Direction::Short => bar.open.max(price),
                }
            }

            // A buy stop sits above price and triggers on a rise; a sell stop
            // sits below. A gap past it fills at the open, the worse price.
            OrderKind::Stop(price) => {
                let reached = match order.direction {
                    Direction::Long => bar.high >= price,
                    Direction::Short => bar.low <= price,
                };
                if !reached {
                    return None;
                }
                match order.direction {
                    Direction::Long => bar.open.max(price),
                    Direction::Short => bar.open.min(price),
                }
            }

            // Once the stop is reached this bar, treat the armed limit as a
            // limit order for the rest of the same bar.
            OrderKind::StopLimit { stop, limit } => {
                let armed = match order.direction {
                    Direction::Long => bar.high >= stop,
                    Direction::Short => bar.low <= stop,
                };
                if !armed {
                    return None;
                }
                let reached = match order.direction {
                    Direction::Long => bar.low <= limit,
                    Direction::Short => bar.high >= limit,
                };
                if !reached {
                    return None;
                }
                limit
            }
        };

        Some(self.slip(order.direction, raw))
    }
}
