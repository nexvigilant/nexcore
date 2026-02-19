//! Declension Classes (∂-Boundary Partitioning)
//!
//! Latin partitions its entire lexicon into 5 declension classes.
//! Each class has its own inflection rules. A noun belongs to exactly one.
//!
//! We partition NexCore crates into 5 declension classes, each with
//! architectural constraints (what deps are allowed, what traits required).
//!
//! ## Primitive Grounding
//! ∂ Boundary (dominant) + σ Sequence + κ Comparison = T2-C

use serde::{Deserialize, Serialize};

/// Tier: T2-C — ∂ Boundary + σ Sequence + κ Comparison
///
/// The 5 architectural declension classes of the NexCore workspace.
/// Each class determines what a crate can depend on, what traits it must
/// implement, and how it exposes its API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Declension {
    /// **First Declension (-a)**: Foundation layer.
    /// Pure types, no side effects, no async.
    /// Latin analog: *puella, puellae* — feminine, regular, foundational.
    /// Examples: nexcore-primitives, nexcore-lex-primitiva, nexcore-constants.
    ///
    /// **Rules:**
    /// - No dependencies on higher declensions
    /// - No async, no IO
    /// - All types must implement GroundsTo
    First,

    /// **Second Declension (-us)**: Domain layer.
    /// Business logic, state machines, domain types.
    /// Latin analog: *dominus, domini* — masculine, authoritative, structural.
    /// Examples: nexcore-vigilance, nexcore-pvos, signal.
    ///
    /// **Rules:**
    /// - May depend on First declension only
    /// - State machines allowed (ς-modal)
    /// - GroundsTo required for all public engine types
    Second,

    /// **Third Declension (-is)**: Orchestration layer.
    /// Async, event-driven, cross-crate coordination.
    /// Latin analog: *rex, regis* — irregular, powerful, complex.
    /// Examples: nexcore-brain, nexcore-friday, nexcore-orchestration.
    ///
    /// **Rules:**
    /// - May depend on First and Second
    /// - Async permitted
    /// - Must implement lifecycle traits (start/stop/health)
    Third,

    /// **Fourth Declension (-us)**: Service layer.
    /// IO boundaries, external-facing APIs, tool interfaces.
    /// Latin analog: *manus, manus* — action-oriented, instrumental.
    /// Examples: nexcore-mcp, nexcore-api, nexcore-cli.
    ///
    /// **Rules:**
    /// - May depend on any lower declension
    /// - IO and network permitted
    /// - μ-inflected endpoints (tool families, not individual tools)
    Fourth,

    /// **Fifth Declension (-ei)**: Infrastructure layer.
    /// System-level, security boundaries, deployment.
    /// Latin analog: *res, rei* — abstract, foundational to governance.
    /// Examples: nexcore-sentinel, nexcore-hooks, nexcore-config.
    ///
    /// **Rules:**
    /// - May depend on First only (minimal coupling)
    /// - ∂-enforced security boundaries
    /// - Must handle failure gracefully (no panics)
    Fifth,
}

impl Declension {
    /// Latin stem vowel for this declension.
    pub fn stem_vowel(&self) -> &'static str {
        match self {
            Self::First => "-a",
            Self::Second => "-us",
            Self::Third => "-is",
            Self::Fourth => "-us (4th)",
            Self::Fifth => "-ei",
        }
    }

    /// Layer name in the NexCore architecture.
    pub fn layer(&self) -> &'static str {
        match self {
            Self::First => "foundation",
            Self::Second => "domain",
            Self::Third => "orchestration",
            Self::Fourth => "service",
            Self::Fifth => "infrastructure",
        }
    }

    /// Which declensions this one may depend on.
    pub fn allowed_deps(&self) -> &'static [Declension] {
        match self {
            Self::First => &[],
            Self::Second => &[Declension::First],
            Self::Third => &[Declension::First, Declension::Second],
            Self::Fourth => &[Declension::First, Declension::Second, Declension::Third],
            Self::Fifth => &[Declension::First],
        }
    }

    /// Whether async is permitted in this declension.
    pub fn async_permitted(&self) -> bool {
        matches!(self, Self::Third | Self::Fourth)
    }

    /// Whether IO/network is permitted in this declension.
    pub fn io_permitted(&self) -> bool {
        matches!(self, Self::Fourth | Self::Fifth)
    }

    /// Whether GroundsTo is required for public types.
    pub fn grounds_to_required(&self) -> bool {
        matches!(self, Self::First | Self::Second)
    }

    /// All 5 declensions in order.
    pub fn all() -> &'static [Declension] {
        &[
            Self::First,
            Self::Second,
            Self::Third,
            Self::Fourth,
            Self::Fifth,
        ]
    }
}

/// A crate classified into its declension.
///
/// Tier: T2-C — ∂ + σ + κ + λ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclinedCrate {
    /// Crate name.
    pub name: String,
    /// Assigned declension class.
    pub declension: Declension,
    /// Layer name.
    pub layer: String,
    /// Dependency violations (crates in disallowed declensions).
    pub violations: Vec<String>,
}

/// Check if a dependency from `from` declension to `to` declension is valid.
pub fn is_valid_dep(from: Declension, to: Declension) -> bool {
    if from == to {
        return true; // Same-declension deps are always valid
    }
    from.allowed_deps().contains(&to)
}

/// Classify a NexCore crate into its declension based on name.
pub fn classify_crate(crate_name: &str) -> Declension {
    // First declension — foundation types
    if crate_name.contains("primitives")
        || crate_name.contains("lex-primitiva")
        || crate_name.contains("constants")
        || crate_name.starts_with("stem-")
        || crate_name.contains("aggregate")
        || crate_name.contains("edit-distance")
        || crate_name.contains("declension")
        || crate_name.contains("caesura")
    {
        return Declension::First;
    }

    // Second declension — domain logic
    if crate_name.contains("vigilance")
        || crate_name.contains("pvos")
        || crate_name.contains("signal")
        || crate_name.contains("dtree")
        || crate_name.contains("energy")
        || crate_name.contains("ccp")
        || crate_name.contains("trust")
        || crate_name.contains("harm")
        || crate_name.contains("pv-core")
        || crate_name.contains("algovigilance")
        || crate_name.contains("jeopardy")
        || crate_name.contains("transcriptase")
        || crate_name.contains("ribosome")
        || crate_name.contains("labs")
    {
        return Declension::Second;
    }

    // Third declension — orchestration
    if crate_name.contains("brain")
        || crate_name.contains("friday")
        || crate_name.contains("orchestration")
        || crate_name.contains("cytokine")
        || crate_name.contains("hormones")
        || crate_name.contains("synapse")
    {
        return Declension::Third;
    }

    // Fourth declension — service
    if crate_name.contains("mcp")
        || crate_name.contains("api")
        || crate_name.contains("cli")
        || crate_name.contains("repl")
    {
        return Declension::Fourth;
    }

    // Fifth declension — infrastructure
    if crate_name.contains("sentinel")
        || crate_name.contains("hook")
        || crate_name.contains("config")
        || crate_name.contains("vault")
        || crate_name.contains("cloud")
        || crate_name.contains("network")
        || crate_name.contains("immunity")
    {
        return Declension::Fifth;
    }

    // Default: Second (most domain crates)
    Declension::Second
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declension_order() {
        assert_eq!(Declension::all().len(), 5);
    }

    #[test]
    fn test_first_no_deps() {
        assert!(Declension::First.allowed_deps().is_empty());
    }

    #[test]
    fn test_second_deps_first_only() {
        let deps = Declension::Second.allowed_deps();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], Declension::First);
    }

    #[test]
    fn test_fourth_deps_all_lower() {
        let deps = Declension::Fourth.allowed_deps();
        assert_eq!(deps.len(), 3);
    }

    #[test]
    fn test_fifth_deps_first_only() {
        let deps = Declension::Fifth.allowed_deps();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], Declension::First);
    }

    #[test]
    fn test_valid_dep() {
        assert!(is_valid_dep(Declension::Second, Declension::First));
        assert!(is_valid_dep(Declension::First, Declension::First));
    }

    #[test]
    fn test_invalid_dep() {
        assert!(!is_valid_dep(Declension::First, Declension::Second));
        assert!(!is_valid_dep(Declension::Fifth, Declension::Third));
    }

    #[test]
    fn test_async_permissions() {
        assert!(!Declension::First.async_permitted());
        assert!(!Declension::Second.async_permitted());
        assert!(Declension::Third.async_permitted());
        assert!(Declension::Fourth.async_permitted());
        assert!(!Declension::Fifth.async_permitted());
    }

    #[test]
    fn test_grounds_to_required() {
        assert!(Declension::First.grounds_to_required());
        assert!(Declension::Second.grounds_to_required());
        assert!(!Declension::Third.grounds_to_required());
    }

    #[test]
    fn test_classify_foundation() {
        assert_eq!(classify_crate("nexcore-primitives"), Declension::First);
        assert_eq!(classify_crate("nexcore-lex-primitiva"), Declension::First);
        assert_eq!(classify_crate("stem-math"), Declension::First);
    }

    #[test]
    fn test_classify_domain() {
        assert_eq!(classify_crate("nexcore-vigilance"), Declension::Second);
        assert_eq!(classify_crate("nexcore-pvos"), Declension::Second);
    }

    #[test]
    fn test_classify_orchestration() {
        assert_eq!(classify_crate("nexcore-brain"), Declension::Third);
        assert_eq!(classify_crate("nexcore-friday"), Declension::Third);
    }

    #[test]
    fn test_classify_service() {
        assert_eq!(classify_crate("nexcore-mcp"), Declension::Fourth);
        assert_eq!(classify_crate("nexcore-api"), Declension::Fourth);
    }

    #[test]
    fn test_classify_infrastructure() {
        assert_eq!(classify_crate("nexcore-sentinel"), Declension::Fifth);
        assert_eq!(classify_crate("nexcore-vault"), Declension::Fifth);
    }

    #[test]
    fn test_layer_names() {
        assert_eq!(Declension::First.layer(), "foundation");
        assert_eq!(Declension::Fourth.layer(), "service");
    }
}
