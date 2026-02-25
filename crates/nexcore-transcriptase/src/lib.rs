// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # NexVigilant Core — transcriptase — Bidirectional Data ↔ Schema Engine
//!
//! Reverse transcriptase for structured data: observes JSON records,
//! infers schemas, merges observations, synthesizes boundary violations,
//! and verifies round-trip fidelity.
//!
//! ## Biological Analogy
//!
//! In biology, reverse transcriptase synthesizes DNA from RNA.
//! This engine synthesizes structural knowledge (schema) from
//! observed data (JSON), then uses that knowledge to generate
//! boundary assertions (violations).
//!
//! ## Pipeline
//!
//! ```text
//! JSON records → Schema Inference → Schema Merging → Violation Synthesis
//!      ↓               ↓                  ↓                  ↓
//!   observe()    InferredSchema     merge_schemas()    Violations
//! ```
//!
//! ## Tier: T2-C (κ + σ + μ + ∂)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "Schema inference engine prioritizes explicit deterministic logic over style-only lint constraints"
)]

pub mod grounding;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ─── Error Types ────────────────────────────────────────────────────────────

/// Transcriptase error type.
#[derive(Debug, nexcore_error::Error)]
pub enum TranscriptaseError {
    #[error("∂[json]: {0}")]
    Json(#[from] serde_json::Error),

    #[error("∂[io]: {0}")]
    Io(#[from] std::io::Error),

    #[error("∂[engine]: {0}")]
    Engine(String),
}

pub type Result<T> = std::result::Result<T, TranscriptaseError>;

// ─── Schema Types ───────────────────────────────────────────────────────────

/// Inferred schema from observed data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Field name (if from an object key).
    pub name: Option<String>,
    /// The inferred type.
    pub kind: SchemaKind,
    /// Number of observations that contributed to this schema.
    pub observations: usize,
}

/// What type was observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaKind {
    /// Null/void.
    Null,
    /// Boolean.
    Bool {
        true_count: usize,
        false_count: usize,
    },
    /// Integer with observed range.
    Int { min: i64, max: i64, sum: i64 },
    /// Float with observed range.
    Float { min: f64, max: f64, sum: f64 },
    /// String with length statistics.
    Str {
        min_len: usize,
        max_len: usize,
        unique_count: usize,
    },
    /// Array with element schema.
    Array {
        element: Box<Schema>,
        min_len: usize,
        max_len: usize,
    },
    /// Object with field schemas.
    Record(BTreeMap<String, Schema>),
    /// Mixed types observed (fallback).
    Mixed,
}

impl Schema {
    fn new(name: Option<String>, kind: SchemaKind) -> Self {
        Self {
            name,
            kind,
            observations: 1,
        }
    }
}

// ─── Schema Inference ───────────────────────────────────────────────────────

/// Infer a schema from a single JSON value.
pub fn infer(json: &serde_json::Value) -> Schema {
    infer_named(json, None)
}

fn infer_named(json: &serde_json::Value, name: Option<String>) -> Schema {
    use serde_json::Value;
    let kind = match json {
        Value::Null => SchemaKind::Null,
        Value::Bool(b) => SchemaKind::Bool {
            true_count: usize::from(*b),
            false_count: usize::from(!*b),
        },
        Value::Number(n) => n.as_i64().map_or_else(
            || {
                n.as_f64().map_or(SchemaKind::Mixed, |f| SchemaKind::Float {
                    min: f,
                    max: f,
                    sum: f,
                })
            },
            |i| SchemaKind::Int {
                min: i,
                max: i,
                sum: i,
            },
        ),
        Value::String(s) => SchemaKind::Str {
            min_len: s.len(),
            max_len: s.len(),
            unique_count: 1,
        },
        Value::Array(arr) => {
            let element = if arr.is_empty() {
                Schema::new(None, SchemaKind::Null)
            } else {
                // Infer from first, merge rest
                let mut schema = infer(&arr[0]);
                for item in arr.iter().skip(1) {
                    let other = infer(item);
                    schema = merge(&schema, &other);
                }
                schema
            };
            SchemaKind::Array {
                element: Box::new(element),
                min_len: arr.len(),
                max_len: arr.len(),
            }
        }
        Value::Object(obj) => {
            let fields: BTreeMap<String, Schema> = obj
                .iter()
                .map(|(k, v)| (k.clone(), infer_named(v, Some(k.clone()))))
                .collect();
            SchemaKind::Record(fields)
        }
    };
    Schema::new(name, kind)
}

// ─── Schema Merging ─────────────────────────────────────────────────────────

/// Merge two schemas, widening ranges and unioning fields.
///
/// This is how the engine learns from multiple observations:
/// - Int ranges widen to encompass all observed values
/// - Float ranges widen similarly, with sum accumulation
/// - String lengths track min/max
/// - Records union their field sets
/// - Incompatible types become Mixed
pub fn merge(a: &Schema, b: &Schema) -> Schema {
    let name = a.name.clone().or_else(|| b.name.clone());
    let kind = merge_kinds(&a.kind, &b.kind);
    Schema {
        name,
        kind,
        observations: a.observations + b.observations,
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "Exhaustive SchemaKind merge matrix is clearest as one match"
)]
fn merge_kinds(a: &SchemaKind, b: &SchemaKind) -> SchemaKind {
    match (a, b) {
        (SchemaKind::Null, SchemaKind::Null) => SchemaKind::Null,

        (
            SchemaKind::Bool {
                true_count: at,
                false_count: af,
            },
            SchemaKind::Bool {
                true_count: bt,
                false_count: bf,
            },
        ) => SchemaKind::Bool {
            true_count: at + bt,
            false_count: af + bf,
        },

        (
            SchemaKind::Int {
                min: a_min,
                max: a_max,
                sum: a_sum,
            },
            SchemaKind::Int {
                min: b_min,
                max: b_max,
                sum: b_sum,
            },
        ) => SchemaKind::Int {
            min: (*a_min).min(*b_min),
            max: (*a_max).max(*b_max),
            sum: a_sum.saturating_add(*b_sum),
        },

        (
            SchemaKind::Float {
                min: a_min,
                max: a_max,
                sum: a_sum,
            },
            SchemaKind::Float {
                min: b_min,
                max: b_max,
                sum: b_sum,
            },
        ) => SchemaKind::Float {
            min: a_min.min(*b_min),
            max: a_max.max(*b_max),
            sum: a_sum + b_sum,
        },

        (
            SchemaKind::Str {
                min_len: a_min,
                max_len: a_max,
                unique_count: au,
            },
            SchemaKind::Str {
                min_len: b_min,
                max_len: b_max,
                unique_count: bu,
            },
        ) => SchemaKind::Str {
            min_len: (*a_min).min(*b_min),
            max_len: (*a_max).max(*b_max),
            unique_count: au + bu, // approximate
        },

        (
            SchemaKind::Array {
                element: a_elem,
                min_len: a_min,
                max_len: a_max,
            },
            SchemaKind::Array {
                element: b_elem,
                min_len: b_min,
                max_len: b_max,
            },
        ) => SchemaKind::Array {
            element: Box::new(merge(a_elem, b_elem)),
            min_len: (*a_min).min(*b_min),
            max_len: (*a_max).max(*b_max),
        },

        (SchemaKind::Record(a_fields), SchemaKind::Record(b_fields)) => {
            let mut merged = a_fields.clone();
            for (key, b_schema) in b_fields {
                let entry = merged
                    .entry(key.clone())
                    .and_modify(|existing| *existing = merge(existing, b_schema));
                // Insert if new
                if !a_fields.contains_key(key) {
                    entry.or_insert_with(|| b_schema.clone());
                }
            }
            SchemaKind::Record(merged)
        }

        // Int + Float → Float (widening), order-independent
        #[allow(
            clippy::cast_precision_loss,
            reason = "Range tracking intentionally widens i64 into f64 for merged bounds"
        )]
        (
            SchemaKind::Int {
                min: imin,
                max: imax,
                sum: isum,
            },
            SchemaKind::Float {
                min: fmin,
                max: fmax,
                sum: fsum,
            },
        )
        | (
            SchemaKind::Float {
                min: fmin,
                max: fmax,
                sum: fsum,
            },
            SchemaKind::Int {
                min: imin,
                max: imax,
                sum: isum,
            },
        ) => SchemaKind::Float {
            min: fmin.min(*imin as f64),
            max: fmax.max(*imax as f64),
            sum: fsum + *isum as f64,
        },

        _ => SchemaKind::Mixed,
    }
}

// ─── Violation Synthesis ────────────────────────────────────────────────────

/// A schema boundary violation — something that SHOULD NOT be true.
///
/// Tier: T2-C (∂ + κ — boundary assertion with comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaViolation {
    /// Human-readable description.
    pub description: String,
    /// The assertion expression (in a not-true format).
    pub assertion: String,
    /// Which field this applies to.
    pub field: Option<String>,
    /// Diagnostic level: how dangerous is this boundary crossing?
    pub severity: DiagnosticLevel,
}

/// Backward-compatible alias.
#[deprecated(note = "use SchemaViolation — F2 equivocation fix")]
pub type Violation = SchemaViolation;

/// Classification of validation/schema issue severity.
///
/// Tier: T2-P (κ + ∂ — comparison with boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    /// Structural impossibility (e.g., negative count).
    Critical,
    /// Out of observed range.
    Warning,
    /// Informational boundary.
    Info,
}

/// Backward-compatible alias.
#[deprecated(note = "use DiagnosticLevel — F2 equivocation fix")]
pub type Severity = DiagnosticLevel;

/// Synthesize boundary violations from a schema.
///
/// For each field, generates assertions about what SHOULD NOT hold:
/// values outside observed ranges, empty where non-empty was seen, etc.
pub fn synthesize_violations(schema: &Schema) -> Vec<SchemaViolation> {
    let mut violations = Vec::new();
    synthesize_inner(schema, &mut violations, "");
    violations
}

#[allow(
    clippy::too_many_lines,
    reason = "Violation synthesis is intentionally explicit per variant and boundary rule"
)]
fn synthesize_inner(schema: &Schema, violations: &mut Vec<SchemaViolation>, prefix: &str) {
    let path = match &schema.name {
        Some(n) if prefix.is_empty() => n.clone(),
        Some(n) => format!("{prefix}.{n}"),
        None => prefix.to_string(),
    };

    match &schema.kind {
        SchemaKind::Int { min, max, .. } => {
            if *min >= 0 {
                violations.push(SchemaViolation {
                    description: format!("{path}: negative values never observed"),
                    assertion: format!("{path} < 0"),
                    field: Some(path.clone()),
                    severity: DiagnosticLevel::Critical,
                });
            }
            violations.push(SchemaViolation {
                description: format!("{path}: below observed minimum ({min})"),
                assertion: format!("{path} < {min}"),
                field: Some(path.clone()),
                severity: DiagnosticLevel::Warning,
            });
            violations.push(SchemaViolation {
                description: format!("{path}: above observed maximum ({max})"),
                assertion: format!("{path} > {max}"),
                field: Some(path),
                severity: DiagnosticLevel::Warning,
            });
        }
        SchemaKind::Float { min, max, .. } => {
            violations.push(SchemaViolation {
                description: format!("{path}: below observed minimum ({min})"),
                assertion: format!("{path} < {min}"),
                field: Some(path.clone()),
                severity: DiagnosticLevel::Warning,
            });
            violations.push(SchemaViolation {
                description: format!("{path}: above observed maximum ({max})"),
                assertion: format!("{path} > {max}"),
                field: Some(path.clone()),
                severity: DiagnosticLevel::Warning,
            });
            violations.push(SchemaViolation {
                description: format!("{path}: NaN is always invalid"),
                assertion: format!("{path} == NaN"),
                field: Some(path),
                severity: DiagnosticLevel::Critical,
            });
        }
        SchemaKind::Str {
            min_len, max_len, ..
        } => {
            if *min_len > 0 {
                violations.push(SchemaViolation {
                    description: format!("{path}: empty string never observed"),
                    assertion: format!("len({path}) == 0"),
                    field: Some(path.clone()),
                    severity: DiagnosticLevel::Warning,
                });
            }
            violations.push(SchemaViolation {
                description: format!("{path}: exceeds observed max length ({max_len})"),
                assertion: format!("len({path}) > {max_len}"),
                field: Some(path),
                severity: DiagnosticLevel::Info,
            });
        }
        SchemaKind::Array {
            element,
            min_len,
            max_len,
        } => {
            if *min_len > 0 {
                violations.push(SchemaViolation {
                    description: format!("{path}: empty array never observed"),
                    assertion: format!("len({path}) == 0"),
                    field: Some(path.clone()),
                    severity: DiagnosticLevel::Warning,
                });
            }
            violations.push(SchemaViolation {
                description: format!("{path}: exceeds observed max length ({max_len})"),
                assertion: format!("len({path}) > {max_len}"),
                field: Some(path.clone()),
                severity: DiagnosticLevel::Info,
            });
            synthesize_inner(element, violations, &format!("{path}[]"));
        }
        SchemaKind::Record(fields) => {
            for field_schema in fields.values() {
                synthesize_inner(field_schema, violations, &path);
            }
        }
        SchemaKind::Bool { .. } => {
            violations.push(SchemaViolation {
                description: format!("{path}: null where boolean expected"),
                assertion: format!("{path} == null"),
                field: Some(path),
                severity: DiagnosticLevel::Critical,
            });
        }
        SchemaKind::Null | SchemaKind::Mixed => {}
    }
}

// ─── Data Generation ─────────────────────────────────────────────────────────

/// Generate a synthetic JSON value that conforms to the observed schema.
///
/// Uses deterministic midpoints of observed ranges:
/// - Int → `(min + max) / 2`
/// - Float → `(min + max) / 2.0`
/// - Str → field name repeated to average length
/// - Bool → whichever value was observed more
/// - Array → `min_len` elements generated recursively
/// - Record → all fields generated recursively
/// - Null / Mixed → `Value::Null`
pub fn generate(schema: &Schema) -> serde_json::Value {
    use serde_json::Value;

    match &schema.kind {
        SchemaKind::Null | SchemaKind::Mixed => Value::Null,

        SchemaKind::Bool {
            true_count,
            false_count,
        } => Value::Bool(*true_count >= *false_count),

        SchemaKind::Int { min, max, .. } => {
            // Midpoint avoids overflow via (min + max) / 2 rewritten
            let mid = min / 2 + max / 2 + (min % 2 + max % 2) / 2;
            Value::Number(serde_json::Number::from(mid))
        }

        SchemaKind::Float { min, max, .. } => {
            let mid = (min + max) / 2.0;
            serde_json::Number::from_f64(mid).map_or(Value::Null, Value::Number)
        }

        SchemaKind::Str {
            min_len, max_len, ..
        } => {
            let avg_len = (min_len + max_len) / 2;
            let seed = schema.name.as_deref().unwrap_or("x");
            // Repeat the seed to fill the target length
            let generated: String = seed.chars().cycle().take(avg_len.max(1)).collect();
            Value::String(generated)
        }

        SchemaKind::Array {
            element, min_len, ..
        } => {
            let items: Vec<Value> = (0..*min_len).map(|_| generate(element)).collect();
            Value::Array(items)
        }

        SchemaKind::Record(fields) => {
            let map: serde_json::Map<String, Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), generate(v)))
                .collect();
            Value::Object(map)
        }
    }
}

// ─── Fidelity ───────────────────────────────────────────────────────────────

/// Round-trip fidelity result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fidelity {
    /// Perfect: serialize(deserialize(data)) == data
    Exact,
    /// Structural match but possible precision loss.
    Approximate,
    /// Round-trip failed.
    Failed,
}

/// Check JSON round-trip fidelity: serialize → deserialize → compare.
pub fn check_fidelity(original: &serde_json::Value) -> Fidelity {
    let serialized = serde_json::to_string(original);
    serialized.map_or(Fidelity::Failed, |s| {
        serde_json::from_str::<serde_json::Value>(&s).map_or(Fidelity::Failed, |roundtripped| {
            if *original == roundtripped {
                Fidelity::Exact
            } else {
                Fidelity::Approximate
            }
        })
    })
}

impl std::fmt::Display for Fidelity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "EXACT"),
            Self::Approximate => write!(f, "APPROXIMATE"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

// ─── Engine ─────────────────────────────────────────────────────────────────

/// Engine configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to generate violations alongside schema.
    pub synthesize_violations: bool,
    /// Whether to verify round-trip fidelity.
    pub verify_fidelity: bool,
    /// Source name for output headers.
    pub source_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            synthesize_violations: true,
            verify_fidelity: false,
            source_name: "stdin".to_string(),
        }
    }
}

/// Engine statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub records_observed: usize,
    pub schemas_merged: usize,
    pub violations_generated: usize,
    pub fidelity_exact: usize,
    pub fidelity_approx: usize,
    pub fidelity_failed: usize,
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Transcriptase Engine Statistics")?;
        writeln!(f, "─────────────────────────────────────")?;
        writeln!(f, "  Records observed:     {}", self.records_observed)?;
        writeln!(f, "  Schemas merged:       {}", self.schemas_merged)?;
        writeln!(f, "  Violations generated: {}", self.violations_generated)?;
        writeln!(
            f,
            "  Fidelity: {} exact, {} approx, {} failed",
            self.fidelity_exact, self.fidelity_approx, self.fidelity_failed
        )?;
        Ok(())
    }
}

/// Output from a transcription run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionOutput {
    /// The inferred/merged schema.
    pub schema: Schema,
    /// Generated violations (if enabled).
    pub violations: Vec<SchemaViolation>,
    /// Per-record fidelity results (if enabled).
    pub fidelity: Vec<Fidelity>,
    /// Engine statistics.
    pub stats: Stats,
}

/// The Transcription Engine.
///
/// Observes data records, infers schemas, merges across observations,
/// synthesizes boundary violations, and tracks statistics.
pub struct Engine {
    config: Config,
    stats: Stats,
    merged: Option<Schema>,
}

impl Engine {
    /// Create a new engine with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            stats: Stats::default(),
            merged: None,
        }
    }

    /// Create a new engine with custom config.
    #[must_use]
    pub fn with_config(config: Config) -> Self {
        Self {
            config,
            stats: Stats::default(),
            merged: None,
        }
    }

    /// Get statistics.
    #[must_use]
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    /// Get the merged schema.
    #[must_use]
    pub fn schema(&self) -> Option<&Schema> {
        self.merged.as_ref()
    }

    /// Observe a single JSON record.
    pub fn observe(&mut self, json: &serde_json::Value) {
        let schema = infer(json);
        self.merged = Some(match self.merged.take() {
            Some(existing) => {
                self.stats.schemas_merged += 1;
                merge(&existing, &schema)
            }
            None => schema,
        });
        self.stats.records_observed += 1;

        if self.config.verify_fidelity {
            match check_fidelity(json) {
                Fidelity::Exact => self.stats.fidelity_exact += 1,
                Fidelity::Approximate => self.stats.fidelity_approx += 1,
                Fidelity::Failed => self.stats.fidelity_failed += 1,
            }
        }
    }

    /// Observe a JSON string (parses first).
    pub fn observe_str(&mut self, json_str: &str) -> Result<()> {
        let json: serde_json::Value = serde_json::from_str(json_str)?;
        self.observe(&json);
        Ok(())
    }

    /// Process a batch of records (JSON array or single record).
    pub fn process(&mut self, json_str: &str) -> Result<TranscriptionOutput> {
        let json: serde_json::Value = serde_json::from_str(json_str)?;

        let records: Vec<&serde_json::Value> = match &json {
            serde_json::Value::Array(arr) => arr.iter().collect(),
            other => vec![other],
        };

        let mut fidelity_results = Vec::new();

        for record in &records {
            self.observe(record);
            if self.config.verify_fidelity {
                fidelity_results.push(check_fidelity(record));
            }
        }

        let schema = self
            .merged
            .clone()
            .unwrap_or_else(|| Schema::new(None, SchemaKind::Null));

        let violations = if self.config.synthesize_violations {
            let v = synthesize_violations(&schema);
            self.stats.violations_generated += v.len();
            v
        } else {
            Vec::new()
        };

        Ok(TranscriptionOutput {
            schema,
            violations,
            fidelity: fidelity_results,
            stats: self.stats.clone(),
        })
    }

    /// Generate synthetic data from the current merged schema.
    ///
    /// Returns `None` if no data has been observed yet.
    /// Uses deterministic midpoints of all observed ranges.
    #[must_use]
    pub fn generate(&self) -> Option<serde_json::Value> {
        self.merged.as_ref().map(generate)
    }

    /// Generate `count` synthetic records from the current merged schema.
    ///
    /// Returns `None` if no data has been observed yet.
    #[must_use]
    pub fn generate_batch(&self, count: usize) -> Option<Vec<serde_json::Value>> {
        self.merged
            .as_ref()
            .map(|schema| (0..count).map(|_| generate(schema)).collect())
    }

    /// Reset engine state.
    pub fn reset(&mut self) {
        self.stats = Stats::default();
        self.merged = None;
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Display ────────────────────────────────────────────────────────────────

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

impl std::fmt::Display for SchemaViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.severity, self.description, self.assertion
        )
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Schema Inference ────────────────────────────────────────────────

    #[test]
    fn test_infer_null() {
        let schema = infer(&serde_json::json!(null));
        assert!(matches!(schema.kind, SchemaKind::Null));
    }

    #[test]
    fn test_infer_bool() {
        let schema = infer(&serde_json::json!(true));
        if let SchemaKind::Bool {
            true_count,
            false_count,
        } = schema.kind
        {
            assert_eq!(true_count, 1);
            assert_eq!(false_count, 0);
        } else {
            assert!(false, "Expected Bool");
        }
    }

    #[test]
    fn test_infer_int() {
        let schema = infer(&serde_json::json!(42));
        if let SchemaKind::Int { min, max, sum } = schema.kind {
            assert_eq!(min, 42);
            assert_eq!(max, 42);
            assert_eq!(sum, 42);
        } else {
            assert!(false, "Expected Int");
        }
    }

    #[test]
    fn test_infer_float() {
        let schema = infer(&serde_json::json!(3.14));
        assert!(matches!(schema.kind, SchemaKind::Float { .. }));
    }

    #[test]
    fn test_infer_string() {
        let schema = infer(&serde_json::json!("hello"));
        if let SchemaKind::Str {
            min_len, max_len, ..
        } = schema.kind
        {
            assert_eq!(min_len, 5);
            assert_eq!(max_len, 5);
        } else {
            assert!(false, "Expected Str");
        }
    }

    #[test]
    fn test_infer_array() {
        let schema = infer(&serde_json::json!([1, 2, 3]));
        if let SchemaKind::Array {
            min_len, max_len, ..
        } = schema.kind
        {
            assert_eq!(min_len, 3);
            assert_eq!(max_len, 3);
        } else {
            assert!(false, "Expected Array");
        }
    }

    #[test]
    fn test_infer_empty_array() {
        let schema = infer(&serde_json::json!([]));
        if let SchemaKind::Array {
            min_len, max_len, ..
        } = schema.kind
        {
            assert_eq!(min_len, 0);
            assert_eq!(max_len, 0);
        } else {
            assert!(false, "Expected Array");
        }
    }

    #[test]
    fn test_infer_record() {
        let schema = infer(&serde_json::json!({"drug": "aspirin", "cases": 42}));
        if let SchemaKind::Record(fields) = &schema.kind {
            assert_eq!(fields.len(), 2);
            assert!(fields.contains_key("drug"));
            assert!(fields.contains_key("cases"));
        } else {
            assert!(false, "Expected Record");
        }
    }

    #[test]
    fn test_infer_nested() {
        let schema = infer(&serde_json::json!({"signals": [1, 2, 3]}));
        if let SchemaKind::Record(fields) = &schema.kind {
            let signals = fields.get("signals");
            assert!(signals.is_some());
            if let Some(s) = signals {
                assert!(matches!(s.kind, SchemaKind::Array { .. }));
            }
        } else {
            assert!(false, "Expected Record");
        }
    }

    // ── Schema Merging ──────────────────────────────────────────────────

    #[test]
    fn test_merge_int_widens_range() {
        let a = infer(&serde_json::json!(10));
        let b = infer(&serde_json::json!(90));
        let merged = merge(&a, &b);
        if let SchemaKind::Int { min, max, sum } = merged.kind {
            assert_eq!(min, 10);
            assert_eq!(max, 90);
            assert_eq!(sum, 100);
        } else {
            assert!(false, "Expected Int");
        }
        assert_eq!(merged.observations, 2);
    }

    #[test]
    fn test_merge_str_widens_length() {
        let a = infer(&serde_json::json!("hi"));
        let b = infer(&serde_json::json!("hello world"));
        let merged = merge(&a, &b);
        if let SchemaKind::Str {
            min_len, max_len, ..
        } = merged.kind
        {
            assert_eq!(min_len, 2);
            assert_eq!(max_len, 11);
        } else {
            assert!(false, "Expected Str");
        }
    }

    #[test]
    fn test_merge_record_unions_fields() {
        let a = infer(&serde_json::json!({"name": "alice"}));
        let b = infer(&serde_json::json!({"age": 30}));
        let merged = merge(&a, &b);
        if let SchemaKind::Record(fields) = &merged.kind {
            assert_eq!(fields.len(), 2);
            assert!(fields.contains_key("name"));
            assert!(fields.contains_key("age"));
        } else {
            assert!(false, "Expected Record");
        }
    }

    #[test]
    fn test_merge_bool_accumulates() {
        let a = infer(&serde_json::json!(true));
        let b = infer(&serde_json::json!(false));
        let merged = merge(&a, &b);
        if let SchemaKind::Bool {
            true_count,
            false_count,
        } = merged.kind
        {
            assert_eq!(true_count, 1);
            assert_eq!(false_count, 1);
        } else {
            assert!(false, "Expected Bool");
        }
    }

    #[test]
    fn test_merge_int_float_widens() {
        let a = infer(&serde_json::json!(42));
        let b = infer(&serde_json::json!(3.14));
        let merged = merge(&a, &b);
        assert!(matches!(merged.kind, SchemaKind::Float { .. }));
    }

    #[test]
    fn test_merge_incompatible_becomes_mixed() {
        let a = infer(&serde_json::json!(true));
        let b = infer(&serde_json::json!("hello"));
        let merged = merge(&a, &b);
        assert!(matches!(merged.kind, SchemaKind::Mixed));
    }

    #[test]
    fn test_merge_array_element_schemas() {
        let a = infer(&serde_json::json!([1, 2]));
        let b = infer(&serde_json::json!([3, 4, 5]));
        let merged = merge(&a, &b);
        if let SchemaKind::Array {
            min_len,
            max_len,
            element,
        } = &merged.kind
        {
            assert_eq!(*min_len, 2);
            assert_eq!(*max_len, 3);
            // Element schema should be Int with widened range
            if let SchemaKind::Int { min, max, .. } = &element.kind {
                assert_eq!(*min, 1);
                assert_eq!(*max, 5);
            } else {
                assert!(false, "Expected Int element");
            }
        } else {
            assert!(false, "Expected Array");
        }
    }

    // ── Violation Synthesis ──────────────────────────────────────────────

    #[test]
    fn test_violations_int_range() {
        let schema = Schema {
            name: Some("score".into()),
            kind: SchemaKind::Int {
                min: 0,
                max: 100,
                sum: 500,
            },
            observations: 10,
        };
        let violations = synthesize_violations(&schema);
        assert!(violations.len() >= 2);
        assert!(violations.iter().any(|v| v.assertion.contains("> 100")));
        assert!(violations.iter().any(|v| v.assertion.contains("< 0")));
    }

    #[test]
    fn test_violations_string_empty() {
        let schema = Schema {
            name: Some("drug".into()),
            kind: SchemaKind::Str {
                min_len: 3,
                max_len: 50,
                unique_count: 10,
            },
            observations: 10,
        };
        let violations = synthesize_violations(&schema);
        assert!(violations.iter().any(|v| v.assertion.contains("== 0")));
    }

    #[test]
    fn test_violations_float_nan() {
        let schema = Schema {
            name: Some("prr".into()),
            kind: SchemaKind::Float {
                min: 0.0,
                max: 10.0,
                sum: 50.0,
            },
            observations: 10,
        };
        let violations = synthesize_violations(&schema);
        assert!(violations.iter().any(|v| v.assertion.contains("NaN")));
    }

    #[test]
    fn test_violations_record_recursive() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "count".to_string(),
            Schema {
                name: Some("count".into()),
                kind: SchemaKind::Int {
                    min: 1,
                    max: 1000,
                    sum: 5000,
                },
                observations: 10,
            },
        );
        let schema = Schema {
            name: None,
            kind: SchemaKind::Record(fields),
            observations: 10,
        };
        let violations = synthesize_violations(&schema);
        assert!(!violations.is_empty());
        assert!(
            violations
                .iter()
                .any(|v| v.field.as_deref() == Some("count"))
        );
    }

    #[test]
    fn test_violations_severity_levels() {
        let schema = Schema {
            name: Some("count".into()),
            kind: SchemaKind::Int {
                min: 0,
                max: 100,
                sum: 500,
            },
            observations: 10,
        };
        let violations = synthesize_violations(&schema);
        assert!(
            violations
                .iter()
                .any(|v| v.severity == DiagnosticLevel::Critical)
        );
        assert!(
            violations
                .iter()
                .any(|v| v.severity == DiagnosticLevel::Warning)
        );
    }

    // ── Fidelity ────────────────────────────────────────────────────────

    #[test]
    fn test_fidelity_exact() {
        assert_eq!(check_fidelity(&serde_json::json!(42)), Fidelity::Exact);
        assert_eq!(
            check_fidelity(&serde_json::json!({"a": 1})),
            Fidelity::Exact
        );
        assert_eq!(
            check_fidelity(&serde_json::json!([1, 2, 3])),
            Fidelity::Exact
        );
    }

    #[test]
    fn test_fidelity_null() {
        assert_eq!(check_fidelity(&serde_json::json!(null)), Fidelity::Exact);
    }

    #[test]
    fn test_fidelity_display() {
        assert_eq!(format!("{}", Fidelity::Exact), "EXACT");
        assert_eq!(format!("{}", Fidelity::Failed), "FAILED");
    }

    // ── Engine ──────────────────────────────────────────────────────────

    #[test]
    fn test_engine_default() {
        let engine = Engine::new();
        assert_eq!(engine.stats().records_observed, 0);
        assert!(engine.schema().is_none());
    }

    #[test]
    fn test_engine_observe() {
        let mut engine = Engine::new();
        engine.observe(&serde_json::json!({"x": 42}));
        assert_eq!(engine.stats().records_observed, 1);
        assert!(engine.schema().is_some());
    }

    #[test]
    fn test_engine_observe_str() {
        let mut engine = Engine::new();
        let result = engine.observe_str(r#"{"x": 42}"#);
        assert!(result.is_ok());
        assert_eq!(engine.stats().records_observed, 1);
    }

    #[test]
    fn test_engine_observe_str_invalid() {
        let mut engine = Engine::new();
        let result = engine.observe_str("{invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_observe_merges() {
        let mut engine = Engine::new();
        engine.observe(&serde_json::json!({"score": 10}));
        engine.observe(&serde_json::json!({"score": 90}));
        assert_eq!(engine.stats().records_observed, 2);
        assert_eq!(engine.stats().schemas_merged, 1);

        if let Some(schema) = engine.schema() {
            if let SchemaKind::Record(fields) = &schema.kind {
                if let Some(score) = fields.get("score") {
                    if let SchemaKind::Int { min, max, .. } = &score.kind {
                        assert_eq!(*min, 10);
                        assert_eq!(*max, 90);
                    }
                }
            }
        }
    }

    #[test]
    fn test_engine_process_single() {
        let mut engine = Engine::new();
        let result = engine.process(r#"{"drug": "aspirin", "cases": 42}"#);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert!(!output.violations.is_empty());
            assert_eq!(output.stats.records_observed, 1);
        }
    }

    #[test]
    fn test_engine_process_batch() {
        let mut engine = Engine::new();
        let result = engine.process(r#"[{"score": 10}, {"score": 50}, {"score": 90}]"#);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output.stats.records_observed, 3);
            // Schema should show merged range
            if let SchemaKind::Record(fields) = &output.schema.kind {
                if let Some(score) = fields.get("score") {
                    if let SchemaKind::Int { min, max, .. } = &score.kind {
                        assert_eq!(*min, 10);
                        assert_eq!(*max, 90);
                    }
                }
            }
        }
    }

    #[test]
    fn test_engine_process_with_fidelity() {
        let config = Config {
            verify_fidelity: true,
            ..Config::default()
        };
        let mut engine = Engine::with_config(config);
        let result = engine.process(r#"[1, 2, 3]"#);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output.fidelity.len(), 3);
            assert!(output.fidelity.iter().all(|f| *f == Fidelity::Exact));
        }
    }

    #[test]
    fn test_engine_process_no_violations() {
        let config = Config {
            synthesize_violations: false,
            ..Config::default()
        };
        let mut engine = Engine::with_config(config);
        let result = engine.process(r#"{"x": 42}"#);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert!(output.violations.is_empty());
        }
    }

    #[test]
    fn test_engine_reset() {
        let mut engine = Engine::new();
        engine.observe(&serde_json::json!(42));
        assert_eq!(engine.stats().records_observed, 1);
        engine.reset();
        assert_eq!(engine.stats().records_observed, 0);
        assert!(engine.schema().is_none());
    }

    #[test]
    fn test_engine_stats_display() {
        let engine = Engine::new();
        let display = format!("{}", engine.stats());
        assert!(display.contains("Transcriptase"));
        assert!(display.contains("Records"));
    }

    // ── Display ─────────────────────────────────────────────────────────

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", DiagnosticLevel::Critical), "CRITICAL");
        assert_eq!(format!("{}", DiagnosticLevel::Warning), "WARNING");
        assert_eq!(format!("{}", DiagnosticLevel::Info), "INFO");
    }

    #[test]
    fn test_violation_display() {
        let v = SchemaViolation {
            description: "score out of range".into(),
            assertion: "score > 100".into(),
            field: Some("score".into()),
            severity: DiagnosticLevel::Warning,
        };
        let s = format!("{v}");
        assert!(s.contains("WARNING"));
        assert!(s.contains("score > 100"));
    }

    // ── Serialization ───────────────────────────────────────────────────

    #[test]
    fn test_schema_serializes() {
        let schema = infer(&serde_json::json!({"x": 42}));
        let json = serde_json::to_string(&schema);
        assert!(json.is_ok());
    }

    #[test]
    fn test_output_serializes() {
        let mut engine = Engine::new();
        let output = engine.process(r#"{"x": 42}"#);
        assert!(output.is_ok());
        if let Ok(o) = output {
            let json = serde_json::to_string_pretty(&o);
            assert!(json.is_ok());
        }
    }

    // ── Data Generation ────────────────────────────────────────────────

    #[test]
    fn test_generate_null() {
        let schema = Schema::new(None, SchemaKind::Null);
        let val = generate(&schema);
        assert!(val.is_null());
    }

    #[test]
    fn test_generate_mixed() {
        let schema = Schema::new(None, SchemaKind::Mixed);
        let val = generate(&schema);
        assert!(val.is_null());
    }

    #[test]
    fn test_generate_bool_true_dominant() {
        let schema = Schema::new(
            Some("active".into()),
            SchemaKind::Bool {
                true_count: 7,
                false_count: 3,
            },
        );
        let val = generate(&schema);
        assert_eq!(val, serde_json::json!(true));
    }

    #[test]
    fn test_generate_bool_false_dominant() {
        let schema = Schema::new(
            Some("deleted".into()),
            SchemaKind::Bool {
                true_count: 2,
                false_count: 8,
            },
        );
        let val = generate(&schema);
        assert_eq!(val, serde_json::json!(false));
    }

    #[test]
    fn test_generate_int_midpoint() {
        let schema = Schema::new(
            Some("score".into()),
            SchemaKind::Int {
                min: 10,
                max: 90,
                sum: 500,
            },
        );
        let val = generate(&schema);
        assert_eq!(val, serde_json::json!(50));
    }

    #[test]
    fn test_generate_float_midpoint() {
        let schema = Schema::new(
            Some("prr".into()),
            SchemaKind::Float {
                min: 1.0,
                max: 5.0,
                sum: 30.0,
            },
        );
        let val = generate(&schema);
        assert_eq!(val, serde_json::json!(3.0));
    }

    #[test]
    fn test_generate_string_from_name() {
        let schema = Schema::new(
            Some("drug".into()),
            SchemaKind::Str {
                min_len: 4,
                max_len: 8,
                unique_count: 5,
            },
        );
        let val = generate(&schema);
        if let serde_json::Value::String(s) = &val {
            // avg_len = (4+8)/2 = 6, so "drug" repeated: "drugdr"
            assert_eq!(s.len(), 6);
        } else {
            assert!(false, "Expected String");
        }
    }

    #[test]
    fn test_generate_array() {
        let elem_schema = Schema::new(
            None,
            SchemaKind::Int {
                min: 1,
                max: 9,
                sum: 50,
            },
        );
        let schema = Schema::new(
            Some("items".into()),
            SchemaKind::Array {
                element: Box::new(elem_schema),
                min_len: 3,
                max_len: 7,
            },
        );
        let val = generate(&schema);
        if let serde_json::Value::Array(arr) = &val {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], serde_json::json!(5));
        } else {
            assert!(false, "Expected Array");
        }
    }

    #[test]
    fn test_generate_record() {
        let schema = infer(&serde_json::json!({"drug": "aspirin", "cases": 42}));
        let val = generate(&schema);
        assert!(val.is_object());
        let obj = val.as_object();
        assert!(obj.is_some());
        if let Some(map) = obj {
            assert!(map.contains_key("drug"));
            assert!(map.contains_key("cases"));
            // cases: Int{42,42} → midpoint = 42
            assert_eq!(map.get("cases"), Some(&serde_json::json!(42)));
        }
    }

    #[test]
    fn test_engine_generate_none_when_empty() {
        let engine = Engine::new();
        assert!(engine.generate().is_none());
    }

    #[test]
    fn test_engine_generate_after_observe() {
        let mut engine = Engine::new();
        engine.observe(&serde_json::json!({"score": 10}));
        engine.observe(&serde_json::json!({"score": 90}));
        let generated = engine.generate();
        assert!(generated.is_some());
        if let Some(val) = generated {
            // score: Int{10,90} → midpoint = 50
            assert_eq!(val.get("score"), Some(&serde_json::json!(50)));
        }
    }

    #[test]
    fn test_engine_generate_batch() {
        let mut engine = Engine::new();
        engine.observe(&serde_json::json!({"x": 100}));
        let batch = engine.generate_batch(3);
        assert!(batch.is_some());
        if let Some(records) = batch {
            assert_eq!(records.len(), 3);
            for r in &records {
                assert_eq!(r.get("x"), Some(&serde_json::json!(100)));
            }
        }
    }

    #[test]
    fn test_generate_roundtrip_schema_valid() {
        // Observe data, generate synthetic, observe synthetic → schema still valid
        let mut engine = Engine::new();
        engine.observe(&serde_json::json!({"a": 10, "b": "hello"}));
        engine.observe(&serde_json::json!({"a": 90, "b": "world!"}));
        let synthetic = engine.generate();
        assert!(synthetic.is_some());
        if let Some(val) = synthetic {
            // The generated data should itself be observable without error
            engine.observe(&val);
            assert_eq!(engine.stats().records_observed, 3);
            // Schema should still be Record with both fields
            if let Some(schema) = engine.schema() {
                assert!(matches!(schema.kind, SchemaKind::Record(_)));
            }
        }
    }
}
