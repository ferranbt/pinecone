use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unterminated string at line {line}, column {column}")]
    UnterminatedString { line: usize, column: usize },

    #[error("Invalid hex color format '{value}' at line {line}, column {column}")]
    InvalidHexColor { value: String, line: usize, column: usize },

    #[error("Unexpected character '{ch}' at line {line}, column {column}")]
    UnexpectedCharacter { ch: char, line: usize, column: usize },

    #[error("Indentation error at line {line}")]
    IndentationError { line: usize },

    #[error("Invalid number '{value}' at line {line}, column {column}")]
    InvalidNumber { value: String, line: usize, column: usize },
}

// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Bool(bool),
    HexColor(String), // #RRGGBB or #RRGGBBAA

    // Identifiers and keywords
    Ident(String),
    Var,
    If,
    Else,
    For,
    While,
    Break,
    Continue,
    To,
    Switch, // keywords
    Int,
    Float, // type keywords
    Na,    // special value
    And,
    Or,
    Not, // logical operators

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Assign,
    ColonAssign,
    Arrow, // =, :=, =>
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign, // +=, -=, *=, /=

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Question,
    Newline,
    Indent,
    Dedent,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,   // Stack of indentation levels
    pending_tokens: Vec<Token>, // Queue for Indent/Dedent tokens
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0], // Start with base indentation level
            pending_tokens: vec![],
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.current).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.current += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn scan_number(&mut self) -> Result<Token, LexerError> {
        let start_line = self.line;
        let start_col = self.column;
        let mut num_str = String::new();

        // Handle numbers starting with '.' like .5 or .088
        if self.peek() == Some('.') {
            num_str.push('.');
            self.advance();
        }

        while let Some(ch) = self.peek() {
            if ch.is_numeric() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !num_str.contains('.') {
                // Only consume '.' if we haven't seen one yet and it's followed by a digit
                if let Some(next_ch) = self.input.get(self.current + 1) {
                    if next_ch.is_numeric() {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let value = num_str.parse::<f64>().map_err(|_| LexerError::InvalidNumber {
            value: num_str.clone(),
            line: start_line,
            column: start_col,
        })?;
        Ok(Token {
            typ: TokenType::Number(value),
            lexeme: num_str,
            line: start_line,
            column: start_col,
        })
    }

    fn scan_identifier(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.column;
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords
        let typ = match ident.as_str() {
            "var" => TokenType::Var,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "true" => TokenType::Bool(true),
            "false" => TokenType::Bool(false),
            "for" => TokenType::For,
            "while" => TokenType::While,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "to" => TokenType::To,
            "switch" => TokenType::Switch,
            "int" => TokenType::Int,
            "float" => TokenType::Float,
            "na" => TokenType::Na,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => TokenType::Ident(ident.clone()),
        };

        Token {
            typ,
            lexeme: ident,
            line: start_line,
            column: start_col,
        }
    }

    fn scan_string(&mut self, quote_char: char) -> Result<Token, LexerError> {
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // consume opening quote
        let mut string = String::new();

        while let Some(ch) = self.peek() {
            if ch == quote_char {
                self.advance();
                return Ok(Token {
                    typ: TokenType::String(string.clone()),
                    lexeme: format!("{}{}{}", quote_char, string, quote_char),
                    line: start_line,
                    column: start_col,
                });
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.advance() {
                    string.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        '"' => '"',
                        '\'' => '\'',
                        '\\' => '\\',
                        _ => escaped,
                    });
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Err(LexerError::UnterminatedString {
            line: start_line,
            column: start_col,
        })
    }

    fn scan_hex_color(&mut self) -> Result<Token, LexerError> {
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // consume '#'
        let mut hex = String::from("#");

        // Hex color format: #RRGGBB or #RRGGBBAA (6 or 8 hex digits)
        while let Some(ch) = self.peek() {
            if ch.is_ascii_hexdigit() {
                hex.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Validate length (should be 6 or 8 hex digits after #)
        let hex_len = hex.len() - 1;
        if hex_len != 6 && hex_len != 8 {
            return Err(LexerError::InvalidHexColor {
                value: hex,
                line: start_line,
                column: start_col,
            });
        }

        Ok(Token {
            typ: TokenType::HexColor(hex.clone()),
            lexeme: hex,
            line: start_line,
            column: start_col,
        })
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    typ: TokenType::Eof,
                    lexeme: String::new(),
                    line: self.line,
                    column: self.column,
                });
            }
        };

        let line = self.line;
        let col = self.column;

        let token = match ch {
            '+' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::PlusAssign,
                        lexeme: "+=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Plus,
                        lexeme: "+".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '-' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::MinusAssign,
                        lexeme: "-=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Minus,
                        lexeme: "-".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '*' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::StarAssign,
                        lexeme: "*=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Star,
                        lexeme: "*".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '/' => {
                self.advance();
                if self.peek() == Some('/') {
                    // Comment - skip to end of line
                    while self.peek().is_some() && self.peek() != Some('\n') {
                        self.advance();
                    }
                    return self.next_token();
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::SlashAssign,
                        lexeme: "/=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Slash,
                        lexeme: "/".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '%' => {
                self.advance();
                Token {
                    typ: TokenType::Percent,
                    lexeme: "%".to_string(),
                    line,
                    column: col,
                }
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::Equal,
                        lexeme: "==".to_string(),
                        line,
                        column: col,
                    }
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token {
                        typ: TokenType::Arrow,
                        lexeme: "=>".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Assign,
                        lexeme: "=".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::NotEqual,
                        lexeme: "!=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    return Err(LexerError::UnexpectedCharacter {
                        ch: '!',
                        line,
                        column: col,
                    });
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::LessEqual,
                        lexeme: "<=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Less,
                        lexeme: "<".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::GreaterEqual,
                        lexeme: ">=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Greater,
                        lexeme: ">".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '(' => {
                self.advance();
                Token {
                    typ: TokenType::LParen,
                    lexeme: "(".to_string(),
                    line,
                    column: col,
                }
            }
            ')' => {
                self.advance();
                Token {
                    typ: TokenType::RParen,
                    lexeme: ")".to_string(),
                    line,
                    column: col,
                }
            }
            '[' => {
                self.advance();
                Token {
                    typ: TokenType::LBracket,
                    lexeme: "[".to_string(),
                    line,
                    column: col,
                }
            }
            ']' => {
                self.advance();
                Token {
                    typ: TokenType::RBracket,
                    lexeme: "]".to_string(),
                    line,
                    column: col,
                }
            }
            ',' => {
                self.advance();
                Token {
                    typ: TokenType::Comma,
                    lexeme: ",".to_string(),
                    line,
                    column: col,
                }
            }
            '.' => {
                // Check if this is a decimal number like .5 or .088
                if let Some(next_ch) = self.input.get(self.current + 1) {
                    if next_ch.is_numeric() {
                        // This is a decimal number starting with .
                        return self.scan_number();
                    }
                }
                self.advance();
                Token {
                    typ: TokenType::Dot,
                    lexeme: ".".to_string(),
                    line,
                    column: col,
                }
            }
            ':' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token {
                        typ: TokenType::ColonAssign,
                        lexeme: ":=".to_string(),
                        line,
                        column: col,
                    }
                } else {
                    Token {
                        typ: TokenType::Colon,
                        lexeme: ":".to_string(),
                        line,
                        column: col,
                    }
                }
            }
            '?' => {
                self.advance();
                Token {
                    typ: TokenType::Question,
                    lexeme: "?".to_string(),
                    line,
                    column: col,
                }
            }
            '\n' => {
                self.advance();
                Token {
                    typ: TokenType::Newline,
                    lexeme: "\\n".to_string(),
                    line,
                    column: col,
                }
            }
            '"' => return self.scan_string('"'),
            '\'' => return self.scan_string('\''),
            '#' => return self.scan_hex_color(),
            _ if ch.is_numeric() => return self.scan_number(),
            _ if ch.is_alphabetic() || ch == '_' => self.scan_identifier(),
            _ => return Err(LexerError::UnexpectedCharacter {
                ch,
                line,
                column: col,
            }),
        };

        Ok(token)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];
        let mut at_line_start = true;

        loop {
            // Check if we have pending tokens (Indent/Dedent)
            if !self.pending_tokens.is_empty() {
                tokens.push(self.pending_tokens.remove(0));
                continue;
            }

            // Handle indentation at the start of a line
            if at_line_start {
                at_line_start = false;

                // Skip blank lines and comments
                let saved_line = self.line;
                let saved_col = self.column;

                // Count leading spaces
                let mut indent_level = 0;
                while let Some(ch) = self.peek() {
                    if ch == ' ' {
                        indent_level += 1;
                        self.advance();
                    } else if ch == '\t' {
                        indent_level += 4; // Treat tab as 4 spaces
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Check if this is a blank line or comment
                if let Some(ch) = self.peek() {
                    if ch == '\n' || ch == '\r' {
                        // Blank line - skip the newline and continue
                        self.advance();
                        at_line_start = true;
                        continue;
                    } else if ch == '/' && self.peek_ahead(1) == Some('/') {
                        // Comment line - skip to end of line
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        if self.peek() == Some('\n') {
                            self.advance();
                        }
                        at_line_start = true;
                        continue;
                    }
                } else {
                    // EOF - emit dedents for all remaining levels
                    let current_line = self.line;
                    let current_col = self.column;
                    while self.indent_stack.len() > 1 {
                        self.indent_stack.pop();
                        tokens.push(Token {
                            typ: TokenType::Dedent,
                            lexeme: String::new(),
                            line: current_line,
                            column: current_col,
                        });
                    }
                    tokens.push(Token {
                        typ: TokenType::Eof,
                        lexeme: String::new(),
                        line: current_line,
                        column: current_col,
                    });
                    break;
                }

                // Handle indent/dedent
                // SAFETY: indent_stack is initialized with vec![0] and we never pop the last element
                let current_indent = *self.indent_stack.last().unwrap();
                let line = saved_line;
                let col = saved_col;

                if indent_level > current_indent {
                    // Indent
                    self.indent_stack.push(indent_level);
                    tokens.push(Token {
                        typ: TokenType::Indent,
                        lexeme: String::new(),
                        line,
                        column: col,
                    });
                } else if indent_level < current_indent {
                    // Dedent - possibly multiple levels
                    // SAFETY: checked by len() > 1
                    while self.indent_stack.len() > 1
                        && *self.indent_stack.last().unwrap() > indent_level
                    {
                        self.indent_stack.pop();
                        tokens.push(Token {
                            typ: TokenType::Dedent,
                            lexeme: String::new(),
                            line,
                            column: col,
                        });
                    }

                    // Check for indentation error
                    // SAFETY: indent_stack always has at least one element
                    if *self.indent_stack.last().unwrap() != indent_level {
                        return Err(LexerError::IndentationError { line });
                    }
                }
            }

            // Get next token
            let token = self.next_token()?;

            // Check if this is a newline
            if matches!(token.typ, TokenType::Newline) {
                at_line_start = true;
                tokens.push(token);
            } else if matches!(token.typ, TokenType::Eof) {
                // Emit dedents for all remaining levels
                while self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    tokens.push(Token {
                        typ: TokenType::Dedent,
                        lexeme: String::new(),
                        line: token.line,
                        column: token.column,
                    });
                }
                tokens.push(token);
                break;
            } else {
                tokens.push(token);
            }
        }

        Ok(tokens)
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.input.get(self.current + offset).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literals() -> eyre::Result<()> {
        // Numbers
        let mut lexer = Lexer::new("42 3.14");
        let tokens = lexer.tokenize()?;
        assert!(matches!(tokens[0].typ, TokenType::Number(n) if n == 42.0));
        assert!(matches!(tokens[1].typ, TokenType::Number(n) if n == 3.14));

        // Strings
        let mut lexer = Lexer::new(r#""hello" "world\n""#);
        let tokens = lexer.tokenize()?;
        assert!(matches!(&tokens[0].typ, TokenType::String(s) if s == "hello"));
        assert!(matches!(&tokens[1].typ, TokenType::String(s) if s == "world\n"));

        // Booleans
        let mut lexer = Lexer::new("true false");
        let tokens = lexer.tokenize()?;
        assert!(matches!(tokens[0].typ, TokenType::Bool(true)));
        assert!(matches!(tokens[1].typ, TokenType::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_identifiers_and_keywords() -> eyre::Result<()> {
        let mut lexer = Lexer::new("my_var var if else for while int float na");
        let tokens = lexer.tokenize()?;
        assert!(matches!(&tokens[0].typ, TokenType::Ident(s) if s == "my_var"));
        assert!(matches!(tokens[1].typ, TokenType::Var));
        assert!(matches!(tokens[2].typ, TokenType::If));
        assert!(matches!(tokens[3].typ, TokenType::Else));
        assert!(matches!(tokens[4].typ, TokenType::For));
        assert!(matches!(tokens[5].typ, TokenType::While));
        assert!(matches!(tokens[6].typ, TokenType::Int));
        assert!(matches!(tokens[7].typ, TokenType::Float));
        assert!(matches!(tokens[8].typ, TokenType::Na));
        Ok(())
    }

    #[test]
    fn test_operators() -> eyre::Result<()> {
        let mut lexer = Lexer::new("+ - * / = == < >");
        let tokens = lexer.tokenize()?;
        assert!(matches!(tokens[0].typ, TokenType::Plus));
        assert!(matches!(tokens[1].typ, TokenType::Minus));
        assert!(matches!(tokens[2].typ, TokenType::Star));
        assert!(matches!(tokens[3].typ, TokenType::Slash));
        assert!(matches!(tokens[4].typ, TokenType::Assign));
        assert!(matches!(tokens[5].typ, TokenType::Equal));
        assert!(matches!(tokens[6].typ, TokenType::Less));
        assert!(matches!(tokens[7].typ, TokenType::Greater));
        Ok(())
    }

    #[test]
    fn test_delimiters() -> eyre::Result<()> {
        let mut lexer = Lexer::new("( ) [ ] , . : ? \n");
        let tokens = lexer.tokenize()?;
        assert!(matches!(tokens[0].typ, TokenType::LParen));
        assert!(matches!(tokens[1].typ, TokenType::RParen));
        assert!(matches!(tokens[2].typ, TokenType::LBracket));
        assert!(matches!(tokens[3].typ, TokenType::RBracket));
        assert!(matches!(tokens[4].typ, TokenType::Comma));
        assert!(matches!(tokens[5].typ, TokenType::Dot));
        assert!(matches!(tokens[6].typ, TokenType::Colon));
        assert!(matches!(tokens[7].typ, TokenType::Question));
        assert!(matches!(tokens[8].typ, TokenType::Newline));
        Ok(())
    }

    #[test]
    fn test_member_access() -> eyre::Result<()> {
        let mut lexer = Lexer::new("input.int ta.stoch");
        let tokens = lexer.tokenize()?;
        assert!(matches!(&tokens[0].typ, TokenType::Ident(s) if s == "input"));
        assert!(matches!(tokens[1].typ, TokenType::Dot));
        assert!(matches!(tokens[2].typ, TokenType::Int)); // 'int' is now a keyword
        assert!(matches!(&tokens[3].typ, TokenType::Ident(s) if s == "ta"));
        assert!(matches!(tokens[4].typ, TokenType::Dot));
        assert!(matches!(&tokens[5].typ, TokenType::Ident(s) if s == "stoch"));
        Ok(())
    }

    #[test]
    fn test_comments() -> eyre::Result<()> {
        let mut lexer = Lexer::new("42 // comment\n10");
        let tokens = lexer.tokenize()?;
        assert!(matches!(tokens[0].typ, TokenType::Number(n) if n == 42.0));
        assert!(matches!(tokens[1].typ, TokenType::Newline));
        assert!(matches!(tokens[2].typ, TokenType::Number(n) if n == 10.0));
        Ok(())
    }

    #[test]
    fn test_errors() {
        // Unterminated string
        let mut lexer = Lexer::new(r#""hello"#);
        assert!(lexer.tokenize().is_err());

        // Unexpected character
        let mut lexer = Lexer::new("@");
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn test_complex_expressions() -> eyre::Result<()> {
        // Variable declaration
        let mut lexer = Lexer::new("var x = 10");
        let tokens = lexer.tokenize()?;
        assert!(matches!(tokens[0].typ, TokenType::Var));
        assert!(matches!(&tokens[1].typ, TokenType::Ident(s) if s == "x"));
        assert!(matches!(tokens[2].typ, TokenType::Assign));
        assert!(matches!(tokens[3].typ, TokenType::Number(n) if n == 10.0));

        // Array access
        let mut lexer = Lexer::new("close[1]");
        let tokens = lexer.tokenize()?;
        assert!(matches!(&tokens[0].typ, TokenType::Ident(s) if s == "close"));
        assert!(matches!(tokens[1].typ, TokenType::LBracket));
        assert!(matches!(tokens[2].typ, TokenType::Number(n) if n == 1.0));
        assert!(matches!(tokens[3].typ, TokenType::RBracket));

        // Comparison
        let mut lexer = Lexer::new("x > 5");
        let tokens = lexer.tokenize()?;
        assert!(matches!(&tokens[0].typ, TokenType::Ident(s) if s == "x"));
        assert!(matches!(tokens[1].typ, TokenType::Greater));
        assert!(matches!(tokens[2].typ, TokenType::Number(n) if n == 5.0));
        Ok(())
    }
}
