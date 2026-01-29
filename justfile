generate-ast: GENERATE_AST=1 cargo test --package pine-parser --lib -- tests::test_parse_testdata_files

# Query pine script reference (lists if prefix, shows content if exact match)
ref path="": cargo run --package pine-reference -- query {{path}}
