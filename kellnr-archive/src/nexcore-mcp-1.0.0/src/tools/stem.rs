//! STEM primitives MCP tools
//!
//! Exposes cross-domain T2-P primitives from the STEM crate system
//! (Science, Technology, Engineering, Mathematics) via MCP.
//!
//! 32 traits across 4 domains, all grounded to T1 primitives.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    StemChemBalanceParams, StemChemFractionParams, StemConfidenceCombineParams,
    StemMathBoundsCheckParams, StemMathRelationInvertParams, StemPhysConservationParams,
    StemPhysFmaParams, StemPhysPeriodParams, StemTierInfoParams,
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
