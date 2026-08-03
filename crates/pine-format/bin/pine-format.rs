use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};

#[derive(Parser)]
#[command(name = "pine-format")]
#[command(about = "Format PineScript source", long_about = None)]
struct Cli {
    /// Input file to format. Reads from stdin when omitted or with --stdin.
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Read input from stdin instead of a file.
    #[arg(long)]
    stdin: bool,

    /// Rewrite the file in place instead of printing to stdout.
    #[arg(short, long, requires = "file")]
    write: bool,
}

fn main() {
    let cli = Cli::parse();

    let input = match &cli.file {
        Some(filename) if !cli.stdin => fs::read_to_string(filename).unwrap_or_else(|e| {
            eprintln!("Error reading file '{filename}': {e}");
            std::process::exit(1);
        }),
        _ => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .expect("Failed to read from stdin");
            buffer
        }
    };

    let formatted = match pine_format::format(&input) {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Format error: {e}");
            std::process::exit(1);
        }
    };

    if cli.write {
        let filename = cli.file.expect("--write requires a file");
        fs::write(&filename, formatted).unwrap_or_else(|e| {
            eprintln!("Error writing file '{filename}': {e}");
            std::process::exit(1);
        });
    } else {
        io::stdout().write_all(formatted.as_bytes()).ok();
    }
}
