//! The accounting half of the simulator: order book, position, trades, equity.
//!
//! Venue-independent — the only pluggable part is the [`FillModel`]. Trades are
//! paired first-in-first-out: closing reduces the oldest open lots first, which
//! is what `strategy.closedtrades` reports.

use crate::{
    Broker, Commission, Direction, EntryFilter, Exit, FillModel, OcaType, Order, OrderKind,
    Position, RiskRule, RiskType, Sizing, Trade,
};
use pine_core::Bar;
use std::collections::HashMap;

pub struct BarBroker<F: FillModel> {
    fills: F,
    commission: Option<Commission>,
    /// How an order without an explicit `qty` is sized.
    sizing: Sizing,
    /// Maximum concurrent open entries in one direction (`pyramiding`).
    max_entries: usize,
    /// Tick size, so `strategy.exit` distances given in ticks become prices.
    mintick: f64,
    /// Starting capital, kept so `strategy.netprofit` can be derived from the
    /// equity identity.
    initial: f64,
    /// Cash balance; commission is charged here, price P&L in `realized`.
    cash: f64,
    realized: f64,

    /// Pending orders keyed by id, so a resubmission replaces rather than
    /// stacks — matching Pine's order commands.
    pending: HashMap<String, Order>,
    /// Insertion order of `pending`, so fills happen in submission order.
    order: Vec<String>,
    /// Exit brackets, evaluated each bar after the pending orders.
    exits: Vec<Exit>,

    open: Vec<Trade>,
    closed: Vec<Trade>,

    bar_index: u64,

    // Risk rules (`strategy.risk.*`), and the running state that enforces them.
    entry_filter: EntryFilter,
    max_position_size: Option<f64>,
    max_drawdown: Option<RiskType>,
    max_intraday_loss: Option<RiskType>,
    max_cons_loss_days: Option<u32>,
    max_intraday_filled_orders: Option<u32>,
    /// Highest equity seen over the whole run (for `max_drawdown`).
    peak_equity: f64,
    /// The current trading day, as a UTC day bucket of `bar.time`.
    day: Option<i64>,
    /// Equity entering the current day (for the daily loss/win verdict).
    day_start_equity: f64,
    /// Highest equity seen so far today (for `max_intraday_loss`).
    intraday_peak: f64,
    /// The previous bar's close equity — the day's final value at a rollover.
    last_equity: f64,
    /// Orders filled so far today (for `max_intraday_filled_orders`).
    filled_today: u32,
    consecutive_loss_days: u32,
    /// Halted for the rest of the run (`max_drawdown`, `max_cons_loss_days`).
    halted: bool,
    /// The bar the run halted on, once a rest-of-run rule fired.
    halted_bar: Option<u64>,
    /// Halted for the rest of the day (`max_intraday_loss`).
    halted_today: bool,
}

impl<F: FillModel> BarBroker<F> {
    pub fn new(fills: F, initial_capital: f64) -> Self {
        Self {
            fills,
            commission: None,
            sizing: Sizing::Contracts(1.0),
            max_entries: 1,
            mintick: 0.0,
            initial: initial_capital,
            cash: initial_capital,
            realized: 0.0,
            pending: HashMap::new(),
            order: Vec::new(),
            exits: Vec::new(),
            open: Vec::new(),
            closed: Vec::new(),
            bar_index: 0,
            entry_filter: EntryFilter::All,
            max_position_size: None,
            max_drawdown: None,
            max_intraday_loss: None,
            max_cons_loss_days: None,
            max_intraday_filled_orders: None,
            peak_equity: initial_capital,
            day: None,
            day_start_equity: initial_capital,
            intraday_peak: initial_capital,
            last_equity: initial_capital,
            filled_today: 0,
            consecutive_loss_days: 0,
            halted: false,
            halted_bar: None,
            halted_today: false,
        }
    }

    pub fn with_commission(mut self, commission: Commission) -> Self {
        self.commission = Some(commission);
        self
    }

    pub fn with_sizing(mut self, sizing: Sizing) -> Self {
        self.sizing = sizing;
        self
    }

    pub fn with_mintick(mut self, mintick: f64) -> Self {
        self.mintick = mintick;
        self
    }

    pub fn with_pyramiding(mut self, pyramiding: usize) -> Self {
        self.max_entries = pyramiding.max(1);
        self
    }

    fn open_lots_toward(&self, direction: Direction) -> usize {
        self.open
            .iter()
            .filter(|t| t.size.signum() == direction.sign())
            .count()
    }

    fn net_size(&self) -> f64 {
        self.open.iter().map(|t| t.size).sum()
    }

    /// Net signed size of the lots matching `target` (all lots if `None`).
    fn matched_size(&self, target: Option<&str>) -> f64 {
        self.open
            .iter()
            .filter(|t| target.is_none_or(|id| t.entry_id == id))
            .map(|t| t.size)
            .sum()
    }

    /// Average entry price of the lots matching `target`, weighted by size.
    fn matched_avg(&self, target: Option<&str>) -> f64 {
        let (value, qty): (f64, f64) = self
            .open
            .iter()
            .filter(|t| target.is_none_or(|id| t.entry_id == id))
            .fold((0.0, 0.0), |(v, q), t| {
                (v + t.entry_price * t.size, q + t.size)
            });
        if qty == 0.0 {
            0.0
        } else {
            value / qty
        }
    }

    fn commission_on(&self, qty: f64, price: f64) -> f64 {
        self.commission.map_or(0.0, |c| c.charge(qty, price))
    }

    /// Apply a fill of `signed_qty` contracts at `price`: close opposing lots
    /// first (FIFO), then open a lot with whatever direction remains. `target`
    /// restricts which lots may be closed to those from that entry — a reducing
    /// order leaves the remainder unopened, so it only ever shrinks them.
    fn apply_fill(&mut self, mut signed_qty: f64, price: f64, id: &str, target: Option<&str>) {
        // This fill's commission, split across the portions it closes and opens
        // by contract count, so each closed trade carries its exit commission
        // and each opened lot its entry commission.
        let order_qty_abs = signed_qty.abs();
        let order_commission = self.commission_on(signed_qty, price);
        self.cash -= order_commission;

        // Close opposing open lots, oldest first. A partial close records a
        // closed trade for the exited portion and leaves the rest open, as Pine
        // does, so `strategy.closedtrades` counts partial exits too.
        while signed_qty != 0.0 {
            let Some(index) = self.open.iter().position(|t| {
                t.size.signum() != signed_qty.signum()
                    && target.is_none_or(|want| t.entry_id == want)
            }) else {
                break;
            };

            let lot = &self.open[index];
            let closed = signed_qty.abs().min(lot.size.abs());
            let closed_signed = closed * lot.size.signum();
            let entry_share = lot.commission * closed / lot.size.abs();
            let exit_share = order_commission * closed / order_qty_abs;

            self.realized += (price - lot.entry_price) * closed_signed;
            signed_qty += closed_signed; // moves signed_qty toward zero

            self.closed.push(Trade {
                entry_id: lot.entry_id.clone(),
                size: closed_signed,
                entry_price: lot.entry_price,
                entry_bar: lot.entry_bar,
                exit_price: Some(price),
                exit_bar: Some(self.bar_index),
                commission: entry_share + exit_share,
            });

            let lot = &mut self.open[index];
            lot.size -= closed_signed;
            lot.commission -= entry_share;
            if lot.size == 0.0 {
                self.open.remove(index);
            }
        }

        // Whatever quantity is left opens a new lot — but only for an entry. A
        // targeted reduce never flips into a new position, so it stops here.
        if signed_qty != 0.0 && target.is_none() {
            self.open.push(Trade {
                entry_id: id.to_string(),
                size: signed_qty,
                entry_price: price,
                entry_bar: self.bar_index,
                exit_price: None,
                exit_bar: None,
                commission: order_commission * signed_qty.abs() / order_qty_abs,
            });
        }
    }

    /// The signed quantity an order actually trades at `price`, resolving the
    /// default quantity and, for a reducing or reversing order, the position.
    fn resolve_qty(&self, order: &Order, price: f64) -> f64 {
        if order.reduce_only {
            // Never flips: close at most the matched position. An explicit qty
            // wins; otherwise `qty_percent` closes that share, and with neither
            // `strategy.close` shuts the whole position.
            let pool = self.matched_size(order.close_target.as_deref());
            let closable = match (order.qty, order.qty_percent) {
                (Some(q), _) => pool.abs().min(q.abs()),
                (None, Some(pct)) => pool.abs() * (pct / 100.0),
                (None, None) => pool.abs(),
            };
            return -pool.signum() * closable;
        }

        let requested = match order.qty {
            Some(q) => q.abs(),
            None => {
                // Pine sizes a default-qty order from the close of the bar it
                // was generated on; fall back to the fill price if unstamped.
                let sizing_price = order.sizing_price.unwrap_or(price);
                self.sizing
                    .contracts(sizing_price, self.equity(sizing_price))
            }
        };
        let net = self.net_size();
        let want = order.direction.sign() * requested;
        if order.reverses && net != 0.0 && net.signum() != order.direction.sign() {
            // Close the opposite position and open `requested` the other way.
            want - net
        } else {
            want
        }
    }

    /// Evaluate every exit bracket against `bar`: for a matched position, fill
    /// the stop-loss, trailing stop or take-profit if the bar reaches it (a stop
    /// wins when several do, the conservative assumption), then retire it.
    fn evaluate_exits(&mut self, bar: &Bar) {
        let ids: Vec<String> = self.exits.iter().map(|e| e.id.clone()).collect();
        for id in ids {
            let Some(exit) = self.exits.iter().find(|e| e.id == id).cloned() else {
                continue;
            };
            let target = exit.from_entry.as_deref();
            let pos = self.matched_size(target);
            if pos == 0.0 {
                continue; // Nothing to protect yet (the entry has not filled).
            }
            let dir = pos.signum();
            let entry_avg = self.matched_avg(target);
            let mintick = self.mintick;
            let exit_dir = if dir > 0.0 {
                Direction::Short
            } else {
                Direction::Long
            };

            // Take-profit and stop-loss prices, from an explicit level or a tick
            // distance either side of the entry.
            let tp = exit
                .limit
                .or_else(|| exit.profit_ticks.map(|t| entry_avg + dir * t * mintick));
            let sl = exit
                .stop
                .or_else(|| exit.loss_ticks.map(|t| entry_avg - dir * t * mintick));

            // Arm and advance the trailing stop with this bar: the reference
            // trails "each time the trade's profit reaches a new high", so it
            // follows the peak within the bar and can fill the same one.
            let trail_stop = self.advance_trail(&id, dir, entry_avg, bar);

            // A stop wins over the take-profit when a bar reaches both. The
            // trailing stop fills at its level — price set the peak this bar,
            // then retraced to the stop.
            let hit = sl
                .and_then(|p| self.leg_fill(OrderKind::Stop(p), exit_dir, bar))
                .or_else(|| {
                    trail_stop.filter(|&ts| {
                        if dir > 0.0 {
                            bar.low <= ts
                        } else {
                            bar.high >= ts
                        }
                    })
                })
                .or_else(|| tp.and_then(|p| self.leg_fill(OrderKind::Limit(p), exit_dir, bar)));

            if let Some(price) = hit {
                let requested = match (exit.qty, exit.qty_percent) {
                    (Some(q), _) => pos.abs().min(q.abs()),
                    (None, Some(pct)) => pos.abs() * (pct / 100.0),
                    (None, None) => pos.abs(),
                };
                self.apply_fill(-dir * requested, price, &exit.id, target);
                self.exits.retain(|e| e.id != id);
            }
        }
    }

    /// Arm a trailing exit and advance its peak from `bar`, returning the stop
    /// price if it is active — `trail_offset` ticks behind the best price seen.
    fn advance_trail(&mut self, id: &str, dir: f64, entry_avg: f64, bar: &Bar) -> Option<f64> {
        let mintick = self.mintick;
        let exit = self.exits.iter_mut().find(|e| e.id == id)?;
        let offset = exit.trail_offset?;
        let bar_best = if dir > 0.0 { bar.high } else { bar.low };

        if !exit.activated {
            let level = exit
                .trail_price
                .or_else(|| exit.trail_points.map(|pts| entry_avg + dir * pts * mintick));
            if let Some(level) = level {
                exit.activated = if dir > 0.0 {
                    bar.high >= level
                } else {
                    bar.low <= level
                };
            }
        }
        if !exit.activated {
            return None;
        }

        exit.peak = Some(match exit.peak {
            Some(pk) if dir > 0.0 => pk.max(bar_best),
            Some(pk) => pk.min(bar_best),
            None => bar_best,
        });
        exit.peak.map(|pk| pk - dir * offset * mintick)
    }

    /// The fill price of one exit leg against `bar`, or `None` if unreached.
    fn leg_fill(&self, kind: OrderKind, direction: Direction, bar: &Bar) -> Option<f64> {
        let leg = Order {
            kind,
            ..Order::market("", direction, None)
        };
        self.fills.fill(&leg, bar)
    }

    /// Whether an entry order is blocked by the pyramiding limit: it would add a
    /// new lot to an already-full stack on its own side.
    fn pyramiding_blocks(&self, order: &Order) -> bool {
        if order.reduce_only || !order.reverses {
            return false; // Only `strategy.entry` obeys pyramiding.
        }
        let net = self.net_size();
        let same_side = net != 0.0 && net.signum() == order.direction.sign();
        same_side && self.open_lots_toward(order.direction) >= self.max_entries
    }

    /// Apply an OCA group's effect after `filled` executes: cancel the group's
    /// other unfilled orders, or reduce them by the filled size.
    fn apply_oca(&mut self, filled: &Order, filled_qty: f64) {
        let Some(group) = filled.oca_name.clone() else {
            return;
        };
        if filled.oca_type == OcaType::None {
            return;
        }
        let siblings: Vec<String> = self
            .pending
            .values()
            .filter(|o| o.id != filled.id && o.oca_name.as_deref() == Some(group.as_str()))
            .map(|o| o.id.clone())
            .collect();
        for id in siblings {
            match filled.oca_type {
                OcaType::Cancel => {
                    self.pending.remove(&id);
                    self.order.retain(|o| o != &id);
                }
                OcaType::Reduce => {
                    if let Some(o) = self.pending.get_mut(&id) {
                        // Shrink by the filled size; a non-positive remainder
                        // cancels the order outright.
                        let base = o.qty.unwrap_or(filled_qty.abs());
                        let left = base - filled_qty.abs();
                        if left > 0.0 {
                            o.qty = Some(left);
                        } else {
                            self.pending.remove(&id);
                            self.order.retain(|o| o != &id);
                        }
                    }
                }
                OcaType::None => {}
            }
        }
    }

    /// Whether a risk rule rejects `order` outright at submission. Exits and
    /// reduce-only orders always pass — a rule may stop new exposure but never
    /// traps an open position.
    fn risk_rejects(&self, order: &Order) -> bool {
        if order.reduce_only {
            return false;
        }
        if self.halted || self.halted_today {
            return true;
        }
        !self.entry_filter.allows(order.direction)
        // The daily fill cap is enforced per fill in `advance`, not here — an
        // order under the cap at submission may still be over it by the time it
        // fills.
    }

    /// Halt the strategy for the rest of the run, recording the bar it happened
    /// on (a rest-of-run risk rule fired).
    fn halt(&mut self) {
        self.halted = true;
        self.halted_bar = Some(self.bar_index);
    }

    /// Roll intraday state when `time` lands on a new UTC day, and settle the day
    /// that just ended for `max_cons_loss_days`.
    fn roll_day(&mut self, time: i64) {
        // TODO: this buckets by the UTC calendar day. TradingView rolls the
        // trading day at the exchange session start in `syminfo.timezone`, so the
        // intraday rules (max_intraday_loss / max_intraday_filled_orders and the
        // per-day P&L behind max_cons_loss_days) diverge for sub-daily
        // equity/futures. Correct once the broker is given the symbol's timezone
        // and session; UTC is exact for 24/7 (crypto) symbols.
        let bucket = time.div_euclid(86_400_000);
        match self.day {
            Some(current) if current == bucket => return,
            Some(_) => {
                // The day just ended: a losing day advances the streak, a
                // non-losing one resets it.
                if let Some(limit) = self.max_cons_loss_days {
                    if self.last_equity < self.day_start_equity {
                        self.consecutive_loss_days += 1;
                        if self.consecutive_loss_days >= limit {
                            self.halt();
                        }
                    } else {
                        self.consecutive_loss_days = 0;
                    }
                }
            }
            None => {}
        }
        // Start the new day from the equity carried across the boundary.
        self.day = Some(bucket);
        self.day_start_equity = self.last_equity;
        self.intraday_peak = self.last_equity;
        self.filled_today = 0;
        self.halted_today = false;
    }

    /// Reduce an entry `qty` so the resulting position stays within
    /// `max_position_size`; returns 0 when even the smallest step would exceed it
    /// (Pine then places nothing).
    fn clamp_to_max_position(&self, order: &Order, qty: f64) -> f64 {
        let Some(max) = self.max_position_size else {
            return qty;
        };
        if order.reduce_only {
            return qty;
        }
        let after = self.position().size + qty;
        if after.abs() <= max {
            return qty;
        }
        // Allow only up to `max` in the resulting direction; if that flips the
        // order's sign, the position is already at the cap — place nothing.
        let clamped = max * after.signum() - self.position().size;
        if clamped == 0.0 || clamped.signum() != qty.signum() {
            0.0
        } else {
            clamped
        }
    }

    /// Close the whole position at `price` — the forced exit a breached drawdown
    /// or intraday-loss rule performs.
    fn flatten(&mut self, price: f64) {
        let size = self.position().size;
        if size != 0.0 {
            self.apply_fill(-size, price, "risk_flatten", None);
        }
    }

    /// Mark equity at the bar's close, update the peaks, and enforce the
    /// equity-drop rules — cancelling and flattening on a breach.
    fn mark_and_check_risk(&mut self, bar: &Bar) {
        let equity = self.equity(bar.close);
        self.peak_equity = self.peak_equity.max(equity);
        self.intraday_peak = self.intraday_peak.max(equity);

        // Drawdown is measured to the bar's adverse intrabar extreme (equity
        // marked at the high and the low), so the rule agrees with the reported
        // `strategy.max_drawdown`. The peak still tracks close equity, so an
        // intrabar swing does not move the mark.
        let intrabar_low = self.equity(bar.high).min(self.equity(bar.low));

        if let Some(rule) = self.max_drawdown {
            if !self.halted && self.peak_equity - intrabar_low >= rule.threshold(self.peak_equity) {
                self.cancel_all();
                self.flatten(bar.close);
                self.halt();
            }
        }
        if let Some(rule) = self.max_intraday_loss {
            if !self.halted_today
                && self.intraday_peak - intrabar_low >= rule.threshold(self.intraday_peak)
            {
                self.cancel_all();
                self.flatten(bar.close);
                self.halted_today = true;
            }
        }

        // Recompute after a possible flatten, so the day P&L and next mark start
        // from the settled equity.
        self.last_equity = self.equity(bar.close);
    }
}

impl<F: FillModel> Broker for BarBroker<F> {
    fn submit(&mut self, order: Order) {
        if self.risk_rejects(&order) {
            return;
        }
        if !self.pending.contains_key(&order.id) {
            self.order.push(order.id.clone());
        }
        self.pending.insert(order.id.clone(), order);
    }

    fn set_risk(&mut self, rule: RiskRule) {
        match rule {
            RiskRule::AllowEntryIn(filter) => self.entry_filter = filter,
            RiskRule::MaxPositionSize(size) => self.max_position_size = Some(size.abs()),
            RiskRule::MaxDrawdown(threshold) => self.max_drawdown = Some(threshold),
            RiskRule::MaxIntradayLoss(threshold) => self.max_intraday_loss = Some(threshold),
            RiskRule::MaxConsLossDays(days) => self.max_cons_loss_days = Some(days),
            RiskRule::MaxIntradayFilledOrders(count) => {
                self.max_intraday_filled_orders = Some(count)
            }
        }
    }

    fn submit_exit(&mut self, mut exit: Exit) {
        if let Some(slot) = self.exits.iter_mut().find(|e| e.id == exit.id) {
            // Re-submitting the same exit each bar must not restart a trailing
            // stop, so carry its runtime state onto the replacement.
            exit.activated = slot.activated;
            exit.peak = slot.peak;
            *slot = exit;
        } else {
            self.exits.push(exit);
        }
    }

    fn cancel(&mut self, id: &str) {
        if self.pending.remove(id).is_some() {
            self.order.retain(|o| o != id);
        }
        self.exits.retain(|e| e.id != id);
    }

    fn cancel_all(&mut self) {
        self.pending.clear();
        self.order.clear();
        self.exits.clear();
    }

    fn advance(&mut self, bar: &Bar) {
        self.bar_index = bar.index;
        self.roll_day(bar.time);

        // Fill in submission order; a filled order leaves the book.
        let ids: Vec<String> = self.order.clone();
        for id in ids {
            let Some(order) = self.pending.get(&id).cloned() else {
                continue;
            };
            // While halted (for the run or the day), drop new entries; a
            // reduce-only exit still fills so an open position can be closed.
            if (self.halted || self.halted_today) && !order.reduce_only {
                self.pending.remove(&id);
                self.order.retain(|o| o != &id);
                continue;
            }
            if self.pyramiding_blocks(&order) {
                // The stack is full: drop the entry, as Pine rejects it.
                self.pending.remove(&id);
                self.order.retain(|o| o != &id);
                continue;
            }
            // Once the day's fill cap is reached, no more orders fill — except a
            // reduce-only exit of the current position. Enforced here, per fill,
            // so orders already pending when the bar opens are capped too.
            if !order.reduce_only {
                if let Some(cap) = self.max_intraday_filled_orders {
                    if self.filled_today >= cap {
                        self.pending.remove(&id);
                        self.order.retain(|o| o != &id);
                        continue;
                    }
                }
            }
            if let Some(price) = self.fills.fill(&order, bar) {
                let qty = self.clamp_to_max_position(&order, self.resolve_qty(&order, price));
                if qty != 0.0 {
                    self.apply_fill(qty, price, &order.id, order.close_target.as_deref());
                    self.apply_oca(&order, qty);
                    self.filled_today += 1;
                }
                self.pending.remove(&id);
                self.order.retain(|o| o != &id);
            }
        }

        // Then the protective exits, against the position those fills produced.
        self.evaluate_exits(bar);

        // Finally settle equity for the bar and enforce the equity-drop rules.
        self.mark_and_check_risk(bar);
    }

    fn position(&self) -> Position {
        let size = self.net_size();
        if size == 0.0 {
            return Position::default();
        }
        // Average price weighted over the open lots on the net side.
        let (value, qty): (f64, f64) = self
            .open
            .iter()
            .filter(|t| t.size.signum() == size.signum())
            .fold((0.0, 0.0), |(v, q), t| {
                (v + t.entry_price * t.size, q + t.size)
            });
        Position {
            size,
            avg_price: if qty == 0.0 { 0.0 } else { value / qty },
        }
    }

    fn initial_capital(&self) -> f64 {
        self.initial
    }

    fn equity(&self, price: f64) -> f64 {
        let unrealized: f64 = self
            .open
            .iter()
            .map(|t| (price - t.entry_price) * t.size)
            .sum();
        self.cash + self.realized + unrealized
    }

    fn open_trades(&self) -> Vec<&Trade> {
        self.open.iter().collect()
    }

    fn closed_trades(&self) -> &[Trade] {
        &self.closed
    }

    fn halted_bar(&self) -> Option<u64> {
        self.halted_bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Commission, Direction, OrderKind, PineFills};

    fn bar(index: u64, open: f64, high: f64, low: f64, close: f64) -> Bar {
        Bar {
            open,
            high,
            low,
            close,
            volume: 0.0,
            index,
            ..Bar::default()
        }
    }

    fn broker() -> BarBroker<PineFills> {
        BarBroker::new(PineFills::default(), 10_000.0)
    }

    /// A bar carrying a `time`, so day-boundary rules can be exercised.
    fn bar_at(index: u64, time: i64, open: f64, high: f64, low: f64, close: f64) -> Bar {
        Bar {
            time,
            ..bar(index, open, high, low, close)
        }
    }

    const DAY: i64 = 86_400_000;

    #[test]
    fn allow_entry_in_blocks_the_disallowed_direction() {
        let mut b = broker();
        b.set_risk(RiskRule::AllowEntryIn(EntryFilter::LongOnly));
        b.submit(Order::market("s", Direction::Short, Some(1.0))); // rejected
        b.submit(Order::market("l", Direction::Long, Some(1.0))); // allowed
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));
        assert_eq!(b.position().size, 1.0);
    }

    #[test]
    fn max_position_size_caps_the_entry() {
        let mut b = broker();
        b.set_risk(RiskRule::MaxPositionSize(3.0));
        b.submit(Order::market("l", Direction::Long, Some(10.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));
        assert_eq!(b.position().size, 3.0);
    }

    #[test]
    fn max_drawdown_flattens_and_halts() {
        let mut b = broker();
        b.set_risk(RiskRule::MaxDrawdown(RiskType::Cash(500.0)));
        b.submit(Order::market("l", Direction::Long, Some(100.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0)); // equity 10_000 (peak)
        b.advance(&bar(1, 100.0, 100.0, 90.0, 90.0)); // −1_000 > 500 → flatten + halt
        assert!(b.position().is_flat());
        assert_eq!(b.halted_bar(), Some(1));

        // A new entry after the halt is rejected for the rest of the run.
        b.submit(Order::market("l2", Direction::Long, Some(1.0)));
        b.advance(&bar(2, 90.0, 90.0, 90.0, 90.0));
        assert!(b.position().is_flat());
    }

    #[test]
    fn max_drawdown_measures_the_intrabar_low() {
        let mut b = broker();
        b.set_risk(RiskRule::MaxDrawdown(RiskType::Cash(500.0)));
        b.submit(Order::market("l", Direction::Long, Some(100.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0)); // long 100 @ 100, peak 10_000

        // The close recovers to 96 (−400, within the cap) but the low hit 90
        // (−1_000): the intrabar drawdown trips the rule the close alone would not.
        b.advance(&bar(1, 100.0, 100.0, 90.0, 96.0));
        assert!(b.position().is_flat());
    }

    #[test]
    fn max_intraday_filled_orders_resets_next_day() {
        let mut b = broker();
        b.set_risk(RiskRule::MaxIntradayFilledOrders(1));

        b.submit(Order::market("a", Direction::Long, Some(1.0)));
        b.advance(&bar_at(0, 0, 100.0, 100.0, 100.0, 100.0)); // fills → 1/1
                                                              // A second order the same day is blocked by the cap (a reversal, so
                                                              // pyramiding is not what stops it).
        b.submit(Order::market("b", Direction::Short, Some(2.0)));
        b.advance(&bar_at(1, 1_000, 100.0, 100.0, 100.0, 100.0));
        assert_eq!(b.position().size, 1.0);

        // The next day resets the count, so a fresh order fills.
        b.advance(&bar_at(2, DAY, 100.0, 100.0, 100.0, 100.0));
        b.submit(Order::market("c", Direction::Short, Some(1.0)));
        b.advance(&bar_at(3, DAY + 1_000, 100.0, 100.0, 100.0, 100.0));
        assert_eq!(b.position().size, -1.0);
    }

    #[test]
    fn max_intraday_filled_orders_caps_already_pending_orders() {
        let mut b = broker();
        b.set_risk(RiskRule::MaxIntradayFilledOrders(1));

        // Both are under the cap at submission and both are pending when the bar
        // opens; only one may fill, since the cap is enforced per fill.
        b.submit(Order::market("a", Direction::Long, Some(1.0)));
        b.submit(Order::market("b", Direction::Short, Some(3.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));

        // "a" filled (long 1); "b" was blocked, not reversing the position.
        assert_eq!(b.position().size, 1.0);
    }

    #[test]
    fn max_cons_loss_days_halts_after_two_losing_days() {
        let mut b = broker();
        b.set_risk(RiskRule::MaxConsLossDays(2));

        // A long carried across three down-closing days.
        b.submit(Order::market("l", Direction::Long, Some(10.0)));
        b.advance(&bar_at(0, 0, 100.0, 100.0, 100.0, 99.0)); // day 0 ends at a loss
        b.advance(&bar_at(1, DAY, 99.0, 99.0, 98.0, 98.0)); // rolls day 0 → streak 1
        b.advance(&bar_at(2, 2 * DAY, 98.0, 98.0, 97.0, 97.0)); // rolls day 1 → streak 2 → halt

        // Halted: a reversing entry is rejected, so the position is unchanged.
        b.submit(Order::market("rev", Direction::Short, Some(20.0)));
        b.advance(&bar_at(3, 2 * DAY + 1_000, 97.0, 97.0, 97.0, 97.0));
        assert_eq!(b.position().size, 10.0);
    }

    #[test]
    fn a_market_entry_fills_at_the_open() {
        let mut b = broker();
        b.submit(Order::market("long", Direction::Long, Some(2.0)));
        b.advance(&bar(0, 100.0, 105.0, 99.0, 104.0));

        let pos = b.position();
        assert_eq!(pos.size, 2.0);
        assert_eq!(pos.avg_price, 100.0);
        // Marked at 104: two contracts up 4 each.
        assert_eq!(b.equity(104.0), 10_008.0);
    }

    #[test]
    fn a_closed_trade_keeps_its_size_and_profit() {
        let mut b = broker();
        b.submit(Order::market("L", Direction::Long, Some(2.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));
        b.submit(Order {
            reduce_only: true,
            ..Order::market("L", Direction::Short, None)
        });
        b.advance(&bar(1, 110.0, 110.0, 110.0, 110.0));

        let trade = &b.closed_trades()[0];
        assert_eq!(trade.size, 2.0);
        assert_eq!(trade.entry_price, 100.0);
        assert_eq!(trade.exit_price, Some(110.0));
        assert_eq!(trade.profit(0.0), 20.0); // (110 - 100) * 2, price ignored once closed
    }

    #[test]
    fn closing_realises_profit_and_flattens() {
        let mut b = broker();
        b.submit(Order::market("long", Direction::Long, Some(1.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));

        b.submit(Order {
            reduce_only: true,
            ..Order::market("exit", Direction::Short, Some(1.0))
        });
        b.advance(&bar(1, 110.0, 110.0, 110.0, 110.0));

        assert!(b.position().is_flat());
        assert_eq!(b.closed_trades().len(), 1);
        assert_eq!(b.equity(110.0), 10_010.0);
    }

    #[test]
    fn an_opposite_entry_reverses_the_position() {
        let mut b = broker();
        b.submit(Order::market("a", Direction::Long, Some(5.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));

        // Short 5 against long 5 sells 10: closes the long and opens short 5.
        b.submit(Order::market("b", Direction::Short, Some(5.0)));
        b.advance(&bar(1, 100.0, 100.0, 100.0, 100.0));

        assert_eq!(b.position().size, -5.0);
        assert_eq!(b.closed_trades().len(), 1);
    }

    #[test]
    fn a_buy_limit_waits_for_the_price() {
        let mut b = broker();
        b.submit(Order {
            kind: OrderKind::Limit(95.0),
            reverses: false,
            ..Order::market("buy", Direction::Long, Some(1.0))
        });

        // Bar stays above 95: no fill.
        b.advance(&bar(0, 100.0, 101.0, 96.0, 99.0));
        assert!(b.position().is_flat());

        // Next bar dips to 94: fills at the limit.
        b.advance(&bar(1, 97.0, 98.0, 94.0, 96.0));
        assert_eq!(b.position().size, 1.0);
        assert_eq!(b.position().avg_price, 95.0);
    }

    #[test]
    fn commission_reduces_equity() {
        let mut b = broker().with_commission(Commission::Percent(1.0));
        b.submit(Order::market("long", Direction::Long, Some(1.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));

        // 1% of 100 = 1.0 charged on entry.
        assert_eq!(b.equity(100.0), 9_999.0);
    }

    #[test]
    fn a_take_profit_exit_closes_when_price_reaches_it() {
        let mut b = broker();
        b.submit(Order::market("L", Direction::Long, Some(1.0)));
        b.submit_exit(Exit {
            limit: Some(110.0),
            ..Exit::resting("X", Some("L".into()), None, None)
        });

        // Entry fills at 100; this bar's high 105 does not reach 110.
        b.advance(&bar(0, 100.0, 105.0, 99.0, 104.0));
        assert_eq!(b.position().size, 1.0);

        // Next bar reaches 110: the take-profit sells at 110.
        b.advance(&bar(1, 106.0, 112.0, 105.0, 108.0));
        assert!(b.position().is_flat());
        assert_eq!(b.closed_trades().len(), 1);
        assert_eq!(b.equity(108.0), 10_010.0); // realised +10
    }

    #[test]
    fn a_stop_loss_in_ticks_sits_a_distance_from_the_entry() {
        // mintick 0.5, loss 4 ticks -> stop 2.0 below a long entry.
        let fills = PineFills {
            slippage: 0.0,
            mintick: 0.5,
        };
        let mut b = BarBroker::new(fills, 10_000.0).with_mintick(0.5);
        b.submit(Order::market("L", Direction::Long, Some(1.0)));
        b.submit_exit(Exit {
            loss_ticks: Some(4.0),
            ..Exit::resting("X", Some("L".into()), None, None)
        });

        // Entry fills at 100; stop is 98. This bar's low 99 stays above it.
        b.advance(&bar(0, 100.0, 105.0, 99.0, 104.0));
        assert_eq!(b.position().size, 1.0);

        // Next bar dips to 97: the stop sells at 98.
        b.advance(&bar(1, 100.0, 101.0, 97.0, 99.0));
        assert!(b.position().is_flat());
        assert_eq!(b.equity(99.0), 9_998.0); // realised -2
    }

    #[test]
    fn close_targets_only_the_named_entry() {
        let mut b = broker();
        b.submit(Order {
            reverses: false,
            ..Order::market("A", Direction::Long, Some(1.0))
        });
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));
        b.submit(Order {
            reverses: false,
            ..Order::market("B", Direction::Long, Some(1.0))
        });
        b.advance(&bar(1, 101.0, 101.0, 101.0, 101.0));
        assert_eq!(b.position().size, 2.0);

        // Close only A: its lot goes, B's remains.
        b.submit(Order {
            reduce_only: true,
            close_target: Some("A".into()),
            qty: None,
            ..Order::market("A", Direction::Long, None)
        });
        b.advance(&bar(2, 102.0, 102.0, 102.0, 102.0));
        assert_eq!(b.position().size, 1.0);
        assert_eq!(b.closed_trades().len(), 1);
        assert_eq!(b.position().avg_price, 101.0); // B's entry
    }

    #[test]
    fn cash_sizing_buys_contracts_worth_the_cash() {
        let mut b = broker().with_sizing(Sizing::Cash(1_000.0));
        // No explicit qty: 1000 cash / 100 price = 10 contracts.
        b.submit(Order::market("L", Direction::Long, None));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));
        assert_eq!(b.position().size, 10.0);
    }

    #[test]
    fn percent_of_equity_sizing_scales_with_the_account() {
        let mut b = broker().with_sizing(Sizing::PercentOfEquity(50.0));
        // 50% of 10000 equity = 5000, at price 100 = 50 contracts.
        b.submit(Order::market("L", Direction::Long, None));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));
        assert_eq!(b.position().size, 50.0);
    }

    #[test]
    fn pyramiding_caps_entries_in_one_direction() {
        let mut b = broker().with_pyramiding(2);
        for (i, id) in ["A", "B", "C"].iter().enumerate() {
            b.submit(Order {
                reverses: true,
                ..Order::market(*id, Direction::Long, Some(1.0))
            });
            b.advance(&bar(i as u64, 100.0, 100.0, 100.0, 100.0));
        }
        // Two lots allowed; the third entry is rejected.
        assert_eq!(b.position().size, 2.0);
    }

    #[test]
    fn oca_cancel_removes_the_sibling_when_one_fills() {
        let mut b = broker();
        // A buy stop at 105 and a buy limit at 95, same OCA group.
        b.submit(Order {
            kind: OrderKind::Stop(105.0),
            oca_name: Some("G".into()),
            oca_type: OcaType::Cancel,
            ..Order::market("up", Direction::Long, Some(1.0))
        });
        b.submit(Order {
            kind: OrderKind::Limit(95.0),
            oca_name: Some("G".into()),
            oca_type: OcaType::Cancel,
            ..Order::market("down", Direction::Long, Some(1.0))
        });

        // This bar reaches both 105 and 95; the first to fill cancels the other.
        b.advance(&bar(0, 100.0, 106.0, 94.0, 100.0));
        assert_eq!(b.position().size, 1.0);
    }

    #[test]
    fn close_qty_percent_reduces_the_position() {
        let mut b = broker();
        b.submit(Order::market("L", Direction::Long, Some(4.0)));
        b.advance(&bar(0, 100.0, 100.0, 100.0, 100.0));

        // Close 50% of the 4-contract position: 2 remain.
        b.submit(Order {
            reduce_only: true,
            close_target: Some("L".into()),
            qty_percent: Some(50.0),
            qty: None,
            ..Order::market("L", Direction::Long, None)
        });
        b.advance(&bar(1, 110.0, 110.0, 110.0, 110.0));
        assert_eq!(b.position().size, 2.0);

        // The exited half is a closed trade; the rest stays open.
        assert_eq!(b.closed_trades().len(), 1);
        assert_eq!(b.closed_trades()[0].size, 2.0);
        assert_eq!(b.closed_trades()[0].profit(0.0), 20.0); // (110 - 100) * 2
        assert_eq!(b.open_trades().len(), 1);
        assert_eq!(b.open_trades()[0].size, 2.0);
    }

    #[test]
    fn a_trailing_stop_follows_the_peak_and_fills_at_its_level() {
        // mintick 0.5: activation 4 ticks (2.0) above entry, trailing 2 ticks
        // (1.0) behind the peak.
        let fills = PineFills {
            slippage: 0.0,
            mintick: 0.5,
        };
        let mut b = BarBroker::new(fills, 10_000.0).with_mintick(0.5);
        b.submit(Order::market("L", Direction::Long, Some(1.0)));
        b.submit_exit(Exit {
            trail_points: Some(4.0),
            trail_offset: Some(2.0),
            ..Exit::resting("X", Some("L".into()), None, None)
        });

        // Entry at 100; high 101 has not reached the 102 activation level.
        b.advance(&bar(0, 100.0, 101.0, 99.0, 100.0));
        assert_eq!(b.position().size, 1.0);

        // Same bar arms and fills: price rallies to 105 (peak, so the stop is at
        // 104), then the low 101 retraces through it — the stop fills at its own
        // level, 104, not the earlier open, for a profit of 4.
        b.advance(&bar(1, 102.0, 105.0, 101.0, 104.0));
        assert!(b.position().is_flat());
        assert_eq!(b.closed_trades().len(), 1);
        assert_eq!(b.equity(104.0), 10_004.0);
    }
}
