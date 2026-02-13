// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Lexer
//!
//! Tokenizer grounded in σ (Sequence) — transforms character stream to token stream.
//!
//! ## Mathematical Foundation
//!
//! The lexer is a pure μ (Mapping): σ[char] → σ[Token]
//!
//! ## Tier: T2-P (σ + μ)

use crate::error::{PrimaError, PrimaResult};
use crate::token::{Span, Token, TokenKind};
use lex_primitiva::LexPrimitiva;

/// Prima lexer — μ: σ[char] → σ[Token]
pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    line: usize,
    start: usize,
}

impl<'a> Lexer<'a> {
    /// ∃: Create lexer from source.
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            line: 0,
            start: 0,
        }
    }

    /// μ: σ[char] → σ[Token]
    pub fn tokenize(&mut self) -> PrimaResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.is_eof();
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    /// →: Get next token.
    pub fn next_token(&mut self) -> PrimaResult<Token> {
        self.skip_whitespace();
        self.start = self.pos;

        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        let c = self.advance();
        self.dispatch_char(c)
    }

    /// Σ: Dispatch character to appropriate handler.
    fn dispatch_char(&mut self, c: char) -> PrimaResult<Token> {
        // Special case: {-} Entropy Capsule
        if c == '{' && self.peek() == Some('-') && self.peek_next() == Some('}') {
            self.advance(); // consume '-'
            self.advance(); // consume '}'
            return Ok(self.make_token(TokenKind::Ident("{-}".to_string())));
        }

        // Try primitive symbol first
        if let Some(kind) = self.try_primitive_token(c) {
            return Ok(self.make_token(kind));
        }

        // Special case: κ (kappa) for comparison operators (κ=, κ<, κ>)
        // Must check before is_alphabetic() since Greek letters are alphabetic
        if c == 'κ' {
            return self.kappa_variants();
        }

        // Homoiconicity: `:symbol`, `'expr`, `` `expr ``, `~expr`, `~@expr` syntax
        // Quote (') returns AST as data — ρ (Recursion/self-reference)
        if c == '\'' {
            return Ok(self.make_token(TokenKind::Quote));
        }
        // Quasiquote (`) returns AST with selective unquoting — ρ + σ
        if c == '`' {
            return Ok(self.make_token(TokenKind::Quasiquote));
        }
        // Unquote (~) and Unquote-splice (~@) — → (Causality)
        if c == '~' {
            return self.unquote_variants();
        }

        // Dispatch by character class
        match c {
            '"' => self.string(),
            // Disambiguate `:` — symbol literal `:name` vs type annotation `x: N`
            // Symbol: immediately followed by alphabetic/underscore (`:foo`, `:_bar`)
            // Colon: followed by space, `=`, or type name
            ':' => self.colon_or_symbol(),
            c if c.is_ascii_digit() => self.number(),
            // Identifiers: alphabetic, underscore, or math symbols
            c if Self::is_ident_start(c) => self.identifier_or_underscore(c),
            _ => self.operator_or_delimiter(c),
        }
    }

    /// Disambiguate `:` — symbol literal `:name` vs type annotation colon.
    ///
    /// ## Grammar
    /// - `:name` where name starts with alphabetic/underscore → Symbol (λ)
    /// - `:` followed by space/nothing/type → Colon (μ for type annotation)
    fn colon_or_symbol(&mut self) -> PrimaResult<Token> {
        // Peek ahead to determine if this is a symbol
        match self.peek() {
            Some(c) if c.is_alphabetic() || c == '_' => {
                // Symbol literal: `:name`
                self.symbol_literal()
            }
            _ => {
                // Type annotation colon
                Ok(self.make_token(TokenKind::Colon))
            }
        }
    }

    /// Disambiguate `~` — unquote vs unquote-splice.
    ///
    /// ## Grammar
    /// - `~@` → `UnquoteSplice` (evaluate and splice list)
    /// - `~`  → `Unquote` (evaluate single expression)
    fn unquote_variants(&mut self) -> PrimaResult<Token> {
        if self.peek() == Some('@') {
            self.advance(); // consume '@'
            Ok(self.make_token(TokenKind::UnquoteSplice))
        } else {
            Ok(self.make_token(TokenKind::Unquote))
        }
    }

    /// Parse symbol literal `:name` — λ (Location) primitive.
    /// Symbols evaluate to themselves (like Lisp/Clojure keywords).
    ///
    /// ## Mathematical Grounding
    /// Symbol → λ (Location) → ∃ (Existence) → → (Causality) → 1
    fn symbol_literal(&mut self) -> PrimaResult<Token> {
        // Consume identifier characters after `:`
        while self.peek().is_some_and(|c| c.is_alphanumeric() || c == '_') {
            self.advance();
        }
        let text = &self.source[self.start..self.pos];
        // Strip the leading `:` to get the symbol name
        let name = text[1..].to_string();
        Ok(self.make_token(TokenKind::Symbol(name)))
    }

    /// Try to match a primitive symbol.
    fn try_primitive_token(&self, c: char) -> Option<TokenKind> {
        let prim = match c {
            'σ' => LexPrimitiva::Sequence,
            'μ' => LexPrimitiva::Mapping,
            'ς' => LexPrimitiva::State,
            'ρ' => LexPrimitiva::Recursion,
            '∅' => LexPrimitiva::Void,
            '∂' => LexPrimitiva::Boundary,
            'ν' => LexPrimitiva::Frequency, // Greek nu - physics standard for frequency
            '∃' => LexPrimitiva::Existence,
            'π' => LexPrimitiva::Persistence,
            'λ' => LexPrimitiva::Location,
            '∝' => LexPrimitiva::Irreversibility,
            'Σ' => LexPrimitiva::Sum,
            _ => return None,
        };
        Some(TokenKind::Primitive(prim))
    }

    /// Check if a character is a valid identifier start.
    /// Includes: ASCII letters, underscore, Greek, math symbols.
    fn is_ident_start(c: char) -> bool {
        c.is_alphabetic() || c == '_' || Self::is_math_symbol(c)
    }

    /// Check if a character is a valid identifier continuation.
    fn is_ident_continue(c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '?' || Self::is_math_symbol(c)
    }

    /// Check if character is a math/compressed symbol usable in identifiers.
    ///
    /// Delegates to [`vocabulary::is_math_identifier_char`] — the single source of truth.
    fn is_math_symbol(c: char) -> bool {
        crate::vocabulary::is_math_identifier_char(c)
    }

    /// Handle identifier, keyword, or underscore.
    fn identifier_or_underscore(&mut self, first: char) -> PrimaResult<Token> {
        if first == '_' && !self.peek().is_some_and(Self::is_ident_continue) {
            return Ok(self.make_token(TokenKind::Underscore));
        }
        self.identifier()
    }

    /// Parse identifier or keyword.
    fn identifier(&mut self) -> PrimaResult<Token> {
        while self.peek().is_some_and(Self::is_ident_continue) {
            self.advance();
        }
        let text = &self.source[self.start..self.pos];
        let kind = self.keyword_or_ident(text);
        Ok(self.make_token(kind))
    }

    /// Map text to keyword or identifier.
    fn keyword_or_ident(&self, text: &str) -> TokenKind {
        match text {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "type" => TokenKind::Type,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "N" => TokenKind::Primitive(LexPrimitiva::Quantity),
            // Note: Frequency uses ν (nu), not f - to avoid identifier conflicts
            _ => TokenKind::Ident(text.to_string()),
        }
    }

    /// Handle operators and delimiters.
    fn operator_or_delimiter(&mut self, c: char) -> PrimaResult<Token> {
        let kind = match c {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '%' => TokenKind::Percent,
            // Note: ':' handled in colon_or_symbol() for symbol/annotation disambiguation
            '→' | '≡' | '≢' | '≤' | '≥' | '∧' | '∨' | '¬' | '×' | '÷' | '−' | '‼' | '⟶' =>
            {
                return self.unicode_operator(c);
            }
            '/' => return self.slash_or_comment(),
            '-' => return self.minus_or_arrow(),
            '=' => return self.equal_variants(),
            '!' => return self.not_variants(),
            '<' => return self.less_variants(),
            '>' => return self.greater_variants(),
            '|' => return self.pipe_variants(),
            '&' => return self.and_variant(),
            'κ' => return self.kappa_variants(),
            _ => return Err(self.error(format!("unexpected: '{}'", c))),
        };
        Ok(self.make_token(kind))
    }

    fn unicode_operator(&mut self, c: char) -> PrimaResult<Token> {
        let kind = match c {
            '→' => TokenKind::Arrow,
            '≡' => TokenKind::EqualEqual,
            '≢' => TokenKind::NotEqual,
            '≤' => TokenKind::LessEqual,
            '≥' => TokenKind::GreaterEqual,
            '∧' => TokenKind::And,
            '∨' => TokenKind::Or,
            '¬' => TokenKind::Not,
            '×' => TokenKind::Star,
            '÷' => TokenKind::Slash,
            '−' => TokenKind::Minus,
            '‼' => TokenKind::Ident("assert".to_string()),
            '⟶' => return self.conversion_variants(),
            _ => return Err(self.error("invalid unicode operator")),
        };
        Ok(self.make_token(kind))
    }

    fn conversion_variants(&mut self) -> PrimaResult<Token> {
        let kind = match self.peek() {
            Some('S') => {
                self.advance();
                TokenKind::Ident("to_string".to_string())
            }
            Some('N') => {
                self.advance();
                TokenKind::Ident("to_int".to_string())
            }
            Some('F') => {
                self.advance();
                TokenKind::Ident("to_float".to_string())
            }
            _ => TokenKind::Arrow,
        };
        Ok(self.make_token(kind))
    }

    fn slash_or_comment(&mut self) -> PrimaResult<Token> {
        if self.match_char('/') {
            self.skip_line_comment();
            self.next_token()
        } else {
            Ok(self.make_token(TokenKind::Slash))
        }
    }

    fn minus_or_arrow(&mut self) -> PrimaResult<Token> {
        let kind = if self.match_char('>') {
            TokenKind::Arrow
        } else {
            TokenKind::Minus
        };
        Ok(self.make_token(kind))
    }

    fn equal_variants(&mut self) -> PrimaResult<Token> {
        let kind = if self.match_char('=') {
            TokenKind::EqualEqual
        } else {
            TokenKind::Equal
        };
        Ok(self.make_token(kind))
    }

    fn not_variants(&mut self) -> PrimaResult<Token> {
        let kind = if self.match_char('=') {
            TokenKind::NotEqual
        } else {
            TokenKind::Not
        };
        Ok(self.make_token(kind))
    }

    fn less_variants(&mut self) -> PrimaResult<Token> {
        let kind = if self.match_char('=') {
            TokenKind::LessEqual
        } else {
            TokenKind::Less
        };
        Ok(self.make_token(kind))
    }

    fn greater_variants(&mut self) -> PrimaResult<Token> {
        let kind = if self.match_char('=') {
            TokenKind::GreaterEqual
        } else {
            TokenKind::Greater
        };
        Ok(self.make_token(kind))
    }

    fn pipe_variants(&mut self) -> PrimaResult<Token> {
        let kind = if self.match_char('|') {
            TokenKind::Or
        } else if self.match_char('>') {
            TokenKind::Pipeline
        } else {
            TokenKind::Pipe
        };
        Ok(self.make_token(kind))
    }

    fn and_variant(&mut self) -> PrimaResult<Token> {
        if self.match_char('&') {
            Ok(self.make_token(TokenKind::And))
        } else {
            Err(self.error("expected '&&'"))
        }
    }

    fn kappa_variants(&mut self) -> PrimaResult<Token> {
        let kind = match self.peek() {
            Some('=') => {
                self.advance();
                TokenKind::KappaEq
            }
            Some('!') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::KappaNe
                } else {
                    return Err(self.error("expected '=' after 'κ!'"));
                }
            }
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::KappaLe
                } else {
                    TokenKind::KappaLt
                }
            }
            Some('>') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::KappaGe
                } else {
                    TokenKind::KappaGt
                }
            }
            _ => TokenKind::Primitive(LexPrimitiva::Comparison),
        };
        Ok(self.make_token(kind))
    }

    /// Parse string literal.
    fn string(&mut self) -> PrimaResult<Token> {
        let mut value = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            if c == '\\' {
                self.advance();
                value.push(self.escape_char()?);
            } else {
                value.push(self.advance());
            }
        }
        if self.is_at_end() {
            return Err(self.error("unterminated string"));
        }
        self.advance();
        Ok(self.make_token(TokenKind::String(value)))
    }

    fn escape_char(&mut self) -> PrimaResult<char> {
        match self.peek() {
            Some('n') => {
                self.advance();
                Ok('\n')
            }
            Some('t') => {
                self.advance();
                Ok('\t')
            }
            Some('r') => {
                self.advance();
                Ok('\r')
            }
            Some('\\') => {
                self.advance();
                Ok('\\')
            }
            Some('"') => {
                self.advance();
                Ok('"')
            }
            Some(c) => Err(self.error(format!("invalid escape: \\{}", c))),
            None => Err(self.error("unterminated escape")),
        }
    }

    /// Parse number literal.
    fn number(&mut self) -> PrimaResult<Token> {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
            self.parse_float()
        } else {
            self.parse_int()
        }
    }

    fn parse_float(&self) -> PrimaResult<Token> {
        let text = &self.source[self.start..self.pos];
        let value: f64 = text.parse().map_err(|_| self.error("invalid float"))?;
        Ok(self.make_token(TokenKind::Float(value)))
    }

    fn parse_int(&self) -> PrimaResult<Token> {
        let text = &self.source[self.start..self.pos];
        let value: i64 = text.parse().map_err(|_| self.error("invalid integer"))?;
        Ok(self.make_token(TokenKind::Int(value)))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while self.peek().is_some_and(|c| c != '\n') {
            self.advance();
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.pos..].chars().next().unwrap_or('\0');
        self.pos += c.len_utf8();
        c
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }
    fn peek_next(&self) -> Option<char> {
        self.source[self.pos..].chars().nth(1)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }
    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, Span::new(self.start, self.pos, self.line))
    }
    fn error(&self, message: impl Into<String>) -> PrimaError {
        PrimaError::lexer(Span::new(self.start, self.pos, self.line), message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<TokenKind> {
        Lexer::new(source)
            .tokenize()
            .map(|t| t.into_iter().map(|t| t.kind).collect())
            .unwrap_or_default()
    }

    #[test]
    fn test_root_constants() {
        let tokens = lex("0 1");
        assert_eq!(tokens[0], TokenKind::Int(0));
        assert_eq!(tokens[1], TokenKind::Int(1));
    }

    #[test]
    fn test_primitives() {
        let tokens = lex("σ μ ς ρ ∅ ∂ ∃ π λ ∝ Σ N f");
        assert!(tokens.contains(&TokenKind::Primitive(LexPrimitiva::Sequence)));
        assert!(tokens.contains(&TokenKind::Primitive(LexPrimitiva::Quantity)));
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("fn let if else for match return true false");
        assert!(tokens.contains(&TokenKind::Fn));
        assert!(tokens.contains(&TokenKind::If));
        assert!(tokens.contains(&TokenKind::True));
    }

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / -> → == != κ= κ<");
        assert!(tokens.contains(&TokenKind::Plus));
        assert!(tokens.contains(&TokenKind::Arrow));
        assert!(tokens.contains(&TokenKind::KappaEq));
    }

    #[test]
    fn test_kappa_complete() {
        // All 6 κ operators: κ= κ!= κ< κ> κ<= κ>=
        let tokens = lex("κ= κ!= κ< κ> κ<= κ>=");
        assert!(tokens.contains(&TokenKind::KappaEq));
        assert!(tokens.contains(&TokenKind::KappaNe));
        assert!(tokens.contains(&TokenKind::KappaLt));
        assert!(tokens.contains(&TokenKind::KappaGt));
        assert!(tokens.contains(&TokenKind::KappaLe));
        assert!(tokens.contains(&TokenKind::KappaGe));
    }

    #[test]
    fn test_kappa_le_disambiguation() {
        // κ< vs κ<= — must check lookahead correctly
        let tokens = lex("x κ<= 5");
        assert!(tokens.contains(&TokenKind::KappaLe));
        let tokens2 = lex("x κ< 5");
        assert!(tokens2.contains(&TokenKind::KappaLt));
    }

    #[test]
    fn test_kappa_ge_disambiguation() {
        // κ> vs κ>= — must check lookahead correctly
        let tokens = lex("x κ>= 5");
        assert!(tokens.contains(&TokenKind::KappaGe));
        let tokens2 = lex("x κ> 5");
        assert!(tokens2.contains(&TokenKind::KappaGt));
    }

    #[test]
    fn test_unicode_math() {
        let tokens = lex("≡ ≢ ≤ ≥ ∧ ∨ ¬");
        assert!(tokens.contains(&TokenKind::EqualEqual));
        assert!(tokens.contains(&TokenKind::And));
    }

    #[test]
    fn test_literals() {
        let tokens = lex("42 3.14 \"hello\"");
        assert_eq!(tokens[0], TokenKind::Int(42));
        assert_eq!(tokens[1], TokenKind::Float(3.14));
        assert_eq!(tokens[2], TokenKind::String("hello".into()));
    }

    #[test]
    fn test_function_signature() {
        let tokens = lex("fn add(x: N, y: N) → N");
        assert!(tokens.contains(&TokenKind::Fn));
        assert!(tokens.contains(&TokenKind::Arrow));
    }

    #[test]
    fn test_symbol_literal() {
        // Symbol: `:name` — λ (Location) primitive
        let tokens = lex(":foo :bar_baz :_underscore");
        assert_eq!(tokens[0], TokenKind::Symbol("foo".into()));
        assert_eq!(tokens[1], TokenKind::Symbol("bar_baz".into()));
        assert_eq!(tokens[2], TokenKind::Symbol("_underscore".into()));
    }

    #[test]
    fn test_quote_token() {
        // Quote: `'` — ρ (Recursion) primitive for homoiconicity
        let tokens = lex("'x '42");
        assert_eq!(tokens[0], TokenKind::Quote);
        assert_eq!(tokens[2], TokenKind::Quote);
    }

    #[test]
    fn test_colon_vs_symbol_disambiguation() {
        // `:` followed by type → Colon (type annotation)
        // `:` followed by identifier → Symbol
        let tokens = lex("x: N :sym");
        assert!(tokens.contains(&TokenKind::Colon));
        assert!(tokens.contains(&TokenKind::Symbol("sym".into())));
    }

    #[test]
    fn test_quasiquote_token() {
        // Quasiquote: `` ` `` — ρ + σ for template with unquoting
        let tokens = lex("`x `(1 2 3)");
        assert_eq!(tokens[0], TokenKind::Quasiquote);
        assert_eq!(tokens[2], TokenKind::Quasiquote);
    }

    #[test]
    fn test_unquote_token() {
        // Unquote: `~` — evaluate within quasiquote
        let tokens = lex("~x ~y");
        assert_eq!(tokens[0], TokenKind::Unquote);
        assert_eq!(tokens[2], TokenKind::Unquote);
    }

    #[test]
    fn test_unquote_splice_token() {
        // Unquote-splice: `~@` — evaluate and splice list
        let tokens = lex("~@xs ~@ys");
        assert_eq!(tokens[0], TokenKind::UnquoteSplice);
        assert_eq!(tokens[2], TokenKind::UnquoteSplice);
    }

    #[test]
    fn test_quasiquote_with_unquote() {
        // Combined: `` `(1 ~x 3) ``
        let tokens = lex("`(1 ~x 3)");
        assert_eq!(tokens[0], TokenKind::Quasiquote);
        assert_eq!(tokens[1], TokenKind::LParen);
        assert_eq!(tokens[2], TokenKind::Int(1));
        assert_eq!(tokens[3], TokenKind::Unquote);
        assert_eq!(tokens[4], TokenKind::Ident("x".into()));
    }
}
