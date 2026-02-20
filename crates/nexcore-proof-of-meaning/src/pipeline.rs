//! Transformation Pipeline — the full proof-of-meaning chain.
//!
//! Input: regulatory text
//! Output: EquivalenceProof { trail: [step1..step5] }
//!
//! Every step produces an auditable intermediate.
//! The chain of intermediates IS the proof.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::chromatography::{Column, SeparationQuality};
use crate::distillation::Distiller;
use crate::element::{Atom, ElementClass};
use crate::titration::{self, EquivalenceProof, Titrator};

/// A single step in the proof pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStep {
    pub step_number: usize,
    pub method: TransformationMethod,
    pub input_description: String,
    pub output_description: String,
    pub verification: StepVerification,
}

/// Which chemistry method was used at this step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationMethod {
    Distillation,
    Chromatography,
    Spectroscopy,
    StoichiometricComposition,
    Titration,
}

/// Verification status of a single proof step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepVerification {
    /// Step passed all checks.
    Verified { confidence: f64 },
    /// Step passed with warnings.
    VerifiedWithWarnings { warnings: Vec<String> },
    /// Step failed — the proof trail is broken here.
    Failed { reason: String },
}

/// The complete proof trail for transforming an expression into canonical form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofTrail {
    pub id: Uuid,
    pub input_expression: String,
    pub steps: Vec<ProofStep>,
    /// Is the entire trail valid?
    pub trail_valid: bool,
    /// Aggregate warnings from all steps.
    pub warnings: Vec<String>,
}

/// The complete proof that two expressions are (or aren't) equivalent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEquivalenceProof {
    pub id: Uuid,
    pub trail_a: ProofTrail,
    pub trail_b: ProofTrail,
    pub equivalence: EquivalenceProof,
    /// Both trails must be valid for the equivalence to be proven.
    pub proof_valid: bool,
}

/// The pipeline engine that chains all transformation methods.
pub struct ProofPipeline {
    distiller: Distiller,
    column: Column,
    titrator: Titrator,
}

impl ProofPipeline {
    /// Create a standard PV proof pipeline with comprehensive titrant set.
    pub fn pv_standard() -> Self {
        let titrants = vec![
            // Organ systems
            Atom::new("cardiac", ElementClass::OrganSystem, 0.15),
            Atom::new("hepatic", ElementClass::OrganSystem, 0.15),
            Atom::new("renal", ElementClass::OrganSystem, 0.15),
            Atom::new("neurological", ElementClass::OrganSystem, 0.15),
            Atom::new("pulmonary", ElementClass::OrganSystem, 0.15),
            Atom::new("gastrointestinal", ElementClass::OrganSystem, 0.15),
            Atom::new("dermatologic", ElementClass::OrganSystem, 0.15),
            // Causality
            Atom::new("related", ElementClass::Causality, 0.35),
            Atom::new("unrelated", ElementClass::Causality, 0.20),
            Atom::new("suspected", ElementClass::Causality, 0.40),
            Atom::new("probable", ElementClass::Causality, 0.20),
            // Temporality
            Atom::new("acute", ElementClass::Temporality, 0.15),
            Atom::new("chronic", ElementClass::Temporality, 0.15),
            Atom::new("delayed", ElementClass::Temporality, 0.20),
            Atom::new("following", ElementClass::Temporality, 0.25),
            // Severity
            Atom::new("mild", ElementClass::Severity, 0.15),
            Atom::new("moderate", ElementClass::Severity, 0.20),
            Atom::new("severe", ElementClass::Severity, 0.15),
            Atom::new("serious", ElementClass::Severity, 0.55),
            Atom::new("fatal", ElementClass::Severity, 0.05),
            // Observation types
            Atom::new("event", ElementClass::ObservationType, 0.25),
            Atom::new("reaction", ElementClass::ObservationType, 0.25),
            Atom::new("experience", ElementClass::ObservationType, 0.30),
            Atom::new("effect", ElementClass::ObservationType, 0.25),
            Atom::new("finding", ElementClass::ObservationType, 0.20),
            // Modifiers
            Atom::new("adverse", ElementClass::Modifier, 0.35),
            Atom::new("drug", ElementClass::Modifier, 0.05),
            Atom::new("unexpected", ElementClass::Modifier, 0.80),
            Atom::new("expected", ElementClass::Modifier, 0.55),
            Atom::new("treatment-emergent", ElementClass::Modifier, 0.45),
            // Actions
            Atom::new("immunization", ElementClass::Action, 0.10),
            Atom::new("vaccine", ElementClass::Action, 0.10),
            Atom::new("withdrawn", ElementClass::Action, 0.05),
            Atom::new("hospitalized", ElementClass::Action, 0.05),
            // Outcomes
            Atom::new("recovered", ElementClass::Outcome, 0.05),
            Atom::new("recovering", ElementClass::Outcome, 0.10),
            Atom::new("died", ElementClass::Outcome, 0.05),
        ];

        Self {
            distiller: Distiller::new(),
            column: Column::pv_standard(),
            titrator: Titrator::new(titrants),
        }
    }

    /// Transform a single expression through the full pipeline.
    pub fn transform(&self, expression: &str) -> ProofTrail {
        let mut steps = Vec::new();
        let mut warnings = Vec::new();
        let mut all_valid = true;

        // === STEP 1: DISTILLATION ===
        let distillation = self.distiller.distill(expression);
        let distill_step = ProofStep {
            step_number: 1,
            method: TransformationMethod::Distillation,
            input_description: format!("Raw expression: \"{expression}\""),
            output_description: format!(
                "Separated into {} fractions, {} residue items. Mass balance: {:.1}% recovered.",
                distillation.fractions.len(),
                distillation.residue.len(),
                100.0 - distillation.mass_balance.loss_percent,
            ),
            verification: if distillation.mass_balance.loss_percent < 2.0 {
                StepVerification::Verified {
                    confidence: 1.0 - distillation.mass_balance.loss_percent / 100.0,
                }
            } else if distillation.mass_balance.loss_percent < 10.0 {
                let w = format!(
                    "Distillation lost {:.1}% semantic mass",
                    distillation.mass_balance.loss_percent
                );
                warnings.push(w.clone());
                StepVerification::VerifiedWithWarnings { warnings: vec![w] }
            } else {
                all_valid = false;
                StepVerification::Failed {
                    reason: format!(
                        "Unacceptable mass loss: {:.1}%. Expression may be non-compositional.",
                        distillation.mass_balance.loss_percent
                    ),
                }
            },
        };
        steps.push(distill_step);

        // === STEP 2: CHROMATOGRAPHY ===
        let chromatogram = self.column.separate(expression);
        let chromat_step = ProofStep {
            step_number: 2,
            method: TransformationMethod::Chromatography,
            input_description: format!("{} distilled fractions", distillation.fractions.len()),
            output_description: format!(
                "Bound {} atoms to hierarchy positions. Quality: {:?}",
                chromatogram.bands.len(),
                chromatogram.quality,
            ),
            verification: match &chromatogram.quality {
                SeparationQuality::BaselineResolved => {
                    StepVerification::Verified { confidence: 0.95 }
                }
                SeparationQuality::PartiallyResolved { co_eluting_pairs } => {
                    let w = format!("Co-eluting pairs detected: {co_eluting_pairs:?}");
                    warnings.push(w.clone());
                    StepVerification::VerifiedWithWarnings { warnings: vec![w] }
                }
                SeparationQuality::PoorResolution => {
                    all_valid = false;
                    StepVerification::Failed {
                        reason: "Poor chromatographic resolution — hierarchy bindings ambiguous"
                            .into(),
                    }
                }
            },
        };
        steps.push(chromat_step);

        // === STEP 3: TITRATION ===
        let titration = self.titrator.titrate(expression);
        let titration_step = ProofStep {
            step_number: 3,
            method: TransformationMethod::Titration,
            input_description: format!("Expression: \"{expression}\""),
            output_description: format!(
                "Detected {} equivalence points. Residual unmatched meaning: {:.1}%",
                titration.equivalence_points.len(),
                titration.residual * 100.0,
            ),
            verification: if titration.residual < 0.15 {
                StepVerification::Verified {
                    confidence: 1.0 - titration.residual,
                }
            } else if titration.residual < 0.30 {
                let w = format!(
                    "{:.0}% of meaning unaccounted for by canonical atoms",
                    titration.residual * 100.0
                );
                warnings.push(w.clone());
                StepVerification::VerifiedWithWarnings { warnings: vec![w] }
            } else {
                all_valid = false;
                StepVerification::Failed {
                    reason: format!(
                        "{:.0}% of meaning unaccounted for — expression contains \
                         significant novel or non-standard semantics",
                        titration.residual * 100.0,
                    ),
                }
            },
        };
        steps.push(titration_step);

        ProofTrail {
            id: Uuid::new_v4(),
            input_expression: expression.to_string(),
            steps,
            trail_valid: all_valid,
            warnings,
        }
    }

    /// Prove equivalence between two expressions.
    pub fn prove_equivalence(
        &self,
        expression_a: &str,
        expression_b: &str,
    ) -> SemanticEquivalenceProof {
        let trail_a = self.transform(expression_a);
        let trail_b = self.transform(expression_b);
        let equivalence = titration::prove_equivalence(&self.titrator, expression_a, expression_b);

        let proof_valid = trail_a.trail_valid && trail_b.trail_valid;

        SemanticEquivalenceProof {
            id: Uuid::new_v4(),
            trail_a,
            trail_b,
            equivalence,
            proof_valid,
        }
    }
}
