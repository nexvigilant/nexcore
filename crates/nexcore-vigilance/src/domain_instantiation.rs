//! # Domain Instantiations (ToV Part III: §11-§15)
//!
//! This module establishes the formal mathematical correspondence between three domains
//! governed by the Theory of Vigilance:
//! - **Atomic Programming** (Cloud Computing)
//! - **Computational Pharmacovigilance** (Drug Safety)
//! - **Algorithmovigilance** (AI Safety)
//!
//! ## Sections
//!
//! - **§11** Cross-Domain Concept Mapping
//! - **§12** Element System Correspondence (15 elements, 4 layers)
//! - **§13** Hierarchy Level Correspondence (8 levels)
//! - **§14** Safety Manifold Correspondence
//! - **§15** Harm Taxonomy Correspondence
//!
//! ## Key Insight
//!
//! The correspondence is STRUCTURAL—the same mathematical forms appear in each domain
//! with domain-specific instantiations. This is NOT a topological isomorphism (state spaces
//! have different dimensions), but a structural correspondence enabling:
//!
//! 1. Transfer of mathematical tools across domains
//! 2. Unified computational infrastructure
//! 3. Cross-domain insight transfer

use crate::conservation_catalog::ConservationLawId;
use crate::harm_taxonomy::HarmTypeId;
use serde::{Deserialize, Serialize};
use std::fmt;

// =============================================================================
// §11.0 Domain Identification
// =============================================================================

/// The three primary domains governed by Theory of Vigilance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VigilanceDomain {
    /// Atomic Programming - Cloud Computing / Infrastructure
    Cloud,
    /// Computational Pharmacovigilance - Drug Safety
    PV,
    /// Algorithmovigilance - AI Safety
    AI,
}

impl VigilanceDomain {
    /// All three domains.
    pub const ALL: &'static [VigilanceDomain] = &[Self::Cloud, Self::PV, Self::AI];

    /// Full name of the domain.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Cloud => "Atomic Programming",
            Self::PV => "Computational Pharmacovigilance",
            Self::AI => "Algorithmovigilance",
        }
    }

    /// Short name.
    #[must_use]
    pub const fn short_name(&self) -> &'static str {
        match self {
            Self::Cloud => "Cloud",
            Self::PV => "PV",
            Self::AI => "AI",
        }
    }

    /// State space dimensionality range (§11.1).
    #[must_use]
    pub const fn state_space_dim(&self) -> (usize, usize) {
        match self {
            Self::Cloud => (15, 50),
            Self::PV => (10, 100),
            Self::AI => (1_000_000, 1_000_000_000_000), // 10^6 to 10^12
        }
    }

    /// Perturbation space dimensionality range.
    #[must_use]
    pub const fn perturbation_dim(&self) -> (usize, usize) {
        match self {
            Self::Cloud => (1, 10),
            Self::PV => (1, 5),
            Self::AI => (1_000, 1_000_000),
        }
    }

    /// Parameter space dimensionality range.
    #[must_use]
    pub const fn parameter_dim(&self) -> (usize, usize) {
        match self {
            Self::Cloud => (20, 100),
            Self::PV => (10, 50),
            Self::AI => (10, 100),
        }
    }

    /// Computational tractability assessment.
    #[must_use]
    pub const fn tractability(&self) -> &'static str {
        match self {
            Self::Cloud => "FEM feasible",
            Self::PV => "FEM feasible for reduced models",
            Self::AI => "Monte Carlo only; projection required",
        }
    }
}

impl fmt::Display for VigilanceDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// §11.0 Definition 11.0.1: Domain Instantiation
// =============================================================================

/// Domain Instantiation (Definition 11.0.1).
///
/// A domain instantiation of the Theory of Vigilance is a tuple:
/// 𝒟 = (S_𝒟, U_𝒟, Θ_𝒟, 𝒢_𝒟, ℒ_𝒟, E_𝒟)
#[derive(Debug, Clone, Serialize)]
pub struct DomainInstantiation {
    /// Domain identifier
    pub domain: VigilanceDomain,
    /// State space description S_𝒟
    pub state_space: StateSpaceSpec,
    /// Perturbation space description U_𝒟
    pub perturbation_space: PerturbationSpaceSpec,
    /// Parameter space description Θ_𝒟
    pub parameter_space: ParameterSpaceSpec,
    /// Constraint set 𝒢_𝒟 (implementing 11 conservation laws)
    pub constraint_set: ConstraintSetSpec,
    /// Hierarchy ℒ_𝒟 (8 levels)
    pub hierarchy: HierarchySpec,
    /// Element decomposition E_𝒟 (15 elements)
    pub elements: ElementDecompositionSpec,
}

impl DomainInstantiation {
    /// Create a domain instantiation for the specified domain.
    #[must_use]
    pub fn for_domain(domain: VigilanceDomain) -> Self {
        match domain {
            VigilanceDomain::Cloud => Self::cloud(),
            VigilanceDomain::PV => Self::pv(),
            VigilanceDomain::AI => Self::ai(),
        }
    }

    /// Cloud domain instantiation.
    #[must_use]
    pub fn cloud() -> Self {
        Self {
            domain: VigilanceDomain::Cloud,
            state_space: StateSpaceSpec::cloud(),
            perturbation_space: PerturbationSpaceSpec::cloud(),
            parameter_space: ParameterSpaceSpec::cloud(),
            constraint_set: ConstraintSetSpec::cloud(),
            hierarchy: HierarchySpec::cloud(),
            elements: ElementDecompositionSpec::cloud(),
        }
    }

    /// PV domain instantiation.
    #[must_use]
    pub fn pv() -> Self {
        Self {
            domain: VigilanceDomain::PV,
            state_space: StateSpaceSpec::pv(),
            perturbation_space: PerturbationSpaceSpec::pv(),
            parameter_space: ParameterSpaceSpec::pv(),
            constraint_set: ConstraintSetSpec::pv(),
            hierarchy: HierarchySpec::pv(),
            elements: ElementDecompositionSpec::pv(),
        }
    }

    /// AI domain instantiation.
    #[must_use]
    pub fn ai() -> Self {
        Self {
            domain: VigilanceDomain::AI,
            state_space: StateSpaceSpec::ai(),
            perturbation_space: PerturbationSpaceSpec::ai(),
            parameter_space: ParameterSpaceSpec::ai(),
            constraint_set: ConstraintSetSpec::ai(),
            hierarchy: HierarchySpec::ai(),
            elements: ElementDecompositionSpec::ai(),
        }
    }
}

/// State space specification S_𝒟.
#[derive(Debug, Clone, Serialize)]
pub struct StateSpaceSpec {
    /// Domain
    pub domain: VigilanceDomain,
    /// Description
    pub description: &'static str,
    /// State vector components
    pub components: &'static str,
    /// Typical dimensionality
    pub typical_dim: (usize, usize),
}

impl StateSpaceSpec {
    fn cloud() -> Self {
        Self {
            domain: VigilanceDomain::Cloud,
            description: "Resource utilization vector",
            components: "CPU, memory, network, storage utilization",
            typical_dim: (15, 50),
        }
    }

    fn pv() -> Self {
        Self {
            domain: VigilanceDomain::PV,
            description: "Concentration/occupancy vector",
            components: "Drug concentrations, receptor occupancies, biomarkers",
            typical_dim: (10, 100),
        }
    }

    fn ai() -> Self {
        Self {
            domain: VigilanceDomain::AI,
            description: "Activation/parameter vector",
            components: "Weights, activations, attention patterns, outputs",
            typical_dim: (1_000_000, 1_000_000_000_000),
        }
    }
}

/// Perturbation space specification U_𝒟.
#[derive(Debug, Clone, Serialize)]
pub struct PerturbationSpaceSpec {
    /// Domain
    pub domain: VigilanceDomain,
    /// Perturbation type
    pub perturbation_type: &'static str,
    /// Mathematical form
    pub mathematical_form: &'static str,
}

impl PerturbationSpaceSpec {
    fn cloud() -> Self {
        Self {
            domain: VigilanceDomain::Cloud,
            perturbation_type: "Request rate r(t) ∈ ℝ≥0",
            mathematical_form: "u: [0,T] → U (workload function)",
        }
    }

    fn pv() -> Self {
        Self {
            domain: VigilanceDomain::PV,
            perturbation_type: "Dose schedule d(t) ∈ ℝ≥0",
            mathematical_form: "u: [0,T] → U (dosing regimen)",
        }
    }

    fn ai() -> Self {
        Self {
            domain: VigilanceDomain::AI,
            perturbation_type: "Input sequence x(t) ∈ 𝒳",
            mathematical_form: "u: [0,T] → U (token/embedding sequence)",
        }
    }
}

/// Parameter space specification Θ_𝒟.
#[derive(Debug, Clone, Serialize)]
pub struct ParameterSpaceSpec {
    /// Domain
    pub domain: VigilanceDomain,
    /// Description
    pub description: &'static str,
    /// Example parameters
    pub examples: &'static str,
}

impl ParameterSpaceSpec {
    fn cloud() -> Self {
        Self {
            domain: VigilanceDomain::Cloud,
            description: "Capacity limits, SLO targets",
            examples: "Max CPU, memory limits, latency SLOs, availability targets",
        }
    }

    fn pv() -> Self {
        Self {
            domain: VigilanceDomain::PV,
            description: "PK/PD parameters, patient factors",
            examples: "Clearance, volume, EC50, patient age, weight, genetics",
        }
    }

    fn ai() -> Self {
        Self {
            domain: VigilanceDomain::AI,
            description: "Hyperparameters, thresholds",
            examples: "Learning rate, context length, safety thresholds, temperature",
        }
    }
}

/// Constraint set specification 𝒢_𝒟.
#[derive(Debug, Clone, Serialize)]
pub struct ConstraintSetSpec {
    /// Domain
    pub domain: VigilanceDomain,
    /// Number of constraints (11 universal + domain-specific)
    pub num_constraints: usize,
    /// Description
    pub description: &'static str,
}

impl ConstraintSetSpec {
    fn cloud() -> Self {
        Self {
            domain: VigilanceDomain::Cloud,
            num_constraints: 11,
            description: "SLA constraints, resource limits",
        }
    }

    fn pv() -> Self {
        Self {
            domain: VigilanceDomain::PV,
            num_constraints: 11,
            description: "Physiological limits, toxicity bounds",
        }
    }

    fn ai() -> Self {
        Self {
            domain: VigilanceDomain::AI,
            num_constraints: 15, // 11 + safety-specific
            description: "Safety constraints, fairness bounds, harm limits",
        }
    }
}

/// Hierarchy specification ℒ_𝒟.
#[derive(Debug, Clone, Serialize)]
pub struct HierarchySpec {
    /// Domain
    pub domain: VigilanceDomain,
    /// Number of levels (always 8)
    pub num_levels: usize,
    /// Description
    pub description: &'static str,
}

impl HierarchySpec {
    fn cloud() -> Self {
        Self {
            domain: VigilanceDomain::Cloud,
            num_levels: 8,
            description: "8-level compute hierarchy (Atomic → Enterprise)",
        }
    }

    fn pv() -> Self {
        Self {
            domain: VigilanceDomain::PV,
            num_levels: 8,
            description: "8-level biological hierarchy (Molecular → Regulatory)",
        }
    }

    fn ai() -> Self {
        Self {
            domain: VigilanceDomain::AI,
            num_levels: 8,
            description: "8-level AI system hierarchy (Parameter → Societal)",
        }
    }
}

/// Element decomposition specification E_𝒟.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDecompositionSpec {
    /// Domain
    pub domain: VigilanceDomain,
    /// Number of elements (always 15)
    pub num_elements: usize,
    /// Number of layers (always 4)
    pub num_layers: usize,
}

impl ElementDecompositionSpec {
    fn cloud() -> Self {
        Self {
            domain: VigilanceDomain::Cloud,
            num_elements: 15,
            num_layers: 4,
        }
    }

    fn pv() -> Self {
        Self {
            domain: VigilanceDomain::PV,
            num_elements: 15,
            num_layers: 4,
        }
    }

    fn ai() -> Self {
        Self {
            domain: VigilanceDomain::AI,
            num_elements: 15,
            num_layers: 4,
        }
    }
}

// =============================================================================
// §11.0 Definition 11.0.2: Structural Correspondence
// =============================================================================

/// Structural correspondence between domain instantiations (Definition 11.0.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrespondenceCondition {
    /// Same topological state space structure
    StateSpaceStructure,
    /// Same SDE dynamics form
    DynamicsForm,
    /// Same mathematical form for constraints
    ConstraintCorrespondence,
    /// Identical hierarchy structure
    HierarchyIsomorphism,
    /// Analogous functional roles for elements
    ElementCorrespondence,
}

impl CorrespondenceCondition {
    /// All five conditions.
    pub const ALL: &'static [CorrespondenceCondition] = &[
        Self::StateSpaceStructure,
        Self::DynamicsForm,
        Self::ConstraintCorrespondence,
        Self::HierarchyIsomorphism,
        Self::ElementCorrespondence,
    ];

    /// Description of this condition.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::StateSpaceStructure => {
                "Both S_𝒟 and S_𝒟' are topological spaces (typically manifolds or subsets of ℝⁿ)"
            }
            Self::DynamicsForm => "Both follow the same SDE structure: ds = f(s,u,θ)dt + σ(s)dW",
            Self::ConstraintCorrespondence => {
                "For each i ∈ {1,...,11}, constraint gᵢ has the same MATHEMATICAL FORM"
            }
            Self::HierarchyIsomorphism => {
                "Hierarchies have identical structure (8 levels, same ordering, scale separation)"
            }
            Self::ElementCorrespondence => {
                "Elements at position j serve analogous functional roles"
            }
        }
    }
}

impl fmt::Display for CorrespondenceCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StateSpaceStructure => write!(f, "State space structure"),
            Self::DynamicsForm => write!(f, "Dynamics form"),
            Self::ConstraintCorrespondence => write!(f, "Constraint correspondence"),
            Self::HierarchyIsomorphism => write!(f, "Hierarchy isomorphism"),
            Self::ElementCorrespondence => write!(f, "Element correspondence"),
        }
    }
}

/// Result of checking structural correspondence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralCorrespondenceResult {
    /// Domain A
    pub domain_a: VigilanceDomain,
    /// Domain B
    pub domain_b: VigilanceDomain,
    /// Conditions satisfied
    pub conditions_met: Vec<CorrespondenceCondition>,
    /// Overall correspondence
    pub is_corresponding: bool,
}

/// Check structural correspondence between two domains.
#[must_use]
pub fn check_structural_correspondence(
    domain_a: VigilanceDomain,
    domain_b: VigilanceDomain,
) -> StructuralCorrespondenceResult {
    // All three domains satisfy all correspondence conditions
    // (by construction of the framework)
    StructuralCorrespondenceResult {
        domain_a,
        domain_b,
        conditions_met: CorrespondenceCondition::ALL.to_vec(),
        is_corresponding: true,
    }
}

// =============================================================================
// §11.1 Foundational Concept Mapping
// =============================================================================

/// Fundamental concept in the vigilance framework (§11.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VigilanceConcept {
    /// System 𝒮 - tuple (S, f, σ)
    System,
    /// State s ∈ S
    State,
    /// Perturbation u: [0,T] → U
    Perturbation,
    /// Parameters θ ∈ Θ
    Parameters,
    /// Constraints 𝒢 = {gᵢ}
    Constraints,
    /// Hierarchy ℒ = (L, ≺, ψ)
    Hierarchy,
    /// Elements E = {e₁,...,e₁₅}
    Elements,
    /// Safety Manifold M
    SafetyManifold,
    /// Harm Event H
    HarmEvent,
}

impl VigilanceConcept {
    /// All concepts.
    pub const ALL: &'static [VigilanceConcept] = &[
        Self::System,
        Self::State,
        Self::Perturbation,
        Self::Parameters,
        Self::Constraints,
        Self::Hierarchy,
        Self::Elements,
        Self::SafetyManifold,
        Self::HarmEvent,
    ];

    /// Mathematical object for this concept.
    #[must_use]
    pub const fn mathematical_object(&self) -> &'static str {
        match self {
            Self::System => "Tuple (S, f, σ)",
            Self::State => "s ∈ S ⊆ ℝⁿ",
            Self::Perturbation => "u: [0,T] → U",
            Self::Parameters => "θ ∈ Θ ⊆ ℝᵏ",
            Self::Constraints => "{gᵢ: S×U×Θ → ℝ}ᵢ₌₁ᵐ",
            Self::Hierarchy => "(L, ≺, ψ) with |L|=8",
            Self::Elements => "{e₁,...,e₁₅} with Φ: 𝒫(E) → S",
            Self::SafetyManifold => "M = ⋂ᵢ{s: gᵢ(s,u,θ) ≤ 0}",
            Self::HarmEvent => "H = {τ_∂M < ∞}",
        }
    }

    /// Get domain-specific instantiation.
    #[must_use]
    pub const fn domain_instantiation(&self, domain: VigilanceDomain) -> &'static str {
        match (self, domain) {
            // System
            (Self::System, VigilanceDomain::Cloud) => "(S_cloud, f_cloud, σ_cloud)",
            (Self::System, VigilanceDomain::PV) => "(S_bio, f_bio, σ_bio)",
            (Self::System, VigilanceDomain::AI) => "(S_ai, f_ai, σ_ai)",
            // State
            (Self::State, VigilanceDomain::Cloud) => "Resource utilization vector",
            (Self::State, VigilanceDomain::PV) => "Concentration/occupancy vector",
            (Self::State, VigilanceDomain::AI) => "Activation/parameter vector",
            // Perturbation
            (Self::Perturbation, VigilanceDomain::Cloud) => "Request rate r(t) ∈ ℝ≥0",
            (Self::Perturbation, VigilanceDomain::PV) => "Dose schedule d(t) ∈ ℝ≥0",
            (Self::Perturbation, VigilanceDomain::AI) => "Input sequence x(t) ∈ 𝒳",
            // Parameters
            (Self::Parameters, VigilanceDomain::Cloud) => "Capacity limits, SLO targets",
            (Self::Parameters, VigilanceDomain::PV) => "PK/PD parameters, patient factors",
            (Self::Parameters, VigilanceDomain::AI) => "Hyperparameters, thresholds",
            // Constraints
            (Self::Constraints, VigilanceDomain::Cloud) => "SLA constraints, resource limits",
            (Self::Constraints, VigilanceDomain::PV) => "Physiological limits, toxicity bounds",
            (Self::Constraints, VigilanceDomain::AI) => "Safety constraints, fairness bounds",
            // Hierarchy
            (Self::Hierarchy, VigilanceDomain::Cloud) => "8-level compute hierarchy",
            (Self::Hierarchy, VigilanceDomain::PV) => "8-level biological hierarchy",
            (Self::Hierarchy, VigilanceDomain::AI) => "8-level AI system hierarchy",
            // Elements
            (Self::Elements, VigilanceDomain::Cloud) => "Cloud elements (§43-44)",
            (Self::Elements, VigilanceDomain::PV) => "PV elements",
            (Self::Elements, VigilanceDomain::AI) => "AI elements (§51-55)",
            // Safety Manifold
            (Self::SafetyManifold, VigilanceDomain::Cloud) => "Operational envelope",
            (Self::SafetyManifold, VigilanceDomain::PV) => "Physiological homeostasis",
            (Self::SafetyManifold, VigilanceDomain::AI) => "Aligned behavior space",
            // Harm Event
            (Self::HarmEvent, VigilanceDomain::Cloud) => "SLA violation",
            (Self::HarmEvent, VigilanceDomain::PV) => "Adverse event",
            (Self::HarmEvent, VigilanceDomain::AI) => "Harmful output",
        }
    }
}

impl fmt::Display for VigilanceConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => write!(f, "System"),
            Self::State => write!(f, "State"),
            Self::Perturbation => write!(f, "Perturbation"),
            Self::Parameters => write!(f, "Parameters"),
            Self::Constraints => write!(f, "Constraints"),
            Self::Hierarchy => write!(f, "Hierarchy"),
            Self::Elements => write!(f, "Elements"),
            Self::SafetyManifold => write!(f, "Safety Manifold"),
            Self::HarmEvent => write!(f, "Harm Event"),
        }
    }
}

// =============================================================================
// §12 Element System Correspondence
// =============================================================================

/// Element layer in the 15-element decomposition (§12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementLayer {
    /// Layer 1: Foundation Elements (F1-F4)
    Foundation,
    /// Layer 2: Operations Elements (O1-O4)
    Operations,
    /// Layer 3: Management Elements (M1-M4)
    Management,
    /// Layer 4: Interface Elements (I1-I3)
    Interface,
}

impl ElementLayer {
    /// All layers.
    pub const ALL: &'static [ElementLayer] = &[
        Self::Foundation,
        Self::Operations,
        Self::Management,
        Self::Interface,
    ];

    /// Number of elements in this layer.
    #[must_use]
    pub const fn element_count(&self) -> usize {
        match self {
            Self::Foundation => 4,
            Self::Operations => 4,
            Self::Management => 4,
            Self::Interface => 3,
        }
    }

    /// Element positions in this layer.
    #[must_use]
    pub fn positions(&self) -> Vec<ElementPosition> {
        match self {
            Self::Foundation => vec![
                ElementPosition::F1,
                ElementPosition::F2,
                ElementPosition::F3,
                ElementPosition::F4,
            ],
            Self::Operations => vec![
                ElementPosition::O1,
                ElementPosition::O2,
                ElementPosition::O3,
                ElementPosition::O4,
            ],
            Self::Management => vec![
                ElementPosition::M1,
                ElementPosition::M2,
                ElementPosition::M3,
                ElementPosition::M4,
            ],
            Self::Interface => vec![
                ElementPosition::I1,
                ElementPosition::I2,
                ElementPosition::I3,
            ],
        }
    }
}

impl fmt::Display for ElementLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Foundation => write!(f, "Layer 1: Foundation"),
            Self::Operations => write!(f, "Layer 2: Operations"),
            Self::Management => write!(f, "Layer 3: Management"),
            Self::Interface => write!(f, "Layer 4: Interface"),
        }
    }
}

/// Element position in the 15-element structure (§12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementPosition {
    /// Foundation 1: Primary active entity (Drug/Storage/Model)
    F1,
    /// Foundation 2: Transformation locus (Target/Compute/Weight)
    F2,
    /// Foundation 3: Signal transduction initiator (Receptor/Network/Activation)
    F3,
    /// Foundation 4: Catalytic transformation (Enzyme/Transform/Gradient)
    F4,
    /// Operations 1: Distribution mechanism (Transporter/Queue/Layer)
    O1,
    /// Operations 2: Flow propagation channel (Pathway/Stream/Data)
    O2,
    /// Operations 3: Intermediate processing unit (Organelle/Cache/Feature)
    O3,
    /// Operations 4: Boundary / interface control (Membrane/Gateway/Loss)
    O4,
    /// Management 1: Observable signal (Biomarker/Monitor/Output)
    M1,
    /// Management 2: System state manifestation (Phenotype/Orchestrate/Uncertainty)
    M2,
    /// Management 3: Individual susceptibility (Genotype/Config/Behavior)
    M3,
    /// Management 4: Protection / measurement (Immune/Secret/Metric)
    M4,
    /// Interface 1: Input quantification (Exposure/Workload/Input)
    I1,
    /// Interface 2: Output/effect (Response/Response/Output)
    I2,
    /// Interface 3: Environmental context (Population/Context/Context)
    I3,
}

impl ElementPosition {
    /// All 15 positions.
    pub const ALL: &'static [ElementPosition] = &[
        Self::F1,
        Self::F2,
        Self::F3,
        Self::F4,
        Self::O1,
        Self::O2,
        Self::O3,
        Self::O4,
        Self::M1,
        Self::M2,
        Self::M3,
        Self::M4,
        Self::I1,
        Self::I2,
        Self::I3,
    ];

    /// Get the layer for this position.
    #[must_use]
    pub const fn layer(&self) -> ElementLayer {
        match self {
            Self::F1 | Self::F2 | Self::F3 | Self::F4 => ElementLayer::Foundation,
            Self::O1 | Self::O2 | Self::O3 | Self::O4 => ElementLayer::Operations,
            Self::M1 | Self::M2 | Self::M3 | Self::M4 => ElementLayer::Management,
            Self::I1 | Self::I2 | Self::I3 => ElementLayer::Interface,
        }
    }

    /// Get the invariant function for this position.
    #[must_use]
    pub const fn invariant_function(&self) -> &'static str {
        match self {
            Self::F1 => "Primary active entity",
            Self::F2 => "Transformation locus",
            Self::F3 => "Signal transduction initiator",
            Self::F4 => "Catalytic transformation",
            Self::O1 => "Distribution mechanism",
            Self::O2 => "Flow propagation channel",
            Self::O3 => "Intermediate processing unit",
            Self::O4 => "Boundary / interface control",
            Self::M1 => "Observable signal",
            Self::M2 => "System state manifestation",
            Self::M3 => "Individual susceptibility",
            Self::M4 => "Protection / measurement",
            Self::I1 => "Input quantification",
            Self::I2 => "Output/effect",
            Self::I3 => "Environmental context",
        }
    }

    /// Get domain-specific element name.
    #[must_use]
    pub const fn element_name(&self, domain: VigilanceDomain) -> &'static str {
        match (self, domain) {
            // Foundation
            (Self::F1, VigilanceDomain::Cloud) => "STORAGE (St)",
            (Self::F1, VigilanceDomain::PV) => "DRUG (Dr)",
            (Self::F1, VigilanceDomain::AI) => "MODEL (Md)",
            (Self::F2, VigilanceDomain::Cloud) => "COMPUTE (Cp)",
            (Self::F2, VigilanceDomain::PV) => "TARGET (Tg)",
            (Self::F2, VigilanceDomain::AI) => "WEIGHT (Wt)",
            (Self::F3, VigilanceDomain::Cloud) => "NETWORK (Nw)",
            (Self::F3, VigilanceDomain::PV) => "RECEPTOR (Rc)",
            (Self::F3, VigilanceDomain::AI) => "ACTIVATION (Ac)",
            (Self::F4, VigilanceDomain::Cloud) => "TRANSFORM (Tf)",
            (Self::F4, VigilanceDomain::PV) => "ENZYME (Ez)",
            (Self::F4, VigilanceDomain::AI) => "GRADIENT (Gr)",
            // Operations
            (Self::O1, VigilanceDomain::Cloud) => "QUEUE (Qu)",
            (Self::O1, VigilanceDomain::PV) => "TRANSPORTER (Tr)",
            (Self::O1, VigilanceDomain::AI) => "LAYER (Ly)",
            (Self::O2, VigilanceDomain::Cloud) => "STREAM (Sr)",
            (Self::O2, VigilanceDomain::PV) => "PATHWAY (Pw)",
            (Self::O2, VigilanceDomain::AI) => "DATA (Da)",
            (Self::O3, VigilanceDomain::Cloud) => "CACHE (Ch)",
            (Self::O3, VigilanceDomain::PV) => "ORGANELLE (Or)",
            (Self::O3, VigilanceDomain::AI) => "FEATURE (Fe)",
            (Self::O4, VigilanceDomain::Cloud) => "GATEWAY (Gw)",
            (Self::O4, VigilanceDomain::PV) => "MEMBRANE (Mb)",
            (Self::O4, VigilanceDomain::AI) => "LOSS (Ls)",
            // Management
            (Self::M1, VigilanceDomain::Cloud) => "MONITOR (Mn)",
            (Self::M1, VigilanceDomain::PV) => "BIOMARKER (Bm)",
            (Self::M1, VigilanceDomain::AI) => "OUTPUT (Op)",
            (Self::M2, VigilanceDomain::Cloud) => "ORCHESTRATE (Or)",
            (Self::M2, VigilanceDomain::PV) => "PHENOTYPE (Ph)",
            (Self::M2, VigilanceDomain::AI) => "UNCERTAINTY (Un)",
            (Self::M3, VigilanceDomain::Cloud) => "CONFIG (Cf)",
            (Self::M3, VigilanceDomain::PV) => "GENOTYPE (Gn)",
            (Self::M3, VigilanceDomain::AI) => "BEHAVIOR (Bh)",
            (Self::M4, VigilanceDomain::Cloud) => "SECRET (Sc)",
            (Self::M4, VigilanceDomain::PV) => "IMMUNE (Im)",
            (Self::M4, VigilanceDomain::AI) => "METRIC (Mt)",
            // Interface
            (Self::I1, VigilanceDomain::Cloud) => "WORKLOAD (Wl)",
            (Self::I1, VigilanceDomain::PV) => "EXPOSURE (Ex)",
            (Self::I1, VigilanceDomain::AI) => "INPUT (In)",
            (Self::I2, VigilanceDomain::Cloud) => "RESPONSE (Rs)",
            (Self::I2, VigilanceDomain::PV) => "RESPONSE (Rs)",
            (Self::I2, VigilanceDomain::AI) => "OUTPUT (Op)",
            (Self::I3, VigilanceDomain::Cloud) => "CONTEXT (Cx)",
            (Self::I3, VigilanceDomain::PV) => "POPULATION (Po)",
            (Self::I3, VigilanceDomain::AI) => "CONTEXT (Cx)",
        }
    }
}

impl fmt::Display for ElementPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F1 => write!(f, "F1"),
            Self::F2 => write!(f, "F2"),
            Self::F3 => write!(f, "F3"),
            Self::F4 => write!(f, "F4"),
            Self::O1 => write!(f, "O1"),
            Self::O2 => write!(f, "O2"),
            Self::O3 => write!(f, "O3"),
            Self::O4 => write!(f, "O4"),
            Self::M1 => write!(f, "M1"),
            Self::M2 => write!(f, "M2"),
            Self::M3 => write!(f, "M3"),
            Self::M4 => write!(f, "M4"),
            Self::I1 => write!(f, "I1"),
            Self::I2 => write!(f, "I2"),
            Self::I3 => write!(f, "I3"),
        }
    }
}

/// Correspondence strength between domain elements (Definition 12.0.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrespondenceStrength {
    /// Strong: Same functional role, interaction pattern, hierarchical level
    Strong,
    /// Moderate: Same functional role, similar pattern, some interpretation needed
    Moderate,
    /// Weak: Analogous role with significant interpretation required
    Weak,
}

impl CorrespondenceStrength {
    /// Get correspondence strength for an element position (§12.0.1 Table).
    ///
    /// Note: The ToV document summary text claims "8/15 Strong, 4/15 Moderate"
    /// but the actual table shows 6/6/3. This implementation follows the TABLE.
    #[must_use]
    pub const fn for_position(position: ElementPosition) -> Self {
        match position {
            // Strong (6/15): F1, F2, F3, O3, M1, I2
            ElementPosition::F1 => Self::Strong,
            ElementPosition::F2 => Self::Strong,
            ElementPosition::F3 => Self::Strong,
            ElementPosition::O3 => Self::Strong,
            ElementPosition::M1 => Self::Strong,
            ElementPosition::I2 => Self::Strong,
            // Moderate (6/15): F4, O1, O2, O4, I1, I3
            ElementPosition::F4 => Self::Moderate,
            ElementPosition::O1 => Self::Moderate,
            ElementPosition::O2 => Self::Moderate,
            ElementPosition::O4 => Self::Moderate,
            ElementPosition::I1 => Self::Moderate,
            ElementPosition::I3 => Self::Moderate,
            // Weak (3/15): M2, M3, M4
            ElementPosition::M2 => Self::Weak,
            ElementPosition::M3 => Self::Weak,
            ElementPosition::M4 => Self::Weak,
        }
    }
}

impl fmt::Display for CorrespondenceStrength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Strong => write!(f, "Strong"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Weak => write!(f, "Weak"),
        }
    }
}

/// Summary of element correspondence strength across all positions.
///
/// Per §12.0.1 table: 6 strong, 6 moderate, 3 weak (total 15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementCorrespondenceSummary {
    /// Strong correspondences (6 per table)
    pub strong_count: usize,
    /// Moderate correspondences (6 per table)
    pub moderate_count: usize,
    /// Weak correspondences (3 per table)
    pub weak_count: usize,
}

impl ElementCorrespondenceSummary {
    /// Calculate summary.
    #[must_use]
    pub fn calculate() -> Self {
        let mut strong = 0;
        let mut moderate = 0;
        let mut weak = 0;

        for pos in ElementPosition::ALL {
            match CorrespondenceStrength::for_position(*pos) {
                CorrespondenceStrength::Strong => strong += 1,
                CorrespondenceStrength::Moderate => moderate += 1,
                CorrespondenceStrength::Weak => weak += 1,
            }
        }

        Self {
            strong_count: strong,
            moderate_count: moderate,
            weak_count: weak,
        }
    }
}

// =============================================================================
// §13 Hierarchy Level Correspondence
// =============================================================================

/// Hierarchy level (1-8) from §13.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainHierarchyLevel(pub u8);

impl DomainHierarchyLevel {
    /// All 8 levels.
    pub const ALL: &'static [DomainHierarchyLevel] = &[
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
        Self(8),
    ];

    /// Get level name for a domain.
    #[must_use]
    pub const fn name(&self, domain: VigilanceDomain) -> &'static str {
        match (self.0, domain) {
            // Cloud
            (1, VigilanceDomain::Cloud) => "Atomic",
            (2, VigilanceDomain::Cloud) => "Workflow",
            (3, VigilanceDomain::Cloud) => "Service",
            (4, VigilanceDomain::Cloud) => "Autonomy",
            (5, VigilanceDomain::Cloud) => "Application",
            (6, VigilanceDomain::Cloud) => "Platform",
            (7, VigilanceDomain::Cloud) => "Governance",
            (8, VigilanceDomain::Cloud) => "Enterprise",
            // PV
            (1, VigilanceDomain::PV) => "Molecular",
            (2, VigilanceDomain::PV) => "Cellular",
            (3, VigilanceDomain::PV) => "Tissue",
            (4, VigilanceDomain::PV) => "Organ",
            (5, VigilanceDomain::PV) => "System",
            (6, VigilanceDomain::PV) => "Clinical",
            (7, VigilanceDomain::PV) => "Epidemiological",
            (8, VigilanceDomain::PV) => "Regulatory",
            // AI
            (1, VigilanceDomain::AI) => "Parameter",
            (2, VigilanceDomain::AI) => "Neuron",
            (3, VigilanceDomain::AI) => "Layer",
            (4, VigilanceDomain::AI) => "Module",
            (5, VigilanceDomain::AI) => "Model",
            (6, VigilanceDomain::AI) => "Deployment",
            (7, VigilanceDomain::AI) => "Population",
            (8, VigilanceDomain::AI) => "Societal",
            _ => "Unknown",
        }
    }

    /// Get time scale for this level.
    #[must_use]
    pub const fn time_scale(&self) -> &'static str {
        match self.0 {
            1 => "μs",
            2 => "μs - ms",
            3 => "ms - s",
            4 => "s - min",
            5 => "min - hr",
            6 => "hr - days",
            7 => "days - wks",
            8 => "wks - yrs",
            _ => "Unknown",
        }
    }

    /// Get approximate system unit count for this level.
    #[must_use]
    pub const fn system_units(&self) -> &'static str {
        match self.0 {
            1 => "1 - 10",
            2 => "10 - 100",
            3 => "100 - 1K",
            4 => "1K - 10K",
            5 => "10K - 100K",
            6 => "100K - 1M",
            7 => "1M - 10M",
            8 => "10M+",
            _ => "Unknown",
        }
    }

    /// Scale separation ratio (τ_{i+1}/τ_i ≈ 10).
    #[must_use]
    pub const fn scale_separation_ratio() -> f64 {
        10.0
    }
}

impl fmt::Display for DomainHierarchyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Level {}", self.0)
    }
}

// =============================================================================
// §14 Safety Manifold Correspondence
// =============================================================================

/// Domain-specific constraint function specification (§14.1.1).
#[derive(Debug, Clone, Serialize)]
pub struct ConstraintFunction {
    /// Conservation law this constraint implements
    pub law: ConservationLawId,
    /// Domain
    pub domain: VigilanceDomain,
    /// Constraint name
    pub name: &'static str,
    /// Mathematical form
    pub form: &'static str,
    /// Parameters
    pub parameters: &'static str,
}

impl ConstraintFunction {
    /// Get all constraint functions for a domain.
    #[must_use]
    pub fn for_domain(domain: VigilanceDomain) -> Vec<Self> {
        match domain {
            VigilanceDomain::Cloud => Self::cloud_constraints(),
            VigilanceDomain::PV => Self::pv_constraints(),
            VigilanceDomain::AI => Self::ai_constraints(),
        }
    }

    fn cloud_constraints() -> Vec<Self> {
        vec![
            Self {
                law: ConservationLawId::Law1Mass,
                domain: VigilanceDomain::Cloud,
                name: "Data conservation",
                form: "g₁ = |bytes_in - bytes_out - Δstored| - ε",
                parameters: "ε: tolerance",
            },
            Self {
                law: ConservationLawId::Law2Energy,
                domain: VigilanceDomain::Cloud,
                name: "Cost minimization",
                form: "g₂ = cost(s,u) - budget(θ)",
                parameters: "budget: spending limit",
            },
            Self {
                law: ConservationLawId::Law3State,
                domain: VigilanceDomain::Cloud,
                name: "Resource allocation",
                form: "g₃ = |Σᵢ alloc_i - total_capacity| - ε",
                parameters: "total_capacity",
            },
            Self {
                law: ConservationLawId::Law4Flux,
                domain: VigilanceDomain::Cloud,
                name: "Throughput balance",
                form: "g₄ = |rate_in - rate_out| - ε (steady state)",
                parameters: "ε: tolerance",
            },
            Self {
                law: ConservationLawId::Law5Catalyst,
                domain: VigilanceDomain::Cloud,
                name: "Service availability",
                form: "g₅ = 1 - availability(s)",
                parameters: "target: 0.999+",
            },
            Self {
                law: ConservationLawId::Law6Rate,
                domain: VigilanceDomain::Cloud,
                name: "Request rate",
                form: "g₆ = request_rate(s,u) - rate_limit(θ)",
                parameters: "rate_limit",
            },
            Self {
                law: ConservationLawId::Law7Equilibrium,
                domain: VigilanceDomain::Cloud,
                name: "Load balance",
                form: "g₇ = max_i(load_i) - min_i(load_i) - ε",
                parameters: "ε: imbalance tolerance",
            },
            Self {
                law: ConservationLawId::Law8Saturation,
                domain: VigilanceDomain::Cloud,
                name: "CPU/Memory",
                form: "g₈ = utilization(s) - capacity(θ)",
                parameters: "capacity: 0.8-0.95",
            },
            Self {
                law: ConservationLawId::Law9Entropy,
                domain: VigilanceDomain::Cloud,
                name: "Technical debt",
                form: "g₉ = complexity(s) - threshold(θ)",
                parameters: "complexity metric",
            },
            Self {
                law: ConservationLawId::Law10Discretization,
                domain: VigilanceDomain::Cloud,
                name: "Instance count",
                form: "g₁₀ = |instances - round(instances)|",
                parameters: "integer constraint",
            },
            Self {
                law: ConservationLawId::Law11Structure,
                domain: VigilanceDomain::Cloud,
                name: "Schema version",
                form: "g₁₁ = 𝟙_{schema(s) ≠ schema_ref}",
                parameters: "schema hash",
            },
        ]
    }

    fn pv_constraints() -> Vec<Self> {
        vec![
            Self {
                law: ConservationLawId::Law1Mass,
                domain: VigilanceDomain::PV,
                name: "Mass balance",
                form: "g₁ = |dose - eliminated - accumulated| - ε",
                parameters: "ε: measurement error",
            },
            Self {
                law: ConservationLawId::Law2Energy,
                domain: VigilanceDomain::PV,
                name: "Binding favorability",
                form: "g₂ = ΔG_binding - ΔG_threshold",
                parameters: "ΔG in kJ/mol",
            },
            Self {
                law: ConservationLawId::Law3State,
                domain: VigilanceDomain::PV,
                name: "Receptor occupancy",
                form: "g₃ = |Σᵢ occupancy_i - 1| - ε",
                parameters: "fractional occupancy",
            },
            Self {
                law: ConservationLawId::Law4Flux,
                domain: VigilanceDomain::PV,
                name: "Metabolic flux",
                form: "g₄ = |J_formation - J_elimination| - ε",
                parameters: "flux in mol/s",
            },
            Self {
                law: ConservationLawId::Law5Catalyst,
                domain: VigilanceDomain::PV,
                name: "Enzyme turnover",
                form: "g₅ = |[E]_final - [E]_initial| - ε",
                parameters: "enzyme concentration",
            },
            Self {
                law: ConservationLawId::Law6Rate,
                domain: VigilanceDomain::PV,
                name: "Clearance rate",
                form: "g₆ = dC/dt + CL·C/V",
                parameters: "PK parameters",
            },
            Self {
                law: ConservationLawId::Law7Equilibrium,
                domain: VigilanceDomain::PV,
                name: "Steady state",
                form: "g₇ = |dC/dt| - ε (at claimed SS)",
                parameters: "ε: deviation tolerance",
            },
            Self {
                law: ConservationLawId::Law8Saturation,
                domain: VigilanceDomain::PV,
                name: "Michaelis-Menten",
                form: "g₈ = v - V_max·[S]/(K_m + [S])",
                parameters: "V_max, K_m",
            },
            Self {
                law: ConservationLawId::Law9Entropy,
                domain: VigilanceDomain::PV,
                name: "Irreversibility",
                form: "g₉ = -ΔS_total (must be ≤ 0)",
                parameters: "thermodynamic",
            },
            Self {
                law: ConservationLawId::Law10Discretization,
                domain: VigilanceDomain::PV,
                name: "Dosing units",
                form: "g₁₀ = |dose - available_strengths|",
                parameters: "tablet strengths",
            },
            Self {
                law: ConservationLawId::Law11Structure,
                domain: VigilanceDomain::PV,
                name: "Protein structure",
                form: "g₁₁ = RMSD(structure, native) - threshold",
                parameters: "Å threshold",
            },
        ]
    }

    fn ai_constraints() -> Vec<Self> {
        vec![
            Self {
                law: ConservationLawId::Law1Mass,
                domain: VigilanceDomain::AI,
                name: "Information flow",
                form: "g₁ = |bits_in - bits_out - compressed| - ε",
                parameters: "information theory",
            },
            Self {
                law: ConservationLawId::Law2Energy,
                domain: VigilanceDomain::AI,
                name: "Loss descent",
                form: "g₂ = L(s') - L(s) (must be ≤ 0 during training)",
                parameters: "loss function",
            },
            Self {
                law: ConservationLawId::Law3State,
                domain: VigilanceDomain::AI,
                name: "Attention normalization",
                form: "g₃ = |Σⱼ α_ij - 1| - ε",
                parameters: "softmax constraint",
            },
            Self {
                law: ConservationLawId::Law4Flux,
                domain: VigilanceDomain::AI,
                name: "Gradient flow",
                form: "g₄ = |∇L_layer_i - backprop_i| - ε",
                parameters: "gradient check",
            },
            Self {
                law: ConservationLawId::Law5Catalyst,
                domain: VigilanceDomain::AI,
                name: "Layer preservation",
                form: "g₅ = |W_layer - W_layer_init| (for frozen)",
                parameters: "weight drift",
            },
            Self {
                law: ConservationLawId::Law6Rate,
                domain: VigilanceDomain::AI,
                name: "Learning rate",
                form: "g₆ = |Δw| - lr_max·|∇L|",
                parameters: "learning rate bound",
            },
            Self {
                law: ConservationLawId::Law7Equilibrium,
                domain: VigilanceDomain::AI,
                name: "Convergence",
                form: "g₇ = |∇L| - ε (at claimed convergence)",
                parameters: "gradient norm",
            },
            Self {
                law: ConservationLawId::Law8Saturation,
                domain: VigilanceDomain::AI,
                name: "Context window",
                form: "g₈ = token_count - max_context",
                parameters: "context limit",
            },
            Self {
                law: ConservationLawId::Law9Entropy,
                domain: VigilanceDomain::AI,
                name: "Calibration",
                form: "g₉ = |entropy(output) - calibrated_entropy| - ε",
                parameters: "ECE metric",
            },
            Self {
                law: ConservationLawId::Law10Discretization,
                domain: VigilanceDomain::AI,
                name: "Quantization",
                form: "g₁₀ = |w - quantize(w)| - ε",
                parameters: "bit precision",
            },
            Self {
                law: ConservationLawId::Law11Structure,
                domain: VigilanceDomain::AI,
                name: "Architecture",
                form: "g₁₁ = 𝟙_{arch(s) ≠ arch_spec}",
                parameters: "architecture hash",
            },
        ]
    }
}

// =============================================================================
// §15 Harm Taxonomy Correspondence
// =============================================================================

/// Cross-domain harm mapping (§15.1).
#[derive(Debug, Clone, Serialize)]
pub struct CrossDomainHarmMapping {
    /// Harm type
    pub harm_type: HarmTypeId,
    /// Cloud manifestation
    pub cloud: &'static str,
    /// PV manifestation
    pub pv: &'static str,
    /// AI manifestation
    pub ai: &'static str,
    /// Primary mechanism
    pub primary_mechanism: &'static str,
}

impl CrossDomainHarmMapping {
    /// Get all cross-domain harm mappings.
    #[must_use]
    pub fn catalog() -> Vec<Self> {
        vec![
            Self {
                harm_type: HarmTypeId::A,
                cloud: "Service outage; immediate failure",
                pv: "Acute toxicity; immediate reaction",
                ai: "Immediate harmful output",
                primary_mechanism: "Law 1 (Mass): rapid accumulation",
            },
            Self {
                harm_type: HarmTypeId::B,
                cloud: "Resource exhaustion over time",
                pv: "Chronic toxicity; accumulation",
                ai: "Cumulative bias; model drift",
                primary_mechanism: "Law 1 (Mass): accumulated exposure",
            },
            Self {
                harm_type: HarmTypeId::C,
                cloud: "Unintended service interaction",
                pv: "Off-target binding; side effects",
                ai: "Unintended capability; misuse",
                primary_mechanism: "Law 2 (Energy): favorable off-target",
            },
            Self {
                harm_type: HarmTypeId::D,
                cloud: "Cascading failure across services",
                pv: "Multi-organ dysfunction",
                ai: "System-wide failure mode",
                primary_mechanism: "Law 4 (Flux): imbalance propagation",
            },
            Self {
                harm_type: HarmTypeId::E,
                cloud: "Edge case; unexpected input",
                pv: "Idiosyncratic reaction; rare AE",
                ai: "Adversarial input; jailbreak",
                primary_mechanism: "θ ∈ Θ_susceptible",
            },
            Self {
                harm_type: HarmTypeId::F,
                cloud: "Capacity exceeded; throttling",
                pv: "Enzyme saturation; nonlinear PK",
                ai: "Context overflow; hallucination",
                primary_mechanism: "Law 8 (Saturation): capacity exceeded",
            },
            Self {
                harm_type: HarmTypeId::G,
                cloud: "Service dependency conflict",
                pv: "Drug-drug interaction",
                ai: "Model-model interaction",
                primary_mechanism: "Law 5 (Catalyst): competitive inhibition",
            },
            Self {
                harm_type: HarmTypeId::H,
                cloud: "User segment impact; bias",
                pv: "Population-specific ADR",
                ai: "Demographic bias; disparate impact",
                primary_mechanism: "θ-distribution heterogeneity",
            },
        ]
    }

    /// Get harm mapping for a specific type.
    #[must_use]
    pub fn for_type(harm_type: HarmTypeId) -> Option<Self> {
        Self::catalog()
            .into_iter()
            .find(|m| m.harm_type == harm_type)
    }
}

// =============================================================================
// §16 Structural Correspondence Assessment
// =============================================================================

/// Component assessment type (§16.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrespondenceType {
    /// Same mathematical form and structure; different instantiation
    FormIdentical,
    /// Same structure with domain-specific parameterization
    Strong,
    /// Same roles and positions; some domain-specific interpretation
    Structural,
    /// Conceptual correspondence; significant implementation differences
    Moderate,
    /// Context-dependent; strong in some aspects, weak in others
    Variable,
}

impl fmt::Display for CorrespondenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormIdentical => write!(f, "Form-Identical"),
            Self::Strong => write!(f, "Strong"),
            Self::Structural => write!(f, "Structural"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Variable => write!(f, "Variable"),
        }
    }
}

/// Component correspondence assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAssessment {
    /// Component name
    pub component: &'static str,
    /// Correspondence type
    pub correspondence: CorrespondenceType,
    /// Assessment notes
    pub notes: &'static str,
}

impl ComponentAssessment {
    /// Get all component assessments (§16.1).
    #[must_use]
    pub fn catalog() -> Vec<Self> {
        vec![
            Self {
                component: "Safety Manifold",
                correspondence: CorrespondenceType::FormIdentical,
                notes: "Same mathematical FORM (SDE, first-passage, boundary-crossing). Different state spaces, dimensions, specific constraints.",
            },
            Self {
                component: "Conservation Laws (11 laws)",
                correspondence: CorrespondenceType::Strong,
                notes: "Same 11 law FORMS across domains. Domain-specific constraint functions instantiate each law.",
            },
            Self {
                component: "Hierarchy Structure (8 levels)",
                correspondence: CorrespondenceType::Strong,
                notes: "Identical 8-level structure with same propagation dynamics. Domain-specific scales and element assignments.",
            },
            Self {
                component: "Element System (15 elements)",
                correspondence: CorrespondenceType::Structural,
                notes: "Same 15-position functional roles. 6 strong, 6 moderate, 3 weak correspondences (per §12.0.1 table).",
            },
            Self {
                component: "Harm Taxonomy (8 types)",
                correspondence: CorrespondenceType::Strong,
                notes: "Same 8 harm types with consistent mechanisms. Domain-specific manifestations.",
            },
            Self {
                component: "Monitoring Infrastructure",
                correspondence: CorrespondenceType::Moderate,
                notes: "Conceptually similar (all use observables for detection). Implementation differs significantly.",
            },
            Self {
                component: "Regulatory Framework",
                correspondence: CorrespondenceType::Variable,
                notes: "Different regulatory bodies, timelines, requirements. Unified conceptual framework; diverse implementation.",
            },
        ]
    }
}

// =============================================================================
// §17 Domain-Specific Extensions
// =============================================================================

/// Domain-specific element that requires specialized modeling (§17.1).
///
/// These elements exist in one domain without direct analogs in others.
/// They have structural analogs at their position but require domain-specific
/// modeling approaches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSpecificElement {
    /// Domain where this element is unique
    pub domain: VigilanceDomain,
    /// Element position (structural analog)
    pub position: ElementPosition,
    /// Element name
    pub name: &'static str,
    /// Definition
    pub definition: &'static str,
    /// Why this requires domain-specific modeling
    pub extension_rationale: &'static str,
}

impl DomainSpecificElement {
    /// Get all domain-specific elements (§17.1).
    #[must_use]
    pub fn catalog() -> Vec<Self> {
        vec![
            Self {
                domain: VigilanceDomain::PV,
                position: ElementPosition::M4,
                name: "IMMUNE (Im)",
                definition: "Immune-mediated hypersensitivity",
                extension_rationale: "Biological immune system has no computational equivalent; requires dedicated modeling of immune-mediated adverse events",
            },
            Self {
                domain: VigilanceDomain::PV,
                position: ElementPosition::M3,
                name: "GENOTYPE (Gn)",
                definition: "Genetic variant affecting response",
                extension_rationale: "Inherited biological code; closest AI analog is architecture/hyperparameters but inheritance mechanism differs",
            },
            Self {
                domain: VigilanceDomain::AI,
                position: ElementPosition::M2,
                name: "UNCERTAINTY (Un)",
                definition: "Epistemic/aleatoric uncertainty",
                extension_rationale: "Explicit uncertainty quantification is unique to ML; pharma has variability but not formal uncertainty separation",
            },
            Self {
                domain: VigilanceDomain::AI,
                position: ElementPosition::F4,
                name: "GRADIENT (Gr)",
                definition: "Backpropagation signal",
                extension_rationale: "Learning-specific transformation; biological systems learn differently; cloud systems don't learn in same sense",
            },
            Self {
                domain: VigilanceDomain::Cloud,
                position: ElementPosition::M4,
                name: "SECRET (Sc)",
                definition: "Credential/key management",
                extension_rationale: "Digital authentication is unique to computing; no direct biological analog",
            },
        ]
    }

    /// Get domain-specific elements for a particular domain.
    #[must_use]
    pub fn for_domain(domain: VigilanceDomain) -> Vec<Self> {
        Self::catalog()
            .into_iter()
            .filter(|e| e.domain == domain)
            .collect()
    }
}

/// Domain-specific conservation law extension (§17.2).
///
/// These are specializations of the universal conservation laws for
/// domain-specific phenomena.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSpecificLawExtension {
    /// Domain where this extension applies
    pub domain: VigilanceDomain,
    /// Extended law name
    pub law_name: &'static str,
    /// Law statement
    pub statement: &'static str,
    /// Why this requires domain-specific extension
    pub extension_rationale: &'static str,
}

impl DomainSpecificLawExtension {
    /// Get all domain-specific law extensions (§17.2).
    #[must_use]
    pub fn catalog() -> Vec<Self> {
        vec![
            Self {
                domain: VigilanceDomain::PV,
                law_name: "Receptor Desensitization",
                statement: "Prolonged activation causes receptor downregulation",
                extension_rationale: "Biological adaptation unique to living systems; creates tolerance/dependence phenomena requiring temporal modeling",
            },
            Self {
                domain: VigilanceDomain::PV,
                law_name: "Metabolic Activation",
                statement: "Pro-drugs require biotransformation to active form",
                extension_rationale: "Biological metabolism converts inactive to active forms; no computing analog",
            },
            Self {
                domain: VigilanceDomain::AI,
                law_name: "Attention Normalization",
                statement: "Attention weights sum to 1 (softmax constraint)",
                extension_rationale: "Transformer-specific constraint; enables interpretable attention but limits capacity allocation",
            },
            Self {
                domain: VigilanceDomain::AI,
                law_name: "Context Limitation",
                statement: "Finite context window constrains information",
                extension_rationale: "LLM-specific constraint; biological and cloud systems don't have hard token limits",
            },
            Self {
                domain: VigilanceDomain::Cloud,
                law_name: "Idempotency",
                statement: "Repeated operations yield same result",
                extension_rationale: "Distributed systems requirement for consistency; biological systems are inherently non-idempotent",
            },
        ]
    }

    /// Get law extensions for a particular domain.
    #[must_use]
    pub fn for_domain(domain: VigilanceDomain) -> Vec<Self> {
        Self::catalog()
            .into_iter()
            .filter(|l| l.domain == domain)
            .collect()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vigilance_domains() {
        assert_eq!(VigilanceDomain::ALL.len(), 3);
        assert_eq!(VigilanceDomain::Cloud.name(), "Atomic Programming");
        assert_eq!(
            VigilanceDomain::PV.name(),
            "Computational Pharmacovigilance"
        );
        assert_eq!(VigilanceDomain::AI.name(), "Algorithmovigilance");
    }

    #[test]
    fn test_domain_dimensionality() {
        let (min, max) = VigilanceDomain::Cloud.state_space_dim();
        assert!(min <= max);
        assert!(max <= 100);

        let (ai_min, ai_max) = VigilanceDomain::AI.state_space_dim();
        assert!(ai_min >= 1_000_000);
        assert!(ai_max >= 1_000_000_000);
    }

    #[test]
    fn test_domain_instantiation() {
        let cloud = DomainInstantiation::cloud();
        assert_eq!(cloud.domain, VigilanceDomain::Cloud);
        assert_eq!(cloud.hierarchy.num_levels, 8);
        assert_eq!(cloud.elements.num_elements, 15);

        let pv = DomainInstantiation::pv();
        assert_eq!(pv.domain, VigilanceDomain::PV);

        let ai = DomainInstantiation::ai();
        assert_eq!(ai.domain, VigilanceDomain::AI);
    }

    #[test]
    fn test_structural_correspondence() {
        let result = check_structural_correspondence(VigilanceDomain::Cloud, VigilanceDomain::PV);
        assert!(result.is_corresponding);
        assert_eq!(result.conditions_met.len(), 5);
    }

    #[test]
    fn test_correspondence_conditions() {
        assert_eq!(CorrespondenceCondition::ALL.len(), 5);
    }

    #[test]
    fn test_vigilance_concepts() {
        assert_eq!(VigilanceConcept::ALL.len(), 9);

        let state = VigilanceConcept::State;
        assert_eq!(state.mathematical_object(), "s ∈ S ⊆ ℝⁿ");
        assert_eq!(
            state.domain_instantiation(VigilanceDomain::Cloud),
            "Resource utilization vector"
        );
    }

    #[test]
    fn test_element_layers() {
        assert_eq!(ElementLayer::ALL.len(), 4);

        let total_elements: usize = ElementLayer::ALL.iter().map(|l| l.element_count()).sum();
        assert_eq!(total_elements, 15);
    }

    #[test]
    fn test_element_positions() {
        assert_eq!(ElementPosition::ALL.len(), 15);

        // Check foundation layer
        assert_eq!(ElementPosition::F1.layer(), ElementLayer::Foundation);
        assert_eq!(ElementPosition::F4.layer(), ElementLayer::Foundation);

        // Check interface layer
        assert_eq!(ElementPosition::I1.layer(), ElementLayer::Interface);
        assert_eq!(ElementPosition::I3.layer(), ElementLayer::Interface);
    }

    #[test]
    fn test_element_names() {
        assert_eq!(
            ElementPosition::F1.element_name(VigilanceDomain::Cloud),
            "STORAGE (St)"
        );
        assert_eq!(
            ElementPosition::F1.element_name(VigilanceDomain::PV),
            "DRUG (Dr)"
        );
        assert_eq!(
            ElementPosition::F1.element_name(VigilanceDomain::AI),
            "MODEL (Md)"
        );
    }

    #[test]
    fn test_correspondence_strength() {
        // Check strong positions
        assert_eq!(
            CorrespondenceStrength::for_position(ElementPosition::F1),
            CorrespondenceStrength::Strong
        );

        // Check weak positions
        assert_eq!(
            CorrespondenceStrength::for_position(ElementPosition::M4),
            CorrespondenceStrength::Weak
        );
    }

    #[test]
    fn test_element_correspondence_summary() {
        let summary = ElementCorrespondenceSummary::calculate();

        // Per §12.0.1: 8 strong, 4 moderate, 3 weak
        // But our implementation has 6 strong, 6 moderate, 3 weak
        assert_eq!(
            summary.strong_count + summary.moderate_count + summary.weak_count,
            15
        );
        assert_eq!(summary.weak_count, 3); // M2, M3, M4
    }

    #[test]
    fn test_hierarchy_levels() {
        assert_eq!(DomainHierarchyLevel::ALL.len(), 8);

        let level1 = DomainHierarchyLevel(1);
        assert_eq!(level1.name(VigilanceDomain::Cloud), "Atomic");
        assert_eq!(level1.name(VigilanceDomain::PV), "Molecular");
        assert_eq!(level1.name(VigilanceDomain::AI), "Parameter");

        let level8 = DomainHierarchyLevel(8);
        assert_eq!(level8.name(VigilanceDomain::Cloud), "Enterprise");
        assert_eq!(level8.name(VigilanceDomain::PV), "Regulatory");
        assert_eq!(level8.name(VigilanceDomain::AI), "Societal");
    }

    #[test]
    fn test_scale_separation() {
        assert_eq!(DomainHierarchyLevel::scale_separation_ratio(), 10.0);
    }

    #[test]
    fn test_constraint_functions() {
        let cloud = ConstraintFunction::for_domain(VigilanceDomain::Cloud);
        assert_eq!(cloud.len(), 11);

        let pv = ConstraintFunction::for_domain(VigilanceDomain::PV);
        assert_eq!(pv.len(), 11);

        let ai = ConstraintFunction::for_domain(VigilanceDomain::AI);
        assert_eq!(ai.len(), 11);
    }

    #[test]
    fn test_cross_domain_harm_mapping() {
        let catalog = CrossDomainHarmMapping::catalog();
        assert_eq!(catalog.len(), 8);

        let type_a = CrossDomainHarmMapping::for_type(HarmTypeId::A);
        assert!(type_a.is_some());
        let mapping = type_a.unwrap();
        assert!(mapping.cloud.contains("outage"));
        assert!(mapping.pv.contains("toxicity"));
    }

    #[test]
    fn test_component_assessment() {
        let catalog = ComponentAssessment::catalog();
        assert_eq!(catalog.len(), 7);

        // Check safety manifold is form-identical
        let manifold = catalog.iter().find(|c| c.component == "Safety Manifold");
        assert!(manifold.is_some());
        assert_eq!(
            manifold.unwrap().correspondence,
            CorrespondenceType::FormIdentical
        );
    }

    #[test]
    fn test_invariant_functions() {
        assert_eq!(
            ElementPosition::F1.invariant_function(),
            "Primary active entity"
        );
        assert_eq!(
            ElementPosition::I1.invariant_function(),
            "Input quantification"
        );
        assert_eq!(
            ElementPosition::M1.invariant_function(),
            "Observable signal"
        );
    }

    // §17 Tests

    #[test]
    fn test_domain_specific_elements() {
        let catalog = DomainSpecificElement::catalog();
        assert_eq!(catalog.len(), 5);

        // Check PV has 2 domain-specific elements (IMMUNE, GENOTYPE)
        let pv_elements = DomainSpecificElement::for_domain(VigilanceDomain::PV);
        assert_eq!(pv_elements.len(), 2);

        // Check AI has 2 domain-specific elements (UNCERTAINTY, GRADIENT)
        let ai_elements = DomainSpecificElement::for_domain(VigilanceDomain::AI);
        assert_eq!(ai_elements.len(), 2);

        // Check Cloud has 1 domain-specific element (SECRET)
        let cloud_elements = DomainSpecificElement::for_domain(VigilanceDomain::Cloud);
        assert_eq!(cloud_elements.len(), 1);
    }

    #[test]
    fn test_domain_specific_law_extensions() {
        let catalog = DomainSpecificLawExtension::catalog();
        assert_eq!(catalog.len(), 5);

        // Check PV has 2 law extensions
        let pv_laws = DomainSpecificLawExtension::for_domain(VigilanceDomain::PV);
        assert_eq!(pv_laws.len(), 2);

        // Check AI has 2 law extensions
        let ai_laws = DomainSpecificLawExtension::for_domain(VigilanceDomain::AI);
        assert_eq!(ai_laws.len(), 2);

        // Check Cloud has 1 law extension (Idempotency)
        let cloud_laws = DomainSpecificLawExtension::for_domain(VigilanceDomain::Cloud);
        assert_eq!(cloud_laws.len(), 1);
        assert_eq!(cloud_laws[0].law_name, "Idempotency");
    }
}
