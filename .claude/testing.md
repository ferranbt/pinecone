# Testing Guide for Pinecone

## Overview

Testing is **critical** for Pinecone. Every interpreter feature, builtin function, and language construct **MUST** have both positive (success) and negative (error) test cases.

## Test Organization

Tests are located in `/tests/testdata/` and organized by functionality:

```
tests/testdata/
├── array/           # Array builtin tests
├── matrix/          # Matrix builtin tests
├── builtins/        # General builtin functions
├── operators/       # Language operators (+, -, *, etc.)
├── control_flow/    # if/else, switch, ternary
├── loops/           # for, while, for-in
├── functions/       # User-defined functions
├── types/           # User-defined types, enums, methods
├── errors/          # Error handling tests
├── series/          # Series and historical references
├── math/            # Math namespace
...
```

## Test File Format

### Success Tests

Test files use the `.pine` extension and follow this format:

```pine
// Descriptive comment about what this tests
var x = 5
var y = 10
log.info(x + y)
log.info(x * y)

// Expected output:
// 15
// 50
```

**Key points:**

- Use `log.info()` to output results
- Add `// Expected output:` comment followed by expected values (one per line)
- Each `log.info()` call should match one line in expected output
- Comments after `//` in expected output are ignored

### Error Tests

Error test files test that invalid code produces the expected error:

```pine
// Test that division by zero produces an error
var x = 10 / 0
log.info(x)

// Expected error: division by zero
```

**Key points:**

- The test expects a runtime error
- The comment must contain `// Expected error:` followed by text that should appear in the error message
- The test framework checks that the error message **contains** the specified text (case-insensitive substring match)

### Example Error Tests

```pine
// Test missing type parameter
m = matrix.new(2, 3, 0.0)

// Expected error: Missing type parameter
```

```pine
// Test invalid type
m = matrix.new<foobar>(2, 3, 0.0)

// Expected error: Invalid matrix element type
```

## Running Tests

### Using Just (Recommended)

```bash
# Run all integration tests
just test-integration

# Run a specific test file
just test-integration-test basic_operations.pine
```

## Test Requirements

### For Every Feature

When adding ANY new feature to Pinecone, you **MUST** create:

1. **Positive Test**: Test that the feature works correctly
   - Test basic functionality
   - Test edge cases
   - Test interaction with other features

2. **Negative Test**: Test that invalid usage produces appropriate errors
   - Test missing required parameters
   - Test invalid parameter types
   - Test invalid parameter values
   - Test out-of-bounds access (if applicable)

### For Builtin Functions

When adding a builtin function, create tests for:

1. **Basic usage** - The happy path
2. **All parameter variations** - Optional params, different types
3. **Type parameters** - If using generics like `<type>`
4. **Error cases**:
   - Missing required parameters
   - Invalid type arguments
   - Invalid parameter values
   - Out-of-bounds/out-of-range errors

### Example: Complete Test Coverage for `matrix.new<type>()`

```
tests/testdata/matrix/
├── basic_operations.pine       # Basic create, get, set
├── typed_matrices.pine          # All valid types (int, float, string, bool)
├── transpose.pine              # Transpose operation
├── fill_copy.pine              # Fill and copy operations
├── error_no_type.pine          # Error: missing type parameter
└── error_invalid_type.pine     # Error: invalid type like "foobar"
```

## Test Implementation Details

The test framework (`tests/lib.rs`) works as follows:

1. **Discovers** all `.pine` files in `/tests/testdata/`
2. **Parses** each file to extract:
   - The PineScript code
   - Expected output (from `// Expected output:` comments)
   - Expected error (from `// Expected error:` comments)
3. **Executes** the code through the interpreter
4. **Compares** actual output/errors with expected values
5. **Reports** ✅ for passing tests, ❌ for failures

### Success Test Validation

- Captures all `log.info()` outputs
- Compares line-by-line with expected output
- Ignores whitespace differences
- Reports mismatch if output doesn't match

### Error Test Validation

- Expects code to fail with a RuntimeError
- Checks that error message contains expected text (case-insensitive)
- Fails if no error occurs
- Fails if error doesn't contain expected text

## Best Practices

### 1. One Concept Per Test

Each test file should focus on testing one specific feature or behavior.

**Good:**

- `matrix/typed_matrices.pine` - Tests all matrix types
- `matrix/transpose.pine` - Tests transpose operation

**Bad:**

- `matrix/everything.pine` - Tests everything matrix-related

### 2. Clear Test Names

File names should clearly indicate what is being tested.

**Good:**

- `error_no_type.pine` - Clear what error is being tested
- `typed_matrices.pine` - Clear it tests type variations

**Bad:**

- `test1.pine`
- `misc.pine`

### 3. Descriptive Comments

Start each test with a comment explaining what it tests.

```pine
// Test that matrix.new<type>() works with all valid types
// and preserves type information through operations
```

### 4. Comprehensive Error Testing

For every way a function can fail, write an error test.

**Example for `matrix.new<type>(rows, cols, value)`:**

- ❌ No type parameter: `matrix.new(2, 3, 0)`
- ❌ Invalid type: `matrix.new<invalid>(2, 3, 0)`
- ❌ Negative rows: `matrix.new<int>(-1, 3, 0)`
- ❌ Non-number rows: `matrix.new<int>("foo", 3, 0)`

### 5. Test Edge Cases

Always test boundary conditions:

- Empty collections
- Single elements
- Maximum values
- Zero values
- Negative values (where applicable)

### 6. Use Expected Output Comments

Always include expected output for clarity:

```pine
log.info(matrix.rows(m))
log.info(matrix.columns(m))

// Expected output:
// 2
// 3
```

## Adding New Test Categories

To add a new test category:

1. Create a new directory in `/tests/testdata/`
2. Add `.pine` files following the format above
3. Tests are automatically discovered and run
4. No code changes needed to the test runner

## Debugging Failed Tests

When a test fails:

1. Check the error message for what went wrong
2. Run the specific test with `TEST_FILE=name.pine`
3. Compare expected vs actual output
4. Use `env DEBUG=1 TEST_FILE=name.pine cargo test` for verbose output
5. Fix the code or update the expected output

## Summary

**Remember:**

- ✅ Every feature needs tests
- ✅ Every feature needs error tests
- ✅ Use `log.info()` for output
- ✅ Use `// Expected output:` and `// Expected error:` comments
- ✅ Run tests with `just test-integration` or cargo
- ✅ Test files are automatically discovered
- ✅ One concept per test file
- ✅ Clear, descriptive file names and comments

Testing is not optional - it ensures Pinecone works correctly and prevents regressions!
