//! # PVDSL Parser
//!
//! Recursive descent parser that builds an AST from tokens.

use super::ast::*;
use super::error::PvdslError;
use super::lexer::{Token, TokenType};

/// PVDSL Parser
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Create a new parser from a list of tokens
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parse the tokens into a program AST
    ///
    /// # Errors
    ///
    /// Returns a parse error if the tokens cannot be parsed into a valid program.
    pub fn parse(&mut self) -> Result<Program, PvdslError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
        }

        let has_functions = statements
            .iter()
            .any(|s| matches!(s, Statement::FunctionDef { .. }));

        Ok(Program {
            statements,
            metadata: ProgramMetadata { has_functions },
        })
    }

    fn parse_statement(&mut self) -> Result<Option<Statement>, PvdslError> {
        self.skip_whitespace();
        if self.is_at_end() {
            return Ok(None);
        }

        match self.peek_type() {
            TokenType::Function => self.parse_function_def().map(Some),
            TokenType::Return => self.parse_return_statement().map(Some),
            TokenType::If => self.parse_if_statement().map(Some),
            TokenType::While => self.parse_while_statement().map(Some),
            TokenType::For => self.parse_for_statement().map(Some),
            TokenType::Identifier => {
                if self.peek_next_type() == Some(TokenType::Assign) {
                    self.parse_variable_declaration().map(Some)
                } else {
                    self.parse_expression_statement().map(Some)
                }
            }
            _ => {
                self.advance();
                Ok(None)
            }
        }
    }

    fn parse_function_def(&mut self) -> Result<Statement, PvdslError> {
        self.consume(TokenType::Function, "Expect 'function' or 'fn'")?;
        let name_token = self.consume(TokenType::Identifier, "Expect function name")?;
        self.consume(TokenType::LParen, "Expect '(' after function name")?;

        let mut params = Vec::new();
        if self.peek_type() != TokenType::RParen {
            loop {
                let param = self.consume(TokenType::Identifier, "Expect parameter name")?;
                params.push(param.value);
                if self.peek_type() == TokenType::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.consume(TokenType::RParen, "Expect ')' after parameters")?;

        let body = self.parse_block()?;

        Ok(Statement::FunctionDef {
            identifier: name_token.value,
            params,
            body,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, PvdslError> {
        self.consume(TokenType::LBrace, "Expect '{' before block")?;
        let mut statements = Vec::new();
        while self.peek_type() != TokenType::RBrace && !self.is_at_end() {
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
        }
        self.consume(TokenType::RBrace, "Expect '}' after block")?;
        Ok(statements)
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, PvdslError> {
        let name_token = self.consume(TokenType::Identifier, "Expect variable name")?;
        self.consume(TokenType::Assign, "Expect '=' after variable name")?;
        let value = self.parse_expression()?;
        Ok(Statement::VariableDeclaration {
            identifier: name_token.value,
            value,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, PvdslError> {
        self.advance(); // consume 'return'
        let value = if self.peek_type() != TokenType::Newline
            && self.peek_type() != TokenType::Semicolon
            && self.peek_type() != TokenType::RBrace
            && self.peek_type() != TokenType::Eof
        {
            Some(self.parse_expression()?)
        } else {
            None
        };
        Ok(Statement::ReturnStatement { value })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, PvdslError> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        let consequent = self.parse_block()?;
        let mut alternate = None;

        self.skip_whitespace();
        if self.peek_type() == TokenType::Else {
            self.advance();
            self.skip_whitespace();
            if self.peek_type() == TokenType::If {
                alternate = Some(vec![self.parse_if_statement()?]);
            } else {
                alternate = Some(self.parse_block()?);
            }
        }

        Ok(Statement::IfStatement {
            condition,
            consequent,
            alternate,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, PvdslError> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::WhileStatement { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, PvdslError> {
        self.advance(); // consume 'for'
        let iterator = self
            .consume(TokenType::Identifier, "Expect iterator name")?
            .value;
        self.consume(TokenType::In, "Expect 'in' after iterator")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::ForStatement {
            iterator,
            iterable,
            body,
        })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, PvdslError> {
        let expression = self.parse_expression()?;
        Ok(Statement::ExpressionStatement { expression })
    }

    fn parse_expression(&mut self) -> Result<Expression, PvdslError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, PvdslError> {
        let mut expr = self.parse_and()?;

        while self.peek_type() == TokenType::Or {
            let operator = self.advance().value;
            let right = self.parse_and()?;
            expr = Expression::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expression, PvdslError> {
        let mut expr = self.parse_comparison()?;

        while self.peek_type() == TokenType::And {
            let operator = self.advance().value;
            let right = self.parse_comparison()?;
            expr = Expression::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, PvdslError> {
        let mut expr = self.parse_term()?;

        while matches!(
            self.peek_type(),
            TokenType::Greater
                | TokenType::Less
                | TokenType::GreaterEqual
                | TokenType::LessEqual
                | TokenType::Equals
                | TokenType::NotEquals
        ) {
            let operator = self.advance().value;
            let right = self.parse_term()?;
            expr = Expression::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, PvdslError> {
        let mut expr = self.parse_factor()?;

        while matches!(self.peek_type(), TokenType::Plus | TokenType::Minus) {
            let operator = self.advance().value;
            let right = self.parse_factor()?;
            expr = Expression::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, PvdslError> {
        let mut expr = self.parse_primary()?;

        while matches!(
            self.peek_type(),
            TokenType::Multiply | TokenType::Divide | TokenType::Modulo
        ) {
            let operator = self.advance().value;
            let right = self.parse_primary()?;
            expr = Expression::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, PvdslError> {
        let token = self.advance();
        match token.token_type {
            TokenType::String => Ok(Expression::StringLiteral { value: token.value }),
            TokenType::Integer | TokenType::Float => {
                let val: f64 = token.value.parse().unwrap_or(0.0);
                Ok(Expression::NumberLiteral { value: val })
            }
            TokenType::True => Ok(Expression::BooleanLiteral { value: true }),
            TokenType::False => Ok(Expression::BooleanLiteral { value: false }),
            TokenType::LBracket => self.parse_list_literal(),
            TokenType::Identifier => {
                if self.peek_type() == TokenType::LParen {
                    self.parse_function_call(None, token.value)
                } else if self.peek_type() == TokenType::DoubleColon {
                    self.advance();
                    let func_name = self
                        .consume(TokenType::Identifier, "Expect function name after '::'")?
                        .value;
                    self.parse_function_call(Some(token.value), func_name)
                } else {
                    Ok(Expression::Identifier { name: token.value })
                }
            }
            TokenType::LParen => {
                let expr = self.parse_expression()?;
                self.consume(TokenType::RParen, "Expect ')' after expression")?;
                Ok(expr)
            }
            _ => Err(PvdslError::ParseError {
                line: token.line,
                column: token.column,
                message: format!("Unexpected token: {:?}", token.token_type),
            }),
        }
    }

    fn parse_list_literal(&mut self) -> Result<Expression, PvdslError> {
        let mut elements = Vec::new();
        if self.peek_type() != TokenType::RBracket {
            loop {
                elements.push(self.parse_expression()?);
                if self.peek_type() == TokenType::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.consume(TokenType::RBracket, "Expect ']' after list")?;
        Ok(Expression::ListLiteral { elements })
    }

    fn parse_function_call(
        &mut self,
        namespace: Option<String>,
        identifier: String,
    ) -> Result<Expression, PvdslError> {
        self.consume(TokenType::LParen, "Expect '('")?;
        let mut arguments = Vec::new();
        if self.peek_type() != TokenType::RParen {
            loop {
                arguments.push(self.parse_expression()?);
                if self.peek_type() == TokenType::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.consume(TokenType::RParen, "Expect ')'")?;
        Ok(Expression::FunctionCall {
            namespace,
            identifier,
            arguments,
        })
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, PvdslError> {
        if self.peek_type() == token_type {
            Ok(self.advance())
        } else {
            let token = self.peek();
            Err(PvdslError::ParseError {
                line: token.line,
                column: token.column,
                message: message.to_string(),
            })
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_type(&self) -> TokenType {
        self.tokens[self.current].token_type.clone()
    }

    fn peek_next_type(&self) -> Option<TokenType> {
        self.tokens
            .get(self.current + 1)
            .map(|t| t.token_type.clone())
    }

    fn is_at_end(&self) -> bool {
        self.peek_type() == TokenType::Eof
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end()
            && (self.peek_type() == TokenType::Newline || self.peek_type() == TokenType::Comment)
        {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::lexer::Lexer;
    use super::*;

    #[test]
    fn test_parse_basic() {
        let mut lexer = Lexer::new("x = 10\nfunction test(a) { return a }");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 2);
        assert!(program.metadata.has_functions);
    }

    #[test]
    fn test_parse_namespaced_call() {
        let mut lexer = Lexer::new("result = signal::prr(10, 90, 100, 9800)");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_arithmetic() {
        let mut lexer = Lexer::new("x = 1 + 2 * 3");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_if_else() {
        let mut lexer = Lexer::new("if x > 5 { return true } else { return false }");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }
}
