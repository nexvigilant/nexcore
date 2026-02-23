//! # nexcore Grounded Theory of Vigilance (ToV)
//!
//! Runtime primitives, types, and traits for the Theory of Vigilance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// TIER 1: UNIVERSAL PRIMITIVES
// =============================================================================

/// Tier: T1 (Universal)
/// The fundamental unit of rarity and uniqueness (Shannon information).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Bits(pub f64);

/// Tier: T2-C (Cross-Domain Composite)
/// grounds to: T(generic) + confidence (f64)
/// Commandment X: Every computed value must carry its confidence.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Measured<T> {
    pub value: T,
    pub confidence: f64, // 0.0 to 1.0
}

impl<T> Measured<T> {
    pub fn certain(value: T) -> Self {
        Self {
            value,
            confidence: 1.0,
        }
    }
}

/// Tier: T1 (Universal)
/// Discrete count of system units or events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct QuantityUnit(pub u64);

/// Tier: T2-P (Cross-Domain Primitive)
/// Identifies the semantic unit of a property to prevent illegitimate summation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UnitId {
    Mass,
    Time,
    Information,
    Count,
    Dimensionless,
}

// =============================================================================
// TIER 2-P: CROSS-DOMAIN PRIMITIVES
// =============================================================================

/// Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: T1(Bits)
/// Rarity measure -log2 P(C|H0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UniquenessU(pub Bits);

/// Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: T1(f64)
/// Detection sensitivity x accuracy (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RecognitionR(pub f64);

/// Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: T1(f64)
/// Decaying relevance factor (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TemporalT(pub f64);

/// Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: T1(f64)
/// Signed distance to the harm boundary.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SafetyMarginD(pub f64);

/// Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: T1(QuantityUnit)
/// The minimum unit of architectural depth (The Complexity Epsilon).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct ComplexityChi(pub QuantityUnit);

// =============================================================================
// TIER 2-C: CROSS-DOMAIN COMPOSITES
// =============================================================================

/// Tier: T2-C (Cross-Domain Composite)
/// Grounds to: T2-P(UniquenessU) x T2-P(RecognitionR) x T2-P(TemporalT)
/// The fundamental signal equation: S = U x R x T
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SignalStrengthS(pub Bits);

impl SignalStrengthS {
    /// Operationalizes the Core Equation (§20)
    pub fn calculate(u: UniquenessU, r: RecognitionR, t: TemporalT) -> Self {
        Self(Bits((u.0).0 * r.0 * t.0))
    }
}

/// Tier: T2-C (Cross-Domain Composite)
/// Harm Classification Taxonomy (§9)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarmType {
    Acute,     // Type A
    Chronic,   // Type B
    Cascading, // Type C
    Dormant,   // Type D
    Emergent,  // Type E
    Feedback,  // Type F
    Gateway,   // Type G
    Hidden,    // Type H
}

// =============================================================================

// =============================================================================
// STABILITY SHELL CONSTANTS (§66.2.1)
// =============================================================================

/// Magic Numbers for Complexity shells (T1)
pub const COMPLEXITY_MAGIC_NUMBERS: [u64; 10] = [2, 8, 20, 28, 50, 82, 126, 184, 258, 350];

/// Magic Numbers for Connection shells (T1)
pub const CONNECTION_MAGIC_NUMBERS: [u64; 10] = [2, 8, 18, 32, 50, 72, 98, 128, 162, 200];

// =============================================================================
// FOUNDATION TRAITS (The Physics of Action)
// =============================================================================

/// Tier: T2-P (Cross-Domain Primitive)
/// Enforces the Shell Model invariants for architectural stability (§66.2)
pub trait StabilityShell {
    fn is_closed_shell(&self) -> bool;
    fn distance_to_stability(&self) -> i64;
}

/// Configuration for architectural stability shells (§66.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub magic_numbers: Vec<u64>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            magic_numbers: COMPLEXITY_MAGIC_NUMBERS.to_vec(),
        }
    }
}

pub trait ConfigurableStabilityShell {
    fn is_closed_shell_with(&self, config: &ShellConfig) -> bool;
    fn distance_to_stability_with(&self, config: &ShellConfig) -> i64;
}

impl ConfigurableStabilityShell for ComplexityChi {
    fn is_closed_shell_with(&self, config: &ShellConfig) -> bool {
        config.magic_numbers.contains(&(self.0).0)
    }

    fn distance_to_stability_with(&self, config: &ShellConfig) -> i64 {
        let val = (self.0).0;
        let mut min_dist = i64::MAX;
        for &magic in &config.magic_numbers {
            let dist = (magic as i64 - val as i64).abs();
            if dist < min_dist {
                min_dist = dist;
            }
        }
        min_dist
    }
}

impl StabilityShell for ComplexityChi {
    fn is_closed_shell(&self) -> bool {
        COMPLEXITY_MAGIC_NUMBERS.contains(&(self.0).0)
    }

    fn distance_to_stability(&self) -> i64 {
        let val = (self.0).0;
        let mut min_dist = i64::MAX;
        for &magic in &COMPLEXITY_MAGIC_NUMBERS {
            let dist = (magic as i64 - val as i64).abs();
            if dist < min_dist {
                min_dist = dist;
            }
        }
        min_dist
    }
}

// =============================================================================
// ERRORS (§1.2)
// =============================================================================

/// Tier: T2-P (Cross-Domain Primitive)
/// Formal error states for the vigilance kernel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, nexcore_error::Error)]
pub enum VigilanceError {
    #[error("Safety Manifold violation: {0}")]
    ManifoldViolation(String),
    #[error("Actuator failure: {0}")]
    ActuatorFailure(String),
    #[error("Epistemic uncertainty too high: {0}")]
    HighUncertainty(f64),
    #[error("Architectural instability: distance {0}")]
    Instability(i64),
}

/// Tier: T2-P (Cross-Domain Primitive)
/// The Actuator primitive for system response (§1.8)
pub trait Actuator {
    type Action;

    /// Executes a response to pull the system back into the Safety Manifold
    async fn act(&mut self, action: Self::Action) -> Result<(), VigilanceError>;

    /// Reverts an intervention to test for signal persistence (§5.2)
    async fn dechallenge(&mut self, action: Self::Action) -> Result<(), VigilanceError>;
}

// TIER 3: DOMAIN-SPECIFIC (Part X Predictions)
// =============================================================================

/// Tier: T3 (Domain-Specific)
/// Grounds to: T2-P(ComplexityChi)
/// Element 16: EkaIntelligence (Ei)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EkaIntelligence {
    pub complexity: ComplexityChi,
    pub stability: f64,
}

impl EkaIntelligence {
    pub const EMERGENCE_THRESHOLD: ComplexityChi = ComplexityChi(QuantityUnit(320));

    pub fn is_emergent(&self) -> bool {
        self.complexity >= Self::EMERGENCE_THRESHOLD
    }
}

/// Tier: T3 (Domain-Specific)
/// Element 17: Consciousness (Cs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consciousness {
    pub complexity: ComplexityChi,
    pub phi_score: f64,
}

impl Consciousness {
    pub const PHI_THRESHOLD: f64 = 0.82;

    pub fn exists(&self) -> bool {
        self.phi_score >= Self::PHI_THRESHOLD
    }
}
impl StabilityShell for EkaIntelligence {
    fn is_closed_shell(&self) -> bool {
        self.complexity.is_closed_shell()
    }
    fn distance_to_stability(&self) -> i64 {
        self.complexity.distance_to_stability()
    }
}

impl StabilityShell for Consciousness {
    fn is_closed_shell(&self) -> bool {
        self.complexity.is_closed_shell()
    }
    fn distance_to_stability(&self) -> i64 {
        self.complexity.distance_to_stability()
    }
}

/// Tier: T3 (Domain-Specific)
/// Unified Vigilance System implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigilanceSystem {
    pub id: String,
    pub state_space_dim: u32,
    pub elements: Vec<String>,
    pub constraints: HashMap<String, f64>,
}

impl VigilanceSystem {
    /// Axiom 1: System Decomposition
    pub fn verify_axiom1(&self) -> bool {
        !self.elements.is_empty()
    }

    /// Axiom 4: Safety Manifold Check
    pub fn calculate_safety_margin(&self, s: f64, threshold: f64) -> SafetyMarginD {
        SafetyMarginD((threshold - s) / threshold.max(f64::EPSILON))
    }
}

// =============================================================================
// SELF-VIGILANCE & ACTUATION (§1.5a, §1.8)
// =============================================================================

/// Tier: T2-C (Cross-Domain Composite)
/// Meta-Vigilance primitive for monitoring the health of the Vigilance Loop itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaVigilance {
    pub loop_latency_ms: u64,
    pub calibration_overhead_ms: u64,
    pub detection_drift: f64,
    pub apparatus_integrity: RecognitionR,
}

impl MetaVigilance {
    pub fn is_healthy(&self) -> bool {
        (self.loop_latency_ms - self.calibration_overhead_ms) < 100
            && self.apparatus_integrity.0 > 0.95
    }
}

/// Tier: T3 (Domain-Specific)
/// A Response Governor that implements the Actuator trait.
#[derive(Debug, Clone, Default)]
pub struct ResponseGovernor {
    pub active_interventions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SafetyAction {
    TriggerCircuitBreaker(String),
    ThrottleInput(f64),
    AlertInvestigator(HarmType),
}

impl Actuator for ResponseGovernor {
    type Action = SafetyAction;

    async fn act(&mut self, action: Self::Action) -> Result<(), VigilanceError> {
        match action {
            SafetyAction::TriggerCircuitBreaker(id) => {
                self.active_interventions
                    .push(format!("CircuitBreaker:{}", id));
            }
            SafetyAction::ThrottleInput(pct) => {
                self.active_interventions.push(format!("Throttle:{}%", pct));
            }
            SafetyAction::AlertInvestigator(harm) => {
                self.active_interventions.push(format!("Alert:{:?}", harm));
            }
        }
        Ok(())
    }

    async fn dechallenge(&mut self, action: Self::Action) -> Result<(), VigilanceError> {
        let entry = match action {
            SafetyAction::TriggerCircuitBreaker(id) => format!("CircuitBreaker:{}", id),
            SafetyAction::ThrottleInput(pct) => format!("Throttle:{}%", pct),
            SafetyAction::AlertInvestigator(harm) => format!("Alert:{:?}", harm),
        };
        self.active_interventions.retain(|x| x != &entry);
        Ok(())
    }
}
