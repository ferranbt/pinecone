---
name: parser-ast-sync
description: Use when modifying the Pine parser (crates/pine-parser) — grammar, node shapes, or how anything is parsed. The parser's behavior is pinned by the `_ast.json` AST snapshot fixtures under crates/pine-parser/testdata, so a parser change must regenerate and re-verify those, and new syntax needs a new fixture.
---

# Update the AST snapshot fixtures when the parser changes

The parser is guarded by AST snapshot tests: every `crates/pine-parser/testdata/<area>/<case>.pine`
has a paired `<case>_ast.json` holding the expected parse tree. The test
`test_parse_testdata_files` (in `crates/pine-parser/src/lib.rs`) parses each
`.pine` and asserts the AST matches its `_ast.json`. So any change to how the
parser builds the tree changes those snapshots.

## After changing the parser

1. **Regenerate and review the diff** — never hand-edit `_ast.json`:
   ```sh
   GENERATE_AST=1 cargo test -p pine-parser test_parse_testdata_files
   ```
   Then `git diff crates/pine-parser/testdata` and read every change. The diff is
   the real review: it should show *exactly* the structural change you intended
   and nothing else. An unexpected snapshot change means the parser did something
   you didn't mean to.

2. **Add a fixture for new syntax** — a new construct isn't covered until there's
   a `.pine` for it. Drop `crates/pine-parser/testdata/<area>/<case>.pine`, then
   generate its snapshot with the command above.

## Iterating on one case
```sh
DEBUG=1 TEST_FILE=<case> cargo test -p pine-parser test_parse_testdata_files   # tokens + AST
GENERATE_AST=1 TEST_FILE=<case> cargo test -p pine-parser test_parse_testdata_files  # regen just that one
```

Rule of thumb: if you touched the parser and `git diff crates/pine-parser/testdata`
is empty, either nothing changed structurally or you forgot to run
`GENERATE_AST=1` — check which. Commit the regenerated `_ast.json` with the parser change.
