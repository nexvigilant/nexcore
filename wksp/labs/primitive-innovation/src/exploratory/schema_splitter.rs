// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # SchemaGuidedSplitter
//!
//! **Tier**: T2-C (κ + σ + μ + ∂ + N)
//! **Dominant**: κ (Comparison)
//! **Bridge**: transcriptase (schema inference) × dtree (decision trees)
//! **Confidence**: 0.82
//!
//! Schema-constrained decision tree splitting. Instead of searching the full
//! value range for optimal splits, uses schema metadata to:
//!
//! 1. Constrain candidate splits to valid schema ranges
//! 2. Use schema-defined categories as natural split points
//! 3. Score schemas by how well they classify data

use core::fmt;
use std::collections::BTreeMap;

/// Schema type for a field.
///
/// ## Tier: T2-P (μ + ∂)
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaFieldType {
    /// Numeric field with known bounds.
    Numeric {
        /// Minimum valid value.
        min: f64,
        /// Maximum valid value.
        max: f64,
    },
    /// Categorical field with enumerated values.
    Categorical {
        /// Allowed categories.
        categories: Vec<String>,
    },
    /// Boolean field.
    Boolean,
}

/// Schema definition for a dataset.
///
/// ## Tier: T2-P (μ + σ)
#[derive(Debug, Clone)]
pub struct Schema {
    /// Fields by name.
    pub fields: BTreeMap<String, SchemaFieldType>,
}

impl Schema {
    /// Create a new empty schema.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    /// Add a numeric field.
    pub fn add_numeric(&mut self, name: impl Into<String>, min: f64, max: f64) {
        self.fields
            .insert(name.into(), SchemaFieldType::Numeric { min, max });
    }

    /// Add a categorical field.
    pub fn add_categorical(&mut self, name: impl Into<String>, categories: Vec<String>) {
        self.fields
            .insert(name.into(), SchemaFieldType::Categorical { categories });
    }

    /// Add a boolean field.
    pub fn add_boolean(&mut self, name: impl Into<String>) {
        self.fields.insert(name.into(), SchemaFieldType::Boolean);
    }

    /// Get a field's type.
    #[must_use]
    pub fn field_type(&self, name: &str) -> Option<&SchemaFieldType> {
        self.fields.get(name)
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

/// A candidate split point for a numeric field.
///
/// ## Tier: T2-P (κ + N)
#[derive(Debug, Clone)]
pub struct SplitCandidate {
    /// Field name.
    pub field: String,
    /// Split threshold value.
    pub threshold: f64,
    /// Information gain (or Gini reduction) from this split.
    pub gain: f64,
    /// Whether this split respects schema bounds.
    pub schema_valid: bool,
}

/// A data row: field name → numeric value.
pub type DataRow = BTreeMap<String, f64>;

/// Split evaluation result.
#[derive(Debug, Clone)]
pub struct SplitEvaluation {
    /// Best split found.
    pub best_split: Option<SplitCandidate>,
    /// All candidates evaluated.
    pub candidates_evaluated: usize,
    /// Candidates rejected by schema constraints.
    pub schema_rejected: usize,
}

/// Schema quality score for a dataset.
#[derive(Debug, Clone)]
pub struct SchemaQuality {
    /// How many rows conform to schema bounds (0.0 to 1.0).
    pub conformance_rate: f64,
    /// Number of violations found.
    pub violations: usize,
    /// Fields with violations.
    pub violating_fields: Vec<String>,
}

/// Schema-guided decision tree splitter.
///
/// ## Tier: T2-C (κ + σ + μ + ∂ + N)
/// Dominant: κ (Comparison) — split comparison is the core operation
///
/// Primitives:
/// - κ: Threshold comparison for splits, gain evaluation, violation detection
/// - σ: Ordered candidate evaluation, sorted data sequences
/// - μ: Field name → value mapping, schema → field type mapping
/// - ∂: Schema bounds as hard constraints, valid range boundaries
/// - N: Threshold values, gain scores, conformance rates
#[derive(Debug, Clone)]
pub struct SchemaGuidedSplitter {
    /// Schema constraining the split search.
    schema: Schema,
    /// Number of candidate splits per numeric field.
    num_candidates: usize,
    /// Whether to enforce strict schema compliance (reject out-of-bound splits).
    strict_mode: bool,
}

impl SchemaGuidedSplitter {
    /// Create a new splitter with a schema.
    #[must_use]
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            num_candidates: 10,
            strict_mode: true,
        }
    }

    /// Set number of candidate splits per numeric field.
    #[must_use]
    pub fn with_candidates(mut self, n: usize) -> Self {
        self.num_candidates = n.max(2);
        self
    }

    /// Set strict mode (rejects splits outside schema bounds).
    #[must_use]
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Generate candidate split points for a field, constrained by schema.
    ///
    /// For numeric fields: evenly-spaced points within [min, max].
    /// For categorical: one split per category (binary: is/isn't this category).
    #[must_use]
    pub fn generate_candidates(&self, field: &str) -> Vec<f64> {
        match self.schema.field_type(field) {
            Some(SchemaFieldType::Numeric { min, max }) => {
                let range = max - min;
                if range <= 0.0 || self.num_candidates == 0 {
                    return Vec::new();
                }

                let step = range / (self.num_candidates as f64 + 1.0);
                (1..=self.num_candidates)
                    .map(|i| min + step * i as f64)
                    .collect()
            }
            Some(SchemaFieldType::Categorical { categories }) => {
                // For categorical, return indices as split points
                (0..categories.len()).map(|i| i as f64).collect()
            }
            Some(SchemaFieldType::Boolean) => {
                vec![0.5] // Single split at 0.5 (false=0, true=1)
            }
            None => Vec::new(),
        }
    }

    /// Evaluate the best split for a field given data and labels.
    ///
    /// Uses Gini impurity reduction as the gain metric.
    /// Labels are binary: 0.0 or 1.0.
    #[must_use]
    pub fn evaluate_field(&self, field: &str, data: &[DataRow], labels: &[f64]) -> SplitEvaluation {
        let candidates = self.generate_candidates(field);
        let mut best: Option<SplitCandidate> = None;
        let mut evaluated = 0_usize;
        let mut rejected = 0_usize;

        for threshold in &candidates {
            // Check if threshold is within schema bounds
            let schema_valid = self.is_valid_threshold(field, *threshold);

            if self.strict_mode && !schema_valid {
                rejected += 1;
                continue;
            }

            evaluated += 1;

            // Split data into left (≤ threshold) and right (> threshold)
            let mut left_labels = Vec::new();
            let mut right_labels = Vec::new();

            for (i, row) in data.iter().enumerate() {
                if i >= labels.len() {
                    break;
                }
                let val = row.get(field).copied().unwrap_or(0.0);
                if val <= *threshold {
                    left_labels.push(labels[i]);
                } else {
                    right_labels.push(labels[i]);
                }
            }

            // Calculate Gini gain
            let parent_gini = gini_impurity(labels);
            let n = labels.len() as f64;

            if n == 0.0 {
                continue;
            }

            let left_weight = left_labels.len() as f64 / n;
            let right_weight = right_labels.len() as f64 / n;
            let weighted_child_gini = left_weight * gini_impurity(&left_labels)
                + right_weight * gini_impurity(&right_labels);

            let gain = parent_gini - weighted_child_gini;

            let candidate = SplitCandidate {
                field: field.to_string(),
                threshold: *threshold,
                gain,
                schema_valid,
            };

            if best.as_ref().is_none_or(|b| gain > b.gain) {
                best = Some(candidate);
            }
        }

        SplitEvaluation {
            best_split: best,
            candidates_evaluated: evaluated,
            schema_rejected: rejected,
        }
    }

    /// Find the best split across all fields.
    #[must_use]
    pub fn find_best_split(&self, data: &[DataRow], labels: &[f64]) -> SplitEvaluation {
        let mut global_best: Option<SplitCandidate> = None;
        let mut total_evaluated = 0_usize;
        let mut total_rejected = 0_usize;

        for field_name in self.schema.fields.keys() {
            let eval = self.evaluate_field(field_name, data, labels);
            total_evaluated += eval.candidates_evaluated;
            total_rejected += eval.schema_rejected;

            if let Some(candidate) = eval.best_split
                && global_best.as_ref().is_none_or(|b| candidate.gain > b.gain)
            {
                global_best = Some(candidate);
            }
        }

        SplitEvaluation {
            best_split: global_best,
            candidates_evaluated: total_evaluated,
            schema_rejected: total_rejected,
        }
    }

    /// Score schema quality against actual data.
    ///
    /// Counts how many rows have values within schema-defined bounds.
    #[must_use]
    pub fn score_schema_quality(&self, data: &[DataRow]) -> SchemaQuality {
        let mut violations = 0_usize;
        let mut violating_fields: BTreeMap<String, bool> = BTreeMap::new();

        for row in data {
            for (field_name, field_type) in &self.schema.fields {
                if let Some(&value) = row.get(field_name) {
                    let valid = match field_type {
                        SchemaFieldType::Numeric { min, max } => value >= *min && value <= *max,
                        SchemaFieldType::Categorical { categories } => {
                            let idx = value as usize;
                            idx < categories.len()
                        }
                        SchemaFieldType::Boolean => value == 0.0 || value == 1.0,
                    };

                    if !valid {
                        violations += 1;
                        violating_fields.insert(field_name.clone(), true);
                    }
                }
            }
        }

        let total_checks = data.len() * self.schema.fields.len();
        let conformance_rate = if total_checks > 0 {
            1.0 - (violations as f64 / total_checks as f64)
        } else {
            1.0
        };

        SchemaQuality {
            conformance_rate,
            violations,
            violating_fields: violating_fields.into_keys().collect(),
        }
    }

    /// Check if a threshold is valid for a given field.
    #[must_use]
    pub fn is_valid_threshold(&self, field: &str, threshold: f64) -> bool {
        match self.schema.field_type(field) {
            Some(SchemaFieldType::Numeric { min, max }) => threshold >= *min && threshold <= *max,
            Some(SchemaFieldType::Categorical { categories }) => {
                (threshold as usize) < categories.len()
            }
            Some(SchemaFieldType::Boolean) => (0.0..=1.0).contains(&threshold),
            None => false,
        }
    }

    /// Get the schema.
    #[must_use]
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Number of fields in the schema.
    #[must_use]
    pub fn field_count(&self) -> usize {
        self.schema.fields.len()
    }
}

/// Calculate Gini impurity for binary labels (0.0 or 1.0).
fn gini_impurity(labels: &[f64]) -> f64 {
    let n = labels.len();
    if n == 0 {
        return 0.0;
    }

    let positive: usize = labels.iter().filter(|&&l| l > 0.5).count();
    let p1 = positive as f64 / n as f64;
    let p0 = 1.0 - p1;

    1.0 - (p0 * p0 + p1 * p1)
}

impl fmt::Display for SchemaGuidedSplitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SchemaGuidedSplitter({} fields, {} candidates/field, strict={})",
            self.field_count(),
            self.num_candidates,
            self.strict_mode,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_schema() -> Schema {
        let mut schema = Schema::new();
        schema.add_numeric("age", 0.0, 120.0);
        schema.add_numeric("income", 0.0, 500_000.0);
        schema.add_categorical("color", vec!["red".into(), "green".into(), "blue".into()]);
        schema.add_boolean("active");
        schema
    }

    fn build_test_data() -> (Vec<DataRow>, Vec<f64>) {
        let data: Vec<DataRow> = vec![
            BTreeMap::from([
                ("age".into(), 25.0),
                ("income".into(), 50_000.0),
                ("active".into(), 1.0),
            ]),
            BTreeMap::from([
                ("age".into(), 45.0),
                ("income".into(), 80_000.0),
                ("active".into(), 1.0),
            ]),
            BTreeMap::from([
                ("age".into(), 65.0),
                ("income".into(), 120_000.0),
                ("active".into(), 0.0),
            ]),
            BTreeMap::from([
                ("age".into(), 30.0),
                ("income".into(), 45_000.0),
                ("active".into(), 1.0),
            ]),
            BTreeMap::from([
                ("age".into(), 70.0),
                ("income".into(), 150_000.0),
                ("active".into(), 0.0),
            ]),
            BTreeMap::from([
                ("age".into(), 55.0),
                ("income".into(), 90_000.0),
                ("active".into(), 0.0),
            ]),
        ];

        let labels = vec![1.0, 1.0, 0.0, 1.0, 0.0, 0.0];

        (data, labels)
    }

    #[test]
    fn test_candidate_generation_numeric() {
        let schema = build_test_schema();
        let splitter = SchemaGuidedSplitter::new(schema).with_candidates(5);

        let candidates = splitter.generate_candidates("age");
        assert_eq!(candidates.len(), 5);

        // All candidates should be within [0, 120]
        for c in &candidates {
            assert!(*c >= 0.0 && *c <= 120.0);
        }
    }

    #[test]
    fn test_candidate_generation_categorical() {
        let schema = build_test_schema();
        let splitter = SchemaGuidedSplitter::new(schema);

        let candidates = splitter.generate_candidates("color");
        assert_eq!(candidates.len(), 3); // red, green, blue
    }

    #[test]
    fn test_candidate_generation_boolean() {
        let schema = build_test_schema();
        let splitter = SchemaGuidedSplitter::new(schema);

        let candidates = splitter.generate_candidates("active");
        assert_eq!(candidates.len(), 1);
        assert!((candidates[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_field() {
        let schema = build_test_schema();
        let splitter = SchemaGuidedSplitter::new(schema).with_candidates(10);
        let (data, labels) = build_test_data();

        let eval = splitter.evaluate_field("age", &data, &labels);

        assert!(eval.best_split.is_some());
        let best = eval.best_split.unwrap_or(SplitCandidate {
            field: String::new(),
            threshold: 0.0,
            gain: 0.0,
            schema_valid: false,
        });

        assert!(best.gain > 0.0, "Should find a split with positive gain");
        assert!(best.schema_valid, "Best split should be schema-valid");
    }

    #[test]
    fn test_find_best_split_across_fields() {
        let schema = build_test_schema();
        let splitter = SchemaGuidedSplitter::new(schema).with_candidates(10);
        let (data, labels) = build_test_data();

        let eval = splitter.find_best_split(&data, &labels);

        assert!(eval.best_split.is_some());
        assert!(eval.candidates_evaluated > 0);
    }

    #[test]
    fn test_schema_quality_scoring() {
        let schema = build_test_schema();
        let splitter = SchemaGuidedSplitter::new(schema);

        // All-conforming data
        let (good_data, _) = build_test_data();
        let quality = splitter.score_schema_quality(&good_data);
        assert!((quality.conformance_rate - 1.0).abs() < 0.01);
        assert_eq!(quality.violations, 0);

        // Data with violations
        let bad_data = vec![BTreeMap::from([
            ("age".into(), 200.0),     // Violates [0, 120]
            ("income".into(), -100.0), // Violates [0, 500000]
            ("active".into(), 5.0),    // Violates boolean
        ])];

        let bad_quality = splitter.score_schema_quality(&bad_data);
        assert!(bad_quality.violations > 0);
        assert!(bad_quality.conformance_rate < 1.0);
    }

    #[test]
    fn test_strict_mode_rejects_invalid() {
        let mut schema = Schema::new();
        schema.add_numeric("x", 0.0, 10.0);

        let splitter = SchemaGuidedSplitter::new(schema.clone())
            .with_candidates(5)
            .with_strict_mode(true);

        // All candidates should be within [0, 10]
        for c in splitter.generate_candidates("x") {
            assert!(splitter.is_valid_threshold("x", c));
        }
    }

    #[test]
    fn test_gini_impurity() {
        // Pure labels → Gini = 0
        let pure = vec![1.0, 1.0, 1.0];
        assert!(gini_impurity(&pure).abs() < 0.01);

        // Mixed 50/50 → Gini = 0.5
        let mixed = vec![0.0, 1.0, 0.0, 1.0];
        assert!((gini_impurity(&mixed) - 0.5).abs() < 0.01);

        // Empty → Gini = 0
        let empty: Vec<f64> = vec![];
        assert!(gini_impurity(&empty).abs() < 0.01);
    }
}
