//! Ad-hoc CLI: parse each .pine path given as an argument and print sema errors.
//! `cargo run -p pine-sema --example scan -- <file>...`
use pine_ast::Program;
use pine_lexer::Lexer;
use pine_parser::Parser;

fn main() {
    for path in std::env::args().skip(1) {
        let src = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{path}: read error: {e}");
                continue;
            }
        };
        let tokens = match Lexer::new(&src).tokenize() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let statements = match Parser::new(tokens).parse() {
            Ok(s) => s,
            Err(_) => continue,
        };
        for d in pine_sema::analyze(&Program::new(statements)) {
            println!("{path}: {d}");
        }
    }
}
