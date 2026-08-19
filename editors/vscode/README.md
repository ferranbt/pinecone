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
- **Hover** — signatures and kinds for your variables, functions and types; for
  a function, its declaration and every call site.
- **Completion** — fields, enum cases and library exports after `.`, plus
  builtin namespace members (`ta.`, `math.`, …) with their signatures.
- **Go to definition** (F12) and **find all references** (Shift+F12), resolved
  into imported libraries.
- **Document outline** and breadcrumbs (Ctrl+Shift+O).
- **Highlight occurrences** of the symbol under the cursor.
- **Rename** (F2) across every occurrence, including imported libraries.

## Requirements

The extension talks to the `pinecone` language server. If it isn't found, the
extension offers to **download** it for you. Otherwise, make it available by
either:

- putting a `pinecone` binary on your `PATH` (install it with
  [`up.sh`](https://github.com/ferranbt/pinecone#pinecone-binary), download a
  prebuilt one from the
  [releases](https://github.com/ferranbt/pinecone/releases), or build it with
  `cargo build --release -p pinecone`), **or**
- pointing the extension at it with the `pinecone.server.path` setting.

## Extension settings

| Setting                | Default    | Description                                             |
| ---------------------- | ---------- | ------------------------------------------------------- |
| `pinecone.server.path` | `pinecone` | Path to the `pinecone` executable used as the server. |

Server logs are available under **View → Output → Pinecone**.
