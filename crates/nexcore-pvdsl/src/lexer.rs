//! # PVDSL Lexer
//!
//! Tokenizes PVDSL source code into a stream of tokens.

use serde::{Deserialize, Serialize};

/// Token types for PVDSL
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    /// Integer literal
    Integer,
    /// Float literal
    Float,
    /// String literal
    String,
    /// Boolean literal
    Boolean,
    /// Identifier
    Identifier,
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `elif` keyword
    Elif,
    /// `while` keyword
    While,
    /// `for` keyword
    For,
    /// `in` keyword
    In,
    /// `function` or `fn` keyword
    Function,
    /// `return` keyword
    Return,
    /// `break` keyword
    Break,
    /// `continue` keyword
    Continue,
    /// `true` keyword
    True,
    /// `false` keyword
    False,
    /// `+` operator
    Plus,
    /// `-` operator
    Minus,
    /// `*` operator
    Multiply,
    /// `/` operator
    Divide,
    /// `%` operator
    Modulo,
    /// `=` operator
    Assign,
    /// `==` operator
    Equals,
    /// `!=` operator
    NotEquals,
    /// `<` operator
    Less,
    /// `>` operator
    Greater,
    /// `<=` operator
    LessEqual,
    /// `>=` operator
    GreaterEqual,
    /// `and` or `&&` operator
    And,
    /// `or` or `||` operator
    Or,
    /// `not` operator
    Not,
    /// `(` delimiter
    LParen,
    /// `)` delimiter
    RParen,
    /// `{` delimiter
    LBrace,
    /// `}` delimiter
    RBrace,
    /// `[` delimiter
    LBracket,
    /// `]` delimiter
    RBracket,
    /// `,` delimiter
    Comma,
    /// `:` delimiter
    Colon,
    /// `;` delimiter
    Semicolon,
    /// `.` delimiter
    Dot,
    /// `::` delimiter
    DoubleColon,
    /// Newline
    Newline,
    /// End of file
    Eof,
    /// Comment
    Comment,
}

/// A token with position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The type of token
    pub token_type: TokenType,
    /// The raw value of the token
    pub value: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

/// PVDSL Lexer
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given source code
    #[must_use]
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire source into a vector of tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.pos < self.source.len() {
            if let Some(token) = self.scan_token() {
                tokens.push(token);
            }
        }
        tokens.push(Token {
            token_type: TokenType::Eof,
            value: String::new(),
            line: self.line,
            column: self.column,
        });
        tokens
    }

    fn scan_token(&mut self) -> Option<Token> {
        let c = self.current();

        if c.is_whitespace() && c != '\n' {
            self.advance();
            return None;
        }

        if c == '\n' {
            let token = Token {
                token_type: TokenType::Newline,
                value: "\n".to_string(),
                line: self.line,
                column: self.column,
            };
            self.advance();
            self.line += 1;
            self.column = 1;
            return Some(token);
        }

        if c == '#' || (c == '/' && self.peek() == '/') {
            return self.scan_comment();
        }

        if c == '"' || c == '\'' {
            return Some(self.scan_string(c));
        }

        if c.is_ascii_digit() || (c == '-' && self.peek().is_ascii_digit()) {
            return Some(self.scan_number());
        }

        if c.is_alphabetic() || c == '_' {
            return Some(self.scan_identifier());
        }

        self.scan_operator()
    }

    fn scan_comment(&mut self) -> Option<Token> {
        let start_col = self.column;
        if self.current() == '/' {
            self.advance();
        }
        self.advance();

        let mut value = String::new();
        while self.pos < self.source.len() && self.current() != '\n' {
            value.push(self.current());
            self.advance();
        }
        Some(Token {
            token_type: TokenType::Comment,
            value: value.trim().to_string(),
            line: self.line,
            column: start_col,
        })
    }

    fn scan_string(&mut self, quote: char) -> Token {
        let start_col = self.column;
        self.advance();
        let mut value = String::new();
        while self.pos < self.source.len() && self.current() != quote {
            if self.current() == '\\' {
                self.advance();
                match self.current() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    c if c == quote => value.push(quote),
                    c => {
                        value.push('\\');
                        value.push(c);
                    }
                }
            } else {
                value.push(self.current());
            }
            self.advance();
        }
        self.advance();
        Token {
            token_type: TokenType::String,
            value,
            line: self.line,
            column: start_col,
        }
    }

    fn scan_number(&mut self) -> Token {
        let start_col = self.column;
        let mut value = String::new();
        if self.current() == '-' {
            value.push('-');
            self.advance();
        }
        while self.pos < self.source.len() && self.current().is_ascii_digit() {
            value.push(self.current());
            self.advance();
        }
        if self.current() == '.' && self.peek().is_ascii_digit() {
            value.push('.');
            self.advance();
            while self.pos < self.source.len() && self.current().is_ascii_digit() {
                value.push(self.current());
                self.advance();
            }
            Token {
                token_type: TokenType::Float,
                value,
                line: self.line,
                column: start_col,
            }
        } else {
            Token {
                token_type: TokenType::Integer,
                value,
                line: self.line,
                column: start_col,
            }
        }
    }

    fn scan_identifier(&mut self) -> Token {
        let start_col = self.column;
        let mut value = String::new();
        while self.pos < self.source.len()
            && (self.current().is_alphanumeric() || self.current() == '_')
        {
            value.push(self.current());
            self.advance();
        }
        let token_type = match value.as_str() {
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "elif" => TokenType::Elif,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "function" | "fn" => TokenType::Function,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "true" | "True" => TokenType::True,
            "false" | "False" => TokenType::False,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => TokenType::Identifier,
        };
        Token {
            token_type,
            value,
            line: self.line,
            column: start_col,
        }
    }

    fn scan_operator(&mut self) -> Option<Token> {
        let start_col = self.column;
        let c = self.current();
        let next = self.peek();

        let (token_type, value, advance_twice) = match (c, next) {
            (':', ':') => (TokenType::DoubleColon, "::".to_string(), true),
            ('=', '=') => (TokenType::Equals, "==".to_string(), true),
            ('!', '=') => (TokenType::NotEquals, "!=".to_string(), true),
            ('<', '=') => (TokenType::LessEqual, "<=".to_string(), true),
            ('>', '=') => (TokenType::GreaterEqual, ">=".to_string(), true),
            ('&', '&') => (TokenType::And, "&&".to_string(), true),
            ('|', '|') => (TokenType::Or, "||".to_string(), true),
            ('+', _) => (TokenType::Plus, "+".to_string(), false),
            ('-', _) => (TokenType::Minus, "-".to_string(), false),
            ('*', _) => (TokenType::Multiply, "*".to_string(), false),
            ('/', _) => (TokenType::Divide, "/".to_string(), false),
            ('%', _) => (TokenType::Modulo, "%".to_string(), false),
            ('=', _) => (TokenType::Assign, "=".to_string(), false),
            ('<', _) => (TokenType::Less, "<".to_string(), false),
            ('>', _) => (TokenType::Greater, ">".to_string(), false),
            ('(', _) => (TokenType::LParen, "(".to_string(), false),
            (')', _) => (TokenType::RParen, ")".to_string(), false),
            ('{', _) => (TokenType::LBrace, "{".to_string(), false),
            ('}', _) => (TokenType::RBrace, "}".to_string(), false),
            ('[', _) => (TokenType::LBracket, "[".to_string(), false),
            (']', _) => (TokenType::RBracket, "]".to_string(), false),
            (',', _) => (TokenType::Comma, ",".to_string(), false),
            (':', _) => (TokenType::Colon, ":".to_string(), false),
            (';', _) => (TokenType::Semicolon, ";".to_string(), false),
            ('.', _) => (TokenType::Dot, ".".to_string(), false),
            _ => {
                self.advance();
                return None;
            }
        };

        if advance_twice {
            self.advance();
            self.advance();
        } else {
            self.advance();
        }

        Some(Token {
            token_type,
            value,
            line: self.line,
            column: start_col,
        })
    }

    fn current(&self) -> char {
        *self.source.get(self.pos).unwrap_or(&'\0')
    }

    fn peek(&self) -> char {
        *self.source.get(self.pos + 1).unwrap_or(&'\0')
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.column += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let mut lexer = Lexer::new("x = 10\nif x > 5 {\n  return true\n}");
        let tokens = lexer.tokenize();
        assert!(tokens.len() > 5);
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].value, "x");
    }

    #[test]
    fn test_tokenize_namespaced_call() {
        let mut lexer = Lexer::new("signal::prr(10, 90, 100, 9800)");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].value, "signal");
        assert_eq!(tokens[1].token_type, TokenType::DoubleColon);
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].value, "prr");
    }

    #[test]
    fn test_tokenize_string_escapes() {
        let mut lexer = Lexer::new(r#""hello\nworld""#);
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].value, "hello\nworld");
    }

    #[test]
    fn test_tokenize_float() {
        let mut lexer = Lexer::new("3.14159");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token_type, TokenType::Float);
        assert_eq!(tokens[0].value, "3.14159");
    }
}
