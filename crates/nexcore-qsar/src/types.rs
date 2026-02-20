// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Core types for QSAR toxicity prediction.

use serde::{Deserialize, Serialize};

/// Classification of a toxicity endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToxClass {
    /// Compound is predicted to be toxic for this endpoint.
    Positive,
    /// Compound is predicted to be non-toxic for this endpoint.
    Negative,
    /// Prediction is uncertain; manual review recommended.
    Inconclusive,
}

/// Overall risk level derived from the worst-case endpoint probability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Max endpoint probability < 0.3.
    Low,
    /// Max endpoint probability in [0.3, 0.5).
    Medium,
    /// Max endpoint probability in [0.5, 0.7).
    High,
    /// Max endpoint probability >= 0.7.
    VeryHigh,
}

/// Result of a single toxicity prediction endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Predicted probability of toxicity (0.0 – 1.0).
    pub probability: f64,
    /// Binary/ternary classification derived from `probability`.
    pub classification: ToxClass,
    /// Model confidence in this prediction (0.0 – 1.0).
    pub confidence: f64,
    /// Whether the compound falls within the model's applicability domain.
    pub in_domain: bool,
    /// Identifier of the model version that produced this result.
    pub model_version: String,
}

/// Off-target binding hit from the binding-affinity module (Phase 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingHit {
    /// Name of the off-target receptor or channel.
    pub target_name: String,
    /// Predicted binding probability (0.0 – 1.0).
    pub probability: f64,
    /// Proposed pharmacological mechanism.
    pub mechanism: String,
}

/// Applicability domain assessment for a compound.
///
/// The domain is defined by the descriptor boundaries of the training set
/// (simplified bounding-box approach).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainStatus {
    /// Compound is within all descriptor boundaries.
    InDomain {
        /// Confidence derived from descriptor coverage.
        confidence: f64,
    },
    /// Compound violates two or more descriptor boundaries.
    OutOfDomain {
        /// Number of violated boundaries (used as a distance proxy).
        distance: f64,
        /// Human-readable description of which boundaries are violated.
        warning: String,
    },
    /// Compound violates exactly one descriptor boundary.
    Borderline {
        /// Reduced confidence relative to in-domain predictions.
        confidence: f64,
        /// Human-readable description of which boundary is violated.
        warning: String,
    },
}

/// Complete toxicity profile for a single compound.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToxProfile {
    /// Ames-surrogate mutagenicity prediction.
    pub mutagenicity: PredictionResult,
    /// Drug-induced liver injury (DILI) / hepatotoxicity prediction.
    pub hepatotoxicity: PredictionResult,
    /// hERG-channel cardiotoxicity prediction.
    pub cardiotoxicity: PredictionResult,
    /// Off-target binding hits (populated in Phase 2; empty in Phase 1).
    pub off_target_binding: Vec<BindingHit>,
    /// Applicability domain assessment.
    pub applicability_domain: DomainStatus,
    /// Worst-case risk level across all endpoints.
    pub overall_risk: RiskLevel,
}

impl Default for ToxProfile {
    fn default() -> Self {
        let default_result = PredictionResult {
            probability: 0.0,
            classification: ToxClass::Negative,
            confidence: 0.0,
            in_domain: false,
            model_version: String::new(),
        };
        Self {
            mutagenicity: default_result.clone(),
            hepatotoxicity: default_result.clone(),
            cardiotoxicity: default_result,
            off_target_binding: Vec::new(),
            applicability_domain: DomainStatus::OutOfDomain {
                distance: 0.0,
                warning: String::new(),
            },
            overall_risk: RiskLevel::Low,
        }
    }
}
