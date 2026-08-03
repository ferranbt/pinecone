//! An opinionated formatter for Pine Script.
//!
//! [`format`] lexes and parses the source, then renders the AST back through a
//! Prettier-style document layout. It preserves `//` comments (carried as lexer
//! trivia) and normalizes spacing, indentation, and line wrapping. Formatting
//! requires the source to parse; a lex or parse error is returned unchanged.
//!
//! # Example
//!
//! ```
//! let src = "x=close+1\n";
//! assert_eq!(pine_format::format(src).unwrap(), "x = close + 1\n");
//! ```

mod comments;
mod doc;
mod rules;

#[cfg(test)]
mod tests;

use std::fmt;

use pine_ast::Program;
use pine_core::{PineVersion, VersionError};
use pine_lexer::{Lexer, LexerError};
use pine_parser::{Parser, ParserError};

/// The target line width before a bracketed construct wraps.
const MAX_WIDTH: usize = 100;

/// Why formatting could not run: the source did not lex or parse.
#[derive(Debug)]
pub enum FormatError {
    Version(VersionError),
    Lex(LexerError),
    Parse(ParserError),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Version(e) => write!(f, "{e}"),
            FormatError::Lex(e) => write!(f, "{e}"),
            FormatError::Parse(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for FormatError {}

/// Format `source`, returning canonical Pine text. Returns [`FormatError`] when
/// the source does not lex or parse.
pub fn format(source: &str) -> Result<String, FormatError> {
    let version = PineVersion::detect(source)
        .map_err(FormatError::Version)?
        .unwrap_or(PineVersion::LATEST);

    let tokens = Lexer::with_version(source, version)
        .tokenize()
        .map_err(FormatError::Lex)?;

    let statements = Parser::new(tokens.clone())
        .parse()
        .map_err(FormatError::Parse)?;
    let program = Program::new(statements);

    let comments = comments::Comments::extract(&tokens);
    let document = rules::Rules::new(comments).program(&program);

    let laid_out = doc::layout(&document, MAX_WIDTH);
    if laid_out.is_empty() {
        return Ok(laid_out);
    }

    // Blank lines land with indentation; trim every line's trailing whitespace.
    let mut out: String = laid_out
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    out.push('\n');
    Ok(out)
}
