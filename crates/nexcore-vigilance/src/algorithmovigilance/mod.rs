//! # Algorithmovigilance (ToV Part VIII: §51-§60)
//!
//! The science and activities relating to detection, assessment, understanding,
//! and prevention of adverse effects arising from AI/ML-enabled systems throughout
//! the system lifecycle.
//!
//! # ToV Integration
//!
//! This module implements Part VIII of the Theory of Vigilance, applying
//! pharmacovigilance methodology to clinical AI oversight.
//!
//! | Section | Module | Content |
//! |---------|--------|---------|
//! | §51 | `axioms` | Alignment Principle |
//! | §52 | `axioms` | Four ACA Axioms, O→C→A→H Causal Chain |
//! | §53 | `scoring` | Algorithm Causality Assessment (8 Lemmas) |
//! | §54 | `axioms` | Four-Case Logic Engine |
//! | §55 | `iair` | Individual Algorithm Incident Report Schema |
//!
//! # Safety Axioms
//!
//! - **Pharmakon Principle**: Capability and misalignment are inseparable
//! - **Override Paradox**: Correct algorithm overridden cannot cause harm from override
//! - **Causal Chain**: O → C → A → H must be connected for attribution
//!
//! # ACA Scoring (§53)
//!
//! | Category | Score | Description |
//! |----------|-------|-------------|
//! | Definite | ≥6 | Beyond reasonable doubt |
//! | Probable | 4-5 | Likely contributed |
//! | Possible | 2-3 | May have contributed |
//! | Unlikely | <2 | Insufficient evidence |
//! | Unassessable | — | Required lemmas missing |
//! | Exculpated | — | Override Paradox applies |
//!
//! # IAIR Schema (§55)
//!
//! Eight-block incident report structure:
//! - **Block A**: Metadata (sender, timestamps)
//! - **Block B**: Algorithm Identification (name, version, FDA status)
//! - **Block C**: Patient Characteristics (demographics, conditions)
//! - **Block D**: Incident Description (timeline, actions)
//! - **Block E**: Outcome Information (severity, category)
//! - **Block F**: Causality Assessment (ACA integration)
//! - **Block G**: Signal Indicators (drift, similar cases)
//! - **Block H**: Administrative (regulatory, corrective actions)

pub mod axioms;
pub mod iair;
pub mod scoring;
pub mod signals;

// Re-export core types from axioms (§51-§52, §54)
pub use axioms::{
    ACA_SIGMOID_MU, ACA_SIGMOID_SIGMA, AcaAxiom, AlgorithmOutput, AxiomSatisfaction, CausalChain,
    CausalChainLink, ChainLinkEvidence, ClinicianAction, ClinicianCognition, EvidenceStrength,
    HarmOutcome, HarmSeverity, LogicCase, OverrideParadox,
};

// Re-export scoring types (§53)
pub use scoring::{
    AcaCausalityCategory, AcaLemma, AcaScore, AcaScoringInput, AcaScoringResult,
    GroundTruthStandard, LemmaResponse, LemmaSatisfaction, score_aca, score_aca_quick, sigmoid,
};

// Re-export IAIR types (§55)
pub use iair::{
    AlgorithmCategory,
    // Block structs
    BlockA,
    BlockB,
    BlockC,
    BlockD,
    BlockE,
    BlockF,
    BlockG,
    BlockH,
    CaseStatus,
    ClinicalDomain,
    DeploymentContext,
    // Block enums (T2-P)
    FdaClearanceStatus,
    // Complete report
    IairReport,
    IncidentCategory,
    OutcomeSeverity,
};

// Re-export signal types (§56)
pub use signals::{
    // Complete signal (T3)
    AiSignal,
    AiSignalAggregate,
    AiSignalSeverity,
    // Signal types (T2-P)
    AiSignalType,
    // Drift metrics (T2-C)
    CusumResult,
    DriftIndicator,
    KlDivergenceResult,
    SubgroupDimension,
    SubgroupDisparityResult,
    UContribution,
};
