#[cfg(test)]
mod tests {
    use pine_core::{SymInfo, Timeframe, TimeframeUnit};
    use pine_data::StaticProvider;
    use pine_interpreter::{AlertConditionOutput, DefaultPineOutput, LibraryLoader, LogOutput};
    use pine_lang::ScriptBuilder;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn load_test_data(path: PathBuf) -> StaticProvider {
        StaticProvider::from_csv(&path)
            .expect("bar fixture should load")
            .with_syminfo(test_syminfo())
    }

    const TICKER_NAME: &str = "AAPL";

    /// The timeframe of the shared bars.csv
    const DEFAULT_TIMEFRAME: Timeframe = Timeframe {
        multiplier: 1,
        unit: TimeframeUnit::Seconds,
    };

    enum ExpectedResult {
        Output(Vec<String>),
        Error(Vec<String>),
    }

    /// Extract the expected result from the `// Expected output:` or
    /// `// Expected error:` marker and the comment lines that follow it.
    fn extract_expected_result(source: &str) -> eyre::Result<ExpectedResult> {
        if source.contains("// Expected output:") {
            Ok(ExpectedResult::Output(collect_expected_block(
                source,
                "// Expected output:",
            )))
        } else if source.contains("// Expected error:") {
            Ok(ExpectedResult::Error(collect_expected_block(
                source,
                "// Expected error:",
            )))
        } else {
            Err(eyre::eyre!("failed to decode expected result"))
        }
    }

    /// Collect an expected block: any inline text after `marker`, plus the
    /// following `//` comment lines, stopping at the first non-comment line.
    fn collect_expected_block(source: &str, marker: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let mut in_section = false;
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix(marker) {
                in_section = true;
                let rest = rest.trim();
                if !rest.is_empty() {
                    lines.push(rest.to_string());
                }
                continue;
            }
            if in_section {
                if let Some(stripped) = trimmed.strip_prefix("//") {
                    let value = stripped.trim();
                    if !value.is_empty() {
                        lines.push(value.to_string());
                    }
                } else if !trimmed.is_empty() {
                    break;
                }
            }
        }
        lines
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
        fn load_library(&self, path: &str) -> Result<String, String> {
            let file_path = self.base_path.join(format!("{}.pine", path));
            fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to load library {}: {}", path, e))
        }
    }

    fn directive<T: std::str::FromStr>(source: &str, marker: &str) -> Option<T> {
        source
            .lines()
            .find_map(|l| l.trim().strip_prefix(marker))
            .and_then(|rest| rest.trim().parse().ok())
    }

    /// Fixed symbol information every script is compiled with, so a fixture can
    /// assert `syminfo.*` against known values.
    fn test_syminfo() -> SymInfo {
        SymInfo {
            ticker: TICKER_NAME.to_string(),
            tickerid: "NASDAQ:AAPL".to_string(),
            description: "Apple Inc.".to_string(),
            prefix: "NASDAQ".to_string(),
            currency: "USD".to_string(),
            basecurrency: "USD".to_string(),
            type_: "stock".to_string(),
            mintick: 0.01,
            pointvalue: 1.0,
            timezone: "America/New_York".to_string(),
            session: "0930-1600".to_string(),
        }
    }

    fn execute_pine_script_with_logger(source: &str) -> eyre::Result<Vec<String>> {
        let library_loader = TestLibraryLoader::new();

        // Use the `// Data: <name>` to choose which data set of ohlc bars
        // defaults to bars.csv.
        let name = directive::<String>(source, "// Data:").unwrap_or_else(|| "bars.csv".into());
        let data_file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join(name);

        let provider = load_test_data(data_file);

        // Use `// Timeframe:` to set a custom timeframe, defaults to 1 second
        let timeframe =
            directive::<Timeframe>(source, "// Timeframe:").unwrap_or(DEFAULT_TIMEFRAME);

        // Use `// Bars: N` to choose how many bars to use.
        let bar_count = directive::<usize>(source, "// Bars:").unwrap_or(1);

        let outputs = ScriptBuilder::<DefaultPineOutput>::with_code(source)
            .with_library_loader(Box::new(library_loader))
            .with_ticker(TICKER_NAME.to_string())
            .with_timeframe(timeframe)
            .with_bar_count(bar_count)
            .with_request_provider(Box::new(provider))
            .compile()?
            .run()?
            .outputs;

        let mut logs: Vec<String> = outputs
            .iter()
            .flat_map(|output| output.get_logs())
            .map(|log| log.message.clone())
            .collect();

        // After the logs, surface the final bar's declared alert conditions as
        // `(alert): <message>` lines so fixtures can assert them.
        if let Some(output) = outputs.last() {
            for alert in output.alertconditions() {
                logs.push(format!("(alert): {}", alert.message));
            }
        }
        Ok(logs)
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
                    let expected_error = expected_error.join("\n");
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
