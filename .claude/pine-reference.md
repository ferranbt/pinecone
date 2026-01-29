# Pine Script Reference Documentation

This repository includes a tool for querying the official Pine Script reference documentation.

## Overview

The `pine-reference` crate provides a CLI tool to query Pine Script language features (variables, constants, functions, etc.) from the official TradingView documentation. This is essential when implementing Pine Script language features to ensure correctness and completeness.

## When to Use the Reference Tool

**Always consult the reference when:**

- Implementing a new Pine Script built-in function or variable
- Fixing bugs related to built-in functionality
- Understanding the expected behavior of language features
- Verifying function signatures, parameters, and return types
- Checking which version of Pine Script a feature was introduced

## Quick Reference Command

Use the `just ref` command to query the reference:

```bash
# List all top-level sections
just ref

# List all items in a section
just ref Variables
just ref Functions

# List items with prefix matching
just ref Variables.bar
# Returns: bar_index, barstate.isconfirmed, barstate.isfirst, etc.

# Get full documentation for exact match
just ref Variables.ask
# Returns: Full documentation including description, type, examples, etc.
```

## How It Works

The tool uses smart matching:

1. **No argument**: Lists all top-level sections (Variables, Constants, Functions, etc.)
2. **Section name**: Lists all items under that section
3. **Prefix match**: Lists all items starting with the prefix
4. **Exact match**: Shows full content/documentation for that specific item

## Examples

### Example 1: Finding a Function

```bash
# Not sure of the exact name? Start with prefix
$ just ref Functions.ta.
ta.accdist
ta.alma
ta.atr
ta.bb
ta.bbw
...

# Found it! Get the full documentation
$ just ref Functions.ta.sma
# Returns full documentation with signature, parameters, examples
```

### Example 2: Exploring Variables

```bash
# See what's available
$ just ref Variables

# Narrow down to bar-related variables
$ just ref Variables.bar
bar_index
barstate.isconfirmed
barstate.isfirst
barstate.ishistory
barstate.islast
barstate.islastconfirmedhistory
barstate.isnew
barstate.isrealtime

# Get documentation for a specific variable
$ just ref Variables.bar_index
```

### Example 3: Implementation Workflow

When implementing `ta.sma()`:

1. **Query the reference**:
   ```bash
   just ref Functions.ta.sma
   ```

2. **Read the documentation** to understand:
   - Function signature: `ta.sma(source, length)`
   - Parameter types
   - Return type
   - Special behavior and edge cases
   - Example usage

3. **Implement the function** based on the official spec

4. **Test against examples** from the documentation

## Direct CLI Usage

You can also use the CLI directly without `just`:

```bash
# Query
cargo run --package pine-reference -- query [PATH]
```

## Architecture

- **lib.rs**: Core functionality (parsing markdown, querying sections)
- **bin/main.rs**: CLI interface
- **spec/v6.md**: The reference documentation (embedded at compile time)

The tool parses the markdown document to extract sections (## for categories, ### for items) and provides smart querying based on exact match vs prefix match.
