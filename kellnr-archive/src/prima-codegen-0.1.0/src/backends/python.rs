// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Python code generation backend.
//!
//! ## Tier: T2-C (μ + σ + → + Σ)
//!
//! Maps Prima constructs to idiomatic Python code.

use crate::{Backend, CodegenError, EmitContext, PrimitiveMapping, TargetConstruct};
use lex_primitiva::LexPrimitiva;
use prima::ast::{BinOp, Expr, Literal, Pattern, TypeKind, UnOp};
use prima::prelude::{Block, Program, Stmt};

/// Python code generation backend.
///
/// ## Tier: T2-C (μ + ς)
#[derive(Debug, Clone, Default)]
pub struct PythonBackend {
    mapping: PrimitiveMapping,
}

impl PythonBackend {
    /// Create new Python backend
    #[must_use]
    pub fn new() -> Self {
        Self {
            mapping: PrimitiveMapping::python(),
        }
    }

    /// Emit a type hint from TypeKind
    fn emit_type_kind(&self, kind: &TypeKind) -> String {
        match kind {
            TypeKind::Primitive(p) => match p {
                LexPrimitiva::Quantity => "int".to_string(),
                LexPrimitiva::Sequence => "list".to_string(),
                LexPrimitiva::Void => "None".to_string(),
                _ => format!("{:?}", p),
            },
            TypeKind::Named(name) => name.clone(),
            TypeKind::Sequence(inner) => format!("list[{}]", self.emit_type_kind(&inner.kind)),
            TypeKind::Mapping(k, v) => {
                format!(
                    "dict[{}, {}]",
                    self.emit_type_kind(&k.kind),
                    self.emit_type_kind(&v.kind)
                )
            }
            TypeKind::Sum(variants) => {
                let parts: Vec<String> = variants
                    .iter()
                    .map(|t| self.emit_type_kind(&t.kind))
                    .collect();
                format!("Union[{}]", parts.join(", "))
            }
            TypeKind::Function(params, ret) => {
                let p: Vec<String> = params
                    .iter()
                    .map(|t| self.emit_type_kind(&t.kind))
                    .collect();
                format!(
                    "Callable[[{}], {}]",
                    p.join(", "),
                    self.emit_type_kind(&ret.kind)
                )
            }
            TypeKind::Optional(inner) => format!("Optional[{}]", self.emit_type_kind(&inner.kind)),
            TypeKind::Void => "None".to_string(),
            TypeKind::Infer => "Any".to_string(),
        }
    }

    /// Emit a pattern for match arms
    fn emit_pattern(
        &self,
        pattern: &Pattern,
        ctx: &mut EmitContext,
    ) -> Result<String, CodegenError> {
        match pattern {
            Pattern::Wildcard { .. } => Ok("_".to_string()),
            Pattern::Literal { value, .. } => {
                match value {
                    Literal::Int(n) => Ok(n.to_string()),
                    Literal::Float(f) => Ok(f.to_string()),
                    Literal::String(s) => Ok(format!("\"{}\"", s)),
                    Literal::Bool(b) => Ok(if *b {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    }),
                    Literal::Void => Ok("None".to_string()),
                    Literal::Symbol(s) => Ok(format!("\"{}\"", s)), // Python uses strings for symbols
                }
            }
            Pattern::Ident { name, .. } => {
                ctx.add_to_scope(name);
                Ok(name.clone())
            }
            Pattern::Constructor { name, fields, .. } => {
                if fields.is_empty() {
                    Ok(name.clone())
                } else {
                    let field_strs: Result<Vec<_>, _> =
                        fields.iter().map(|p| self.emit_pattern(p, ctx)).collect();
                    Ok(format!("{}({})", name, field_strs?.join(", ")))
                }
            }
        }
    }

    /// Emit a block
    fn emit_block(&self, block: &Block, ctx: &mut EmitContext) -> Result<String, CodegenError> {
        let mut lines = Vec::new();
        if block.statements.is_empty() && block.expr.is_none() {
            lines.push(format!("{}pass", ctx.indentation()));
        } else {
            for stmt in &block.statements {
                lines.push(self.emit_stmt(stmt, ctx)?);
            }
            if let Some(expr) = &block.expr {
                // Statement-like expressions (If, For, Match) handle their own indentation and returns
                if Self::is_statement_expr(expr) {
                    lines.push(self.emit_expr(expr, ctx)?);
                } else {
                    lines.push(format!(
                        "{}return {}",
                        ctx.indentation(),
                        self.emit_expr(expr, ctx)?
                    ));
                }
            }
        }
        Ok(lines.join("\n"))
    }

    /// Check if expression is statement-like (emits its own structure, not a value)
    fn is_statement_expr(expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::If { .. } | Expr::For { .. } | Expr::Match { .. }
        )
    }
}

impl Backend for PythonBackend {
    fn name(&self) -> &'static str {
        "Python"
    }

    fn extension(&self) -> &'static str {
        "py"
    }

    fn map_primitive(&self, prim: LexPrimitiva) -> TargetConstruct {
        self.mapping.get(prim).cloned().unwrap_or_else(|| {
            TargetConstruct::new("# unknown", crate::primitives::ConstructCategory::Type)
        })
    }

    fn emit_program(
        &self,
        program: &Program,
        ctx: &mut EmitContext,
    ) -> Result<String, CodegenError> {
        let mut lines = Vec::new();
        lines.push("# Generated by Prima Code Generator".to_string());
        lines.push("# Tier: T2-C (μ + σ + →)".to_string());
        lines.push(String::new());

        for stmt in &program.statements {
            lines.push(self.emit_stmt(stmt, ctx)?);
        }

        Ok(lines.join("\n"))
    }

    fn emit_stmt(&self, stmt: &Stmt, ctx: &mut EmitContext) -> Result<String, CodegenError> {
        let indent = ctx.indentation();
        match stmt {
            Stmt::Let { name, value, .. } => {
                ctx.record_primitive(LexPrimitiva::Location);
                ctx.add_to_scope(name);
                let val = self.emit_expr(value, ctx)?;
                Ok(format!("{}{} = {}", indent, name, val))
            }

            Stmt::FnDef {
                name,
                params,
                body,
                ret,
                ..
            } => {
                ctx.record_primitive(LexPrimitiva::Mapping);
                ctx.record_primitive(LexPrimitiva::Causality);

                let param_str: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.emit_type_kind(&p.ty.kind)))
                    .collect();

                let ret_str = format!(" -> {}", self.emit_type_kind(&ret.kind));

                let mut fn_lines = vec![format!(
                    "{}def {}({}){}:",
                    indent,
                    name,
                    param_str.join(", "),
                    ret_str
                )];

                ctx.indent();
                fn_lines.push(self.emit_block(body, ctx)?);
                ctx.dedent();

                Ok(fn_lines.join("\n"))
            }

            Stmt::Expr { expr, .. } => {
                let e = self.emit_expr(expr, ctx)?;
                Ok(format!("{}{}", indent, e))
            }

            Stmt::Return { value, .. } => {
                ctx.record_primitive(LexPrimitiva::Causality);
                match value {
                    Some(e) => {
                        let val = self.emit_expr(e, ctx)?;
                        Ok(format!("{}return {}", indent, val))
                    }
                    None => Ok(format!("{}return", indent)),
                }
            }

            Stmt::TypeDef { name, .. } => {
                ctx.warn(format!("Type definition {} not yet supported", name));
                Ok(format!("{}# type {} = ...", indent, name))
            }
        }
    }

    fn emit_expr(&self, expr: &Expr, ctx: &mut EmitContext) -> Result<String, CodegenError> {
        match expr {
            Expr::Literal { value, .. } => {
                match value {
                    Literal::Int(n) => {
                        ctx.record_primitive(LexPrimitiva::Quantity);
                        Ok(n.to_string())
                    }
                    Literal::Float(f) => {
                        ctx.record_primitive(LexPrimitiva::Quantity);
                        Ok(f.to_string())
                    }
                    Literal::String(s) => Ok(format!("\"{}\"", s)),
                    Literal::Bool(b) => {
                        ctx.record_primitive(LexPrimitiva::Comparison);
                        Ok(if *b {
                            "True".to_string()
                        } else {
                            "False".to_string()
                        })
                    }
                    Literal::Void => {
                        ctx.record_primitive(LexPrimitiva::Void);
                        Ok("None".to_string())
                    }
                    Literal::Symbol(s) => {
                        ctx.record_primitive(LexPrimitiva::Location);
                        // Python uses strings for symbols
                        Ok(format!("\"{}\"", s))
                    }
                }
            }

            Expr::Ident { name, .. } => {
                if !ctx.in_scope(name) {
                    ctx.warn(format!("Identifier '{}' not in scope", name));
                }
                Ok(name.clone())
            }

            Expr::Binary {
                left, op, right, ..
            } => {
                ctx.record_primitive(LexPrimitiva::Causality);
                let l = self.emit_expr(left, ctx)?;
                let r = self.emit_expr(right, ctx)?;
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq | BinOp::KappaEq => "==",
                    BinOp::Ne | BinOp::KappaNe => "!=",
                    BinOp::Lt | BinOp::KappaLt => "<",
                    BinOp::Le | BinOp::KappaLe => "<=",
                    BinOp::Gt | BinOp::KappaGt => ">",
                    BinOp::Ge | BinOp::KappaGe => ">=",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                };
                Ok(format!("({} {} {})", l, op_str, r))
            }

            Expr::Unary { op, operand, .. } => {
                let o = self.emit_expr(operand, ctx)?;
                let op_str = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "not ",
                };
                Ok(format!("({}{})", op_str, o))
            }

            Expr::Call { func, args, .. } => {
                ctx.record_primitive(LexPrimitiva::Mapping);
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_expr(a, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(format!("{}({})", func, arg_strs.join(", ")))
            }

            Expr::Sequence { elements, .. } => {
                ctx.record_primitive(LexPrimitiva::Sequence);
                let elems: Vec<String> = elements
                    .iter()
                    .map(|i| self.emit_expr(i, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(format!("[{}]", elems.join(", ")))
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                ctx.record_primitive(LexPrimitiva::Boundary);
                let c = self.emit_expr(cond, ctx)?;
                // Include indentation since this is a statement-like expression
                let mut lines = vec![format!("{}if {}:", ctx.indentation(), c)];
                ctx.indent();
                lines.push(self.emit_block(then_branch, ctx)?);
                ctx.dedent();
                if let Some(else_b) = else_branch {
                    lines.push(format!("{}else:", ctx.indentation()));
                    ctx.indent();
                    lines.push(self.emit_block(else_b, ctx)?);
                    ctx.dedent();
                }
                Ok(lines.join("\n"))
            }

            Expr::For {
                var, iter, body, ..
            } => {
                ctx.record_primitive(LexPrimitiva::Sequence);
                ctx.record_primitive(LexPrimitiva::Frequency);
                let i = self.emit_expr(iter, ctx)?;
                let mut lines = vec![format!("for {} in {}:", var, i)];
                ctx.indent();
                lines.push(self.emit_block(body, ctx)?);
                ctx.dedent();
                Ok(lines.join("\n"))
            }

            Expr::Match {
                scrutinee, arms, ..
            } => {
                ctx.record_primitive(LexPrimitiva::Sum);
                let s = self.emit_expr(scrutinee, ctx)?;
                let mut lines = vec![format!("match {}:", s)];
                ctx.indent();
                for arm in arms {
                    let pat = self.emit_pattern(&arm.pattern, ctx)?;
                    let body = self.emit_expr(&arm.body, ctx)?;
                    lines.push(format!(
                        "{}case {}: return {}",
                        ctx.indentation(),
                        pat,
                        body
                    ));
                }
                ctx.dedent();
                Ok(lines.join("\n"))
            }

            Expr::Lambda { params, body, .. } => {
                ctx.record_primitive(LexPrimitiva::Mapping);
                let param_str: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let b = self.emit_expr(body, ctx)?;
                Ok(format!("lambda {}: {}", param_str.join(", "), b))
            }

            _ => {
                ctx.warn("Unhandled expression type".to_string());
                Ok("# unhandled".to_string())
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use prima::ast::{Param, TypeExpr, TypeKind};
    use prima::prelude::{Lexer, Parser};
    use prima::token::Span;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new(source).tokenize().ok().unwrap_or_default();
        Parser::new(tokens)
            .parse()
            .unwrap_or_else(|_| Program { statements: vec![] })
    }

    fn dummy_span() -> Span {
        Span::default()
    }

    fn dummy_type() -> TypeExpr {
        TypeExpr {
            kind: TypeKind::Infer,
            span: dummy_span(),
        }
    }

    #[test]
    fn test_python_backend_name() {
        let backend = PythonBackend::new();
        assert_eq!(backend.name(), "Python");
        assert_eq!(backend.extension(), "py");
    }

    #[test]
    fn test_emit_bool_true() {
        let backend = PythonBackend::new();
        let mut ctx = EmitContext::new();
        let expr = Expr::Literal {
            value: Literal::Bool(true),
            span: dummy_span(),
        };
        let code = backend.emit_expr(&expr, &mut ctx);
        assert!(code.is_ok());
        assert_eq!(code.ok().unwrap_or_default(), "True");
    }

    #[test]
    fn test_emit_bool_false() {
        let backend = PythonBackend::new();
        let mut ctx = EmitContext::new();
        let expr = Expr::Literal {
            value: Literal::Bool(false),
            span: dummy_span(),
        };
        let code = backend.emit_expr(&expr, &mut ctx);
        assert!(code.is_ok());
        assert_eq!(code.ok().unwrap_or_default(), "False");
    }

    #[test]
    fn test_emit_void_is_none() {
        let backend = PythonBackend::new();
        let mut ctx = EmitContext::new();
        let expr = Expr::Literal {
            value: Literal::Void,
            span: dummy_span(),
        };
        let code = backend.emit_expr(&expr, &mut ctx);
        assert!(code.is_ok());
        assert_eq!(code.ok().unwrap_or_default(), "None");
    }

    #[test]
    fn test_emit_sequence() {
        let backend = PythonBackend::new();
        let mut ctx = EmitContext::new();
        let expr = Expr::Sequence {
            elements: vec![
                Expr::Literal {
                    value: Literal::Int(1),
                    span: dummy_span(),
                },
                Expr::Literal {
                    value: Literal::Int(2),
                    span: dummy_span(),
                },
                Expr::Literal {
                    value: Literal::Int(3),
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        let code = backend.emit_expr(&expr, &mut ctx);
        assert!(code.is_ok());
        assert_eq!(code.ok().unwrap_or_default(), "[1, 2, 3]");
    }

    #[test]
    fn test_emit_lambda() {
        let backend = PythonBackend::new();
        let mut ctx = EmitContext::new();
        let expr = Expr::Lambda {
            params: vec![Param {
                name: "x".to_string(),
                ty: dummy_type(),
                span: dummy_span(),
            }],
            body: Box::new(Expr::Binary {
                left: Box::new(Expr::Ident {
                    name: "x".to_string(),
                    span: dummy_span(),
                }),
                op: BinOp::Mul,
                right: Box::new(Expr::Literal {
                    value: Literal::Int(2),
                    span: dummy_span(),
                }),
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        let code = backend.emit_expr(&expr, &mut ctx);
        assert!(code.is_ok());
        assert_eq!(code.ok().unwrap_or_default(), "lambda x: (x * 2)");
    }

    #[test]
    fn test_logical_operators() {
        let backend = PythonBackend::new();
        let mut ctx = EmitContext::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Bool(true),
                span: dummy_span(),
            }),
            op: BinOp::And,
            right: Box::new(Expr::Literal {
                value: Literal::Bool(false),
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        let code = backend.emit_expr(&expr, &mut ctx);
        assert!(code.is_ok());
        assert_eq!(code.ok().unwrap_or_default(), "(True and False)");
    }

    #[test]
    fn test_emit_simple_program() {
        let backend = PythonBackend::new();
        let program = parse("λ x = 42");
        let mut ctx = EmitContext::new();
        let code = backend.emit_program(&program, &mut ctx);
        assert!(code.is_ok());
        let result = code.ok().unwrap_or_default();
        assert!(result.contains("x = 42"));
    }
}
