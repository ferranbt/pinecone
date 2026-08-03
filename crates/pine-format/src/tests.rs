use std::path::{Path, PathBuf};

use pine_ast::Program;
use pine_parser::Parser;

use crate::format;

fn ast(source: &str) -> Option<Program> {
    Parser::parse_source(source).ok()
}

fn pine_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(dir, &mut files);
    files.sort();
    files
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("pine") {
            out.push(path);
        }
    }
}

/// Each fixture in `tests/fixtures/` is `input`, a `----` line, then the
/// expected output. One test drives them all; the file name is the case name.
#[test]
fn fixtures() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut failures = Vec::new();

    for file in pine_files(&dir) {
        let name = file.file_stem().unwrap().to_string_lossy().into_owned();
        let content = std::fs::read_to_string(&file).expect("read fixture");
        let (before, after) = content
            .split_once("\n----\n")
            .unwrap_or_else(|| panic!("{name}: fixture needs a `----` separator line"));

        let got = format(&format!("{before}\n")).expect("fixture formats");
        if got != after {
            failures.push(format!(
                "\n[{name}]\n--- expected ---\n{after}--- got ---\n{got}"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Formatting must not change the parsed program, and must be idempotent, for
/// every fixture in the shared `.pine` corpus.
#[test]
fn preserves_ast_and_is_idempotent() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/testdata");
    let mut ast_diff = Vec::new();
    let mut non_idempotent = Vec::new();

    for file in pine_files(&corpus) {
        let source = std::fs::read_to_string(&file).expect("read fixture");
        let Some(original) = ast(&source) else {
            continue;
        };

        // Fixtures for unsupported versions etc. don't format; skip those.
        let Ok(formatted) = format(&source) else {
            continue;
        };

        match ast(&formatted) {
            Some(reparsed) if reparsed == original => {}
            _ => ast_diff.push(file.clone()),
        }
        if format(&formatted).ok().as_deref() != Some(formatted.as_str()) {
            non_idempotent.push(file.clone());
        }
    }

    assert!(
        ast_diff.is_empty() && non_idempotent.is_empty(),
        "ast-diff: {ast_diff:#?}\nnon-idempotent: {non_idempotent:#?}"
    );
}
