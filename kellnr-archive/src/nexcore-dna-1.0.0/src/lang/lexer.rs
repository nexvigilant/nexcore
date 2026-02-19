//! Lexer: source text → token stream.
//!
//! Phase 7: Extended with compound assignment (+=, -=, *=, /=, %=),
//! range operator (..), and `in` keyword for range loops.
//!
//! Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary)

use crate::error::{DnaError, Result};

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

/// A single lexical token.
///
/// Tier: T2-P (∂ Boundary + ∃ Existence)
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// Integer literal (may be negative via unary minus in parser).
    Int(i64),
    /// Identifier (variable or function name).
    Ident(String),

    // Arithmetic operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,

    // Comparison operators
    /// `==`
    EqEq,
    /// `!=`
    BangEq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    LtEq,
    /// `>=`
    GtEq,

    // Assignment
    /// `=`
    Assign,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    StarEq,
    /// `/=`
    SlashEq,
    /// `%=`
    PercentEq,

    // Delimiters
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `,`
    Comma,

    // Keywords
    /// `let`
    Let,
    /// `if`
    If,
    /// `else`
    Else,
    /// `while`
    While,
    /// `fn`
    Fn,
    /// `end`
    End,
    /// `do`
    Do,
    /// `and`
    And,
    /// `or`
    Or,
    /// `not`
    Not,
    /// `return`
    Return,
    /// `for`
    For,
    /// `to`
    To,
    /// `in` (for range loops: `for i in 0..10 do`)
    In,
    /// `true` (boolean literal → 1)
    True,
    /// `false` (boolean literal → 0)
    False,
    /// `..` (range operator)
    DotDot,

    // Bitwise operators
    /// `&` (bitwise AND)
    Ampersand,
    /// `|` (bitwise OR)
    Pipe,
    /// `^` (bitwise XOR)
    Caret,
    /// `~` (bitwise NOT)
    Tilde,
    /// `<<` (shift left)
    LtLt,
    /// `>>` (shift right)
    GtGt,

    // Keywords (additional)
    /// `elif`
    Elif,

    // Structure
    /// Newline (statement separator).
    Newline,
    /// End of input.
    Eof,
}

impl core::fmt::Display for Token {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{n}"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::EqEq => write!(f, "=="),
            Token::BangEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::LtEq => write!(f, "<="),
            Token::GtEq => write!(f, ">="),
            Token::Assign => write!(f, "="),
            Token::PlusEq => write!(f, "+="),
            Token::MinusEq => write!(f, "-="),
            Token::StarEq => write!(f, "*="),
            Token::SlashEq => write!(f, "/="),
            Token::PercentEq => write!(f, "%="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::Fn => write!(f, "fn"),
            Token::End => write!(f, "end"),
            Token::Do => write!(f, "do"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Return => write!(f, "return"),
            Token::For => write!(f, "for"),
            Token::To => write!(f, "to"),
            Token::In => write!(f, "in"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::DotDot => write!(f, ".."),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::Caret => write!(f, "^"),
            Token::Tilde => write!(f, "~"),
            Token::LtLt => write!(f, "<<"),
            Token::GtGt => write!(f, ">>"),
            Token::Elif => write!(f, "elif"),
            Token::Newline => write!(f, "\\n"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

/// Tokenizes source text into a stream of `Token`s.
///
/// Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary)
pub struct Lexer<'a> {
    source: &'a [u8],
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer over the given source.
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            pos: 0,
            line: 1,
        }
    }

    /// Current line number (1-based).
    pub fn line(&self) -> usize {
        self.line
    }

    /// Peek at current byte without consuming.
    fn peek(&self) -> Option<u8> {
        self.source.get(self.pos).copied()
    }

    /// Advance one byte.
    fn advance(&mut self) -> Option<u8> {
        let ch = self.source.get(self.pos).copied();
        if ch == Some(b'\n') {
            self.line += 1;
        }
        self.pos += 1;
        ch
    }

    /// Skip whitespace (spaces, tabs) but NOT newlines.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == b' ' || ch == b'\t' || ch == b'\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a comment from `;` to end of line.
    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == b'\n' {
                break;
            }
            self.advance();
        }
    }

    /// Read an integer literal: sequence of digits.
    fn read_int(&mut self) -> Result<i64> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let slice = &self.source[start..self.pos];
        let s = core::str::from_utf8(slice)
            .map_err(|_| DnaError::SyntaxError(self.line, "invalid UTF-8 in integer".into()))?;

        s.parse::<i64>()
            .map_err(|_| DnaError::InvalidLiteral(s.to_string()))
    }

    /// Read an identifier or keyword: [a-zA-Z_][a-zA-Z0-9_]*
    fn read_ident(&mut self) -> Result<Token> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        let slice = &self.source[start..self.pos];
        let s = core::str::from_utf8(slice)
            .map_err(|_| DnaError::SyntaxError(self.line, "invalid UTF-8 in identifier".into()))?;

        // Check for keywords
        let tok = match s {
            "let" => Token::Let,
            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "while" => Token::While,
            "fn" => Token::Fn,
            "end" => Token::End,
            "do" => Token::Do,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "return" => Token::Return,
            "for" => Token::For,
            "to" => Token::To,
            "in" => Token::In,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(s.to_string()),
        };

        Ok(tok)
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::Eof),
            Some(b';') => {
                self.skip_comment();
                // After comment, produce newline or EOF
                if self.peek() == Some(b'\n') {
                    self.advance();
                    Ok(Token::Newline)
                } else {
                    Ok(Token::Eof)
                }
            }
            Some(b'\n') => {
                self.advance();
                Ok(Token::Newline)
            }
            Some(b'+') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::PlusEq)
                } else {
                    Ok(Token::Plus)
                }
            }
            Some(b'-') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::MinusEq)
                } else {
                    Ok(Token::Minus)
                }
            }
            Some(b'*') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::StarEq)
                } else {
                    Ok(Token::Star)
                }
            }
            Some(b'/') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::SlashEq)
                } else {
                    Ok(Token::Slash)
                }
            }
            Some(b'%') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::PercentEq)
                } else {
                    Ok(Token::Percent)
                }
            }
            Some(b'(') => {
                self.advance();
                Ok(Token::LParen)
            }
            Some(b')') => {
                self.advance();
                Ok(Token::RParen)
            }
            Some(b',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some(b'=') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::EqEq)
                } else {
                    Ok(Token::Assign)
                }
            }
            Some(b'!') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::BangEq)
                } else {
                    Err(DnaError::SyntaxError(
                        self.line,
                        "expected '=' after '!'".into(),
                    ))
                }
            }
            Some(b'&') => {
                self.advance();
                Ok(Token::Ampersand)
            }
            Some(b'|') => {
                self.advance();
                Ok(Token::Pipe)
            }
            Some(b'^') => {
                self.advance();
                Ok(Token::Caret)
            }
            Some(b'~') => {
                self.advance();
                Ok(Token::Tilde)
            }
            Some(b'<') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::LtEq)
                } else if self.peek() == Some(b'<') {
                    self.advance();
                    Ok(Token::LtLt)
                } else {
                    Ok(Token::Lt)
                }
            }
            Some(b'>') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    Ok(Token::GtEq)
                } else if self.peek() == Some(b'>') {
                    self.advance();
                    Ok(Token::GtGt)
                } else {
                    Ok(Token::Gt)
                }
            }
            Some(b'.') => {
                self.advance();
                if self.peek() == Some(b'.') {
                    self.advance();
                    Ok(Token::DotDot)
                } else {
                    Err(DnaError::SyntaxError(
                        self.line,
                        "expected '..' (range operator), got single '.'".into(),
                    ))
                }
            }
            Some(ch) if ch.is_ascii_digit() => {
                let n = self.read_int()?;
                Ok(Token::Int(n))
            }
            Some(ch) if ch.is_ascii_alphabetic() || ch == b'_' => self.read_ident(),
            Some(ch) => Err(DnaError::SyntaxError(
                self.line,
                format!("unexpected character: '{}'", ch as char),
            )),
        }
    }

    /// Tokenize entire input into a Vec of (token, line) pairs.
    ///
    /// Each token is paired with the 1-based line number where it starts.
    /// This enables the parser to report accurate line numbers in errors.
    pub fn tokenize(source: &str) -> Result<Vec<(Token, usize)>> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let line = lexer.line();
            let tok = lexer.next_token()?;
            if tok == Token::Eof {
                tokens.push((Token::Eof, line));
                break;
            }
            tokens.push((tok, line));
        }
        Ok(tokens)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Strip line numbers for backward-compatible token assertions.
    fn toks(source: &str) -> Vec<Token> {
        Lexer::tokenize(source)
            .unwrap_or_default()
            .into_iter()
            .map(|(tok, _)| tok)
            .collect()
    }

    #[test]
    fn tokenize_integer() {
        assert_eq!(toks("42"), vec![Token::Int(42), Token::Eof]);
    }

    #[test]
    fn tokenize_addition() {
        let tokens = toks("2 + 3");
        assert_eq!(
            tokens,
            vec![Token::Int(2), Token::Plus, Token::Int(3), Token::Eof]
        );
    }

    #[test]
    fn tokenize_all_arith_ops() {
        let tokens = toks("1+2-3*4/5%6");
        assert_eq!(
            tokens,
            vec![
                Token::Int(1),
                Token::Plus,
                Token::Int(2),
                Token::Minus,
                Token::Int(3),
                Token::Star,
                Token::Int(4),
                Token::Slash,
                Token::Int(5),
                Token::Percent,
                Token::Int(6),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_parens() {
        let tokens = toks("(2 + 3)");
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Int(2),
                Token::Plus,
                Token::Int(3),
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_multiline() {
        let tokens = toks("1\n2");
        assert_eq!(
            tokens,
            vec![Token::Int(1), Token::Newline, Token::Int(2), Token::Eof]
        );
    }

    #[test]
    fn tokenize_comment() {
        let tokens = toks("42 ; this is a comment\n5");
        assert_eq!(
            tokens,
            vec![Token::Int(42), Token::Newline, Token::Int(5), Token::Eof]
        );
    }

    #[test]
    fn tokenize_whitespace_ignored() {
        let tokens = toks("  2  +  3  ");
        assert_eq!(
            tokens,
            vec![Token::Int(2), Token::Plus, Token::Int(3), Token::Eof]
        );
    }

    #[test]
    fn tokenize_invalid_char() {
        let result = Lexer::tokenize("2 @ 3");
        assert!(result.is_err());
    }

    #[test]
    fn tokenize_identifier() {
        let tokens = toks("foo");
        assert_eq!(tokens, vec![Token::Ident("foo".into()), Token::Eof]);
    }

    #[test]
    fn tokenize_identifier_with_digits() {
        let tokens = toks("x1");
        assert_eq!(tokens, vec![Token::Ident("x1".into()), Token::Eof]);
    }

    #[test]
    fn tokenize_underscore_ident() {
        let tokens = toks("_count");
        assert_eq!(tokens, vec![Token::Ident("_count".into()), Token::Eof]);
    }

    #[test]
    fn tokenize_keywords() {
        let tokens = toks("let if else while fn end do return");
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::If,
                Token::Else,
                Token::While,
                Token::Fn,
                Token::End,
                Token::Do,
                Token::Return,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_logical_keywords() {
        let tokens = toks("and or not");
        assert_eq!(tokens, vec![Token::And, Token::Or, Token::Not, Token::Eof]);
    }

    #[test]
    fn tokenize_comparison_ops() {
        let tokens = toks("== != < > <= >=");
        assert_eq!(
            tokens,
            vec![
                Token::EqEq,
                Token::BangEq,
                Token::Lt,
                Token::Gt,
                Token::LtEq,
                Token::GtEq,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_assignment() {
        let tokens = toks("x = 5");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("x".into()),
                Token::Assign,
                Token::Int(5),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_let_stmt() {
        let tokens = toks("let x = 42");
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident("x".into()),
                Token::Assign,
                Token::Int(42),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_comma() {
        let tokens = toks("f(1, 2)");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("f".into()),
                Token::LParen,
                Token::Int(1),
                Token::Comma,
                Token::Int(2),
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_if_stmt() {
        let tokens = toks("if x > 0 do end");
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::Ident("x".into()),
                Token::Gt,
                Token::Int(0),
                Token::Do,
                Token::End,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_bang_alone_is_error() {
        let result = Lexer::tokenize("!");
        assert!(result.is_err());
    }

    #[test]
    fn tokenize_compound_assign() {
        let tokens = toks("x += 1");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("x".into()),
                Token::PlusEq,
                Token::Int(1),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_all_compound_ops() {
        let tokens = toks("+= -= *= /= %=");
        assert_eq!(
            tokens,
            vec![
                Token::PlusEq,
                Token::MinusEq,
                Token::StarEq,
                Token::SlashEq,
                Token::PercentEq,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_plus_vs_plus_eq() {
        // Ensure `+` alone still works
        let tokens = toks("a + b");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::Plus,
                Token::Ident("b".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_range_syntax() {
        let tokens = toks("for i in 0..10 do end");
        assert_eq!(
            tokens,
            vec![
                Token::For,
                Token::Ident("i".into()),
                Token::In,
                Token::Int(0),
                Token::DotDot,
                Token::Int(10),
                Token::Do,
                Token::End,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_dotdot_isolated() {
        let tokens = toks("1..5");
        assert_eq!(
            tokens,
            vec![Token::Int(1), Token::DotDot, Token::Int(5), Token::Eof]
        );
    }

    #[test]
    fn tokenize_single_dot_is_error() {
        let result = Lexer::tokenize(".");
        assert!(result.is_err());
    }

    // --- Boolean literal tests ---

    #[test]
    fn tokenize_true() {
        let tokens = toks("true");
        assert_eq!(tokens, vec![Token::True, Token::Eof]);
    }

    #[test]
    fn tokenize_false() {
        let tokens = toks("false");
        assert_eq!(tokens, vec![Token::False, Token::Eof]);
    }

    #[test]
    fn tokenize_bool_in_expr() {
        let tokens = toks("true and false");
        assert_eq!(
            tokens,
            vec![Token::True, Token::And, Token::False, Token::Eof]
        );
    }

    #[test]
    fn tokenize_true_ident_prefix() {
        // `trueness` is an identifier, not keyword `true` + `ness`
        let tokens = toks("trueness");
        assert_eq!(tokens, vec![Token::Ident("trueness".into()), Token::Eof]);
    }

    #[test]
    fn tokenize_false_ident_prefix() {
        let tokens = toks("falsehood");
        assert_eq!(tokens, vec![Token::Ident("falsehood".into()), Token::Eof]);
    }

    // --- Bitwise operator tests ---

    #[test]
    fn tokenize_ampersand() {
        let tokens = toks("a & b");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::Ampersand,
                Token::Ident("b".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_pipe() {
        let tokens = toks("a | b");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::Pipe,
                Token::Ident("b".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_caret() {
        let tokens = toks("a ^ b");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::Caret,
                Token::Ident("b".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_tilde() {
        let tokens = toks("~x");
        assert_eq!(
            tokens,
            vec![Token::Tilde, Token::Ident("x".into()), Token::Eof]
        );
    }

    #[test]
    fn tokenize_shift_left() {
        let tokens = toks("a << 2");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::LtLt,
                Token::Int(2),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_shift_right() {
        let tokens = toks("a >> 2");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::GtGt,
                Token::Int(2),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_shift_vs_comparison() {
        // `<=` should still work, `<<` should work, `<` alone should work
        let tokens = toks("a <= b");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".into()),
                Token::LtEq,
                Token::Ident("b".into()),
                Token::Eof,
            ]
        );
        let tokens2 = toks("a >= b");
        assert_eq!(
            tokens2,
            vec![
                Token::Ident("a".into()),
                Token::GtEq,
                Token::Ident("b".into()),
                Token::Eof,
            ]
        );
    }

    // --- Elif token test ---

    #[test]
    fn tokenize_elif() {
        let tokens = toks("elif");
        assert_eq!(tokens, vec![Token::Elif, Token::Eof]);
    }

    #[test]
    fn tokenize_elif_in_context() {
        let tokens = toks("if x do\n  1\nelif y do\n  2\nend");
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::Ident("x".into()),
                Token::Do,
                Token::Newline,
                Token::Int(1),
                Token::Newline,
                Token::Elif,
                Token::Ident("y".into()),
                Token::Do,
                Token::Newline,
                Token::Int(2),
                Token::Newline,
                Token::End,
                Token::Eof,
            ]
        );
    }

    // --- Spanned tokenization tests ---

    #[test]
    fn tokenize_spanned_single_line() {
        let spanned = Lexer::tokenize("2 + 3").unwrap_or_default();
        // All tokens on line 1
        for (_, line) in &spanned {
            assert_eq!(*line, 1);
        }
    }

    #[test]
    fn tokenize_spanned_multiline() {
        let spanned = Lexer::tokenize("let x = 1\nx + 2").unwrap_or_default();
        // Line 1 tokens: let, x, =, 1, newline
        // Line 2 tokens: x, +, 2
        let lines: Vec<usize> = spanned.iter().map(|(_, line)| *line).collect();
        // let=1, x=1, ==1, 1=1, \n=1, x=2, +=2, 2=2, eof=2
        assert_eq!(lines[0], 1); // let
        assert_eq!(lines[4], 1); // newline
        assert_eq!(lines[5], 2); // x (line 2)
        assert_eq!(lines[7], 2); // 2 (line 2)
    }

    #[test]
    fn tokenize_spanned_three_lines() {
        let spanned = Lexer::tokenize("1\n2\n3").unwrap_or_default();
        let lines: Vec<usize> = spanned.iter().map(|(_, line)| *line).collect();
        // 1=1, \n=1, 2=2, \n=2, 3=3, eof=3
        assert_eq!(lines[0], 1); // 1
        assert_eq!(lines[2], 2); // 2
        assert_eq!(lines[4], 3); // 3
    }
}
