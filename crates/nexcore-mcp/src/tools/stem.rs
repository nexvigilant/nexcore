//! STEM primitives MCP tools
//!
//! Exposes cross-domain T2-P primitives from the STEM crate system
//! (Science, Technology, Engineering, Mathematics) via MCP.
//!
//! 32 traits across 4 domains, all grounded to T1 primitives.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    StemBioBehaviorProfileParams, StemBioToneProfileParams, StemChemAffinityParams,
    StemChemBalanceParams, StemChemFractionParams, StemChemRateParams, StemChemRatioParams,
    StemConfidenceCombineParams, StemDeterminismScoreParams, StemFinanceArbitrageParams,
    StemFinanceCompoundParams, StemFinanceDiscountParams, StemFinanceDiversifyParams,
    StemFinanceExposureParams, StemFinanceMaturityParams, StemFinanceReturnParams,
    StemFinanceSpreadParams, StemIntegrityCheckParams, StemMathBoundsCheckParams,
    StemMathIdentityParams, StemMathProofParams, StemMathRelationInvertParams,
    StemPhysAmplitudeParams, StemPhysConservationParams, StemPhysFmaParams, StemPhysInertiaParams,
    StemPhysPeriodParams, StemPhysScaleParams, StemRetryBudgetParams, StemSpatialDimensionParams,
    StemSpatialDistanceParams, StemSpatialNeighborhoodParams, StemSpatialOrientationParams,
    StemSpatialTriangleParams, StemStatsAnalyzeParams, StemStatsCiParams, StemStatsPValueParams,
    StemStatsZTestParams, StemTierInfoParams, StemTransferConfidenceParams,
};

/// Get STEM version, trait count, and domain summary.
pub fn version() -> Result<CallToolResult, McpError> {
    let result = serde_json::json!({
        "version": stem::VERSION,
        "trait_count": stem::TRAIT_COUNT,
        "domain_count": stem::DOMAIN_COUNT,
        "domains": {
            "science": {
                "crate": "stem-core",
                "traits": ["Sense", "Classify", "Infer", "Experiment", "Normalize", "Codify", "Extend"],
                "count": 7,
                "composite": "Science"
            },
            "chemistry": {
                "crate": "stem-chem",
                "traits": ["Concentrate", "Harmonize", "Energize", "Modulate", "Interact", "Saturate", "Transform", "Regulate", "Yield"],
                "count": 9,
                "composite": "Chemistry"
            },
            "physics": {
                "crate": "stem-phys",
                "traits": ["Preserve", "Harmonics", "YieldForce", "Superpose", "Inertia", "Couple", "Scale"],
                "count": 7,
                "composite": "Physics"
            },
            "mathematics": {
                "crate": "stem-math",
                "traits": ["Membership", "Associate", "Transit", "Homeomorph", "Symmetric", "Bound", "Prove", "Commute", "Identify"],
                "count": 9,
                "composite": "Mathematics"
            }
        },
        "t1_distribution": {
            "MAPPING": 14,
            "SEQUENCE": 5,
            "RECURSION": 5,
            "STATE": 4,
            "PERSISTENCE": 2,
            "BOUNDARY": 1,
            "SUM": 1
        },
        "trait_aliases": {"Apply": "Transform", "Map": "Transform", "Convert": "Transform"},
        "three_unfixable_limits": [
            "Heisenberg: Observation alters the observed",
            "Gödel: No system proves its own consistency",
            "Shannon: Codification has irreducible loss"
        ]
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Get the full 32-trait taxonomy with T1 groundings.
pub fn taxonomy() -> Result<CallToolResult, McpError> {
    let result = serde_json::json!({
        "taxonomy": {
            "SEQUENCE": [
                {"trait": "Experiment", "domain": "Science", "transfer": "Action → Outcome causality test"},
                {"trait": "Interact", "domain": "Chemistry", "transfer": "Ligand → Affinity binding"},
                {"trait": "Couple", "domain": "Physics", "transfer": "Action → Reaction (Newton's 3rd)"},
                {"trait": "Transit", "domain": "Mathematics", "transfer": "a→b ∧ b→c ⟹ a→c"},
                {"trait": "Prove", "domain": "Mathematics", "transfer": "Premises → Conclusion"}
            ],
            "MAPPING": [
                {"trait": "Sense", "domain": "Science", "transfer": "Environment → Signal"},
                {"trait": "Classify", "domain": "Science", "transfer": "Signal → Category"},
                {"trait": "Codify", "domain": "Science", "transfer": "Belief → Representation"},
                {"trait": "Extend", "domain": "Science", "transfer": "Source → Target domain"},
                {"trait": "Concentrate", "domain": "Chemistry", "transfer": "Substance → Ratio"},
                {"trait": "Energize", "domain": "Chemistry", "transfer": "Energy → Rate"},
                {"trait": "Transform", "domain": "Chemistry", "transfer": "Reactants → Products"},
                {"trait": "Yield", "domain": "Chemistry", "transfer": "Actual / Theoretical"},
                {"trait": "YieldForce", "domain": "Physics", "transfer": "Force → Acceleration"},
                {"trait": "Scale", "domain": "Physics", "transfer": "Proportional transform"},
                {"trait": "Membership", "domain": "Mathematics", "transfer": "Element ∈ Set"},
                {"trait": "Homeomorph", "domain": "Mathematics", "transfer": "Structure-preserving map"},
                {"trait": "Symmetric", "domain": "Mathematics", "transfer": "a~b ⟹ b~a"},
                {"trait": "Commute", "domain": "Mathematics", "transfer": "a·b = b·a"}
            ],
            "RECURSION": [
                {"trait": "Infer", "domain": "Science", "transfer": "Pattern × Data → Prediction"},
                {"trait": "Modulate", "domain": "Chemistry", "transfer": "Catalyst → Rate change"},
                {"trait": "Regulate", "domain": "Chemistry", "transfer": "Inhibitor → Rate decrease"},
                {"trait": "Harmonics", "domain": "Physics", "transfer": "Oscillation around center"},
                {"trait": "Associate", "domain": "Mathematics", "transfer": "(a·b)·c = a·(b·c)"}
            ],
            "STATE": [
                {"trait": "Normalize", "domain": "Science", "transfer": "Prior × Evidence → Posterior"},
                {"trait": "Harmonize", "domain": "Chemistry", "transfer": "System → Equilibrium"},
                {"trait": "Saturate", "domain": "Chemistry", "transfer": "Capacity → Fraction"},
                {"trait": "Identify", "domain": "Mathematics", "transfer": "Neutral element"}
            ],
            "PERSISTENCE": [
                {"trait": "Preserve", "domain": "Physics", "transfer": "Quantity unchanged across transform"},
                {"trait": "Inertia", "domain": "Physics", "transfer": "Resistance to change"}
            ],
            "BOUNDARY": [
                {"trait": "Bound", "domain": "Mathematics", "transfer": "Upper/lower limits"}
            ],
            "SUM": [
                {"trait": "Superpose", "domain": "Physics", "transfer": "Sum of parts = whole"}
            ]
        },
        "total": 32,
        "distribution": "MAPPING(14) · SEQUENCE(5) · RECURSION(5) · STATE(4) · PERSISTENCE(2) · BOUNDARY(1) · SUM(1)",
        "trait_aliases": {
            "Apply": {"canonical": "Transform", "grounding": "MAPPING", "rationale": "Apply is Transform with no additional discriminating power — substitution test confirms equivalence"},
            "Map": {"canonical": "Transform", "grounding": "MAPPING", "rationale": "Map is Transform in functional programming terminology"},
            "Convert": {"canonical": "Transform", "grounding": "MAPPING", "rationale": "Convert is domain-narrowed Transform (type A → type B)"}
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Combine two confidence values (multiplicative composition).
pub fn confidence_combine(params: StemConfidenceCombineParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Confidence;

    let a = Confidence::new(params.a);
    let b = Confidence::new(params.b);
    let combined = a.combine(b);

    let result = serde_json::json!({
        "a": a.value(),
        "b": b.value(),
        "combined": combined.value(),
        "method": "multiplicative",
        "formula": "combined = a × b (clamped to [0.0, 1.0])",
        "note": "Confidence decreases when composing uncertain measurements"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Get tier classification info and transfer multiplier.
pub fn tier_info(params: StemTierInfoParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Tier;

    let tier = match params.tier.to_lowercase().as_str() {
        "t1" | "t1universal" | "universal" => Tier::T1Universal,
        "t2p" | "t2-p" | "t2primitive" | "primitive" => Tier::T2Primitive,
        "t2c" | "t2-c" | "t2composite" | "composite" => Tier::T2Composite,
        "t3" | "t3domainspecific" | "domain" | "domainspecific" => Tier::T3DomainSpecific,
        _ => {
            let result = serde_json::json!({
                "error": format!("Unknown tier: '{}'. Use: T1, T2-P, T2-C, or T3", params.tier),
                "valid_tiers": ["T1 (Universal)", "T2-P (Cross-Domain Primitive)", "T2-C (Cross-Domain Composite)", "T3 (Domain-Specific)"]
            });
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]));
        }
    };

    let multiplier = tier.transfer_multiplier();

    let (name, rust_pattern, example) = match tier {
        Tier::T1Universal => (
            "Universal Primitive",
            "Native types, (), POD structs",
            "type Exists = bool;",
        ),
        Tier::T2Primitive => (
            "Cross-Domain Primitive",
            "Newtypes over T1, no logic",
            "struct Score(f64);",
        ),
        Tier::T2Composite => (
            "Cross-Domain Composite",
            "Composed from T1/T2-P + traits",
            "struct Signal { s: Score }",
        ),
        Tier::T3DomainSpecific => (
            "Domain-Specific",
            "Full types with domain logic",
            "struct AdverseEvent { ... }",
        ),
    };

    let result = serde_json::json!({
        "tier": format!("{tier:?}"),
        "name": name,
        "transfer_multiplier": multiplier,
        "rust_pattern": rust_pattern,
        "example": example,
        "interpretation": format!("Cross-domain transfer confidence is scaled by {multiplier:.1}")
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Chemistry Domain Tools
// ============================================================================

/// Calculate equilibrium balance from forward and reverse rates.
pub fn chem_balance(params: StemChemBalanceParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Balance, Rate};

    let forward = Rate::new(params.forward_rate);
    let reverse = Rate::new(params.reverse_rate);
    let balance = Balance::new(forward, reverse);

    let tolerance = params.tolerance.unwrap_or(0.01);

    let result = serde_json::json!({
        "forward_rate": forward.value(),
        "reverse_rate": reverse.value(),
        "equilibrium_constant_k": balance.constant,
        "is_equilibrium": balance.is_equilibrium(tolerance),
        "products_favored": balance.products_favored(),
        "tolerance": tolerance,
        "t1_grounding": "STATE (Harmonize trait)",
        "cross_domain": "Baseline reporting rate equilibrium"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Create a Fraction and check saturation status.
pub fn chem_fraction(params: StemChemFractionParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Fraction;

    let fraction = Fraction::new(params.value);

    let result = serde_json::json!({
        "value": fraction.value(),
        "is_saturated": fraction.is_saturated(),
        "saturation_threshold": 0.99,
        "clamped_to": "[0.0, 1.0]",
        "t1_grounding": "STATE (Saturate trait)",
        "cross_domain": "Capacity utilization fraction"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Physics Domain Tools
// ============================================================================

/// Calculate acceleration from force and mass (F = ma → a = F/m).
pub fn phys_fma(params: StemPhysFmaParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Acceleration, Force, Mass};

    let force = Force::new(params.force);
    let mass = Mass::new(params.mass);
    let accel = Acceleration::from_force_and_mass(force, mass);

    let result = serde_json::json!({
        "force": force.value(),
        "mass": mass.value(),
        "acceleration": accel.value(),
        "formula": "a = F / m",
        "t1_grounding": "MAPPING (YieldForce trait)",
        "cross_domain": "Dose-response relationship (force → effect)"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Check quantity conservation (before vs after within tolerance).
pub fn phys_conservation(params: StemPhysConservationParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Quantity;

    let before = Quantity::new(params.before);
    let after = Quantity::new(params.after);
    let conserved = before.conserved_with(&after, params.tolerance);

    let result = serde_json::json!({
        "before": before.value(),
        "after": after.value(),
        "tolerance": params.tolerance,
        "is_conserved": conserved,
        "difference": (before.value() - after.value()).abs(),
        "t1_grounding": "STATE (Preserve trait)",
        "cross_domain": "Case count conservation in reporting pipelines"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Convert frequency to period (period = 1/frequency).
pub fn phys_period(params: StemPhysPeriodParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Frequency;

    let freq = Frequency::new(params.frequency);
    let period = freq.period();

    let result = serde_json::json!({
        "frequency": freq.value(),
        "period": period,
        "formula": "T = 1/f",
        "t1_grounding": "RECURSION (Harmonics trait)",
        "cross_domain": "Seasonal reporting patterns (cycle detection)"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Mathematics Domain Tools
// ============================================================================

/// Check if a value is within bounds.
pub fn math_bounds_check(params: StemMathBoundsCheckParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Bounded;

    let bounded = Bounded::new(params.value, params.lower, params.upper);
    let in_bounds = bounded.in_bounds();
    let clamped = bounded.clamp();

    let result = serde_json::json!({
        "value": params.value,
        "lower": params.lower,
        "upper": params.upper,
        "in_bounds": in_bounds,
        "clamped_value": clamped,
        "needed_clamping": (clamped - params.value).abs() > f64::EPSILON,
        "t1_grounding": "STATE (Bound trait)",
        "cross_domain": "Confidence interval / threshold boundary checking"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Invert a mathematical relation.
pub fn math_relation_invert(
    params: StemMathRelationInvertParams,
) -> Result<CallToolResult, McpError> {
    use stem::prelude::Relation;

    let relation = match params.relation.to_lowercase().as_str() {
        "lessthan" | "lt" | "<" => Relation::LessThan,
        "equal" | "eq" | "=" | "==" => Relation::Equal,
        "greaterthan" | "gt" | ">" => Relation::GreaterThan,
        "incomparable" | "nc" | "?" => Relation::Incomparable,
        _ => {
            let result = serde_json::json!({
                "error": format!("Unknown relation: '{}'. Use: LessThan, Equal, GreaterThan, Incomparable", params.relation),
                "valid_relations": ["LessThan (<)", "Equal (=)", "GreaterThan (>)", "Incomparable (?)"]
            });
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]));
        }
    };

    let inverted = relation.invert();
    let is_symmetric = relation.is_symmetric();

    let result = serde_json::json!({
        "original": format!("{relation:?}"),
        "inverted": format!("{inverted:?}"),
        "is_symmetric": is_symmetric,
        "t1_grounding": "MAPPING (Symmetric trait)",
        "cross_domain": "Bidirectional signal comparison"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// New Chemistry Tools
// ============================================================================

/// Create a Ratio value and optionally compute fold change vs a second ratio.
pub fn chem_ratio(params: StemChemRatioParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Ratio;

    let ratio = Ratio::new(params.value);
    let fold_change = params.compare_to.and_then(|other| {
        let other_val = Ratio::new(other).value();
        if other_val > 0.0 {
            Some(ratio.value() / other_val)
        } else {
            None
        }
    });
    let direction = fold_change.map(|fc| {
        if fc > 1.0 {
            "increased"
        } else if fc < 1.0 {
            "decreased"
        } else {
            "unchanged"
        }
    });

    let result = serde_json::json!({
        "value": ratio.value(),
        "clamped_to": "[0.0, ∞)",
        "compare_to": params.compare_to,
        "fold_change": fold_change,
        "direction": direction,
        "t1_grounding": "MAPPING (Concentrate trait: Substance → Ratio)",
        "cross_domain": "Case density per population / request rate per server"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Create a Rate value and optionally compute ratio vs a second rate.
pub fn chem_rate(params: StemChemRateParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Rate;

    let rate = Rate::new(params.value);
    let rate_ratio = params.compare_to.map(|other| {
        let other_val = Rate::new(other).value();
        if other_val > 0.0 {
            rate.value() / other_val
        } else {
            f64::INFINITY
        }
    });

    let result = serde_json::json!({
        "value": rate.value(),
        "clamped_to": "[0.0, ∞)",
        "compare_to": params.compare_to,
        "rate_ratio": rate_ratio,
        "t1_grounding": "MAPPING (Energize trait: Input → Activation → Rate)",
        "cross_domain": "Signal detection rate / event processing rate"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Create an Affinity value and classify binding strength.
pub fn chem_affinity(params: StemChemAffinityParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Affinity;

    let affinity = Affinity::new(params.value);
    let threshold = params.strong_threshold.unwrap_or(0.7);
    let strength = if affinity.value() >= threshold {
        "strong"
    } else if affinity.value() >= 0.3 {
        "moderate"
    } else {
        "weak"
    };

    let result = serde_json::json!({
        "value": affinity.value(),
        "clamped_to": "[0.0, 1.0]",
        "strong_threshold": threshold,
        "strength": strength,
        "is_strong": affinity.value() >= threshold,
        "t1_grounding": "MAPPING (Interact trait: Ligand → Affinity binding)",
        "cross_domain": "Drug-receptor binding strength / API coupling tightness"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// New Physics Tools
// ============================================================================

/// Create an Amplitude value and optionally superpose with a second amplitude.
pub fn phys_amplitude(params: StemPhysAmplitudeParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Amplitude;

    let amplitude = Amplitude::new(params.value);
    let superposed = params
        .superpose_with
        .map(|other| (amplitude + Amplitude::new(other)).value());

    let result = serde_json::json!({
        "value": amplitude.value(),
        "clamped_to": "[0.0, ∞)",
        "superpose_with": params.superpose_with,
        "superposed_value": superposed,
        "t1_grounding": "SUM (Superpose trait: Sum of parts = whole)",
        "cross_domain": "Combined signal amplitude / aggregated portfolio value"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Apply a ScaleFactor to a value.
pub fn phys_scale(params: StemPhysScaleParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::ScaleFactor;

    let factor = ScaleFactor::new(params.factor);
    let scaled = factor.apply(params.apply_to);
    let is_identity = (factor.value() - 1.0).abs() < f64::EPSILON;

    let result = serde_json::json!({
        "factor": factor.value(),
        "input": params.apply_to,
        "scaled_output": scaled,
        "is_identity": is_identity,
        "formula": "output = input × factor",
        "t1_grounding": "MAPPING (Scale trait: Proportional dimension transform)",
        "cross_domain": "Population adjustment / currency conversion / unit normalization"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate resistance to change from mass and proposed change magnitude.
pub fn phys_inertia(params: StemPhysInertiaParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Force, Mass};

    let mass = Mass::new(params.mass);
    let resistance = Force::new(mass.value() * params.proposed_change.abs());
    let inertia_class = if mass.value() < 0.1 {
        "negligible"
    } else if mass.value() < 1.0 {
        "low"
    } else if mass.value() < 10.0 {
        "moderate"
    } else {
        "high"
    };

    let result = serde_json::json!({
        "mass": mass.value(),
        "proposed_change": params.proposed_change,
        "resistance_force": resistance.value(),
        "inertia_class": inertia_class,
        "formula": "F = m × |Δx|",
        "t1_grounding": "PERSISTENCE (Inertia trait: Resistance to state change)",
        "cross_domain": "Reporting behavior lag / market momentum / cache persistence"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// New Mathematics Tools
// ============================================================================

/// Construct a Proof from premises and conclusion.
pub fn math_proof(params: StemMathProofParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Proof;

    let proof: Proof<String, String> = if params.valid {
        Proof::valid(params.premises.clone(), params.conclusion.clone())
    } else {
        Proof::invalid(params.premises.clone(), params.conclusion.clone())
    };
    let proof_type = if proof.valid {
        "valid deduction"
    } else {
        "counterexample"
    };

    let result = serde_json::json!({
        "premises": proof.premises,
        "conclusion": proof.conclusion,
        "valid": proof.valid,
        "premise_count": params.premises.len(),
        "proof_type": proof_type,
        "t1_grounding": "SEQUENCE (Prove trait: Premises → Conclusion)",
        "cross_domain": "Causality assessment / due diligence validation / type checking"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Check if a value is the identity element for an operation.
pub fn math_identity(params: StemMathIdentityParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Identity;

    let (expected, op_formula) = match params.operation.to_lowercase().as_str() {
        "multiply" | "mul" | "*" | "×" => (1.0_f64, "a × e = e × a = a"),
        _ => (0.0_f64, "a + e = e + a = a"),
    };
    let identity = Identity(expected);
    let is_identity = (params.value - expected).abs() < f64::EPSILON;
    let test_result = if is_identity {
        params.test_value
    } else {
        match params.operation.to_lowercase().as_str() {
            "multiply" | "mul" | "*" | "×" => params.test_value * params.value,
            _ => params.test_value + params.value,
        }
    };

    let result = serde_json::json!({
        "value": params.value,
        "operation": params.operation,
        "expected_identity": *identity.value(),
        "is_identity": is_identity,
        "test_value": params.test_value,
        "test_result": test_result,
        "formula": op_formula,
        "t1_grounding": "STATE (Identify trait: Neutral element)",
        "cross_domain": "Baseline (no effect) / zero position / Option::None"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Spatial Tools (computed — no STEM spatial module)
// ============================================================================

/// Create a Distance value and optionally compare for approximate equality.
pub fn spatial_distance(params: StemSpatialDistanceParams) -> Result<CallToolResult, McpError> {
    let distance = params.value.max(0.0);
    let tolerance = params.tolerance.unwrap_or(0.001);
    let approximately_equal = params
        .compare_to
        .map(|other| (distance - other.max(0.0)).abs() <= tolerance);

    let result = serde_json::json!({
        "value": distance,
        "clamped_to": "[0.0, ∞)",
        "compare_to": params.compare_to,
        "tolerance": tolerance,
        "approximately_equal": approximately_equal,
        "t1_grounding": "COMPARISON (κ: Distance comparison)",
        "cross_domain": "Signal divergence / edit distance / semantic proximity"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Check the triangle inequality: d(a,c) ≤ d(a,b) + d(b,c).
pub fn spatial_triangle(params: StemSpatialTriangleParams) -> Result<CallToolResult, McpError> {
    let ab = params.ab.max(0.0);
    let bc = params.bc.max(0.0);
    let ac = params.ac.max(0.0);
    let check_ac = ac <= ab + bc;
    let check_bc = bc <= ab + ac;
    let check_ab = ab <= bc + ac;
    let valid = check_ac && check_bc && check_ab;

    let result = serde_json::json!({
        "ab": ab,
        "bc": bc,
        "ac": ac,
        "triangle_inequality_valid": valid,
        "checks": {
            "ac_le_ab_plus_bc": check_ac,
            "bc_le_ab_plus_ac": check_bc,
            "ab_le_bc_plus_ac": check_ab
        },
        "perimeter": ab + bc + ac,
        "t1_grounding": "COMPARISON (κ: Triangle inequality d(a,c) ≤ d(a,b) + d(b,c))",
        "cross_domain": "Metric space validity / distance consistency / similarity measure"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Check if a point at test_distance is contained in the neighborhood of radius.
pub fn spatial_neighborhood(
    params: StemSpatialNeighborhoodParams,
) -> Result<CallToolResult, McpError> {
    let radius = params.radius.max(0.0);
    let contained = if params.open {
        params.test_distance < radius
    } else {
        params.test_distance <= radius
    };
    let topology = if params.open {
        "open ball B(x, r)"
    } else {
        "closed ball B̄(x, r)"
    };
    let boundary_type = if params.open {
        "strict (<)"
    } else {
        "inclusive (≤)"
    };

    let result = serde_json::json!({
        "radius": radius,
        "open": params.open,
        "topology": topology,
        "test_distance": params.test_distance,
        "contained": contained,
        "boundary_type": boundary_type,
        "t1_grounding": "BOUNDARY (∂: Neighborhood containment)",
        "cross_domain": "Tolerance range / confidence interval containment / cluster membership"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Report dimension rank and codimension relative to an ambient space.
pub fn spatial_dimension(params: StemSpatialDimensionParams) -> Result<CallToolResult, McpError> {
    let rank = params.rank;
    let codimension = params
        .subspace_of
        .map(|ambient| if ambient >= rank { ambient - rank } else { 0 });
    let is_subspace = params.subspace_of.is_some_and(|ambient| rank <= ambient);
    let classification = match rank {
        0 => "point (0-dimensional)",
        1 => "curve (1-dimensional)",
        2 => "surface (2-dimensional)",
        3 => "volume (3-dimensional)",
        _ => "hyperspace (n-dimensional)",
    };

    let result = serde_json::json!({
        "rank": rank,
        "subspace_of": params.subspace_of,
        "codimension": codimension,
        "is_subspace": is_subspace,
        "classification": classification,
        "t1_grounding": "QUANTITY (N: Rank as cardinal number of independent axes)",
        "cross_domain": "Feature dimensionality / parameter space / degrees of freedom"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Compose orientations (positive/negative/unoriented).
pub fn spatial_orientation(
    params: StemSpatialOrientationParams,
) -> Result<CallToolResult, McpError> {
    let parse_orient = |s: &str| -> i32 {
        match s.to_lowercase().as_str() {
            "positive" | "+" | "pos" => 1,
            "negative" | "-" | "neg" => -1,
            _ => 0,
        }
    };
    let orient_val = parse_orient(&params.orientation);
    let composed = params
        .compose_with
        .as_deref()
        .map(|other| orient_val * parse_orient(other));
    let composed_orientation = composed.map(|v| match v {
        1 => "positive",
        -1 => "negative",
        _ => "unoriented",
    });

    let result = serde_json::json!({
        "orientation": params.orientation,
        "value": orient_val,
        "compose_with": params.compose_with,
        "composed_value": composed,
        "composed_orientation": composed_orientation,
        "t1_grounding": "STATE (ς: Orientation as binary state ±1)",
        "cross_domain": "Signal polarity / direction / gradient sign"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Core Cross-Domain Tools
// ============================================================================

/// Compute cross-domain transfer confidence from structural, functional, and contextual similarity.
pub fn transfer_confidence(
    params: StemTransferConfidenceParams,
) -> Result<CallToolResult, McpError> {
    use stem::prelude::Confidence;

    let structural = Confidence::new(params.structural);
    let functional = Confidence::new(params.functional);
    let contextual = Confidence::new(params.contextual);
    let sf = structural.combine(functional);
    let combined = sf.combine(contextual);
    let classification = if combined.value() >= 0.7 {
        "high — transfer is reliable"
    } else if combined.value() >= 0.4 {
        "moderate — transfer with validation"
    } else {
        "low — requires domain expert review"
    };

    let result = serde_json::json!({
        "structural": structural.value(),
        "functional": functional.value(),
        "contextual": contextual.value(),
        "combined": combined.value(),
        "method": "multiplicative (a × b × c)",
        "classification": classification,
        "t1_grounding": "MAPPING (Extend trait: Source → Target domain)",
        "cross_domain": "Cross-domain knowledge transfer / analogy strength"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Check a value is within a min/max safety gate.
pub fn integrity_check(params: StemIntegrityCheckParams) -> Result<CallToolResult, McpError> {
    let passes = params.value >= params.min && params.value <= params.max;
    let label = params.label.as_deref().unwrap_or("value");
    let deviation = if params.value < params.min {
        params.min - params.value
    } else if params.value > params.max {
        params.value - params.max
    } else {
        0.0
    };

    let result = serde_json::json!({
        "label": label,
        "value": params.value,
        "min": params.min,
        "max": params.max,
        "passes": passes,
        "deviation_from_gate": deviation,
        "range": params.max - params.min,
        "t1_grounding": "BOUNDARY (∂: Value within safety gate)",
        "cross_domain": "Safety signal threshold / config validation / range assertion"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate remaining retry budget and exhaustion status.
pub fn retry_budget(params: StemRetryBudgetParams) -> Result<CallToolResult, McpError> {
    let remaining = params.max_attempts.saturating_sub(params.current_attempt);
    let exhausted = remaining == 0;
    let progress = if params.max_attempts > 0 {
        params.current_attempt as f64 / params.max_attempts as f64
    } else {
        1.0
    };

    let result = serde_json::json!({
        "max_attempts": params.max_attempts,
        "current_attempt": params.current_attempt,
        "remaining": remaining,
        "exhausted": exhausted,
        "progress_fraction": progress,
        "t1_grounding": "QUANTITY (N: Discrete countdown toward ∅ Void)",
        "cross_domain": "Exponential backoff budget / token allowance / rate limit"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Classify a repeatability score on the determinism spectrum.
pub fn determinism_score(params: StemDeterminismScoreParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Confidence;

    let score = Confidence::new(params.score);
    let classification = if score.value() >= 0.9 {
        "deterministic"
    } else if score.value() >= 0.7 {
        "mostly deterministic"
    } else if score.value() >= 0.3 {
        "stochastic"
    } else {
        "highly stochastic"
    };

    let result = serde_json::json!({
        "score": score.value(),
        "clamped_to": "[0.0, 1.0]",
        "classification": classification,
        "t1_grounding": "STATE (ς: Determinism as system state)",
        "cross_domain": "Test reproducibility / model stability / cache hit predictability"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Bio Tools (endocrine analog — computed without direct crate import)
// ============================================================================

/// Generate a behavioral modulation profile from a stimulus.
pub fn bio_behavior_profile(
    params: StemBioBehaviorProfileParams,
) -> Result<CallToolResult, McpError> {
    let intensity = params.intensity.unwrap_or(0.5).clamp(0.0, 1.0);
    let stimulus = params.stimulus.as_deref().unwrap_or("none");

    let (risk_tolerance, validation_depth, exploration_rate, warmth, urgency) = match stimulus {
        "stress" => (
            (0.5 - intensity * 0.4).max(0.0),
            (0.5 + intensity * 0.4).min(1.0),
            (0.5 - intensity * 0.3).max(0.0),
            (0.5 - intensity * 0.3).max(0.0),
            (0.5 + intensity * 0.4).min(1.0),
        ),
        "reward" => (
            (0.5 + intensity * 0.2).min(1.0),
            (0.5 - intensity * 0.1).max(0.0),
            (0.5 + intensity * 0.2).min(1.0),
            (0.5 + intensity * 0.4).min(1.0),
            (0.5 - intensity * 0.2).max(0.0),
        ),
        "social" => (
            0.5_f64,
            0.5_f64,
            (0.5 + intensity * 0.3).min(1.0),
            (0.5 + intensity * 0.4).min(1.0),
            0.4_f64,
        ),
        "urgency" => (
            (0.5 - intensity * 0.2).max(0.0),
            (0.5 - intensity * 0.3).max(0.0),
            (0.5 - intensity * 0.2).max(0.0),
            (0.5 - intensity * 0.1).max(0.0),
            (0.5 + intensity * 0.5).min(1.0),
        ),
        "temporal" => (
            0.5,
            0.5 * (1.0 - intensity * 0.1),
            0.5,
            0.5,
            0.5 * (1.0 - intensity * 0.2),
        ),
        _ => (0.5, 0.5, 0.5, 0.5, 0.3),
    };
    let dominant_trait = if urgency > 0.7 {
        "urgency"
    } else if warmth > 0.7 {
        "warmth"
    } else if validation_depth > 0.7 {
        "rigor"
    } else {
        "balanced"
    };

    let result = serde_json::json!({
        "stimulus": stimulus,
        "intensity": intensity,
        "behavior_modulation": {
            "risk_tolerance": risk_tolerance,
            "validation_depth": validation_depth,
            "exploration_rate": exploration_rate,
            "warmth": warmth,
            "urgency": urgency
        },
        "dominant_trait": dominant_trait,
        "t1_grounding": "STATE (ς: Endocrine behavioral state)",
        "cross_domain": "AI agent behavioral modulation / stress response / reward circuit"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Generate a communication tone profile from a stimulus.
pub fn bio_tone_profile(params: StemBioToneProfileParams) -> Result<CallToolResult, McpError> {
    let intensity = params.intensity.unwrap_or(0.5).clamp(0.0, 1.0);
    let stimulus = params.stimulus.as_deref().unwrap_or("none");

    let (directness, verbosity, warmth, precision, formality) = match stimulus {
        "stress" => (
            (0.85 + intensity * 0.10).min(1.0),
            (0.35 - intensity * 0.20).max(0.0),
            (0.75 - intensity * 0.30).max(0.0),
            0.95_f64,
            (0.55 - intensity * 0.20).max(0.0),
        ),
        "reward" => (
            0.80_f64,
            (0.40 + intensity * 0.15).min(1.0),
            (0.75 + intensity * 0.20).min(1.0),
            0.90_f64,
            (0.55 - intensity * 0.15).max(0.0),
        ),
        "social" => (
            (0.85 - intensity * 0.10).max(0.0),
            (0.40 + intensity * 0.10).min(1.0),
            (0.75 + intensity * 0.20).min(1.0),
            0.85_f64,
            (0.55 - intensity * 0.10).max(0.0),
        ),
        "urgency" => (
            (0.85 + intensity * 0.10).min(1.0),
            (0.35 - intensity * 0.25).max(0.0),
            (0.75 - intensity * 0.20).max(0.0),
            0.95_f64,
            (0.55 + intensity * 0.20).min(1.0),
        ),
        "temporal" => (0.85, 0.35, 0.75, 0.90, 0.60),
        _ => (0.85, 0.35, 0.75, 0.95, 0.55),
    };
    let tone_class = if directness > 0.9 && verbosity < 0.2 {
        "terse"
    } else if warmth > 0.9 {
        "warm"
    } else if precision > 0.9 && formality > 0.7 {
        "clinical"
    } else {
        "balanced"
    };

    let result = serde_json::json!({
        "stimulus": stimulus,
        "intensity": intensity,
        "tone_profile": {
            "directness": directness,
            "verbosity": verbosity,
            "warmth": warmth,
            "precision": precision,
            "formality": formality
        },
        "tone_class": tone_class,
        "t1_grounding": "STATE (ς: Communication tone as endocrine state)",
        "cross_domain": "AI persona tone calibration / patient communication style"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Finance Tools
// ============================================================================

/// Compute present value from a future value using the time value of money.
pub fn finance_discount(params: StemFinanceDiscountParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Discount, InterestRate, Price, TimeValueOfMoney};

    let tvm = TimeValueOfMoney;
    let fv = Price::new(params.future_value);
    let rate = InterestRate::new(params.rate).unwrap_or(InterestRate::ZERO);
    let pv = tvm.present_value(fv, rate, params.periods);
    let discount_factor = if params.future_value > 0.0 {
        pv.value() / params.future_value
    } else {
        0.0
    };

    let result = serde_json::json!({
        "future_value": fv.value(),
        "rate": rate.value(),
        "periods": params.periods,
        "present_value": pv.value(),
        "discount_factor": discount_factor,
        "formula": "PV = FV / (1 + r)^n",
        "t1_grounding": "CAUSALITY (→: Future value → Present value via time preference)",
        "cross_domain": "Signal urgency decay / data depreciation / cache staleness"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Compute compound growth (discrete or continuous).
pub fn finance_compound(params: StemFinanceCompoundParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Compound, InterestRate, Price, TimeValueOfMoney};

    let tvm = TimeValueOfMoney;
    let principal = Price::new(params.principal);
    let rate = InterestRate::new(params.rate).unwrap_or(InterestRate::ZERO);
    let final_value = if params.continuous {
        tvm.compound_continuous(principal, rate, params.periods as f64)
    } else {
        tvm.compound(principal, rate, params.periods)
    };
    let growth_factor = if params.principal > 0.0 {
        final_value.value() / params.principal
    } else {
        1.0
    };
    let formula = if params.continuous {
        "P × e^(r×t)"
    } else {
        "P × (1 + r)^n"
    };

    let result = serde_json::json!({
        "principal": principal.value(),
        "rate": rate.value(),
        "periods": params.periods,
        "continuous": params.continuous,
        "final_value": final_value.value(),
        "growth_factor": growth_factor,
        "formula": formula,
        "t1_grounding": "RECURSION (ρ: Growth applied to growth)",
        "cross_domain": "Tech debt accumulation / signal amplification / viral spread"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Compute the bid-ask spread and mid price.
pub fn finance_spread(params: StemFinanceSpreadParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Price, Spread};

    let bid = Price::new(params.bid);
    let ask = Price::new(params.ask);
    let spread = Spread::from_bid_ask(bid, ask);
    let mid = Price::mid(bid, ask);
    let spread_pct = spread.as_percent_of_mid(bid, ask);

    let result = serde_json::json!({
        "bid": bid.value(),
        "ask": ask.value(),
        "spread": spread.value(),
        "mid": mid.value(),
        "spread_percent_of_mid": spread_pct,
        "formula": "spread = ask - bid",
        "t1_grounding": "BOUNDARY (∂: Price gap between two levels)",
        "cross_domain": "Cross-source disagreement / signal calibration gap / detection variance"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Check maturity status of a time-bounded instrument.
pub fn finance_maturity(params: StemFinanceMaturityParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Maturity;

    let maturity = Maturity::new(params.years);

    let result = serde_json::json!({
        "years": maturity.years(),
        "months": maturity.years() * 12.0,
        "days": maturity.years() * 365.25,
        "is_expired": maturity.is_expired(),
        "t1_grounding": "IRREVERSIBILITY (∝: Countdown to maturity — one-way)",
        "cross_domain": "Regulatory deadline / session expiry / token timeout"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Report position exposure direction and magnitude.
pub fn finance_exposure(params: StemFinanceExposureParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Exposure;

    let exposure = Exposure::new(params.value);
    let direction = if exposure.is_long() {
        "long"
    } else if exposure.is_short() {
        "short"
    } else {
        "flat"
    };

    let result = serde_json::json!({
        "value": exposure.value(),
        "absolute": exposure.abs(),
        "direction": direction,
        "is_long": exposure.is_long(),
        "is_short": exposure.is_short(),
        "t1_grounding": "SUM (Σ: Aggregate position value)",
        "cross_domain": "Risk position / cumulative signal strength / net effect"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Detect arbitrage opportunity between two prices.
pub fn finance_arbitrage(params: StemFinanceArbitrageParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Price, Spread};

    let price_a = Price::new(params.price_a);
    let price_b = Price::new(params.price_b);
    let (bid, ask) = if price_a.value() <= price_b.value() {
        (price_a, price_b)
    } else {
        (price_b, price_a)
    };
    let spread = Spread::from_bid_ask(bid, ask);
    let cost = params.cost.unwrap_or(0.0);
    let net_profit = spread.value() - cost;
    let is_exploitable = spread.value() > cost;

    let result = serde_json::json!({
        "price_a": price_a.value(),
        "price_b": price_b.value(),
        "spread": spread.value(),
        "transaction_cost": cost,
        "net_profit": net_profit,
        "is_exploitable": is_exploitable,
        "t1_grounding": "COMPARISON (κ: Two prices for same value → exploit gap)",
        "cross_domain": "Cross-source signal discrepancy / calibration offset / A/B test divergence"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Compute diversification score across a portfolio of positions.
pub fn finance_diversify(params: StemFinanceDiversifyParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::Exposure;

    let exposures: Vec<Exposure> = params.positions.iter().map(|&v| Exposure::new(v)).collect();
    let net = exposures
        .iter()
        .copied()
        .fold(Exposure::ZERO, |acc, e| acc + e);
    let gross: f64 = exposures.iter().map(|e| e.abs()).sum();
    let hhi = if gross > 0.0 {
        exposures
            .iter()
            .map(|e| (e.abs() / gross).powi(2))
            .sum::<f64>()
    } else {
        1.0
    };
    let diversification_score = 1.0 - hhi;
    let max_position_pct = if gross > 0.0 {
        exposures
            .iter()
            .map(|e| e.abs() / gross * 100.0)
            .fold(0.0_f64, |acc, v| acc.max(v))
    } else {
        0.0
    };

    let result = serde_json::json!({
        "position_count": params.positions.len(),
        "net_exposure": net.value(),
        "gross_exposure": gross,
        "hhi_concentration": hhi,
        "diversification_score": diversification_score,
        "max_position_pct": max_position_pct,
        "t1_grounding": "SUM (Σ: Portfolio risk < sum of individual risks)",
        "cross_domain": "Multi-source signal confirmation / redundancy / multi-region deployment"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Compute simple or log return between two prices.
pub fn finance_return(params: StemFinanceReturnParams) -> Result<CallToolResult, McpError> {
    use stem::prelude::{Price, Return};

    let p0 = Price::new(params.p0);
    let p1 = Price::new(params.p1);
    let method = params.method.as_deref().unwrap_or("simple");
    let (ret_value, formula) = match method {
        "log" => (Return::log(p0, p1).map(|r| r.value()), "ln(P₁ / P₀)"),
        _ => (Return::simple(p0, p1).map(|r| r.value()), "(P₁ - P₀) / P₀"),
    };
    let direction = match ret_value {
        Some(v) if v > 0.0 => "gain",
        Some(v) if v < 0.0 => "loss",
        Some(_) => "flat",
        None => "undefined",
    };

    let result = serde_json::json!({
        "p0": p0.value(),
        "p1": p1.value(),
        "method": method,
        "return": ret_value,
        "is_positive": ret_value.is_some_and(|v| v > 0.0),
        "direction": direction,
        "formula": formula,
        "t1_grounding": "MAPPING (μ: Price change → Proportion)",
        "cross_domain": "Signal change rate / performance delta / version improvement"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Statistics Tools (stem-math statistical inference)
// ============================================================================

fn parse_tail(s: Option<&str>) -> stem_math::statistics::Tail {
    match s {
        Some("left") => stem_math::statistics::Tail::Left,
        Some("right") => stem_math::statistics::Tail::Right,
        _ => stem_math::statistics::Tail::Two,
    }
}

/// One-sample z-test: compute z-score, p-value, CI, and significance.
pub fn stats_z_test(params: StemStatsZTestParams) -> Result<CallToolResult, McpError> {
    use stem_math::statistics::{StatisticalOutcome, Tail};

    let tail = parse_tail(params.tail.as_deref());
    let confidence = params.confidence_level.unwrap_or(0.95);
    let tail_str = match tail {
        Tail::Left => "left",
        Tail::Right => "right",
        Tail::Two => "two",
    };

    let outcome = StatisticalOutcome::new(
        params.observed,
        params.null_value,
        params.std_error,
        confidence,
        tail,
        format!(
            "z-test: observed={} vs null={}",
            params.observed, params.null_value
        ),
    );

    match outcome {
        Some(o) => {
            let result = serde_json::json!({
                "value": o.value,
                "z_score": o.z_score,
                "p_value": o.p_value,
                "significance": format!("{}", o.significance),
                "significant": o.is_significant(),
                "stars": o.significance.stars(),
                "ci": {
                    "estimate": o.ci.estimate,
                    "lower": o.ci.lower,
                    "upper": o.ci.upper,
                    "level": o.ci.level,
                    "margin": o.ci.margin,
                    "z_critical": o.ci.z_critical,
                },
                "ci_excludes_null": o.ci_excludes_null(params.null_value),
                "tail": tail_str,
                "interpretation": format!("{o}"),
                "t1_grounding": "COMPARISON (κ) + QUANTITY (N) + BOUNDARY (∂) + STATE (ς) + MAPPING (μ)",
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        None => {
            let err = serde_json::json!({
                "error": "Invalid parameters: std_error must be > 0, confidence_level must be in (0, 1)",
                "observed": params.observed,
                "null_value": params.null_value,
                "std_error": params.std_error,
            });
            Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&err).unwrap_or_default(),
            )]))
        }
    }
}

/// Construct a confidence interval (mean, proportion, or difference).
pub fn stats_ci(params: StemStatsCiParams) -> Result<CallToolResult, McpError> {
    use stem_math::statistics::{
        confidence_interval_diff, confidence_interval_mean, confidence_interval_proportion,
    };

    let level = params.confidence_level.unwrap_or(0.95);
    let ci_type = params.ci_type.as_deref().unwrap_or("mean");

    let ci = match ci_type {
        "proportion" => {
            let n = params.n.unwrap_or(100);
            confidence_interval_proportion(params.estimate, n, level)
        }
        "diff" => {
            let mean2 = params.mean2.unwrap_or(0.0);
            let se2 = params.se2.unwrap_or(params.std_error);
            confidence_interval_diff(params.estimate, params.std_error, mean2, se2, level)
        }
        _ => confidence_interval_mean(params.estimate, params.std_error, level),
    };

    match ci {
        Some(c) => {
            let result = serde_json::json!({
                "estimate": c.estimate,
                "lower": c.lower,
                "upper": c.upper,
                "level": c.level,
                "margin": c.margin,
                "width": c.width(),
                "z_critical": c.z_critical,
                "std_error": c.std_error,
                "ci_type": ci_type,
                "interpretation": format!(
                    "{}% CI: [{:.6}, {:.6}]",
                    c.level * 100.0, c.lower, c.upper
                ),
                "t1_grounding": "BOUNDARY (∂) + QUANTITY (N)",
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        None => {
            let err = serde_json::json!({
                "error": "Invalid parameters: check std_error > 0 and confidence_level in (0, 1)",
                "estimate": params.estimate,
                "std_error": params.std_error,
                "level": level,
            });
            Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&err).unwrap_or_default(),
            )]))
        }
    }
}

/// Compute p-value from z-score or from observed/null/SE.
pub fn stats_p_value(params: StemStatsPValueParams) -> Result<CallToolResult, McpError> {
    use stem_math::statistics::{Significance, p_value, p_value_from_z, z_score};

    let tail = parse_tail(params.tail.as_deref());
    let tail_str = match tail {
        stem_math::statistics::Tail::Left => "left",
        stem_math::statistics::Tail::Right => "right",
        stem_math::statistics::Tail::Two => "two",
    };

    // Either use provided z-score or compute from observed/null/SE
    let (z, p) = if let Some(z_val) = params.z_score {
        let p_val = p_value_from_z(z_val, tail);
        (z_val, p_val)
    } else if let (Some(obs), Some(null), Some(se)) =
        (params.observed, params.null_value, params.std_error)
    {
        match (z_score(obs, null, se), p_value(obs, null, se, tail)) {
            (Some(z_val), Some(p_val)) => (z_val, p_val),
            _ => {
                let err = serde_json::json!({
                    "error": "Could not compute z-score: std_error must be > 0",
                });
                return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                    serde_json::to_string_pretty(&err).unwrap_or_default(),
                )]));
            }
        }
    } else {
        let err = serde_json::json!({
            "error": "Provide either z_score OR (observed + null_value + std_error)",
        });
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&err).unwrap_or_default(),
        )]));
    };

    let sig = Significance::from_p(p);

    let result = serde_json::json!({
        "z_score": z,
        "p_value": p,
        "significance": format!("{sig}"),
        "significant": sig.is_significant(),
        "stars": sig.stars(),
        "alpha": sig.alpha(),
        "tail": tail_str,
        "t1_grounding": "COMPARISON (κ) + STATE (ς)",
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Analyze a sample: compute descriptive stats, z-test, CI, and significance.
pub fn stats_analyze(params: StemStatsAnalyzeParams) -> Result<CallToolResult, McpError> {
    use stem_math::statistics::{
        Significance, StatisticalOutcome, Tail, confidence_interval_mean, mean, standard_error,
        variance,
    };

    if params.values.is_empty() {
        let err = serde_json::json!({ "error": "Empty sample" });
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&err).unwrap_or_default(),
        )]));
    }

    let n = params.values.len();
    let null_val = params.null_value.unwrap_or(0.0);
    let confidence = params.confidence_level.unwrap_or(0.95);

    let sample_mean = mean(&params.values).unwrap_or(0.0);
    let sample_var = variance(&params.values);
    let sample_se = standard_error(&params.values);

    let outcome = sample_se.and_then(|se| {
        StatisticalOutcome::new(
            sample_mean,
            null_val,
            se,
            confidence,
            Tail::Two,
            "sample z-test",
        )
    });

    let ci = sample_se.and_then(|se| confidence_interval_mean(sample_mean, se, confidence));

    let result = serde_json::json!({
        "n": n,
        "mean": sample_mean,
        "variance": sample_var,
        "std_dev": sample_var.map(|v| v.sqrt()),
        "std_error": sample_se,
        "null_value": null_val,
        "z_test": outcome.as_ref().map(|o| serde_json::json!({
            "z_score": o.z_score,
            "p_value": o.p_value,
            "significance": format!("{}", o.significance),
            "significant": o.is_significant(),
            "stars": o.significance.stars(),
        })),
        "ci": ci.map(|c| serde_json::json!({
            "estimate": c.estimate,
            "lower": c.lower,
            "upper": c.upper,
            "level": c.level,
            "margin": c.margin,
        })),
        "confidence_level": confidence,
        "t1_grounding": "COMPARISON (κ) + QUANTITY (N) + BOUNDARY (∂) + STATE (ς) + MAPPING (μ)",
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
