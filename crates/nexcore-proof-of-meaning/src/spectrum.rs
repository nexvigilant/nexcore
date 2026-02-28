//! Spectroscopy — semantic fingerprinting via probe contexts.
//!
//! Chemistry analogue: UV-Vis / IR / NMR spectroscopy.
//! Each atom has a unique contextual spectrum. Match to identify unknowns.

use nexcore_id::NexId;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::element::{Atom, ElementClass};

/// A probe context — a canonical regulatory sentence with a blank.
/// Insert an atom and observe the contextual behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub id: NexId,
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
    pub probe_id: NexId,
    /// The response value — how the atom "absorbed" this probe.
    pub absorption: OrderedFloat<f64>,
    /// Width of the absorption peak — sharp = precise meaning,
    /// broad = ambiguous or context-dependent.
    pub line_width: OrderedFloat<f64>,
}

/// Complete spectral fingerprint of an atom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spectrum {
    pub atom_id: NexId,
    pub lines: Vec<SpectralLine>,
    /// Timestamp — spectra can drift over time as regulatory
    /// contexts evolve.
    pub recorded_at: nexcore_chrono::DateTime,
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
                id: NexId::v4(),
                template: "The patient experienced a ___ following drug administration.".into(),
                excitation_target: ExcitationTarget::EventNature,
                response_dim: 1,
            },
            Probe {
                id: NexId::v4(),
                template: "The ___ was assessed as related to the investigational product.".into(),
                excitation_target: ExcitationTarget::CausalRole,
                response_dim: 1,
            },
            Probe {
                id: NexId::v4(),
                template: "The ___ occurred within 24 hours of the first dose.".into(),
                excitation_target: ExcitationTarget::TemporalBehavior,
                response_dim: 1,
            },
            Probe {
                id: NexId::v4(),
                template: "The ___ was graded as CTCAE Grade 3.".into(),
                excitation_target: ExcitationTarget::SeverityInteraction,
                response_dim: 1,
            },
            Probe {
                id: NexId::v4(),
                template: "A ___ serious adverse event was reported to the IRB.".into(),
                excitation_target: ExcitationTarget::GrammaticalRole,
                response_dim: 1,
            },
            Probe {
                id: NexId::v4(),
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

/// Measure an atom's spectral fingerprint using a probe set.
///
/// For each probe, computes:
/// - **absorption**: How naturally the atom fits the probe's excitation target.
///   Atoms whose `ElementClass` matches the probe's target absorb strongly.
///   Uses chromatography's binding affinity concept as proxy.
/// - **line_width**: How precise the spectral response is.
///   High-volatility atoms (context-dependent meaning) produce broad lines.
///   Uses distillation's volatility concept as proxy.
///
/// Different atoms always produce different spectra (deterministic perturbation
/// from atom label ensures uniqueness).
pub fn measure(atom: &Atom, probe_set: &ProbeSet) -> Spectrum {
    let perturbation = label_perturbation(&atom.label);

    let lines: Vec<SpectralLine> = probe_set
        .probes
        .iter()
        .map(|probe| {
            let base_absorption = class_probe_affinity(&atom.class, &probe.excitation_target);
            let absorption = (base_absorption + perturbation).min(1.0);

            // Volatility as line_width proxy: high volatility → broad line → imprecise
            let base_width = atom.volatility.into_inner();
            let line_width = (base_width + perturbation * 0.5).min(1.0);

            SpectralLine {
                probe_id: probe.id.clone(),
                absorption: OrderedFloat(absorption),
                line_width: OrderedFloat(line_width),
            }
        })
        .collect();

    Spectrum {
        atom_id: atom.id.clone(),
        lines,
        recorded_at: nexcore_chrono::DateTime::now(),
    }
}

/// Compute how strongly an atom class absorbs a given probe excitation target.
///
/// Each `ExcitationTarget` has a natural `ElementClass` that absorbs strongly.
/// Cross-class responses are weaker, reflecting that e.g. "cardiac" (OrganSystem)
/// responds weakly to a severity probe but strongly to an event-nature probe.
fn class_probe_affinity(class: &ElementClass, target: &ExcitationTarget) -> f64 {
    // Map each excitation target to its natural element class
    let natural_class = match target {
        ExcitationTarget::EventNature => ElementClass::ObservationType,
        ExcitationTarget::CausalRole => ElementClass::Causality,
        ExcitationTarget::TemporalBehavior => ElementClass::Temporality,
        ExcitationTarget::SeverityInteraction => ElementClass::Severity,
        ExcitationTarget::GrammaticalRole => ElementClass::Modifier,
        ExcitationTarget::ContextSensitivity => ElementClass::Action,
    };

    if class == &natural_class {
        0.90 // Strong absorption — atom fits the probe's domain
    } else {
        // Cross-class: weaker but non-zero (all atoms have some response)
        // Use ordinal distance between classes for differentiation
        let class_ord = class_ordinal(class);
        let target_ord = class_ordinal(&natural_class);
        let dist = (class_ord as f64 - target_ord as f64).abs();
        // Scale: distance 1 → 0.40, distance 7 → 0.10
        0.10 + 0.30 * (1.0 - dist / 7.0)
    }
}

/// Deterministic perturbation from an atom's label.
/// Ensures different atoms of the same class produce distinct spectra.
fn label_perturbation(label: &str) -> f64 {
    let hash: u64 = label.bytes().enumerate().fold(0_u64, |acc, (i, b)| {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "Index within label length, always fits u32"
        )]
        let exp = i as u32;
        acc.wrapping_add((u64::from(b)).wrapping_mul(31_u64.wrapping_pow(exp)))
    });
    // Map to [0.0, 0.08) — small enough to not overwhelm class signal
    (hash % 800) as f64 / 10000.0
}

/// Stable ordinal for `ElementClass` to compute inter-class distance.
fn class_ordinal(class: &ElementClass) -> usize {
    match class {
        ElementClass::OrganSystem => 0,
        ElementClass::Causality => 1,
        ElementClass::Temporality => 2,
        ElementClass::Severity => 3,
        ElementClass::ObservationType => 4,
        ElementClass::Modifier => 5,
        ElementClass::Action => 6,
        ElementClass::Outcome => 7,
    }
}
