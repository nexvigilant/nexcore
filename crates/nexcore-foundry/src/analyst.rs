//! Analyst pipeline artifact types for The Foundry architecture.
//!
//! Defines the typed artifacts that flow through the three analyst stations
//! (A1, A2, A3) and across the bridges that connect them to the builder
//! pipeline.
//!
//! # Pipeline overview
//!
//! ```text
//! Extraction bridge  →  A1 (Aggregator)      →  AggregatedMetrics
//! Crystallization bridge  →  A2 (Patterner)  →  PatternReport
//! Inference bridge  →  A3 (Intelligence)     →  IntelligenceReport
//! ```
//!
//! All types are fully serialisable so they can cross process boundaries
//! as JSON payloads carried in [`nexcore_brain`] artifacts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Bridge input types (extraction bridge → A1)
// ---------------------------------------------------------------------------

/// A single scalar measurement emitted by an extraction station.
///
/// # Examples
///
/// ```
/// use nexcore_foundry::analyst::Metric;
///
/// let m = Metric {
///     name: "line_count".to_string(),
///     value: 412.0,
///     unit: "lines".to_string(),
/// };
/// assert_eq!(m.name, "line_count");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Human-readable metric identifier (e.g. `"line_count"`, `"cyclomatic_complexity"`).
    pub name: String,

    /// Numeric measurement.
    pub value: f64,

    /// SI or domain unit string (e.g. `"lines"`, `"ms"`, `"ratio"`).
    pub unit: String,
}

/// Bundle of metrics produced by one extraction station run.
///
/// This is the canonical input artifact for the A1 aggregator station and
/// acts as the output carried across the extraction bridge.
///
/// # Examples
///
/// ```
/// use chrono::Utc;
/// use nexcore_foundry::analyst::{Metric, MetricReport};
///
/// let report = MetricReport {
///     source_station: "extractor-alpha".to_string(),
///     timestamp: Utc::now(),
///     metrics: vec![
///         Metric { name: "fn_count".to_string(), value: 23.0, unit: "functions".to_string() },
///     ],
/// };
/// assert!(!report.metrics.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReport {
    /// Identifier of the extraction station that produced this report.
    pub source_station: String,

    /// Wall-clock time at which the report was generated.
    pub timestamp: DateTime<Utc>,

    /// All measurements captured during this station run.
    pub metrics: Vec<Metric>,
}

// ---------------------------------------------------------------------------
// A1 output
// ---------------------------------------------------------------------------

/// Qualitative complexity classification derived from aggregated metrics.
///
/// Variants are ordered from least to most complex so that simple numeric
/// comparisons (`<=`, `>=`) reflect relative severity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplexityRating {
    /// Straightforward structure, minimal branching.
    Low,
    /// Acceptable complexity; warrants periodic review.
    Moderate,
    /// Elevated complexity; active refactoring recommended.
    High,
    /// Complexity exceeds safe operating limits; immediate action required.
    Critical,
}

/// Aggregated quality and complexity summary produced by the A1 station.
///
/// A1 consumes one or more [`MetricReport`] bundles and fuses them into a
/// single scored snapshot that downstream stations and the governance layer
/// consume.
///
/// # Examples
///
/// ```
/// use nexcore_foundry::analyst::{AggregatedMetrics, ComplexityRating, MetricReport};
/// use chrono::Utc;
///
/// let agg = AggregatedMetrics {
///     quality_score: 0.91,
///     primitive_density: 0.65,
///     complexity_rating: ComplexityRating::Low,
///     coverage_delta: 0.03,
///     raw_metrics: vec![],
/// };
/// assert!(agg.quality_score > 0.9);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Composite quality score in the range `[0.0, 1.0]`.
    ///
    /// Higher values indicate better overall code health.
    pub quality_score: f64,

    /// Ratio of Lex Primitiva primitives present per unit of structure.
    ///
    /// A score close to `1.0` indicates thorough primitive coverage.
    pub primitive_density: f64,

    /// Qualitative complexity classification derived from the raw metrics.
    pub complexity_rating: ComplexityRating,

    /// Change in test-coverage percentage relative to the previous run.
    ///
    /// Positive values indicate coverage growth; negative values indicate
    /// regression.
    pub coverage_delta: f64,

    /// All raw [`MetricReport`] bundles that contributed to this aggregate.
    pub raw_metrics: Vec<MetricReport>,
}

// ---------------------------------------------------------------------------
// Bridge input types (crystallisation bridge → A2)
// ---------------------------------------------------------------------------

/// A detected structural pattern with an associated confidence score.
///
/// Produced by the crystallisation bridge and consumed by the A2 patterner
/// station.
///
/// # Examples
///
/// ```
/// use nexcore_foundry::analyst::PatternSignature;
///
/// let sig = PatternSignature {
///     name: "command-query-separation".to_string(),
///     confidence: 0.88,
/// };
/// assert!(sig.confidence > 0.0 && sig.confidence <= 1.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSignature {
    /// Canonical pattern name as recognised by the crystallisation bridge.
    pub name: String,

    /// Detection confidence in the range `[0.0, 1.0]`.
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// A2 output
// ---------------------------------------------------------------------------

/// Structural pattern analysis produced by the A2 patterner station.
///
/// Classifies detected signals into four orthogonal categories so that the
/// A3 intelligence station and the governance layer can reason about them
/// independently.
///
/// # Examples
///
/// ```
/// use nexcore_foundry::analyst::PatternReport;
///
/// let report = PatternReport {
///     patterns: vec!["repository-pattern".to_string()],
///     anti_patterns: vec![],
///     structural_risks: vec![],
///     opportunities: vec!["extract-service-layer".to_string()],
/// };
/// assert_eq!(report.patterns.len(), 1);
/// assert!(report.anti_patterns.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternReport {
    /// Beneficial structural patterns that were positively detected.
    pub patterns: Vec<String>,

    /// Anti-patterns that degrade maintainability or reliability.
    pub anti_patterns: Vec<String>,

    /// Structural risks that may not be anti-patterns themselves but create
    /// fragility (e.g. deep call chains, large modules).
    pub structural_risks: Vec<String>,

    /// Improvement opportunities surfaced by the pattern analysis.
    pub opportunities: Vec<String>,
}

// ---------------------------------------------------------------------------
// Bridge input types (inference bridge → A3)
// ---------------------------------------------------------------------------

/// A directed causal edge between two named entities.
///
/// Strength is normalised to `[0.0, 1.0]`; higher values indicate stronger
/// causal evidence.
///
/// # Examples
///
/// ```
/// use nexcore_foundry::analyst::CausalEdge;
///
/// let edge = CausalEdge {
///     from: "high_cyclomatic_complexity".to_string(),
///     to: "elevated_defect_rate".to_string(),
///     strength: 0.74,
/// };
/// assert!(edge.strength > 0.5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    /// Source node label.
    pub from: String,

    /// Target node label.
    pub to: String,

    /// Causal strength in the range `[0.0, 1.0]`.
    pub strength: f64,
}

/// Directed causal graph carried across the inference bridge into A3.
///
/// # Examples
///
/// ```
/// use nexcore_foundry::analyst::{CausalEdge, CausalGraph};
///
/// let graph = CausalGraph {
///     edges: vec![CausalEdge {
///         from: "missing_tests".to_string(),
///         to: "regression_risk".to_string(),
///         strength: 0.9,
///     }],
/// };
/// assert_eq!(graph.edges.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraph {
    /// All directed edges in the causal graph.
    pub edges: Vec<CausalEdge>,
}

// ---------------------------------------------------------------------------
// A3 output
// ---------------------------------------------------------------------------

/// Actionable risk classification emitted by the A3 intelligence station.
///
/// Variants are ordered from least to most severe so that ordinal comparisons
/// are semantically correct.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No immediate concerns; routine monitoring is sufficient.
    Low,
    /// Elevated attention warranted; schedule remediation.
    Moderate,
    /// Significant risk present; prioritise remediation.
    High,
    /// Critical exposure; block delivery until resolved.
    Critical,
}

/// Final intelligence synthesis produced by the A3 station.
///
/// Combines pattern analysis, causal reasoning, and metric aggregation into
/// an actionable intelligence report that the governance layer consumes to
/// make release decisions.
///
/// # Examples
///
/// ```
/// use nexcore_foundry::analyst::{IntelligenceReport, RiskLevel};
///
/// let report = IntelligenceReport {
///     findings: vec!["Cyclomatic complexity exceeds threshold in 3 modules".to_string()],
///     recommendations: vec!["Decompose `process_pipeline` into smaller functions".to_string()],
///     risk_level: RiskLevel::Moderate,
///     confidence: 0.85,
/// };
/// assert!(!report.findings.is_empty());
/// assert!(!report.recommendations.is_empty());
/// assert!(report.confidence > 0.0 && report.confidence <= 1.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceReport {
    /// Key findings synthesised from all upstream analyst stations.
    pub findings: Vec<String>,

    /// Prioritised, actionable recommendations ordered by expected impact.
    pub recommendations: Vec<String>,

    /// Overall risk classification for the current pipeline run.
    pub risk_level: RiskLevel,

    /// Confidence in the report's conclusions in the range `[0.0, 1.0]`.
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        AggregatedMetrics, CausalEdge, CausalGraph, ComplexityRating, IntelligenceReport, Metric,
        MetricReport, PatternReport, RiskLevel,
    };
    use chrono::Utc;

    /// Serialise a `MetricReport` to JSON and deserialise it back; assert
    /// that all fields round-trip correctly.
    #[test]
    fn metric_report_roundtrip() {
        let original = MetricReport {
            source_station: "extractor-beta".to_string(),
            timestamp: Utc::now(),
            metrics: vec![
                Metric {
                    name: "fn_count".to_string(),
                    value: 42.0,
                    unit: "functions".to_string(),
                },
                Metric {
                    name: "loc".to_string(),
                    value: 1024.0,
                    unit: "lines".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&original).unwrap();
        let recovered: MetricReport = serde_json::from_str(&json).unwrap();

        assert_eq!(recovered.source_station, original.source_station);
        assert_eq!(recovered.metrics.len(), 2);
        assert_eq!(recovered.metrics[0].name, "fn_count");
        assert_eq!(recovered.metrics[0].value, 42.0);
        assert_eq!(recovered.metrics[1].name, "loc");
        assert_eq!(recovered.metrics[1].unit, "lines");
    }

    /// Verify that an `AggregatedMetrics` value with a quality score of 0.82
    /// is correctly stored and compares greater than 0.8.
    #[test]
    fn aggregated_metrics_quality_score() {
        let agg = AggregatedMetrics {
            quality_score: 0.82,
            primitive_density: 0.71,
            complexity_rating: ComplexityRating::Moderate,
            coverage_delta: 0.05,
            raw_metrics: vec![],
        };

        assert!(agg.quality_score > 0.8);
    }

    /// Verify that a `PatternReport` carrying one pattern and zero
    /// anti-patterns reflects those counts accurately.
    #[test]
    fn pattern_report_contains_findings() {
        let report = PatternReport {
            patterns: vec!["repository-pattern".to_string()],
            anti_patterns: vec![],
            structural_risks: vec![],
            opportunities: vec![],
        };

        assert_eq!(report.patterns.len(), 1);
        assert!(report.anti_patterns.is_empty());
    }

    /// Verify that a `CausalGraph` containing one edge stores that edge with
    /// a strength value greater than 0.5.
    #[test]
    fn causal_graph_builds_edges() {
        let graph = CausalGraph {
            edges: vec![CausalEdge {
                from: "missing_tests".to_string(),
                to: "regression_risk".to_string(),
                strength: 0.76,
            }],
        };

        assert_eq!(graph.edges.len(), 1);
        assert!(graph.edges[0].strength > 0.5);
    }

    /// Verify that an `IntelligenceReport` constructed with at least one
    /// finding and one recommendation has non-empty collections for both.
    #[test]
    fn intelligence_report_has_recommendations() {
        let report = IntelligenceReport {
            findings: vec!["High cyclomatic complexity in core module".to_string()],
            recommendations: vec!["Decompose `evaluate` into smaller functions".to_string()],
            risk_level: RiskLevel::High,
            confidence: 0.91,
        };

        assert!(!report.findings.is_empty());
        assert!(!report.recommendations.is_empty());
    }
}
