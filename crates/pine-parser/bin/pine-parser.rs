use clap::Parser;
use pine_lexer::Lexer;
use pine_parser::Parser as PineParser;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "pine-parser")]
#[command(about = "Parse PineScript files and output AST as JSON", long_about = None)]
struct Cli {
    /// Input file to parse. If not provided and --stdin is not set, reads from stdin
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Read input from stdin instead of a file
    #[arg(long)]
    stdin: bool,
}

fn main() {
    let cli = Cli::parse();

    // Read input
    let input = match cli.file {
        Some(filename) if !cli.stdin => {
            // Read from file
            fs::read_to_string(&filename).unwrap_or_else(|e| {
                eprintln!("Error reading file '{}': {}", filename, e);
                std::process::exit(1);
            })
        }
        _ => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .expect("Failed to read from stdin");
            buffer
        }
    };

    // Tokenize
    let mut lexer = Lexer::new(&input);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1);
        }
    };

    // Parse
    let mut parser = PineParser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            std::process::exit(1);
        }
    };

    // Output JSON
    match serde_json::to_string_pretty(&ast) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("JSON serialization error: {}", e);
            std::process::exit(1);
        }
    }
}
