---
name: vscode-integration-tests
description: Use whenever you change the language server (crates/pine-lsp) or the VS Code extension (editors/vscode) — a new/changed capability like hover, go-to-definition, diagnostics, formatting, or client behavior. Every such change gets an end-to-end test in the VS Code extension test suite that drives the real server through VS Code's LSP client.
---

# Every LSP/extension change gets a VS Code integration test

The language server is validated **end-to-end through VS Code**: a real editor
launches the extension, which spawns the real `pinecone lsp` binary, and the
test drives it with VS Code's built-in LSP commands. There are no Rust
subprocess harnesses and no mocked client — if you add or change an LSP feature,
add or extend a test here.

## Where things live

- `editors/vscode/src/test/suite/extension.test.ts` — the tests (Mocha `tdd`).
- `editors/vscode/testFixture/*.pine` — the `.pine` documents tests open.
- `editors/vscode/src/test/runTest.ts` — launches VS Code via
  `@vscode/test-electron`. It sets `SERVER_PATH` to `target/debug/pinecone`, so
  **the server binary must be built first**.
- `editors/vscode/src/test/suite/index.ts` — Mocha runner; globs `**/*.test.js`.

## Writing a test

1. If you need a new document, add a fixture under `testFixture/` (e.g.
   `symbols.pine`). Keep it minimal.
2. Open it with the `open(uri)` helper — it awaits the server's first
   diagnostics via `onDidChangeDiagnostics`, so the symbol table is ready.
   **Never `setTimeout`/sleep to wait for the server**; wait on the real event.
3. Drive the feature with a VS Code command and assert on the result:

```ts
test("hover lists a function's call sites", async () => {
  const uri = fixture("symbols.pine");
  await open(uri);

  // Positions are 0-based (line, character). `double` in its call on line 4.
  const hovers = await vscode.commands.executeCommand<vscode.Hover[]>(
    "vscode.executeHoverProvider",
    uri,
    new vscode.Position(3, 6)
  );
  const content = hovers[0].contents[0] as vscode.MarkdownString;
  assert.ok(content.value.includes("1 call"), content.value);
});
```

Commands by feature: `vscode.executeHoverProvider`,
`vscode.executeDefinitionProvider`, `vscode.executeDocumentSymbolProvider`,
`vscode.executeFormatDocumentProvider`; diagnostics are read with
`vscode.languages.getDiagnostics(uri)` after `open()`. Positions are **0-based**;
sema reports 1-based, so subtract one when translating a fixture line/column.

Formatting note: VS Code minimizes a full-document replace into smaller edits, so
apply the returned edits to a `WorkspaceEdit` and compare `doc.getText()` rather
than asserting on a single edit's text (see the "formats a document" test).

## Running

Build the server, compile the tests, then run under a virtual display. In this
environment `ELECTRON_RUN_AS_NODE` is set and must be unset or VS Code rejects
its own launch flags:

```sh
cargo build -p pinecone
cd editors/vscode
npm run pretest                                   # compile tests + bundle extension
env -u ELECTRON_RUN_AS_NODE xvfb-run -a npm test
```

Outside this sandbox it is just `npm test`. CI runs the same steps in the
`extension-tests` job of `.github/workflows/test.yml`.

Before finishing: the extension suite passes, and `cargo test -p pine-lsp` +
`cargo clippy -p pine-lsp` are green.
