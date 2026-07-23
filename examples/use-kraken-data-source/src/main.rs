/// Example: Running a script over live exchange data
///
/// Fetches hourly BTC/USD candles from Kraken and runs a moving-average
/// crossover over them. The data carries its own symbol and timeframe, so
/// `syminfo.*` and `timeframe.*` are filled in without being set by hand.
use pine::ScriptBuilder;
use pine_data::{DataSource, KrakenSource};
use pine_interpreter::DefaultPineOutput;

const SCRIPT: &str = r#"
//@version=5
indicator("MA cross")
fast = ta.sma(close, 10)
slow = ta.sma(close, 30)
plot(fast, title="fast")
plot(slow, title="slow")
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = KrakenSource::new("XBTUSD", "1h".parse()?).load()?;
    println!(
        "{}: {} bars at {}",
        data.syminfo.tickerid,
        data.bars.len(),
        data.timeframe.period()
    );

    let outputs = ScriptBuilder::<DefaultPineOutput>::with_code(SCRIPT)
        .with_data(data)
        .compile()?
        .run()?;
    let result = pine::RunResult::collect(&outputs);

    let fast = result.plot("fast").expect("fast is plotted");
    let slow = result.plot("slow").expect("slow is plotted");

    match (
        fast.last().copied().flatten(),
        slow.last().copied().flatten(),
    ) {
        (Some(fast), Some(slow)) => {
            let trend = if fast > slow { "above" } else { "below" };
            println!("fast {fast:.2} is {trend} slow {slow:.2}");
        }
        // Both are na until their windows fill.
        _ => println!("not enough bars to compute both averages"),
    }

    Ok(())
}
