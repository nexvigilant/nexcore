//! # NexVigilant Core — Vigilance Kernel
//!
//! Theory of Vigilance (ToV) axioms, Guardian-AV risk detection, and consolidated domain modules.
//!
//! This is the core vigilance crate of the NexVigilant platform — the single source of truth
//! for all consolidated pharmacovigilance modules.
//!
//! Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.

#![forbid(unsafe_code)]
#![allow(missing_docs)] // Internal library - docs enforced on public API crates only
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

// Autonomous Vigilance Company (T3: κ+σ+∂+ρ+ς+μ+π+→+N)
pub mod avc;

// Pharmacovigilance Operating System (T3: μ+σ+κ+∂+ρ+ς+π+→)
pub mod pvos;

// pub mod academy;

// Betting signal detection (consolidated from NexBet)
// Cross-domain transfer: PRR→BDI, EBGM→ECS
// pub mod betting;

// AI clients (consolidated from nexcore-ai)
pub mod ai;

// Algorithmovigilance (ToV Part VIII: §51-§60)
// AI safety axioms, ACA framework, Four-Case Logic Engine
pub mod algorithmovigilance;

// Bioinformatics domain (consolidated from nexcore-bioinfo)
pub mod bioinfo;

// CABA Competency-Based Assessment (consolidated from nexcore-caba)
pub mod caba;

// Capability assessment via chemistry equations (Arrhenius, Michaelis-Menten, Hill, etc.)
pub mod capabilities;

// Cloud provider integrations (consolidated from nexcore-cloud)
pub mod cloud;

// Learning ecosystem (consolidated from nexcore-learning)
pub mod learning;

// Google Workspace integrations (consolidated from nexcore-google)
pub mod google;

// Quiz platform (consolidated from nexcore-quiz)
// pub mod quiz;

// Guardian compliance domain (consolidated from nexcore-guardian)
pub mod guardian_domain;

// Command Center operations (consolidated from nexcore-command-center)
pub mod command_center;

// Vocabulary intelligence (consolidated from nexcore-vocab)
pub mod vocab;

// MedWatch form types (consolidated from nexcore-medwatch)
pub mod medwatch;
pub mod network_nodes;
pub mod transmission;

// CTVP framework (consolidated from nexcore-ctvp)
pub mod ctvp;

// VDAG framework (Validated SMART-DAG)
pub mod vdag;

// Validation engine (consolidated from nexcore-validation)
pub mod validation;

// UI Mapper (consolidated from nexcore-ui-mapper)
pub mod ui_mapper;

// Crypto operations (consolidated from nexcore-crypto)
// pub mod crypto;

// Documentation generation (FORGE-generated)
pub mod docs;

// Medical coding (consolidated from nexcore-coding)
pub mod coding;

// Foundation algorithms (consolidated from nexcore-foundation)
pub mod foundation;

// Pharmacovigilance (consolidated from nexcore-pv)
pub mod pv;

// PVDSL scripting (consolidated from nexcore-pvdsl)
pub mod pvdsl;

// Skills registry (consolidated from nexcore-skills)
pub mod skills;

// Orchestration (consolidated from nexcore-orchestrator)
pub mod orchestrator;

// SPIS - Strategic Pipeline Innovation System (consolidated from nexcore-spis)
pub mod spis;

// Security scanning (consolidated from nexcore-security)
pub mod security;

pub mod anti_pattern;
pub mod axiom_summary;
pub mod axioms;
pub mod conservation;
pub mod conservation_catalog;
pub mod decomposition;
pub mod definitions;
pub mod domain_instantiation;
pub mod emergence;
pub mod epistemic;

// FDA AI Credibility Assessment Framework (January 2025)
pub mod fda;

pub mod guardian;
pub mod harm_taxonomy;
pub mod hierarchy;
pub mod hud;
pub mod manifold;
pub mod principal_theorems;
pub mod proof;
pub mod stark;
pub mod tov;
pub mod validated_bounds;

// Grounded Theory of Vigilance primitives
pub use nexcore_tov::grounded;

// Domain Discovery Framework (primitives, grammar, translation)
pub mod domain_discovery;

// Domain-to-Code Pipeline (pattern extraction → code generation)
pub mod domain_to_code;

// Universal Primitives (chemistry equations as computational patterns)
pub mod primitives;

// Lex Primitiva: The 15 irreducible T1 symbols (σ, μ, ς, ρ, ∅, ∂, f, ∃, π, →, κ, N, λ, ∝, Σ)
// Grounds all higher-tier types to universal primitives via GroundsTo trait
pub mod lex_primitiva;

// Cognitive Evolution Pipeline (8-stage knowledge discovery)
// Patent: NV-2026-001
pub mod cep;

// Control Loop Abstraction (T2-C cross-domain pattern)
// Aerospace → PV domain translation
pub mod control;

// Telemetry (LLM usage monitoring)
pub mod telemetry;

// §1 Preliminary Definitions
pub use definitions::{
    DetectionResult, DynamicsType, HarmEvent, KnownParameter, LoopStage, MonitoringApparatus,
    Observability, ParameterSpace, Perturbation, PerturbationClass, PerturbationType,
    UncertainParameter, VigilanceLoop, VigilanceSystem, constraint_to_harm, harm_to_constraint,
};

// §2 Axiom 1: System Decomposition
pub use decomposition::{
    AccessibleStateSpace, Axiom1Verification, CompositionFunction, Decomposition,
    DecompositionError, Element, ElementSet, InteractionGraph, InteractionMatrix, SumComposition,
};

// §3 Axiom 2: Hierarchical Organization
pub use hierarchy::{
    AveragingCoarseGrain, Axiom2Verification, BinaryEmergentProperty, CoarseGrainingMap,
    ContinuousEmergentProperty, EmergentProperty, HierarchicalState, Hierarchy, HierarchyError,
    Level, LevelStateSpace, PVLevel,
};

// §4 Axiom 3: Conservation Constraints
pub use conservation::{
    Axiom3Verification, CapacityConstraint, ConservationError, ConservationLaw,
    ConservationLawType, ConstraintSet, FeasibleRegionResult, LinearConstraint,
    MassBalanceConstraint, StateNormalizationConstraint,
};

// §2-§6 Axioms (legacy)
pub use axioms::{HierarchyLevel, SafetyManifold};

// Part VIII: Algorithmovigilance (§51-§60)
pub use algorithmovigilance::{
    ACA_SIGMOID_MU,
    ACA_SIGMOID_SIGMA,
    // §51-§52 Axioms & Causal Chain
    AcaAxiom,
    AcaCausalityCategory,
    // §53-§54 ACA Scoring
    AcaLemma,
    AcaScore,
    AcaScoringInput,
    AcaScoringResult,
    AiSignal,
    AiSignalAggregate,
    AiSignalSeverity,
    // §56 AI Signal Detection
    AiSignalType,
    AlgorithmCategory,
    AlgorithmOutput,
    AxiomSatisfaction,
    BlockA,
    BlockB,
    BlockC,
    BlockD,
    BlockE,
    BlockF,
    BlockG,
    BlockH,
    CaseStatus,
    CausalChain,
    CausalChainLink,
    ChainLinkEvidence,
    ClinicalDomain,
    ClinicianAction,
    ClinicianCognition,
    CusumResult,
    DeploymentContext,
    DriftIndicator,
    EvidenceStrength,
    // §55 IAIR Schema
    FdaClearanceStatus,
    GroundTruthStandard,
    HarmOutcome,
    HarmSeverity,
    IairReport,
    IncidentCategory,
    KlDivergenceResult,
    LemmaResponse,
    LemmaSatisfaction,
    LogicCase,
    OutcomeSeverity,
    OverrideParadox,
    SubgroupDimension,
    SubgroupDisparityResult,
    UContribution,
    score_aca,
    score_aca_quick,
    sigmoid,
};

// §5 Axiom 4: Safety Manifold (Geometric)
pub use manifold::{
    GeometricSafetyManifold, HarmBoundary, SignalPoint, SignalThresholds, SignedDistance,
};

// §5 Axiom 4: Safety Manifold (ToV Generic)
pub use manifold::{
    Axiom4SafetyManifold,
    Axiom4Verification,
    // §5 Definitions 5.6-5.7
    ConstraintCompatibility,
    FirstPassageTime,
    HarmBoundaryInfo,
    ManifoldPointType,
    ManifoldRegularityCase,
    RegularityConditionResult,
    SafeConfigurationOpenness,
    SafetyMarginResult,
    StratifiedStructure,
    UnsafeConfigurationResult,
};

// §6 Axiom 5: Emergence
pub use emergence::{
    Axiom5Verification, BufferingCapacity, BufferingMechanism, EmergenceFramework,
    ExponentialPropagation, HarmLevel, HarmProbabilityResult, LevelPerturbation,
    NonMarkovianHistory, PropagationFunction, PropagationParams, PropertyVerification,
    SigmoidalPropagation,
};

// §7 Axiom Summary (Dependencies, Completeness, Consistency)
pub use axiom_summary::{
    Axiom, Axiom7Verification, AxiomDependencyGraph, AxiomInfo, AxiomStatus, CompletenessResult,
    CompletenessVerifier, ConsistencyDomain, ConsistencyResult, ConsistencyVerifier,
    PrincipalTheorem, VerificationStatus, verify_axiom_system,
};

// §8 Conservation Law Catalog (11 Laws, Math Types, Tolerances)
pub use conservation_catalog::{
    ConservationLawDefinition, ConservationLawId, ConservationTypeCategory, ConstraintMathType,
    SaturationFunction, SaturationFunctionType, SaturationParams, StructuralSignature,
    StructuralSignatureType, ToleranceDomain, ToleranceFunction,
};

// §9 Harm Classification Taxonomy (Full Implementation)
pub use harm_taxonomy::{
    // §9.0 Taxonomic Foundations
    CrossDomainMapping,
    ExhaustivenessResult,
    // §9.3 Harm-Axiom Connections
    HarmAxiomConnection,
    HarmCharacteristics,
    // Harm Classification
    HarmClassification,
    HarmTypeCombination,
    // §9.1 Harm Type Enumeration
    HarmTypeDefinition,
    HarmTypeId,
    // §9.1.1 Manifestation Level
    ManifestationDerivation,
    ManifestationLevel,
    PerturbationMultiplicity,
    PrimaryAxiom,
    ResponseDeterminism,
    TemporalProfile,
    // Verification
    classify_harm_event,
    verify_exhaustiveness,
};

// §10 Principal Theorems (Predictability, Attenuation, Intervention, Conservation, Equivalence)
pub use principal_theorems::{
    // Theorem 10.2: Attenuation
    AttenuationInterpretation,
    AttenuationResult,
    AttenuationTheorem,
    AttenuationVersion,
    // §10.7 Complexity
    ComputationalComplexity,
    // Theorem 10.4: Conservation
    ConservationTheorem,
    ConservationTheoremPart,
    ConstraintDiagnosis,
    // Theorem 10.5: Manifold Equivalence
    EquivalenceResult,
    // Theorem 10.1: Predictability
    FirstPassageResult,
    // Theorem 10.3: Intervention
    InterventionParams,
    InterventionResult,
    InterventionTheorem,
    ManifoldEquivalenceTheorem,
    ManifoldGeometry,
    PredictabilityHypotheses,
    PredictabilityTheorem,
    PropagationModel,
    PropagationProperty,
    TheoremComplexity,
    // §10.6 Dependencies
    TheoremDependencies,
    TheoremId,
};

// §11-§15 Domain Instantiations (Cloud, PV, AI Correspondence)
pub use domain_instantiation::{
    ComponentAssessment,
    // §14 Constraint Functions
    ConstraintFunction,
    ConstraintSetSpec,
    // §11.0 Structural Correspondence
    CorrespondenceCondition,
    CorrespondenceStrength,
    // §16 Assessment
    CorrespondenceType,
    // §15 Harm Mapping
    CrossDomainHarmMapping,
    // §13 Hierarchy Levels
    DomainHierarchyLevel,
    DomainInstantiation,
    // §17 Domain-Specific Extensions
    DomainSpecificElement,
    DomainSpecificLawExtension,
    ElementCorrespondenceSummary,
    ElementDecompositionSpec,
    // §12 Element System
    ElementLayer,
    ElementPosition,
    HierarchySpec,
    ParameterSpaceSpec,
    PerturbationSpaceSpec,
    StateSpaceSpec,
    StructuralCorrespondenceResult,
    // §11.1 Foundational Concepts
    VigilanceConcept,
    // §11.0 Domain Framework
    VigilanceDomain,
    check_structural_correspondence,
};

// §9 Harm Types (legacy - use harm_taxonomy for full implementation)
pub use tov::{HarmType, SafetyMargin};

// Validated Bounds (Wolfram-verified)
pub use validated_bounds::{
    Axiom1Bounds, Axiom2ScaleSeparation, ComplexityClass, EmergenceProbability, FeasibleRegion,
    SafetyState, SignalStrength, SignalUniqueness, ValidatedSafetyMargin, ValidationSummary,
};

// Supporting modules
pub use stark::{AtomResult, AtomStatus, ExecutionContext, VerificationAudit};

// Formal Proof Infrastructure (Curry-Howard)
pub use proof::{
    // Logic types
    And,
    BoundedProbability,
    Exists,
    NonRecurrenceThreshold,
    Not,
    Or,
    Proof,
    Truth,
    ValidatedDomainIndex,
    ValidatedHarmTypeIndex,
    ValidatedLawIndex,
    // Type-level constraints
    ValidatedLevel,
    Void,
    // Attenuation theorem
    attenuation::{
        AttenuationAnalysis, PropagationProbability, analyze_attenuation, attenuation_rate,
        harm_probability, harm_probability_exponential, protective_depth,
    },
};

// AI Clients (consolidated from nexcore-ai)
pub use ai::{ClaudeClient, GeminiClient, GenerationOptions, ModelClient, ModelOrchestrator};

// SPIS (consolidated from nexcore-spis)
pub use spis::{
    CapabilityGap, CostTracker as SpisCostTracker, Pipeline, PipelineComponent, PipelineExecution,
    StrategicAnalysis, StrategicAnalyzer,
};

// Lex Primitiva: T1 symbolic foundation
pub use lex_primitiva::{
    GroundingTier, GroundsTo, LexPrimitiva, PrimitiveComposition,
    bridge::{from_primitive_tier, to_primitive_tier},
};

/// Returns the NexCore Vigilance version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// CTVP Phase 1: Chaos engineering tests
// Feature-gated to avoid running in normal test suite
#[cfg(all(test, feature = "chaos-tests"))]
mod tests;
