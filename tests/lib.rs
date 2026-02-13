#[cfg(test)]
mod tests {
    use pine::Script;
    use pine_ast::Program;
    use pine_interpreter::{Bar, HistoricalDataProvider, LibraryLoader, Value};
    use pine_lexer::Lexer;
    use pine_parser::Parser;
    use std::cell::Cell;
    use std::fs;
    use std::path::Path;

    /// Generate synthetic OHLCV bar data for testing
    fn generate_test_bars(count: usize) -> Vec<Bar> {
        let mut bars = Vec::with_capacity(count);

        for i in 0..count {
            let base = 100.0 + (i as f64);
            bars.push(Bar {
                open: base,
                high: base + 5.0,
                low: base - 5.0,
                close: base + 2.0,
                volume: 1000.0 + (i as f64 * 10.0),
            });
        }

        bars
    }

    /// Mock historical data provider for tests
    struct TestHistoricalData {
        bars: Vec<Bar>,
        current_index: Cell<usize>,
    }

    impl TestHistoricalData {
        fn new(bars: Vec<Bar>) -> Self {
            Self {
                bars,
                current_index: Cell::new(0),
            }
        }

        fn set_current_bar(&self, index: usize) {
            self.current_index.set(index);
        }
    }

    impl HistoricalDataProvider for TestHistoricalData {
        fn get_historical(&self, series_id: &str, offset: usize) -> Option<Value> {
            let current_index = self.current_index.get();

            if current_index < offset {
                return None;
            }

            let bar_index = current_index - offset;
            if bar_index >= self.bars.len() {
                return None;
            }

            let bar = &self.bars[bar_index];
            let value = match series_id {
                "open" => bar.open,
                "high" => bar.high,
                "low" => bar.low,
                "close" => bar.close,
                "volume" => bar.volume,
                _ => return None,
            };

            Some(Value::Number(value))
        }
    }

    enum ExpectedResult {
        Output(Vec<String>),
        Error(String),
    }

    /// Extract expected output from comments at the end of a Pine script
    fn extract_expected_result(source: &str) -> eyre::Result<ExpectedResult> {
        if source.contains("// Expected output:") {
            let mut expected = Vec::new();
            let mut in_expected_section = false;

            for line in source.lines() {
                let trimmed = line.trim();

                // Check if we've reached the "Expected output:" marker
                if trimmed == "// Expected output:" {
                    in_expected_section = true;
                    continue;
                }

                // If we're in the expected section and line starts with //, extract the value
                if in_expected_section {
                    if let Some(stripped) = trimmed.strip_prefix("//") {
                        let value = stripped.trim();
                        if !value.is_empty() {
                            expected.push(value.to_string());
                        }
                    } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                        // Stop if we hit a non-comment line
                        break;
                    }
                }
            }
            Ok(ExpectedResult::Output(expected))
        } else if source.contains("// Expected error:") {
            for line in source.lines() {
                let trimmed = line.trim();
                if let Some(stripped) = trimmed.strip_prefix("// Expected error:") {
                    return Ok(ExpectedResult::Error(stripped.trim().to_string()));
                }
            }
            Err(eyre::eyre!("expected error but output not found"))
        } else {
            Err(eyre::eyre!("failed to decode expected error"))
        }
    }

    /// Library loader that loads from testdata/libraries directory
    struct TestLibraryLoader {
        base_path: std::path::PathBuf,
    }

    impl TestLibraryLoader {
        fn new() -> Self {
            let base_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join("libraries");
            Self { base_path }
        }
    }

    impl LibraryLoader for TestLibraryLoader {
        fn load_library(&self, path: &str) -> Result<pine_ast::Program, String> {
            let file_path = self.base_path.join(format!("{}.pine", path));

            let source = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to load library {}: {}", path, e))?;

            let mut lexer = Lexer::new(&source);
            let tokens = lexer
                .tokenize()
                .map_err(|e| format!("Lexer error: {:?}", e))?;
            let mut parser = Parser::new(tokens);
            let statements = parser
                .parse()
                .map_err(|e| format!("Parser error: {:?}", e))?;

            Ok(Program::new(statements))
        }
    }

    fn execute_pine_script_with_logger(source: &str) -> eyre::Result<Vec<String>> {
        let library_loader = TestLibraryLoader::new();

        let mut script = Script::compile(source)?;

        // Generate historical bar data for TA functions
        let bars = generate_test_bars(200);
        let historical_data = TestHistoricalData::new(bars.clone());
        historical_data.set_current_bar(bars.len() - 1);

        // Set up historical data provider
        script.set_historical_provider(Box::new(historical_data));
        script.set_library_loader(Box::new(library_loader));

        // Execute with the last bar
        let pine_output = script.execute(&bars[bars.len() - 1])?;

        // Extract log messages from output
        let result = pine_output.logs.iter().map(|log| log.message.clone()).collect();
        Ok(result)
    }

    #[test]
    fn test_integration_scripts() -> eyre::Result<()> {
        let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");

        let mut has_failed = false;
        let filter = std::env::var("TEST_FILE").ok();

        // Walk through all .pine files in testdata
        for entry in walkdir::WalkDir::new(&test_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pine"))
        {
            let path = entry.path();

            let relative_path = path
                .strip_prefix(&test_dir)
                .unwrap()
                .to_string_lossy()
                .to_string();

            if relative_path.contains("libraries/") {
                // Skip libraries since they do not have expected result
                continue;
            }

            let filename = path.file_name().unwrap().to_str().unwrap();

            // Skip if filter is set and doesn't match
            if let Some(ref filter_name) = filter {
                if filename != filter_name {
                    continue;
                }
            }

            let source = fs::read_to_string(path)?;
            let result = execute_pine_script_with_logger(&source);

            let expected = extract_expected_result(&source)?;

            match (expected, result) {
                (ExpectedResult::Output(expected_output), Ok(actual)) => {
                    if actual != expected_output {
                        println!(
                            "❌ {}\n   Expected: {:?}\n   Actual:   {:?}\n",
                            relative_path, expected_output, actual
                        );
                        has_failed = true;
                    } else {
                        println!("✅ {}", relative_path);
                    }
                }
                (ExpectedResult::Error(_), Ok(_)) => {
                    println!(
                        "❌ {} - Expected error but script succeeded\n",
                        relative_path
                    );
                    has_failed = true;
                }
                (ExpectedResult::Output(_), Err(err)) => {
                    println!("❌ {} - Error: {}\n", relative_path, err);
                    has_failed = true;
                }
                (ExpectedResult::Error(expected_error), Err(err)) => {
                    if err.to_string().contains(&expected_error) {
                        println!("✅ {}", relative_path);
                    } else {
                        println!(
                            "❌ {}\n   Expected error containing: {}\n   Actual error: {}\n",
                            relative_path, expected_error, err
                        );
                        has_failed = true;
                    }
                }
            }
        }

        if has_failed {
            Err(eyre::eyre!("At least one test failed"))
        } else {
            Ok(())
        }
    }
}
