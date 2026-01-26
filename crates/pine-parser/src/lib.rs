pub use pine_ast::{Argument, BinOp, Expr, Literal, Program, Stmt, UnOp};
use pine_lexer::{Token, TokenType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Unexpected token: {0:?} at line {1}")]
    UnexpectedToken(TokenType, usize),

    #[error("Expected {expected} but found {found:?} at line {line}")]
    ExpectedToken {
        expected: String,
        found: TokenType,
        line: usize,
    },

    #[error("Expected variable name at line {0}")]
    ExpectedVariableName(usize),

    #[error("Expected parameter name at line {0}")]
    ExpectedParameterName(usize),

    #[error("Can only call identifiers or member access at line {0}")]
    InvalidCallTarget(usize),

    #[error("Expected identifier after '.' at line {0}")]
    ExpectedIdentifierAfterDot(usize),
}

impl From<ParserError> for String {
    fn from(err: ParserError) -> String {
        err.to_string()
    }
}

/// Helper trait to convert TokenType to operators
trait TokenTypeExt {
    fn to_binop(&self) -> Option<BinOp>;
}

impl TokenTypeExt for TokenType {
    /// Convert token type to binary operator, if applicable
    fn to_binop(&self) -> Option<BinOp> {
        match self {
            TokenType::Plus => Some(BinOp::Add),
            TokenType::Minus => Some(BinOp::Sub),
            TokenType::Star => Some(BinOp::Mul),
            TokenType::Slash => Some(BinOp::Div),
            TokenType::Percent => Some(BinOp::Mod),
            TokenType::Equal => Some(BinOp::Eq),
            TokenType::NotEqual => Some(BinOp::NotEq),
            TokenType::Less => Some(BinOp::Less),
            TokenType::Greater => Some(BinOp::Greater),
            TokenType::LessEqual => Some(BinOp::LessEq),
            TokenType::GreaterEqual => Some(BinOp::GreaterEq),
            TokenType::And => Some(BinOp::And),
            TokenType::Or => Some(BinOp::Or),
            TokenType::PlusAssign => Some(BinOp::Add),
            TokenType::MinusAssign => Some(BinOp::Sub),
            TokenType::StarAssign => Some(BinOp::Mul),
            TokenType::SlashAssign => Some(BinOp::Div),
            _ => None,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().typ, TokenType::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn check(&self, typ: &TokenType) -> bool {
        !self.is_at_end() && &self.peek().typ == typ
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for typ in types {
            if self.check(typ) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Try to parse something speculatively. If parsing fails, restore position and return None.
    /// This is useful for lookahead/backtracking scenarios.
    fn try_parse<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> Result<T, ParserError>,
    {
        let saved_pos = self.current;
        match f(self) {
            Ok(val) => Some(val),
            Err(_) => {
                self.current = saved_pos;
                None
            }
        }
    }

    /// Skip any newline tokens
    fn skip_newlines(&mut self) {
        while self.match_token(&[TokenType::Newline]) {}
    }

    /// Skip newlines, indents, and dedents (whitespace tokens)
    fn skip_whitespace(&mut self) {
        while self.match_token(&[TokenType::Newline, TokenType::Indent, TokenType::Dedent]) {}
    }

    /// Parse optional array suffix [] and append to type name if present
    fn parse_array_suffix(&mut self, type_name: String) -> Result<String, ParserError> {
        if self.match_token(&[TokenType::LBracket]) {
            self.consume(TokenType::RBracket, "Expected ']' after '[' in array type")?;
            Ok(format!("{}[]", type_name))
        } else {
            Ok(type_name)
        }
    }

    /// Parse an expression that may be on an indented continuation line.
    /// Handles: newlines + optional indent + expression + optional dedent
    fn parse_indented_expression(&mut self) -> Result<Expr, ParserError> {
        self.skip_newlines();

        // Check if expression is on an indented line
        let has_indent = self.match_token(&[TokenType::Indent]);

        let expr = self.expression()?;

        // Consume dedent if we had indent
        if has_indent {
            self.match_token(&[TokenType::Dedent]);
        }

        Ok(expr)
    }

    fn consume(&mut self, typ: TokenType, message: &str) -> Result<&Token, ParserError> {
        if self.check(&typ) {
            Ok(self.advance())
        } else {
            Err(ParserError::ExpectedToken {
                expected: message.to_string(),
                found: self.peek().typ.clone(),
                line: self.peek().line,
            })
        }
    }

    // Parse a program (top-level)
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = vec![];

        while !self.is_at_end() {
            // Skip any leading newlines and dedents (dedents at top level are from EOF)
            self.skip_whitespace();

            // Check if we reached EOF after skipping
            if self.is_at_end() {
                break;
            }

            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    // Declarations (var declarations, assignments, etc.)
    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        // Check for var or varip keyword (can be followed by type annotation)
        let is_varip = if self.match_token(&[TokenType::Varip]) {
            true
        } else if self.match_token(&[TokenType::Var]) {
            false
        } else {
            // Not a var/varip declaration, continue to other statement types
            return self.check_type_annotated_declaration();
        };

        // Check if followed by type annotation: var int x = ..., var float y = ..., var label l = ...
        let type_annotation = if self.match_token(&[TokenType::Int, TokenType::Float]) {
            let type_name = self.tokens[self.current - 1].lexeme.clone();
            // Check for array type: var int[] or var float[]
            Some(self.parse_array_suffix(type_name)?)
        } else if let TokenType::Ident(type_name) = &self.peek().typ {
            // Check if this is a type annotation by looking ahead for another identifier or []
            let type_name = type_name.clone();
            self.try_parse(|p| {
                p.advance(); // consume potential type name

                // Check for array type: var string[] or var label[]
                let final_type = p.parse_array_suffix(type_name.clone())?;

                // Must be followed by identifier to be a type annotation
                if !matches!(p.peek().typ, TokenType::Ident(_)) {
                    return Err(ParserError::ExpectedVariableName(p.peek().line));
                }

                Ok(final_type)
            })
        } else {
            None
        };
        return self.typed_var_declaration(type_annotation, is_varip);
    }

    fn check_type_annotated_declaration(&mut self) -> Result<Stmt, ParserError> {
        // Check for type declaration: type TypeName
        if self.match_token(&[TokenType::Type]) {
            return self.type_declaration();
        }

        // Check for enum declaration: enum EnumName
        if self.match_token(&[TokenType::Enum]) {
            return self.enum_declaration();
        }

        // Check for method declaration: method methodName(params) =>
        if self.match_token(&[TokenType::Method]) {
            return self.method_declaration();
        }

        // Check for type-annotated declaration without var: int x = ..., float y = ..., int[] x = ...
        if self.match_token(&[TokenType::Int, TokenType::Float]) {
            let type_name = self.tokens[self.current - 1].lexeme.clone();
            // Check for array type: int[] or float[]
            let type_name = self.parse_array_suffix(type_name)?;
            return self.typed_var_declaration(Some(type_name), false);
        }

        // Check for identifier type with optional []: string x = ..., string[] x = ...
        if let TokenType::Ident(type_name) = &self.peek().typ {
            let type_name = type_name.clone();
            if let Some(final_type) = self.try_parse(|p| {
                p.advance(); // consume type name

                // Check for array type: string[]
                let final_type = p.parse_array_suffix(type_name.clone())?;

                // Must be followed by identifier to be a type annotation
                if !matches!(p.peek().typ, TokenType::Ident(_)) {
                    return Err(ParserError::ExpectedVariableName(p.peek().line));
                }

                Ok(final_type)
            }) {
                return self.typed_var_declaration(Some(final_type), false);
            }
        }

        self.statement()
    }

    fn type_declaration(&mut self) -> Result<Stmt, ParserError> {
        // Parse type name
        let type_name = if let TokenType::Ident(name) = &self.peek().typ {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParserError::UnexpectedToken(self.peek().typ.clone(), self.peek().line));
        };

        // Expect newline before fields
        self.consume(TokenType::Newline, "Expected newline after type name")?;

        // Expect indent to start field block
        self.consume(TokenType::Indent, "Expected indent for type fields")?;

        // Parse fields
        let mut fields = Vec::new();

        loop {
            // Skip newlines between fields
            self.skip_newlines();

            // Check for dedent (end of type declaration)
            if self.check(&TokenType::Dedent) {
                self.advance();
                break;
            }

            // Check for end of file
            if self.is_at_end() {
                break;
            }

            // Parse field: type_annotation field_name [= default_value]
            // First, get the type annotation (int, float, or identifier)
            let field_type = if self.match_token(&[TokenType::Int, TokenType::Float]) {
                self.tokens[self.current - 1].lexeme.clone()
            } else if let TokenType::Ident(type_name) = &self.peek().typ {
                let type_name = type_name.clone();
                self.advance();
                type_name
            } else {
                return Err(ParserError::UnexpectedToken(self.peek().typ.clone(), self.peek().line));
            };

            // Parse field name
            let field_name = if let TokenType::Ident(name) = &self.peek().typ {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParserError::ExpectedVariableName(self.peek().line));
            };

            // Parse optional default value
            let default_value = if self.match_token(&[TokenType::Assign]) {
                Some(self.expression()?)
            } else {
                None
            };

            fields.push(pine_ast::TypeField {
                name: field_name,
                type_annotation: field_type,
                default_value,
            });
        }

        Ok(Stmt::TypeDecl {
            name: type_name,
            fields,
        })
    }

    fn enum_declaration(&mut self) -> Result<Stmt, ParserError> {
        // Parse enum name
        let enum_name = if let TokenType::Ident(name) = &self.peek().typ {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParserError::UnexpectedToken(self.peek().typ.clone(), self.peek().line));
        };

        // Expect newline before fields
        self.consume(TokenType::Newline, "Expected newline after enum name")?;

        // Expect indent to start field block
        self.consume(TokenType::Indent, "Expected indent for enum fields")?;

        // Parse fields
        let mut fields = Vec::new();

        loop {
            // Skip newlines between fields
            self.skip_newlines();

            // Check for dedent (end of enum declaration)
            if self.check(&TokenType::Dedent) {
                self.advance();
                break;
            }

            // Check for end of file
            if self.is_at_end() {
                break;
            }

            // Parse field: field_name [= "title"]
            let field_name = if let TokenType::Ident(name) = &self.peek().typ {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParserError::ExpectedVariableName(self.peek().line));
            };

            // Parse optional title
            let title = if self.match_token(&[TokenType::Assign]) {
                // Expect a string literal for the title
                if let TokenType::String(s) = &self.peek().typ {
                    let s = s.clone();
                    self.advance();
                    Some(s)
                } else {
                    return Err(ParserError::UnexpectedToken(self.peek().typ.clone(), self.peek().line));
                }
            } else {
                None
            };

            fields.push(pine_ast::EnumField {
                name: field_name,
                title,
            });
        }

        Ok(Stmt::EnumDecl {
            name: enum_name,
            fields,
        })
    }

    fn method_declaration(&mut self) -> Result<Stmt, ParserError> {
        // Parse method name
        let method_name = if let TokenType::Ident(name) = &self.peek().typ {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParserError::UnexpectedToken(self.peek().typ.clone(), self.peek().line));
        };

        // Expect '('
        self.consume(TokenType::LParen, "Expected '(' after method name")?;

        // Parse parameters
        let mut params = Vec::new();

        if !self.check(&TokenType::RParen) {
            loop {
                // Parse optional type annotation
                let type_annotation = if self.match_token(&[TokenType::Int, TokenType::Float]) {
                    Some(self.tokens[self.current - 1].lexeme.clone())
                } else if let TokenType::Ident(type_name) = &self.peek().typ {
                    // Could be a type annotation or parameter name
                    let saved_pos = self.current;
                    let type_name = type_name.clone();
                    self.advance();

                    // Check if followed by another identifier (param name)
                    if matches!(self.peek().typ, TokenType::Ident(_)) {
                        Some(type_name)
                    } else {
                        // It's actually the parameter name, backtrack
                        self.current = saved_pos;
                        None
                    }
                } else {
                    None
                };

                // Parse parameter name
                let param_name = if let TokenType::Ident(name) = &self.peek().typ {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err(ParserError::ExpectedParameterName(self.peek().line));
                };

                // Parse optional default value
                let default_value = if self.match_token(&[TokenType::Assign]) {
                    Some(self.expression()?)
                } else {
                    None
                };

                params.push(pine_ast::MethodParam {
                    type_annotation,
                    name: param_name,
                    default_value,
                });

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RParen, "Expected ')' after parameters")?;

        // Expect '=>'
        self.consume(TokenType::Arrow, "Expected '=>' after method parameters")?;

        // Skip optional newline after =>
        self.match_token(&[TokenType::Newline]);

        // Parse method body (can be a block or single expression)
        let body = self.parse_block()?;

        Ok(Stmt::MethodDecl {
            name: method_name,
            params,
            body,
        })
    }

    fn typed_var_declaration(
        &mut self,
        type_annotation: Option<String>,
        is_varip: bool,
    ) -> Result<Stmt, ParserError> {
        let name = if let TokenType::Ident(n) = &self.peek().typ {
            let name = n.clone();
            self.advance();
            name
        } else {
            return Err(ParserError::ExpectedVariableName(self.peek().line));
        };

        let initializer = if self.match_token(&[TokenType::Assign]) {
            Some(self.parse_indented_expression()?)
        } else {
            None
        };

        Ok(Stmt::VarDecl {
            name,
            type_annotation,
            initializer,
            is_varip,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        // Check for if statement
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }

        // Check for for loop
        if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        }

        // Check for while loop
        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }

        // Check for break
        if self.match_token(&[TokenType::Break]) {
            return Ok(Stmt::Break);
        }

        // Check for continue
        if self.match_token(&[TokenType::Continue]) {
            return Ok(Stmt::Continue);
        }

        // Check for tuple destructuring: [a, b, c] = func()
        // But only if followed by = (otherwise it's an array literal)
        if self.check(&TokenType::LBracket) {
            if let Some((names, value)) = self.try_parse(|p| {
                p.advance(); // consume [

                let mut names = vec![];

                // Parse identifiers separated by commas
                if !p.check(&TokenType::RBracket) {
                    loop {
                        if let TokenType::Ident(name) = &p.peek().typ {
                            names.push(name.clone());
                            p.advance();
                        } else {
                            // Not all identifiers, not tuple destructuring
                            return Err(ParserError::ExpectedVariableName(p.peek().line));
                        }

                        if !p.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }

                p.consume(TokenType::RBracket, "Expected ']' in tuple destructuring")?;
                p.consume(TokenType::Assign, "Expected '=' after tuple pattern")?;

                // Skip newlines after =
                p.skip_newlines();

                let value = p.expression()?;

                Ok((names, value))
            }) {
                return Ok(Stmt::TupleAssignment { names, value });
            }
        }

        // Check for implicit variable declaration, reassignment, or function definition
        // name = expr (declaration)
        // name := expr (reassignment)
        // name(params) => body (function definition)
        if let TokenType::Ident(name) = &self.peek().typ {
            let name = name.clone();

            // Check for function definition: name(params) =>
            if let Some((params, body)) = self.try_parse(|p| {
                p.advance(); // consume identifier
                p.consume(TokenType::LParen, "Expected '('")?;

                let params = p.function_params()?;
                p.consume(TokenType::RParen, "Expected ')' after function parameters")?;
                p.consume(TokenType::Arrow, "Expected '=>'")?;

                // Skip optional newline after =>
                p.match_token(&[TokenType::Newline]);

                // Parse function body (can be a block or single expression)
                let body = p.parse_block()?;

                Ok((params, body))
            }) {
                let initializer = Some(Expr::Function { params, body });
                return Ok(Stmt::VarDecl {
                    name,
                    type_annotation: None,
                    initializer,
                    is_varip: false,
                });
            }

            // Try to parse as assignment/declaration
            if let Some(stmt) = self.try_parse(|p| {
                p.advance(); // consume identifier

                if p.match_token(&[TokenType::Assign]) {
                    // This is an assignment with =, treat it as a var declaration
                    let initializer = Some(p.parse_indented_expression()?);

                    return Ok(Stmt::VarDecl {
                        name: name.clone(),
                        type_annotation: None,
                        initializer,
                        is_varip: false,
                    });
                } else if p.match_token(&[TokenType::ColonAssign]) {
                    // This is a reassignment with :=
                    let value = p.parse_indented_expression()?;

                    return Ok(Stmt::Assignment {
                        target: Expr::Variable(name.clone()),
                        value,
                    });
                } else if p.match_token(&[
                    TokenType::PlusAssign,
                    TokenType::MinusAssign,
                    TokenType::StarAssign,
                    TokenType::SlashAssign,
                ]) {
                    // Compound assignment: x += 5 is equivalent to x := x + 5
                    let op = p.tokens[p.current - 1]
                        .typ
                        .to_binop()
                        .expect("compound assign token should convert to binop");

                    let right = p.parse_indented_expression()?;

                    let value = Expr::Binary {
                        left: Box::new(Expr::Variable(name.clone())),
                        op,
                        right: Box::new(right),
                    };
                    return Ok(Stmt::Assignment {
                        target: Expr::Variable(name.clone()),
                        value,
                    });
                } else {
                    // Not an assignment operator, fail
                    return Err(ParserError::UnexpectedToken(
                        p.peek().typ.clone(),
                        p.peek().line,
                    ));
                }
            }) {
                return Ok(stmt);
            }
        }

        self.expression_statement()
    }

    fn function_params(&mut self) -> Result<Vec<String>, ParserError> {
        let mut params = vec![];

        if !self.check(&TokenType::RParen) {
            loop {
                if let TokenType::Ident(name) = &self.peek().typ {
                    params.push(name.clone());
                    self.advance();

                    // Check for default value: param = value
                    if self.match_token(&[TokenType::Assign]) {
                        // Skip the default value expression (we're not storing it in our simple AST)
                        // Just parse and discard it
                        self.expression()?;
                    }
                } else {
                    return Err(ParserError::ExpectedParameterName(self.peek().line));
                }

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }

                // Skip newlines after comma in parameter list
                self.skip_newlines();
            }
        }

        Ok(params)
    }

    fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        // Check if it's a tuple form: for [index, item] in collection
        if self.check(&TokenType::LBracket) {
            self.advance(); // consume [

            let index_var = if let TokenType::Ident(name) = &self.peek().typ {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParserError::ExpectedVariableName(self.peek().line));
            };

            self.consume(TokenType::Comma, "Expected ',' in for...in tuple")?;

            let item_var = if let TokenType::Ident(name) = &self.peek().typ {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParserError::ExpectedVariableName(self.peek().line));
            };

            self.consume(TokenType::RBracket, "Expected ']' after for...in tuple")?;
            self.consume(TokenType::In, "Expected 'in' in for...in loop")?;

            let collection = self.expression()?;

            // Skip optional newline
            self.match_token(&[TokenType::Newline]);

            let body = self.parse_block()?;

            return Ok(Stmt::ForIn {
                index_var: Some(index_var),
                item_var,
                collection,
                body,
            });
        }

        // Parse variable name
        let var_name = if let TokenType::Ident(name) = &self.peek().typ {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParserError::ExpectedVariableName(self.peek().line));
        };

        // Check if it's for...in (simple form) or for...to
        if self.check(&TokenType::In) {
            self.advance(); // consume 'in'

            let collection = self.expression()?;

            // Skip optional newline
            self.match_token(&[TokenType::Newline]);

            let body = self.parse_block()?;

            Ok(Stmt::ForIn {
                index_var: None,
                item_var: var_name,
                collection,
                body,
            })
        } else {
            // Traditional for...to loop
            self.consume(TokenType::Assign, "Expected '=' in for loop")?;
            let from = self.expression()?;
            self.consume(TokenType::To, "Expected 'to' in for loop")?;
            let to = self.expression()?;

            // Skip optional newline after to
            self.match_token(&[TokenType::Newline]);

            // Parse the body - multiple statements
            let body = self.parse_block()?;

            Ok(Stmt::For {
                var_name,
                from,
                to,
                body,
            })
        }
    }

    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        // Parse: while condition
        let condition = self.expression()?;

        // Skip optional newline after condition
        self.match_token(&[TokenType::Newline]);

        // Parse the body - multiple statements
        let body = self.parse_block()?;

        Ok(Stmt::While { condition, body })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        // Parse the condition (no parentheses required in PineScript)
        let condition = self.expression()?;

        // Skip optional newline after condition
        self.match_token(&[TokenType::Newline]);

        // Parse the then branch - multiple statements until we hit 'else', dedent, or certain keywords
        let then_branch = self.parse_block()?;

        // Parse else if branches
        let mut else_if_branches = Vec::new();

        loop {
            // Skip any newlines before else
            self.skip_newlines();

            // Check if we have "else if"
            if self.check(&TokenType::Else) {
                let saved_pos = self.current;
                self.advance(); // consume 'else'

                // Check if next token is 'if'
                if self.match_token(&[TokenType::If]) {
                    // This is an else if
                    let else_if_condition = self.expression()?;
                    self.match_token(&[TokenType::Newline]);
                    let else_if_body = self.parse_block()?;
                    else_if_branches.push((else_if_condition, else_if_body));
                } else {
                    // This is just 'else', restore position to before 'else'
                    self.current = saved_pos;
                    break;
                }
            } else {
                break;
            }
        }

        // Check for final else branch
        self.skip_newlines();

        let else_branch = if self.match_token(&[TokenType::Else]) {
            // Skip optional newline after else
            self.match_token(&[TokenType::Newline]);

            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        })
    }

    fn if_expression(&mut self) -> Result<Expr, ParserError> {
        // Consume 'if' token
        self.consume(TokenType::If, "Expected 'if'")?;

        // Parse the condition
        let condition = self.expression()?;

        // Skip optional newline after condition
        self.match_token(&[TokenType::Newline]);

        // Skip optional indent
        self.match_token(&[TokenType::Indent]);

        // Parse the then expression (single expression, not a block of statements)
        let then_expr = self.expression()?;

        // Skip newlines and dedent
        self.skip_newlines();
        self.match_token(&[TokenType::Dedent]);

        // Parse else if branches
        let mut else_if_branches = Vec::new();

        loop {
            // Skip any newlines before else
            self.skip_newlines();

            // Check if we have "else if"
            if self.check(&TokenType::Else) {
                let saved_pos = self.current;
                self.advance(); // consume 'else'

                // Check if next token is 'if'
                if self.match_token(&[TokenType::If]) {
                    // This is an else if
                    let else_if_condition = self.expression()?;
                    self.match_token(&[TokenType::Newline]);
                    self.match_token(&[TokenType::Indent]);
                    let else_if_expr = self.expression()?;
                    self.skip_newlines();
                    self.match_token(&[TokenType::Dedent]);
                    else_if_branches.push((else_if_condition, else_if_expr));
                } else {
                    // This is just 'else', restore position to before 'else'
                    self.current = saved_pos;
                    break;
                }
            } else {
                break;
            }
        }

        // Parse final else branch (optional - if not present, returns na)
        self.skip_newlines();
        let else_expr = if self.match_token(&[TokenType::Else]) {
            // Skip optional newline after else
            self.match_token(&[TokenType::Newline]);

            // Skip optional indent
            self.match_token(&[TokenType::Indent]);

            // Parse else expression
            let expr = self.expression()?;

            // Skip newlines and optional dedent
            self.skip_newlines();
            self.match_token(&[TokenType::Dedent]);

            Some(Box::new(expr))
        } else {
            None // Will return na if no branch matches
        };

        Ok(Expr::IfExpr {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_if_branches,
            else_expr,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut stmts = vec![];

        // Expect an indent token to start the block
        if !self.match_token(&[TokenType::Indent]) {
            // No indent means single-line block or empty block
            // Try to parse a single statement on the same line
            if !self.check(&TokenType::Newline)
                && !self.check(&TokenType::Else)
                && !self.is_at_end()
            {
                stmts.push(self.declaration()?);
            }
            return Ok(stmts);
        }

        // Parse statements until we hit a dedent
        loop {
            // Skip leading newlines
            self.skip_newlines();

            // Check for else (which ends the then branch)
            if self.check(&TokenType::Else) {
                break;
            }

            // Check for end of block
            if self.check(&TokenType::Dedent) {
                self.advance(); // consume the dedent

                // Check if else follows the dedent
                self.skip_newlines();
                if self.check(&TokenType::Else) {
                    break;
                }

                // If not else, we're truly done
                break;
            }

            // Stop at EOF
            if self.is_at_end() {
                break;
            }

            // Parse a statement
            stmts.push(self.declaration()?);
        }

        Ok(stmts)
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;

        // Check if this is an assignment statement (e.g., obj.field := value)
        if self.match_token(&[TokenType::ColonAssign]) {
            let value = self.parse_indented_expression()?;
            return Ok(Stmt::Assignment {
                target: expr,
                value,
            });
        }

        Ok(Stmt::Expression(expr))
    }

    // Expression parsing with precedence
    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.ternary()
    }

    /// Generic binary operator parser using left-associativity
    fn binary_left_assoc(
        &mut self,
        operators: &[TokenType],
        next_precedence: fn(&mut Self) -> Result<Expr, ParserError>,
    ) -> Result<Expr, ParserError> {
        let mut expr = next_precedence(self)?;

        loop {
            // Skip newlines before operators (for leading operators on continuation lines)
            self.skip_newlines();

            if !self.match_token(operators) {
                break;
            }

            let op = self.tokens[self.current - 1]
                .typ
                .to_binop()
                .expect("matched operator token should convert to binop");

            // Skip newlines after binary operators (for multi-line expressions)
            self.skip_newlines();
            // Skip indent after newline (for indented continuation lines)
            self.match_token(&[TokenType::Indent]);

            let right = next_precedence(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Expr, ParserError> {
        // Check for if expression first
        if self.check(&TokenType::If) {
            return self.if_expression();
        }

        let mut expr = self.logical_or()?;

        // Skip newlines before '?' for multi-line ternaries
        self.skip_newlines();

        // Skip indent if followed by '?' (for multiline ternaries)
        if self.check(&TokenType::Indent) {
            let saved_pos = self.current;
            self.advance(); // consume indent
            if !self.check(&TokenType::Question) {
                // Not followed by '?', restore position
                self.current = saved_pos;
            }
        }

        if self.match_token(&[TokenType::Question]) {
            // Skip newlines after '?'
            self.skip_newlines();

            // Skip indent after '?'
            self.match_token(&[TokenType::Indent]);

            let then_expr = self.expression()?;

            // Skip newlines before ':'
            self.skip_newlines();

            // Skip indent if followed by ':' (for multiline ternaries)
            if self.check(&TokenType::Indent) {
                let saved_pos = self.current;
                self.advance(); // consume indent
                if !self.check(&TokenType::Colon) {
                    // Not followed by ':', restore position
                    self.current = saved_pos;
                }
            }

            self.consume(TokenType::Colon, "Expected ':' in ternary expression")?;

            // Skip newlines after ':'
            self.skip_newlines();

            // Skip indent after ':'
            self.match_token(&[TokenType::Indent]);

            let else_expr = self.expression()?;
            expr = Expr::Ternary {
                condition: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            };
        }

        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expr, ParserError> {
        self.binary_left_assoc(&[TokenType::Or], Self::logical_and)
    }

    fn logical_and(&mut self) -> Result<Expr, ParserError> {
        self.binary_left_assoc(&[TokenType::And], Self::equality)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        self.binary_left_assoc(&[TokenType::Equal, TokenType::NotEqual], Self::comparison)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        self.binary_left_assoc(
            &[
                TokenType::Greater,
                TokenType::Less,
                TokenType::GreaterEqual,
                TokenType::LessEqual,
            ],
            Self::term,
        )
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        loop {
            // Skip newlines before operators (for leading operators on continuation lines)
            self.skip_newlines();

            // Skip indent/dedent if followed by an operator (for leading operators on continuation lines)
            if self.check(&TokenType::Indent) || self.check(&TokenType::Dedent) {
                let saved_pos = self.current;
                self.advance(); // consume indent or dedent
                if !self.check(&TokenType::Plus) && !self.check(&TokenType::Minus) {
                    // Not followed by operator, restore position
                    self.current = saved_pos;
                }
            }

            if !self.match_token(&[TokenType::Plus, TokenType::Minus]) {
                break;
            }

            let op = self.tokens[self.current - 1]
                .typ
                .to_binop()
                .expect("term token should convert to binop");
            // Skip newlines after binary operators (for multi-line expressions)
            self.skip_newlines();
            // Skip indent after newline (for indented continuation lines)
            self.match_token(&[TokenType::Indent]);
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        self.binary_left_assoc(
            &[TokenType::Star, TokenType::Slash, TokenType::Percent],
            Self::unary,
        )
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&[TokenType::Minus]) {
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                op: UnOp::Neg,
                expr: Box::new(expr),
            });
        }

        if self.match_token(&[TokenType::Not]) {
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(expr),
            });
        }

        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::Dot]) {
                // Member access: expr.member
                // Allow keywords as member names (e.g., input.int, color.new)
                let member = match &self.peek().typ {
                    TokenType::Ident(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    TokenType::Int => {
                        self.advance();
                        "int".to_string()
                    }
                    TokenType::Float => {
                        self.advance();
                        "float".to_string()
                    }
                    _ => {
                        // Try to use the lexeme if it's a keyword
                        let lexeme = self.peek().lexeme.clone();
                        if !lexeme.is_empty() {
                            self.advance();
                            lexeme
                        } else {
                            return Err(ParserError::ExpectedIdentifierAfterDot(self.peek().line));
                        }
                    }
                };
                expr = Expr::MemberAccess {
                    object: Box::new(expr),
                    member,
                };
            } else if self.match_token(&[TokenType::LBracket]) {
                // Historical reference: expr[index]
                let index = self.expression()?;
                self.consume(TokenType::RBracket, "Expected ']'")?;
                expr = Expr::Index {
                    expr: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&[TokenType::LParen]) {
                // Function call - callee can be Variable, MemberAccess, or other expressions
                let args = self.arguments()?;
                self.consume(TokenType::RParen, "Expected ')'")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn arguments(&mut self) -> Result<Vec<Argument>, ParserError> {
        let mut args = vec![];

        // Skip leading newlines in argument list
        self.skip_newlines();
        // Skip indent after newline (for indented argument lists)
        self.match_token(&[TokenType::Indent]);

        if !self.check(&TokenType::RParen) {
            loop {
                // Check for named argument: name=value
                // In PineScript, function calls can have named arguments like plot(x, title="foo", color=red)
                if let TokenType::Ident(name) = &self.peek().typ {
                    let name = name.clone();
                    let saved_pos = self.current;
                    self.advance(); // consume identifier

                    if self.check(&TokenType::Assign) {
                        // This is a named argument
                        self.advance(); // consume =
                        let value = self.expression()?;
                        args.push(Argument::Named { name, value });
                    } else {
                        // Not a named argument, backtrack and parse as expression
                        self.current = saved_pos;
                        let expr = self.expression()?;
                        args.push(Argument::Positional(expr));
                    }
                } else {
                    let expr = self.expression()?;
                    args.push(Argument::Positional(expr));
                }

                // Skip newlines after each argument
                self.skip_newlines();

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }

                // Skip newlines after comma
                self.skip_newlines();
                // Skip indent after newline (for indented argument lists)
                self.match_token(&[TokenType::Indent]);
            }
        }

        // Skip trailing newlines and dedent before closing paren
        self.skip_newlines();
        self.match_token(&[TokenType::Dedent]);

        Ok(args)
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        if let TokenType::Number(n) = self.peek().typ {
            self.advance();
            return Ok(Expr::Literal(Literal::Number(n)));
        }

        if let TokenType::String(ref s) = self.peek().typ {
            let s = s.clone();
            self.advance();
            return Ok(Expr::Literal(Literal::String(s)));
        }

        if let TokenType::Bool(b) = self.peek().typ {
            self.advance();
            return Ok(Expr::Literal(Literal::Bool(b)));
        }

        if let TokenType::HexColor(ref hex) = self.peek().typ {
            let hex = hex.clone();
            self.advance();
            return Ok(Expr::Literal(Literal::HexColor(hex)));
        }

        // Handle na as a literal
        if self.match_token(&[TokenType::Na]) {
            return Ok(Expr::Literal(Literal::Na));
        }

        // Handle keywords that can be used as identifiers (int, float)
        // These can be function names (e.g., int(), float())
        if self.match_token(&[TokenType::Int, TokenType::Float]) {
            let name = self.tokens[self.current - 1].lexeme.clone();
            return Ok(Expr::Variable(name));
        }

        if let TokenType::Ident(ref name) = self.peek().typ {
            let name = name.clone();
            self.advance();
            return Ok(Expr::Variable(name));
        }

        if self.match_token(&[TokenType::LParen]) {
            // Skip newlines and indents after opening parenthesis for multiline expressions
            self.skip_newlines();
            let had_indent = self.match_token(&[TokenType::Indent]);

            let expr = self.expression()?;

            // Skip newlines and consume dedent if we had indent
            self.skip_newlines();
            if had_indent {
                self.match_token(&[TokenType::Dedent]);
            }

            self.consume(TokenType::RParen, "Expected ')'")?;
            return Ok(expr);
        }

        // Switch expression: switch value \n case => result
        if self.match_token(&[TokenType::Switch]) {
            let value = Box::new(self.expression()?);

            // Skip newline after switch value
            self.match_token(&[TokenType::Newline]);

            // Skip indent for switch block
            let has_indent = self.match_token(&[TokenType::Indent]);

            let mut cases = vec![];

            // Parse cases until we can't parse any more
            loop {
                // Skip leading newlines
                self.skip_newlines();

                // Check for dedent (end of switch block)
                if self.check(&TokenType::Dedent) {
                    if has_indent {
                        self.advance(); // consume dedent
                    }
                    break;
                }

                // Check if we're done (end of block or EOF)
                if self.is_at_end() {
                    break;
                }

                // Check for default case: => result (no pattern)
                if self.match_token(&[TokenType::Arrow]) {
                    // Skip newlines after =>
                    self.skip_newlines();

                    // Parse the result expression
                    let result = self.expression()?;

                    // Use a special "default" literal as the pattern
                    let default_pattern = Expr::Literal(Literal::Bool(true));
                    cases.push((default_pattern, result));
                    continue;
                }

                // Try to parse a case
                let saved_pos = self.current;

                // Parse the pattern (could be a string, number, identifier, etc.)
                match self.expression() {
                    Ok(pattern) => {
                        // Expect =>
                        if !self.match_token(&[TokenType::Arrow]) {
                            // Not a case, backtrack
                            self.current = saved_pos;
                            break;
                        }

                        // Skip newlines after =>
                        self.skip_newlines();

                        // Parse the result expression
                        let result = self.expression()?;
                        cases.push((pattern, result));
                    }
                    Err(_) => {
                        // Failed to parse pattern, we're done
                        self.current = saved_pos;
                        break;
                    }
                }
            }

            return Ok(Expr::Switch { value, cases });
        }

        // Array literal: [1, 2, 3]
        if self.match_token(&[TokenType::LBracket]) {
            let mut elements = vec![];

            // Skip leading newlines
            self.skip_newlines();
            // Skip indent after newline (for indented array elements)
            self.match_token(&[TokenType::Indent]);

            if !self.check(&TokenType::RBracket) {
                loop {
                    elements.push(self.expression()?);

                    // Skip newlines after each element
                    self.skip_newlines();

                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }

                    // Skip newlines after comma
                    self.skip_newlines();
                    // Skip indent after newline (for indented array elements)
                    self.match_token(&[TokenType::Indent]);
                }
            }

            // Skip trailing newlines and dedent
            self.skip_newlines();
            self.match_token(&[TokenType::Dedent]);

            self.consume(TokenType::RBracket, "Expected ']'")?;
            return Ok(Expr::Array(elements));
        }

        Err(ParserError::UnexpectedToken(
            self.peek().typ.clone(),
            self.peek().line,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_lexer::Lexer;

    fn parse_expr(input: &str) -> eyre::Result<Expr> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse()?;

        if let Some(Stmt::Expression(expr)) = stmts.first() {
            Ok(expr.clone())
        } else {
            Err(eyre::eyre!("Expected expression statement".to_string()))
        }
    }

    #[test]
    fn test_literals() {
        // Numbers
        let expr = parse_expr("42").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));

        // Strings
        let expr = parse_expr(r#""hello""#).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::String("hello".to_string())));

        // Booleans
        let expr = parse_expr("true").unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Bool(true)));
    }

    #[test]
    fn test_variables() {
        let expr = parse_expr("close").unwrap();
        assert_eq!(expr, Expr::Variable("close".to_string()));

        let expr = parse_expr("my_var").unwrap();
        assert_eq!(expr, Expr::Variable("my_var".to_string()));
    }

    #[test]
    fn test_historical_references() {
        // close[1] - previous close
        let expr = parse_expr("close[1]").unwrap();
        assert!(matches!(expr, Expr::Index { .. }));
        if let Expr::Index { expr: base, index } = expr {
            assert_eq!(*base, Expr::Variable("close".to_string()));
            assert_eq!(*index, Expr::Literal(Literal::Number(1.0)));
        }

        // high[5] - 5 bars ago
        let expr = parse_expr("high[5]").unwrap();
        if let Expr::Index { expr: base, index } = expr {
            assert_eq!(*base, Expr::Variable("high".to_string()));
            assert_eq!(*index, Expr::Literal(Literal::Number(5.0)));
        }
    }

    #[test]
    fn test_function_calls() {
        // Simple function call
        let expr = parse_expr("sma(close, 14)").unwrap();
        if let Expr::Call { callee, args } = expr {
            assert_eq!(*callee, Expr::Variable("sma".to_string()));
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Argument::Positional(Expr::Variable("close".to_string())));
            assert_eq!(args[1], Argument::Positional(Expr::Literal(Literal::Number(14.0))));
        } else {
            panic!("Expected function call");
        }

        // No arguments
        let expr = parse_expr("foo()").unwrap();
        if let Expr::Call { callee, args } = expr {
            assert_eq!(*callee, Expr::Variable("foo".to_string()));
            assert_eq!(args.len(), 0);
        }
    }

    #[test]
    fn test_arithmetic_expressions() {
        // Addition
        let expr = parse_expr("2 + 3").unwrap();
        if let Expr::Binary { left, op, right } = expr {
            assert_eq!(*left, Expr::Literal(Literal::Number(2.0)));
            assert_eq!(op, BinOp::Add);
            assert_eq!(*right, Expr::Literal(Literal::Number(3.0)));
        }

        // Multiplication has higher precedence: 2 + 3 * 4 = 2 + (3 * 4)
        let expr = parse_expr("2 + 3 * 4").unwrap();
        if let Expr::Binary {
            left,
            op: op1,
            right,
        } = expr
        {
            assert_eq!(*left, Expr::Literal(Literal::Number(2.0)));
            assert_eq!(op1, BinOp::Add);
            if let Expr::Binary {
                left: l2,
                op: op2,
                right: r2,
            } = *right
            {
                assert_eq!(*l2, Expr::Literal(Literal::Number(3.0)));
                assert_eq!(op2, BinOp::Mul);
                assert_eq!(*r2, Expr::Literal(Literal::Number(4.0)));
            }
        }

        // Division
        let expr = parse_expr("10 / 2").unwrap();
        if let Expr::Binary { left, op, right } = expr {
            assert_eq!(*left, Expr::Literal(Literal::Number(10.0)));
            assert_eq!(op, BinOp::Div);
            assert_eq!(*right, Expr::Literal(Literal::Number(2.0)));
        }

        // Subtraction
        let expr = parse_expr("5 - 3").unwrap();
        if let Expr::Binary { left, op, right } = expr {
            assert_eq!(*left, Expr::Literal(Literal::Number(5.0)));
            assert_eq!(op, BinOp::Sub);
            assert_eq!(*right, Expr::Literal(Literal::Number(3.0)));
        }
    }

    #[test]
    fn test_comparison_expressions() {
        // Greater than
        let expr = parse_expr("close > open").unwrap();
        if let Expr::Binary { left, op, right } = expr {
            assert_eq!(*left, Expr::Variable("close".to_string()));
            assert_eq!(op, BinOp::Greater);
            assert_eq!(*right, Expr::Variable("open".to_string()));
        }

        // Less than
        let expr = parse_expr("rsi < 30").unwrap();
        if let Expr::Binary { left, op, right } = expr {
            assert_eq!(*left, Expr::Variable("rsi".to_string()));
            assert_eq!(op, BinOp::Less);
            assert_eq!(*right, Expr::Literal(Literal::Number(30.0)));
        }

        // Equality
        let expr = parse_expr("x == 5").unwrap();
        if let Expr::Binary { left, op, right } = expr {
            assert_eq!(*left, Expr::Variable("x".to_string()));
            assert_eq!(op, BinOp::Eq);
            assert_eq!(*right, Expr::Literal(Literal::Number(5.0)));
        }
    }

    #[test]
    fn test_unary_expressions() {
        // Negation
        let expr = parse_expr("-5").unwrap();
        if let Expr::Unary { op, expr } = expr {
            assert_eq!(op, UnOp::Neg);
            assert_eq!(*expr, Expr::Literal(Literal::Number(5.0)));
        }

        // Double negation
        let expr = parse_expr("--10").unwrap();
        if let Expr::Unary { op: op1, expr: e1 } = expr {
            assert_eq!(op1, UnOp::Neg);
            if let Expr::Unary { op: op2, expr: e2 } = *e1 {
                assert_eq!(op2, UnOp::Neg);
                assert_eq!(*e2, Expr::Literal(Literal::Number(10.0)));
            }
        }
    }

    #[test]
    fn test_var_declarations() {
        let mut lexer = Lexer::new("var x = 10");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        assert_eq!(stmts.len(), 1);
        if let Stmt::VarDecl {
            name,
            type_annotation,
            initializer,
            is_varip,
        } = &stmts[0]
        {
            assert_eq!(name, "x");
            assert_eq!(*type_annotation, None);
            assert_eq!(
                initializer.as_ref().unwrap(),
                &Expr::Literal(Literal::Number(10.0))
            );
            assert_eq!(*is_varip, false);
        } else {
            panic!("Expected VarDecl");
        }

        // Var without initializer
        let mut lexer = Lexer::new("var y");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        if let Stmt::VarDecl {
            name, initializer, ..
        } = &stmts[0]
        {
            assert_eq!(name, "y");
            assert!(initializer.is_none());
        }
    }

    #[test]
    fn test_pinescript_examples() {
        // PineScript: close[1] > close[2]
        let expr = parse_expr("close[1] > close[2]").unwrap();
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinOp::Greater,
                ..
            }
        ));

        // PineScript: sma(close, 14) > sma(close, 28)
        let expr = parse_expr("sma(close, 14) > sma(close, 28)").unwrap();
        if let Expr::Binary { left, op, right } = expr {
            assert_eq!(op, BinOp::Greater);
            assert!(matches!(*left, Expr::Call { .. }));
            assert!(matches!(*right, Expr::Call { .. }));
        }

        // PineScript: (high + low) / 2
        let expr = parse_expr("(high + low) / 2").unwrap();
        if let Expr::Binary {
            left,
            op: div_op,
            right,
        } = expr
        {
            assert_eq!(div_op, BinOp::Div);
            assert!(matches!(*left, Expr::Binary { op: BinOp::Add, .. }));
            assert_eq!(*right, Expr::Literal(Literal::Number(2.0)));
        }
    }

    /// Helper function to recursively collect all .pine files in a directory
    fn collect_pine_files_recursive(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pine"))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    #[test]
    fn test_parse_testdata_files() -> eyre::Result<()> {
        use std::fs;
        use std::path::Path;

        let testdata_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");

        let filter = std::env::var("TEST_FILE").ok();
        let debug = std::env::var("DEBUG").is_ok();
        let generate_ast = std::env::var("GENERATE_AST").is_ok();

        let pine_files = collect_pine_files_recursive(&testdata_dir);

        let process_file = |path: &std::path::PathBuf| -> eyre::Result<()> {
            let content = fs::read_to_string(&path)?;

            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize()?;

            if debug {
                println!("Tokens: {:#?}", tokens);
            }

            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;

            if debug {
                let ast_json = serde_json::to_string(&ast)?;
                println!("AST JSON: {:?}", ast_json);
            }

            // Check for corresponding _ast.json file
            let json_path = path.with_file_name(format!(
                "{}_ast.json",
                path.file_stem().unwrap().to_str().unwrap()
            ));

            if generate_ast {
                // Generate/overwrite AST JSON file
                let json = serde_json::to_string_pretty(&ast)?;
                fs::write(&json_path, &json)?;
            } else if json_path.exists() {
                // Compare with expected AST
                let expected_json = fs::read_to_string(&json_path)?;
                let expected_ast: Vec<Stmt> = serde_json::from_str(&expected_json)?;

                if ast != expected_ast {
                    return Err(eyre::eyre!(
                        "AST mismatch, expected AST from {:?}",
                        json_path
                    ));
                }
            }

            Ok(())
        };

        for path in pine_files {
            let filename = path.file_name().unwrap().to_str().unwrap();

            // Skip if filter is set and doesn't match
            if let Some(ref filter_name) = filter {
                if filename != filter_name {
                    continue;
                }
            }

            if let Err(e) = process_file(&path) {
                return Err(eyre::eyre!("Failed to process {}: {}", filename, e));
            }
        }

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_parse_external_pinescript_indicators() -> eyre::Result<()> {
        use std::fs;
        use std::path::Path;

        let testdata_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tradingview-pinescript-indicators");

        let filter = std::env::var("TEST_FILE").ok();
        let debug = std::env::var("DEBUG").is_ok();

        let pine_files = collect_pine_files_recursive(&testdata_dir);

        let process_file = |path: &std::path::PathBuf| -> eyre::Result<()> {
            let content = fs::read_to_string(&path)?;

            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize()?;

            if debug {
                println!("Tokens: {:#?}", tokens);
            }

            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;

            let ast_json = serde_json::to_string(&ast)?;

            if debug {
                println!("AST JSON: {:?}", ast_json);
            }

            Ok(())
        };

        for path in pine_files {
            let filename = path.file_name().unwrap().to_str().unwrap();

            // Skip if filter is set and doesn't match
            if let Some(ref filter_name) = filter {
                if filename != filter_name {
                    continue;
                }
            }

            if let Err(e) = process_file(&path) {
                return Err(eyre::eyre!("Failed to process {}: {}", filename, e));
            }
        }

        Ok(())
    }
}
