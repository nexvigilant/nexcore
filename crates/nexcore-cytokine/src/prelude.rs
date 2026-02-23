//! # Cytokine Prelude
//!
//! Convenience re-exports for the most common cytokine signaling types.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_cytokine::prelude::*;
//!
//! let signal = Cytokine::new(CytokineFamily::Il1, "threat_detected");
//! let mut bus = CytokineBus::new();
//! bus.emit(signal);
//! ```

pub use crate::{
    ApoptosisController,
    CascadeRule,
    ChemotacticAgent,
    // Core signaling
    Cytokine,
    CytokineBus,
    CytokineFamily,
    DifferentiableCell,
    Emitter,
    EndocyticReceptor,
    ExocyticEmitter,
    // Biology primitives
    Gradient,
    GradientField,
    InternalizationPolicy,
    MembraneGate,
    Phagocyte,
    PopulationController,
    PopulationHealth,
    PostMortem,
    Potency,
    QuorumResult,
    QuorumSensor,
    Receptor,
    Scope,
    ShutdownPhase,
    SignalBundle,
    SpawnResult,
    Specialization,
    ThreatClass,
    ThreatDigest,
    ThreatLevel,
    Tropism,
    Vesicle,
    VesiclePool,
};
