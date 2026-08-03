use std::collections::{HashMap, VecDeque};

use pine_lexer::{Token, TokenType};

/// A piece of leading trivia above a statement, in source order.
pub(crate) enum Lead {
    Comment(String),
    Blank,
}

/// The trivia recovered from the token stream: whole-line comments and blank
/// lines (anchored before the statement that follows them) and trailing
/// comments (kept on the code line they sit on).
pub(crate) struct Comments {
    leading: VecDeque<(u32, Lead)>,
    trailing: HashMap<u32, String>,
}

impl Comments {
    pub(crate) fn extract(tokens: &[Token]) -> Self {
        let mut leading = VecDeque::new();
        let mut trailing = HashMap::new();
        let mut last_code_line: Option<usize> = None;

        for token in tokens {
            match &token.typ {
                TokenType::Comment(text) => {
                    let rendered = render(text);
                    if last_code_line == Some(token.line) {
                        trailing.insert(token.line as u32, rendered);
                    } else {
                        leading.push_back((token.line as u32, Lead::Comment(rendered)));
                    }
                }
                TokenType::BlankLine => leading.push_back((token.line as u32, Lead::Blank)),
                TokenType::Newline | TokenType::Indent | TokenType::Dedent | TokenType::Eof => {}
                _ => last_code_line = Some(token.line),
            }
        }
        Self { leading, trailing }
    }

    /// The comments and blank lines that sit above the statement starting on
    /// `line`, in source order.
    pub(crate) fn take_leading(&mut self, line: Option<u32>) -> Vec<Lead> {
        let mut out = Vec::new();
        let Some(line) = line else { return out };
        while let Some((trivia_line, _)) = self.leading.front() {
            if *trivia_line < line {
                out.push(self.leading.pop_front().expect("front exists").1);
            } else {
                break;
            }
        }
        out
    }

    /// The comment trailing the code on `line`, if any.
    pub(crate) fn take_trailing(&mut self, line: Option<u32>) -> Option<String> {
        line.and_then(|line| self.trailing.remove(&line))
    }

    /// Trailing comments never anchored to a statement's code line, in source
    /// order. (Leftover leading trivia is drained via [`Comments::take_leading`]
    /// with an unbounded line.)
    pub(crate) fn drain_trailing(&mut self) -> Vec<String> {
        let mut rest: Vec<(u32, String)> = self.trailing.drain().collect();
        rest.sort_by_key(|(line, _)| *line);
        rest.into_iter().map(|(_, text)| text).collect()
    }
}

/// A comment's rendered form: `//` plus its verbatim text, trailing space
/// trimmed. Verbatim keeps annotations (`//@version=5`) and dividers intact.
fn render(text: &str) -> String {
    format!("//{}", text.trim_end())
}
