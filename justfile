generate-ast:
    GENERATE_AST=1 cargo test --package pine-parser --lib -- tests::test_parse_testdata_files

# Query pine script reference (lists if prefix, shows content if exact match)
ref path="":
    cargo run --package pine-reference -- query {{path}}

# Run integration tests with verbose output
test-integration:
    cargo test -p pine-integration-tests -- --nocapture

# Run integration tests for a specific test
test-integration-test test_name:
    TEST_FILE={{test_name}} cargo test -p pine-integration-tests -- --nocapture

# Run all benchmarks
bench:
    cargo bench

# Run a specific benchmark (e.g. just bench-one lexer)
bench-one name:
    cargo bench --bench {{name}}
