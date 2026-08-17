# Pine Script for VS Code

Language support for [TradingView Pine Script](https://www.tradingview.com/pine-script-docs/),
powered by the [`pinecone`](https://github.com/ferranbt/pinecone) toolchain.

## Features

- **Syntax highlighting** for `.pine` files.
- **Diagnostics** as you type — semantic errors plus lint warnings for things
  that quietly break strategies, such as repainting `request.security` calls,
  `lookahead` bias, and intrabar strategy recalculation.
- **Formatting** — format a document with the standard *Format Document*
  command (Shift+Alt+F).
- **Hover** — signatures and kinds for your variables, functions and types.
- **Go to definition** (F12) for symbols declared in the file.

## Requirements

The extension talks to the `pinecone` language server, so that binary needs to
be available:

- Download a prebuilt `pinecone` from the
  [releases](https://github.com/ferranbt/pinecone/releases) (or build it from
  source with `cargo build --release -p pinecone`), and
- put it on your `PATH`, **or** point the extension at it with the
  `pinecone.server.path` setting.

## Extension settings

| Setting                | Default    | Description                                             |
| ---------------------- | ---------- | ------------------------------------------------------- |
| `pinecone.server.path` | `pinecone` | Path to the `pinecone` executable used as the server. |

Server logs are available under **View → Output → Pinecone**.