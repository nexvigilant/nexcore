//! R&D pipeline stages with Chomsky grammar classification.
//!
//! 9 stages from target identification through post-market pharmacovigilance,
//! each assigned its minimum-complexity Chomsky grammar level.

use crate::chomsky::{ChomskyLevel, Generator};
use crate::lex::LexSymbol;
use crate::taxonomy::{PharmaComposite, PharmaPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The 9 stages of the predictive pharmaceutical R&D pipeline.
///
/// Tier: T2-C | Dominant: σ (Sequence) — the pipeline IS a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineStage {
    /// Identify druggable biological targets for a disease.
    TargetIdentification,
    /// High-throughput screening for active compounds.
    HitFinding,
    /// Iterative DMTA optimization of lead candidates.
    LeadOptimization,
    /// Predict ADMET properties from molecular structure.
    AdmetPrediction,
    /// Assess safety in animal models (GLP).
    PreclinicalSafety,
    /// Phase I/II/III human clinical trials.
    ClinicalTrials,
    /// Regulatory submission strategy (IND, NDA, CTD).
    RegulatorySubmission,
    /// Post-market signal detection and risk management.
    Pharmacovigilance,
    /// AI/ML prediction layer (cross-cutting all stages).
    AiMlLayer,
}

impl PipelineStage {
    /// All 9 stages in pipeline order.
    pub const ALL: &'static [Self] = &[
        Self::TargetIdentification,
        Self::HitFinding,
        Self::LeadOptimization,
        Self::AdmetPrediction,
        Self::PreclinicalSafety,
        Self::ClinicalTrials,
        Self::RegulatorySubmission,
        Self::Pharmacovigilance,
        Self::AiMlLayer,
    ];

    /// Minimum Chomsky grammar level for this stage.
    ///
    /// This is the key architectural insight: different stages require
    /// different computational power.
    #[must_use]
    pub const fn chomsky_level(&self) -> ChomskyLevel {
        match self {
            // Type-3: threshold comparison, pass/fail
            Self::HitFinding => ChomskyLevel::Type3Regular,
            // Type-2: hierarchical composition, formal document grammar
            Self::AdmetPrediction | Self::RegulatorySubmission => ChomskyLevel::Type2ContextFree,
            // Type-1: context determines meaning
            Self::TargetIdentification | Self::PreclinicalSafety => {
                ChomskyLevel::Type1ContextSensitive
            }
            // Type-0 or approaching: adaptive, Turing-complete
            Self::LeadOptimization
            | Self::ClinicalTrials
            | Self::Pharmacovigilance
            | Self::AiMlLayer => ChomskyLevel::Type0Unrestricted,
        }
    }

    /// Generators needed at this stage.
    #[must_use]
    pub fn generators(&self) -> &'static [Generator] {
        match self {
            Self::HitFinding => &[Generator::Sigma, Generator::BigSigma],
            Self::AdmetPrediction | Self::RegulatorySubmission => {
                &[Generator::Sigma, Generator::BigSigma, Generator::Rho]
            }
            Self::TargetIdentification | Self::PreclinicalSafety => &[
                Generator::Sigma,
                Generator::BigSigma,
                Generator::Rho,
                Generator::Kappa,
            ],
            Self::LeadOptimization
            | Self::ClinicalTrials
            | Self::Pharmacovigilance
            | Self::AiMlLayer => Generator::ALL,
        }
    }

    /// Dominant Lex Primitiva symbols at this stage.
    #[must_use]
    pub fn dominant_symbols(&self) -> &'static [LexSymbol] {
        use LexSymbol::*;
        match self {
            Self::TargetIdentification => &[
                Existence, Causality, State, Mapping, Boundary, Location, Quantity,
            ],
            Self::HitFinding => &[Quantity, Boundary, Comparison, Existence],
            Self::LeadOptimization => &[
                Recursion, Location, Product, Mapping, Quantity, Causality, Boundary, Comparison,
                State,
            ],
            Self::AdmetPrediction => &[
                Boundary,
                Location,
                Quantity,
                State,
                Causality,
                Irreversibility,
                Void,
                Sum,
                Sequence,
            ],
            Self::PreclinicalSafety => &[
                Causality,
                Boundary,
                Quantity,
                Sequence,
                Irreversibility,
                Comparison,
                Frequency,
            ],
            Self::ClinicalTrials => &[
                Mapping, Void, Comparison, Existence, Quantity, Sequence, Boundary, Causality,
                Recursion,
            ],
            Self::RegulatorySubmission => &[
                Sequence,
                Mapping,
                Product,
                Boundary,
                Persistence,
                Sum,
                Quantity,
            ],
            Self::Pharmacovigilance => &[
                Boundary,
                Frequency,
                Existence,
                Causality,
                Comparison,
                Sequence,
                Irreversibility,
                Persistence,
                Quantity,
            ],
            Self::AiMlLayer => &[
                Mapping, Recursion, Boundary, Sequence, Sum, Quantity, Comparison,
            ],
        }
    }

    /// Key T2-P primitives active at this stage.
    #[must_use]
    pub fn active_primitives(&self) -> &'static [PharmaPrimitive] {
        match self {
            Self::TargetIdentification => &[
                PharmaPrimitive::Target,
                PharmaPrimitive::MechanismOfAction,
                PharmaPrimitive::Biomarker,
            ],
            Self::HitFinding => &[
                PharmaPrimitive::BindingAffinity,
                PharmaPrimitive::MolecularStructure,
            ],
            Self::LeadOptimization => &[
                PharmaPrimitive::MolecularStructure,
                PharmaPrimitive::Potency,
                PharmaPrimitive::DoseResponse,
                PharmaPrimitive::Permeability,
                PharmaPrimitive::Solubility,
            ],
            Self::AdmetPrediction => &[
                PharmaPrimitive::Absorption,
                PharmaPrimitive::Distribution,
                PharmaPrimitive::Metabolism,
                PharmaPrimitive::Elimination,
                PharmaPrimitive::Toxicity,
                PharmaPrimitive::Exposure,
                PharmaPrimitive::HalfLife,
            ],
            Self::PreclinicalSafety => &[
                PharmaPrimitive::Toxicity,
                PharmaPrimitive::DoseResponse,
                PharmaPrimitive::Exposure,
            ],
            Self::ClinicalTrials => &[
                PharmaPrimitive::Randomization,
                PharmaPrimitive::Blinding,
                PharmaPrimitive::Endpoint,
                PharmaPrimitive::StatisticalPower,
                PharmaPrimitive::Confounder,
                PharmaPrimitive::PlaceboEffect,
                PharmaPrimitive::Efficacy,
            ],
            Self::RegulatorySubmission => &[
                PharmaPrimitive::Endpoint,
                PharmaPrimitive::Efficacy,
                PharmaPrimitive::Toxicity,
            ],
            Self::Pharmacovigilance => &[
                PharmaPrimitive::Signal,
                PharmaPrimitive::Confounder,
                PharmaPrimitive::Biomarker,
            ],
            Self::AiMlLayer => &[
                PharmaPrimitive::MolecularStructure,
                PharmaPrimitive::DoseResponse,
                PharmaPrimitive::Signal,
                PharmaPrimitive::Biomarker,
            ],
        }
    }

    /// Key T2-C composites relevant at this stage.
    #[must_use]
    pub fn active_composites(&self) -> &'static [PharmaComposite] {
        match self {
            Self::TargetIdentification => &[],
            Self::HitFinding => &[],
            Self::LeadOptimization => &[
                PharmaComposite::StructureActivityRelationship,
                PharmaComposite::DrugLikeness,
                PharmaComposite::LeadOptimization,
            ],
            Self::AdmetPrediction => &[
                PharmaComposite::AdmetProfile,
                PharmaComposite::Bioavailability,
                PharmaComposite::Clearance,
                PharmaComposite::PharmacokineticModel,
            ],
            Self::PreclinicalSafety => &[PharmaComposite::TherapeuticWindow],
            Self::ClinicalTrials => &[
                PharmaComposite::ClinicalTrialDesign,
                PharmaComposite::SurrogateEndpoint,
                PharmaComposite::BenefitRiskRatio,
            ],
            Self::RegulatorySubmission => &[PharmaComposite::BenefitRiskRatio],
            Self::Pharmacovigilance => &[PharmaComposite::DisproportionalityAnalysis],
            Self::AiMlLayer => &[PharmaComposite::PredictiveModel],
        }
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::TargetIdentification => {
                "Identify druggable biological targets via genomics, proteomics, pathway analysis"
            }
            Self::HitFinding => {
                "High-throughput screening: test millions of compounds against threshold"
            }
            Self::LeadOptimization => {
                "Iterative DMTA cycle: design, make, test, analyze to optimize candidates"
            }
            Self::AdmetPrediction => {
                "Predict absorption, distribution, metabolism, elimination, toxicity"
            }
            Self::PreclinicalSafety => {
                "GLP toxicology in animal models: NOAEL, dose-response, organ toxicity"
            }
            Self::ClinicalTrials => {
                "Phase I-III human trials: safety, efficacy, confirmatory evidence"
            }
            Self::RegulatorySubmission => {
                "CTD assembly, IND/NDA submission, regulatory strategy execution"
            }
            Self::Pharmacovigilance => {
                "Post-market signal detection, REMS, periodic safety reports (PSUR/PBRER)"
            }
            Self::AiMlLayer => {
                "Cross-cutting AI/ML: GNN, QSAR, Bayesian optimization, causal inference"
            }
        }
    }

    /// Whether this stage is cross-cutting (applies to multiple other stages).
    #[must_use]
    pub const fn is_cross_cutting(&self) -> bool {
        matches!(self, Self::AiMlLayer)
    }

    /// Pipeline order index (0-based).
    #[must_use]
    pub const fn order(&self) -> u8 {
        match self {
            Self::TargetIdentification => 0,
            Self::HitFinding => 1,
            Self::LeadOptimization => 2,
            Self::AdmetPrediction => 3,
            Self::PreclinicalSafety => 4,
            Self::ClinicalTrials => 5,
            Self::RegulatorySubmission => 6,
            Self::Pharmacovigilance => 7,
            Self::AiMlLayer => 8,
        }
    }
}

impl fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {:?} — {} ({})",
            self.order(),
            self,
            self.chomsky_level(),
            self.chomsky_level().automaton()
        )
    }
}

/// Symbol utilization across pipeline: how many stages use each symbol.
#[must_use]
pub fn symbol_coverage() -> Vec<(LexSymbol, usize)> {
    let mut counts: Vec<(LexSymbol, usize)> = LexSymbol::all()
        .iter()
        .map(|&sym| {
            let count = PipelineStage::ALL
                .iter()
                .filter(|stage| stage.dominant_symbols().contains(&sym))
                .count();
            (sym, count)
        })
        .collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_stages() {
        assert_eq!(PipelineStage::ALL.len(), 9);
    }

    #[test]
    fn hts_is_type3() {
        assert_eq!(
            PipelineStage::HitFinding.chomsky_level(),
            ChomskyLevel::Type3Regular
        );
    }

    #[test]
    fn clinical_trials_is_type0() {
        assert_eq!(
            PipelineStage::ClinicalTrials.chomsky_level(),
            ChomskyLevel::Type0Unrestricted
        );
    }

    #[test]
    fn pipeline_order_is_sequential() {
        for (i, stage) in PipelineStage::ALL.iter().enumerate() {
            assert_eq!(stage.order() as usize, i);
        }
    }

    #[test]
    fn only_ai_is_cross_cutting() {
        for stage in PipelineStage::ALL {
            if *stage == PipelineStage::AiMlLayer {
                assert!(stage.is_cross_cutting());
            } else {
                assert!(!stage.is_cross_cutting());
            }
        }
    }

    #[test]
    fn quantity_is_universal() {
        let cov = symbol_coverage();
        let quantity_coverage = cov
            .iter()
            .find(|(s, _)| *s == LexSymbol::Quantity)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        // N should appear in most stages
        assert!(quantity_coverage >= 7, "N coverage: {quantity_coverage}");
    }

    #[test]
    fn boundary_is_dominant() {
        let cov = symbol_coverage();
        let boundary_coverage = cov
            .iter()
            .find(|(s, _)| *s == LexSymbol::Boundary)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        assert!(boundary_coverage >= 6, "∂ coverage: {boundary_coverage}");
    }

    #[test]
    fn all_stages_have_descriptions() {
        for stage in PipelineStage::ALL {
            assert!(!stage.description().is_empty());
        }
    }

    #[test]
    fn display_includes_chomsky() {
        let s = format!("{}", PipelineStage::HitFinding);
        assert!(s.contains("Type-3"));
        assert!(s.contains("Finite Automaton"));
    }
}
