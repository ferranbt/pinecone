# Development Guidelines

## Parser Error Handling

When encountering syntax parsing errors, follow the workflow documented in `.claude/parsing-workflow.md`.

## Pine Script Reference

When implementing or debugging Pine Script features, **always consult the official reference** using `just ref`. See `.claude/pine-reference.md` for details.

## Testing

**Every interpreter feature, builtin function, and language construct MUST have both positive and negative integration test cases.** See `.claude/testing.md` for:

- Test organization and file structure
- How to write success and error tests
- Running tests with just/cargo
- Best practices and requirements
