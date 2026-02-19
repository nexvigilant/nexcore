//! JSON AST Import/Export for the nexcore-dna language.
//!
//! Enables AI agents to generate programs as structured JSON (tool_use output)
//! rather than raw text, eliminating parse ambiguity.
//!
//! ## JSON Schema
//!
//! ```json
//! {"type": "program", "body": [
//!   {"type": "let", "name": "x", "value": {"type": "lit", "value": 5}},
//!   {"type": "expr", "value": {"type": "binop", "op": "+",
//!     "left": {"type": "var", "name": "x"},
//!     "right": {"type": "lit", "value": 3}}}
//! ]}
//! ```
//!
//! ## Bidirectional
//!
//! - `ast_to_json(&[Stmt]) -> String` — Export AST as JSON
//! - `json_to_ast(&str) -> Result<Vec<Stmt>>` — Import JSON to AST
//! - `source_to_json(&str) -> Result<String>` — Parse source, emit JSON
//! - `json_to_program(&str) -> Result<Program>` — JSON → compile → Program
//!
//! Tier: T3 (μ Mapping + σ Sequence + ∂ Boundary + κ Comparison + → Causality + ∃ Existence)

use crate::error::{DnaError, Result};
use crate::lang::ast::{BinOp, Expr, Stmt};
use crate::lang::parser;
use crate::program::Program;

// ============================================================================
// Minimal JSON Value Type (zero external deps)
// ============================================================================

/// A minimal JSON value for AST serialization.
///
/// Tier: T2-C (Σ Sum + μ Mapping + σ Sequence + ∃ Existence)
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// JSON null
    Null,
    /// JSON boolean
    Bool(bool),
    /// JSON integer (i64, no floats needed for AST)
    Int(i64),
    /// JSON string
    Str(String),
    /// JSON array: ordered sequence
    Array(Vec<JsonValue>),
    /// JSON object: key-value pairs (ordered, not hashed — zero deps)
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Get a field from a JSON object by key.
    fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Get a string field or return an error.
    fn get_str(&self, key: &str) -> Result<&str> {
        match self.get(key) {
            Some(JsonValue::Str(s)) => Ok(s.as_str()),
            _ => Err(DnaError::SyntaxError(
                0,
                format!("expected string field '{key}'"),
            )),
        }
    }

    /// Get an integer field or return an error.
    fn get_int(&self, key: &str) -> Result<i64> {
        match self.get(key) {
            Some(JsonValue::Int(n)) => Ok(*n),
            _ => Err(DnaError::SyntaxError(
                0,
                format!("expected integer field '{key}'"),
            )),
        }
    }

    /// Get an array field or return an error.
    fn get_array(&self, key: &str) -> Result<&[JsonValue]> {
        match self.get(key) {
            Some(JsonValue::Array(arr)) => Ok(arr.as_slice()),
            _ => Err(DnaError::SyntaxError(
                0,
                format!("expected array field '{key}'"),
            )),
        }
    }
}

// ============================================================================
// JSON Emitter: JsonValue → String
// ============================================================================

impl core::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            JsonValue::Int(n) => write!(f, "{n}"),
            JsonValue::Str(s) => {
                write!(f, "\"")?;
                for ch in s.chars() {
                    match ch {
                        '"' => write!(f, "\\\"")?,
                        '\\' => write!(f, "\\\\")?,
                        '\n' => write!(f, "\\n")?,
                        '\r' => write!(f, "\\r")?,
                        '\t' => write!(f, "\\t")?,
                        c => write!(f, "{c}")?,
                    }
                }
                write!(f, "\"")
            }
            JsonValue::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            JsonValue::Object(pairs) => {
                write!(f, "{{")?;
                for (i, (key, val)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{key}\":{val}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

// ============================================================================
// JSON Parser: String → JsonValue (recursive descent, zero deps)
// ============================================================================

struct JsonParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, ch: u8) -> Result<()> {
        self.skip_whitespace();
        match self.advance() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(DnaError::SyntaxError(
                0,
                format!("expected '{}', found '{}'", ch as char, c as char),
            )),
            None => Err(DnaError::SyntaxError(
                0,
                format!("expected '{}', found EOF", ch as char),
            )),
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'"') => self.parse_string().map(JsonValue::Str),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') => self.parse_keyword(b"true", JsonValue::Bool(true)),
            Some(b'f') => self.parse_keyword(b"false", JsonValue::Bool(false)),
            Some(b'n') => self.parse_keyword(b"null", JsonValue::Null),
            Some(ch) if ch == b'-' || ch.is_ascii_digit() => self.parse_number(),
            Some(ch) => Err(DnaError::SyntaxError(
                0,
                format!("unexpected character '{}' in JSON", ch as char),
            )),
            None => Err(DnaError::SyntaxError(0, "unexpected EOF in JSON".into())),
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect(b'"')?;
        let mut s = String::new();
        loop {
            match self.advance() {
                Some(b'"') => return Ok(s),
                Some(b'\\') => match self.advance() {
                    Some(b'"') => s.push('"'),
                    Some(b'\\') => s.push('\\'),
                    Some(b'n') => s.push('\n'),
                    Some(b'r') => s.push('\r'),
                    Some(b't') => s.push('\t'),
                    Some(b'/') => s.push('/'),
                    Some(b'u') => {
                        // Parse 4-hex-digit unicode escape
                        let mut hex = String::with_capacity(4);
                        for _ in 0..4 {
                            match self.advance() {
                                Some(h) => hex.push(h as char),
                                None => {
                                    return Err(DnaError::SyntaxError(
                                        0,
                                        "unexpected EOF in unicode escape".into(),
                                    ));
                                }
                            }
                        }
                        let code = u32::from_str_radix(&hex, 16).map_err(|_| {
                            DnaError::SyntaxError(0, format!("invalid unicode escape: \\u{hex}"))
                        })?;
                        if let Some(ch) = char::from_u32(code) {
                            s.push(ch);
                        }
                    }
                    Some(c) => {
                        return Err(DnaError::SyntaxError(
                            0,
                            format!("invalid escape: \\{}", c as char),
                        ));
                    }
                    None => {
                        return Err(DnaError::SyntaxError(0, "unexpected EOF in string".into()));
                    }
                },
                Some(ch) => s.push(ch as char),
                None => return Err(DnaError::SyntaxError(0, "unterminated string".into())),
            }
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str = core::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| DnaError::SyntaxError(0, "invalid UTF-8 in number".into()))?;
        let n: i64 = num_str
            .parse()
            .map_err(|_| DnaError::InvalidLiteral(num_str.to_string()))?;
        Ok(JsonValue::Int(n))
    }

    fn parse_keyword(&mut self, keyword: &[u8], value: JsonValue) -> Result<JsonValue> {
        let end = self.pos + keyword.len();
        if end <= self.input.len() && &self.input[self.pos..end] == keyword {
            self.pos = end;
            Ok(value)
        } else {
            Err(DnaError::SyntaxError(
                0,
                format!(
                    "expected '{}'",
                    core::str::from_utf8(keyword).unwrap_or("?")
                ),
            ))
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => {
                    return Err(DnaError::SyntaxError(
                        0,
                        "expected ',' or ']' in array".into(),
                    ));
                }
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.expect(b'{')?;
        let mut pairs = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(pairs));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            let val = self.parse_value()?;
            pairs.push((key, val));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(pairs));
                }
                _ => {
                    return Err(DnaError::SyntaxError(
                        0,
                        "expected ',' or '}}' in object".into(),
                    ));
                }
            }
        }
    }
}

/// Parse a JSON string into a JsonValue.
pub fn parse_json(input: &str) -> Result<JsonValue> {
    let mut parser = JsonParser::new(input);
    let val = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return Err(DnaError::SyntaxError(
            0,
            "trailing characters after JSON value".into(),
        ));
    }
    Ok(val)
}

// ============================================================================
// AST → JSON (Export)
// ============================================================================

/// Convert an operator to its JSON string representation.
fn binop_to_str(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Eq => "==",
        BinOp::Neq => "!=",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::Le => "<=",
        BinOp::Ge => ">=",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::And => "and",
        BinOp::Or => "or",
    }
}

/// Parse a BinOp from its JSON string representation.
fn str_to_binop(s: &str) -> Result<BinOp> {
    match s {
        "+" => Ok(BinOp::Add),
        "-" => Ok(BinOp::Sub),
        "*" => Ok(BinOp::Mul),
        "/" => Ok(BinOp::Div),
        "%" => Ok(BinOp::Mod),
        "==" => Ok(BinOp::Eq),
        "!=" => Ok(BinOp::Neq),
        "<" => Ok(BinOp::Lt),
        ">" => Ok(BinOp::Gt),
        "<=" => Ok(BinOp::Le),
        ">=" => Ok(BinOp::Ge),
        "&" => Ok(BinOp::BitAnd),
        "|" => Ok(BinOp::BitOr),
        "^" => Ok(BinOp::BitXor),
        "<<" => Ok(BinOp::Shl),
        ">>" => Ok(BinOp::Shr),
        "and" => Ok(BinOp::And),
        "or" => Ok(BinOp::Or),
        _ => Err(DnaError::SyntaxError(0, format!("unknown operator: '{s}'"))),
    }
}

/// Convert an expression AST node to a JsonValue.
fn expr_to_json(expr: &Expr) -> JsonValue {
    match expr {
        Expr::Lit(n) => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("lit".into())),
            ("value".into(), JsonValue::Int(*n)),
        ]),
        Expr::Var(name) => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("var".into())),
            ("name".into(), JsonValue::Str(name.clone())),
        ]),
        Expr::Neg(inner) => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("neg".into())),
            ("expr".into(), expr_to_json(inner)),
        ]),
        Expr::Not(inner) => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("not".into())),
            ("expr".into(), expr_to_json(inner)),
        ]),
        Expr::BitNot(inner) => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("bitnot".into())),
            ("expr".into(), expr_to_json(inner)),
        ]),
        Expr::BinOp { left, op, right } => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("binop".into())),
            ("op".into(), JsonValue::Str(binop_to_str(op).into())),
            ("left".into(), expr_to_json(left)),
            ("right".into(), expr_to_json(right)),
        ]),
        Expr::Call { name, args } => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("call".into())),
            ("name".into(), JsonValue::Str(name.clone())),
            (
                "args".into(),
                JsonValue::Array(args.iter().map(expr_to_json).collect()),
            ),
        ]),
    }
}

/// Convert a statement AST node to a JsonValue.
fn stmt_to_json(stmt: &Stmt) -> JsonValue {
    match stmt {
        Stmt::ExprStmt(expr) => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("expr".into())),
            ("value".into(), expr_to_json(expr)),
        ]),
        Stmt::Let { name, value } => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("let".into())),
            ("name".into(), JsonValue::Str(name.clone())),
            ("value".into(), expr_to_json(value)),
        ]),
        Stmt::Assign { name, value } => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("assign".into())),
            ("name".into(), JsonValue::Str(name.clone())),
            ("value".into(), expr_to_json(value)),
        ]),
        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let mut pairs = vec![
                ("type".into(), JsonValue::Str("if".into())),
                ("cond".into(), expr_to_json(cond)),
                (
                    "then".into(),
                    JsonValue::Array(then_body.iter().map(stmt_to_json).collect()),
                ),
            ];
            if !else_body.is_empty() {
                pairs.push((
                    "else".into(),
                    JsonValue::Array(else_body.iter().map(stmt_to_json).collect()),
                ));
            }
            JsonValue::Object(pairs)
        }
        Stmt::While { cond, body } => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("while".into())),
            ("cond".into(), expr_to_json(cond)),
            (
                "body".into(),
                JsonValue::Array(body.iter().map(stmt_to_json).collect()),
            ),
        ]),
        Stmt::FnDef { name, params, body } => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("fn".into())),
            ("name".into(), JsonValue::Str(name.clone())),
            (
                "params".into(),
                JsonValue::Array(params.iter().map(|p| JsonValue::Str(p.clone())).collect()),
            ),
            (
                "body".into(),
                JsonValue::Array(body.iter().map(stmt_to_json).collect()),
            ),
        ]),
        Stmt::Return(expr) => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("return".into())),
            ("value".into(), expr_to_json(expr)),
        ]),
        Stmt::For {
            var,
            start,
            end,
            body,
        } => JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("for".into())),
            ("var".into(), JsonValue::Str(var.clone())),
            ("start".into(), expr_to_json(start)),
            ("end".into(), expr_to_json(end)),
            (
                "body".into(),
                JsonValue::Array(body.iter().map(stmt_to_json).collect()),
            ),
        ]),
    }
}

/// Convert a program (list of statements) to a JSON program value.
fn program_to_json(stmts: &[Stmt]) -> JsonValue {
    JsonValue::Object(vec![
        ("type".into(), JsonValue::Str("program".into())),
        (
            "body".into(),
            JsonValue::Array(stmts.iter().map(stmt_to_json).collect()),
        ),
    ])
}

// ============================================================================
// JSON → AST (Import)
// ============================================================================

/// Reconstruct an Expr from a JsonValue.
fn json_to_expr(val: &JsonValue) -> Result<Expr> {
    let type_str = val.get_str("type")?;
    match type_str {
        "lit" => {
            let n = val.get_int("value")?;
            Ok(Expr::Lit(n))
        }
        "var" => {
            let name = val.get_str("name")?;
            Ok(Expr::Var(name.to_string()))
        }
        "neg" => {
            let inner = val
                .get("expr")
                .ok_or_else(|| DnaError::SyntaxError(0, "neg: missing 'expr'".into()))?;
            Ok(Expr::Neg(Box::new(json_to_expr(inner)?)))
        }
        "not" => {
            let inner = val
                .get("expr")
                .ok_or_else(|| DnaError::SyntaxError(0, "not: missing 'expr'".into()))?;
            Ok(Expr::Not(Box::new(json_to_expr(inner)?)))
        }
        "bitnot" => {
            let inner = val
                .get("expr")
                .ok_or_else(|| DnaError::SyntaxError(0, "bitnot: missing 'expr'".into()))?;
            Ok(Expr::BitNot(Box::new(json_to_expr(inner)?)))
        }
        "binop" => {
            let op_str = val.get_str("op")?;
            let op = str_to_binop(op_str)?;
            let left = val
                .get("left")
                .ok_or_else(|| DnaError::SyntaxError(0, "binop: missing 'left'".into()))?;
            let right = val
                .get("right")
                .ok_or_else(|| DnaError::SyntaxError(0, "binop: missing 'right'".into()))?;
            Ok(Expr::BinOp {
                left: Box::new(json_to_expr(left)?),
                op,
                right: Box::new(json_to_expr(right)?),
            })
        }
        "call" => {
            let name = val.get_str("name")?.to_string();
            let args_arr = val.get_array("args")?;
            let args: Result<Vec<Expr>> = args_arr.iter().map(json_to_expr).collect();
            Ok(Expr::Call { name, args: args? })
        }
        _ => Err(DnaError::SyntaxError(
            0,
            format!("unknown expression type: '{type_str}'"),
        )),
    }
}

/// Reconstruct a Stmt from a JsonValue.
fn json_to_stmt(val: &JsonValue) -> Result<Stmt> {
    let type_str = val.get_str("type")?;
    match type_str {
        "expr" => {
            let expr_val = val
                .get("value")
                .ok_or_else(|| DnaError::SyntaxError(0, "expr: missing 'value'".into()))?;
            Ok(Stmt::ExprStmt(json_to_expr(expr_val)?))
        }
        "let" => {
            let name = val.get_str("name")?.to_string();
            let expr_val = val
                .get("value")
                .ok_or_else(|| DnaError::SyntaxError(0, "let: missing 'value'".into()))?;
            Ok(Stmt::Let {
                name,
                value: json_to_expr(expr_val)?,
            })
        }
        "assign" => {
            let name = val.get_str("name")?.to_string();
            let expr_val = val
                .get("value")
                .ok_or_else(|| DnaError::SyntaxError(0, "assign: missing 'value'".into()))?;
            Ok(Stmt::Assign {
                name,
                value: json_to_expr(expr_val)?,
            })
        }
        "if" => {
            let cond_val = val
                .get("cond")
                .ok_or_else(|| DnaError::SyntaxError(0, "if: missing 'cond'".into()))?;
            let then_arr = val.get_array("then")?;
            let then_body: Result<Vec<Stmt>> = then_arr.iter().map(json_to_stmt).collect();
            let else_body = match val.get_array("else") {
                Ok(arr) => {
                    let stmts: Result<Vec<Stmt>> = arr.iter().map(json_to_stmt).collect();
                    stmts?
                }
                Err(_) => Vec::new(),
            };
            Ok(Stmt::If {
                cond: json_to_expr(cond_val)?,
                then_body: then_body?,
                else_body,
            })
        }
        "while" => {
            let cond_val = val
                .get("cond")
                .ok_or_else(|| DnaError::SyntaxError(0, "while: missing 'cond'".into()))?;
            let body_arr = val.get_array("body")?;
            let body: Result<Vec<Stmt>> = body_arr.iter().map(json_to_stmt).collect();
            Ok(Stmt::While {
                cond: json_to_expr(cond_val)?,
                body: body?,
            })
        }
        "fn" => {
            let name = val.get_str("name")?.to_string();
            let params_arr = val.get_array("params")?;
            let params: Result<Vec<String>> = params_arr
                .iter()
                .map(|v| match v {
                    JsonValue::Str(s) => Ok(s.clone()),
                    _ => Err(DnaError::SyntaxError(
                        0,
                        "fn: params must be strings".into(),
                    )),
                })
                .collect();
            let body_arr = val.get_array("body")?;
            let body: Result<Vec<Stmt>> = body_arr.iter().map(json_to_stmt).collect();
            Ok(Stmt::FnDef {
                name,
                params: params?,
                body: body?,
            })
        }
        "return" => {
            let expr_val = val
                .get("value")
                .ok_or_else(|| DnaError::SyntaxError(0, "return: missing 'value'".into()))?;
            Ok(Stmt::Return(json_to_expr(expr_val)?))
        }
        "for" => {
            let var = val.get_str("var")?.to_string();
            let start_val = val
                .get("start")
                .ok_or_else(|| DnaError::SyntaxError(0, "for: missing 'start'".into()))?;
            let end_val = val
                .get("end")
                .ok_or_else(|| DnaError::SyntaxError(0, "for: missing 'end'".into()))?;
            let body_arr = val.get_array("body")?;
            let body: Result<Vec<Stmt>> = body_arr.iter().map(json_to_stmt).collect();
            Ok(Stmt::For {
                var,
                start: json_to_expr(start_val)?,
                end: json_to_expr(end_val)?,
                body: body?,
            })
        }
        _ => Err(DnaError::SyntaxError(
            0,
            format!("unknown statement type: '{type_str}'"),
        )),
    }
}

/// Reconstruct a program (Vec<Stmt>) from a JsonValue.
fn json_program_to_stmts(val: &JsonValue) -> Result<Vec<Stmt>> {
    let type_str = val.get_str("type")?;
    if type_str != "program" {
        return Err(DnaError::SyntaxError(
            0,
            format!("expected type 'program', found '{type_str}'"),
        ));
    }
    let body = val.get_array("body")?;
    body.iter().map(json_to_stmt).collect()
}

// ============================================================================
// Public API
// ============================================================================

/// Export a list of statements as a JSON string.
///
/// ```text
/// {"type":"program","body":[...]}
/// ```
pub fn ast_to_json(stmts: &[Stmt]) -> String {
    let json = program_to_json(stmts);
    format!("{json}")
}

/// Import a JSON string to a list of AST statements.
///
/// Accepts the `{"type":"program","body":[...]}` format.
pub fn json_to_ast(json_str: &str) -> Result<Vec<Stmt>> {
    let val = parse_json(json_str)?;
    json_program_to_stmts(&val)
}

/// Parse source text and return JSON AST.
///
/// Convenience: source → parser → AST → JSON string.
pub fn source_to_json(source: &str) -> Result<String> {
    let stmts = parser::parse(source)?;
    Ok(ast_to_json(&stmts))
}

/// Import JSON AST and compile to a Program.
///
/// Convenience: JSON string → AST → optimizer → codegen → Program.
pub fn json_to_program(json_str: &str) -> Result<Program> {
    let stmts = json_to_ast(json_str)?;
    // Reconstruct source from AST by compiling directly
    // We go through the optimizer + codegen path
    let optimized = crate::lang::optimizer::optimize(&stmts);
    crate::lang::codegen::compile_stmts(&optimized)
}

/// Import JSON AST, compile, and execute.
///
/// Convenience: JSON string → AST → compile → VM → result.
pub fn json_eval(json_str: &str) -> Result<crate::vm::VmResult> {
    let program = json_to_program(json_str)?;
    program.run()
}

// ============================================================================
// Pretty Printer (indented JSON)
// ============================================================================

/// Emit indented JSON for human readability.
pub fn ast_to_json_pretty(stmts: &[Stmt]) -> String {
    let json = program_to_json(stmts);
    let mut out = String::new();
    pretty_print(&json, &mut out, 0);
    out
}

fn pretty_print(val: &JsonValue, out: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    let pad_inner = "  ".repeat(indent + 1);
    match val {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JsonValue::Int(n) => {
            let s = format!("{n}");
            out.push_str(&s);
        }
        JsonValue::Str(s) => {
            out.push('"');
            for ch in s.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        JsonValue::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&pad_inner);
                pretty_print(item, out, indent + 1);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push(']');
        }
        JsonValue::Object(pairs) => {
            if pairs.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (i, (key, value)) in pairs.iter().enumerate() {
                out.push_str(&pad_inner);
                out.push('"');
                out.push_str(key);
                out.push_str("\": ");
                pretty_print(value, out, indent + 1);
                if i + 1 < pairs.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push('}');
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- JSON parser tests ---

    #[test]
    fn parse_json_null() {
        let val = parse_json("null");
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(v, JsonValue::Null);
        }
    }

    #[test]
    fn parse_json_bool() {
        let t = parse_json("true");
        assert!(t.is_ok());
        if let Ok(v) = t {
            assert_eq!(v, JsonValue::Bool(true));
        }
        let f = parse_json("false");
        assert!(f.is_ok());
        if let Ok(v) = f {
            assert_eq!(v, JsonValue::Bool(false));
        }
    }

    #[test]
    fn parse_json_int() {
        let val = parse_json("42");
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(v, JsonValue::Int(42));
        }
        let neg = parse_json("-7");
        assert!(neg.is_ok());
        if let Ok(v) = neg {
            assert_eq!(v, JsonValue::Int(-7));
        }
    }

    #[test]
    fn parse_json_string() {
        let val = parse_json("\"hello world\"");
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(v, JsonValue::Str("hello world".into()));
        }
    }

    #[test]
    fn parse_json_string_escapes() {
        let val = parse_json(r#""line\none\ttab\\slash\"quote""#);
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(v, JsonValue::Str("line\none\ttab\\slash\"quote".into()));
        }
    }

    #[test]
    fn parse_json_array() {
        let val = parse_json("[1, 2, 3]");
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(
                v,
                JsonValue::Array(vec![
                    JsonValue::Int(1),
                    JsonValue::Int(2),
                    JsonValue::Int(3)
                ])
            );
        }
    }

    #[test]
    fn parse_json_empty_array() {
        let val = parse_json("[]");
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(v, JsonValue::Array(vec![]));
        }
    }

    #[test]
    fn parse_json_object() {
        let val = parse_json(r#"{"name": "alice", "age": 30}"#);
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(
                v,
                JsonValue::Object(vec![
                    ("name".into(), JsonValue::Str("alice".into())),
                    ("age".into(), JsonValue::Int(30)),
                ])
            );
        }
    }

    #[test]
    fn parse_json_empty_object() {
        let val = parse_json("{}");
        assert!(val.is_ok());
        if let Ok(v) = val {
            assert_eq!(v, JsonValue::Object(vec![]));
        }
    }

    #[test]
    fn parse_json_nested() {
        let val = parse_json(r#"{"a": [1, {"b": 2}]}"#);
        assert!(val.is_ok());
        if let Ok(v) = val {
            let expected = JsonValue::Object(vec![(
                "a".into(),
                JsonValue::Array(vec![
                    JsonValue::Int(1),
                    JsonValue::Object(vec![("b".into(), JsonValue::Int(2))]),
                ]),
            )]);
            assert_eq!(v, expected);
        }
    }

    #[test]
    fn parse_json_error_trailing() {
        let result = parse_json("42 garbage");
        assert!(result.is_err());
    }

    #[test]
    fn parse_json_error_unterminated_string() {
        let result = parse_json("\"hello");
        assert!(result.is_err());
    }

    // --- JSON Display roundtrip ---

    #[test]
    fn json_display_roundtrip() {
        let val = JsonValue::Object(vec![
            ("type".into(), JsonValue::Str("lit".into())),
            ("value".into(), JsonValue::Int(42)),
        ]);
        let s = format!("{val}");
        let parsed = parse_json(&s);
        assert!(parsed.is_ok());
        if let Ok(v) = parsed {
            assert_eq!(v, val);
        }
    }

    // --- AST → JSON → AST roundtrip tests ---

    #[test]
    fn roundtrip_literal() {
        let stmts = vec![Stmt::ExprStmt(Expr::Lit(42))];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_variable() {
        let stmts = vec![Stmt::ExprStmt(Expr::Var("x".into()))];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_binop() {
        let stmts = vec![Stmt::ExprStmt(Expr::BinOp {
            left: Box::new(Expr::Lit(2)),
            op: BinOp::Add,
            right: Box::new(Expr::Lit(3)),
        })];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_all_binops() {
        let ops = [
            BinOp::Add,
            BinOp::Sub,
            BinOp::Mul,
            BinOp::Div,
            BinOp::Mod,
            BinOp::Eq,
            BinOp::Neq,
            BinOp::Lt,
            BinOp::Gt,
            BinOp::Le,
            BinOp::Ge,
            BinOp::BitAnd,
            BinOp::BitOr,
            BinOp::BitXor,
            BinOp::Shl,
            BinOp::Shr,
            BinOp::And,
            BinOp::Or,
        ];
        for op in ops {
            let stmts = vec![Stmt::ExprStmt(Expr::BinOp {
                left: Box::new(Expr::Lit(1)),
                op,
                right: Box::new(Expr::Lit(2)),
            })];
            let json = ast_to_json(&stmts);
            let result = json_to_ast(&json);
            assert!(result.is_ok(), "failed roundtrip for op: {op}");
            if let Ok(round) = result {
                assert_eq!(round, stmts, "mismatch for op: {op}");
            }
        }
    }

    #[test]
    fn roundtrip_neg() {
        let stmts = vec![Stmt::ExprStmt(Expr::Neg(Box::new(Expr::Lit(5))))];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_not() {
        let stmts = vec![Stmt::ExprStmt(Expr::Not(Box::new(Expr::Var(
            "flag".into(),
        ))))];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_bitnot() {
        let stmts = vec![Stmt::ExprStmt(Expr::BitNot(Box::new(Expr::Lit(0))))];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_call() {
        let stmts = vec![Stmt::ExprStmt(Expr::Call {
            name: "abs".into(),
            args: vec![Expr::Lit(-5)],
        })];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_let() {
        let stmts = vec![Stmt::Let {
            name: "x".into(),
            value: Expr::Lit(10),
        }];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_assign() {
        let stmts = vec![Stmt::Assign {
            name: "x".into(),
            value: Expr::Lit(20),
        }];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_if_then() {
        let stmts = vec![Stmt::If {
            cond: Expr::BinOp {
                left: Box::new(Expr::Var("x".into())),
                op: BinOp::Gt,
                right: Box::new(Expr::Lit(0)),
            },
            then_body: vec![Stmt::ExprStmt(Expr::Lit(42))],
            else_body: vec![],
        }];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_if_else() {
        let stmts = vec![Stmt::If {
            cond: Expr::Lit(1),
            then_body: vec![Stmt::ExprStmt(Expr::Lit(1))],
            else_body: vec![Stmt::ExprStmt(Expr::Lit(2))],
        }];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_while() {
        let stmts = vec![Stmt::While {
            cond: Expr::BinOp {
                left: Box::new(Expr::Var("n".into())),
                op: BinOp::Gt,
                right: Box::new(Expr::Lit(0)),
            },
            body: vec![Stmt::Assign {
                name: "n".into(),
                value: Expr::BinOp {
                    left: Box::new(Expr::Var("n".into())),
                    op: BinOp::Sub,
                    right: Box::new(Expr::Lit(1)),
                },
            }],
        }];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_fn() {
        let stmts = vec![Stmt::FnDef {
            name: "double".into(),
            params: vec!["x".into()],
            body: vec![Stmt::Return(Expr::BinOp {
                left: Box::new(Expr::Var("x".into())),
                op: BinOp::Mul,
                right: Box::new(Expr::Lit(2)),
            })],
        }];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_return() {
        let stmts = vec![Stmt::Return(Expr::Lit(0))];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    #[test]
    fn roundtrip_for() {
        let stmts = vec![Stmt::For {
            var: "i".into(),
            start: Expr::Lit(1),
            end: Expr::Lit(10),
            body: vec![Stmt::ExprStmt(Expr::Var("i".into()))],
        }];
        let json = ast_to_json(&stmts);
        let result = json_to_ast(&json);
        assert!(result.is_ok());
        if let Ok(round) = result {
            assert_eq!(round, stmts);
        }
    }

    // --- Integration: source → JSON → AST → compile → run ---

    #[test]
    fn source_to_json_roundtrip() {
        let source = "2 + 3 * 4";
        let json = source_to_json(source);
        assert!(json.is_ok());
        if let Ok(json_str) = json {
            assert!(json_str.contains("\"type\":\"program\""));
            assert!(json_str.contains("\"type\":\"binop\""));
        }
    }

    #[test]
    fn json_to_program_eval() {
        let json = r#"{"type":"program","body":[{"type":"expr","value":{"type":"binop","op":"+","left":{"type":"lit","value":2},"right":{"type":"lit","value":3}}}]}"#;
        let result = json_eval(json);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output, vec![5]);
        }
    }

    #[test]
    fn json_eval_with_variables() {
        let json = r#"{"type":"program","body":[
            {"type":"let","name":"x","value":{"type":"lit","value":10}},
            {"type":"let","name":"y","value":{"type":"lit","value":20}},
            {"type":"expr","value":{"type":"binop","op":"+",
                "left":{"type":"var","name":"x"},
                "right":{"type":"var","name":"y"}}}
        ]}"#;
        let result = json_eval(json);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output, vec![30]);
        }
    }

    #[test]
    fn json_eval_function() {
        let json = r#"{"type":"program","body":[
            {"type":"fn","name":"triple","params":["x"],"body":[
                {"type":"return","value":{"type":"binop","op":"*",
                    "left":{"type":"var","name":"x"},
                    "right":{"type":"lit","value":3}}}
            ]},
            {"type":"expr","value":{"type":"call","name":"triple","args":[
                {"type":"lit","value":14}
            ]}}
        ]}"#;
        let result = json_eval(json);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output, vec![42]);
        }
    }

    // --- Pretty printer ---

    #[test]
    fn pretty_print_basic() {
        let stmts = vec![Stmt::ExprStmt(Expr::Lit(42))];
        let pretty = ast_to_json_pretty(&stmts);
        assert!(pretty.contains("  "));
        assert!(pretty.contains("\"type\": \"program\""));
        // Should parse back
        let reparsed = json_to_ast(&pretty);
        assert!(reparsed.is_ok());
        if let Ok(round) = reparsed {
            assert_eq!(round, stmts);
        }
    }

    // --- Error handling ---

    #[test]
    fn json_error_wrong_root_type() {
        let json = r#"{"type":"not_program","body":[]}"#;
        let result = json_to_ast(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_error_unknown_expr_type() {
        let json =
            r#"{"type":"program","body":[{"type":"expr","value":{"type":"unknown","foo":1}}]}"#;
        let result = json_to_ast(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_error_unknown_stmt_type() {
        let json = r#"{"type":"program","body":[{"type":"bogus"}]}"#;
        let result = json_to_ast(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_error_unknown_op() {
        let json = r#"{"type":"program","body":[{"type":"expr","value":{"type":"binop","op":"???","left":{"type":"lit","value":1},"right":{"type":"lit","value":2}}}]}"#;
        let result = json_to_ast(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_error_invalid_json() {
        let result = json_to_ast("{bad json");
        assert!(result.is_err());
    }

    // --- Complex roundtrip: full program ---

    #[test]
    fn roundtrip_complex_program() {
        let source = "
fn fib(n) do
  if n <= 1 do
    return n
  end
  let a = 0
  let b = 1
  let i = 2
  while i <= n do
    let temp = b
    b = a + b
    a = temp
    i = i + 1
  end
  return b
end
fib(10)
";
        let json = source_to_json(source);
        assert!(json.is_ok());
        if let Ok(json_str) = json {
            let stmts = json_to_ast(&json_str);
            assert!(stmts.is_ok());
            // Compile from JSON and run
            let result = json_eval(&json_str);
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![55]); // fib(10) = 55
            }
        }
    }

    #[test]
    fn roundtrip_source_json_source() {
        // Source → JSON → AST → JSON → AST  (double roundtrip)
        let source = "let x = 5\nlet y = x * 2\ny + 1";
        let json1 = source_to_json(source);
        assert!(json1.is_ok());
        if let Ok(j1) = json1 {
            let ast1 = json_to_ast(&j1);
            assert!(ast1.is_ok());
            if let Ok(a1) = ast1 {
                let j2 = ast_to_json(&a1);
                let ast2 = json_to_ast(&j2);
                assert!(ast2.is_ok());
                if let Ok(a2) = ast2 {
                    assert_eq!(a1, a2);
                }
            }
        }
    }
}
