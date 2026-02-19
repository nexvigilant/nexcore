//! # §8 Conservation Law Catalog
//!
//! This section enumerates the eleven specific conservation laws that govern vigilance
//! systems. These laws are instantiations of the eight type categories defined in §4.5.
//!
//! ## §8.0 Constraint Type Taxonomy
//!
//! While colloquially called "conservation laws," the eleven laws comprise four distinct
//! mathematical structures requiring different analytical treatments:
//!
//! | Mathematical Type | Laws | Analysis Method |
//! |-------------------|------|-----------------|
//! | Strict Conservation | 1, 3, 4, 6 | Noether's theorem, Hamiltonian mechanics |
//! | Inequality Constraint | 8, 10 | KKT conditions, constrained optimization |
//! | Lyapunov Function | 2, 7, 9 | Stability theory, La Salle invariance |
//! | Structural Invariant | 5, 11 | Topological methods, algebraic invariants |
//!
//! ## §8.1 Law Enumeration
//!
//! | # | Law Name | Type Category | Mathematical Form |
//! |---|----------|---------------|-------------------|
//! | 1 | Mass/Information | Mass/Amount | Input = Output + Stored |
//! | 2 | Energy/Gradient | Energy/Gradient | ΔG < 0 for spontaneous |
//! | 3 | State Conservation | State | Σ states = constant |
//! | 4 | Flux Conservation | Flux | Σ J_in = Σ J_out |
//! | 5 | Catalyst Regeneration | Catalyst | Catalyst unchanged after cycle |
//! | 6 | Rate Conservation | Flux (derivative) | dA/dt = Rate_in - Rate_out |
//! | 7 | Equilibrium | Equilibrium | dX/dt → 0 at steady state |
//! | 8 | Saturation | Capacity | v = V_max·S/(K_m + S) |
//! | 9 | Entropy Production | Entropy | ΔS_total ≥ 0 |
//! | 10 | Discretization | Capacity (quantized) | Continuous → discrete |
//! | 11 | Structural Invariance | Structural | Structure preserved |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 8.0: CONSTRAINT TYPE TAXONOMY
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 8.0.1-8.0.4: Mathematical types of conservation constraints.
///
/// Despite their different mathematical natures, all eleven are treated uniformly
/// as constraint functions gᵢ(s,u,θ) ≤ 0 for the purpose of defining the safety
/// manifold M. The taxonomy clarifies which mathematical tools apply to each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintMathType {
    /// Definition 8.0.1: Strict Conservation Law.
    ///
    /// A smooth function C: S → ℝ such that dC(s(t))/dt = 0 along all system
    /// trajectories. Equivalently: C is a first integral of the dynamics.
    ///
    /// Laws: 1, 3, 4, 6 (mass, state, flux, rate)
    /// Analysis: Noether's theorem, Hamiltonian mechanics
    StrictConservation,

    /// Definition 8.0.2: Inequality Constraint.
    ///
    /// A function g: S × U × Θ → ℝ defining feasible region F = {s : g(s,u,θ) ≤ 0}.
    ///
    /// Laws: 8, 10 (saturation, discretization)
    /// Analysis: KKT conditions, constrained optimization
    InequalityConstraint,

    /// Definition 8.0.3: Lyapunov Function.
    ///
    /// V: S → ℝ≥0 with V(s*) = 0 at equilibrium s* and dV/dt ≤ 0 for s ≠ s*
    /// (with strict inequality for asymptotic stability).
    ///
    /// Laws: 2, 7, 9 (energy, equilibrium, entropy)
    /// Analysis: Stability theory, La Salle invariance principle
    LyapunovFunction,

    /// Definition 8.0.4: Structural Invariant.
    ///
    /// A discrete-valued function I: S → D (for discrete set D) constant on
    /// connected components of accessible state space.
    ///
    /// Laws: 5, 11 (catalyst, structure)
    /// Analysis: Topological methods, algebraic invariants
    StructuralInvariant,
}

impl ConstraintMathType {
    /// Get the analysis method for this constraint type.
    #[must_use]
    pub const fn analysis_method(&self) -> &'static str {
        match self {
            Self::StrictConservation => "Noether's theorem, Hamiltonian mechanics",
            Self::InequalityConstraint => "KKT conditions, constrained optimization",
            Self::LyapunovFunction => "Stability theory, La Salle invariance principle",
            Self::StructuralInvariant => "Topological methods, algebraic invariants",
        }
    }

    /// Get the definition reference.
    #[must_use]
    pub const fn definition(&self) -> &'static str {
        match self {
            Self::StrictConservation => "Definition 8.0.1",
            Self::InequalityConstraint => "Definition 8.0.2",
            Self::LyapunovFunction => "Definition 8.0.3",
            Self::StructuralInvariant => "Definition 8.0.4",
        }
    }

    /// Get which laws belong to this type.
    #[must_use]
    pub fn laws(&self) -> Vec<ConservationLawId> {
        match self {
            Self::StrictConservation => vec![
                ConservationLawId::Law1Mass,
                ConservationLawId::Law3State,
                ConservationLawId::Law4Flux,
                ConservationLawId::Law6Rate,
            ],
            Self::InequalityConstraint => vec![
                ConservationLawId::Law8Saturation,
                ConservationLawId::Law10Discretization,
            ],
            Self::LyapunovFunction => vec![
                ConservationLawId::Law2Energy,
                ConservationLawId::Law7Equilibrium,
                ConservationLawId::Law9Entropy,
            ],
            Self::StructuralInvariant => vec![
                ConservationLawId::Law5Catalyst,
                ConservationLawId::Law11Structure,
            ],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 8.0.5: TOLERANCE FUNCTION
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 8.0.5: Tolerance Function.
///
/// For each conservation constraint gᵢ, the tolerance εᵢ: Θ → ℝ>0 is a positive
/// function specifying acceptable deviation from exact satisfaction.
///
/// ## Specification
/// 1. εᵢ represents measurement precision, numerical tolerance, or acceptable deviation
/// 2. εᵢ(θ) may depend on system scale: εᵢ ∝ ||s|| (relative tolerance)
/// 3. εᵢ = 0 recovers exact conservation (mathematical limit case)
///
/// ## Nested Manifold Property
/// The ε-parameterized family of constraints defines nested safety manifolds:
/// M(ε₁) ⊇ M(ε₂) for ε₁ ≥ ε₂ (component-wise)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ToleranceFunction {
    /// Absolute tolerance (fixed deviation allowed).
    pub absolute: f64,
    /// Relative tolerance (proportion of value allowed).
    pub relative: f64,
}

impl Default for ToleranceFunction {
    fn default() -> Self {
        Self {
            absolute: 1e-10,
            relative: 0.0,
        }
    }
}

impl ToleranceFunction {
    /// Create a tolerance with only absolute value.
    #[must_use]
    pub const fn absolute(epsilon: f64) -> Self {
        Self {
            absolute: epsilon,
            relative: 0.0,
        }
    }

    /// Create a tolerance with only relative value.
    #[must_use]
    pub const fn relative(proportion: f64) -> Self {
        Self {
            absolute: 0.0,
            relative: proportion,
        }
    }

    /// Create a tolerance with both absolute and relative components.
    #[must_use]
    pub const fn combined(absolute: f64, relative: f64) -> Self {
        Self { absolute, relative }
    }

    /// Calculate effective tolerance at a given scale.
    ///
    /// ε_effective = ε_absolute + ε_relative × |scale|
    #[must_use]
    pub fn effective(&self, scale: f64) -> f64 {
        self.absolute + self.relative * scale.abs()
    }

    /// Domain-specific tolerance presets.
    #[must_use]
    pub fn for_domain(domain: ToleranceDomain) -> Self {
        match domain {
            ToleranceDomain::Pharmacovigilance => Self::relative(0.15), // 10-20% typical
            ToleranceDomain::Cloud => Self::relative(0.03),             // 1-5% typical
            ToleranceDomain::AI => Self::absolute(1e-4),                // 10⁻⁶ to 10⁻² typical
        }
    }
}

/// Domain for tolerance specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToleranceDomain {
    /// Pharmacovigilance: measurement error, bioequivalence margins (10-20% relative).
    Pharmacovigilance,
    /// Cloud: SLA tolerance, measurement precision (1-5% relative).
    Cloud,
    /// AI: Floating-point precision, acceptable drift (10⁻⁶ to 10⁻²).
    AI,
}

// ═══════════════════════════════════════════════════════════════════════════
// §8.1-8.2: CONSERVATION LAW CATALOG
// ═══════════════════════════════════════════════════════════════════════════

/// The eleven conservation laws of the Theory of Vigilance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConservationLawId {
    /// Law 1: Mass/Information Conservation.
    Law1Mass = 1,
    /// Law 2: Thermodynamic Directionality.
    Law2Energy = 2,
    /// Law 3: State Conservation.
    Law3State = 3,
    /// Law 4: Flux Conservation.
    Law4Flux = 4,
    /// Law 5: Catalyst Regeneration.
    Law5Catalyst = 5,
    /// Law 6: Rate Conservation (Local Form).
    Law6Rate = 6,
    /// Law 7: Equilibrium.
    Law7Equilibrium = 7,
    /// Law 8: Capacity Saturation.
    Law8Saturation = 8,
    /// Law 9: Entropy Production.
    Law9Entropy = 9,
    /// Law 10: Discretization.
    Law10Discretization = 10,
    /// Law 11: Structural Invariance.
    Law11Structure = 11,
}

impl ConservationLawId {
    /// All conservation laws in order.
    pub const ALL: [Self; 11] = [
        Self::Law1Mass,
        Self::Law2Energy,
        Self::Law3State,
        Self::Law4Flux,
        Self::Law5Catalyst,
        Self::Law6Rate,
        Self::Law7Equilibrium,
        Self::Law8Saturation,
        Self::Law9Entropy,
        Self::Law10Discretization,
        Self::Law11Structure,
    ];

    /// Get law number (1-11).
    #[must_use]
    pub const fn number(&self) -> u8 {
        *self as u8
    }

    /// Get the mathematical type of this law.
    #[must_use]
    pub const fn math_type(&self) -> ConstraintMathType {
        match self {
            Self::Law1Mass | Self::Law3State | Self::Law4Flux | Self::Law6Rate => {
                ConstraintMathType::StrictConservation
            }
            Self::Law8Saturation | Self::Law10Discretization => {
                ConstraintMathType::InequalityConstraint
            }
            Self::Law2Energy | Self::Law7Equilibrium | Self::Law9Entropy => {
                ConstraintMathType::LyapunovFunction
            }
            Self::Law5Catalyst | Self::Law11Structure => ConstraintMathType::StructuralInvariant,
        }
    }
}

/// Complete definition of a conservation law.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationLawDefinition {
    /// Law identifier.
    pub id: ConservationLawId,
    /// Law name.
    pub name: String,
    /// Statement of the law.
    pub statement: String,
    /// Mathematical form.
    pub mathematical_form: String,
    /// Constraint form: gᵢ(s, u, θ) ≤ 0.
    pub constraint_form: String,
    /// Mathematical type.
    pub math_type: ConstraintMathType,
    /// Relationship to other laws (if any).
    pub relationships: Vec<String>,
    /// Domain-specific examples.
    pub domain_examples: HashMap<String, String>,
}

impl ConservationLawDefinition {
    /// Get the full catalog of all 11 conservation laws.
    #[must_use]
    pub fn catalog() -> Vec<Self> {
        vec![
            Self::law1_mass(),
            Self::law2_energy(),
            Self::law3_state(),
            Self::law4_flux(),
            Self::law5_catalyst(),
            Self::law6_rate(),
            Self::law7_equilibrium(),
            Self::law8_saturation(),
            Self::law9_entropy(),
            Self::law10_discretization(),
            Self::law11_structure(),
        ]
    }

    /// Law 1: Mass/Information Conservation.
    #[must_use]
    pub fn law1_mass() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Drug mass balance: absorbed = excreted + metabolized + accumulated".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Data volume: bytes_in = bytes_out + bytes_stored".to_string(),
        );
        domain_examples.insert(
            "AI".to_string(),
            "Token count: input_tokens + generated_tokens = total_processed".to_string(),
        );

        Self {
            id: ConservationLawId::Law1Mass,
            name: "Mass/Information Conservation".to_string(),
            statement: "The total amount of conserved quantity (mass, information, data) \
                        entering a system boundary equals the amount leaving plus the \
                        amount accumulated within."
                .to_string(),
            mathematical_form: "dM/dt = J_in - J_out".to_string(),
            constraint_form: "g₁(s, u, θ) = |M_in - M_out - ΔM_stored| - ε ≤ 0".to_string(),
            math_type: ConstraintMathType::StrictConservation,
            relationships: vec!["Law 6 is the local/differential form of Law 1".to_string()],
            domain_examples,
        }
    }

    /// Law 2: Thermodynamic Directionality.
    #[must_use]
    pub fn law2_energy() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Binding free energy, metabolic potential".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Resource utilization cost, latency potential".to_string(),
        );
        domain_examples.insert(
            "AI".to_string(),
            "Loss function, alignment metric".to_string(),
        );

        Self {
            id: ConservationLawId::Law2Energy,
            name: "Thermodynamic Directionality".to_string(),
            statement: "System evolution follows energetically favorable directions. \
                        Three variants: (2a) Equilibrium ΔG < 0, (2b) Gradient flow ds/dt = -∇V, \
                        (2c) Dissipative dynamics dV/dt ≤ 0."
                .to_string(),
            mathematical_form: "dV/dt ≤ 0 along system trajectories".to_string(),
            constraint_form: "g₂(s, s', u, θ) = V(s') - V(s) ≤ 0 for transition s → s'".to_string(),
            math_type: ConstraintMathType::LyapunovFunction,
            relationships: vec![
                "2b ⟹ 2c (gradient flow is special case of dissipative)".to_string(),
                "2a ⟹ 2c (for isothermal, isobaric systems with V = G)".to_string(),
            ],
            domain_examples,
        }
    }

    /// Law 3: State Conservation.
    #[must_use]
    pub fn law3_state() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Receptor occupancy fractions sum to 1".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Service state probabilities sum to 1".to_string(),
        );
        domain_examples.insert("AI".to_string(), "Attention weights sum to 1".to_string());

        Self {
            id: ConservationLawId::Law3State,
            name: "State Conservation".to_string(),
            statement: "The total probability or fraction of system states sums to unity. \
                        States are mutually exclusive and exhaustive."
                .to_string(),
            mathematical_form: "Σᵢ pᵢ = 1".to_string(),
            constraint_form: "g₃(s, u, θ) = |Σᵢ sᵢ - 1| - ε ≤ 0".to_string(),
            math_type: ConstraintMathType::StrictConservation,
            relationships: vec![],
            domain_examples,
        }
    }

    /// Law 4: Flux Conservation.
    #[must_use]
    pub fn law4_flux() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Metabolic pathway flux balance".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Network traffic at routing nodes".to_string(),
        );
        domain_examples.insert("AI".to_string(), "Gradient flow through layers".to_string());

        Self {
            id: ConservationLawId::Law4Flux,
            name: "Flux Conservation".to_string(),
            statement: "At any node in a network, the total flux entering equals the \
                        total flux leaving (at steady state)."
                .to_string(),
            mathematical_form: "Σ J_in = Σ J_out at each node".to_string(),
            constraint_form: "g₄(s, u, θ) = |Σ J_in(s) - Σ J_out(s)| - ε ≤ 0".to_string(),
            math_type: ConstraintMathType::StrictConservation,
            relationships: vec![],
            domain_examples,
        }
    }

    /// Law 5: Catalyst Regeneration.
    #[must_use]
    pub fn law5_catalyst() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "E + S ⇌ ES → E + P (enzyme unchanged)".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Service instance available after request".to_string(),
        );
        domain_examples.insert(
            "AI".to_string(),
            "Model weights unchanged by inference".to_string(),
        );

        Self {
            id: ConservationLawId::Law5Catalyst,
            name: "Catalyst Regeneration".to_string(),
            statement: "Catalytic entities (enzymes, services, compute resources) return \
                        to their original state after facilitating a transformation."
                .to_string(),
            mathematical_form: "E + S ⇌ ES → E + P".to_string(),
            constraint_form: "g₅(s, u, θ) = |[E]_final - [E]_initial| - ε ≤ 0".to_string(),
            math_type: ConstraintMathType::StructuralInvariant,
            relationships: vec![],
            domain_examples,
        }
    }

    /// Law 6: Rate Conservation (Local Form).
    #[must_use]
    pub fn law6_rate() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Compartmental kinetics: dA/dt = k_in - k_out".to_string(),
        );
        domain_examples.insert("Cloud".to_string(), "Queue depth change rate".to_string());
        domain_examples.insert(
            "AI".to_string(),
            "Activation rate in recurrent units".to_string(),
        );

        Self {
            id: ConservationLawId::Law6Rate,
            name: "Rate Conservation (Local Form)".to_string(),
            statement: "The rate of change of a quantity at any internal node equals \
                        the net flux into that node. This is the LOCAL, DIFFERENTIAL \
                        form of Law 1."
                .to_string(),
            mathematical_form: "dAᵢ/dt = Σⱼ Jⱼ→ᵢ - Σₖ Jᵢ→ₖ".to_string(),
            constraint_form: "g₆(s, u, θ) = |dAᵢ/dt - (Σⱼ Jⱼ→ᵢ - Σₖ Jᵢ→ₖ)| - ε ≤ 0".to_string(),
            math_type: ConstraintMathType::StrictConservation,
            relationships: vec![
                "Law 1: GLOBAL (system boundary)".to_string(),
                "Law 6: LOCAL (internal nodes)".to_string(),
                "For multi-compartment: Law 6 gives n-1 constraints, Law 1 gives 1 global"
                    .to_string(),
            ],
            domain_examples,
        }
    }

    /// Law 7: Equilibrium.
    #[must_use]
    pub fn law7_equilibrium() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Steady-state drug concentration".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Queue length at saturation".to_string(),
        );
        domain_examples.insert("AI".to_string(), "Converged model parameters".to_string());

        Self {
            id: ConservationLawId::Law7Equilibrium,
            name: "Equilibrium".to_string(),
            statement: "Systems tend toward equilibrium states where time derivatives \
                        vanish and opposing processes balance."
                .to_string(),
            mathematical_form: "At equilibrium: dX/dt = 0, k_forward·[A] = k_reverse·[B]"
                .to_string(),
            constraint_form: "g₇(s, u, θ) = ||ds/dt|| - ε ≤ 0 (at claimed steady state)"
                .to_string(),
            math_type: ConstraintMathType::LyapunovFunction,
            relationships: vec![],
            domain_examples,
        }
    }

    /// Law 8: Capacity Saturation.
    #[must_use]
    pub fn law8_saturation() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Maximum enzyme velocity, clearance rate".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Maximum throughput (req/sec), bandwidth limit".to_string(),
        );
        domain_examples.insert(
            "AI".to_string(),
            "Maximum batch size, context window, inference rate".to_string(),
        );

        Self {
            id: ConservationLawId::Law8Saturation,
            name: "Capacity Saturation".to_string(),
            statement: "Processing rate is bounded and approaches maximum asymptotically \
                        as load increases. General properties: f(0)=0, f'(L)>0, \
                        lim_{L→∞}f(L)=V_max<∞, f''(L)≤0 for large L."
                .to_string(),
            mathematical_form: "v(L) = f(L) satisfying saturation properties".to_string(),
            constraint_form: "g₈(s, u, θ) = v(s) - V_max(θ) ≤ 0".to_string(),
            math_type: ConstraintMathType::InequalityConstraint,
            relationships: vec![],
            domain_examples,
        }
    }

    /// Law 9: Entropy Production.
    #[must_use]
    pub fn law9_entropy() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Irreversible metabolic degradation".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Information loss in lossy compression".to_string(),
        );
        domain_examples.insert(
            "AI".to_string(),
            "Increasing uncertainty in long generations".to_string(),
        );

        Self {
            id: ConservationLawId::Law9Entropy,
            name: "Entropy Production".to_string(),
            statement: "Total entropy of a closed system never decreases. \
                        Irreversible processes produce entropy."
                .to_string(),
            mathematical_form: "ΔS_total = ΔS_system + ΔS_surroundings ≥ 0".to_string(),
            constraint_form: "g₉(s, u, θ) = -ΔS_total(s → s') ≤ 0".to_string(),
            math_type: ConstraintMathType::LyapunovFunction,
            relationships: vec![],
            domain_examples,
        }
    }

    /// Law 10: Discretization.
    #[must_use]
    pub fn law10_discretization() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Discrete dosing units (tablets, vials)".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Discrete resource allocation (CPU cores, memory pages)".to_string(),
        );
        domain_examples.insert(
            "AI".to_string(),
            "Discrete token generation, quantized weights".to_string(),
        );

        Self {
            id: ConservationLawId::Law10Discretization,
            name: "Discretization".to_string(),
            statement: "Continuous quantities are allocated in discrete quanta. \
                        Allocation granularity imposes constraints."
                .to_string(),
            mathematical_form: "X_allocated ∈ {0, q, 2q, 3q, ...} for quantum q".to_string(),
            constraint_form: "g₁₀(s, u, θ) = |X - round(X/q)·q| - ε ≤ 0".to_string(),
            math_type: ConstraintMathType::InequalityConstraint,
            relationships: vec!["Constraint on allocation, not strict conservation".to_string()],
            domain_examples,
        }
    }

    /// Law 11: Structural Invariance.
    #[must_use]
    pub fn law11_structure() -> Self {
        let mut domain_examples = HashMap::new();
        domain_examples.insert(
            "Pharmacovigilance".to_string(),
            "Metabolic pathway topology preserved".to_string(),
        );
        domain_examples.insert(
            "Cloud".to_string(),
            "Service dependency graph, database schema".to_string(),
        );
        domain_examples.insert(
            "AI".to_string(),
            "Model architecture, attention pattern structure".to_string(),
        );

        Self {
            id: ConservationLawId::Law11Structure,
            name: "Structural Invariance".to_string(),
            statement: "Certain structural properties are preserved under system evolution. \
                        Architecture, schema, and interface contracts remain stable. \
                        Σ(s(t)) = Σ(s(0)) for structural signature Σ."
                .to_string(),
            mathematical_form: "Σ(s(t)) = Σ(s(0)) ∀t".to_string(),
            constraint_form: "g₁₁(s, u, θ) = 𝟙_{Σ(s) ≠ Σ_ref} ≤ 0".to_string(),
            math_type: ConstraintMathType::StructuralInvariant,
            relationships: vec![
                "Stability constraint, not strict conservation".to_string(),
                "Violation indicates architectural change or schema migration".to_string(),
            ],
            domain_examples,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LAW 8: SATURATION FUNCTION INSTANTIATIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Saturation function types for Law 8.
///
/// | Name | Form | Parameters | Use Case |
/// |------|------|------------|----------|
/// | Michaelis-Menten | V_max·L/(K_m + L) | V_max, K_m | Enzyme kinetics |
/// | Hill | V_max·Lⁿ/(K^n + Lⁿ) | V_max, K, n | Cooperative binding |
/// | Sigmoidal | V_max/(1 + e^{-k(L-L₀)}) | V_max, k, L₀ | Neural activation |
/// | Queueing | μ·min(L/λ, 1) | μ, λ | Service systems |
/// | Logarithmic | V_max·log(1 + L/K)/log(1 + L_max/K) | V_max, K, L_max | Diminishing returns |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SaturationFunctionType {
    /// Michaelis-Menten: v = V_max·S/(K_m + S).
    MichaelisMenten,
    /// Hill equation: v = V_max·Sⁿ/(K^n + Sⁿ).
    Hill,
    /// Sigmoidal: v = V_max/(1 + e^{-k(S-S₀)}).
    Sigmoidal,
    /// Queueing: v = μ·min(ρ, 1) where ρ = λ/μ.
    Queueing,
    /// Logarithmic: v = V_max·log(1 + S/K)/log(1 + S_max/K).
    Logarithmic,
}

/// Saturation function parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaturationParams {
    /// Maximum rate V_max.
    pub v_max: f64,
    /// Half-saturation constant K_m or K.
    pub k: f64,
    /// Hill coefficient (for Hill equation) or steepness (for sigmoidal).
    pub n: f64,
    /// Midpoint (for sigmoidal) or max load (for logarithmic).
    pub l0: f64,
}

impl Default for SaturationParams {
    fn default() -> Self {
        Self {
            v_max: 1.0,
            k: 1.0,
            n: 1.0,
            l0: 0.5,
        }
    }
}

impl SaturationParams {
    /// Michaelis-Menten parameters.
    #[must_use]
    pub fn michaelis_menten(v_max: f64, k_m: f64) -> Self {
        Self {
            v_max,
            k: k_m,
            n: 1.0,
            l0: 0.0,
        }
    }

    /// Hill equation parameters.
    #[must_use]
    pub fn hill(v_max: f64, k: f64, n: f64) -> Self {
        Self {
            v_max,
            k,
            n,
            l0: 0.0,
        }
    }

    /// Sigmoidal parameters.
    #[must_use]
    pub fn sigmoidal(v_max: f64, k: f64, l0: f64) -> Self {
        Self {
            v_max,
            k,
            n: 1.0,
            l0,
        }
    }
}

/// Saturation function evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaturationFunction {
    /// Function type.
    pub function_type: SaturationFunctionType,
    /// Parameters.
    pub params: SaturationParams,
}

impl SaturationFunction {
    /// Create a Michaelis-Menten saturation function.
    #[must_use]
    pub fn michaelis_menten(v_max: f64, k_m: f64) -> Self {
        Self {
            function_type: SaturationFunctionType::MichaelisMenten,
            params: SaturationParams::michaelis_menten(v_max, k_m),
        }
    }

    /// Create a Hill saturation function.
    #[must_use]
    pub fn hill(v_max: f64, k: f64, n: f64) -> Self {
        Self {
            function_type: SaturationFunctionType::Hill,
            params: SaturationParams::hill(v_max, k, n),
        }
    }

    /// Create a sigmoidal saturation function.
    #[must_use]
    pub fn sigmoidal(v_max: f64, k: f64, l0: f64) -> Self {
        Self {
            function_type: SaturationFunctionType::Sigmoidal,
            params: SaturationParams::sigmoidal(v_max, k, l0),
        }
    }

    /// Evaluate the saturation function at load L.
    #[must_use]
    pub fn evaluate(&self, load: f64) -> f64 {
        let p = &self.params;

        match self.function_type {
            SaturationFunctionType::MichaelisMenten => {
                // v = V_max·L/(K_m + L)
                p.v_max * load / (p.k + load)
            }
            SaturationFunctionType::Hill => {
                // v = V_max·Lⁿ/(K^n + Lⁿ)
                let l_n = load.powf(p.n);
                let k_n = p.k.powf(p.n);
                p.v_max * l_n / (k_n + l_n)
            }
            SaturationFunctionType::Sigmoidal => {
                // v = V_max/(1 + e^{-k(L-L₀)})
                p.v_max / (1.0 + (-p.k * (load - p.l0)).exp())
            }
            SaturationFunctionType::Queueing => {
                // v = μ·min(ρ, 1) where ρ = L/capacity
                // Using k as capacity (λ) and v_max as service rate (μ)
                let utilization = load / p.k;
                p.v_max * utilization.min(1.0)
            }
            SaturationFunctionType::Logarithmic => {
                // v = V_max·log(1 + L/K)/log(1 + L_max/K)
                let max_term = (1.0 + p.l0 / p.k).ln();
                let load_term = (1.0 + load / p.k).ln();
                p.v_max * load_term / max_term
            }
        }
    }

    /// Check if load is below capacity (constraint satisfied).
    #[must_use]
    pub fn is_below_capacity(&self, load: f64) -> bool {
        self.evaluate(load) < self.params.v_max
    }

    /// Calculate saturation ratio (how close to V_max).
    #[must_use]
    pub fn saturation_ratio(&self, load: f64) -> f64 {
        self.evaluate(load) / self.params.v_max
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// §8.3: TYPE-LAW RELATIONSHIPS
// ═══════════════════════════════════════════════════════════════════════════

/// Conservation type categories (from §4.5) and their associated laws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConservationTypeCategory {
    /// Mass/Amount: Law 1.
    MassAmount,
    /// Energy/Gradient: Law 2.
    EnergyGradient,
    /// State: Law 3.
    State,
    /// Flux: Laws 4, 6.
    Flux,
    /// Capacity: Laws 8, 10.
    Capacity,
    /// Catalyst/Regeneration: Law 5.
    Catalyst,
    /// Equilibrium: Law 7.
    Equilibrium,
    /// Entropy: Law 9.
    Entropy,
    /// Structure: Law 11.
    Structure,
}

impl ConservationTypeCategory {
    /// Get laws in this category.
    #[must_use]
    pub fn laws(&self) -> Vec<ConservationLawId> {
        match self {
            Self::MassAmount => vec![ConservationLawId::Law1Mass],
            Self::EnergyGradient => vec![ConservationLawId::Law2Energy],
            Self::State => vec![ConservationLawId::Law3State],
            Self::Flux => vec![ConservationLawId::Law4Flux, ConservationLawId::Law6Rate],
            Self::Capacity => vec![
                ConservationLawId::Law8Saturation,
                ConservationLawId::Law10Discretization,
            ],
            Self::Catalyst => vec![ConservationLawId::Law5Catalyst],
            Self::Equilibrium => vec![ConservationLawId::Law7Equilibrium],
            Self::Entropy => vec![ConservationLawId::Law9Entropy],
            Self::Structure => vec![ConservationLawId::Law11Structure],
        }
    }

    /// Get category for a given law.
    #[must_use]
    pub fn from_law(law: ConservationLawId) -> Self {
        match law {
            ConservationLawId::Law1Mass => Self::MassAmount,
            ConservationLawId::Law2Energy => Self::EnergyGradient,
            ConservationLawId::Law3State => Self::State,
            ConservationLawId::Law4Flux | ConservationLawId::Law6Rate => Self::Flux,
            ConservationLawId::Law5Catalyst => Self::Catalyst,
            ConservationLawId::Law7Equilibrium => Self::Equilibrium,
            ConservationLawId::Law8Saturation | ConservationLawId::Law10Discretization => {
                Self::Capacity
            }
            ConservationLawId::Law9Entropy => Self::Entropy,
            ConservationLawId::Law11Structure => Self::Structure,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 8.1: STRUCTURAL SIGNATURE
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 8.1: Structural Signature.
///
/// A function Σ: S → D where D is a discrete space capturing topologically
/// or algebraically significant properties that should remain constant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSignature {
    /// Signature name.
    pub name: String,
    /// Signature type.
    pub signature_type: StructuralSignatureType,
    /// Reference value Σ_ref.
    pub reference_value: i64,
    /// Current value Σ(s).
    pub current_value: i64,
}

/// Types of structural signatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructuralSignatureType {
    /// Graph connectivity: number of connected components.
    Connectivity,
    /// Dimensional structure: rank of system Jacobian.
    Dimension,
    /// Type signature: interface type hash.
    TypeSignature,
    /// Symmetry group: stabilizer subgroup identifier.
    Symmetry,
    /// Schema version: database schema hash.
    SchemaVersion,
}

impl StructuralSignature {
    /// Create a new structural signature.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        signature_type: StructuralSignatureType,
        reference_value: i64,
    ) -> Self {
        Self {
            name: name.into(),
            signature_type,
            reference_value,
            current_value: reference_value,
        }
    }

    /// Check if signature is preserved (Law 11 satisfied).
    #[must_use]
    pub fn is_preserved(&self) -> bool {
        self.current_value == self.reference_value
    }

    /// Update current value.
    pub fn update(&mut self, value: i64) {
        self.current_value = value;
    }

    /// Evaluate constraint: g₁₁ = 𝟙_{Σ(s) ≠ Σ_ref}.
    #[must_use]
    pub fn evaluate_constraint(&self) -> f64 {
        if self.is_preserved() { 0.0 } else { 1.0 }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_math_types() {
        assert_eq!(
            ConservationLawId::Law1Mass.math_type(),
            ConstraintMathType::StrictConservation
        );
        assert_eq!(
            ConservationLawId::Law8Saturation.math_type(),
            ConstraintMathType::InequalityConstraint
        );
        assert_eq!(
            ConservationLawId::Law2Energy.math_type(),
            ConstraintMathType::LyapunovFunction
        );
        assert_eq!(
            ConservationLawId::Law11Structure.math_type(),
            ConstraintMathType::StructuralInvariant
        );
    }

    #[test]
    fn test_law_numbers() {
        for (i, law) in ConservationLawId::ALL.iter().enumerate() {
            assert_eq!(law.number() as usize, i + 1);
        }
    }

    #[test]
    fn test_conservation_law_catalog() {
        let catalog = ConservationLawDefinition::catalog();
        assert_eq!(catalog.len(), 11);

        // Verify each law has required fields
        for def in &catalog {
            assert!(!def.name.is_empty());
            assert!(!def.statement.is_empty());
            assert!(!def.mathematical_form.is_empty());
            assert!(!def.constraint_form.is_empty());
        }
    }

    #[test]
    fn test_tolerance_function() {
        let tol = ToleranceFunction::combined(0.01, 0.05);

        // At scale 1.0: 0.01 + 0.05*1.0 = 0.06
        assert!((tol.effective(1.0) - 0.06).abs() < 1e-10);

        // At scale 10.0: 0.01 + 0.05*10.0 = 0.51
        assert!((tol.effective(10.0) - 0.51).abs() < 1e-10);
    }

    #[test]
    fn test_domain_tolerances() {
        let pv = ToleranceFunction::for_domain(ToleranceDomain::Pharmacovigilance);
        assert!(pv.relative > 0.1); // 10-20% range

        let cloud = ToleranceFunction::for_domain(ToleranceDomain::Cloud);
        assert!(cloud.relative < 0.1); // 1-5% range

        let ai = ToleranceFunction::for_domain(ToleranceDomain::AI);
        assert!(ai.absolute < 0.01); // 10⁻² or smaller
    }

    #[test]
    fn test_michaelis_menten() {
        let mm = SaturationFunction::michaelis_menten(100.0, 10.0);

        // At S = K_m, v = V_max/2
        let v_half = mm.evaluate(10.0);
        assert!((v_half - 50.0).abs() < 1e-10);

        // At S >> K_m, v → V_max
        let v_high = mm.evaluate(1000.0);
        assert!(v_high > 99.0);

        // At S = 0, v = 0
        let v_zero = mm.evaluate(0.0);
        assert!(v_zero.abs() < 1e-10);
    }

    #[test]
    fn test_hill_equation() {
        // Hill coefficient n = 2 (cooperative)
        let hill = SaturationFunction::hill(100.0, 10.0, 2.0);

        // At S = K, v = V_max/2 (for any n)
        let v_half = hill.evaluate(10.0);
        assert!((v_half - 50.0).abs() < 1e-10);

        // Hill n > 1 gives sigmoidal response (steeper at K)
        let v_low = hill.evaluate(5.0); // Below K
        let mm = SaturationFunction::michaelis_menten(100.0, 10.0);
        let v_low_mm = mm.evaluate(5.0);

        // Hill should be lower than MM at low substrate (cooperative binding)
        assert!(v_low < v_low_mm);
    }

    #[test]
    fn test_sigmoidal() {
        let sig = SaturationFunction::sigmoidal(1.0, 1.0, 0.5);

        // At L = L0, v = V_max/2
        let v_mid = sig.evaluate(0.5);
        assert!((v_mid - 0.5).abs() < 1e-10);

        // At L >> L0, v → V_max
        let v_high = sig.evaluate(10.0);
        assert!(v_high > 0.99);

        // At L << L0, v → 0
        let v_low = sig.evaluate(-10.0);
        assert!(v_low < 0.01);
    }

    #[test]
    fn test_saturation_ratio() {
        let mm = SaturationFunction::michaelis_menten(100.0, 10.0);

        // At K_m, ratio = 0.5
        let ratio = mm.saturation_ratio(10.0);
        assert!((ratio - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_type_category_laws() {
        // Flux category contains Laws 4 and 6
        let flux_laws = ConservationTypeCategory::Flux.laws();
        assert_eq!(flux_laws.len(), 2);
        assert!(flux_laws.contains(&ConservationLawId::Law4Flux));
        assert!(flux_laws.contains(&ConservationLawId::Law6Rate));

        // Capacity category contains Laws 8 and 10
        let cap_laws = ConservationTypeCategory::Capacity.laws();
        assert_eq!(cap_laws.len(), 2);
        assert!(cap_laws.contains(&ConservationLawId::Law8Saturation));
        assert!(cap_laws.contains(&ConservationLawId::Law10Discretization));
    }

    #[test]
    fn test_category_from_law() {
        assert_eq!(
            ConservationTypeCategory::from_law(ConservationLawId::Law1Mass),
            ConservationTypeCategory::MassAmount
        );
        assert_eq!(
            ConservationTypeCategory::from_law(ConservationLawId::Law4Flux),
            ConservationTypeCategory::Flux
        );
        assert_eq!(
            ConservationTypeCategory::from_law(ConservationLawId::Law6Rate),
            ConservationTypeCategory::Flux
        );
    }

    #[test]
    fn test_structural_signature() {
        let mut sig =
            StructuralSignature::new("ComponentCount", StructuralSignatureType::Connectivity, 5);

        // Initially preserved
        assert!(sig.is_preserved());
        assert!((sig.evaluate_constraint() - 0.0).abs() < 1e-10);

        // After structural change
        sig.update(4);
        assert!(!sig.is_preserved());
        assert!((sig.evaluate_constraint() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_math_type_laws() {
        // Strict conservation has 4 laws
        let strict_laws = ConstraintMathType::StrictConservation.laws();
        assert_eq!(strict_laws.len(), 4);

        // Lyapunov has 3 laws
        let lyap_laws = ConstraintMathType::LyapunovFunction.laws();
        assert_eq!(lyap_laws.len(), 3);

        // Inequality has 2 laws
        let ineq_laws = ConstraintMathType::InequalityConstraint.laws();
        assert_eq!(ineq_laws.len(), 2);

        // Structural has 2 laws
        let struct_laws = ConstraintMathType::StructuralInvariant.laws();
        assert_eq!(struct_laws.len(), 2);

        // Total: 4 + 3 + 2 + 2 = 11
        assert_eq!(
            strict_laws.len() + lyap_laws.len() + ineq_laws.len() + struct_laws.len(),
            11
        );
    }

    #[test]
    fn test_law2_variants_documented() {
        let law2 = ConservationLawDefinition::law2_energy();
        assert!(law2.statement.contains("(2a)"));
        assert!(law2.statement.contains("(2b)"));
        assert!(law2.statement.contains("(2c)"));
        assert_eq!(law2.relationships.len(), 2);
    }

    #[test]
    fn test_law1_law6_relationship() {
        let law1 = ConservationLawDefinition::law1_mass();
        let law6 = ConservationLawDefinition::law6_rate();

        // Law 1 references Law 6
        assert!(law1.relationships.iter().any(|r| r.contains("Law 6")));

        // Law 6 references Law 1
        assert!(law6.relationships.iter().any(|r| r.contains("Law 1")));
    }
}
