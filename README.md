# Pinecone

A modular PineScript interpreter written in Rust.

Pinecone executes PineScript code (TradingView's scripting language) with support for technical analysis, custom indicators, and strategy backtesting. The interpreter is designed to be extensible - you can add custom builtin functions and output types to integrate with your own systems.

Pinecone comes in two parts:

- **Pinecone SDK** — the set of Rust crates below (interpreter, parser, formatter, linter, language server), used as a library. `pine-lang` is the main entry point.
- **[`pinecone` binary](#pinecone-binary)** — a command-line tool built on the SDK: format, lint and check scripts, and run the language server for editors.

## Features

- PineScript v4 and v5 language support
- Technical analysis functions (moving averages, oscillators, etc.)
- Drawing objects (plots, labels, boxes)
- Market data from CSV files, or any source you implement
- Modular output system - extend with [custom types and builtins](examples/custom-builtin-func)
- Type-safe generic architecture

## Pinecone binary

Install the latest release with `up.sh`:

```sh
curl -fsSL https://raw.githubusercontent.com/ferranbt/pinecone/main/up.sh | bash
```

It provides these commands:

| Command | Description |
| --- | --- |
| `pinecone format <paths>` | Format scripts in place (`--stdout`, `--check`). |
| `pinecone lint <paths>` | Report lint findings (repainting, lookahead, …). |
| `pinecone check <paths>` | Parse, semantically analyze and lint. |
| `pinecone lsp` | Run the language server over stdio, for editor integration. |

Paths may be files or directories (searched for `.pine` files).

`pinecone lsp` starts a language server — diagnostics, formatting, hover, go-to-definition, find references, document symbols, rename and completion, resolved across imported libraries. It powers the [VS Code extension](editors/vscode).

## Pinecone SDK

### Install

```toml
[dependencies]
pine-lang = "0.1"
```

### Example

A script is replayed over a whole series of bars — series history and indicator
state build up as they execute.

```rust
use pine_lang::data::StaticProvider;
use pine_lang::ScriptBuilder;

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
