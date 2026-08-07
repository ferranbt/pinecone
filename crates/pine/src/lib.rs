// Re-export all public types from sub-crates
pub use pine_ast as ast;
pub use pine_broker as broker;
pub use pine_builtins as builtins;
use pine_builtins::DefaultPineOutput;
pub use pine_core as core;
pub use pine_data as data;
pub use pine_diagnostics as diagnostics;
pub use pine_format as format;
pub use pine_interpreter as interpreter;
pub use pine_lexer as lexer;
pub use pine_lint as lint;
pub use pine_parser as parser;
pub use pine_sema as sema;

mod backtest;
mod run;

pub use backtest::{Backtest, Metrics};
pub use pine_core::{DataProvider, DirLoader, FileResolver, LibraryLoader};
pub use run::{Run, RunResult};

use pine_ast::Program;
use pine_core::{
    AlertConditionOutput, BoxOutput, DrawingOutput, FillOutput, GlobalOutput, InputOutput,
    LabelOutput, LineOutput, LogOutput, MetadataOutput, PineOutput, PlotOutput, TableOutput,
};
use pine_core::{Bar, Data, PineVersion, Timeframe, VersionError};
use pine_diagnostics::Diagnostic;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use pine_lexer::{Lexer, LexerError};
use pine_parser::{Parser, ParserError};
use std::collections::HashMap;
use std::rc::Rc;

/// Error type for Pine operations
#[derive(Debug)]
pub enum Error {
    Lexer(LexerError),
    Parser(ParserError),
    Runtime(RuntimeError),
    /// Semantic analysis failed; the program is invalid. Carries every
    /// diagnostic found.
    Sema(Vec<Diagnostic>),
    /// The script's `//@version=N` annotation names a version this toolchain
    /// cannot compile.
    Version(VersionError),
    /// No bars to run over: neither data nor a provider was given, or the
    /// provider could not produce the requested feed.
    Data(pine_core::ProviderError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(e) => write!(f, "Lexer error: {}", e),
            Error::Parser(e) => write!(f, "Parser error: {}", e),
            Error::Runtime(e) => write!(f, "Runtime error: {}", e),
            Error::Version(e) => write!(f, "Version error: {}", e),
            Error::Data(e) => write!(f, "Data error: {}", e),
            // One diagnostic per line, so multiple errors are simply appended.
            Error::Sema(diags) => {
                for (i, d) in diags.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{}", d)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<RuntimeError> for Error {
    fn from(e: RuntimeError) -> Self {
        Error::Runtime(e)
    }
}

impl From<LexerError> for Error {
    fn from(e: LexerError) -> Self {
        Error::Lexer(e)
    }
}

impl From<ParserError> for Error {
    fn from(e: ParserError) -> Self {
        Error::Parser(e)
    }
}

impl From<VersionError> for Error {
    fn from(e: VersionError) -> Self {
        Error::Version(e)
    }
}

/// Parse and semantically analyze `source` without bars or execution, returning
/// every diagnostic.
pub fn check(source: &str, loader: Option<&dyn LibraryLoader>) -> Result<Vec<Diagnostic>, Error> {
    let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);
    let tokens = Lexer::with_version(source, version).tokenize()?;
    let program = Program::new(Parser::new(tokens).parse()?);

    let mut env: HashMap<String, Value<DefaultPineOutput>> =
        pine_builtins::register_namespace_objects(version, None, None);
    for (name, value) in pine_builtins::per_bar_variables(&Bar::default()) {
        env.insert(name, value);
    }

    Ok(pine_sema::analyze(&program, &env, loader))
}

pub struct ScriptBuilder<O: PineOutput> {
    source: String,
    custom_variables: HashMap<String, Value<O>>,
    library_loader: Option<Box<dyn LibraryLoader>>,
    request_provider: Option<Box<dyn DataProvider>>,
    ticker: Option<String>,
    timeframe: Timeframe,
    data: Option<Data>,
    bar_count: Option<usize>,
    broker_factory: Option<Box<dyn pine_broker::BrokerFactory>>,
}

impl<O: PineOutput> ScriptBuilder<O> {
    pub fn with_code(source: &str) -> ScriptBuilder<O> {
        Self {
            source: source.to_string(),
            custom_variables: HashMap::new(),
            library_loader: None,
            request_provider: None,
            ticker: None,
            timeframe: Timeframe::default(),
            data: None,
            bar_count: None,
            broker_factory: None,
        }
    }

    /// Host-supplied variables the script can reference, registered as consts
    /// alongside the builtin namespaces.
    pub fn with_custom_variables(mut self, variables: HashMap<String, Value<O>>) -> Self {
        self.custom_variables = variables;
        self
    }

    /// Resolves `import` statements. Without one, importing a library fails.
    pub fn with_library_loader(mut self, loader: Box<dyn LibraryLoader>) -> Self {
        self.library_loader = Some(loader);
        self
    }

    /// Supplies bars for `request.security`. Without one, `request.security`
    /// returns na.
    pub fn with_request_provider(mut self, provider: Box<dyn DataProvider>) -> Self {
        self.request_provider = Some(provider);
        self
    }

    /// Swaps the broker a `strategy` trades against. Without one, the built-in
    /// [`DefaultBrokerFactory`](pine_broker::DefaultBrokerFactory) is used.
    pub fn with_broker(mut self, factory: Box<dyn pine_broker::BrokerFactory>) -> Self {
        self.broker_factory = Some(factory);
        self
    }

    pub fn with_ticker(mut self, ticker: String) -> Self {
        self.ticker = Some(ticker);
        self
    }

    /// The chart timeframe exposed to the script as `timeframe.*`. Without one,
    /// the namespace is populated with defaults.
    pub fn with_timeframe(mut self, timeframe: Timeframe) -> Self {
        self.timeframe = timeframe;
        self
    }

    /// Run over only the last `bar_count` bars of the feed. Without one, the
    /// whole feed is used.
    pub fn with_bar_count(mut self, bar_count: usize) -> Self {
        self.bar_count = Some(bar_count);
        self
    }

    /// The market to run over: the bars, and the symbol and timeframe they
    /// belong to.
    ///
    /// The data describes itself, so it fills in `syminfo.*` and `timeframe.*`
    /// too. An explicit [`ScriptBuilder::with_syminfo`] or
    /// [`ScriptBuilder::with_timeframe`] still wins, whichever order they are
    /// called in.
    pub fn with_data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    /// Compile PineScript source code into a Script with default output
    pub fn compile(self) -> Result<Script<O>, Error>
    where
        O: LogOutput
            + PlotOutput
            + LabelOutput
            + BoxOutput
            + InputOutput
            + LineOutput
            + TableOutput
            + MetadataOutput
            + GlobalOutput
            + AlertConditionOutput
            + FillOutput
            + DrawingOutput,
    {
        let data = match self.data {
            Some(data) => data,
            None => {
                let provider = self
                    .request_provider
                    .as_ref()
                    .ok_or_else(|| Error::Data("no data or request provider set".into()))?;

                let ticker = self.ticker.clone().unwrap_or_default();
                provider
                    .request(&ticker, self.timeframe.clone())
                    .map_err(Error::Data)?
            }
        };

        let syminfo = data.syminfo;
        let timeframe = self.timeframe;

        // Keep only the last `bar_count` bars when the caller limited the run.
        let mut bars = data.bars;
        if let Some(n) = self.bar_count {
            let len = bars.len();
            bars = bars.split_off(len.saturating_sub(n.max(1)));
        }

        // The chart's bar spacing, so `request.security_lower_tf` can reject a
        // request that is not actually lower than the chart timeframe.
        let chart_period = bars
            .windows(2)
            .next()
            .map(|pair| pair[1].time - pair[0].time);

        let source = self.source.as_str();
        let version = PineVersion::detect(source)?.unwrap_or(PineVersion::LATEST);

        let mut lexer = Lexer::with_version(source, version);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let program = Program::new(statements);

        // The interpreter's const environment: the registered namespaces plus any
        // host-supplied globals. Built once and handed over as-is.
        let mut consts = pine_builtins::register_namespace_objects(
            version,
            Some(syminfo),
            Some(timeframe.clone()),
        );
        for (name, value) in self.custom_variables {
            consts.insert(name, value);
        }

        let mut builtins = consts.clone();
        for (name, value) in pine_builtins::per_bar_variables(&Bar::default()) {
            builtins.insert(name, value);
        }

        // Semantic pre-check: reject if sema produces errors.
        let errors: Vec<_> =
            pine_sema::analyze(&program, &builtins, self.library_loader.as_deref())
                .into_iter()
                .filter(|diagnostic| diagnostic.severity == pine_diagnostics::Severity::Error)
                .collect();
        if !errors.is_empty() {
            return Err(Error::Sema(errors));
        }

        // Create interpreter and load builtin namespace objects
        let mut interpreter = Interpreter::new();
        interpreter.library_loader = self.library_loader;
        interpreter.request_provider = self.request_provider.map(Rc::from);
        interpreter.chart_period = chart_period;
        if let Some(broker_factory) = self.broker_factory {
            interpreter.broker_factory = Some(broker_factory);
        }
        interpreter.set_const_variables(consts);

        Ok(Script {
            program,
            interpreter,
            timeframe,
            bars,
            equity_curve: Vec::new(),
            last_close: 0.0,
            equity_peak: f64::NEG_INFINITY,
            equity_trough: f64::INFINITY,
            max_drawdown: 0.0,
            max_runup: 0.0,
            max_drawdown_percent: 0.0,
            max_runup_percent: 0.0,
            max_contracts_all: 0.0,
            max_contracts_long: 0.0,
            max_contracts_short: 0.0,
        })
    }
}

/// A compiled PineScript program, and the bars it will run over.
///
/// State accumulates across bars — series history, `var` locals, and every
/// stateful builtin's window — exactly as it does in TradingView. That makes a
/// `Script` single-use: [`Script::run`] takes it by value so a second run
/// cannot inherit the first one's state.
pub struct Script<O: PineOutput> {
    program: Program,
    interpreter: Interpreter<O>,
    /// The chart timeframe, carried onto the `Backtest` so its metrics can
    /// annualise per-bar figures.
    timeframe: Timeframe,
    /// Bars from the builder's source; empty when none was given.
    bars: Vec<Bar>,
    /// Account value at each bar's close, accumulated while a `strategy` runs.
    equity_curve: Vec<f64>,
    /// The last bar's close, used to mark open trades at the run's end.
    last_close: f64,
    /// Running equity extremes for `strategy.max_drawdown`/`max_runup`.
    equity_peak: f64,
    equity_trough: f64,
    max_drawdown: f64,
    max_runup: f64,
    /// Drawdown/run-up as a fraction of the peak/trough, tracked separately
    /// because the percentage extreme need not coincide with the cash extreme.
    max_drawdown_percent: f64,
    max_runup_percent: f64,
    /// Largest position (in contracts) ever held, overall and per side.
    max_contracts_all: f64,
    max_contracts_long: f64,
    max_contracts_short: f64,
}

impl<O: PineOutput> Script<O> {
    /// Run one bar. Private: bars must be replayed in order from the first, so
    /// [`Script::run`] is the only way in.
    pub fn execute(&mut self, bar: &Bar) -> Result<O, Error> {
        use interpreter::Value;

        for (name, value) in pine_builtins::per_bar_variables(bar) {
            if matches!(value, Value::Series(_)) {
                self.interpreter.advance_series(&name, value);
            } else {
                self.interpreter.set_variable(&name, value);
            }
        }

        // Fill orders left pending by the previous bar before the body runs, so
        // it reads the position and equity they produced. A no-op unless the
        // script declared a `strategy`.
        self.advance_broker(bar);

        let output = self.interpreter.execute(&self.program)?;

        // Read after the body so the bar a `strategy` is declared on is counted.
        if let Some(broker) = self.interpreter.broker.as_ref() {
            self.equity_curve.push(broker.equity(bar.close));
            self.last_close = bar.close;
        }

        Ok(output)
    }

    /// Advance the simulated broker one bar and refresh the read-only
    /// `strategy.*` values from it. The interpreter only holds the broker
    /// handle; the backtest accounting that maps it onto script variables lives
    /// here, in the host.
    fn advance_broker(&mut self, bar: &Bar) {
        use interpreter::Value;

        let close = bar.close;

        // Read from the broker, then drop the borrow to update `self`'s state.
        let Some(broker) = self.interpreter.broker.as_mut() else {
            return;
        };
        broker.advance(bar);

        let position = broker.position();
        let equity = broker.equity(close);
        let initial = broker.initial_capital();
        // Equity is monotonic in price, so its intrabar extremes are the marks
        // at the bar's high and low — lower is adverse, higher favourable.
        let equity_hi = broker.equity(bar.high);
        let equity_lo = broker.equity(bar.low);
        let intrabar_low = equity_hi.min(equity_lo);
        let intrabar_high = equity_hi.max(equity_lo);
        let open_profit: f64 = broker.open_trades().iter().map(|t| t.profit(close)).sum();
        let open_trades = broker.open_trades().len() as i64;
        let closed_trades = broker.closed_trades().len() as i64;

        let (mut gross_profit, mut gross_loss) = (0.0, 0.0);
        let (mut wins, mut losses, mut evens) = (0i64, 0i64, 0i64);
        for trade in broker.closed_trades() {
            let profit = trade.profit(close); // closed, so the price is ignored
            if profit > 0.0 {
                gross_profit += profit;
                wins += 1;
            } else if profit < 0.0 {
                gross_loss -= profit; // positive magnitude, as Pine reports it
                losses += 1;
            } else {
                evens += 1;
            }
        }

        // Read the rest off the broker while its borrow is live: each closed
        // trade's percent return (for the average-trade-percent figures) and the
        // open position's entry name.
        let (mut trade_pcts, mut win_pcts, mut loss_pcts) = (Vec::new(), Vec::new(), Vec::new());
        for trade in broker.closed_trades() {
            let profit = trade.profit(close);
            let basis = trade.entry_price * trade.size.abs();
            let ret = if basis != 0.0 { profit / basis * 100.0 } else { 0.0 };
            trade_pcts.push(ret);
            if profit > 0.0 {
                win_pcts.push(ret);
            } else if profit < 0.0 {
                loss_pcts.push(ret);
            }
        }
        let position_entry_name = broker
            .open_trades()
            .last()
            .map_or(Value::Na, |t| Value::String(t.entry_id.clone()));

        // Drawdown/run-up measure the intrabar extreme against a peak/trough
        // that tracks close equity — an intrabar swing does not move the mark.
        // Seed with the starting capital (the declaration bar's equity).
        if self.equity_peak == f64::NEG_INFINITY {
            self.equity_peak = initial;
            self.equity_trough = initial;
        }
        self.equity_peak = self.equity_peak.max(equity);
        self.equity_trough = self.equity_trough.min(equity);
        self.max_drawdown = self.max_drawdown.max(self.equity_peak - intrabar_low);
        self.max_runup = self.max_runup.max(intrabar_high - self.equity_trough);
        if self.equity_peak > 0.0 {
            let dd = (self.equity_peak - intrabar_low) / self.equity_peak * 100.0;
            self.max_drawdown_percent = self.max_drawdown_percent.max(dd);
        }
        if self.equity_trough > 0.0 {
            let ru = (intrabar_high - self.equity_trough) / self.equity_trough * 100.0;
            self.max_runup_percent = self.max_runup_percent.max(ru);
        }
        // Largest position held, overall and per side.
        self.max_contracts_all = self.max_contracts_all.max(position.size.abs());
        if position.size > 0.0 {
            self.max_contracts_long = self.max_contracts_long.max(position.size);
        } else if position.size < 0.0 {
            self.max_contracts_short = self.max_contracts_short.max(-position.size);
        }

        // Pine's identity equity = initial + netprofit + openprofit; derive
        // netprofit from it so commission can't make the two drift.
        let net_profit = equity - initial - open_profit;
        // na, not 0, when flat — matching Pine.
        let avg_price = if position.size == 0.0 {
            Value::Na
        } else {
            Value::Number(position.avg_price)
        };

        let refreshed = [
            ("position_size", Value::Number(position.size)),
            ("position_avg_price", avg_price),
            ("equity", Value::Number(equity)),
            ("netprofit", Value::Number(net_profit)),
            ("openprofit", Value::Number(open_profit)),
            ("grossprofit", Value::Number(gross_profit)),
            ("grossloss", Value::Number(gross_loss)),
            ("max_drawdown", Value::Number(self.max_drawdown)),
            ("max_runup", Value::Number(self.max_runup)),
            ("opentrades", Value::Int(open_trades)),
            ("closedtrades", Value::Int(closed_trades)),
            ("wintrades", Value::Int(wins)),
            ("losstrades", Value::Int(losses)),
            ("eventrades", Value::Int(evens)),
        ];
        for (name, value) in refreshed {
            self.interpreter.set_object_field("strategy", name, value);
        }

        // Derived statistics: percentages of the starting capital, and per-trade
        // averages. `na` when there are no trades to average, matching Pine.
        let pct = |x: f64| if initial != 0.0 { x / initial * 100.0 } else { 0.0 };
        let per_trade = |total: f64, count: i64| {
            if count > 0 {
                Value::Number(total / count as f64)
            } else {
                Value::Na
            }
        };
        let mean = |v: &[f64]| {
            if v.is_empty() {
                Value::Na
            } else {
                Value::Number(v.iter().sum::<f64>() / v.len() as f64)
            }
        };
        let derived = [
            ("netprofit_percent", Value::Number(pct(net_profit))),
            ("openprofit_percent", Value::Number(pct(open_profit))),
            ("grossprofit_percent", Value::Number(pct(gross_profit))),
            ("grossloss_percent", Value::Number(pct(gross_loss))),
            ("max_drawdown_percent", Value::Number(self.max_drawdown_percent)),
            ("max_runup_percent", Value::Number(self.max_runup_percent)),
            ("max_contracts_held_all", Value::Number(self.max_contracts_all)),
            ("max_contracts_held_long", Value::Number(self.max_contracts_long)),
            ("max_contracts_held_short", Value::Number(self.max_contracts_short)),
            ("avg_trade", per_trade(net_profit, closed_trades)),
            ("avg_winning_trade", per_trade(gross_profit, wins)),
            // Losing trades are reported as a negative average, so negate the
            // positive gross-loss magnitude.
            ("avg_losing_trade", per_trade(-gross_loss, losses)),
            ("avg_trade_percent", mean(&trade_pcts)),
            ("avg_winning_trade_percent", mean(&win_pcts)),
            ("avg_losing_trade_percent", mean(&loss_pcts)),
            ("position_entry_name", position_entry_name),
        ];
        for (name, value) in derived {
            self.interpreter.set_object_field("strategy", name, value);
        }
    }

    /// Replay the script over every bar from its source, returning what each
    /// one produced.
    pub fn run(mut self) -> Result<Run<O>, Error> {
        let bars = std::mem::take(&mut self.bars);
        let outputs = bars
            .iter()
            .map(|bar| self.execute(bar))
            .collect::<Result<Vec<O>, Error>>()?;
        let backtest = self.take_backtest();
        Ok(Run { outputs, backtest })
    }

    fn take_backtest(&mut self) -> Option<Backtest> {
        let broker = self.interpreter.broker.as_ref()?;
        let close = self.last_close;

        // Closed trades first, then those still open.
        let mut trades: Vec<_> = broker.closed_trades().to_vec();
        trades.extend(broker.open_trades().into_iter().cloned());

        let open_profit: f64 = broker.open_trades().iter().map(|t| t.profit(close)).sum();
        let initial_capital = broker.initial_capital();
        let position_size = broker.position().size;

        let (mut gross_profit, mut gross_loss) = (0.0, 0.0);
        let (mut win_trades, mut loss_trades, mut even_trades) = (0, 0, 0);
        for trade in broker.closed_trades() {
            let profit = trade.profit(close);
            if profit > 0.0 {
                gross_profit += profit;
                win_trades += 1;
            } else if profit < 0.0 {
                gross_loss -= profit;
                loss_trades += 1;
            } else {
                even_trades += 1;
            }
        }

        let equity = std::mem::take(&mut self.equity_curve);
        let final_equity = equity.last().copied().unwrap_or(initial_capital);

        Some(Backtest {
            initial_capital,
            net_profit: final_equity - initial_capital - open_profit,
            open_profit,
            gross_profit,
            gross_loss,
            max_drawdown: self.max_drawdown,
            max_runup: self.max_runup,
            win_trades,
            loss_trades,
            even_trades,
            position_size,
            mark_price: close,
            equity,
            trades,
            halted: broker.halted_bar(),
            timeframe: self.timeframe.clone(),
        })
    }
}

pub fn execute(source: &str, data: Data) -> Result<(), Error> {
    ScriptBuilder::<DefaultPineOutput>::with_code(source)
        .with_data(data)
        .compile()?
        .run()
        .map(|_| ())
}
