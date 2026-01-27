
generate-ast:
    GENERATE_AST=1 cargo test --package pine-parser --lib -- tests::test_parse_testdata_files
