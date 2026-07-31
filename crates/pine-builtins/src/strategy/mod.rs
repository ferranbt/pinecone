//! The `strategy` namespace: the declaration plus the order commands a
//! backtest trades with.
//!
//! `strategy` is both callable and a namespace — `strategy("My Strat", ...)`
//! declares the script and sets up the simulated [`Broker`], while
//! `strategy.entry`/`strategy.close`/… submit orders to it. The read-only
//! values (`strategy.position_size`, `strategy.equity`, …) are seeded here and
//! refreshed each bar by the host after the broker advances; the interpreter
//! itself holds only the broker handle and carries no backtest logic.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pine_broker::{
    BarBroker, Commission, Direction, Exit, OcaType, Order, OrderKind, PineFills, Sizing,
};
use pine_builtin_macro::BuiltinFunction;
use pine_core::PineVersion;
use pine_interpreter::{BuiltinFn, Interpreter, PineOutput, RuntimeError, Value};

/// TradingView's default starting capital.
const DEFAULT_INITIAL_CAPITAL: f64 = 1_000_000.0;

/// strategy(title, shorttitle, overlay, ..., default_qty_type, default_qty_value,
/// initial_capital, ..., slippage, commission_type, commission_value, ...)
///
/// Only the parameters that shape the simulated broker are honoured; display
/// and reporting-only parameters are accepted and ignored. Runs every bar, but
/// only builds the broker on the first, so state persists across the backtest.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy")]
struct StrategyFn {
    #[allow(dead_code)]
    title: String,
    #[arg(default = "")]
    shorttitle: String,
    #[arg(default = false)]
    overlay: bool,
    #[arg(default = "")]
    format: String,
    #[arg(default = None)]
    precision: Option<f64>,
    #[arg(default = "")]
    scale: String,
    #[arg(default = None)]
    pyramiding: Option<f64>,
    #[arg(default = "fixed")]
    default_qty_type: String,
    #[arg(default = 1.0)]
    default_qty_value: f64,
    #[arg(default = None)]
    initial_capital: Option<f64>,
    #[arg(default = "")]
    currency: String,
    #[arg(default = 0.0)]
    slippage: f64,
    #[arg(default = "percent")]
    commission_type: String,
    #[arg(default = 0.0)]
    commission_value: f64,
}

impl StrategyFn {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = (
            &self.shorttitle,
            self.overlay,
            &self.format,
            self.precision,
            &self.scale,
            &self.currency,
        );

        // Runs every bar; build the broker only once so trades accumulate.
        if ctx.broker.is_none() {
            let initial_capital = self.initial_capital.unwrap_or(DEFAULT_INITIAL_CAPITAL);
            let mintick = mintick_of(ctx);
            let fills = PineFills {
                slippage: self.slippage,
                mintick,
            };
            let mut broker = BarBroker::new(fills, initial_capital)
                .with_mintick(mintick)
                .with_sizing(self.sizing())
                .with_pyramiding(self.pyramiding.unwrap_or(0.0) as usize);

            if self.commission_value != 0.0 {
                let commission = match self.commission_type.as_str() {
                    "cash_per_contract" => Commission::CashPerContract(self.commission_value),
                    "cash_per_order" => Commission::CashPerOrder(self.commission_value),
                    // "percent" and anything unrecognised.
                    _ => Commission::Percent(self.commission_value),
                };
                broker = broker.with_commission(commission);
            }

            ctx.broker = Some(Box::new(broker));
            ctx.set_object_field(
                "strategy",
                "initial_capital",
                Value::Number(initial_capital),
            );
            ctx.set_object_field("strategy", "equity", Value::Number(initial_capital));
        }

        Ok(Value::Na)
    }

    /// How an order's absent `qty` is sized, from `default_qty_type`.
    fn sizing(&self) -> Sizing {
        match self.default_qty_type.as_str() {
            "cash" => Sizing::Cash(self.default_qty_value),
            "percent_of_equity" => Sizing::PercentOfEquity(self.default_qty_value),
            // "fixed" and anything unrecognised: a contract count.
            _ => Sizing::Contracts(self.default_qty_value),
        }
    }
}

/// The symbol's tick size from `syminfo.mintick`, or 0 (which disables tick-based
/// slippage and exit distances) when it is unknown.
fn mintick_of<O: PineOutput>(ctx: &Interpreter<O>) -> f64 {
    if let Some(Value::Object { fields, .. }) = ctx.get_variable("syminfo") {
        if let Some(Value::Number(mintick)) = fields.borrow().get("mintick") {
            return *mintick;
        }
    }
    0.0
}

/// The current bar's `close`, used to size a default-qty order the way Pine
/// does — from the close of the bar the order command runs on.
fn close_of<O: PineOutput>(ctx: &Interpreter<O>) -> f64 {
    match ctx.get_variable("close") {
        Some(Value::Series(series)) => match series.current.as_ref() {
            Value::Number(n) => *n,
            _ => f64::NAN,
        },
        Some(Value::Number(n)) => *n,
        _ => f64::NAN,
    }
}

/// A string argument as an option, mapping the empty default to `None` — used
/// for an OCA group name and an exit's `from_entry`.
fn non_empty(name: &str) -> Option<String> {
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// The order condition from an entry/order call's `limit` and `stop`: both set
/// makes a stop-limit, either alone a limit or stop, neither a market order.
fn order_kind(limit: Option<f64>, stop: Option<f64>) -> OrderKind {
    match (limit, stop) {
        (Some(limit), Some(stop)) => OrderKind::StopLimit { stop, limit },
        (Some(limit), None) => OrderKind::Limit(limit),
        (None, Some(stop)) => OrderKind::Stop(stop),
        (None, None) => OrderKind::Market,
    }
}

/// strategy.entry(id, direction, qty, limit, stop, ...)
///
/// Enters or reverses a position: a fill on the opposite side closes the
/// current position and opens the requested one.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy.entry")]
struct StrategyEntry {
    id: String,
    direction: String,
    #[arg(default = None)]
    qty: Option<f64>,
    #[arg(default = None)]
    limit: Option<f64>,
    #[arg(default = None)]
    stop: Option<f64>,
    #[arg(default = "")]
    oca_name: String,
    #[arg(default = "")]
    oca_type: String,
    #[arg(default = "")]
    comment: String,
}

impl StrategyEntry {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let sizing_price = close_of(ctx);
        if let Some(broker) = ctx.broker.as_mut() {
            broker.submit(Order {
                id: self.id.clone(),
                direction: Direction::from(self.direction.as_str()),
                qty: self.qty,
                qty_percent: None,
                sizing_price: Some(sizing_price),
                kind: order_kind(self.limit, self.stop),
                reduce_only: false,
                reverses: true,
                close_target: None,
                oca_name: non_empty(&self.oca_name),
                oca_type: OcaType::from(self.oca_type.as_str()),
                comment: self.comment.clone(),
            });
        }
        Ok(Value::Na)
    }
}

/// strategy.order(id, direction, qty, limit, stop, ...)
///
/// Like [`StrategyEntry`] but a plain order: it neither reverses an opposite
/// position nor obeys pyramiding — it simply adds contracts in `direction`.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy.order")]
struct StrategyOrder {
    id: String,
    direction: String,
    #[arg(default = None)]
    qty: Option<f64>,
    #[arg(default = None)]
    limit: Option<f64>,
    #[arg(default = None)]
    stop: Option<f64>,
    #[arg(default = "")]
    oca_name: String,
    #[arg(default = "")]
    oca_type: String,
    #[arg(default = "")]
    comment: String,
}

impl StrategyOrder {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let sizing_price = close_of(ctx);
        if let Some(broker) = ctx.broker.as_mut() {
            broker.submit(Order {
                id: self.id.clone(),
                direction: Direction::from(self.direction.as_str()),
                qty: self.qty,
                qty_percent: None,
                sizing_price: Some(sizing_price),
                kind: order_kind(self.limit, self.stop),
                reduce_only: false,
                reverses: false,
                close_target: None,
                oca_name: non_empty(&self.oca_name),
                oca_type: OcaType::from(self.oca_type.as_str()),
                comment: self.comment.clone(),
            });
        }
        Ok(Value::Na)
    }
}

/// strategy.close(id, comment, qty, qty_percent, ...)
///
/// Exits the position opened by entry `id` with a market order, closing that
/// entry's lots oldest-first. With no `qty` it closes all of them.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy.close")]
struct StrategyClose {
    id: String,
    #[arg(default = "")]
    comment: String,
    #[arg(default = None)]
    qty: Option<f64>,
    #[arg(default = None)]
    qty_percent: Option<f64>,
}

impl StrategyClose {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if let Some(broker) = ctx.broker.as_mut() {
            // Direction is ignored for a reduce-only order — the broker closes
            // against whatever side is open — so Long is just a placeholder. The
            // order's id names the entry whose lots it closes.
            broker.submit(Order {
                id: self.id.clone(),
                direction: Direction::Long,
                qty: self.qty,
                qty_percent: self.qty_percent,
                sizing_price: None,
                kind: OrderKind::Market,
                reduce_only: true,
                reverses: false,
                close_target: Some(self.id.clone()),
                oca_name: None,
                oca_type: OcaType::None,
                comment: self.comment.clone(),
            });
        }
        Ok(Value::Na)
    }
}

/// strategy.close_all(comment, alert_message)
///
/// Flattens the position with a market order.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy.close_all")]
struct StrategyCloseAll {
    #[arg(default = "")]
    comment: String,
}

impl StrategyCloseAll {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if let Some(broker) = ctx.broker.as_mut() {
            broker.submit(Order {
                id: "Close all".to_string(),
                direction: Direction::Long,
                qty: None,
                qty_percent: None,
                sizing_price: None,
                kind: OrderKind::Market,
                reduce_only: true,
                reverses: false,
                close_target: None,
                oca_name: None,
                oca_type: OcaType::None,
                comment: self.comment.clone(),
            });
        }
        Ok(Value::Na)
    }
}

/// strategy.cancel(id) — remove a pending order by id.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy.cancel")]
struct StrategyCancel {
    id: String,
}

impl StrategyCancel {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if let Some(broker) = ctx.broker.as_mut() {
            broker.cancel(&self.id);
        }
        Ok(Value::Na)
    }
}

/// strategy.cancel_all() — remove every pending order.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy.cancel_all")]
struct StrategyCancelAll {}

impl StrategyCancelAll {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        if let Some(broker) = ctx.broker.as_mut() {
            broker.cancel_all();
        }
        Ok(Value::Na)
    }
}

/// strategy.exit(id, from_entry, qty, qty_percent, profit, limit, loss, stop, ...)
///
/// Attaches a stop-loss / take-profit bracket to a position. Take-profit is a
/// `limit` price or a `profit` distance in ticks; stop-loss a `stop` price or a
/// `loss` in ticks. The broker fills whichever the bar reaches first (the stop
/// when both do) and cancels the other. Trailing stops are not yet modelled.
#[derive(BuiltinFunction)]
#[builtin(name = "strategy.exit")]
struct StrategyExit {
    id: String,
    #[arg(default = "")]
    from_entry: String,
    #[arg(default = None)]
    qty: Option<f64>,
    #[arg(default = None)]
    qty_percent: Option<f64>,
    #[arg(default = None)]
    profit: Option<f64>,
    #[arg(default = None)]
    limit: Option<f64>,
    #[arg(default = None)]
    loss: Option<f64>,
    #[arg(default = None)]
    stop: Option<f64>,
    #[arg(default = None)]
    trail_price: Option<f64>,
    #[arg(default = None)]
    trail_points: Option<f64>,
    #[arg(default = None)]
    trail_offset: Option<f64>,
    #[arg(default = "")]
    comment: String,
}

impl StrategyExit {
    fn execute<O: PineOutput>(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let _ = &self.comment;
        if let Some(broker) = ctx.broker.as_mut() {
            broker.submit_exit(Exit {
                limit: self.limit,
                profit_ticks: self.profit,
                stop: self.stop,
                loss_ticks: self.loss,
                trail_price: self.trail_price,
                trail_points: self.trail_points,
                trail_offset: self.trail_offset,
                ..Exit::resting(
                    self.id.clone(),
                    non_empty(&self.from_entry),
                    self.qty,
                    self.qty_percent,
                )
            });
        }
        Ok(Value::Na)
    }
}

/// Build the `strategy` namespace object: the callable declaration, the order
/// commands, the direction and sizing constants, and the read-only values the
/// host refreshes each bar (seeded to a flat, zero-profit account).
pub fn register<O: PineOutput>(_version: PineVersion) -> Value<O> {
    let mut fields: HashMap<String, Value<O>> = HashMap::new();

    // Order commands.
    fields.insert("entry".to_string(), StrategyEntry::builtin_value::<O>());
    fields.insert("order".to_string(), StrategyOrder::builtin_value::<O>());
    fields.insert("close".to_string(), StrategyClose::builtin_value::<O>());
    fields.insert(
        "close_all".to_string(),
        StrategyCloseAll::builtin_value::<O>(),
    );
    fields.insert("exit".to_string(), StrategyExit::builtin_value::<O>());
    fields.insert("cancel".to_string(), StrategyCancel::builtin_value::<O>());
    fields.insert(
        "cancel_all".to_string(),
        StrategyCancelAll::builtin_value::<O>(),
    );

    // Direction constants.
    fields.insert("long".to_string(), Value::String("long".to_string()));
    fields.insert("short".to_string(), Value::String("short".to_string()));

    // Sizing constants for `default_qty_type`.
    fields.insert("fixed".to_string(), Value::String("fixed".to_string()));
    fields.insert("cash".to_string(), Value::String("cash".to_string()));
    fields.insert(
        "percent_of_equity".to_string(),
        Value::String("percent_of_equity".to_string()),
    );

    // Commission-type constants (`strategy.commission.*`).
    let mut commission: HashMap<String, Value<O>> = HashMap::new();
    commission.insert("percent".to_string(), Value::String("percent".to_string()));
    commission.insert(
        "cash_per_contract".to_string(),
        Value::String("cash_per_contract".to_string()),
    );
    commission.insert(
        "cash_per_order".to_string(),
        Value::String("cash_per_order".to_string()),
    );
    fields.insert(
        "commission".to_string(),
        Value::Object {
            type_name: "strategy.commission".to_string(),
            fields: Rc::new(RefCell::new(commission)),
            call: None,
        },
    );

    // One-Cancels-All type constants (`strategy.oca.*`).
    let mut oca: HashMap<String, Value<O>> = HashMap::new();
    oca.insert("cancel".to_string(), Value::String("cancel".to_string()));
    oca.insert("reduce".to_string(), Value::String("reduce".to_string()));
    oca.insert("none".to_string(), Value::String("none".to_string()));
    fields.insert(
        "oca".to_string(),
        Value::Object {
            type_name: "strategy.oca".to_string(),
            fields: Rc::new(RefCell::new(oca)),
            call: None,
        },
    );

    // Read-only account values, refreshed each bar by the host after the broker
    // advances. Seeded to a flat, zero-profit account.
    for name in [
        "position_size",
        "equity",
        "initial_capital",
        "netprofit",
        "openprofit",
        "grossprofit",
        "grossloss",
        "max_drawdown",
        "max_runup",
    ] {
        fields.insert(name.to_string(), Value::Number(0.0));
    }
    // na while flat, matching Pine.
    fields.insert("position_avg_price".to_string(), Value::Na);
    for name in [
        "opentrades",
        "closedtrades",
        "wintrades",
        "losstrades",
        "eventrades",
    ] {
        fields.insert(name.to_string(), Value::Int(0));
    }

    Value::Object {
        type_name: "strategy".to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: Some(Rc::new(StrategyFn::builtin_fn) as BuiltinFn<O>),
    }
}
