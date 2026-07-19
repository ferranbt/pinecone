//! Checks the compiler front-end against a corpus of real-world Pine scripts
//! from public repositories.
//!
//! The corpus is cloned on demand (it is gitignored).

use pine::Script;
use pine_core::PineVersion;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Public repositories that make up the corpus.
const CORPUS: &[&str] = &["https://github.com/everget/tradingview-pinescript-indicators.git"];

/// Clone every corpus repository that is not present yet.
fn init_corpus(corpus_dir: &Path) -> eyre::Result<()> {
    fs::create_dir_all(corpus_dir)?;

    for url in CORPUS {
        // "https://.../foo.git" -> "foo"
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .trim_end_matches(".git");

        let dest = corpus_dir.join(name);
        if dest.is_dir() {
            continue;
        }

        println!("Cloning {url}");
        let status = Command::new("git")
            .args(["clone", "--depth", "1", url])
            .arg(&dest)
            .status()?;
        if !status.success() {
            return Err(eyre::eyre!("failed to clone {url}"));
        }
    }

    Ok(())
}

#[test]
fn test_corpus() -> eyre::Result<()> {
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus");
    init_corpus(&corpus_dir)?;

    let mut has_failed = false;

    // Each top-level folder is one cloned repository.
    for repo in fs::read_dir(&corpus_dir)? {
        let repo = repo?;
        if !repo.file_type()?.is_dir() {
            continue;
        }

        for entry in walkdir::WalkDir::new(repo.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pine"))
        {
            let path = entry.path();
            let source = fs::read_to_string(path)?;

            // Skip scripts we do not target: no version annotation, or one this
            // toolchain does not support (v3 and older).
            if !matches!(PineVersion::detect(&source), Ok(Some(_))) {
                continue;
            }

            let relative_path = path.strip_prefix(&corpus_dir).unwrap_or(path);
            match Script::compile(&source, None) {
                Ok(_) => println!("✅ {}", relative_path.display()),
                Err(err) => {
                    println!("❌ {}\n{}\n", relative_path.display(), err);
                    has_failed = true;
                }
            }
        }
    }

    if has_failed {
        Err(eyre::eyre!("At least one corpus script failed"))
    } else {
        Ok(())
    }
}
