# PineVM

A modular PineScript interpreter written in Rust.

PineVM executes PineScript code (TradingView's scripting language) with support for technical analysis, custom indicators, and strategy backtesting. The interpreter is designed to be extensible - you can add custom builtin functions and output types to integrate with your own systems.

## Features

- Full PineScript v5 language support
- Technical analysis functions (moving averages, oscillators, etc.)
- Drawing objects (plots, labels, boxes)
- Modular output system - extend with [custom types and builtins](examples/custom-builtin-func)
- Type-safe generic architecture

## Example

```rust
use pine::Script;

let script = Script::compile(r#"
    fast_ma = ta.sma(close, 10)
    slow_ma = ta.sma(close, 20)
    plot(fast_ma, color=color.blue)
    plot(slow_ma, color=color.red)
"#)?;

let output = script.execute(&bar)?;
```
