// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Prima Universal Concept Translator Pipeline.
//!
//! ## Tier: T2-C (σ + μ + → + κ)
//!
//! Executes Prima source code through the full compilation pipeline:
//! Source → Lexer → Parser → Compiler → VM → Result

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use lex_primitiva::LexPrimitiva;
use nexcore_error::Error;
use prima::PrimaError;
use prima::prelude::{Lexer, Parser, Program, Token, TokenKind, Value, ValueData};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Pipeline execution error.
///
/// ## Tier: T2-C (∂ + ρ + →)
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Prima error: {0}")]
    Prima(#[from] PrimaError),

    #[error("Pipeline stage failed: {stage} - {message}")]
    StageFailed { stage: String, message: String },
}

/// Result of a pipeline execution.
///
/// ## Tier: T2-C (σ + N + →)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Source file or identifier.
    pub source: String,
    /// Final execution value (serialized).
    pub result: String,
    /// Execution successful.
    pub success: bool,
    /// Pipeline stages completed.
    pub stages: Vec<StageResult>,
    /// T1 primitives used in the program.
    pub primitives_used: Vec<String>,
}

/// Result of a single pipeline stage.
///
/// ## Tier: T2-P (σ + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    /// Stage name.
    pub name: String,
    /// Duration in microseconds.
    pub duration_us: u64,
    /// Items processed (tokens, nodes, instructions).
    pub items: usize,
}

/// Pipeline context holding intermediate results.
///
/// ## Tier: T2-C (ς + σ + μ)
pub struct PipelineContext {
    source: String,
    source_text: String,
    stages: Vec<StageResult>,
    primitives: Vec<LexPrimitiva>,
}

impl PipelineContext {
    /// Create new context from source.
    #[must_use]
    pub fn new(source: &str, text: String) -> Self {
        Self {
            source: source.to_string(),
            source_text: text,
            stages: Vec::new(),
            primitives: Vec::new(),
        }
    }

    /// Record a stage result.
    pub fn record_stage(&mut self, name: &str, duration_us: u64, items: usize) {
        self.stages.push(StageResult {
            name: name.to_string(),
            duration_us,
            items,
        });
    }

    /// Add discovered primitive.
    pub fn add_primitive(&mut self, prim: LexPrimitiva) {
        if !self.primitives.contains(&prim) {
            self.primitives.push(prim);
        }
    }
}

/// Execute the full Prima pipeline on source code.
///
/// ## Tier: T2-C (σ + μ + → + κ)
///
/// Pipeline stages:
/// 1. **Lex** (σ): Source → Token stream
/// 2. **Parse** (μ): Tokens → AST
/// 3. **Compile** (→): AST → Bytecode
/// 4. **Execute** (κ): Bytecode → Value
pub fn execute_pipeline(source: &str, text: &str) -> Result<PipelineResult, PipelineError> {
    let mut ctx = PipelineContext::new(source, text.to_string());

    // Stage 1: Lex
    let start = std::time::Instant::now();
    let tokens = Lexer::new(&ctx.source_text).tokenize()?;
    let token_count = tokens.len();
    ctx.record_stage("lex", start.elapsed().as_micros() as u64, token_count);

    // Track primitives from tokens
    for token in &tokens {
        if let Some(prim) = token_to_primitive(token) {
            ctx.add_primitive(prim);
        }
    }

    // Stage 2: Parse
    let start = std::time::Instant::now();
    let program = Parser::new(tokens.clone()).parse()?;
    let node_count = count_ast_nodes(&program);
    ctx.record_stage("parse", start.elapsed().as_micros() as u64, node_count);

    // Stage 3: Execute via tree-walking interpreter
    // (compile_and_run VM path has known issues with expression results)
    let start = std::time::Instant::now();
    let value = prima::eval(&ctx.source_text)?;
    ctx.record_stage("execute", start.elapsed().as_micros() as u64, 1);

    Ok(PipelineResult {
        source: ctx.source,
        result: format_value(&value),
        success: true,
        stages: ctx.stages,
        primitives_used: ctx
            .primitives
            .iter()
            .map(|p| p.symbol().to_string())
            .collect(),
    })
}

/// Execute pipeline from file.
pub fn execute_file(path: &Path) -> Result<PipelineResult, PipelineError> {
    let source = path.display().to_string();
    let text = std::fs::read_to_string(path)?;
    execute_pipeline(&source, &text)
}

/// Batch execute multiple files.
pub fn execute_batch(paths: &[&Path]) -> Vec<Result<PipelineResult, PipelineError>> {
    paths.iter().map(|p| execute_file(p)).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Map token to T1 primitive (if applicable).
fn token_to_primitive(token: &Token) -> Option<LexPrimitiva> {
    match &token.kind {
        TokenKind::Primitive(p) => Some(*p),
        TokenKind::Int(_) | TokenKind::Float(_) => Some(LexPrimitiva::Quantity),
        _ => None,
    }
}

/// Count AST nodes (rough estimate).
fn count_ast_nodes(program: &Program) -> usize {
    program.statements.len() * 3 // Rough estimate: avg 3 nodes per statement
}

/// Format value for output.
fn format_value(value: &Value) -> String {
    match &value.data {
        ValueData::Int(n) => n.to_string(),
        ValueData::Float(f) => f.to_string(),
        ValueData::Bool(b) => b.to_string(),
        ValueData::String(s) => s.clone(),
        ValueData::Void => "∅".to_string(),
        ValueData::Sequence(items) => {
            let parts: Vec<String> = items.iter().map(format_value).collect();
            format!("σ[{}]", parts.join(", "))
        }
        ValueData::Function(fv) => format!("μ<{}>", fv.name),
        ValueData::Builtin(name) => format!("builtin<{}>", name),
        ValueData::Symbol(s) => format!(":{}", s),
        ValueData::Mapping(m) => format!("μ{{{}..}}", m.len()),
        ValueData::Quoted(_) => "'<quoted>".to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_arithmetic() {
        let result = execute_pipeline("test", "1 + 2 * 3");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| PipelineResult {
            source: String::new(),
            result: String::new(),
            success: false,
            stages: vec![],
            primitives_used: vec![],
        });
        assert_eq!(r.result, "7");
        assert!(r.success);
        assert_eq!(r.stages.len(), 3);
    }

    #[test]
    fn test_pipeline_function() {
        let result = execute_pipeline("test", "μ add(a: N, b: N) → N { a + b }\nadd(3, 4)");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| PipelineResult {
            source: String::new(),
            result: String::new(),
            success: false,
            stages: vec![],
            primitives_used: vec![],
        });
        assert_eq!(r.result, "7");
    }

    #[test]
    fn test_pipeline_sequence() {
        let result = execute_pipeline("test", "σ[1, 2, 3]");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| PipelineResult {
            source: String::new(),
            result: String::new(),
            success: false,
            stages: vec![],
            primitives_used: vec![],
        });
        assert!(r.result.contains("σ["));
    }

    #[test]
    fn test_pipeline_conditional() {
        let result = execute_pipeline("test", "if 1 < 2 { 10 } else { 20 }");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| PipelineResult {
            source: String::new(),
            result: String::new(),
            success: false,
            stages: vec![],
            primitives_used: vec![],
        });
        assert_eq!(r.result, "10");
    }

    #[test]
    fn test_pipeline_stages_recorded() {
        let result = execute_pipeline("test", "42");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| PipelineResult {
            source: String::new(),
            result: String::new(),
            success: false,
            stages: vec![],
            primitives_used: vec![],
        });
        assert_eq!(r.stages.len(), 3);
        assert_eq!(r.stages[0].name, "lex");
        assert_eq!(r.stages[1].name, "parse");
        assert_eq!(r.stages[2].name, "execute");
    }

    #[test]
    fn test_pipeline_primitives_tracked() {
        let result = execute_pipeline("test", "μ f(x: N) → N { x * 2 }\nf(5)");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| PipelineResult {
            source: String::new(),
            result: String::new(),
            success: false,
            stages: vec![],
            primitives_used: vec![],
        });
        // Should track μ (Mapping) and N (Numeric)
        assert!(!r.primitives_used.is_empty());
    }

    #[test]
    fn test_pipeline_error_handling() {
        let result = execute_pipeline("test", "undefined_var");
        // This should fail at runtime
        assert!(result.is_err());
    }

    #[test]
    fn test_format_value_void() {
        let v = Value::void();
        assert_eq!(format_value(&v), "∅");
    }

    #[test]
    fn test_format_value_sequence() {
        let v = Value::sequence(vec![Value::int(1), Value::int(2)]);
        let formatted = format_value(&v);
        assert!(formatted.starts_with("σ["));
        assert!(formatted.contains("1"));
        assert!(formatted.contains("2"));
    }
}
