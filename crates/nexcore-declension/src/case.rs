//! Component Case System (ς-First Architecture)
//!
//! Latin encodes a noun's role via case inflection, not position.
//! We encode a component's architectural role via `ComponentCase`,
//! making orchestration order-independent.
//!
//! ## Primitive Grounding
//! ς State (dominant) + ∂ Boundary + μ Mapping = T2-C

use serde::{Deserialize, Serialize};

/// Tier: T2-C — ς State + ∂ Boundary + μ Mapping
///
/// The grammatical case of a system component, encoding its architectural
/// role independent of invocation order.
///
/// Like Latin's 7 cases, each case declares *what the component does*
/// in the system sentence, not *where it sits*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentCase {
    /// **Nominative** — The actor. Triggers actions, initiates workflows.
    /// Latin: *puella currit* (the girl runs) — puella is nominative.
    /// System: Event sources, schedulers, CLI entry points.
    Nominative,

    /// **Accusative** — The target. Receives actions, is transformed.
    /// Latin: *videt puerum* (sees the boy) — puerum is accusative.
    /// System: Data stores, state containers, mutation targets.
    Accusative,

    /// **Genitive** — The owner. Provides resources, holds authority.
    /// Latin: *liber puellae* (the girl's book) — puellae is genitive.
    /// System: Config providers, credential vaults, resource pools.
    Genitive,

    /// **Dative** — The beneficiary. Receives results, consumes output.
    /// Latin: *dat puellae librum* (gives the book to the girl) — puellae is dative.
    /// System: Loggers, metrics collectors, output sinks, subscribers.
    Dative,

    /// **Ablative** — The instrument. Provides context, enables action.
    /// Latin: *cum gladio* (with a sword) — gladio is ablative.
    /// System: Middleware, context providers, auth layers, feature flags.
    Ablative,

    /// **Vocative** — The addressed. Directly invoked by name.
    /// Latin: *Marce!* (Marcus!) — direct address.
    /// System: CLI commands, MCP tools, API endpoints — things you call by name.
    Vocative,

    /// **Locative** — The location. Specifies where something operates.
    /// Latin: *Romae* (at Rome) — locative.
    /// System: Service discovery, routing tables, deployment targets.
    Locative,
}

impl ComponentCase {
    /// Latin label for the case.
    pub fn latin(&self) -> &'static str {
        match self {
            Self::Nominative => "nominativus",
            Self::Accusative => "accusativus",
            Self::Genitive => "genitivus",
            Self::Dative => "dativus",
            Self::Ablative => "ablativus",
            Self::Vocative => "vocativus",
            Self::Locative => "locativus",
        }
    }

    /// System role description.
    pub fn role(&self) -> &'static str {
        match self {
            Self::Nominative => "actor (triggers actions)",
            Self::Accusative => "target (receives actions)",
            Self::Genitive => "owner (provides resources)",
            Self::Dative => "beneficiary (receives results)",
            Self::Ablative => "instrument (provides context)",
            Self::Vocative => "addressed (invoked by name)",
            Self::Locative => "location (where it operates)",
        }
    }

    /// Whether this case implies the component acts (vs. is acted upon).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Nominative | Self::Vocative)
    }

    /// Whether this case implies the component receives.
    pub fn is_passive(&self) -> bool {
        matches!(self, Self::Accusative | Self::Dative)
    }

    /// Whether this case provides something to the sentence.
    pub fn is_provider(&self) -> bool {
        matches!(self, Self::Genitive | Self::Ablative | Self::Locative)
    }

    /// All 7 cases in traditional Latin order.
    pub fn all() -> &'static [ComponentCase] {
        &[
            Self::Nominative,
            Self::Accusative,
            Self::Genitive,
            Self::Dative,
            Self::Ablative,
            Self::Vocative,
            Self::Locative,
        ]
    }
}

/// A component with its case assignment.
///
/// Tier: T2-C — ς + ∂ + μ + λ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CasedComponent {
    /// Component name (e.g., crate name, tool name, service name).
    pub name: String,
    /// Assigned grammatical case.
    pub case: ComponentCase,
    /// Optional description of why this case was assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// Classify a NexCore crate into its natural case based on naming conventions
/// and known architectural roles.
pub fn classify_crate(crate_name: &str) -> ComponentCase {
    // Nominative — actors that trigger
    if crate_name.contains("cli")
        || crate_name.contains("scheduler")
        || crate_name.contains("friday")
        || crate_name.contains("vigil")
    {
        return ComponentCase::Nominative;
    }

    // Accusative — targets that are transformed
    if crate_name.contains("brain")
        || crate_name.contains("state")
        || crate_name.contains("pvos")
        || crate_name.contains("signal")
    {
        return ComponentCase::Accusative;
    }

    // Genitive — resource owners
    if crate_name.contains("config")
        || crate_name.contains("vault")
        || crate_name.contains("constants")
        || crate_name.contains("primitives")
    {
        return ComponentCase::Genitive;
    }

    // Dative — result receivers
    if crate_name.contains("measure")
        || crate_name.contains("telemetry")
        || crate_name.contains("insight")
        || crate_name.contains("phenotype")
    {
        return ComponentCase::Dative;
    }

    // Ablative — context/instrument providers
    if crate_name.contains("hook")
        || crate_name.contains("immunity")
        || crate_name.contains("clearance")
        || crate_name.contains("guardian")
        || crate_name.contains("sentinel")
    {
        return ComponentCase::Ablative;
    }

    // Vocative — directly addressed
    if crate_name.contains("mcp") || crate_name.contains("api") || crate_name.contains("repl") {
        return ComponentCase::Vocative;
    }

    // Locative — location/deployment
    if crate_name.contains("cloud") || crate_name.contains("network") || crate_name.contains("mesh")
    {
        return ComponentCase::Locative;
    }

    // Default: Accusative (most crates are domain objects acted upon)
    ComponentCase::Accusative
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_properties() {
        assert!(ComponentCase::Nominative.is_active());
        assert!(ComponentCase::Accusative.is_passive());
        assert!(ComponentCase::Genitive.is_provider());
        assert!(ComponentCase::Ablative.is_provider());
        assert!(!ComponentCase::Dative.is_active());
    }

    #[test]
    fn test_all_cases() {
        assert_eq!(ComponentCase::all().len(), 7);
    }

    #[test]
    fn test_classify_crate_mcp() {
        assert_eq!(classify_crate("nexcore-mcp"), ComponentCase::Vocative);
    }

    #[test]
    fn test_classify_crate_guardian() {
        assert_eq!(
            classify_crate("nexcore-guardian-engine"),
            ComponentCase::Ablative
        );
    }

    #[test]
    fn test_classify_crate_brain() {
        assert_eq!(classify_crate("nexcore-brain"), ComponentCase::Accusative);
    }

    #[test]
    fn test_classify_crate_config() {
        assert_eq!(classify_crate("nexcore-config"), ComponentCase::Genitive);
    }

    #[test]
    fn test_classify_crate_cloud() {
        assert_eq!(classify_crate("nexcore-cloud"), ComponentCase::Locative);
    }

    #[test]
    fn test_classify_crate_measure() {
        assert_eq!(classify_crate("nexcore-measure"), ComponentCase::Dative);
    }

    #[test]
    fn test_classify_crate_friday() {
        assert_eq!(classify_crate("nexcore-friday"), ComponentCase::Nominative);
    }

    #[test]
    fn test_latin_labels() {
        assert_eq!(ComponentCase::Nominative.latin(), "nominativus");
        assert_eq!(ComponentCase::Ablative.latin(), "ablativus");
    }

    #[test]
    fn test_cased_component_serde() {
        let comp = CasedComponent {
            name: "nexcore-mcp".to_string(),
            case: ComponentCase::Vocative,
            rationale: Some("Directly invoked by Claude Code".to_string()),
        };
        let json = serde_json::to_string(&comp);
        assert!(json.is_ok());
    }
}
