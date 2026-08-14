use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use pine_lang::diagnostics::{Diagnostic, Severity};

#[derive(Parser)]
#[command(name = "pinecone", version, about = "Pine Script tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Format scripts in place.
    Format {
        /// Files, or directories searched for `.pine` files.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Write to stdout instead of the file.
        #[arg(long)]
        stdout: bool,
        /// Report unformatted files without writing; exit non-zero if any.
        #[arg(long)]
        check: bool,
    },
    /// Report lint findings (repainting, lookahead, …).
    Lint {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    /// Parse, semantically analyze and lint.
    Check {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    /// Run the language server over stdio (for editor integration).
    Lsp {
        /// Accepted for editor compatibility; communication is always stdio.
        #[arg(long)]
        stdio: bool,
    },
}

/// What a command was pointed at.
enum Target {
    File(PathBuf),
    Dir(PathBuf),
    Many(Vec<PathBuf>),
}

impl Target {
    fn new(mut paths: Vec<PathBuf>) -> Self {
        if paths.len() == 1 {
            let path = paths.pop().expect("one path");
            if path.is_dir() {
                Target::Dir(path)
            } else {
                Target::File(path)
            }
        } else {
            Target::Many(paths)
        }
    }

    /// The `.pine` files this target resolves to, directories searched recursively.
    fn files(&self) -> eyre::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        match self {
            Target::File(file) => files.push(file.clone()),
            Target::Dir(dir) => collect_dir(dir, &mut files)?,
            Target::Many(paths) => {
                for path in paths {
                    if path.is_dir() {
                        collect_dir(path, &mut files)?;
                    } else {
                        files.push(path.clone());
                    }
                }
            }
        }
        files.sort();
        Ok(files)
    }
}

fn main() -> ExitCode {
    let ok = match Cli::parse().command {
        Command::Format {
            paths,
            stdout,
            check,
        } => each_file(paths, |file, source| format(file, source, stdout, check)),
        Command::Lint { paths } => each_file(paths, |file, source| {
            let diagnostics = pine_lang::lint_source(source)?;
            report(file, &diagnostics);
            Ok(diagnostics.is_empty())
        }),
        Command::Check { paths } => each_file(paths, |file, source| {
            // Resolve `import`s relative to the script's own directory.
            let root = file.parent().unwrap_or(Path::new(".")).to_path_buf();
            let loader = pine_lang::DirLoader::new(vec![root]);
            let diagnostics = pine_lang::check(source, Some(&loader))?;
            report(file, &diagnostics);
            Ok(!diagnostics.iter().any(|d| d.severity == Severity::Error))
        }),
        Command::Lsp { .. } => {
            pine_lsp::run();
            Ok(true)
        }
    };

    match ok {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

/// Run `op` over every file the target resolves to, returning whether all
/// succeeded.
fn each_file(
    paths: Vec<PathBuf>,
    mut op: impl FnMut(&Path, &str) -> eyre::Result<bool>,
) -> eyre::Result<bool> {
    let mut ok = true;
    for file in Target::new(paths).files()? {
        let source =
            fs::read_to_string(&file).map_err(|e| eyre::eyre!("{}: {e}", file.display()))?;
        ok &= op(&file, &source)?;
    }
    Ok(ok)
}

fn format(file: &Path, source: &str, stdout: bool, check: bool) -> eyre::Result<bool> {
    let formatted =
        pine_lang::format::format(source).map_err(|e| eyre::eyre!("{}: {e}", file.display()))?;
    if check {
        let formatted_already = formatted == source;
        if !formatted_already {
            println!("{}: not formatted", file.display());
        }
        Ok(formatted_already)
    } else if stdout {
        print!("{formatted}");
        Ok(true)
    } else {
        if formatted != source {
            fs::write(file, formatted).map_err(|e| eyre::eyre!("{}: {e}", file.display()))?;
        }
        Ok(true)
    }
}

fn report(file: &Path, diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        println!("{}: {diagnostic}", file.display());
    }
}

fn collect_dir(dir: &Path, out: &mut Vec<PathBuf>) -> eyre::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_dir(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("pine") {
            out.push(path);
        }
    }
    Ok(())
}
