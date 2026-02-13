//! Pratt parser: token stream → AST.
//!
//! Phase 7: Extended with compound assignment (desugared to BinOp+Assign)
//! and range loop syntax (`for i in 0..10 do`).
//! Precedence-climbing parser for expressions, recursive descent for statements.
//!
//! Tier: T2-C (ρ Recursion + σ Sequence + ∂ Boundary + Σ Sum)

use crate::error::{DnaError, Result};
use crate::lang::ast::{BinOp, Expr, Stmt};
use crate::lang::lexer::Token;

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Pratt parser extended with statement support.
///
/// Tokens are spanned: each paired with its 1-based source line number.
///
/// Tier: T2-C (ρ + σ + ∂ + Σ)
pub struct Parser {
    tokens: Vec<(Token, usize)>,
    pos: usize,
}

impl Parser {
    /// Create a parser from a spanned token stream.
    pub fn new(tokens: Vec<(Token, usize)>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Current source line number (1-based).
    fn current_line(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|(_, line)| *line)
            .or_else(|| self.tokens.last().map(|(_, line)| *line))
            .unwrap_or(1)
    }

    /// Peek at the current token.
    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|(tok, _)| tok)
            .unwrap_or(&Token::Eof)
    }

    /// Advance and return the current token.
    fn advance(&mut self) -> Token {
        let tok = self
            .tokens
            .get(self.pos)
            .map(|(tok, _)| tok.clone())
            .unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    /// Skip consecutive newlines.
    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    /// Expect and consume a specific token, or return error.
    fn expect(&mut self, expected: &Token) -> Result<()> {
        let line = self.current_line();
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(DnaError::SyntaxError(
                line,
                format!("expected {expected}, got {tok}"),
            ))
        }
    }

    /// Consume statement separator (newline) if present.
    fn consume_stmt_sep(&mut self) {
        if self.peek() == &Token::Newline {
            self.skip_newlines();
        }
    }

    // -----------------------------------------------------------------------
    // Program / statement parsing
    // -----------------------------------------------------------------------

    /// Parse the entire program into a list of statements.
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();

        self.skip_newlines();

        while self.peek() != &Token::Eof {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.consume_stmt_sep();
        }

        Ok(stmts)
    }

    /// Parse a single statement.
    fn parse_stmt(&mut self) -> Result<Stmt> {
        self.skip_newlines();

        match self.peek() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Fn => self.parse_fn_def(),
            Token::Return => self.parse_return(),
            Token::For => self.parse_for(),
            Token::Ident(_) => {
                // Could be assignment (x = expr) or expression (x + 1)
                if self.is_assignment() {
                    self.parse_assign()
                } else {
                    let expr = self.parse_expr(0)?;
                    Ok(Stmt::ExprStmt(expr))
                }
            }
            _ => {
                let expr = self.parse_expr(0)?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    /// Check if current position is an assignment: Ident followed by `=` or compound assign.
    fn is_assignment(&self) -> bool {
        matches!(self.tokens.get(self.pos), Some((Token::Ident(_), _)))
            && matches!(
                self.tokens.get(self.pos + 1),
                Some((
                    Token::Assign
                        | Token::PlusEq
                        | Token::MinusEq
                        | Token::StarEq
                        | Token::SlashEq
                        | Token::PercentEq,
                    _
                ))
            )
    }

    /// Map a compound assignment token to its BinOp, or None for plain `=`.
    fn compound_assign_op(tok: &Token) -> Option<BinOp> {
        match tok {
            Token::PlusEq => Some(BinOp::Add),
            Token::MinusEq => Some(BinOp::Sub),
            Token::StarEq => Some(BinOp::Mul),
            Token::SlashEq => Some(BinOp::Div),
            Token::PercentEq => Some(BinOp::Mod),
            _ => None,
        }
    }

    /// Parse `let name = expr`
    fn parse_let(&mut self) -> Result<Stmt> {
        self.advance(); // consume 'let'

        let line = self.current_line();
        let name = match self.advance() {
            Token::Ident(name) => name,
            other => {
                return Err(DnaError::SyntaxError(
                    line,
                    format!("expected identifier after 'let', got {other}"),
                ));
            }
        };

        self.expect(&Token::Assign)?;
        let value = self.parse_expr(0)?;

        Ok(Stmt::Let { name, value })
    }

    /// Parse `name = expr` or `name += expr` (compound assignment desugars).
    fn parse_assign(&mut self) -> Result<Stmt> {
        let line = self.current_line();
        let name = match self.advance() {
            Token::Ident(name) => name,
            other => {
                return Err(DnaError::SyntaxError(
                    line,
                    format!("expected identifier for assignment, got {other}"),
                ));
            }
        };

        let op_token = self.advance(); // consume `=`, `+=`, `-=`, etc.
        let rhs = self.parse_expr(0)?;

        // Desugar compound assignment: `x += expr` → `x = x + expr`
        let value = match Self::compound_assign_op(&op_token) {
            Some(op) => Expr::BinOp {
                left: Box::new(Expr::Var(name.clone())),
                op,
                right: Box::new(rhs),
            },
            None => rhs, // plain `=`
        };

        Ok(Stmt::Assign { name, value })
    }

    /// Parse `if cond do body end` or `if cond do body else body end`
    /// or `if cond do body elif cond do body ... end`
    fn parse_if(&mut self) -> Result<Stmt> {
        self.advance(); // consume 'if'

        let cond = self.parse_expr(0)?;
        self.expect(&Token::Do)?;
        self.skip_newlines();

        let then_body = self.parse_block(&[Token::End, Token::Else, Token::Elif])?;

        let else_body = if self.peek() == &Token::Elif {
            // Desugar elif into nested if/else
            let nested_if = self.parse_elif()?;
            vec![nested_if]
        } else if self.peek() == &Token::Else {
            self.advance(); // consume 'else'
            self.skip_newlines();
            let body = self.parse_block(&[Token::End])?;
            self.expect(&Token::End)?;
            body
        } else {
            self.expect(&Token::End)?;
            Vec::new()
        };

        Ok(Stmt::If {
            cond,
            then_body,
            else_body,
        })
    }

    /// Parse an elif chain, desugaring to nested `Stmt::If`.
    ///
    /// Called when current token is `Token::Elif`.
    /// Consumes the elif keyword, condition, `do`, body, and
    /// recursively handles further elif/else/end.
    fn parse_elif(&mut self) -> Result<Stmt> {
        self.advance(); // consume 'elif'

        let cond = self.parse_expr(0)?;
        self.expect(&Token::Do)?;
        self.skip_newlines();

        let then_body = self.parse_block(&[Token::End, Token::Else, Token::Elif])?;

        let else_body = if self.peek() == &Token::Elif {
            let nested_if = self.parse_elif()?;
            vec![nested_if]
        } else if self.peek() == &Token::Else {
            self.advance(); // consume 'else'
            self.skip_newlines();
            let body = self.parse_block(&[Token::End])?;
            self.expect(&Token::End)?;
            body
        } else {
            self.expect(&Token::End)?;
            Vec::new()
        };

        Ok(Stmt::If {
            cond,
            then_body,
            else_body,
        })
    }

    /// Parse `while cond do body end`
    fn parse_while(&mut self) -> Result<Stmt> {
        self.advance(); // consume 'while'

        let cond = self.parse_expr(0)?;
        self.expect(&Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block(&[Token::End])?;
        self.expect(&Token::End)?;

        Ok(Stmt::While { cond, body })
    }

    /// Parse `fn name(params) do body end`
    fn parse_fn_def(&mut self) -> Result<Stmt> {
        self.advance(); // consume 'fn'

        let line = self.current_line();
        let name = match self.advance() {
            Token::Ident(name) => name,
            other => {
                return Err(DnaError::SyntaxError(
                    line,
                    format!("expected function name, got {other}"),
                ));
            }
        };

        // Parse parameter list
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();

        if self.peek() != &Token::RParen {
            loop {
                let param_line = self.current_line();
                match self.advance() {
                    Token::Ident(param) => params.push(param),
                    other => {
                        return Err(DnaError::SyntaxError(
                            param_line,
                            format!("expected parameter name, got {other}"),
                        ));
                    }
                }

                if self.peek() == &Token::Comma {
                    self.advance(); // consume ','
                } else {
                    break;
                }
            }
        }

        self.expect(&Token::RParen)?;
        self.expect(&Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block(&[Token::End])?;
        self.expect(&Token::End)?;

        Ok(Stmt::FnDef { name, params, body })
    }

    /// Parse `return expr`
    fn parse_return(&mut self) -> Result<Stmt> {
        self.advance(); // consume 'return'
        let value = self.parse_expr(0)?;
        Ok(Stmt::Return(value))
    }

    /// Parse `for var = start to end do body end`
    /// or    `for var in start..end do body end`
    fn parse_for(&mut self) -> Result<Stmt> {
        self.advance(); // consume 'for'

        let for_line = self.current_line();
        let var = match self.advance() {
            Token::Ident(name) => name,
            other => {
                return Err(DnaError::SyntaxError(
                    for_line,
                    format!("expected identifier after 'for', got {other}"),
                ));
            }
        };

        // Two syntaxes:
        //   `for i = start to end do`  (original)
        //   `for i in start..end do`   (range syntax)
        let (start, end) = match self.peek() {
            Token::Assign => {
                self.advance(); // consume '='
                let s = self.parse_expr(0)?;
                self.expect(&Token::To)?;
                let e = self.parse_expr(0)?;
                (s, e)
            }
            Token::In => {
                self.advance(); // consume 'in'
                let s = self.parse_expr(0)?;
                self.expect(&Token::DotDot)?;
                let e = self.parse_expr(0)?;
                (s, e)
            }
            other => {
                return Err(DnaError::SyntaxError(
                    self.current_line(),
                    format!("expected '=' or 'in' after for variable, got {other}"),
                ));
            }
        };

        self.expect(&Token::Do)?;
        self.skip_newlines();

        let body = self.parse_block(&[Token::End])?;
        self.expect(&Token::End)?;

        Ok(Stmt::For {
            var,
            start,
            end,
            body,
        })
    }

    /// Parse a block of statements until one of the terminator tokens.
    fn parse_block(&mut self, terminators: &[Token]) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();

        while !terminators.contains(self.peek()) && self.peek() != &Token::Eof {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.consume_stmt_sep();
        }

        Ok(stmts)
    }

    // -----------------------------------------------------------------------
    // Expression parsing (Pratt algorithm)
    // -----------------------------------------------------------------------

    /// Parse an expression with the given minimum precedence.
    pub fn parse_expr(&mut self, min_prec: u8) -> Result<Expr> {
        let mut left = self.parse_prefix()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::Neq,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::Le,
                Token::GtEq => BinOp::Ge,
                Token::Ampersand => BinOp::BitAnd,
                Token::Pipe => BinOp::BitOr,
                Token::Caret => BinOp::BitXor,
                Token::LtLt => BinOp::Shl,
                Token::GtGt => BinOp::Shr,
                Token::And => BinOp::And,
                Token::Or => BinOp::Or,
                _ => break,
            };

            if op.precedence() <= min_prec {
                break;
            }

            self.advance(); // consume operator token
            let right = self.parse_expr(op.precedence())?;

            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse a prefix expression: literal, ident, unary minus/not, paren, or call.
    fn parse_prefix(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Lit(n))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Lit(1))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Lit(0))
            }
            Token::Ident(name) => {
                self.advance();
                // Check if this is a function call: ident followed by (
                if self.peek() == &Token::LParen {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        loop {
                            let arg = self.parse_expr(0)?;
                            args.push(arg);
                            if self.peek() == &Token::Comma {
                                self.advance(); // consume ,
                            } else {
                                break;
                            }
                        }
                    }
                    if self.peek() != &Token::RParen {
                        return Err(DnaError::SyntaxError(
                            self.current_line(),
                            format!("expected ')' in function call, got {}", self.peek()),
                        ));
                    }
                    self.advance(); // consume )
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::Minus => {
                self.advance();
                let inner = self.parse_prefix()?;
                // Optimize: -Lit(n) → Lit(-n) for constant folding
                match inner {
                    Expr::Lit(n) => Ok(Expr::Lit(n.wrapping_neg())),
                    other => Ok(Expr::Neg(Box::new(other))),
                }
            }
            Token::Not => {
                self.advance();
                let inner = self.parse_prefix()?;
                Ok(Expr::Not(Box::new(inner)))
            }
            Token::Tilde => {
                self.advance();
                let inner = self.parse_prefix()?;
                match inner {
                    Expr::Lit(n) => Ok(Expr::Lit(!n)),
                    other => Ok(Expr::BitNot(Box::new(other))),
                }
            }
            Token::LParen => {
                self.advance(); // consume (
                let expr = self.parse_expr(0)?;
                if self.peek() != &Token::RParen {
                    return Err(DnaError::SyntaxError(
                        self.current_line(),
                        format!("expected ')' but got {}", self.peek()),
                    ));
                }
                self.advance(); // consume )
                Ok(expr)
            }
            other => Err(DnaError::SyntaxError(
                self.current_line(),
                format!("unexpected token: {other}"),
            )),
        }
    }
}

/// Convenience: parse source text directly to AST.
pub fn parse(source: &str) -> Result<Vec<Stmt>> {
    let tokens = crate::lang::lexer::Lexer::tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(source: &str) -> Vec<Stmt> {
        parse(source).unwrap_or_default()
    }

    fn first_expr(source: &str) -> Expr {
        let stmts = p(source);
        assert!(!stmts.is_empty(), "no statements parsed from: {source}");
        match stmts.into_iter().next() {
            Some(Stmt::ExprStmt(e)) => e,
            other => panic!("expected ExprStmt, got {other:?}"),
        }
    }

    // --- Expression tests (backward compat with Phase 3) ---

    #[test]
    fn parse_literal() {
        assert_eq!(first_expr("42"), Expr::Lit(42));
    }

    #[test]
    fn parse_addition() {
        let e = first_expr("2 + 3");
        assert_eq!(
            e,
            Expr::BinOp {
                left: Box::new(Expr::Lit(2)),
                op: BinOp::Add,
                right: Box::new(Expr::Lit(3)),
            }
        );
    }

    #[test]
    fn parse_precedence() {
        // 2 + 3 * 4 → 2 + (3 * 4)
        let e = first_expr("2 + 3 * 4");
        assert_eq!(
            e,
            Expr::BinOp {
                left: Box::new(Expr::Lit(2)),
                op: BinOp::Add,
                right: Box::new(Expr::BinOp {
                    left: Box::new(Expr::Lit(3)),
                    op: BinOp::Mul,
                    right: Box::new(Expr::Lit(4)),
                }),
            }
        );
    }

    #[test]
    fn parse_parens_override() {
        let e = first_expr("(2 + 3) * 4");
        assert_eq!(
            e,
            Expr::BinOp {
                left: Box::new(Expr::BinOp {
                    left: Box::new(Expr::Lit(2)),
                    op: BinOp::Add,
                    right: Box::new(Expr::Lit(3)),
                }),
                op: BinOp::Mul,
                right: Box::new(Expr::Lit(4)),
            }
        );
    }

    #[test]
    fn parse_negation() {
        let e = first_expr("-5");
        assert_eq!(e, Expr::Lit(-5));
    }

    #[test]
    fn parse_multiline() {
        let stmts = p("2 + 3\n4 * 5");
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn parse_left_associative() {
        let e = first_expr("10 - 3 - 2");
        assert_eq!(
            e,
            Expr::BinOp {
                left: Box::new(Expr::BinOp {
                    left: Box::new(Expr::Lit(10)),
                    op: BinOp::Sub,
                    right: Box::new(Expr::Lit(3)),
                }),
                op: BinOp::Sub,
                right: Box::new(Expr::Lit(2)),
            }
        );
    }

    #[test]
    fn parse_modulo() {
        let e = first_expr("17 % 5");
        assert_eq!(
            e,
            Expr::BinOp {
                left: Box::new(Expr::Lit(17)),
                op: BinOp::Mod,
                right: Box::new(Expr::Lit(5)),
            }
        );
    }

    #[test]
    fn parse_empty() {
        let stmts = p("");
        assert!(stmts.is_empty());
    }

    #[test]
    fn parse_error_unmatched_paren() {
        let result = parse("(2 + 3");
        assert!(result.is_err());
    }

    // --- Variable tests ---

    #[test]
    fn parse_var_ref() {
        assert_eq!(first_expr("x"), Expr::Var("x".into()));
    }

    #[test]
    fn parse_let_stmt() {
        let stmts = p("let x = 42");
        assert_eq!(stmts.len(), 1);
        assert_eq!(
            stmts[0],
            Stmt::Let {
                name: "x".into(),
                value: Expr::Lit(42),
            }
        );
    }

    #[test]
    fn parse_assign_stmt() {
        let stmts = p("x = 10");
        assert_eq!(stmts.len(), 1);
        assert_eq!(
            stmts[0],
            Stmt::Assign {
                name: "x".into(),
                value: Expr::Lit(10),
            }
        );
    }

    #[test]
    fn parse_let_with_expr() {
        let stmts = p("let y = x + 1");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "y");
                match value {
                    Expr::BinOp { op: BinOp::Add, .. } => {}
                    other => panic!("expected addition, got {other}"),
                }
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // --- Comparison tests ---

    #[test]
    fn parse_comparison() {
        let e = first_expr("x > 0");
        assert_eq!(
            e,
            Expr::BinOp {
                left: Box::new(Expr::Var("x".into())),
                op: BinOp::Gt,
                right: Box::new(Expr::Lit(0)),
            }
        );
    }

    #[test]
    fn parse_equality() {
        let e = first_expr("a == b");
        assert_eq!(
            e,
            Expr::BinOp {
                left: Box::new(Expr::Var("a".into())),
                op: BinOp::Eq,
                right: Box::new(Expr::Var("b".into())),
            }
        );
    }

    #[test]
    fn parse_not_equal() {
        let e = first_expr("x != 0");
        match e {
            Expr::BinOp { op: BinOp::Neq, .. } => {}
            other => panic!("expected Neq, got {other}"),
        }
    }

    #[test]
    fn parse_le_ge() {
        let e = first_expr("x <= 10");
        match e {
            Expr::BinOp { op: BinOp::Le, .. } => {}
            other => panic!("expected Le, got {other}"),
        }
    }

    // --- Logic tests ---

    #[test]
    fn parse_logical_and() {
        let e = first_expr("a and b");
        match e {
            Expr::BinOp { op: BinOp::And, .. } => {}
            other => panic!("expected And, got {other}"),
        }
    }

    #[test]
    fn parse_logical_or() {
        let e = first_expr("a or b");
        match e {
            Expr::BinOp { op: BinOp::Or, .. } => {}
            other => panic!("expected Or, got {other}"),
        }
    }

    #[test]
    fn parse_not() {
        let e = first_expr("not x");
        assert_eq!(e, Expr::Not(Box::new(Expr::Var("x".into()))));
    }

    #[test]
    fn parse_logic_precedence() {
        // a or b and c → a or (b and c)
        let e = first_expr("a or b and c");
        match e {
            Expr::BinOp {
                op: BinOp::Or,
                right,
                ..
            } => match *right {
                Expr::BinOp { op: BinOp::And, .. } => {}
                other => panic!("expected And in right, got {other}"),
            },
            other => panic!("expected Or at top, got {other}"),
        }
    }

    // --- Control flow tests ---

    #[test]
    fn parse_if_simple() {
        let stmts = p("if x > 0 do\n  x\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_empty());
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn parse_if_else() {
        let stmts = p("if x > 0 do\n  1\nelse\n  0\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 1);
                assert_eq!(else_body.len(), 1);
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn parse_while_loop() {
        let stmts = p("while x > 0 do\n  x = x - 1\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::While { body, .. } => {
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected While, got {other:?}"),
        }
    }

    // --- Function tests ---

    #[test]
    fn parse_fn_def() {
        let stmts = p("fn add(a, b) do\n  return a + b\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::FnDef { name, params, body } => {
                assert_eq!(name, "add");
                assert_eq!(params, &["a".to_string(), "b".to_string()]);
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Stmt::Return(_) => {}
                    other => panic!("expected Return, got {other:?}"),
                }
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_fn_no_params() {
        let stmts = p("fn hello() do\n  42\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::FnDef { params, .. } => {
                assert!(params.is_empty());
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_fn_call() {
        let e = first_expr("add(1, 2)");
        assert_eq!(
            e,
            Expr::Call {
                name: "add".into(),
                args: vec![Expr::Lit(1), Expr::Lit(2)],
            }
        );
    }

    #[test]
    fn parse_fn_call_no_args() {
        let e = first_expr("hello()");
        assert_eq!(
            e,
            Expr::Call {
                name: "hello".into(),
                args: vec![],
            }
        );
    }

    // --- Return test ---

    #[test]
    fn parse_return() {
        let stmts = p("return 42");
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0], Stmt::Return(Expr::Lit(42)));
    }

    // --- Combined tests ---

    #[test]
    fn parse_full_program() {
        let source = "
let x = 10
let y = 20
x + y
";
        let stmts = p(source);
        assert_eq!(stmts.len(), 3);
    }

    #[test]
    fn parse_comment_in_program() {
        let stmts = p("42 ; the answer");
        assert_eq!(stmts.len(), 1);
    }

    // --- Compound assignment tests ---

    #[test]
    fn parse_plus_eq() {
        // `x += 5` desugars to `x = x + 5`
        let stmts = p("x += 5");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Assign { name, value } => {
                assert_eq!(name, "x");
                match value {
                    Expr::BinOp {
                        left,
                        op: BinOp::Add,
                        right,
                    } => {
                        assert_eq!(**left, Expr::Var("x".into()));
                        assert_eq!(**right, Expr::Lit(5));
                    }
                    other => panic!("expected x + 5, got {other}"),
                }
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn parse_minus_eq() {
        let stmts = p("count -= 1");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Assign { name, value } => {
                assert_eq!(name, "count");
                match value {
                    Expr::BinOp { op: BinOp::Sub, .. } => {}
                    other => panic!("expected subtraction, got {other}"),
                }
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn parse_star_eq() {
        let stmts = p("total *= 2");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Assign { name, value } => {
                assert_eq!(name, "total");
                match value {
                    Expr::BinOp { op: BinOp::Mul, .. } => {}
                    other => panic!("expected multiplication, got {other}"),
                }
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn parse_slash_eq() {
        match &p("half /= 2")[0] {
            Stmt::Assign { name, value } => {
                assert_eq!(name, "half");
                match value {
                    Expr::BinOp { op: BinOp::Div, .. } => {}
                    other => panic!("expected division, got {other}"),
                }
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn parse_percent_eq() {
        match &p("rem %= 3")[0] {
            Stmt::Assign { name, value } => {
                assert_eq!(name, "rem");
                match value {
                    Expr::BinOp { op: BinOp::Mod, .. } => {}
                    other => panic!("expected modulo, got {other}"),
                }
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn parse_compound_with_expr() {
        // `x += a * b` desugars to `x = x + (a * b)`
        let stmts = p("x += a * b");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Assign { name, value } => {
                assert_eq!(name, "x");
                // RHS should be BinOp(Var("x"), Add, BinOp(Var("a"), Mul, Var("b")))
                match value {
                    Expr::BinOp {
                        left,
                        op: BinOp::Add,
                        right,
                    } => {
                        assert_eq!(**left, Expr::Var("x".into()));
                        match right.as_ref() {
                            Expr::BinOp { op: BinOp::Mul, .. } => {}
                            other => panic!("expected multiplication in RHS, got {other}"),
                        }
                    }
                    other => panic!("expected addition, got {other}"),
                }
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    // --- Range loop syntax tests ---

    #[test]
    fn parse_for_in_range() {
        let stmts = p("for i in 0..10 do\n  i\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::For {
                var,
                start,
                end,
                body,
            } => {
                assert_eq!(var, "i");
                assert_eq!(*start, Expr::Lit(0));
                assert_eq!(*end, Expr::Lit(10));
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected For, got {other:?}"),
        }
    }

    #[test]
    fn parse_for_in_range_with_exprs() {
        // `for x in 1..n do` — range bounds can be expressions
        let stmts = p("for x in 1..n do\n  x\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::For {
                var, start, end, ..
            } => {
                assert_eq!(var, "x");
                assert_eq!(*start, Expr::Lit(1));
                assert_eq!(*end, Expr::Var("n".into()));
            }
            other => panic!("expected For, got {other:?}"),
        }
    }

    #[test]
    fn parse_original_for_still_works() {
        // Original syntax must not regress
        let stmts = p("for i = 1 to 10 do\n  i\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::For { var, .. } => assert_eq!(var, "i"),
            other => panic!("expected For, got {other:?}"),
        }
    }

    // --- Boolean literal tests ---

    #[test]
    fn parse_true() {
        assert_eq!(first_expr("true"), Expr::Lit(1));
    }

    #[test]
    fn parse_false() {
        assert_eq!(first_expr("false"), Expr::Lit(0));
    }

    #[test]
    fn parse_bool_in_condition() {
        let stmts = p("if true do\n  42\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::If { cond, .. } => assert_eq!(*cond, Expr::Lit(1)),
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn parse_bool_logic() {
        let e = first_expr("true and false");
        match e {
            Expr::BinOp {
                left,
                op: BinOp::And,
                right,
            } => {
                assert_eq!(*left, Expr::Lit(1));
                assert_eq!(*right, Expr::Lit(0));
            }
            other => panic!("expected And, got {other}"),
        }
    }

    #[test]
    fn parse_not_true() {
        let e = first_expr("not true");
        assert_eq!(e, Expr::Not(Box::new(Expr::Lit(1))));
    }

    #[test]
    fn parse_let_bool() {
        let stmts = p("let flag = true");
        assert_eq!(stmts.len(), 1);
        assert_eq!(
            stmts[0],
            Stmt::Let {
                name: "flag".into(),
                value: Expr::Lit(1),
            }
        );
    }

    // --- Bitwise operator parser tests ---

    #[test]
    fn parse_bitwise_and() {
        let e = first_expr("a & b");
        match e {
            Expr::BinOp {
                op: BinOp::BitAnd, ..
            } => {}
            other => panic!("expected BitAnd, got {other}"),
        }
    }

    #[test]
    fn parse_bitwise_or() {
        let e = first_expr("a | b");
        match e {
            Expr::BinOp {
                op: BinOp::BitOr, ..
            } => {}
            other => panic!("expected BitOr, got {other}"),
        }
    }

    #[test]
    fn parse_bitwise_xor() {
        let e = first_expr("a ^ b");
        match e {
            Expr::BinOp {
                op: BinOp::BitXor, ..
            } => {}
            other => panic!("expected BitXor, got {other}"),
        }
    }

    #[test]
    fn parse_shift_left() {
        let e = first_expr("a << 2");
        match e {
            Expr::BinOp { op: BinOp::Shl, .. } => {}
            other => panic!("expected Shl, got {other}"),
        }
    }

    #[test]
    fn parse_shift_right() {
        let e = first_expr("a >> 2");
        match e {
            Expr::BinOp { op: BinOp::Shr, .. } => {}
            other => panic!("expected Shr, got {other}"),
        }
    }

    #[test]
    fn parse_bitnot() {
        let e = first_expr("~x");
        match e {
            Expr::BitNot(_) => {}
            other => panic!("expected BitNot, got {other}"),
        }
    }

    #[test]
    fn parse_bitnot_const() {
        // ~5 folds to Lit(!5) = Lit(-6) in parser
        let e = first_expr("~5");
        assert_eq!(e, Expr::Lit(!5));
    }

    #[test]
    fn parse_bitwise_precedence() {
        // a + b & c → (a + b) & c (add binds tighter than bitand)
        let e = first_expr("a + b & c");
        match e {
            Expr::BinOp {
                op: BinOp::BitAnd,
                left,
                ..
            } => match *left {
                Expr::BinOp { op: BinOp::Add, .. } => {}
                other => panic!("expected Add in left, got {other}"),
            },
            other => panic!("expected BitAnd at top, got {other}"),
        }
    }

    // --- Elif parser tests ---

    #[test]
    fn parse_elif_simple() {
        let stmts = p("if false do\n  1\nelif true do\n  2\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::If { else_body, .. } => {
                assert_eq!(else_body.len(), 1);
                // else_body[0] should be a nested If
                match &else_body[0] {
                    Stmt::If { then_body, .. } => {
                        assert_eq!(then_body.len(), 1);
                    }
                    other => panic!("expected nested If, got {other:?}"),
                }
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn parse_elif_three_way() {
        let stmts = p("if false do\n  1\nelif false do\n  2\nelif true do\n  3\nend");
        assert_eq!(stmts.len(), 1);
        // Verify triple nesting
        match &stmts[0] {
            Stmt::If { else_body, .. } => {
                assert_eq!(else_body.len(), 1);
                match &else_body[0] {
                    Stmt::If { else_body: eb2, .. } => {
                        assert_eq!(eb2.len(), 1);
                        assert!(matches!(&eb2[0], Stmt::If { .. }));
                    }
                    other => panic!("expected nested If, got {other:?}"),
                }
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn parse_elif_with_else() {
        let stmts = p("if false do\n  1\nelif false do\n  2\nelse\n  3\nend");
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::If { else_body, .. } => {
                assert_eq!(else_body.len(), 1);
                match &else_body[0] {
                    Stmt::If { else_body: eb2, .. } => {
                        assert_eq!(eb2.len(), 1);
                        // Final else body is direct (not a nested If)
                        assert!(matches!(&eb2[0], Stmt::ExprStmt(_)));
                    }
                    other => panic!("expected nested If, got {other:?}"),
                }
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn parse_elif_multiline_blocks() {
        let source = "\
if x > 10 do
  let a = 1
  a
elif x > 5 do
  let b = 2
  b
else
  let c = 3
  c
end";
        let stmts = p(source);
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 2);
                assert_eq!(else_body.len(), 1);
                match &else_body[0] {
                    Stmt::If {
                        then_body: tb2,
                        else_body: eb2,
                        ..
                    } => {
                        assert_eq!(tb2.len(), 2);
                        assert_eq!(eb2.len(), 2);
                    }
                    other => panic!("expected nested If, got {other:?}"),
                }
            }
            other => panic!("expected If, got {other:?}"),
        }
    }
}
