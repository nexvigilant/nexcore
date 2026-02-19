// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Parser
//!
//! Recursive descent parser: μ[σ[Token] → AST]
//!
//! ## Tier: T2-C (μ + σ + ρ + Σ)

use crate::ast::*;
use crate::error::{PrimaError, PrimaResult};
use crate::token::{Span, Token, TokenKind};
use lex_primitiva::LexPrimitiva;

/// Static EOF token for reference returns.
static EOF_TOKEN: std::sync::LazyLock<Token> = std::sync::LazyLock::new(|| Token {
    kind: TokenKind::Eof,
    span: Span::default(),
});

/// Parser: μ[σ[Token] → Program]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// ∃: Create parser from tokens.
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SYMBOLIC KEYWORD HELPERS — ASCII ↔ Primitive Symbol Equivalence
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if current token is a specific primitive.
    fn is_primitive(&self, prim: LexPrimitiva) -> bool {
        matches!(&self.peek().kind, TokenKind::Primitive(p) if *p == prim)
    }

    /// μ (Mapping) = fn — function definition
    fn is_fn_keyword(&self) -> bool {
        self.check(&TokenKind::Fn) || self.is_primitive(LexPrimitiva::Mapping)
    }

    /// λ (Location) = let — variable binding
    fn is_let_keyword(&self) -> bool {
        self.check(&TokenKind::Let) || self.is_primitive(LexPrimitiva::Location)
    }

    /// ∂ (Boundary) = if — conditional
    fn is_if_keyword(&self) -> bool {
        self.check(&TokenKind::If) || self.is_primitive(LexPrimitiva::Boundary)
    }

    /// Σ (Sum) = match — pattern matching
    fn is_match_keyword(&self) -> bool {
        self.check(&TokenKind::Match) || self.is_primitive(LexPrimitiva::Sum)
    }

    /// σ (Sequence) = for — iteration
    fn is_for_keyword(&self) -> bool {
        self.check(&TokenKind::For) || self.is_sequence_for()
    }

    /// Check if σ is used as 'for' (followed by identifier) vs literal (followed by '[')
    fn is_sequence_for(&self) -> bool {
        if !self.is_primitive(LexPrimitiva::Sequence) {
            return false;
        }
        // σ followed by identifier = for loop
        // σ followed by [ = sequence literal
        matches!(self.peek_next_kind(), Some(TokenKind::Ident(_)))
    }

    /// → (Causality) = return — value production
    fn is_return_keyword(&self) -> bool {
        self.check(&TokenKind::Return) || self.check(&TokenKind::Arrow)
    }

    fn peek_next_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos + 1).map(|t| &t.kind)
    }

    /// μ: Parse tokens to program.
    pub fn parse(&mut self) -> PrimaResult<Program> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(Program { statements })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATEMENTS (ς modification)
    // ═══════════════════════════════════════════════════════════════════════

    fn statement(&mut self) -> PrimaResult<Stmt> {
        // λ (Location) or 'let' — variable binding
        if self.is_let_keyword() {
            return self.let_stmt();
        }
        // μ (Mapping) or 'fn' — function definition
        if self.is_fn_keyword() {
            return self.fn_stmt();
        }
        // 'type' — type alias (no symbolic equivalent yet)
        if self.check(&TokenKind::Type) {
            return self.type_stmt();
        }
        // → (Causality) or 'return' — value production
        if self.is_return_keyword() {
            return self.return_stmt();
        }
        self.expr_stmt()
    }

    fn let_stmt(&mut self) -> PrimaResult<Stmt> {
        let start = self.advance().span; // consume 'let'
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Equal)?;
        let value = self.expression()?;
        let span = start.merge(value.span());
        Ok(Stmt::Let { name, value, span })
    }

    fn fn_stmt(&mut self) -> PrimaResult<Stmt> {
        let start = self.advance().span;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.param_list()?;
        self.expect(&TokenKind::RParen)?;
        let ret = if self.check(&TokenKind::Arrow) {
            self.advance();
            self.type_expr()?
        } else {
            TypeExpr {
                kind: TypeKind::Void,
                span: self.peek().span,
            }
        };
        let body = self.block()?;
        let span = start.merge(body.span);
        Ok(Stmt::FnDef {
            name,
            params,
            ret,
            body,
            span,
        })
    }

    fn type_stmt(&mut self) -> PrimaResult<Stmt> {
        let start = self.advance().span;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Equal)?;
        let ty = self.type_expr()?;
        let span = start.merge(ty.span);
        Ok(Stmt::TypeDef { name, ty, span })
    }

    fn return_stmt(&mut self) -> PrimaResult<Stmt> {
        let start = self.advance().span;
        let value = if !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        let span = value
            .as_ref()
            .map(|v| start.merge(v.span()))
            .unwrap_or(start);
        Ok(Stmt::Return { value, span })
    }

    fn expr_stmt(&mut self) -> PrimaResult<Stmt> {
        let expr = self.expression()?;
        let span = expr.span();
        Ok(Stmt::Expr { expr, span })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXPRESSIONS (→ value production)
    // ═══════════════════════════════════════════════════════════════════════

    fn expression(&mut self) -> PrimaResult<Expr> {
        self.pipeline_expr()
    }

    /// Pipeline expression: `a |> f |> g` ≡ `g(f(a))`
    /// Grounding: → (Causality) — data flows through transformations
    fn pipeline_expr(&mut self) -> PrimaResult<Expr> {
        let mut left = self.or_expr()?;
        while self.check(&TokenKind::Pipeline) {
            self.advance();
            let func = self.or_expr()?;
            let span = left.span().merge(func.span());
            // Transform `a |> f` into `f(a)`
            left = self.make_pipeline_call(left, func, span)?;
        }
        Ok(left)
    }

    /// Convert pipeline into function call.
    /// `a |> f` → `f(a)`
    /// `a |> f(b)` → `f(a, b)` (partial application)
    fn make_pipeline_call(&self, arg: Expr, func: Expr, span: Span) -> PrimaResult<Expr> {
        match func {
            // `a |> f` → `f(a)`
            Expr::Ident { name, .. } => Ok(Expr::Call {
                func: name,
                args: vec![arg],
                span,
            }),
            // `a |> f(b, c)` → `f(a, b, c)` (prepend arg)
            Expr::Call {
                func: name,
                mut args,
                ..
            } => {
                args.insert(0, arg);
                Ok(Expr::Call {
                    func: name,
                    args,
                    span,
                })
            }
            // `a |> |x| x * 2` → apply lambda to arg
            Expr::Lambda { params, body, .. } => {
                // Create call to anonymous function
                // For now, wrap in a call expression
                Ok(Expr::Call {
                    func: "<pipeline>".into(),
                    args: vec![arg, Expr::Lambda { params, body, span }],
                    span,
                })
            }
            _ => Err(PrimaError::parser(
                span,
                "pipeline requires function or call on right side",
            )),
        }
    }

    fn or_expr(&mut self) -> PrimaResult<Expr> {
        let mut left = self.and_expr()?;
        while self.check(&TokenKind::Or) {
            self.advance();
            let right = self.and_expr()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> PrimaResult<Expr> {
        let mut left = self.equality()?;
        while self.check(&TokenKind::And) {
            self.advance();
            let right = self.equality()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn equality(&mut self) -> PrimaResult<Expr> {
        let mut left = self.comparison()?;
        while let Some(op) = self.match_equality_op() {
            let right = self.comparison()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn match_equality_op(&mut self) -> Option<BinOp> {
        let op = match &self.peek().kind {
            TokenKind::EqualEqual | TokenKind::KappaEq => BinOp::Eq,
            TokenKind::NotEqual | TokenKind::KappaNe => BinOp::Ne,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn comparison(&mut self) -> PrimaResult<Expr> {
        let mut left = self.term()?;
        while let Some(op) = self.match_comparison_op() {
            let right = self.term()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn match_comparison_op(&mut self) -> Option<BinOp> {
        let op = match &self.peek().kind {
            TokenKind::Less | TokenKind::KappaLt => BinOp::Lt,
            TokenKind::Greater | TokenKind::KappaGt => BinOp::Gt,
            TokenKind::LessEqual | TokenKind::KappaLe => BinOp::Le,
            TokenKind::GreaterEqual | TokenKind::KappaGe => BinOp::Ge,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn term(&mut self) -> PrimaResult<Expr> {
        let mut left = self.factor()?;
        while let Some(op) = self.match_term_op() {
            let right = self.factor()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn match_term_op(&mut self) -> Option<BinOp> {
        let op = match &self.peek().kind {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn factor(&mut self) -> PrimaResult<Expr> {
        let mut left = self.unary()?;
        while let Some(op) = self.match_factor_op() {
            let right = self.unary()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn match_factor_op(&mut self) -> Option<BinOp> {
        let op = match &self.peek().kind {
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            TokenKind::Percent => BinOp::Mod,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn unary(&mut self) -> PrimaResult<Expr> {
        if self.check(&TokenKind::Minus) {
            let start = self.advance().span;
            let operand = self.unary()?;
            let span = start.merge(operand.span());
            return Ok(Expr::Unary {
                op: UnOp::Neg,
                operand: Box::new(operand),
                span,
            });
        }
        if self.check(&TokenKind::Not) {
            let start = self.advance().span;
            let operand = self.unary()?;
            let span = start.merge(operand.span());
            return Ok(Expr::Unary {
                op: UnOp::Not,
                operand: Box::new(operand),
                span,
            });
        }
        self.call()
    }

    fn call(&mut self) -> PrimaResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.check(&TokenKind::LParen) {
                expr = self.finish_call(expr)?;
            } else if self.check(&TokenKind::Dot) {
                expr = self.member_access(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> PrimaResult<Expr> {
        self.advance(); // consume '('
        let args = self.arg_list()?;
        let end = self.expect(&TokenKind::RParen)?;
        let span = callee.span().merge(end.span);
        let func = match callee {
            Expr::Ident { name, .. } => name,
            _ => return Err(self.error("expected function name")),
        };
        Ok(Expr::Call { func, args, span })
    }

    fn member_access(&mut self, object: Expr) -> PrimaResult<Expr> {
        self.advance(); // consume '.'
        let field_token = self.advance();
        let field = match &field_token.kind {
            TokenKind::Ident(name) => name.clone(),
            _ => return Err(self.error("expected field name")),
        };
        let span = object.span().merge(field_token.span);
        if self.check(&TokenKind::LParen) {
            self.advance();
            let args = self.arg_list()?;
            let end = self.expect(&TokenKind::RParen)?;
            let span = object.span().merge(end.span);
            Ok(Expr::MethodCall {
                object: Box::new(object),
                method: field,
                args,
                span,
            })
        } else {
            Ok(Expr::Member {
                object: Box::new(object),
                field,
                span,
            })
        }
    }

    fn primary(&mut self) -> PrimaResult<Expr> {
        // ═══════════════════════════════════════════════════════════════════════
        // SYMBOLIC CONTROL FLOW — Check before match to handle dual syntax
        // ═══════════════════════════════════════════════════════════════════════

        // ∂ (Boundary) = if — conditional branching
        if self.is_if_keyword() {
            return self.if_expr();
        }
        // Σ (Sum) = match — pattern dispatch
        if self.is_match_keyword() {
            return self.match_expr();
        }
        // σ (Sequence) = for — iteration (when followed by identifier)
        if self.is_for_keyword() {
            return self.for_expr();
        }

        let token = self.peek().clone();
        match &token.kind {
            // Literals (N): integers, floats, strings, booleans, symbols
            TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::String(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Symbol(_) => self.literal_expr(),
            // Quote (ρ): code-as-data for homoiconicity
            TokenKind::Quote => self.quoted_expr(),
            // Quasiquote (ρ + σ): template with selective unquoting
            TokenKind::Quasiquote => self.quasiquoted_expr(),
            // Unquote (→): evaluate within quasiquote
            TokenKind::Unquote => self.unquoted_expr(),
            // Unquote-splice (→ + σ): evaluate and splice
            TokenKind::UnquoteSplice => self.unquoted_splice_expr(),
            // Identifier (λ): variable reference
            TokenKind::Ident(name) => {
                let name = name.clone();
                let span = self.advance().span;
                Ok(Expr::Ident { name, span })
            }
            // Primitives: ∅, σ[...], μ(...) — as literals/types
            TokenKind::Primitive(p) => self.primitive_expr(*p),
            // Delimiters: grouped, block, lambda
            TokenKind::LParen => self.grouped(),
            TokenKind::LBrace => self.block_expr(),
            TokenKind::Pipe => self.lambda_expr(),
            // Control flow (ASCII fallbacks handled above via helpers)
            TokenKind::If => self.if_expr(),
            TokenKind::Match => self.match_expr(),
            TokenKind::For => self.for_expr(),
            _ => Err(self.error(format!("unexpected token: {}", token.kind))),
        }
    }

    /// Parse literal expression — N (Quantity) primitive.
    fn literal_expr(&mut self) -> PrimaResult<Expr> {
        let token = self.advance();
        let value = match &token.kind {
            TokenKind::Int(n) => Literal::Int(*n),
            TokenKind::Float(n) => Literal::Float(*n),
            TokenKind::String(s) => Literal::String(s.clone()),
            TokenKind::True => Literal::Bool(true),
            TokenKind::False => Literal::Bool(false),
            TokenKind::Symbol(s) => Literal::Symbol(s.clone()),
            _ => return Err(PrimaError::parser(token.span, "expected literal")),
        };
        Ok(Expr::Literal {
            value,
            span: token.span,
        })
    }

    /// Parse primitive expression — ∅, σ[...], μ(...).
    fn primitive_expr(&mut self, prim: LexPrimitiva) -> PrimaResult<Expr> {
        match prim {
            LexPrimitiva::Void => {
                let span = self.advance().span;
                Ok(Expr::Literal {
                    value: Literal::Void,
                    span,
                })
            }
            LexPrimitiva::Sequence => self.sequence_literal(),
            LexPrimitiva::Mapping => self.mapping_literal(),
            _ => Err(self.error(format!("unexpected primitive: {}", prim.symbol()))),
        }
    }

    /// Parse block as expression.
    fn block_expr(&mut self) -> PrimaResult<Expr> {
        let b = self.block()?;
        Ok(Expr::Block {
            span: b.span,
            block: b,
        })
    }

    /// Parse quoted expression: `'expr` — ρ (Recursion) primitive.
    fn quoted_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span; // consume '
        let inner = self.primary()?; // parse the expression after quote
        let span = start.merge(inner.span());
        Ok(Expr::Quoted {
            expr: Box::new(inner),
            span,
        })
    }

    /// Parse quasiquoted expression: `` `expr `` — ρ + σ template.
    fn quasiquoted_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span; // consume `
        let inner = self.primary()?;
        let span = start.merge(inner.span());
        Ok(Expr::Quasiquoted {
            expr: Box::new(inner),
            span,
        })
    }

    /// Parse unquote expression: `~expr` — → (Causality).
    fn unquoted_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span; // consume ~
        let inner = self.primary()?;
        let span = start.merge(inner.span());
        Ok(Expr::Unquoted {
            expr: Box::new(inner),
            span,
        })
    }

    /// Parse unquote-splice: `~@expr` — → + σ.
    fn unquoted_splice_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span; // consume ~@
        let inner = self.primary()?;
        let span = start.merge(inner.span());
        Ok(Expr::UnquotedSplice {
            expr: Box::new(inner),
            span,
        })
    }

    /// Parse lambda: `|params| body` or `|params| → Type { body }`
    fn lambda_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span; // consume |
        let params = self.lambda_params()?;
        self.expect(&TokenKind::Pipe)?;
        let body = self.expression()?;
        let span = start.merge(body.span());
        Ok(Expr::Lambda {
            params,
            body: Box::new(body),
            span,
        })
    }

    /// Parse lambda parameters (without types for inference).
    fn lambda_params(&mut self) -> PrimaResult<Vec<Param>> {
        let mut params = Vec::new();
        if !self.check(&TokenKind::Pipe) {
            params.push(self.lambda_param()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                params.push(self.lambda_param()?);
            }
        }
        Ok(params)
    }

    /// Parse a single lambda parameter (name or name: Type).
    fn lambda_param(&mut self) -> PrimaResult<Param> {
        let name_token = self.advance();
        let name = match &name_token.kind {
            TokenKind::Ident(n) => n.clone(),
            _ => return Err(self.error("expected parameter name")),
        };
        let ty = if self.check(&TokenKind::Colon) {
            self.advance();
            self.type_expr()?
        } else {
            // Inferred type (N by default)
            TypeExpr {
                kind: TypeKind::Infer,
                span: name_token.span,
            }
        };
        Ok(Param {
            name,
            ty: ty.clone(),
            span: name_token.span.merge(ty.span),
        })
    }

    fn sequence_literal(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span; // consume σ
        self.expect(&TokenKind::LBracket)?;
        let elements = self.arg_list()?;
        let end = self.expect(&TokenKind::RBracket)?;
        Ok(Expr::Sequence {
            elements,
            span: start.merge(end.span),
        })
    }

    fn mapping_literal(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span; // consume μ
        self.expect(&TokenKind::LParen)?;
        let pairs = self.pair_list()?;
        let end = self.expect(&TokenKind::RParen)?;
        Ok(Expr::Mapping {
            pairs,
            span: start.merge(end.span),
        })
    }

    fn grouped(&mut self) -> PrimaResult<Expr> {
        self.advance(); // consume '('
        let expr = self.expression()?;
        self.expect(&TokenKind::RParen)?;
        Ok(expr)
    }

    fn if_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span;
        let cond = self.expression()?;
        let then_branch = self.block()?;
        let else_branch = if self.check(&TokenKind::Else) {
            self.advance();
            Some(self.block()?)
        } else {
            None
        };
        let span = else_branch
            .as_ref()
            .map(|b| start.merge(b.span))
            .unwrap_or(start.merge(then_branch.span));
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch,
            else_branch,
            span,
        })
    }

    fn match_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span;
        let scrutinee = self.expression()?;
        self.expect(&TokenKind::LBrace)?;
        let arms = self.match_arms()?;
        let end = self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: start.merge(end.span),
        })
    }

    fn for_expr(&mut self) -> PrimaResult<Expr> {
        let start = self.advance().span;
        let var = self.expect_ident()?;
        self.expect(&TokenKind::In)?;
        let iter = self.expression()?;
        let body = self.block()?;
        Ok(Expr::For {
            var,
            iter: Box::new(iter),
            body: body.clone(),
            span: start.merge(body.span),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════

    fn block(&mut self) -> PrimaResult<Block> {
        let start = self.expect(&TokenKind::LBrace)?.span;
        let mut statements = Vec::new();
        let mut expr = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let stmt = self.statement()?;
            if self.check(&TokenKind::RBrace) {
                if let Stmt::Expr { expr: e, .. } = stmt {
                    expr = Some(Box::new(e));
                } else {
                    statements.push(stmt);
                }
            } else {
                statements.push(stmt);
            }
        }

        let end = self.expect(&TokenKind::RBrace)?;
        Ok(Block {
            statements,
            expr,
            span: start.merge(end.span),
        })
    }

    fn param_list(&mut self) -> PrimaResult<Vec<Param>> {
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            params.push(self.param()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                params.push(self.param()?);
            }
        }
        Ok(params)
    }

    fn param(&mut self) -> PrimaResult<Param> {
        let name_token = self.advance();
        let name = match &name_token.kind {
            TokenKind::Ident(n) => n.clone(),
            _ => return Err(self.error("expected parameter name")),
        };
        self.expect(&TokenKind::Colon)?;
        let ty = self.type_expr()?;
        Ok(Param {
            name,
            ty: ty.clone(),
            span: name_token.span.merge(ty.span),
        })
    }

    fn arg_list(&mut self) -> PrimaResult<Vec<Expr>> {
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) && !self.check(&TokenKind::RBracket) {
            args.push(self.expression()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                args.push(self.expression()?);
            }
        }
        Ok(args)
    }

    fn pair_list(&mut self) -> PrimaResult<Vec<(Expr, Expr)>> {
        let mut pairs = Vec::new();
        if !self.check(&TokenKind::RParen) {
            pairs.push(self.pair()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                pairs.push(self.pair()?);
            }
        }
        Ok(pairs)
    }

    fn pair(&mut self) -> PrimaResult<(Expr, Expr)> {
        let key = self.expression()?;
        self.expect(&TokenKind::Arrow)?;
        let value = self.expression()?;
        Ok((key, value))
    }

    fn match_arms(&mut self) -> PrimaResult<Vec<MatchArm>> {
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            arms.push(self.match_arm()?);
            if !self.check(&TokenKind::RBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }
        Ok(arms)
    }

    fn match_arm(&mut self) -> PrimaResult<MatchArm> {
        let pattern = self.pattern()?;
        self.expect(&TokenKind::Arrow)?;
        let body = self.expression()?;
        let span = pattern.span().merge(body.span());
        Ok(MatchArm {
            pattern,
            body,
            span,
        })
    }

    fn pattern(&mut self) -> PrimaResult<Pattern> {
        let token = self.advance();
        match &token.kind {
            TokenKind::Underscore => Ok(Pattern::Wildcard { span: token.span }),
            TokenKind::Int(n) => Ok(Pattern::Literal {
                value: Literal::Int(*n),
                span: token.span,
            }),
            TokenKind::String(s) => Ok(Pattern::Literal {
                value: Literal::String(s.clone()),
                span: token.span,
            }),
            TokenKind::True => Ok(Pattern::Literal {
                value: Literal::Bool(true),
                span: token.span,
            }),
            TokenKind::False => Ok(Pattern::Literal {
                value: Literal::Bool(false),
                span: token.span,
            }),
            TokenKind::Ident(name) => Ok(Pattern::Ident {
                name: name.clone(),
                span: token.span,
            }),
            _ => Err(self.error("expected pattern")),
        }
    }

    fn type_expr(&mut self) -> PrimaResult<TypeExpr> {
        let token = self.peek().clone();
        let kind = match &token.kind {
            TokenKind::Primitive(LexPrimitiva::Void) => {
                self.advance();
                TypeKind::Void
            }
            TokenKind::Primitive(p) => {
                self.advance();
                TypeKind::Primitive(*p)
            }
            TokenKind::Ident(name) => {
                self.advance();
                TypeKind::Named(name.clone())
            }
            _ => return Err(self.error("expected type")),
        };
        Ok(TypeExpr {
            kind,
            span: token.span,
        })
    }

    fn expect_ident(&mut self) -> PrimaResult<String> {
        let token = self.advance();
        match &token.kind {
            TokenKind::Ident(name) => Ok(name.clone()),
            _ => Err(PrimaError::parser(token.span, "expected identifier")),
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> PrimaResult<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected {}", kind)))
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens.get(self.pos - 1).cloned().unwrap_or_else(|| {
            self.tokens
                .last()
                .cloned()
                .unwrap_or(Token::new(TokenKind::Eof, Span::default()))
        })
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or(self.tokens.last().unwrap_or(&EOF_TOKEN))
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }
    fn error(&self, message: impl Into<String>) -> PrimaError {
        PrimaError::parser(self.peek().span, message)
    }
}

impl Pattern {
    fn span(&self) -> Span {
        match self {
            Self::Wildcard { span }
            | Self::Literal { span, .. }
            | Self::Ident { span, .. }
            | Self::Constructor { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> PrimaResult<Program> {
        let tokens = Lexer::new(src).tokenize()?;
        Parser::new(tokens).parse()
    }

    #[test]
    fn test_let_stmt() {
        let prog = parse("let x = 42").unwrap();
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_fn_def() {
        let prog = parse("fn add(x: N, y: N) → N { x + y }").unwrap();
        assert!(matches!(prog.statements[0], Stmt::FnDef { .. }));
    }

    #[test]
    fn test_binary_expr() {
        let prog = parse("1 + 2 * 3").unwrap();
        assert!(matches!(prog.statements[0], Stmt::Expr { .. }));
    }

    #[test]
    fn test_sequence_literal() {
        let prog = parse("σ[1, 2, 3]").unwrap();
        if let Stmt::Expr {
            expr: Expr::Sequence { elements, .. },
            ..
        } = &prog.statements[0]
        {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("expected sequence");
        }
    }

    #[test]
    fn test_if_expr() {
        let prog = parse("if x κ> 0 { x } else { 0 }").unwrap();
        assert!(matches!(
            prog.statements[0],
            Stmt::Expr {
                expr: Expr::If { .. },
                ..
            }
        ));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SYMBOLIC KEYWORD TESTS — Primitives as Keywords
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_mu_as_fn() {
        // μ (Mapping) = fn
        let prog = parse("μ f(x: N) → N { x }").unwrap();
        assert!(matches!(prog.statements[0], Stmt::FnDef { .. }));
    }

    #[test]
    fn test_lambda_as_let() {
        // λ (Location) = let
        let prog = parse("λ x = 42").unwrap();
        assert!(matches!(prog.statements[0], Stmt::Let { .. }));
    }

    #[test]
    fn test_boundary_as_if() {
        // ∂ (Boundary) = if
        let prog = parse("∂ x { 1 }").unwrap();
        assert!(matches!(
            prog.statements[0],
            Stmt::Expr {
                expr: Expr::If { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_sum_as_match() {
        // Σ (Sum) = match
        let prog = parse("Σ x { 1 → 10, _ → 0 }").unwrap();
        assert!(matches!(
            prog.statements[0],
            Stmt::Expr {
                expr: Expr::Match { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_sequence_as_for() {
        // σ (Sequence) followed by identifier = for loop
        let prog = parse("σ i in σ[1, 2] { i }").unwrap();
        assert!(matches!(
            prog.statements[0],
            Stmt::Expr {
                expr: Expr::For { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_sequence_as_literal() {
        // σ (Sequence) followed by [ = sequence literal
        let prog = parse("σ[1, 2, 3]").unwrap();
        assert!(matches!(
            prog.statements[0],
            Stmt::Expr {
                expr: Expr::Sequence { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_mixed_ascii_and_symbolic() {
        // Both syntaxes can be mixed
        let prog = parse("fn f(x: N) → N { ∂ x κ> 0 { x } else { 0 } }").unwrap();
        assert!(matches!(prog.statements[0], Stmt::FnDef { .. }));
    }
}
