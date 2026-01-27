# PineScript Parser Workflow

This repository contains a parser for PineScript. When encountering syntax parsing errors, follow this systematic workflow.

## Repository Structure

- **`testdata/`**: Contains test cases for syntax patterns that should parse correctly
  - Each `.pine` file is a test case
  - Each test case has a corresponding `_ast.json` file with the expected AST in JSON format

## Available Tools

### 1. Test Suite: `test_parse_testdata_files`

Main test that validates parser against all testdata cases.

**Environment Variables:**

- `DEBUG=1` - Output tokens and AST JSON during test execution
- `TEST_FILE=<filename>` - Filter to run only a specific test file
- `GENERATE_AST=1` - Update/regenerate the `_ast.json` files

**Examples:**

```bash
# Run all tests
cargo test test_parse_testdata_files

# Debug a specific file
DEBUG=1 TEST_FILE=my_test cargo test test_parse_testdata_files

# Generate AST for a specific file
GENERATE_AST=1 TEST_FILE=my_test cargo test test_parse_testdata_files

# Run all tests and regenerate all AST files
GENERATE_AST=1 cargo test test_parse_testdata_files
```

### 2. Test Suite: `test_parse_external_pinescript_indicators`

Corpus tests that validate parsing works on real-world indicators. These tests don't have AST files - they only verify that parsing completes without errors.

```bash
cargo test test_parse_external_pinescript_indicators
```

### 3. Binary: `pine-parser`

Standalone binary for manual testing.

```bash
# Parse from stdin
echo "indicator('test')" | cargo run --bin pine-parser -- --stdin

# Parse from file
cargo run --bin pine-parser -- path/to/file.pine
```

## Workflow for Fixing Parser Errors

When you encounter a syntax parsing error, follow these steps:

### Step 1: Create a Minimal Reproduction

1. Use the `pine-parser` binary to verify the error exists:
   ```bash
   echo "your_problematic_code" | cargo run --bin pine-parser -- --stdin
   ```
2. Reduce the failing code to the smallest possible reproduction case

### Step 2: Create a Test Case

1. Create a new file in `testdata/` with a descriptive name (e.g., `testdata/function_call_with_trailing_comma.pine`)
2. Put your minimal reproduction case in this file
3. Manually create the corresponding `_ast.json` file showing the expected/correct AST structure
   - You can use similar test cases as reference
   - Or use `GENERATE_AST=1` after fixing the parser to generate it

### Step 3: Verify Test Fails

Run the specific test to confirm it fails:

```bash
TEST_FILE=your_test_name cargo test test_parse_testdata_files
```

### Step 4: Fix the Parser

1. Make changes to the parser code to handle the syntax pattern
2. Iterate using the filtered test:
   ```bash
   # Run with debug to see tokens/AST
   DEBUG=1 TEST_FILE=your_test_name cargo test test_parse_testdata_files
   ```
3. Continue until your specific test passes

### Step 5: Validate Against Full Suite

Once your test passes, run the full test suite:

```bash
# Run all testdata tests
cargo test test_parse_testdata_files

# Run corpus tests
cargo test test_parse_external_pinescript_indicators
```

### Step 6: Fix Any Regressions

If `test_parse_external_pinescript_indicators` fails:

1. Identify which indicator file broke
2. Extract the problematic syntax from that indicator
3. Go back to Step 1 and create a new focused test case
4. Repeat the workflow

## Tips

- Always work with the smallest possible reproduction case in `testdata/`
- Use `DEBUG=1` to understand what tokens and AST the parser is producing
- Use `TEST_FILE` to iterate quickly on a single test
- Use `GENERATE_AST=1` carefully - only after you've verified the parser produces correct output
- The corpus tests (`test_parse_external_pinescript_indicators`) are your safety net for regressions
- Keep test cases in `testdata/` focused and minimal - one syntax pattern per file when possible

## Common Commands Summary

```bash
# Quick iteration on a fix
DEBUG=1 TEST_FILE=mytest cargo test test_parse_testdata_files

# Generate AST after fixing parser
GENERATE_AST=1 TEST_FILE=mytest cargo test test_parse_testdata_files

# Full validation
cargo test test_parse_testdata_files && cargo test test_parse_external_pinescript_indicators

# Manual testing
echo "code" | cargo run --bin pine-parser -- --stdin
cargo run --bin pine-parser -- path/to/file.pine
```
