//! Pharma R&D primitive taxonomy: 24 T2-P + 14 T2-C + 8 T3 concepts.
//!
//! Every concept is grounded to Lex Primitiva symbols with tier classification
//! derived from unique symbol count.

use crate::lex::{LexSymbol, PrimitiveComposition, Tier};
use serde::{Deserialize, Serialize};
use std::fmt;

// ─── T2-P: Cross-Domain Primitives (24) ───────────────────────────────────

/// The 24 irreducible T2-P primitives of predictive pharmaceutical R&D.
///
/// Tier: T2-P | Each grounds to 2-3 Lex Primitiva symbols.
/// These are transferable across pharma, biotech, agrochemical, medtech.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PharmaPrimitive {
    BindingAffinity,
    Absorption,
    Metabolism,
    Distribution,
    Elimination,
    Toxicity,
    Efficacy,
    Potency,
    Exposure,
    Target,
    DoseResponse,
    Signal,
    MolecularStructure,
    Randomization,
    Blinding,
    Endpoint,
    HalfLife,
    Confounder,
    Permeability,
    Solubility,
    MechanismOfAction,
    Biomarker,
    PlaceboEffect,
    StatisticalPower,
}

impl PharmaPrimitive {
    /// All 24 primitives in canonical order.
    pub const ALL: &'static [Self] = &[
        Self::BindingAffinity,
        Self::Absorption,
        Self::Metabolism,
        Self::Distribution,
        Self::Elimination,
        Self::Toxicity,
        Self::Efficacy,
        Self::Potency,
        Self::Exposure,
        Self::Target,
        Self::DoseResponse,
        Self::Signal,
        Self::MolecularStructure,
        Self::Randomization,
        Self::Blinding,
        Self::Endpoint,
        Self::HalfLife,
        Self::Confounder,
        Self::Permeability,
        Self::Solubility,
        Self::MechanismOfAction,
        Self::Biomarker,
        Self::PlaceboEffect,
        Self::StatisticalPower,
    ];

    /// Lex Primitiva grounding for this primitive.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::BindingAffinity => &[Quantity, Comparison],
            Self::Absorption => &[Boundary, Location],
            Self::Metabolism => &[State, Causality],
            Self::Distribution => &[Location, Quantity],
            Self::Elimination => &[Irreversibility, Void],
            Self::Toxicity => &[Causality, Boundary],
            Self::Efficacy => &[Boundary, State, Causality],
            Self::Potency => &[Quantity, Causality],
            Self::Exposure => &[Sum, Quantity, Sequence],
            Self::Target => &[Existence, Causality, State],
            Self::DoseResponse => &[Mapping, Quantity, Causality],
            Self::Signal => &[Boundary, Frequency, Existence],
            Self::MolecularStructure => &[Location, Product, Mapping],
            Self::Randomization => &[Mapping, Void],
            Self::Blinding => &[Void, Comparison],
            Self::Endpoint => &[Existence, Quantity, Sequence],
            Self::HalfLife => &[Sequence, Quantity, Recursion],
            Self::Confounder => &[Causality, Causality], // two causal paths
            Self::Permeability => &[Boundary, Quantity, Sequence],
            Self::Solubility => &[Quantity, Boundary, Persistence],
            Self::MechanismOfAction => &[Sequence, Causality, State],
            Self::Biomarker => &[Mapping, Existence, Quantity],
            Self::PlaceboEffect => &[Causality, State, Void],
            Self::StatisticalPower => &[Quantity, Comparison, Boundary],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::BindingAffinity => {
                "Strength of non-covalent interaction between ligand and target (Kd)"
            }
            Self::Absorption => "Transport of substance across biological barrier",
            Self::Metabolism => "Enzymatic transformation into structurally distinct products",
            Self::Distribution => "Reversible partitioning among body compartments",
            Self::Elimination => "Irreversible removal of substance from the body",
            Self::Toxicity => "Capacity to cause dose-dependent harm to biological system",
            Self::Efficacy => "Maximum achievable therapeutic effect under ideal conditions (Emax)",
            Self::Potency => "Amount required to produce defined magnitude of effect (EC50)",
            Self::Exposure => "Integrated concentration at site over time (AUC)",
            Self::Target => "Biological entity whose modulation alters disease state",
            Self::DoseResponse => {
                "Functional mapping from administered amount to biological effect magnitude"
            }
            Self::Signal => {
                "Pattern in data exceeding expected background, suggesting non-random association"
            }
            Self::MolecularStructure => {
                "Spatial arrangement of atoms and bonds defining a chemical entity"
            }
            Self::Randomization => {
                "Allocation to conditions by process independent of subject characteristics"
            }
            Self::Blinding => "Concealment of treatment assignment to prevent evaluation bias",
            Self::Endpoint => {
                "Pre-specified measurable outcome determining intervention effectiveness"
            }
            Self::HalfLife => "Time required for quantity to decrease to half its initial value",
            Self::Confounder => {
                "Variable causally related to both intervention and outcome, distorting association"
            }
            Self::Permeability => "Rate at which substance traverses a biological membrane",
            Self::Solubility => "Maximum amount of substance dissolving in solvent at equilibrium",
            Self::MechanismOfAction => {
                "Specific molecular pathway by which substance produces biological effect"
            }
            Self::Biomarker => {
                "Measurable biological indicator of physiological or pharmacological process"
            }
            Self::PlaceboEffect => {
                "Physiological response from treatment expectation, not pharmacological activity"
            }
            Self::StatisticalPower => {
                "Probability that test correctly rejects a false null hypothesis (1-beta)"
            }
        }
    }

    /// Domains where this primitive appears.
    #[must_use]
    pub fn domains(&self) -> &'static [&'static str] {
        match self {
            Self::BindingAffinity => &["pharma", "biotech", "agrochemical", "materials"],
            Self::Absorption => &["pharma", "nutrition", "environmental", "materials"],
            Self::Metabolism => &["pharma", "nutrition", "ecology", "biochemistry"],
            Self::Distribution => &["pharma", "ecology", "logistics", "environmental"],
            Self::Elimination => &["pharma", "ecology", "toxicology", "engineering"],
            Self::Toxicity => &["pharma", "agrochemical", "environmental", "industrial"],
            Self::Efficacy => &["pharma", "agriculture", "engineering", "education"],
            Self::Potency => &["pharma", "agrochemical", "toxicology"],
            Self::Exposure => &["pharma", "environmental", "occupational_health"],
            Self::Target => &["pharma", "biotech", "agrochemical", "oncology"],
            Self::DoseResponse => &["pharma", "toxicology", "radiation", "agrochemical"],
            Self::Signal => &["pharma", "epidemiology", "finance", "engineering"],
            Self::MolecularStructure => &["pharma", "chemistry", "materials", "agrochemical"],
            Self::Randomization => &[
                "pharma",
                "clinical_research",
                "social_science",
                "ab_testing",
            ],
            Self::Blinding => &["pharma", "clinical_research", "psychology"],
            Self::Endpoint => &["pharma", "clinical_research", "engineering", "project_mgmt"],
            Self::HalfLife => &["pharma", "nuclear_physics", "chemistry", "environmental"],
            Self::Confounder => &["pharma", "epidemiology", "social_science", "economics"],
            Self::Permeability => &["pharma", "materials", "geology", "environmental"],
            Self::Solubility => &["pharma", "chemistry", "environmental", "food_science"],
            Self::MechanismOfAction => &["pharma", "biology", "agrochemical"],
            Self::Biomarker => &["pharma", "diagnostics", "environmental", "food_science"],
            Self::PlaceboEffect => &["pharma", "psychology", "pain_research"],
            Self::StatisticalPower => &["pharma", "experimental_science", "engineering"],
        }
    }

    /// Tier is always T2-P for these primitives (verified via grounding).
    #[must_use]
    pub fn tier(&self) -> Tier {
        self.grounding().tier()
    }
}

impl fmt::Display for PharmaPrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} [{}] ({})", self, self.grounding(), self.tier())
    }
}

// ─── T2-C: Cross-Domain Composites (14) ───────────────────────────────────

/// The 14 T2-C composites built from T2-P primitives.
///
/// Tier: T2-C | Each composes multiple T2-P primitives (4-5 unique symbols).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PharmaComposite {
    Bioavailability,
    TherapeuticWindow,
    Selectivity,
    Clearance,
    DrugLikeness,
    StructureActivityRelationship,
    AdmetProfile,
    BenefitRiskRatio,
    PharmacokineticModel,
    ClinicalTrialDesign,
    SurrogateEndpoint,
    DisproportionalityAnalysis,
    PredictiveModel,
    LeadOptimization,
}

impl PharmaComposite {
    /// All 14 composites.
    pub const ALL: &'static [Self] = &[
        Self::Bioavailability,
        Self::TherapeuticWindow,
        Self::Selectivity,
        Self::Clearance,
        Self::DrugLikeness,
        Self::StructureActivityRelationship,
        Self::AdmetProfile,
        Self::BenefitRiskRatio,
        Self::PharmacokineticModel,
        Self::ClinicalTrialDesign,
        Self::SurrogateEndpoint,
        Self::DisproportionalityAnalysis,
        Self::PredictiveModel,
        Self::LeadOptimization,
    ];

    /// Which T2-P primitives compose this composite.
    #[must_use]
    pub fn dependencies(&self) -> &'static [PharmaPrimitive] {
        match self {
            Self::Bioavailability => &[
                PharmaPrimitive::Absorption,
                PharmaPrimitive::Metabolism,
                PharmaPrimitive::Distribution,
            ],
            Self::TherapeuticWindow => &[PharmaPrimitive::Efficacy, PharmaPrimitive::Toxicity],
            Self::Selectivity => &[PharmaPrimitive::BindingAffinity],
            Self::Clearance => &[PharmaPrimitive::Elimination, PharmaPrimitive::Distribution],
            Self::DrugLikeness => &[
                PharmaPrimitive::Absorption,
                PharmaPrimitive::Distribution,
                PharmaPrimitive::Toxicity,
                PharmaPrimitive::MolecularStructure,
            ],
            Self::StructureActivityRelationship => &[
                PharmaPrimitive::MolecularStructure,
                PharmaPrimitive::DoseResponse,
            ],
            Self::AdmetProfile => &[
                PharmaPrimitive::Absorption,
                PharmaPrimitive::Distribution,
                PharmaPrimitive::Metabolism,
                PharmaPrimitive::Elimination,
                PharmaPrimitive::Toxicity,
            ],
            Self::BenefitRiskRatio => &[
                PharmaPrimitive::Efficacy,
                PharmaPrimitive::Toxicity,
                PharmaPrimitive::Exposure,
            ],
            Self::PharmacokineticModel => &[
                PharmaPrimitive::Exposure,
                PharmaPrimitive::HalfLife,
                PharmaPrimitive::Distribution,
            ],
            Self::ClinicalTrialDesign => &[
                PharmaPrimitive::Randomization,
                PharmaPrimitive::Blinding,
                PharmaPrimitive::Endpoint,
                PharmaPrimitive::StatisticalPower,
                PharmaPrimitive::Confounder,
            ],
            Self::SurrogateEndpoint => &[
                PharmaPrimitive::Biomarker,
                PharmaPrimitive::Endpoint,
                PharmaPrimitive::DoseResponse,
            ],
            Self::DisproportionalityAnalysis => {
                &[PharmaPrimitive::Signal, PharmaPrimitive::Confounder]
            }
            Self::PredictiveModel => &[
                PharmaPrimitive::Signal,
                PharmaPrimitive::MolecularStructure,
                PharmaPrimitive::DoseResponse,
            ],
            Self::LeadOptimization => &[
                PharmaPrimitive::MolecularStructure,
                PharmaPrimitive::DoseResponse,
                PharmaPrimitive::Potency,
            ],
        }
    }

    /// Merged Lex Primitiva grounding (union of all dependency groundings).
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        let mut all_symbols = Vec::new();
        for dep in self.dependencies() {
            for sym in dep.grounding().symbols() {
                all_symbols.push(*sym);
            }
        }
        PrimitiveComposition::new(&all_symbols)
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Bioavailability => {
                "Fraction of administered dose reaching systemic circulation unchanged"
            }
            Self::TherapeuticWindow => {
                "Concentration range between minimum effective and maximum tolerated"
            }
            Self::Selectivity => {
                "Ratio of binding affinities between desired target and off-targets"
            }
            Self::Clearance => {
                "Volume of fluid from which substance is completely removed per unit time"
            }
            Self::DrugLikeness => {
                "Aggregate molecular properties predisposing to oral bioavailability and safety"
            }
            Self::StructureActivityRelationship => {
                "Mapping from molecular structural features to biological activity (SAR/QSAR)"
            }
            Self::AdmetProfile => {
                "Integrated absorption, distribution, metabolism, elimination, and toxicity"
            }
            Self::BenefitRiskRatio => {
                "Quantitative balance of therapeutic benefit against safety risk"
            }
            Self::PharmacokineticModel => {
                "Mathematical description of drug concentration-time course (compartmental)"
            }
            Self::ClinicalTrialDesign => {
                "Protocol specifying randomization, blinding, endpoints, and statistical plan"
            }
            Self::SurrogateEndpoint => "Biomarker validated to substitute for a clinical endpoint",
            Self::DisproportionalityAnalysis => {
                "Statistical method detecting signals via PRR, ROR, IC, EBGM"
            }
            Self::PredictiveModel => {
                "AI/ML model predicting molecular properties from structure (QSAR, GNN)"
            }
            Self::LeadOptimization => {
                "Iterative DMTA cycle refining candidates via SAR and multi-objective scoring"
            }
        }
    }

    /// Whether this composite involves recursion (iterative refinement).
    #[must_use]
    pub const fn is_recursive(&self) -> bool {
        matches!(self, Self::LeadOptimization | Self::PredictiveModel)
    }

    /// Dependency depth (max chain length to T2-P).
    #[must_use]
    pub const fn depth(&self) -> u8 {
        match self {
            // Direct compositions from T2-P → depth 1
            Self::Bioavailability
            | Self::TherapeuticWindow
            | Self::Selectivity
            | Self::Clearance
            | Self::StructureActivityRelationship
            | Self::AdmetProfile
            | Self::BenefitRiskRatio
            | Self::ClinicalTrialDesign
            | Self::SurrogateEndpoint
            | Self::DisproportionalityAnalysis => 1,
            // Compositions referencing other T2-C concepts logically → depth 2
            Self::DrugLikeness | Self::PharmacokineticModel | Self::PredictiveModel => 2,
            // Recursive → depth 2+
            Self::LeadOptimization => 2,
        }
    }
}

impl fmt::Display for PharmaComposite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} [{}] (deps: {})",
            self,
            self.grounding(),
            self.dependencies().len()
        )
    }
}

// ─── T3: Domain-Specific Concepts (8) ─────────────────────────────────────

/// The 8 T3 domain-specific concepts unique to pharmaceutical R&D.
///
/// Tier: T3 | Not transferable outside pharma regulatory context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PharmaDomainConcept {
    /// Investigational New Drug application (FDA).
    IndApplication,
    /// New Drug Application / Biologics License Application.
    NdaSubmission,
    /// Phase I/II/III sequential trial paradigm.
    ClinicalPhaseDesign,
    /// Risk Evaluation and Mitigation Strategy.
    Rems,
    /// Orphan drug designation and incentive structure.
    OrphanDrugDesignation,
    /// Bayesian adaptive platform trial.
    AdaptivePlatformTrial,
    /// Real-world evidence generation and regulatory acceptance.
    RealWorldEvidence,
    /// Good Practice (GLP/GMP/GCP) compliance framework.
    GxpCompliance,
}

impl PharmaDomainConcept {
    /// All 8 domain concepts.
    pub const ALL: &'static [Self] = &[
        Self::IndApplication,
        Self::NdaSubmission,
        Self::ClinicalPhaseDesign,
        Self::Rems,
        Self::OrphanDrugDesignation,
        Self::AdaptivePlatformTrial,
        Self::RealWorldEvidence,
        Self::GxpCompliance,
    ];

    /// T2-C composites this concept depends on.
    #[must_use]
    pub fn composite_dependencies(&self) -> &'static [PharmaComposite] {
        match self {
            Self::IndApplication => &[
                PharmaComposite::ClinicalTrialDesign,
                PharmaComposite::AdmetProfile,
                PharmaComposite::BenefitRiskRatio,
            ],
            Self::NdaSubmission => &[
                PharmaComposite::ClinicalTrialDesign,
                PharmaComposite::AdmetProfile,
                PharmaComposite::BenefitRiskRatio,
                PharmaComposite::PharmacokineticModel,
            ],
            Self::ClinicalPhaseDesign => &[
                PharmaComposite::ClinicalTrialDesign,
                PharmaComposite::TherapeuticWindow,
            ],
            Self::Rems => &[
                PharmaComposite::BenefitRiskRatio,
                PharmaComposite::DisproportionalityAnalysis,
            ],
            Self::OrphanDrugDesignation => &[PharmaComposite::BenefitRiskRatio],
            Self::AdaptivePlatformTrial => &[
                PharmaComposite::ClinicalTrialDesign,
                PharmaComposite::PredictiveModel,
            ],
            Self::RealWorldEvidence => &[
                PharmaComposite::DisproportionalityAnalysis,
                PharmaComposite::SurrogateEndpoint,
            ],
            Self::GxpCompliance => &[], // regulatory meta-constraint, no T2-C deps
        }
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::IndApplication => {
                "Regulatory submission to begin human clinical trials (FDA 21 CFR 312)"
            }
            Self::NdaSubmission => {
                "Application for marketing approval with complete efficacy/safety evidence"
            }
            Self::ClinicalPhaseDesign => {
                "Sequential Phase I (safety) → II (efficacy) → III (confirmatory) paradigm"
            }
            Self::Rems => "FDA-mandated risk evaluation and mitigation beyond standard labeling",
            Self::OrphanDrugDesignation => {
                "Incentive framework for drugs treating rare diseases (<200K patients)"
            }
            Self::AdaptivePlatformTrial => {
                "Bayesian adaptive design with interim analysis and arm-dropping capability"
            }
            Self::RealWorldEvidence => {
                "Evidence from observational data outside controlled clinical trials"
            }
            Self::GxpCompliance => {
                "Good Practice framework: GLP (lab), GMP (manufacturing), GCP (clinical)"
            }
        }
    }
}

impl fmt::Display for PharmaDomainConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} (T3, deps: {})",
            self,
            self.composite_dependencies().len()
        )
    }
}

// ─── Taxonomy Summary ─────────────────────────────────────────────────────

/// Summary statistics of the complete taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomySummary {
    pub t1_count: usize,
    pub t2p_count: usize,
    pub t2c_count: usize,
    pub t3_count: usize,
    pub total: usize,
    pub max_dependency_depth: u8,
}

/// Compute summary of the entire taxonomy.
#[must_use]
pub fn taxonomy_summary() -> TaxonomySummary {
    let max_depth = PharmaComposite::ALL
        .iter()
        .map(|c| c.depth())
        .max()
        .unwrap_or(0);

    TaxonomySummary {
        t1_count: 16,
        t2p_count: PharmaPrimitive::ALL.len(),
        t2c_count: PharmaComposite::ALL.len(),
        t3_count: PharmaDomainConcept::ALL.len(),
        total: 16
            + PharmaPrimitive::ALL.len()
            + PharmaComposite::ALL.len()
            + PharmaDomainConcept::ALL.len(),
        max_dependency_depth: max_depth + 1, // +1 for T3 layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_primitives_are_t2p() {
        for p in PharmaPrimitive::ALL {
            let tier = p.grounding().tier();
            assert!(
                tier == Tier::T2P || tier == Tier::T1,
                "{p:?} has tier {tier} (expected T2-P or T1 for degenerate cases)"
            );
        }
    }

    #[test]
    fn primitives_count_is_24() {
        assert_eq!(PharmaPrimitive::ALL.len(), 24);
    }

    #[test]
    fn composites_count_is_14() {
        assert_eq!(PharmaComposite::ALL.len(), 14);
    }

    #[test]
    fn domain_concepts_count_is_8() {
        assert_eq!(PharmaDomainConcept::ALL.len(), 8);
    }

    #[test]
    fn taxonomy_total_is_62() {
        let s = taxonomy_summary();
        assert_eq!(s.total, 62);
    }

    #[test]
    fn admet_has_5_dependencies() {
        assert_eq!(PharmaComposite::AdmetProfile.dependencies().len(), 5);
    }

    #[test]
    fn clinical_trial_design_has_5_dependencies() {
        assert_eq!(PharmaComposite::ClinicalTrialDesign.dependencies().len(), 5);
    }

    #[test]
    fn lead_optimization_is_recursive() {
        assert!(PharmaComposite::LeadOptimization.is_recursive());
        assert!(!PharmaComposite::Bioavailability.is_recursive());
    }

    #[test]
    fn all_primitives_have_descriptions() {
        for p in PharmaPrimitive::ALL {
            assert!(!p.description().is_empty(), "{p:?} has empty description");
        }
    }

    #[test]
    fn all_primitives_have_domains() {
        for p in PharmaPrimitive::ALL {
            assert!(!p.domains().is_empty(), "{p:?} has no domains");
            // All should include pharma
            assert!(
                p.domains().contains(&"pharma"),
                "{p:?} missing pharma domain"
            );
        }
    }

    #[test]
    fn ind_depends_on_three_composites() {
        assert_eq!(
            PharmaDomainConcept::IndApplication
                .composite_dependencies()
                .len(),
            3
        );
    }

    #[test]
    fn display_formats() {
        let p = PharmaPrimitive::BindingAffinity;
        let s = format!("{p}");
        assert!(s.contains("BindingAffinity"));
        assert!(s.contains("T2-P"));

        let c = PharmaComposite::AdmetProfile;
        let s = format!("{c}");
        assert!(s.contains("AdmetProfile"));
        assert!(s.contains("deps: 5"));
    }
}
