# Pine Script for VS Code

Language support for Pine Script — syntax highlighting plus diagnostics
(semantic + lint), formatting, hover and go-to-definition from the `pinecone`
language server (`pinecone lsp`).

## Develop / debug

Prerequisites: a Rust toolchain and Node.js 18+.

1. Install the extension's dependencies:

   ```sh
   cd editors/vscode
   npm install
   ```

2. Open the repository **root** in VS Code — the root `.vscode/launch.json`
   targets the extension.

3. Press **F5** (the **Run Extension** launch config). This builds the
   `pinecone` binary and compiles the extension, then opens an *Extension
   Development Host* window with the extension loaded and pointed at the freshly
   built server (`../../target/debug/pinecone`).

4. In that window, open or create a `.pine` file to try it out — e.g. the
   fixtures under `testFixture/`. You should see coloring, a squiggle under a
   repainting `request.security(..., close)`, formatting (Shift+Alt+F), hover,
   and go-to-definition (F12).

Set breakpoints in `src/extension.ts` and debug the client directly. To debug
the *server*, attach to / run `pinecone lsp` separately, or add tracing.

## Tests

- **F5 → Extension Tests** runs the integration suite inside a debuggable VS
  Code instance.
- From the command line:

  ```sh
  cargo build -p pinecone      # the server the tests spawn
  cd editors/vscode
  npm test                     # downloads VS Code and runs the suite headless
  ```

  On Linux CI this runs under `xvfb-run -a npm test` (see the `extension-tests`
  job in `.github/workflows/test.yml`).

## Configuration

- `pinecone.server.path` — path to the `pinecone` executable used as the
  language server (default `pinecone`, i.e. found on `PATH`). During development
  the launch config overrides this with the local debug build via `SERVER_PATH`.
