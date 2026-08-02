# Pinecone

A modular PineScript interpreter written in Rust.

Pinecone executes PineScript code (TradingView's scripting language) with support for technical analysis, custom indicators, and strategy backtesting. The interpreter is designed to be extensible - you can add custom builtin functions and output types to integrate with your own systems.

## Features

- PineScript v4 and v5 language support
- Technical analysis functions (moving averages, oscillators, etc.)
- Drawing objects (plots, labels, boxes)
- Market data from CSV files, or any source you implement
- Modular output system - extend with [custom types and builtins](examples/custom-builtin-func)
- Type-safe generic architecture

## Example

A script is replayed over a whole series of bars — series history and indicator
state build up as they execute.

```rust
use pine::data::StaticProvider;
use pine::ScriptBuilder;

let provider = StaticProvider::from_csv("btc_1h.csv")?;

let outputs = ScriptBuilder::with_code(r#"
    fast = ta.sma(close, 10)
    slow = ta.sma(close, 20)
    plot(fast, title="fast")
    plot(slow, title="slow")
"#)
.with_timeframe("60".parse()?)
.with_request_provider(Box::new(provider))
.compile()?
.run()?;
```
