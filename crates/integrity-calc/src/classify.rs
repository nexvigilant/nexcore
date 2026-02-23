//! Classification via Arrhenius threshold
//!
//! Tier: T3 | Primitives: ∂ Boundary, → Causality

use crate::chemistry;
use serde::{Deserialize, Serialize};

/// Arrhenius parameters.
pub const ACTIVATION_ENERGY: f64 = 3.0;
pub const SCALE_FACTOR: f64 = 10.0;

/// Classification verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Human,
    Generated,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Human => write!(f, "human"),
            Verdict::Generated => write!(f, "generated"),
        }
    }
}

/// Full classification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub verdict: Verdict,
    pub probability: f64,
    pub confidence: f64,
}

/// Classify with custom threshold.
#[must_use]
pub fn classify_with_threshold(hill_score: f64, threshold: f64) -> Classification {
    let probability = chemistry::arrhenius_probability(ACTIVATION_ENERGY, hill_score, SCALE_FACTOR);

    let verdict = if probability > threshold {
        Verdict::Generated
    } else {
        Verdict::Human
    };

    let confidence = ((probability - threshold).abs() * 2.0).min(1.0);

    Classification {
        verdict,
        probability,
        confidence,
    }
}
