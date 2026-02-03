#[cfg(test)]
mod tests {
    use pine::{builtins::LogLevel, builtins::Logger, Script};
    use pine_interpreter::Bar;
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;
    use std::rc::Rc;

    /// A logger that captures output for testing
    struct CapturingLogger {
        output: Rc<RefCell<Vec<String>>>,
    }

    impl CapturingLogger {
        fn new() -> (Self, Rc<RefCell<Vec<String>>>) {
            let output = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    output: output.clone(),
                },
                output,
            )
        }
    }

    impl Logger for CapturingLogger {
        fn log(&self, _level: LogLevel, msg: &str) {
            self.output.borrow_mut().push(msg.to_string());
        }
    }

    /// Extract expected output from comments at the end of a Pine script
    fn extract_expected_output(source: &str) -> Vec<String> {
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

        expected
    }

    fn execute_pine_script_with_logger(source: &str) -> Result<Vec<String>, String> {
        let (logger, output) = CapturingLogger::new();

        let mut script =
            Script::compile(source, Some(logger)).map_err(|e| format!("Compile error: {}", e))?;

        // Execute with a single bar of dummy data
        let bar = Bar {
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
        };

        script
            .execute(&bar)
            .map_err(|e| format!("Runtime error: {}", e))?;

        // Clone the output before returning
        let result = output.borrow().clone();
        Ok(result)
    }

    #[test]
    fn test_integration_scripts() {
        let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");

        let mut test_count = 0;
        let mut failures = Vec::new();

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

            test_count += 1;

            let source = fs::read_to_string(path).unwrap();

            // Extract expected output from comments
            let expected = extract_expected_output(&source);

            // Execute and capture actual output
            let result = execute_pine_script_with_logger(&source);

            match result {
                Ok(actual) => {
                    if actual != expected {
                        failures.push(format!(
                            "\n❌ Test: {}\n   Expected: {:?}\n   Actual:   {:?}",
                            relative_path, expected, actual
                        ));
                    }
                }
                Err(err) => {
                    failures.push(format!("\n❌ Test: {} - Error: {}", relative_path, err));
                }
            }
        }

        if !failures.is_empty() {
            panic!(
                "\n{} test(s) failed out of {}:\n{}",
                failures.len(),
                test_count,
                failures.join("\n")
            );
        }

        println!("✅ All {} integration test(s) passed!", test_count);
    }
}
