/// Example: Running a script over live exchange data
///
/// Fetches hourly BTC/USD candles from Kraken and runs a moving-average
/// crossover over them.
use pine_lang::ScriptBuilder;
use pine_data::KrakenSource;
use pine_interpreter::DefaultPineOutput;

const SCRIPT: &str = r#"
//@version=5
indicator("MA cross")
fast = ta.sma(close, 10)
slow = ta.sma(close, 30)
plot(fast, title="fast")
plot(slow, title="slow")
"#;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let outputs = ScriptBuilder::<DefaultPineOutput>::with_code(SCRIPT)
        .with_ticker("XBTUSD".to_string())
        .with_timeframe("60".parse()?)
        .with_request_provider(Box::new(KrakenSource::new()))
        .compile()?
        .run()?
        .outputs;
    let result = pine_lang::RunResult::collect(&outputs);

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
