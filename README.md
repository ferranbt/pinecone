# Pinecone

[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://app.codspeed.io/ferranbt/pinecone/badge.json)](https://app.codspeed.io/ferranbt/pinecone?utm_source=badge)

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
use pine::data::{CsvSource, DataSource};
use pine::{RunResult, ScriptBuilder};

let data = CsvSource::from_path("btc_1h.csv")?.load()?;

let outputs = ScriptBuilder::with_code(r#"
    fast = ta.sma(close, 10)
    slow = ta.sma(close, 20)
    plot(fast, title="fast")
    plot(slow, title="slow")
"#)
.with_data(data)
.compile()?
.run()?;
```
