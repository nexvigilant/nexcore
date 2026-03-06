// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # NexVigilant Core — phenotype — Adversarial Test Fixture Generator
//!
//! In biology, a **phenotype** is the observable expression of a genotype.
//! Mutations in the genotype produce different phenotypes.
//!
//! This crate takes a schema (genotype) and produces **mutated** JSON values
//! (phenotypes) designed to trigger specific drift types in the ribosome
//! drift detection pipeline.
//!
//! ## Pipeline
//!
//! ```text
//! Schema → mutate(Mutation) → Phenotype { data, mutations_applied, expected_drifts }
//!                                ↓
//!                    Ribosome::validate → DriftResult (should detect!)
//! ```
//!
//! ## T1 Grounding
//!
//! | Concept | Primitive | Role |
//! |---------|-----------|------|
//! | Mutation dispatch | ∂ (conditional) | Select mutation strategy |
//! | Value generation | μ (function) | Schema → mutated Value |
//! | Mutation batch | σ (sequence) | Iterate mutation types |
//! | Type swap | κ (comparison) | Expected vs actual drift |
//!
//! ## Tier: T2-C

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod commensal;
pub mod error;
pub mod grounding;
pub mod harm_synthesis;

use std::fmt;

use error::{PhenotypeError, PhenotypeResult};
use nexcore_ribosome::DriftType;
use nexcore_transcriptase::{Schema, SchemaKind};
use serde::{Deserialize, Serialize};

// ─── Mutation Types ────────────────────────────────────────────────────────

/// What kind of mutation to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mutation {
    /// Replace a value's type entirely (Int → String, etc.)
    TypeMismatch,
    /// Add extra fields to a Record.
    AddField,
    /// Remove existing fields from a Record.
    RemoveField,
    /// Push numeric values beyond baseline range.
    RangeExpand,
    /// Change string lengths significantly.
    LengthChange,
    /// Change array sizes (add/remove elements).
    ArrayResize,
    /// Replace the entire structure with a different kind.
    StructureSwap,
}

impl Mutation {
    /// All available mutation types.
    pub const ALL: &[Self] = &[
        Self::TypeMismatch,
        Self::AddField,
        Self::RemoveField,
        Self::RangeExpand,
        Self::LengthChange,
        Self::ArrayResize,
        Self::StructureSwap,
    ];

    /// Which drift types this mutation is expected to trigger.
    #[must_use]
    pub fn expected_drift_types(self) -> Vec<DriftType> {
        match self {
            Self::TypeMismatch | Self::StructureSwap => vec![DriftType::TypeMismatch],
            Self::AddField => vec![DriftType::ExtraField],
            Self::RemoveField => vec![DriftType::MissingField],
            Self::RangeExpand => vec![DriftType::RangeExpansion],
            Self::LengthChange => vec![DriftType::LengthChange],
            Self::ArrayResize => vec![DriftType::ArraySizeChange],
        }
    }
}

impl fmt::Display for Mutation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch => write!(f, "TYPE_MISMATCH"),
            Self::AddField => write!(f, "ADD_FIELD"),
            Self::RemoveField => write!(f, "REMOVE_FIELD"),
            Self::RangeExpand => write!(f, "RANGE_EXPAND"),
            Self::LengthChange => write!(f, "LENGTH_CHANGE"),
            Self::ArrayResize => write!(f, "ARRAY_RESIZE"),
            Self::StructureSwap => write!(f, "STRUCTURE_SWAP"),
        }
    }
}

// ─── Phenotype Result ──────────────────────────────────────────────────────

/// A mutated JSON value with metadata about what was changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phenotype {
    /// The mutated JSON data.
    pub data: serde_json::Value,
    /// Which mutations were applied.
    pub mutations_applied: Vec<Mutation>,
    /// Which drift types the ribosome should detect.
    pub expected_drifts: Vec<DriftType>,
}

// ─── Mutation Engine ───────────────────────────────────────────────────────

/// Apply a single mutation to a schema, producing a mutated JSON value.
#[must_use]
pub fn mutate(schema: &Schema, mutation: Mutation) -> Phenotype {
    let data = apply_mutation(&schema.kind, mutation);
    Phenotype {
        data,
        mutations_applied: vec![mutation],
        expected_drifts: mutation.expected_drift_types(),
    }
}

/// Generate one phenotype per mutation type.
#[must_use]
pub fn mutate_all(schema: &Schema) -> Vec<Phenotype> {
    Mutation::ALL.iter().map(|m| mutate(schema, *m)).collect()
}

/// Generate `count` random-ish phenotypes cycling through mutation types.
#[must_use]
pub fn mutate_batch(schema: &Schema, count: usize) -> Vec<Phenotype> {
    (0..count)
        .map(|i| {
            let mutation = Mutation::ALL[i % Mutation::ALL.len()];
            mutate(schema, mutation)
        })
        .collect()
}

/// Verify a phenotype against the ribosome: does the mutation actually
/// trigger drift detection?
///
/// Uses a very low threshold (0.01) since we're testing that mutations
/// produce measurable drift, not that they exceed production thresholds.
///
/// Returns `Ok(true)` if the ribosome detected drift (good — mutation worked).
///
/// # Errors
///
/// Returns `PhenotypeError::VerificationError` if the ribosome cannot
/// store the contract or validate the phenotype.
pub fn verify(schema: &Schema, phenotype: &Phenotype) -> PhenotypeResult<bool> {
    verify_with_threshold(schema, phenotype, 0.01)
}

/// Verify a phenotype against the ribosome with a custom drift threshold.
///
/// # Errors
///
/// Returns `PhenotypeError::VerificationError` if the ribosome cannot
/// store the contract or validate the phenotype.
pub fn verify_with_threshold(
    schema: &Schema,
    phenotype: &Phenotype,
    threshold: f64,
) -> PhenotypeResult<bool> {
    let mut ribosome = nexcore_ribosome::Ribosome::with_config(nexcore_ribosome::RibosomeConfig {
        drift_threshold: threshold,
        auto_update: false,
    });
    ribosome
        .store_contract("phenotype-test", schema.clone())
        .map_err(|e| PhenotypeError::VerificationError(format!("failed to store contract: {e}")))?;

    match ribosome.validate("phenotype-test", &phenotype.data) {
        Some(result) => Ok(result.drift_detected),
        None => Err(PhenotypeError::VerificationError(
            "ribosome returned no validation result".to_string(),
        )),
    }
}

/// Verify all phenotypes and return (detected_count, total_count).
///
/// # Errors
///
/// Returns `PhenotypeError::VerificationError` if any individual verification fails.
pub fn verify_batch(schema: &Schema, phenotypes: &[Phenotype]) -> PhenotypeResult<(usize, usize)> {
    let mut detected = 0usize;
    for p in phenotypes {
        if verify(schema, p)? {
            detected = detected.saturating_add(1);
        }
    }
    Ok((detected, phenotypes.len()))
}

// ─── Mutation Implementations ──────────────────────────────────────────────

/// Core dispatch: apply a mutation to a schema kind, producing a Value.
fn apply_mutation(kind: &SchemaKind, mutation: Mutation) -> serde_json::Value {
    match mutation {
        Mutation::TypeMismatch => type_mismatch(kind),
        Mutation::AddField => add_field(kind),
        Mutation::RemoveField => remove_field(kind),
        Mutation::RangeExpand => range_expand(kind),
        Mutation::LengthChange => length_change(kind),
        Mutation::ArrayResize => array_resize(kind),
        Mutation::StructureSwap => structure_swap(kind),
    }
}

/// Replace the value with a completely different type.
fn type_mismatch(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Int { .. } => serde_json::Value::String("mutated_to_string".to_string()),
        SchemaKind::Float { .. } => serde_json::Value::Bool(true),
        SchemaKind::Str { .. } => serde_json::json!(42),
        SchemaKind::Bool { .. } => serde_json::json!(99),
        SchemaKind::Null => serde_json::json!("was_null"),
        SchemaKind::Array { .. } => serde_json::json!({"mutated": "from_array"}),
        SchemaKind::Record(fields) => {
            // Turn record into a type-mismatched record: swap field types
            let mut map = serde_json::Map::new();
            for (key, field_schema) in fields {
                map.insert(key.clone(), type_mismatch(&field_schema.kind));
            }
            serde_json::Value::Object(map)
        }
        SchemaKind::Mixed => serde_json::json!([1, "mixed", true]),
    }
}

/// Add extra fields to a Record; for non-records, wrap in a record.
fn add_field(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Record(fields) => {
            let mut map = serde_json::Map::new();
            // Keep existing fields with normal values
            for (key, field_schema) in fields {
                map.insert(key.clone(), generate_normal(&field_schema.kind));
            }
            // Add extra fields
            map.insert("__extra_field_1".to_string(), serde_json::json!("injected"));
            map.insert("__extra_field_2".to_string(), serde_json::json!(999));
            serde_json::Value::Object(map)
        }
        // Non-record: generate normal value (add_field only meaningful for records)
        other => generate_normal(other),
    }
}

/// Remove fields from a Record; for non-records, return normal value.
fn remove_field(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Record(fields) => {
            let mut map = serde_json::Map::new();
            // Only keep first half of fields
            let keep_count = fields.len() / 2;
            for (key, field_schema) in fields.iter().take(keep_count.max(1)) {
                map.insert(key.clone(), generate_normal(&field_schema.kind));
            }
            serde_json::Value::Object(map)
        }
        other => generate_normal(other),
    }
}

/// Expand numeric ranges well beyond the baseline.
fn range_expand(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Int { max, .. } => {
            // Double the max value
            serde_json::json!(max.saturating_mul(2).saturating_add(1000))
        }
        SchemaKind::Float { max, .. } => {
            serde_json::json!(max * 3.0 + 100.0)
        }
        SchemaKind::Record(fields) => {
            let mut map = serde_json::Map::new();
            for (key, field_schema) in fields {
                map.insert(key.clone(), range_expand(&field_schema.kind));
            }
            serde_json::Value::Object(map)
        }
        other => generate_normal(other),
    }
}

/// Change string lengths significantly.
fn length_change(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Str { max_len, .. } => {
            // Generate string much longer than max
            let target_len = (*max_len * 3).max(100);
            let s: String = (0..target_len).map(|_| 'x').collect();
            serde_json::Value::String(s)
        }
        SchemaKind::Record(fields) => {
            let mut map = serde_json::Map::new();
            for (key, field_schema) in fields {
                map.insert(key.clone(), length_change(&field_schema.kind));
            }
            serde_json::Value::Object(map)
        }
        other => generate_normal(other),
    }
}

/// Change array sizes (shrink to empty or grow significantly).
fn array_resize(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Array {
            element, max_len, ..
        } => {
            // Grow array to 3x the max length
            let new_len = (*max_len * 3).max(10);
            let items: Vec<serde_json::Value> = (0..new_len)
                .map(|_| generate_normal(&element.kind))
                .collect();
            serde_json::Value::Array(items)
        }
        SchemaKind::Record(fields) => {
            let mut map = serde_json::Map::new();
            for (key, field_schema) in fields {
                map.insert(key.clone(), array_resize(&field_schema.kind));
            }
            serde_json::Value::Object(map)
        }
        other => generate_normal(other),
    }
}

/// Replace the entire structure with a fundamentally different kind.
fn structure_swap(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Record(_) => serde_json::json!(true), // Record → Bool
        SchemaKind::Array { .. } => serde_json::json!(42), // Array → Int
        SchemaKind::Int { .. } => serde_json::json!([1, 2, 3]), // Int → Array
        SchemaKind::Str { .. } => serde_json::json!(null), // Str → Null
        _ => serde_json::json!({"swapped": true}),        // Anything else → Record
    }
}

// ─── Normal Value Generation ───────────────────────────────────────────────

/// Generate a "normal" (non-mutated) value from a schema kind.
/// Uses the same logic as transcriptase::generate but without importing it.
fn generate_normal(kind: &SchemaKind) -> serde_json::Value {
    match kind {
        SchemaKind::Null | SchemaKind::Mixed => serde_json::Value::Null,
        SchemaKind::Bool {
            true_count,
            false_count,
        } => serde_json::Value::Bool(*true_count >= *false_count),
        SchemaKind::Int { min, max, .. } => {
            let mid = min / 2 + max / 2 + (min % 2 + max % 2) / 2;
            serde_json::json!(mid)
        }
        SchemaKind::Float { min, max, .. } => {
            serde_json::json!((min + max) / 2.0)
        }
        SchemaKind::Str {
            min_len, max_len, ..
        } => {
            let len = usize::midpoint(*min_len, *max_len);
            let s: String = (0..len.max(1)).map(|_| 'a').collect();
            serde_json::Value::String(s)
        }
        SchemaKind::Array {
            element,
            min_len,
            max_len,
        } => {
            let len = usize::midpoint(*min_len, *max_len);
            let items: Vec<serde_json::Value> = (0..len.max(1))
                .map(|_| generate_normal(&element.kind))
                .collect();
            serde_json::Value::Array(items)
        }
        SchemaKind::Record(fields) => {
            let mut map = serde_json::Map::new();
            for (key, field_schema) in fields {
                map.insert(key.clone(), generate_normal(&field_schema.kind));
            }
            serde_json::Value::Object(map)
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_transcriptase::infer;

    fn sample_record() -> serde_json::Value {
        serde_json::json!({
            "name": "Alice",
            "age": 30,
            "score": 95.5,
            "active": true,
            "tags": ["admin", "user"]
        })
    }

    fn sample_schema() -> Schema {
        infer(&sample_record())
    }

    // ── Mutation dispatch ──────────────────────────────────────────────

    #[test]
    fn test_all_mutations_produce_phenotypes() {
        let schema = sample_schema();
        let phenotypes = mutate_all(&schema);
        assert_eq!(phenotypes.len(), Mutation::ALL.len());
    }

    #[test]
    fn test_mutate_batch() {
        let schema = sample_schema();
        let phenotypes = mutate_batch(&schema, 10);
        assert_eq!(phenotypes.len(), 10);
    }

    #[test]
    fn test_each_mutation_has_expected_drifts() {
        for mutation in Mutation::ALL {
            let drifts = mutation.expected_drift_types();
            assert!(!drifts.is_empty(), "{mutation} has no expected drift types");
        }
    }

    // ── Type mismatch ──────────────────────────────────────────────────

    #[test]
    fn test_type_mismatch_int() {
        let schema = infer(&serde_json::json!(42));
        let phenotype = mutate(&schema, Mutation::TypeMismatch);
        assert!(phenotype.data.is_string(), "Int should mutate to String");
    }

    #[test]
    fn test_type_mismatch_string() {
        let schema = infer(&serde_json::json!("hello"));
        let phenotype = mutate(&schema, Mutation::TypeMismatch);
        assert!(phenotype.data.is_number(), "String should mutate to Number");
    }

    #[test]
    fn test_type_mismatch_bool() {
        let schema = infer(&serde_json::json!(true));
        let phenotype = mutate(&schema, Mutation::TypeMismatch);
        assert!(phenotype.data.is_number(), "Bool should mutate to Number");
    }

    #[test]
    fn test_type_mismatch_record() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::TypeMismatch);
        // Record with type-mismatched fields
        assert!(phenotype.data.is_object());
        let empty_map = serde_json::Map::new();
        let obj = phenotype.data.as_object().unwrap_or(&empty_map);
        // "age" was Int, should now be String
        if let Some(age) = obj.get("age") {
            assert!(age.is_string(), "age should be mutated from Int to String");
        }
    }

    // ── Add field ──────────────────────────────────────────────────────

    #[test]
    fn test_add_field_to_record() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::AddField);
        let empty_map = serde_json::Map::new();
        let obj = phenotype.data.as_object().unwrap_or(&empty_map);
        assert!(
            obj.contains_key("__extra_field_1"),
            "Should have extra field"
        );
        assert!(
            obj.contains_key("__extra_field_2"),
            "Should have second extra field"
        );
    }

    // ── Remove field ───────────────────────────────────────────────────

    #[test]
    fn test_remove_field_from_record() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::RemoveField);
        let empty_map = serde_json::Map::new();
        let obj = phenotype.data.as_object().unwrap_or(&empty_map);
        // Original has 5 fields, should have fewer
        assert!(
            obj.len() < 5,
            "Should have removed fields, got {}",
            obj.len()
        );
    }

    // ── Range expand ───────────────────────────────────────────────────

    #[test]
    fn test_range_expand_int() {
        let schema = infer(&serde_json::json!(50));
        let phenotype = mutate(&schema, Mutation::RangeExpand);
        let val = phenotype.data.as_i64().unwrap_or(0);
        assert!(val > 50, "Expanded range should exceed baseline: got {val}");
    }

    #[test]
    fn test_range_expand_float() {
        let schema = infer(&serde_json::json!(10.0));
        let phenotype = mutate(&schema, Mutation::RangeExpand);
        let val = phenotype.data.as_f64().unwrap_or(0.0);
        assert!(
            val > 10.0,
            "Expanded range should exceed baseline: got {val}"
        );
    }

    // ── Length change ──────────────────────────────────────────────────

    #[test]
    fn test_length_change_string() {
        let schema = infer(&serde_json::json!("short"));
        let phenotype = mutate(&schema, Mutation::LengthChange);
        let s = phenotype.data.as_str().unwrap_or("");
        assert!(
            s.len() > 10,
            "Mutated string should be much longer: len={}",
            s.len()
        );
    }

    // ── Array resize ───────────────────────────────────────────────────

    #[test]
    fn test_array_resize() {
        let schema = infer(&serde_json::json!([1, 2]));
        let phenotype = mutate(&schema, Mutation::ArrayResize);
        let arr = phenotype.data.as_array();
        assert!(arr.is_some(), "Should still be an array");
        assert!(
            arr.map(|a| a.len()).unwrap_or(0) > 5,
            "Should be significantly larger"
        );
    }

    // ── Structure swap ─────────────────────────────────────────────────

    #[test]
    fn test_structure_swap_record_to_bool() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::StructureSwap);
        assert!(phenotype.data.is_boolean(), "Record should swap to bool");
    }

    #[test]
    fn test_structure_swap_int_to_array() {
        let schema = infer(&serde_json::json!(42));
        let phenotype = mutate(&schema, Mutation::StructureSwap);
        assert!(phenotype.data.is_array(), "Int should swap to array");
    }

    // ── Verification ───────────────────────────────────────────────────

    #[test]
    fn test_verify_type_mismatch_detected() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::TypeMismatch);
        assert!(
            verify(&schema, &phenotype).unwrap_or(false),
            "Type mismatch should be detected"
        );
    }

    #[test]
    fn test_verify_add_field_detected() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::AddField);
        assert!(
            verify(&schema, &phenotype).unwrap_or(false),
            "Add field should be detected"
        );
    }

    #[test]
    fn test_verify_remove_field_detected() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::RemoveField);
        assert!(
            verify(&schema, &phenotype).unwrap_or(false),
            "Remove field should be detected"
        );
    }

    #[test]
    fn test_verify_structure_swap_detected() {
        let schema = sample_schema();
        let phenotype = mutate(&schema, Mutation::StructureSwap);
        assert!(
            verify(&schema, &phenotype).unwrap_or(false),
            "Structure swap should be detected"
        );
    }

    #[test]
    fn test_verify_batch_coverage() {
        let schema = sample_schema();
        let phenotypes = mutate_all(&schema);
        let (detected, total) = verify_batch(&schema, &phenotypes).unwrap_or((0, 0));
        // At minimum, type_mismatch, add_field, remove_field, structure_swap should trigger
        assert!(
            detected >= 4,
            "At least 4 mutations should trigger drift detection, got {detected}/{total}"
        );
    }

    // ── Display ────────────────────────────────────────────────────────

    #[test]
    fn test_mutation_display() {
        assert_eq!(Mutation::TypeMismatch.to_string(), "TYPE_MISMATCH");
        assert_eq!(Mutation::AddField.to_string(), "ADD_FIELD");
        assert_eq!(Mutation::StructureSwap.to_string(), "STRUCTURE_SWAP");
    }

    // ── Normal generation ──────────────────────────────────────────────

    #[test]
    fn test_generate_normal_int() {
        let val = generate_normal(&SchemaKind::Int {
            min: 0,
            max: 100,
            sum: 5000,
        });
        assert!(val.is_number());
    }

    #[test]
    fn test_generate_normal_string() {
        let val = generate_normal(&SchemaKind::Str {
            min_len: 3,
            max_len: 10,
            unique_count: 5,
        });
        assert!(val.is_string());
    }

    #[test]
    fn test_generate_normal_null() {
        let val = generate_normal(&SchemaKind::Null);
        assert!(val.is_null());
    }

    #[test]
    fn test_generate_normal_bool() {
        let val = generate_normal(&SchemaKind::Bool {
            true_count: 5,
            false_count: 3,
        });
        assert!(val.is_boolean());
    }
}
