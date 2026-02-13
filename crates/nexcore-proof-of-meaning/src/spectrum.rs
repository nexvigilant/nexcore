//! Spectroscopy — semantic fingerprinting via probe contexts.
//!
//! Chemistry analogue: UV-Vis / IR / NMR spectroscopy.
//! Each atom has a unique contextual spectrum. Match to identify unknowns.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A probe context — a canonical regulatory sentence with a blank.
/// Insert an atom and observe the contextual behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub id: Uuid,
    /// The probe template with a placeholder for atom insertion.
    pub template: String,
    /// What aspect of meaning this probe tests.
    pub excitation_target: ExcitationTarget,
    /// The expected dimensionality of the response.
    pub response_dim: usize,
}

/// What aspect of meaning a probe is designed to test.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExcitationTarget {
    /// Does this atom function as a medical event?
    EventNature,
    /// Does this atom carry causal weight?
    CausalRole,
    /// Does this atom have temporal properties?
    TemporalBehavior,
    /// How does this atom interact with severity modifiers?
    SeverityInteraction,
    /// Does this atom behave as a modifier or a noun?
    GrammaticalRole,
    /// Is this atom context-dependent or context-independent?
    ContextSensitivity,
}

/// A single spectral line — the atom's response to one probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralLine {
    pub probe_id: Uuid,
    /// The response value — how the atom "absorbed" this probe.
    pub absorption: OrderedFloat<f64>,
    /// Width of the absorption peak — sharp = precise meaning,
    /// broad = ambiguous or context-dependent.
    pub line_width: OrderedFloat<f64>,
}

/// Complete spectral fingerprint of an atom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spectrum {
    pub atom_id: Uuid,
    pub lines: Vec<SpectralLine>,
    /// Timestamp — spectra can drift over time as regulatory
    /// contexts evolve.
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

impl Spectrum {
    /// Compute spectral distance between two spectra.
    ///
    /// Returns a value between 0.0 (identical) and 1.0 (completely different).
    pub fn distance(&self, other: &Spectrum) -> SpectralDistance {
        if self.lines.len() != other.lines.len() {
            return SpectralDistance::Incomparable {
                reason: "Spectra measured with different probe sets".into(),
            };
        }

        if self.lines.is_empty() {
            return SpectralDistance::Incomparable {
                reason: "Empty spectra cannot be compared".into(),
            };
        }

        let sum_sq: f64 = self
            .lines
            .iter()
            .zip(other.lines.iter())
            .map(|(a, b)| {
                let da = a.absorption.into_inner() - b.absorption.into_inner();
                let dw = a.line_width.into_inner() - b.line_width.into_inner();
                da * da + dw * dw
            })
            .sum();

        let distance = (sum_sq / self.lines.len() as f64).sqrt();

        SpectralDistance::Measured {
            distance: OrderedFloat(distance),
            interpretation: interpret_distance(distance),
        }
    }
}

/// Result of comparing two spectra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpectralDistance {
    /// Successfully measured distance.
    Measured {
        distance: OrderedFloat<f64>,
        interpretation: EquivalenceInterpretation,
    },
    /// Cannot compare — different probe sets used.
    Incomparable { reason: String },
}

/// Human-interpretable equivalence judgment from spectral distance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EquivalenceInterpretation {
    /// Distance < 0.05 — these are the same concept.
    Equivalent,
    /// Distance 0.05 - 0.15 — same concept, minor contextual variation.
    NearEquivalent { divergence_note: String },
    /// Distance 0.15 - 0.40 — overlapping but distinct.
    PartialOverlap { overlap_regions: Vec<String> },
    /// Distance > 0.40 — different concepts.
    Distinct,
}

fn interpret_distance(d: f64) -> EquivalenceInterpretation {
    if d < 0.05 {
        EquivalenceInterpretation::Equivalent
    } else if d < 0.15 {
        EquivalenceInterpretation::NearEquivalent {
            divergence_note: format!("Minor spectral divergence at d={d:.4}"),
        }
    } else if d < 0.40 {
        EquivalenceInterpretation::PartialOverlap {
            overlap_regions: vec![format!("Overlap score: {:.2}%", (1.0 - d) * 100.0)],
        }
    } else {
        EquivalenceInterpretation::Distinct
    }
}

/// A standardized set of probes for consistent spectral measurement.
pub struct ProbeSet {
    pub name: String,
    pub version: u32,
    pub probes: Vec<Probe>,
}

impl ProbeSet {
    /// Create the default pharmacovigilance probe set.
    pub fn pv_standard() -> Self {
        let probes = vec![
            Probe {
                id: Uuid::new_v4(),
                template: "The patient experienced a ___ following drug administration.".into(),
                excitation_target: ExcitationTarget::EventNature,
                response_dim: 1,
            },
            Probe {
                id: Uuid::new_v4(),
                template: "The ___ was assessed as related to the investigational product.".into(),
                excitation_target: ExcitationTarget::CausalRole,
                response_dim: 1,
            },
            Probe {
                id: Uuid::new_v4(),
                template: "The ___ occurred within 24 hours of the first dose.".into(),
                excitation_target: ExcitationTarget::TemporalBehavior,
                response_dim: 1,
            },
            Probe {
                id: Uuid::new_v4(),
                template: "The ___ was graded as CTCAE Grade 3.".into(),
                excitation_target: ExcitationTarget::SeverityInteraction,
                response_dim: 1,
            },
            Probe {
                id: Uuid::new_v4(),
                template: "A ___ serious adverse event was reported to the IRB.".into(),
                excitation_target: ExcitationTarget::GrammaticalRole,
                response_dim: 1,
            },
            Probe {
                id: Uuid::new_v4(),
                template: "The ___ is listed in the Reference Safety Information.".into(),
                excitation_target: ExcitationTarget::ContextSensitivity,
                response_dim: 1,
            },
        ];

        ProbeSet {
            name: "PV Standard Probe Set v1".into(),
            version: 1,
            probes,
        }
    }
}
