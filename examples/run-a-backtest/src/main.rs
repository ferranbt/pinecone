/// Example: Running a strategy backtest
///
/// Declares a `strategy`, replays it over synthetic bars, and reads the
/// equity curve and trade log back from the `RunResult`.
use pine_lang::ScriptBuilder;
use pine_interpreter::DefaultPineOutput;

fn main() {
    let script_source = r#"
        //@version=5
        strategy("Backtest demo", initial_capital = 10000)

        if bar_index == 1
            strategy.entry("Long", strategy.long)
        if bar_index == 8
            strategy.close("Long")
    "#;

    let run = ScriptBuilder::<DefaultPineOutput>::with_code(script_source)
        .with_data(pine_lang::data::synthetic(10))
        .compile()
        .expect("Compilation failed")
        .run()
        .expect("Execution failed");
    let backtest = run.backtest.expect("the script declared a strategy");

    println!("Net profit:    {:.2}", backtest.net_profit);
    println!("Final equity:  {:.2}", backtest.final_equity());
    println!("Gross profit:  {:.2}", backtest.gross_profit);
    println!("Gross loss:    {:.2}", backtest.gross_loss);
    println!("Max drawdown:  {:.2}", backtest.max_drawdown);
    println!(
        "Wins / losses: {} / {}",
        backtest.win_trades, backtest.loss_trades
    );

    println!("\nTrades:");
    for trade in &backtest.trades {
        let exit = trade.exit_price.map_or("open".to_string(), |p| format!("{p:.2}"));
        println!(
            "  {} {:.0} @ {:.2} -> {}  (profit {:.2})",
            trade.entry_id,
            trade.size,
            trade.entry_price,
            exit,
            trade.profit(backtest.mark_price),
        );
    }

    println!("\nEquity curve:");
    for (bar, equity) in backtest.equity.iter().enumerate() {
        println!("  bar {bar}: {equity:.2}");
    }
}
