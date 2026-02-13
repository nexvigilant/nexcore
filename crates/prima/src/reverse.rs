// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Reverse Transcriptase — Data → AST → Source
//!
//! The inverse of Prima's compilation pipeline.
//!
//! ## Biological Analogy
//!
//! In biology, reverse transcriptase synthesizes DNA from RNA,
//! reversing the central dogma (DNA → RNA → Protein).
//!
//! In Prima:
//! - **Forward** (transcription): Source → Tokens → AST → Value
//! - **Reverse** (reverse transcription): Data → AST → Source
//!
//! ## Use Cases
//!
//! 1. **JSON → Prima**: Convert observed data into `.true` source
//! 2. **Pattern inference**: Detect types, ranges, and structure from data
//! 3. **Violation synthesis**: Generate `.not.true` from invalid states
//! 4. **Code generation**: Produce reproducible Prima programs from data
//!
//! ## Mathematical Foundation
//!
//! Forward: f(source) = value
//! Reverse: f⁻¹(value) = source (where f⁻¹ ∘ f ≈ id)
//!
//! The reverse is approximate — multiple sources can produce the same value.
//! We choose the canonical (simplest) source representation.
//!
//! ## Tier: T2-C (ρ + → + σ + μ)

use crate::ast::{BinOp, Block, Expr, Literal, Program, Stmt, TypeExpr, TypeKind};
use crate::error::{PrimaError, PrimaResult};
use crate::token::Span;
use serde_json::Value as JsonValue;

/// Synthetic span for reverse-transcribed AST nodes.
/// All generated nodes use offset 0 since they have no source location.
const SYNTH: Span = Span::new(0, 0, 0);

// ─── Core Transcription Engine ──────────────────────────────────────────────

/// Reverse-transcribe a JSON value into a Prima AST expression.
///
/// This is the heart of the enzyme: observed data → AST node.
///
/// ## Mapping
///
/// | JSON            | Prima AST          | Primitive |
/// |-----------------|--------------------|-----------|
/// | `null`          | `Literal::Void`    | ∅         |
/// | `true`/`false`  | `Literal::Bool`    | Σ         |
/// | integer         | `Literal::Int`     | N         |
/// | float           | `Literal::Float`   | N         |
/// | `"string"`      | `Literal::String`  | σ[N]      |
/// | `[...]`         | `Sequence`         | σ         |
/// | `{...}`         | `Mapping`          | μ         |
pub fn transcribe_json(json: &JsonValue) -> Expr {
    match json {
        JsonValue::Null => Expr::Literal {
            value: Literal::Void,
            span: SYNTH,
        },
        JsonValue::Bool(b) => Expr::Literal {
            value: Literal::Bool(*b),
            span: SYNTH,
        },
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Expr::Literal {
                    value: Literal::Int(i),
                    span: SYNTH,
                }
            } else if let Some(f) = n.as_f64() {
                Expr::Literal {
                    value: Literal::Float(f),
                    span: SYNTH,
                }
            } else {
                // Fallback: represent as 0 (absence)
                Expr::Literal {
                    value: Literal::Int(0),
                    span: SYNTH,
                }
            }
        }
        JsonValue::String(s) => Expr::Literal {
            value: Literal::String(s.clone()),
            span: SYNTH,
        },
        JsonValue::Array(arr) => {
            let elements = arr.iter().map(transcribe_json).collect();
            Expr::Sequence {
                elements,
                span: SYNTH,
            }
        }
        JsonValue::Object(obj) => {
            let pairs = obj
                .iter()
                .map(|(k, v)| {
                    let key = Expr::Literal {
                        value: Literal::String(k.clone()),
                        span: SYNTH,
                    };
                    let val = transcribe_json(v);
                    (key, val)
                })
                .collect();
            Expr::Mapping { pairs, span: SYNTH }
        }
    }
}

/// Reverse-transcribe a JSON value into a Prima AST Program.
///
/// Wraps the expression in a statement, producing a complete program.
pub fn transcribe_json_to_program(json: &JsonValue) -> Program {
    let expr = transcribe_json(json);
    Program {
        statements: vec![Stmt::Expr { expr, span: SYNTH }],
    }
}

/// Reverse-transcribe a JSON array into a Prima program with let-bindings.
///
/// Each top-level key in an object becomes a `let` binding.
/// Arrays become sequences. This produces more idiomatic Prima.
pub fn transcribe_json_bindings(json: &JsonValue) -> Program {
    match json {
        JsonValue::Object(obj) => {
            let statements = obj
                .iter()
                .map(|(k, v)| {
                    let name = sanitize_identifier(k);
                    Stmt::Let {
                        name,
                        value: transcribe_json(v),
                        span: SYNTH,
                    }
                })
                .collect();
            Program { statements }
        }
        _ => transcribe_json_to_program(json),
    }
}

// ─── Source Emitter ─────────────────────────────────────────────────────────

/// Emit Prima source code from an AST expression.
///
/// This is the final step: AST → source text.
/// The emitted source, when compiled forward, reproduces the original data.
pub fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal { value, .. } => emit_literal(value),
        Expr::Ident { name, .. } => name.clone(),
        Expr::Binary {
            left, op, right, ..
        } => {
            format!(
                "{} {} {}",
                emit_expr(left),
                emit_binop(op),
                emit_expr(right)
            )
        }
        Expr::Unary { op, operand, .. } => {
            let op_str = match op {
                crate::ast::UnOp::Neg => "-",
                crate::ast::UnOp::Not => "!",
            };
            format!("{}{}", op_str, emit_expr(operand))
        }
        Expr::Call { func, args, .. } => {
            let arg_strs: Vec<String> = args.iter().map(emit_expr).collect();
            format!("{}({})", func, arg_strs.join(", "))
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            let mut s = format!("∂ {} {}", emit_expr(cond), emit_block(then_branch));
            if let Some(eb) = else_branch {
                s.push_str(&format!(" else {}", emit_block(eb)));
            }
            s
        }
        Expr::For {
            var, iter, body, ..
        } => {
            format!("σ {} in {} {}", var, emit_expr(iter), emit_block(body))
        }
        Expr::Sequence { elements, .. } => {
            let elems: Vec<String> = elements.iter().map(emit_expr).collect();
            format!("σ[{}]", elems.join(", "))
        }
        Expr::Mapping { pairs, .. } => {
            let pair_strs: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{} → {}", emit_expr(k), emit_expr(v)))
                .collect();
            format!("μ({})", pair_strs.join(", "))
        }
        Expr::Lambda { params, body, .. } => {
            let param_strs: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            format!("|{}| {}", param_strs.join(", "), emit_expr(body))
        }
        Expr::Block { block, .. } => emit_block(block),
        Expr::Member { object, field, .. } => {
            format!("{}.{}", emit_expr(object), field)
        }
        Expr::MethodCall {
            object,
            method,
            args,
            ..
        } => {
            let arg_strs: Vec<String> = args.iter().map(emit_expr).collect();
            format!("{}.{}({})", emit_expr(object), method, arg_strs.join(", "))
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            let mut s = format!("Σ {} {{\n", emit_expr(scrutinee));
            for arm in arms {
                s.push_str(&format!(
                    "  {} → {},\n",
                    emit_pattern(&arm.pattern),
                    emit_expr(&arm.body)
                ));
            }
            s.push('}');
            s
        }
        Expr::Quoted { expr: inner, .. } => format!("'{}", emit_expr(inner)),
        Expr::Quasiquoted { expr: inner, .. } => format!("`{}", emit_expr(inner)),
        Expr::Unquoted { expr: inner, .. } => format!("~{}", emit_expr(inner)),
        Expr::UnquotedSplice { expr: inner, .. } => format!("~@{}", emit_expr(inner)),
    }
}

/// Emit a literal value as Prima source.
fn emit_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n) => n.to_string(),
        Literal::Float(f) => format!("{f}"),
        Literal::String(s) => format!("\"{s}\""),
        Literal::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Literal::Void => "∅".to_string(),
        Literal::Symbol(s) => format!(":{s}"),
    }
}

/// Emit a binary operator as Prima source.
fn emit_binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::Le => "<=",
        BinOp::Ge => ">=",
        BinOp::KappaEq => "κ=",
        BinOp::KappaNe => "κ!=",
        BinOp::KappaLt => "κ<",
        BinOp::KappaGt => "κ>",
        BinOp::KappaLe => "κ<=",
        BinOp::KappaGe => "κ>=",
        BinOp::And => "&&",
        BinOp::Or => "||",
    }
}

/// Emit a block as Prima source.
fn emit_block(block: &Block) -> String {
    let mut parts = Vec::new();
    for stmt in &block.statements {
        parts.push(emit_stmt(stmt));
    }
    if let Some(expr) = &block.expr {
        parts.push(emit_expr(expr));
    }
    format!("{{ {} }}", parts.join("; "))
}

/// Emit a statement as Prima source.
pub fn emit_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Let { name, value, .. } => {
            format!("λ {} = {}", name, emit_expr(value))
        }
        Stmt::FnDef {
            name,
            params,
            ret,
            body,
            ..
        } => {
            let param_strs: Vec<String> = params
                .iter()
                .map(|p| format!("{}: {}", p.name, emit_type(&p.ty)))
                .collect();
            format!(
                "μ {}({}) → {} {}",
                name,
                param_strs.join(", "),
                emit_type(ret),
                emit_block(body)
            )
        }
        Stmt::TypeDef { name, ty, .. } => {
            format!("type {} = {}", name, emit_type(ty))
        }
        Stmt::Expr { expr, .. } => emit_expr(expr),
        Stmt::Return { value, .. } => match value {
            Some(v) => format!("return {}", emit_expr(v)),
            None => "return".to_string(),
        },
    }
}

/// Emit a type expression as Prima source.
fn emit_type(ty: &TypeExpr) -> String {
    match &ty.kind {
        TypeKind::Primitive(p) => p.symbol().to_string(),
        TypeKind::Named(n) => n.clone(),
        TypeKind::Sequence(inner) => format!("σ[{}]", emit_type(inner)),
        TypeKind::Mapping(k, v) => format!("μ[{} → {}]", emit_type(k), emit_type(v)),
        TypeKind::Sum(variants) => {
            let vs: Vec<String> = variants.iter().map(emit_type).collect();
            vs.join(" | ")
        }
        TypeKind::Function(params, ret) => {
            let ps: Vec<String> = params.iter().map(emit_type).collect();
            format!("({}) → {}", ps.join(", "), emit_type(ret))
        }
        TypeKind::Optional(inner) => format!("{} | ∅", emit_type(inner)),
        TypeKind::Void => "∅".to_string(),
        TypeKind::Infer => "_".to_string(),
    }
}

/// Emit a pattern as Prima source.
fn emit_pattern(pat: &crate::ast::Pattern) -> String {
    match pat {
        crate::ast::Pattern::Wildcard { .. } => "_".to_string(),
        crate::ast::Pattern::Literal { value, .. } => emit_literal(value),
        crate::ast::Pattern::Ident { name, .. } => name.clone(),
        crate::ast::Pattern::Constructor { name, fields, .. } => {
            let fs: Vec<String> = fields.iter().map(emit_pattern).collect();
            format!("{}({})", name, fs.join(", "))
        }
    }
}

/// Emit a full program as Prima source.
pub fn emit_program(program: &Program) -> String {
    program
        .statements
        .iter()
        .map(emit_stmt)
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── Pattern Inference ──────────────────────────────────────────────────────

/// Inferred schema from observed JSON data.
#[derive(Debug, Clone)]
pub struct InferredSchema {
    /// Name of the field (if from an object key).
    pub name: Option<String>,
    /// The inferred Prima type.
    pub kind: SchemaKind,
}

/// Schema kind — what type was observed.
#[derive(Debug, Clone)]
pub enum SchemaKind {
    /// Void/null observed.
    Void,
    /// Boolean values observed.
    Bool,
    /// Integer values with observed range.
    Int { min: i64, max: i64 },
    /// Float values with observed range.
    Float { min: f64, max: f64 },
    /// String values observed.
    Str { max_len: usize },
    /// Homogeneous array with element schema.
    Sequence(Box<InferredSchema>),
    /// Object with field schemas.
    Record(Vec<InferredSchema>),
    /// Mixed types observed (fallback).
    Mixed,
}

/// Infer a schema from a JSON value.
///
/// This is the "observation" phase: looking at data and
/// inferring the structure that produced it.
pub fn infer_schema(json: &JsonValue) -> InferredSchema {
    infer_schema_named(json, None)
}

fn infer_schema_named(json: &JsonValue, name: Option<String>) -> InferredSchema {
    let kind = match json {
        JsonValue::Null => SchemaKind::Void,
        JsonValue::Bool(_) => SchemaKind::Bool,
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                SchemaKind::Int { min: i, max: i }
            } else if let Some(f) = n.as_f64() {
                SchemaKind::Float { min: f, max: f }
            } else {
                SchemaKind::Mixed
            }
        }
        JsonValue::String(s) => SchemaKind::Str { max_len: s.len() },
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                SchemaKind::Sequence(Box::new(InferredSchema {
                    name: None,
                    kind: SchemaKind::Void,
                }))
            } else {
                // Infer element schema from first element (simplified)
                let elem_schema = infer_schema(&arr[0]);
                SchemaKind::Sequence(Box::new(elem_schema))
            }
        }
        JsonValue::Object(obj) => {
            let fields = obj
                .iter()
                .map(|(k, v)| infer_schema_named(v, Some(k.clone())))
                .collect();
            SchemaKind::Record(fields)
        }
    };
    InferredSchema { name, kind }
}

/// Generate a Prima type annotation from an inferred schema.
pub fn schema_to_type(schema: &InferredSchema) -> TypeExpr {
    let kind = match &schema.kind {
        SchemaKind::Void => TypeKind::Void,
        SchemaKind::Bool => TypeKind::Named("Bool".to_string()),
        SchemaKind::Int { .. } => TypeKind::Named("N".to_string()),
        SchemaKind::Float { .. } => TypeKind::Named("N".to_string()),
        SchemaKind::Str { .. } => TypeKind::Named("Str".to_string()),
        SchemaKind::Sequence(inner) => TypeKind::Sequence(Box::new(schema_to_type(inner))),
        SchemaKind::Record(_) => {
            // Records are mappings in Prima
            TypeKind::Mapping(
                Box::new(TypeExpr {
                    kind: TypeKind::Named("Str".to_string()),
                    span: SYNTH,
                }),
                Box::new(TypeExpr {
                    kind: TypeKind::Named("Any".to_string()),
                    span: SYNTH,
                }),
            )
        }
        SchemaKind::Mixed => TypeKind::Named("Any".to_string()),
    };
    TypeExpr { kind, span: SYNTH }
}

// ─── Violation Synthesis (.not.true Generation) ─────────────────────────────

/// Generate `.not.true` antipattern assertions from a schema.
///
/// For each field, generates expressions that SHOULD be false/error
/// based on the observed valid range. These are the boundaries.
///
/// Example: If we observed `age: Int { min: 0, max: 120 }`,
/// we generate: `age < 0` (should be false) and `age > 120` (should be false).
pub fn synthesize_violations(schema: &InferredSchema) -> Vec<String> {
    let mut violations = Vec::new();
    synthesize_violations_inner(schema, &mut violations, "");
    violations
}

fn synthesize_violations_inner(
    schema: &InferredSchema,
    violations: &mut Vec<String>,
    prefix: &str,
) {
    let field_name = match &schema.name {
        Some(n) => {
            if prefix.is_empty() {
                n.clone()
            } else {
                format!("{prefix}.{n}")
            }
        }
        None => prefix.to_string(),
    };

    match &schema.kind {
        SchemaKind::Int { min, max } => {
            // Out-of-range violations
            if *min > i64::MIN {
                violations.push(format!(
                    "// {field_name} below observed minimum is a violation"
                ));
                violations.push(format!("{} > {}", min - 1, max));
            }
            if *max < i64::MAX {
                violations.push(format!(
                    "// {field_name} above observed maximum is a violation"
                ));
                violations.push(format!("{} > {}", max + 1, min));
            }
        }
        SchemaKind::Float { min, max } => {
            violations.push(format!(
                "// {field_name} outside [{min}, {max}] is a violation"
            ));
            // NaN check: 0/0 should error
            violations.push(format!("// {field_name} NaN is always a violation"));
        }
        SchemaKind::Str { max_len } => {
            // Empty string is falsy in Prima — good violation candidate
            violations.push(format!("// {field_name} empty string is falsy"));
            violations.push("\"\"".to_string());
            if *max_len > 0 {
                violations.push(format!("// {field_name} max observed length: {max_len}"));
            }
        }
        SchemaKind::Sequence(inner) => {
            // Empty sequence is falsy
            violations.push(format!("// {field_name} empty sequence is falsy"));
            violations.push("σ[]".to_string());
            synthesize_violations_inner(inner, violations, &field_name);
        }
        SchemaKind::Record(fields) => {
            for field in fields {
                synthesize_violations_inner(field, violations, &field_name);
            }
        }
        SchemaKind::Void => {
            violations.push(format!("// {field_name} is void (already not-true)"));
        }
        SchemaKind::Bool => {
            // false is already not-true
            violations.push(format!("// {field_name} false is not-true"));
            violations.push("false".to_string());
        }
        SchemaKind::Mixed => {
            violations.push(format!(
                "// {field_name} mixed type — no specific violation"
            ));
        }
    }
}

/// Generate a complete `.not.true` file from a schema.
pub fn synthesize_not_true_file(schema: &InferredSchema, source_name: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("// {source_name}.not.true"));
    lines.push("// ═══════════════════════════════════════════════".to_string());
    lines.push("// Auto-generated by reverse transcriptase".to_string());
    lines.push("// Every expression here MUST be false, void, or error.".to_string());
    lines.push("// ═══════════════════════════════════════════════".to_string());
    lines.push(String::new());

    let violations = synthesize_violations(schema);
    for v in violations {
        lines.push(v);
    }

    lines.join("\n")
}

// ─── Full Pipeline: JSON → Source ───────────────────────────────────────────

/// Full reverse transcription: JSON string → Prima source code.
///
/// This is the main entry point for the CLI.
pub fn reverse_transcribe(json_str: &str) -> PrimaResult<TranscriptionResult> {
    let json: JsonValue = serde_json::from_str(json_str)
        .map_err(|e| PrimaError::runtime(format!("JSON parse error: {e}")))?;

    let schema = infer_schema(&json);
    let program = transcribe_json_bindings(&json);
    let source = emit_program(&program);
    let not_true = synthesize_not_true_file(&schema, "reverse_transcribed");

    Ok(TranscriptionResult {
        source,
        not_true,
        schema,
        program,
    })
}

/// Result of a reverse transcription.
#[derive(Debug)]
pub struct TranscriptionResult {
    /// The generated `.true` source code.
    pub source: String,
    /// The generated `.not.true` antipattern file.
    pub not_true: String,
    /// The inferred schema from the input data.
    pub schema: InferredSchema,
    /// The generated AST program.
    pub program: Program,
}

// ═══════════════════════════════════════════════════════════════════════════
// TranscriptionEngine — The Bidirectional Pipeline
// ═══════════════════════════════════════════════════════════════════════════
//
// Biology: the ribosome orchestrates transcription + translation.
// This engine orchestrates forward (eval) + reverse (data→source)
// and verifies the round-trip fidelity.
//
// Tier: T3 (ρ + → + σ + μ + κ + ∂)

/// How to emit reverse-transcribed source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitMode {
    /// Emit as a single expression (for simple values).
    Expression,
    /// Emit object keys as `let` bindings (more idiomatic).
    Bindings,
}

/// Configuration for the transcription engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// How to emit reverse-transcribed source.
    pub emit_mode: EmitMode,
    /// Whether to generate `.not.true` violations alongside source.
    pub synthesize_violations: bool,
    /// Whether to verify round-trip fidelity on reverse transcription.
    pub verify_roundtrip: bool,
    /// Name used in generated `.not.true` headers.
    pub source_name: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            emit_mode: EmitMode::Bindings,
            synthesize_violations: true,
            verify_roundtrip: false,
            source_name: "engine".to_string(),
        }
    }
}

/// Statistics tracked by the engine across operations.
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    /// Total reverse transcriptions performed.
    pub reverse_count: usize,
    /// Total forward evaluations performed.
    pub forward_count: usize,
    /// Total round-trip verifications performed.
    pub roundtrip_count: usize,
    /// Successful round-trip verifications.
    pub roundtrip_pass: usize,
    /// Failed round-trip verifications.
    pub roundtrip_fail: usize,
    /// Records observed for schema merging.
    pub records_observed: usize,
    /// Batch operations performed.
    pub batch_count: usize,
}

/// Result of a round-trip verification.
#[derive(Debug, Clone)]
pub struct RoundtripResult {
    /// The reverse-transcribed source.
    pub source: String,
    /// The forward-evaluated value from that source.
    pub value: crate::value::Value,
    /// Whether the round-trip preserved the data.
    pub fidelity: RoundtripFidelity,
}

/// How faithful was the round-trip?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundtripFidelity {
    /// Perfect: f(f⁻¹(data)) = data
    Exact,
    /// Structural match but possible type coercion (e.g. float precision).
    Approximate,
    /// Forward evaluation failed — source was not valid Prima.
    Failed,
}

/// Result of a batch transcription.
#[derive(Debug)]
pub struct BatchResult {
    /// Individual transcription results.
    pub items: Vec<TranscriptionResult>,
    /// Merged schema across all records.
    pub merged_schema: InferredSchema,
    /// Combined `.not.true` file for the merged schema.
    pub merged_not_true: String,
}

/// The Transcription Engine — bidirectional pipeline with verification.
///
/// ## Biological Analogy
///
/// The ribosome is the molecular machine that reads mRNA and
/// assembles proteins. This engine reads data (RNA) and assembles
/// Prima source (DNA), then verifies by running it forward (protein).
///
/// ## Pipeline
///
/// ```text
/// ┌──────────┐    reverse()     ┌─────────┐     forward()     ┌───────┐
/// │   Data   │ ───────────────► │ Source   │ ────────────────► │ Value │
/// │  (JSON)  │                  │ (.true)  │                   │       │
/// └──────────┘                  └─────────┘                   └───────┘
///      │                             │
///      │  observe()                  │  synthesize
///      ▼                             ▼
/// ┌──────────┐                ┌────────────┐
/// │  Schema  │                │ .not.true  │
/// │ (merged) │                │ violations │
/// └──────────┘                └────────────┘
/// ```
pub struct TranscriptionEngine {
    config: EngineConfig,
    stats: EngineStats,
    /// Accumulated schema from `observe()` calls for multi-record inference.
    merged: Option<InferredSchema>,
}

impl TranscriptionEngine {
    /// Create a new engine with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            stats: EngineStats::default(),
            merged: None,
        }
    }

    /// Create a new engine with custom config.
    #[must_use]
    pub fn with_config(config: EngineConfig) -> Self {
        Self {
            config,
            stats: EngineStats::default(),
            merged: None,
        }
    }

    /// Get engine statistics.
    #[must_use]
    pub fn stats(&self) -> &EngineStats {
        &self.stats
    }

    /// Get the config.
    #[must_use]
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get the merged schema (if any records have been observed).
    #[must_use]
    pub fn merged_schema(&self) -> Option<&InferredSchema> {
        self.merged.as_ref()
    }

    // ── Reverse: Data → Source ──────────────────────────────────────────

    /// Reverse-transcribe JSON input into Prima source.
    pub fn reverse(&mut self, json_str: &str) -> PrimaResult<TranscriptionResult> {
        let json: JsonValue = serde_json::from_str(json_str)
            .map_err(|e| PrimaError::runtime(format!("JSON parse error: {e}")))?;

        let schema = infer_schema(&json);
        let program = match self.config.emit_mode {
            EmitMode::Bindings => transcribe_json_bindings(&json),
            EmitMode::Expression => transcribe_json_to_program(&json),
        };
        let source = emit_program(&program);

        let not_true = if self.config.synthesize_violations {
            synthesize_not_true_file(&schema, &self.config.source_name)
        } else {
            String::new()
        };

        self.stats.reverse_count += 1;

        Ok(TranscriptionResult {
            source,
            not_true,
            schema,
            program,
        })
    }

    // ── Forward: Source → Value ──────────────────────────────────────────

    /// Forward-evaluate Prima source code.
    pub fn forward(&mut self, source: &str) -> PrimaResult<crate::value::Value> {
        self.stats.forward_count += 1;
        crate::eval(source)
    }

    // ── Round-trip: Data → Source → Value (with verification) ───────────

    /// Reverse-transcribe then forward-evaluate, checking fidelity.
    ///
    /// This is the acid test: `f(f⁻¹(data)) ≈ data`.
    pub fn roundtrip(&mut self, json_str: &str) -> PrimaResult<RoundtripResult> {
        let json: JsonValue = serde_json::from_str(json_str)
            .map_err(|e| PrimaError::runtime(format!("JSON parse error: {e}")))?;

        // Reverse: data → source
        let expr = transcribe_json(&json);
        let source = emit_expr(&expr);

        // Forward: source → value
        let fidelity = match crate::eval(&source) {
            Ok(value) => {
                let fidelity = check_fidelity(&json, &value);
                self.stats.roundtrip_count += 1;
                match fidelity {
                    RoundtripFidelity::Exact | RoundtripFidelity::Approximate => {
                        self.stats.roundtrip_pass += 1;
                    }
                    RoundtripFidelity::Failed => {
                        self.stats.roundtrip_fail += 1;
                    }
                }
                return Ok(RoundtripResult {
                    source,
                    value,
                    fidelity,
                });
            }
            Err(_) => RoundtripFidelity::Failed,
        };

        self.stats.roundtrip_count += 1;
        self.stats.roundtrip_fail += 1;

        Ok(RoundtripResult {
            source,
            value: crate::value::Value::void(),
            fidelity,
        })
    }

    // ── Observe: Accumulate Schema ──────────────────────────────────────

    /// Observe a JSON record to refine the merged schema.
    ///
    /// Call this multiple times with different records to widen
    /// the inferred ranges and discover all fields.
    pub fn observe(&mut self, json_str: &str) -> PrimaResult<()> {
        let json: JsonValue = serde_json::from_str(json_str)
            .map_err(|e| PrimaError::runtime(format!("JSON parse error: {e}")))?;

        let new_schema = infer_schema(&json);
        self.merged = Some(match self.merged.take() {
            Some(existing) => merge_schemas(&existing, &new_schema),
            None => new_schema,
        });
        self.stats.records_observed += 1;
        Ok(())
    }

    /// Generate `.not.true` from the merged schema.
    ///
    /// Must call `observe()` at least once first.
    pub fn synthesize_merged_violations(&self) -> Option<String> {
        self.merged
            .as_ref()
            .map(|s| synthesize_not_true_file(s, &self.config.source_name))
    }

    // ── Batch: Process Arrays ───────────────────────────────────────────

    /// Batch-process a JSON array of records.
    ///
    /// Each element is reverse-transcribed individually, and all
    /// schemas are merged to produce a unified `.not.true` file.
    pub fn batch(&mut self, json_str: &str) -> PrimaResult<BatchResult> {
        let json: JsonValue = serde_json::from_str(json_str)
            .map_err(|e| PrimaError::runtime(format!("JSON parse error: {e}")))?;

        let records = match &json {
            JsonValue::Array(arr) => arr.clone(),
            // Single record → array of one
            other => vec![other.clone()],
        };

        let mut items = Vec::with_capacity(records.len());
        let mut merged: Option<InferredSchema> = None;

        for record in &records {
            let schema = infer_schema(record);
            let program = match self.config.emit_mode {
                EmitMode::Bindings => transcribe_json_bindings(record),
                EmitMode::Expression => transcribe_json_to_program(record),
            };
            let source = emit_program(&program);
            let not_true = if self.config.synthesize_violations {
                synthesize_not_true_file(&schema, &self.config.source_name)
            } else {
                String::new()
            };

            merged = Some(match merged.take() {
                Some(existing) => merge_schemas(&existing, &schema),
                None => schema.clone(),
            });

            items.push(TranscriptionResult {
                source,
                not_true,
                schema,
                program,
            });
        }

        let merged_schema = merged.unwrap_or(InferredSchema {
            name: None,
            kind: SchemaKind::Void,
        });
        let merged_not_true = synthesize_not_true_file(&merged_schema, &self.config.source_name);

        self.stats.batch_count += 1;
        self.stats.reverse_count += items.len();

        Ok(BatchResult {
            items,
            merged_schema,
            merged_not_true,
        })
    }

    /// Reset the engine — clear merged schema and stats.
    pub fn reset(&mut self) {
        self.stats = EngineStats::default();
        self.merged = None;
    }
}

impl Default for TranscriptionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Schema Merging ─────────────────────────────────────────────────────────

/// Merge two schemas, widening ranges and unioning fields.
///
/// This is how the engine learns from multiple observations:
/// - Int ranges widen: `{min: 0, max: 10}` + `{min: -5, max: 20}` → `{min: -5, max: 20}`
/// - Float ranges widen similarly
/// - String max_len takes the maximum
/// - Records union their fields
/// - Incompatible types become Mixed
fn merge_schemas(a: &InferredSchema, b: &InferredSchema) -> InferredSchema {
    let name = a.name.clone().or_else(|| b.name.clone());
    let kind = merge_kinds(&a.kind, &b.kind);
    InferredSchema { name, kind }
}

fn merge_kinds(a: &SchemaKind, b: &SchemaKind) -> SchemaKind {
    match (a, b) {
        // Same simple types
        (SchemaKind::Void, SchemaKind::Void) => SchemaKind::Void,
        (SchemaKind::Bool, SchemaKind::Bool) => SchemaKind::Bool,

        // Widen integer ranges
        (
            SchemaKind::Int {
                min: a_min,
                max: a_max,
            },
            SchemaKind::Int {
                min: b_min,
                max: b_max,
            },
        ) => SchemaKind::Int {
            min: (*a_min).min(*b_min),
            max: (*a_max).max(*b_max),
        },

        // Widen float ranges
        (
            SchemaKind::Float {
                min: a_min,
                max: a_max,
            },
            SchemaKind::Float {
                min: b_min,
                max: b_max,
            },
        ) => {
            let min = if *a_min < *b_min { *a_min } else { *b_min };
            let max = if *a_max > *b_max { *a_max } else { *b_max };
            SchemaKind::Float { min, max }
        }

        // Widen string length
        (SchemaKind::Str { max_len: a_len }, SchemaKind::Str { max_len: b_len }) => {
            SchemaKind::Str {
                max_len: (*a_len).max(*b_len),
            }
        }

        // Merge sequences by merging element schemas
        (SchemaKind::Sequence(a_inner), SchemaKind::Sequence(b_inner)) => {
            SchemaKind::Sequence(Box::new(merge_schemas(a_inner, b_inner)))
        }

        // Merge records by unioning fields
        (SchemaKind::Record(a_fields), SchemaKind::Record(b_fields)) => {
            let mut merged_fields: Vec<InferredSchema> = Vec::new();

            // Start with all fields from A
            for af in a_fields {
                let matching_b = b_fields.iter().find(|bf| bf.name == af.name);
                match matching_b {
                    Some(bf) => merged_fields.push(merge_schemas(af, bf)),
                    None => merged_fields.push(af.clone()),
                }
            }

            // Add fields from B that don't exist in A
            for bf in b_fields {
                let exists = a_fields.iter().any(|af| af.name == bf.name);
                if !exists {
                    merged_fields.push(bf.clone());
                }
            }

            SchemaKind::Record(merged_fields)
        }

        // Int + Float → Float (widening coercion)
        (SchemaKind::Int { min, max }, SchemaKind::Float { .. })
        | (SchemaKind::Float { .. }, SchemaKind::Int { min, max }) => SchemaKind::Float {
            min: *min as f64,
            max: *max as f64,
        },

        // Everything else → Mixed
        _ => SchemaKind::Mixed,
    }
}

// ─── Fidelity Checking ──────────────────────────────────────────────────────

/// Check how faithfully a round-trip preserved the data.
fn check_fidelity(json: &JsonValue, value: &crate::value::Value) -> RoundtripFidelity {
    use crate::value::ValueData;

    match (json, &value.data) {
        // Null → Void: exact
        (JsonValue::Null, ValueData::Void) => RoundtripFidelity::Exact,

        // Bool → Bool: exact
        (JsonValue::Bool(a), ValueData::Bool(b)) if *a == *b => RoundtripFidelity::Exact,

        // Int → Int: exact
        (JsonValue::Number(n), ValueData::Int(i)) if n.as_i64() == Some(*i) => {
            RoundtripFidelity::Exact
        }

        // Float → Float: approximate (precision)
        (JsonValue::Number(n), ValueData::Float(f)) => {
            if let Some(jf) = n.as_f64() {
                if (jf - f).abs() < f64::EPSILON {
                    RoundtripFidelity::Exact
                } else {
                    RoundtripFidelity::Approximate
                }
            } else {
                RoundtripFidelity::Failed
            }
        }

        // String → String: exact
        (JsonValue::String(a), ValueData::String(b)) if a == b => RoundtripFidelity::Exact,

        // Array → Sequence: check element-wise
        (JsonValue::Array(arr), ValueData::Sequence(seq)) if arr.len() == seq.len() => {
            let mut all_exact = true;
            for (j, v) in arr.iter().zip(seq.iter()) {
                match check_fidelity(j, v) {
                    RoundtripFidelity::Exact => {}
                    RoundtripFidelity::Approximate => all_exact = false,
                    RoundtripFidelity::Failed => return RoundtripFidelity::Failed,
                }
            }
            if all_exact {
                RoundtripFidelity::Exact
            } else {
                RoundtripFidelity::Approximate
            }
        }

        // Anything else: failed
        _ => RoundtripFidelity::Failed,
    }
}

// ─── Display Implementations ────────────────────────────────────────────────

impl std::fmt::Display for EngineStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TranscriptionEngine Statistics")?;
        writeln!(f, "─────────────────────────────────────")?;
        writeln!(f, "  Reverse transcriptions: {}", self.reverse_count)?;
        writeln!(f, "  Forward evaluations:    {}", self.forward_count)?;
        writeln!(f, "  Round-trip checks:      {}", self.roundtrip_count)?;
        writeln!(
            f,
            "  Round-trip fidelity:    {}/{}",
            self.roundtrip_pass, self.roundtrip_count
        )?;
        writeln!(f, "  Records observed:       {}", self.records_observed)?;
        writeln!(f, "  Batch operations:       {}", self.batch_count)?;
        Ok(())
    }
}

impl std::fmt::Display for RoundtripFidelity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "EXACT (f⁻¹∘f = id)"),
            Self::Approximate => write!(f, "APPROX (f⁻¹∘f ≈ id)"),
            Self::Failed => write!(f, "FAILED (f⁻¹∘f ≠ id)"),
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Sanitize a string into a valid Prima identifier.
///
/// Replaces non-alphanumeric characters with underscores.
fn sanitize_identifier(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for (i, c) in s.chars().enumerate() {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            result.push('_');
        }
        // Identifiers can't start with a digit
        if i == 0 && c.is_ascii_digit() {
            result.insert(0, '_');
        }
    }
    if result.is_empty() {
        result.push_str("_unnamed");
    }
    result
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Transcription Tests ─────────────────────────────────────────────

    #[test]
    fn test_transcribe_null() {
        let json: JsonValue = serde_json::from_str("null").ok().unwrap_or(JsonValue::Null);
        let expr = transcribe_json(&json);
        let source = emit_expr(&expr);
        assert_eq!(source, "∅");
    }

    #[test]
    fn test_transcribe_bool_true() {
        let json = JsonValue::Bool(true);
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "true");
    }

    #[test]
    fn test_transcribe_bool_false() {
        let json = JsonValue::Bool(false);
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "false");
    }

    #[test]
    fn test_transcribe_integer() {
        let json = serde_json::json!(42);
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "42");
    }

    #[test]
    fn test_transcribe_negative_integer() {
        let json = serde_json::json!(-7);
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "-7");
    }

    #[test]
    fn test_transcribe_float() {
        let json = serde_json::json!(3.14);
        let source = emit_expr(&transcribe_json(&json));
        assert!(source.starts_with("3.14"));
    }

    #[test]
    fn test_transcribe_string() {
        let json = serde_json::json!("hello world");
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "\"hello world\"");
    }

    #[test]
    fn test_transcribe_empty_array() {
        let json = serde_json::json!([]);
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "σ[]");
    }

    #[test]
    fn test_transcribe_int_array() {
        let json = serde_json::json!([1, 2, 3]);
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "σ[1, 2, 3]");
    }

    #[test]
    fn test_transcribe_nested_array() {
        let json = serde_json::json!([[1, 2], [3, 4]]);
        let source = emit_expr(&transcribe_json(&json));
        assert_eq!(source, "σ[σ[1, 2], σ[3, 4]]");
    }

    #[test]
    fn test_transcribe_object() {
        let json = serde_json::json!({"age": 30});
        let source = emit_expr(&transcribe_json(&json));
        // Should produce a mapping
        assert!(source.starts_with("μ("));
        assert!(source.contains("\"age\""));
        assert!(source.contains("30"));
    }

    #[test]
    fn test_transcribe_object_to_bindings() {
        let json = serde_json::json!({"name": "Prima", "version": 1});
        let program = transcribe_json_bindings(&json);
        let source = emit_program(&program);
        assert!(source.contains("λ name = \"Prima\"") || source.contains("λ version = 1"));
    }

    // ── Schema Inference Tests ──────────────────────────────────────────

    #[test]
    fn test_infer_int_schema() {
        let json = serde_json::json!(42);
        let schema = infer_schema(&json);
        assert!(matches!(schema.kind, SchemaKind::Int { min: 42, max: 42 }));
    }

    #[test]
    fn test_infer_string_schema() {
        let json = serde_json::json!("hello");
        let schema = infer_schema(&json);
        assert!(matches!(schema.kind, SchemaKind::Str { max_len: 5 }));
    }

    #[test]
    fn test_infer_array_schema() {
        let json = serde_json::json!([1, 2, 3]);
        let schema = infer_schema(&json);
        if let SchemaKind::Sequence(inner) = &schema.kind {
            assert!(matches!(inner.kind, SchemaKind::Int { .. }));
        } else {
            assert!(false, "Expected Sequence schema");
        }
    }

    #[test]
    fn test_infer_record_schema() {
        let json = serde_json::json!({"drug": "aspirin", "cases": 100});
        let schema = infer_schema(&json);
        if let SchemaKind::Record(fields) = &schema.kind {
            assert_eq!(fields.len(), 2);
        } else {
            assert!(false, "Expected Record schema");
        }
    }

    // ── Violation Synthesis Tests ────────────────────────────────────────

    #[test]
    fn test_synthesize_int_violations() {
        let schema = InferredSchema {
            name: Some("score".to_string()),
            kind: SchemaKind::Int { min: 0, max: 100 },
        };
        let violations = synthesize_violations(&schema);
        // Should have comments + violation expressions
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("violation")));
    }

    #[test]
    fn test_synthesize_string_violations() {
        let schema = InferredSchema {
            name: Some("name".to_string()),
            kind: SchemaKind::Str { max_len: 50 },
        };
        let violations = synthesize_violations(&schema);
        assert!(violations.iter().any(|v| v.contains("\"\"")));
    }

    #[test]
    fn test_synthesize_not_true_file() {
        let schema = InferredSchema {
            name: None,
            kind: SchemaKind::Record(vec![
                InferredSchema {
                    name: Some("prr".to_string()),
                    kind: SchemaKind::Float {
                        min: 0.0,
                        max: 10.0,
                    },
                },
                InferredSchema {
                    name: Some("cases".to_string()),
                    kind: SchemaKind::Int { min: 1, max: 1000 },
                },
            ]),
        };
        let file = synthesize_not_true_file(&schema, "signal_data");
        assert!(file.contains("signal_data.not.true"));
        assert!(file.contains("reverse transcriptase"));
        assert!(file.contains("violation"));
    }

    // ── Full Pipeline Tests ─────────────────────────────────────────────

    #[test]
    fn test_full_pipeline_simple() {
        let result = reverse_transcribe(r#"{"drug": "aspirin", "events": 42}"#);
        assert!(result.is_ok());
        let result = result.ok().unwrap_or_else(|| {
            // This branch should never execute but satisfies no-unwrap
            TranscriptionResult {
                source: String::new(),
                not_true: String::new(),
                schema: InferredSchema {
                    name: None,
                    kind: SchemaKind::Void,
                },
                program: Program { statements: vec![] },
            }
        });
        assert!(result.source.contains("drug"));
        assert!(result.source.contains("aspirin"));
        assert!(result.source.contains("42"));
    }

    #[test]
    fn test_full_pipeline_invalid_json() {
        let result = reverse_transcribe("not valid json {{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_full_pipeline_array() {
        let result = reverse_transcribe("[1, 2, 3]");
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(r.source.contains("σ["));
        }
    }

    // ── Emitter Tests ───────────────────────────────────────────────────

    #[test]
    fn test_emit_literal_void() {
        assert_eq!(emit_literal(&Literal::Void), "∅");
    }

    #[test]
    fn test_emit_literal_symbol() {
        assert_eq!(emit_literal(&Literal::Symbol("foo".into())), ":foo");
    }

    #[test]
    fn test_emit_binop_all() {
        assert_eq!(emit_binop(&BinOp::Add), "+");
        assert_eq!(emit_binop(&BinOp::Sub), "-");
        assert_eq!(emit_binop(&BinOp::Mul), "*");
        assert_eq!(emit_binop(&BinOp::Div), "/");
        assert_eq!(emit_binop(&BinOp::Eq), "==");
        assert_eq!(emit_binop(&BinOp::Lt), "<");
        assert_eq!(emit_binop(&BinOp::Gt), ">");
        assert_eq!(emit_binop(&BinOp::And), "&&");
        assert_eq!(emit_binop(&BinOp::Or), "||");
    }

    // ── Sanitizer Tests ─────────────────────────────────────────────────

    #[test]
    fn test_sanitize_normal() {
        assert_eq!(sanitize_identifier("hello"), "hello");
    }

    #[test]
    fn test_sanitize_special_chars() {
        assert_eq!(sanitize_identifier("hello-world"), "hello_world");
    }

    #[test]
    fn test_sanitize_leading_digit() {
        let result = sanitize_identifier("3cats");
        assert!(!result.starts_with('3'));
        assert!(result.contains("3cats"));
    }

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_identifier(""), "_unnamed");
    }

    // ── Round-Trip Verification ─────────────────────────────────────────

    #[test]
    fn test_roundtrip_int() {
        // Reverse transcribe, then forward-evaluate
        let json = serde_json::json!(42);
        let source = emit_expr(&transcribe_json(&json));
        let result = crate::eval(&source);
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, crate::value::Value::int(42));
        }
    }

    #[test]
    fn test_roundtrip_bool() {
        let json = serde_json::json!(true);
        let source = emit_expr(&transcribe_json(&json));
        let result = crate::eval(&source);
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, crate::value::Value::bool(true));
        }
    }

    #[test]
    fn test_roundtrip_string() {
        let json = serde_json::json!("hello");
        let source = emit_expr(&transcribe_json(&json));
        let result = crate::eval(&source);
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, crate::value::Value::string("hello"));
        }
    }

    #[test]
    fn test_roundtrip_sequence() {
        let json = serde_json::json!([1, 2, 3]);
        let source = emit_expr(&transcribe_json(&json));
        let result = crate::eval(&source);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════
    // Engine Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_engine_new_default() {
        let engine = TranscriptionEngine::new();
        assert_eq!(engine.config().emit_mode, EmitMode::Bindings);
        assert!(engine.config().synthesize_violations);
        assert!(!engine.config().verify_roundtrip);
        assert_eq!(engine.stats().reverse_count, 0);
    }

    #[test]
    fn test_engine_reverse_simple() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.reverse(r#"{"x": 42}"#);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(r.source.contains("42"));
            assert!(!r.not_true.is_empty());
        }
        assert_eq!(engine.stats().reverse_count, 1);
    }

    #[test]
    fn test_engine_reverse_expression_mode() {
        let config = EngineConfig {
            emit_mode: EmitMode::Expression,
            ..EngineConfig::default()
        };
        let mut engine = TranscriptionEngine::with_config(config);
        let result = engine.reverse(r#"{"x": 42}"#);
        assert!(result.is_ok());
        if let Ok(r) = result {
            // Expression mode emits mapping, not let-bindings
            assert!(r.source.contains("μ("));
        }
    }

    #[test]
    fn test_engine_reverse_no_violations() {
        let config = EngineConfig {
            synthesize_violations: false,
            ..EngineConfig::default()
        };
        let mut engine = TranscriptionEngine::with_config(config);
        let result = engine.reverse(r#"42"#);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(r.not_true.is_empty());
        }
    }

    #[test]
    fn test_engine_forward() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.forward("1 + 2");
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, crate::value::Value::int(3));
        }
        assert_eq!(engine.stats().forward_count, 1);
    }

    #[test]
    fn test_engine_forward_error() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.forward("undefined_var");
        assert!(result.is_err());
        assert_eq!(engine.stats().forward_count, 1);
    }

    #[test]
    fn test_engine_roundtrip_exact_int() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.roundtrip("42");
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.fidelity, RoundtripFidelity::Exact);
            assert_eq!(r.source, "42");
        }
        assert_eq!(engine.stats().roundtrip_count, 1);
        assert_eq!(engine.stats().roundtrip_pass, 1);
    }

    #[test]
    fn test_engine_roundtrip_exact_bool() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.roundtrip("true");
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.fidelity, RoundtripFidelity::Exact);
        }
    }

    #[test]
    fn test_engine_roundtrip_exact_string() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.roundtrip(r#""hello""#);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.fidelity, RoundtripFidelity::Exact);
        }
    }

    #[test]
    fn test_engine_roundtrip_exact_null() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.roundtrip("null");
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.fidelity, RoundtripFidelity::Exact);
        }
    }

    #[test]
    fn test_engine_roundtrip_exact_array() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.roundtrip("[1, 2, 3]");
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.fidelity, RoundtripFidelity::Exact);
        }
    }

    #[test]
    fn test_engine_roundtrip_invalid_json() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.roundtrip("{invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_observe_single() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.observe(r#"{"age": 25}"#);
        assert!(result.is_ok());
        assert_eq!(engine.stats().records_observed, 1);
        assert!(engine.merged_schema().is_some());
    }

    #[test]
    fn test_engine_observe_merge_int_ranges() {
        let mut engine = TranscriptionEngine::new();
        assert!(engine.observe(r#"{"score": 10}"#).is_ok());
        assert!(engine.observe(r#"{"score": 90}"#).is_ok());
        assert_eq!(engine.stats().records_observed, 2);

        if let Some(schema) = engine.merged_schema() {
            if let SchemaKind::Record(fields) = &schema.kind {
                assert_eq!(fields.len(), 1);
                if let SchemaKind::Int { min, max } = &fields[0].kind {
                    assert_eq!(*min, 10);
                    assert_eq!(*max, 90);
                } else {
                    assert!(false, "Expected Int schema after merge");
                }
            } else {
                assert!(false, "Expected Record schema");
            }
        }
    }

    #[test]
    fn test_engine_observe_merge_adds_fields() {
        let mut engine = TranscriptionEngine::new();
        assert!(engine.observe(r#"{"name": "alice"}"#).is_ok());
        assert!(engine.observe(r#"{"age": 30}"#).is_ok());

        if let Some(schema) = engine.merged_schema() {
            if let SchemaKind::Record(fields) = &schema.kind {
                // Should have both "name" and "age"
                assert_eq!(fields.len(), 2);
            } else {
                assert!(false, "Expected Record schema");
            }
        }
    }

    #[test]
    fn test_engine_synthesize_merged_violations() {
        let mut engine = TranscriptionEngine::new();
        assert!(engine.observe(r#"{"prr": 2.5}"#).is_ok());
        let violations = engine.synthesize_merged_violations();
        assert!(violations.is_some());
        if let Some(v) = violations {
            assert!(v.contains("not.true"));
        }
    }

    #[test]
    fn test_engine_synthesize_merged_none_before_observe() {
        let engine = TranscriptionEngine::new();
        assert!(engine.synthesize_merged_violations().is_none());
    }

    #[test]
    fn test_engine_batch_array() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.batch(r#"[{"a": 1}, {"a": 2}, {"a": 3}]"#);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.items.len(), 3);
            // Merged schema should have widened range
            if let SchemaKind::Record(fields) = &r.merged_schema.kind {
                if let SchemaKind::Int { min, max } = &fields[0].kind {
                    assert_eq!(*min, 1);
                    assert_eq!(*max, 3);
                }
            }
        }
        assert_eq!(engine.stats().batch_count, 1);
        assert_eq!(engine.stats().reverse_count, 3);
    }

    #[test]
    fn test_engine_batch_single_record() {
        let mut engine = TranscriptionEngine::new();
        let result = engine.batch(r#"{"x": 42}"#);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.items.len(), 1);
        }
    }

    #[test]
    fn test_engine_reset() {
        let mut engine = TranscriptionEngine::new();
        assert!(engine.reverse("42").is_ok());
        assert!(engine.observe(r#"{"x": 1}"#).is_ok());
        assert_eq!(engine.stats().reverse_count, 1);
        assert!(engine.merged_schema().is_some());

        engine.reset();
        assert_eq!(engine.stats().reverse_count, 0);
        assert!(engine.merged_schema().is_none());
    }

    #[test]
    fn test_engine_stats_display() {
        let engine = TranscriptionEngine::new();
        let display = format!("{}", engine.stats());
        assert!(display.contains("TranscriptionEngine"));
        assert!(display.contains("Reverse"));
        assert!(display.contains("Forward"));
    }

    #[test]
    fn test_fidelity_display() {
        assert_eq!(
            format!("{}", RoundtripFidelity::Exact),
            "EXACT (f⁻¹∘f = id)"
        );
        assert_eq!(
            format!("{}", RoundtripFidelity::Approximate),
            "APPROX (f⁻¹∘f ≈ id)"
        );
        assert_eq!(
            format!("{}", RoundtripFidelity::Failed),
            "FAILED (f⁻¹∘f ≠ id)"
        );
    }

    // ── Schema Merge Tests ──────────────────────────────────────────────

    #[test]
    fn test_merge_int_ranges() {
        let a = InferredSchema {
            name: Some("x".to_string()),
            kind: SchemaKind::Int { min: 0, max: 10 },
        };
        let b = InferredSchema {
            name: Some("x".to_string()),
            kind: SchemaKind::Int { min: -5, max: 20 },
        };
        let merged = merge_schemas(&a, &b);
        if let SchemaKind::Int { min, max } = merged.kind {
            assert_eq!(min, -5);
            assert_eq!(max, 20);
        } else {
            assert!(false, "Expected Int after merge");
        }
    }

    #[test]
    fn test_merge_str_max_len() {
        let a = InferredSchema {
            name: None,
            kind: SchemaKind::Str { max_len: 10 },
        };
        let b = InferredSchema {
            name: None,
            kind: SchemaKind::Str { max_len: 50 },
        };
        let merged = merge_schemas(&a, &b);
        if let SchemaKind::Str { max_len } = merged.kind {
            assert_eq!(max_len, 50);
        } else {
            assert!(false, "Expected Str after merge");
        }
    }

    #[test]
    fn test_merge_int_float_widens() {
        let a = InferredSchema {
            name: None,
            kind: SchemaKind::Int { min: 0, max: 100 },
        };
        let b = InferredSchema {
            name: None,
            kind: SchemaKind::Float { min: 0.0, max: 1.0 },
        };
        let merged = merge_schemas(&a, &b);
        assert!(matches!(merged.kind, SchemaKind::Float { .. }));
    }

    #[test]
    fn test_merge_incompatible_becomes_mixed() {
        let a = InferredSchema {
            name: None,
            kind: SchemaKind::Bool,
        };
        let b = InferredSchema {
            name: None,
            kind: SchemaKind::Str { max_len: 5 },
        };
        let merged = merge_schemas(&a, &b);
        assert!(matches!(merged.kind, SchemaKind::Mixed));
    }

    // ── Fidelity Check Tests ────────────────────────────────────────────

    #[test]
    fn test_fidelity_null() {
        let json = JsonValue::Null;
        let value = crate::value::Value::void();
        assert_eq!(check_fidelity(&json, &value), RoundtripFidelity::Exact);
    }

    #[test]
    fn test_fidelity_int() {
        let json = serde_json::json!(42);
        let value = crate::value::Value::int(42);
        assert_eq!(check_fidelity(&json, &value), RoundtripFidelity::Exact);
    }

    #[test]
    fn test_fidelity_mismatch() {
        let json = serde_json::json!(42);
        let value = crate::value::Value::string("not a number");
        assert_eq!(check_fidelity(&json, &value), RoundtripFidelity::Failed);
    }
}
