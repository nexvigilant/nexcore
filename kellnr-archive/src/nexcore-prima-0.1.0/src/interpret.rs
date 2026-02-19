// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Interpreter
//!
//! Tree-walking interpreter with composition tracking.
//!
//! ## Mathematical Foundation
//!
//! The interpreter is → (Causality): AST → Value
//! Every operation tracks primitive compositions.
//!
//! ## Tier: T2-C (→ + ς + σ + Σ)

use crate::ast::*;
use crate::builtins::{builtins, call_builtin};
use crate::error::{PrimaError, PrimaResult};
use crate::token::Span;
use crate::value::{FnValue, Value, ValueData};
use std::collections::HashMap;

/// Environment: μ[String → Value]
#[derive(Debug, Clone, Default)]
pub struct Env {
    scopes: Vec<HashMap<String, Value>>,
}

impl Env {
    /// Create with builtins.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut env = Self {
            scopes: vec![builtins()],
        };
        env
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop scope.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable.
    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Get a variable.
    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }
}

/// Prima interpreter.
pub struct Interpreter {
    env: Env,
    /// Cumulative differential tracking (Entropy {-})
    total_entropy: f64,
}

impl Interpreter {
    /// Create new interpreter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            env: Env::with_builtins(),
            total_entropy: 0.0,
        }
    }

    /// Get current entropy value.
    pub fn entropy(&self) -> f64 {
        self.total_entropy
    }

    /// Evaluate a program.
    pub fn eval_program(&mut self, program: &Program) -> PrimaResult<Value> {
        let mut result = Value::void();
        for stmt in &program.statements {
            // Sync entropy storage for built-ins
            crate::builtins::set_entropy(self.total_entropy);
            result = self.eval_stmt(stmt)?;
        }
        Ok(result)
    }

    /// Evaluate a statement.
    pub fn eval_stmt(&mut self, stmt: &Stmt) -> PrimaResult<Value> {
        match stmt {
            Stmt::Let { name, value, .. } => self.eval_let(name, value),
            Stmt::FnDef {
                name, params, body, ..
            } => self.eval_fn_def(name, params, body),
            Stmt::TypeDef { .. } => Ok(Value::void()),
            Stmt::Expr { expr, .. } => self.eval_expr(expr),
            Stmt::Return { value, .. } => self.eval_return(value.as_ref()),
        }
    }

    fn eval_let(&mut self, name: &str, value: &Expr) -> PrimaResult<Value> {
        let v = self.eval_expr(value)?;
        self.env.define(name.to_string(), v);
        Ok(Value::void())
    }

    fn eval_fn_def(&mut self, name: &str, params: &[Param], body: &Block) -> PrimaResult<Value> {
        let closure: HashMap<String, Value> =
            self.env.scopes.iter().flat_map(|s| s.clone()).collect();
        let func = Value::function(name.to_string(), params.to_vec(), body.clone(), closure);
        self.env.define(name.to_string(), func);
        Ok(Value::void())
    }

    fn eval_return(&mut self, value: Option<&Expr>) -> PrimaResult<Value> {
        match value {
            Some(e) => self.eval_expr(e),
            None => Ok(Value::void()),
        }
    }

    /// Evaluate an expression.
    pub fn eval_expr(&mut self, expr: &Expr) -> PrimaResult<Value> {
        match expr {
            Expr::Literal { value, .. } => self.eval_literal(value),
            Expr::Ident { name, .. } => self.eval_ident(name),
            Expr::Binary {
                left, op, right, ..
            } => self.eval_binary(left, *op, right),
            Expr::Unary { op, operand, .. } => self.eval_unary(*op, operand),
            Expr::Call { func, args, .. } => self.eval_call(func, args),
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => self.eval_if(cond, then_branch, else_branch.as_ref()),
            Expr::Match {
                scrutinee, arms, ..
            } => self.eval_match(scrutinee, arms),
            Expr::For {
                var, iter, body, ..
            } => self.eval_for(var, iter, body),
            Expr::Block { block, .. } => self.eval_block(block),
            Expr::Lambda { params, body, .. } => self.eval_lambda(params, body),
            Expr::Sequence { elements, .. } => self.eval_sequence(elements),
            Expr::Mapping { pairs, .. } => self.eval_mapping(pairs),
            Expr::Member { object, field, .. } => self.eval_member(object, field),
            Expr::MethodCall {
                object,
                method,
                args,
                ..
            } => self.eval_method(object, method, args),
            // Quote: return AST as data (ρ primitive for homoiconicity)
            Expr::Quoted { expr, .. } => Ok(Value::quoted((**expr).clone())),
            // Quasiquote: return AST with selective unquote expansion
            Expr::Quasiquoted { expr, .. } => self.eval_quasiquote(expr),
            // Unquote/UnquoteSplice: error if used outside quasiquote
            Expr::Unquoted { .. } | Expr::UnquotedSplice { .. } => {
                Err(PrimaError::runtime("unquote outside quasiquote"))
            }
        }
    }

    /// Evaluate quasiquoted expression with selective unquoting.
    fn eval_quasiquote(&mut self, expr: &Expr) -> PrimaResult<Value> {
        match expr {
            Expr::Unquoted { expr: inner, .. } => self.eval_expr(inner),
            Expr::UnquotedSplice { expr: inner, .. } => self.eval_unquote_splice(inner),
            Expr::Sequence { elements, .. } => self.eval_quasiquote_seq(elements),
            _ => Ok(Value::quoted(self.quasiquote_transform(expr)?)),
        }
    }

    /// Evaluate unquote-splice (must return sequence).
    fn eval_unquote_splice(&mut self, inner: &Expr) -> PrimaResult<Value> {
        let val = self.eval_expr(inner)?;
        match &val.data {
            ValueData::Sequence(_) => Ok(val),
            _ => Err(PrimaError::runtime("unquote-splice requires sequence")),
        }
    }

    /// Evaluate quasiquote over sequence, flattening splices.
    fn eval_quasiquote_seq(&mut self, elements: &[Expr]) -> PrimaResult<Value> {
        let mut result = Vec::new();
        for elem in elements {
            self.process_quasiquote_element(elem, &mut result)?;
        }
        Ok(Value::sequence(result))
    }

    /// Process one element in quasiquote sequence.
    fn process_quasiquote_element(
        &mut self,
        elem: &Expr,
        result: &mut Vec<Value>,
    ) -> PrimaResult<()> {
        if let Expr::UnquotedSplice { expr: inner, .. } = elem {
            let val = self.eval_expr(inner)?;
            if let ValueData::Sequence(items) = val.data {
                result.extend(items);
            }
        } else {
            result.push(self.eval_quasiquote(elem)?);
        }
        Ok(())
    }

    /// Transform expression within quasiquote.
    fn quasiquote_transform(&mut self, expr: &Expr) -> PrimaResult<Expr> {
        if let Expr::Unquoted { expr: inner, span } = expr {
            let val = self.eval_expr(inner)?;
            return Ok(value_to_expr(&val, *span));
        }
        Ok(expr.clone())
    }

    fn eval_literal(&self, lit: &Literal) -> PrimaResult<Value> {
        Ok(match lit {
            Literal::Int(n) => Value::int(*n),
            Literal::Float(n) => Value::float(*n),
            Literal::String(s) => Value::string(s.clone()),
            Literal::Bool(b) => Value::bool(*b),
            Literal::Void => Value::void(),
            // Symbol: evaluates to itself (λ primitive)
            Literal::Symbol(s) => Value::symbol(s.clone()),
        })
    }

    fn eval_ident(&self, name: &str) -> PrimaResult<Value> {
        self.env
            .get(name)
            .cloned()
            .ok_or_else(|| PrimaError::undefined(name))
    }

    fn eval_binary(&mut self, left: &Expr, op: BinOp, right: &Expr) -> PrimaResult<Value> {
        let l = self.eval_expr(left)?;
        let r = self.eval_expr(right)?;
        self.apply_binary(l, op, r)
    }

    fn apply_binary(&mut self, l: Value, op: BinOp, r: Value) -> PrimaResult<Value> {
        match (&l.data, &r.data, op) {
            // Arithmetic on integers
            (ValueData::Int(a), ValueData::Int(b), BinOp::Add) => Ok(Value::int(a + b)),
            (ValueData::Int(a), ValueData::Int(b), BinOp::Sub) => {
                self.total_entropy += b.abs() as f64;
                crate::builtins::set_entropy(self.total_entropy);
                Ok(Value::int(a - b))
            }
            (ValueData::Int(a), ValueData::Int(b), BinOp::Mul) => Ok(Value::int(a * b)),
            (ValueData::Int(a), ValueData::Int(b), BinOp::Div) => {
                if *b == 0 {
                    return Err(PrimaError::DivisionByZero);
                }
                Ok(Value::int(a / b))
            }
            (ValueData::Int(a), ValueData::Int(b), BinOp::Mod) => Ok(Value::int(a % b)),

            // Arithmetic on floats
            (ValueData::Float(a), ValueData::Float(b), BinOp::Add) => Ok(Value::float(a + b)),
            (ValueData::Float(a), ValueData::Float(b), BinOp::Sub) => {
                self.total_entropy += b.abs();
                crate::builtins::set_entropy(self.total_entropy);
                Ok(Value::float(a - b))
            }
            (ValueData::Float(a), ValueData::Float(b), BinOp::Mul) => Ok(Value::float(a * b)),
            (ValueData::Float(a), ValueData::Float(b), BinOp::Div) => {
                if *b == 0.0 {
                    return Err(PrimaError::DivisionByZero);
                }
                Ok(Value::float(a / b))
            }

            // Mixed arithmetic
            (ValueData::Int(a), ValueData::Float(b), BinOp::Add) => Ok(Value::float(*a as f64 + b)),
            (ValueData::Float(a), ValueData::Int(b), BinOp::Add) => Ok(Value::float(a + *b as f64)),

            // Comparison
            (ValueData::Int(a), ValueData::Int(b), BinOp::Eq | BinOp::KappaEq) => {
                Ok(Value::bool(a == b))
            }
            (ValueData::Int(a), ValueData::Int(b), BinOp::Ne) => Ok(Value::bool(a != b)),
            (ValueData::Int(a), ValueData::Int(b), BinOp::Lt | BinOp::KappaLt) => {
                Ok(Value::bool(a < b))
            }
            (ValueData::Int(a), ValueData::Int(b), BinOp::Gt | BinOp::KappaGt) => {
                Ok(Value::bool(a > b))
            }
            (ValueData::Int(a), ValueData::Int(b), BinOp::Le) => Ok(Value::bool(a <= b)),
            (ValueData::Int(a), ValueData::Int(b), BinOp::Ge) => Ok(Value::bool(a >= b)),

            // Logical
            (ValueData::Bool(a), ValueData::Bool(b), BinOp::And) => Ok(Value::bool(*a && *b)),
            (ValueData::Bool(a), ValueData::Bool(b), BinOp::Or) => Ok(Value::bool(*a || *b)),

            // String concatenation
            (ValueData::String(a), ValueData::String(b), BinOp::Add) => {
                Ok(Value::string(format!("{}{}", a, b)))
            }

            _ => Err(PrimaError::runtime(format!("invalid binary op: {:?}", op))),
        }
    }

    fn eval_unary(&mut self, op: UnOp, operand: &Expr) -> PrimaResult<Value> {
        let v = self.eval_expr(operand)?;
        match (op, &v.data) {
            (UnOp::Neg, ValueData::Int(n)) => Ok(Value::int(-n)),
            (UnOp::Neg, ValueData::Float(n)) => Ok(Value::float(-n)),
            (UnOp::Not, ValueData::Bool(b)) => Ok(Value::bool(!b)),
            (UnOp::Not, _) => Ok(Value::bool(!v.is_truthy())),
            _ => Err(PrimaError::runtime("invalid unary op")),
        }
    }

    fn eval_call(&mut self, name: &str, args: &[Expr]) -> PrimaResult<Value> {
        // Special: eval(quoted) — execute quoted AST (→ Causality)
        if name == "eval" {
            return self.builtin_eval(args);
        }

        let func = self
            .env
            .get(name)
            .cloned()
            .ok_or_else(|| PrimaError::undefined(name))?;
        let arg_values: Vec<Value> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<PrimaResult<_>>()?;

        match &func.data {
            ValueData::Builtin(builtin_name) => call_builtin(builtin_name, &arg_values),
            ValueData::Function(fv) => self.call_function(fv, arg_values),
            _ => Err(PrimaError::runtime(format!("{} is not a function", name))),
        }
    }

    /// eval(quoted) — execute quoted AST (→ Causality primitive)
    /// Completes the homoiconicity stack for self-hosting.
    fn builtin_eval(&mut self, args: &[Expr]) -> PrimaResult<Value> {
        if args.len() != 1 {
            return Err(PrimaError::runtime("eval requires exactly 1 argument"));
        }
        let val = self.eval_expr(&args[0])?;
        match val.data {
            ValueData::Quoted(ast) => self.eval_expr(&ast),
            _ => Err(PrimaError::runtime("eval requires quoted expression")),
        }
    }

    fn call_function(&mut self, fv: &FnValue, args: Vec<Value>) -> PrimaResult<Value> {
        self.env.push_scope();
        for (param, arg) in fv.params.iter().zip(args) {
            self.env.define(param.name.clone(), arg);
        }
        for (name, value) in &fv.closure {
            if self.env.get(name).is_none() {
                self.env.define(name.clone(), value.clone());
            }
        }
        let result = self.eval_block(&fv.body)?;
        self.env.pop_scope();
        Ok(result)
    }

    fn eval_if(
        &mut self,
        cond: &Expr,
        then_branch: &Block,
        else_branch: Option<&Block>,
    ) -> PrimaResult<Value> {
        let c = self.eval_expr(cond)?;
        if c.is_truthy() {
            self.eval_block(then_branch)
        } else if let Some(eb) = else_branch {
            self.eval_block(eb)
        } else {
            Ok(Value::void())
        }
    }

    fn eval_match(&mut self, scrutinee: &Expr, arms: &[MatchArm]) -> PrimaResult<Value> {
        let value = self.eval_expr(scrutinee)?;
        for arm in arms {
            if self.pattern_matches(&arm.pattern, &value) {
                self.env.push_scope();
                self.bind_pattern(&arm.pattern, &value);
                let result = self.eval_expr(&arm.body)?;
                self.env.pop_scope();
                return Ok(result);
            }
        }
        Err(PrimaError::runtime("no matching pattern"))
    }

    fn pattern_matches(&self, pattern: &Pattern, value: &Value) -> bool {
        match pattern {
            Pattern::Wildcard { .. } => true,
            Pattern::Ident { .. } => true,
            Pattern::Literal { value: pv, .. } => {
                let pval = match pv {
                    Literal::Int(n) => Value::int(*n),
                    Literal::Float(n) => Value::float(*n),
                    Literal::String(s) => Value::string(s.clone()),
                    Literal::Bool(b) => Value::bool(*b),
                    Literal::Void => Value::void(),
                    Literal::Symbol(s) => Value::symbol(s.clone()),
                };
                pval == *value
            }
            Pattern::Constructor { .. } => false, // Not implemented
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, value: &Value) {
        if let Pattern::Ident { name, .. } = pattern {
            self.env.define(name.clone(), value.clone());
        }
    }

    fn eval_for(&mut self, var: &str, iter: &Expr, body: &Block) -> PrimaResult<Value> {
        let iterable = self.eval_expr(iter)?;
        match &iterable.data {
            ValueData::Sequence(elements) => {
                let mut last = Value::void();
                for elem in elements {
                    self.env.push_scope();
                    self.env.define(var.to_string(), elem.clone());
                    last = self.eval_block(body)?;
                    self.env.pop_scope();
                }
                Ok(last)
            }
            _ => Err(PrimaError::runtime("for requires a sequence")),
        }
    }

    fn eval_block(&mut self, block: &Block) -> PrimaResult<Value> {
        self.env.push_scope();
        for stmt in &block.statements {
            self.eval_stmt(stmt)?;
        }
        let result = match &block.expr {
            Some(e) => self.eval_expr(e)?,
            None => Value::void(),
        };
        self.env.pop_scope();
        Ok(result)
    }

    fn eval_lambda(&mut self, params: &[Param], body: &Expr) -> PrimaResult<Value> {
        let closure: HashMap<String, Value> =
            self.env.scopes.iter().flat_map(|s| s.clone()).collect();
        let block = Block {
            statements: vec![],
            expr: Some(Box::new(body.clone())),
            span: body.span(),
        };
        Ok(Value::function(
            "<lambda>".into(),
            params.to_vec(),
            block,
            closure,
        ))
    }

    fn eval_sequence(&mut self, elements: &[Expr]) -> PrimaResult<Value> {
        let values: Vec<Value> = elements
            .iter()
            .map(|e| self.eval_expr(e))
            .collect::<PrimaResult<_>>()?;
        Ok(Value::sequence(values))
    }

    fn eval_mapping(&mut self, pairs: &[(Expr, Expr)]) -> PrimaResult<Value> {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            let key = self.eval_expr(k)?;
            let key_str = match &key.data {
                ValueData::String(s) => s.clone(),
                ValueData::Int(n) => n.to_string(),
                _ => {
                    return Err(PrimaError::runtime(
                        "mapping keys must be strings or integers",
                    ));
                }
            };
            let value = self.eval_expr(v)?;
            map.insert(key_str, value);
        }
        Ok(Value::mapping(map))
    }

    fn eval_member(&mut self, object: &Expr, field: &str) -> PrimaResult<Value> {
        let obj = self.eval_expr(object)?;
        match &obj.data {
            ValueData::Mapping(m) => m
                .get(field)
                .cloned()
                .ok_or_else(|| PrimaError::undefined(field)),
            _ => Err(PrimaError::runtime("member access requires a mapping")),
        }
    }

    fn eval_method(&mut self, object: &Expr, method: &str, args: &[Expr]) -> PrimaResult<Value> {
        let obj = self.eval_expr(object)?;
        let arg_values: Vec<Value> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<PrimaResult<_>>()?;

        match method {
            "tier" => Ok(Value::string(obj.tier().code())),
            "composition" => Ok(Value::string(format!("{}", obj.composition))),
            "transfer" => Ok(Value::float(obj.transfer_confidence())),
            "len" => match &obj.data {
                ValueData::Sequence(v) => Ok(Value::int(v.len() as i64)),
                ValueData::String(s) => Ok(Value::int(s.len() as i64)),
                _ => Err(PrimaError::runtime("len requires sequence or string")),
            },
            "map" => self.method_map(obj, &arg_values),
            _ => Err(PrimaError::undefined(method)),
        }
    }

    fn method_map(&mut self, obj: Value, args: &[Value]) -> PrimaResult<Value> {
        match (&obj.data, args.first()) {
            (
                ValueData::Sequence(elements),
                Some(Value {
                    data: ValueData::Function(fv),
                    ..
                }),
            ) => {
                let mut results = Vec::new();
                for elem in elements {
                    let result = self.call_function(fv, vec![elem.clone()])?;
                    results.push(result);
                }
                Ok(Value::sequence(results))
            }
            _ => Err(PrimaError::runtime("map requires sequence and function")),
        }
    }
}

/// Convert a runtime value back to an AST expression (for quasiquote).
fn value_to_expr(value: &Value, span: Span) -> Expr {
    match &value.data {
        ValueData::Int(n) => literal_expr(Literal::Int(*n), span),
        ValueData::Float(n) => literal_expr(Literal::Float(*n), span),
        ValueData::Bool(b) => literal_expr(Literal::Bool(*b), span),
        ValueData::String(s) => literal_expr(Literal::String(s.clone()), span),
        ValueData::Void => literal_expr(Literal::Void, span),
        ValueData::Symbol(s) => literal_expr(Literal::Symbol(s.clone()), span),
        ValueData::Sequence(elems) => sequence_to_expr(elems, span),
        ValueData::Mapping(m) => mapping_to_expr(m, span),
        ValueData::Function(_) | ValueData::Builtin(_) => literal_expr(Literal::Void, span),
        ValueData::Quoted(e) => (**e).clone(),
    }
}

/// Helper: wrap literal in Expr.
fn literal_expr(lit: Literal, span: Span) -> Expr {
    Expr::Literal { value: lit, span }
}

/// Helper: convert sequence to Expr.
fn sequence_to_expr(elems: &[Value], span: Span) -> Expr {
    Expr::Sequence {
        elements: elems.iter().map(|e| value_to_expr(e, span)).collect(),
        span,
    }
}

/// Helper: convert mapping to Expr.
fn mapping_to_expr(m: &std::collections::HashMap<String, Value>, span: Span) -> Expr {
    Expr::Mapping {
        pairs: m
            .iter()
            .map(|(k, v)| {
                (
                    literal_expr(Literal::String(k.clone()), span),
                    value_to_expr(v, span),
                )
            })
            .collect(),
        span,
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn eval(src: &str) -> PrimaResult<Value> {
        let tokens = Lexer::new(src).tokenize()?;
        let program = Parser::new(tokens).parse()?;
        Interpreter::new().eval_program(&program)
    }

    #[test]
    fn test_literals() {
        assert_eq!(eval("42").unwrap(), Value::int(42));
        assert_eq!(eval("true").unwrap(), Value::bool(true));
        assert_eq!(eval("\"hello\"").unwrap(), Value::string("hello"));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("1 + 2").unwrap(), Value::int(3));
        assert_eq!(eval("5 - 3").unwrap(), Value::int(2));
        assert_eq!(eval("2 * 3").unwrap(), Value::int(6));
        assert_eq!(eval("10 / 2").unwrap(), Value::int(5));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval("1 κ< 2").unwrap(), Value::bool(true));
        assert_eq!(eval("2 κ> 1").unwrap(), Value::bool(true));
        assert_eq!(eval("1 κ= 1").unwrap(), Value::bool(true));
    }

    #[test]
    fn test_let() {
        assert_eq!(eval("let x = 42\nx").unwrap(), Value::int(42));
    }

    #[test]
    fn test_function() {
        let result = eval("fn add(x: N, y: N) → N { x + y }\nadd(1, 2)").unwrap();
        assert_eq!(result, Value::int(3));
    }

    #[test]
    fn test_if() {
        assert_eq!(eval("if true { 1 } else { 0 }").unwrap(), Value::int(1));
        assert_eq!(eval("if false { 1 } else { 0 }").unwrap(), Value::int(0));
    }

    #[test]
    fn test_sequence() {
        let result = eval("σ[1, 2, 3]").unwrap();
        if let ValueData::Sequence(v) = result.data {
            assert_eq!(v.len(), 3);
        } else {
            panic!("expected sequence");
        }
    }

    #[test]
    fn test_tier_method() {
        let result = eval("42.tier()").unwrap();
        assert_eq!(result, Value::string("T1"));
    }

    #[test]
    fn test_symbol_eval() {
        // Symbol: `:name` evaluates to itself (λ primitive)
        let result = eval(":foo").unwrap();
        if let ValueData::Symbol(name) = &result.data {
            assert_eq!(name, "foo");
        } else {
            panic!("expected symbol, got {:?}", result);
        }
    }

    #[test]
    fn test_symbol_is_truthy() {
        // Symbols are always truthy (they exist)
        let result = eval("if :sym { 1 } else { 0 }").unwrap();
        assert_eq!(result, Value::int(1));
    }

    #[test]
    fn test_quoted_expr() {
        // Quote: `'expr` returns AST as data (ρ primitive)
        let result = eval("'42").unwrap();
        if let ValueData::Quoted(_) = &result.data {
            // Success — we got a quoted AST node
        } else {
            panic!("expected quoted, got {:?}", result);
        }
    }

    #[test]
    fn test_quoted_is_truthy() {
        // Quoted expressions are truthy (they hold data)
        let result = eval("if '42 { 1 } else { 0 }").unwrap();
        assert_eq!(result, Value::int(1));
    }

    #[test]
    fn test_quasiquote_simple() {
        // Quasiquote: `` `expr `` returns quoted data like quote
        let result = eval("`42").ok();
        assert!(result.is_some(), "quasiquote should parse and eval");
    }

    #[test]
    fn test_unquote_outside_quasiquote_errors() {
        // Unquote outside quasiquote is an error
        let result = eval("~42");
        assert!(result.is_err(), "unquote outside quasiquote should error");
    }

    #[test]
    fn test_quasiquote_with_unquote() {
        // `` `~42 `` — unquote inside quasiquote evaluates
        // The ~42 inside ` evaluates to 42
        let result = eval("`~42").ok();
        assert!(result.is_some(), "quasiquote with unquote should work");
    }

    #[test]
    fn test_eval_quoted_literal() {
        // eval('42) → 42 (execute quoted AST)
        let result = eval("eval('42)").ok();
        assert!(result.is_some(), "eval should work on quoted literal");
        if let Some(v) = result {
            assert_eq!(v, Value::int(42));
        }
    }

    #[test]
    fn test_eval_quoted_expression() {
        // eval('(1 + 2)) → 3 (execute quoted arithmetic)
        let result = eval("eval('(1 + 2))").ok();
        assert!(result.is_some(), "eval should work on quoted expression");
        if let Some(v) = result {
            assert_eq!(v, Value::int(3));
        }
    }

    #[test]
    fn test_eval_non_quoted_errors() {
        // eval(42) → error (not quoted)
        let result = eval("eval(42)");
        assert!(result.is_err(), "eval on non-quoted should error");
    }
}
