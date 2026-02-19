//! Domain profiles for text transformation.
//!
//! A `DomainProfile` captures the vocabulary, conceptual bridges,
//! and rhetorical patterns of a target domain. Five builtins ship
//! with the crate; custom profiles can be registered at runtime.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A conceptual bridge: a source-agnostic term mapped to a target domain term.
///
/// Bridges are the deterministic backbone of domain translation.
/// Each bridge pairs a generic concept (e.g., "governance") with its
/// domain-specific realization (e.g., "signal committee oversight").
///
/// When `source_domain` is `None`, the bridge is universal (matches any source).
/// When set, the bridge only fires for that specific source domain,
/// enabling source-aware translation (e.g., biochemistry→military vs
/// political-philosophy→military produce different mappings).
///
/// Tier: T2-P | Dominant: mu (Mapping)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConceptBridge {
    /// The generic concept name (lowercase, domain-agnostic).
    pub generic: String,
    /// The domain-specific term this concept maps to.
    pub specific: String,
    /// Confidence in the mapping (0.0..=1.0).
    pub confidence: f64,
    /// Optional source domain qualifier. `None` = universal bridge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_domain: Option<String>,
}

/// Rhetorical role classification for paragraphs.
///
/// Each role captures a structural function in argumentation,
/// enabling parallel structure preservation during transformation.
///
/// Tier: T2-P | Dominant: kappa (Comparison)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RhetoricalRole {
    /// Sets scope and stakes.
    Introduction,
    /// Presents the central claim.
    Thesis,
    /// Provides evidence or reasoning.
    Argument,
    /// Addresses opposing views.
    Counterargument,
    /// Refutes the counterargument.
    Rebuttal,
    /// Offers a concrete case.
    Example,
    /// Moves between sections.
    Transition,
    /// Summarizes and calls to action.
    Conclusion,
    /// No clear rhetorical function.
    Expository,
}

impl RhetoricalRole {
    /// Returns a guidance string for rewriting paragraphs of this role.
    pub fn rewrite_guidance(&self) -> &'static str {
        match self {
            Self::Introduction => {
                "Establish the domain context and stakes. Mirror the original's scope."
            }
            Self::Thesis => {
                "State the central claim in domain terms. Preserve argumentative force."
            }
            Self::Argument => "Present domain-specific evidence. Maintain logical structure.",
            Self::Counterargument => "Voice domain-relevant objections. Keep adversarial tension.",
            Self::Rebuttal => "Counter with domain evidence. Preserve rhetorical momentum.",
            Self::Example => "Substitute domain-specific examples. Keep illustrative function.",
            Self::Transition => "Bridge sections using domain vocabulary. Maintain flow.",
            Self::Conclusion => "Synthesize in domain terms. Mirror the call to action.",
            Self::Expository => "Rewrite with domain vocabulary. Preserve informational content.",
        }
    }
}

/// A domain profile capturing vocabulary, bridges, and patterns.
///
/// Tier: T2-C | Dominant: mu (Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProfile {
    /// Unique name (lowercase kebab-case).
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Short description of the domain.
    pub description: String,
    /// Core vocabulary terms (domain-specific keywords).
    pub vocabulary: Vec<String>,
    /// Conceptual bridges from generic concepts to domain terms.
    pub bridges: Vec<ConceptBridge>,
    /// Typical rhetorical patterns in this domain's literature.
    pub rhetorical_notes: String,
}

impl DomainProfile {
    /// Look up a bridge by generic concept name (case-insensitive).
    ///
    /// Two-pass lookup when `source_domain` is provided:
    ///   1. Source-specific match (bridge.source_domain == source_domain)
    ///   2. Universal fallback (bridge.source_domain is None)
    ///
    /// When `source_domain` is `None`, only universal bridges match.
    pub fn find_bridge(
        &self,
        generic: &str,
        source_domain: Option<&str>,
    ) -> Option<&ConceptBridge> {
        let lower = generic.to_lowercase();
        let source_lower = source_domain.map(|s| s.to_lowercase());

        // Pass 1: source-specific match
        if let Some(ref src) = source_lower {
            if let Some(b) = self.bridges.iter().find(|b| {
                b.generic == lower
                    && b.source_domain
                        .as_ref()
                        .is_some_and(|sd| sd.eq_ignore_ascii_case(src))
            }) {
                return Some(b);
            }
        }

        // Pass 2: universal fallback
        self.bridges
            .iter()
            .find(|b| b.generic == lower && b.source_domain.is_none())
    }

    /// Returns all vocabulary terms as a set for fast lookup.
    pub fn vocabulary_set(&self) -> std::collections::HashSet<String> {
        self.vocabulary.iter().map(|v| v.to_lowercase()).collect()
    }

    /// Returns sorted list of source domains that have specialized bridges.
    ///
    /// Useful for discovering which `source_domain` values unlock extra
    /// mappings beyond the universal bridge set.
    pub fn known_source_domains(&self) -> Vec<String> {
        let mut domains: Vec<String> = self
            .bridges
            .iter()
            .filter_map(|b| b.source_domain.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        domains.sort();
        domains
    }
}

/// Registry of available domain profiles.
///
/// Ships with 5 builtins and supports runtime registration of custom profiles.
///
/// Tier: T2-C | Dominant: sigma (Sequence)
#[derive(Debug, Clone)]
pub struct DomainProfileRegistry {
    profiles: HashMap<String, DomainProfile>,
}

impl DomainProfileRegistry {
    /// Create a registry pre-loaded with 5 builtin profiles.
    pub fn new() -> Self {
        let mut registry = Self {
            profiles: HashMap::new(),
        };
        registry.register(builtin_pharmacovigilance());
        registry.register(builtin_software_architecture());
        registry.register(builtin_organizational_design());
        registry.register(builtin_clinical_trials());
        registry.register(builtin_military_defense());
        registry
    }

    /// Create an empty registry (no builtins).
    pub fn empty() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    /// Register a profile. Overwrites if name already exists.
    pub fn register(&mut self, profile: DomainProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    /// Get a profile by name (case-insensitive kebab lookup).
    pub fn get(&self, name: &str) -> Option<&DomainProfile> {
        let key = name.to_lowercase().replace(' ', "-");
        self.profiles.get(&key)
    }

    /// List all registered profile names.
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.profiles.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Number of registered profiles.
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

impl Default for DomainProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BUILTIN PROFILES
// ═══════════════════════════════════════════════════════════════════════════

/// Create a universal bridge (no source domain qualifier).
fn bridge(generic: &str, specific: &str, confidence: f64) -> ConceptBridge {
    ConceptBridge {
        generic: generic.to_lowercase(),
        specific: specific.to_string(),
        confidence,
        source_domain: None,
    }
}

/// Create a source-domain-specific bridge.
///
/// Public so external consumers can build custom profiles with
/// source-aware bridges.
pub fn bridge_for(generic: &str, specific: &str, confidence: f64, source: &str) -> ConceptBridge {
    ConceptBridge {
        generic: generic.to_lowercase(),
        specific: specific.to_string(),
        confidence,
        source_domain: Some(source.to_lowercase()),
    }
}

/// Profile 1: Pharmacovigilance
pub fn builtin_pharmacovigilance() -> DomainProfile {
    DomainProfile {
        name: "pharmacovigilance".to_string(),
        display_name: "Pharmacovigilance".to_string(),
        description: "Drug safety surveillance, signal detection, and adverse event monitoring"
            .to_string(),
        vocabulary: vec![
            "signal",
            "adverse event",
            "ICSR",
            "causality",
            "disproportionality",
            "PSUR",
            "PBRER",
            "ICH E2E",
            "MedDRA",
            "safety database",
            "benefit-risk",
            "risk management plan",
            "pharmacovigilance system",
            "qualified person",
            "expedited reporting",
            "periodic report",
            "signal detection",
            "data mining",
            "case processing",
            "serious adverse event",
            "unexpected adverse reaction",
            "individual case safety report",
            "aggregate analysis",
            "risk minimization",
            "post-marketing surveillance",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        bridges: vec![
            bridge("governance", "pharmacovigilance system master file", 0.85),
            bridge("constitution", "pharmacovigilance system", 0.90),
            bridge("citizen", "patient", 0.95),
            bridge("law", "regulatory guideline", 0.88),
            bridge("republic", "safety monitoring system", 0.82),
            bridge("liberty", "patient safety", 0.90),
            bridge("faction", "confounding signal", 0.78),
            bridge("tyranny", "undetected safety signal", 0.80),
            bridge("democracy", "transparent reporting", 0.75),
            bridge("election", "signal prioritization", 0.72),
            bridge("legislature", "signal committee", 0.80),
            bridge("judiciary", "causality assessment", 0.85),
            bridge("executive", "risk management", 0.83),
            bridge("union", "global pharmacovigilance network", 0.88),
            bridge("state", "marketing authorization holder", 0.70),
            bridge("people", "patients and healthcare professionals", 0.92),
            bridge("power", "regulatory authority", 0.85),
            bridge("danger", "safety signal", 0.90),
            bridge("prosperity", "positive benefit-risk balance", 0.82),
            bridge("revolution", "paradigm shift in safety surveillance", 0.70),
            bridge("war", "drug safety crisis", 0.75),
            bridge("peace", "established safety profile", 0.78),
            bridge("nation", "healthcare system", 0.80),
            bridge("government", "regulatory framework", 0.88),
            bridge("opposition", "competing safety hypothesis", 0.72),
            bridge("debate", "benefit-risk deliberation", 0.80),
            bridge("rights", "patient rights to safety information", 0.85),
            bridge("duty", "reporting obligation", 0.90),
            bridge("vigilance", "pharmacovigilance", 0.98),
            bridge("justice", "equitable access to safety data", 0.78),
            // Biochemistry source-specific bridges
            bridge_for("enzyme", "drug metabolizing enzyme", 0.90, "biochemistry"),
            bridge_for("metabolite", "active metabolite", 0.92, "biochemistry"),
            bridge_for("pathway", "metabolic pathway", 0.88, "biochemistry"),
            bridge_for("substrate", "drug substrate", 0.85, "biochemistry"),
            bridge_for("reaction", "adverse drug reaction", 0.87, "biochemistry"),
            bridge_for("inhibitor", "CYP inhibitor", 0.90, "biochemistry"),
            bridge_for("cofactor", "cofactor dependency", 0.78, "biochemistry"),
            bridge_for("activation", "bioactivation", 0.85, "biochemistry"),
            bridge_for("cascade", "toxicity cascade", 0.82, "biochemistry"),
            bridge_for("regulation", "dose regulation", 0.80, "biochemistry"),
            bridge_for(
                "feedback",
                "therapeutic drug monitoring",
                0.75,
                "biochemistry",
            ),
            bridge_for("kinetics", "pharmacokinetics", 0.95, "biochemistry"),
            bridge_for("receptor", "drug receptor target", 0.90, "biochemistry"),
            bridge_for("binding", "receptor binding", 0.88, "biochemistry"),
        ],
        rhetorical_notes: "Pharmacovigilance literature favors evidence-based argumentation, \
            regulatory citation, quantitative thresholds, and cautious hedging. \
            Replace political passion with scientific rigor while preserving \
            argumentative structure."
            .to_string(),
    }
}

/// Profile 2: Software Architecture
pub fn builtin_software_architecture() -> DomainProfile {
    DomainProfile {
        name: "software-architecture".to_string(),
        display_name: "Software Architecture".to_string(),
        description: "System design, microservices, distributed computing, and DevOps".to_string(),
        vocabulary: vec![
            "microservice",
            "API gateway",
            "circuit breaker",
            "event sourcing",
            "CQRS",
            "load balancer",
            "service mesh",
            "container",
            "orchestration",
            "deployment pipeline",
            "fault tolerance",
            "scalability",
            "distributed system",
            "message queue",
            "cache invalidation",
            "observability",
            "idempotency",
            "saga pattern",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        bridges: vec![
            bridge("governance", "architecture governance board", 0.85),
            bridge("constitution", "architecture decision records", 0.88),
            bridge("citizen", "service instance", 0.75),
            bridge("law", "design principle", 0.82),
            bridge("republic", "microservice ecosystem", 0.78),
            bridge("liberty", "service autonomy", 0.85),
            bridge("faction", "tightly coupled dependency", 0.80),
            bridge("tyranny", "monolithic bottleneck", 0.82),
            bridge("democracy", "consensus protocol", 0.78),
            bridge("union", "service mesh", 0.80),
            bridge("danger", "cascading failure", 0.90),
            bridge("power", "system authority", 0.78),
            bridge("people", "development teams", 0.80),
            bridge("nation", "bounded context", 0.75),
        ],
        rhetorical_notes: "Software architecture discourse uses technical precision, \
            trade-off analysis, and pattern language. Replace political rhetoric \
            with engineering reasoning."
            .to_string(),
    }
}

/// Profile 3: Organizational Design
pub fn builtin_organizational_design() -> DomainProfile {
    DomainProfile {
        name: "organizational-design".to_string(),
        display_name: "Organizational Design".to_string(),
        description:
            "Corporate governance, hierarchy, stakeholder management, and organizational theory"
                .to_string(),
        vocabulary: vec![
            "governance",
            "stakeholder",
            "span of control",
            "corporate charter",
            "board of directors",
            "delegation",
            "accountability",
            "KPI",
            "organizational culture",
            "change management",
            "matrix structure",
            "reporting line",
            "decision rights",
            "RACI matrix",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        bridges: vec![
            bridge("governance", "corporate governance framework", 0.95),
            bridge("constitution", "corporate charter", 0.92),
            bridge("citizen", "employee", 0.85),
            bridge("law", "corporate policy", 0.88),
            bridge("republic", "organization", 0.82),
            bridge("liberty", "employee empowerment", 0.78),
            bridge("faction", "organizational silo", 0.85),
            bridge("tyranny", "micromanagement", 0.80),
            bridge("democracy", "participative management", 0.82),
            bridge("union", "cross-functional team", 0.78),
            bridge("danger", "organizational dysfunction", 0.80),
            bridge("power", "executive authority", 0.90),
            bridge("people", "workforce", 0.88),
        ],
        rhetorical_notes: "Organizational design uses management theory terminology, \
            emphasizes structure-behavior relationships, and values measurable outcomes."
            .to_string(),
    }
}

/// Profile 4: Clinical Trials
pub fn builtin_clinical_trials() -> DomainProfile {
    DomainProfile {
        name: "clinical-trials".to_string(),
        display_name: "Clinical Trials".to_string(),
        description: "Drug development, randomized controlled trials, regulatory submissions"
            .to_string(),
        vocabulary: vec![
            "randomization",
            "blinding",
            "endpoint",
            "protocol",
            "IRB",
            "GCP",
            "informed consent",
            "efficacy",
            "Phase I/II/III/IV",
            "inclusion criteria",
            "exclusion criteria",
            "intention to treat",
            "per protocol",
            "adverse event",
            "serious adverse event",
            "data safety monitoring board",
            "interim analysis",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        bridges: vec![
            bridge("governance", "institutional review board", 0.88),
            bridge("constitution", "clinical trial protocol", 0.90),
            bridge("citizen", "trial participant", 0.92),
            bridge("law", "Good Clinical Practice", 0.90),
            bridge("republic", "clinical trial network", 0.78),
            bridge("liberty", "informed consent", 0.95),
            bridge("faction", "selection bias", 0.75),
            bridge("tyranny", "unblinded assessment", 0.72),
            bridge("democracy", "equipoise", 0.70),
            bridge("union", "multi-center collaboration", 0.85),
            bridge("danger", "safety signal in trial", 0.88),
            bridge("power", "statistical power", 0.82),
            bridge("people", "patient population", 0.90),
        ],
        rhetorical_notes: "Clinical trial writing is methodical, protocol-driven, and \
            regulatory-compliant. Replace political urgency with scientific rigor."
            .to_string(),
    }
}

/// Profile 5: Military/Defense
pub fn builtin_military_defense() -> DomainProfile {
    DomainProfile {
        name: "military-defense".to_string(),
        display_name: "Military & Defense".to_string(),
        description: "Strategy, logistics, command structure, and defense doctrine".to_string(),
        vocabulary: vec![
            "strategy",
            "logistics",
            "chain of command",
            "deterrence",
            "force projection",
            "intelligence",
            "reconnaissance",
            "rules of engagement",
            "theater of operations",
            "doctrine",
            "combined arms",
            "force multiplier",
            "operational tempo",
            "center of gravity",
            "mission command",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        bridges: vec![
            bridge("governance", "unified command structure", 0.88),
            bridge("constitution", "military doctrine", 0.85),
            bridge("citizen", "combatant", 0.72),
            bridge("law", "rules of engagement", 0.90),
            bridge("republic", "defense establishment", 0.80),
            bridge("liberty", "operational freedom", 0.78),
            bridge("faction", "insurgency", 0.82),
            bridge("tyranny", "authoritarian threat", 0.85),
            bridge("democracy", "civilian oversight", 0.80),
            bridge("union", "military alliance", 0.92),
            bridge("danger", "strategic threat", 0.90),
            bridge("power", "military power", 0.95),
            bridge("people", "population", 0.80),
            bridge("nation", "nation-state", 0.95),
            // Biochemistry source-specific bridges
            bridge_for("enzyme", "special operations unit", 0.82, "biochemistry"),
            bridge_for("catalyst", "force multiplier", 0.85, "biochemistry"),
            bridge_for("metabolite", "logistics asset", 0.78, "biochemistry"),
            bridge_for("pathway", "operational corridor", 0.80, "biochemistry"),
            bridge_for("substrate", "strategic resource", 0.75, "biochemistry"),
            bridge_for("reaction", "tactical engagement", 0.83, "biochemistry"),
            bridge_for("inhibitor", "countermeasure", 0.85, "biochemistry"),
            bridge_for("cofactor", "force enabler", 0.78, "biochemistry"),
            bridge_for("activation", "mission readiness", 0.80, "biochemistry"),
            bridge_for("cascade", "escalation sequence", 0.87, "biochemistry"),
            bridge_for("regulation", "command and control", 0.82, "biochemistry"),
            bridge_for("feedback", "after-action review", 0.75, "biochemistry"),
            bridge_for("equilibrium", "force balance", 0.80, "biochemistry"),
            bridge_for("kinetics", "operational tempo", 0.85, "biochemistry"),
        ],
        rhetorical_notes: "Military writing uses direct, decisive language with emphasis \
            on chain of command, threat assessment, and strategic clarity."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_5_builtins() {
        let reg = DomainProfileRegistry::new();
        assert_eq!(reg.len(), 5);
    }

    #[test]
    fn test_registry_lookup_case_insensitive() {
        let reg = DomainProfileRegistry::new();
        assert!(reg.get("pharmacovigilance").is_some());
        assert!(reg.get("Pharmacovigilance").is_some());
        assert!(reg.get("PHARMACOVIGILANCE").is_some());
    }

    #[test]
    fn test_registry_lookup_with_spaces() {
        let reg = DomainProfileRegistry::new();
        assert!(reg.get("software architecture").is_some());
        assert!(reg.get("software-architecture").is_some());
    }

    #[test]
    fn test_registry_list_sorted() {
        let reg = DomainProfileRegistry::new();
        let names = reg.list();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn test_registry_custom_profile() {
        let mut reg = DomainProfileRegistry::new();
        assert_eq!(reg.len(), 5);
        reg.register(DomainProfile {
            name: "test-domain".to_string(),
            display_name: "Test".to_string(),
            description: "Test domain".to_string(),
            vocabulary: vec!["term1".to_string()],
            bridges: vec![bridge("a", "b", 0.5)],
            rhetorical_notes: String::new(),
        });
        assert_eq!(reg.len(), 6);
        assert!(reg.get("test-domain").is_some());
    }

    #[test]
    fn test_profile_find_bridge() {
        let pv = builtin_pharmacovigilance();
        let b = pv.find_bridge("citizen", None);
        assert!(b.is_some());
        let b = b.unwrap_or_else(|| panic!("bridge not found"));
        assert_eq!(b.specific, "patient");
    }

    #[test]
    fn test_profile_vocabulary_set() {
        let pv = builtin_pharmacovigilance();
        let set = pv.vocabulary_set();
        assert!(set.contains("signal"));
        assert!(set.contains("adverse event"));
    }

    #[test]
    fn test_all_builtins_have_bridges() {
        let reg = DomainProfileRegistry::new();
        for name in reg.list() {
            let profile = reg.get(name).unwrap_or_else(|| panic!("missing {}", name));
            assert!(!profile.bridges.is_empty(), "{} has no bridges", name);
            assert!(!profile.vocabulary.is_empty(), "{} has no vocabulary", name);
        }
    }

    #[test]
    fn test_bridge_confidences_in_range() {
        let reg = DomainProfileRegistry::new();
        for name in reg.list() {
            let profile = reg.get(name).unwrap_or_else(|| panic!("missing {}", name));
            for b in &profile.bridges {
                assert!(
                    (0.0..=1.0).contains(&b.confidence),
                    "{}: bridge '{}' confidence {} out of range",
                    name,
                    b.generic,
                    b.confidence
                );
            }
        }
    }

    #[test]
    fn test_rhetorical_role_guidance_not_empty() {
        let roles = [
            RhetoricalRole::Introduction,
            RhetoricalRole::Thesis,
            RhetoricalRole::Argument,
            RhetoricalRole::Counterargument,
            RhetoricalRole::Rebuttal,
            RhetoricalRole::Example,
            RhetoricalRole::Transition,
            RhetoricalRole::Conclusion,
            RhetoricalRole::Expository,
        ];
        for role in &roles {
            assert!(!role.rewrite_guidance().is_empty());
        }
    }

    #[test]
    fn test_empty_registry() {
        let reg = DomainProfileRegistry::empty();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(reg.get("pharmacovigilance").is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // Source-aware bridge tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_source_aware_match() {
        let mil = builtin_military_defense();
        let b = mil.find_bridge("enzyme", Some("biochemistry"));
        assert!(b.is_some());
        let b = b.unwrap_or_else(|| panic!("bridge not found"));
        assert_eq!(b.specific, "special operations unit");
        assert_eq!(b.source_domain.as_deref(), Some("biochemistry"));
    }

    #[test]
    fn test_source_aware_fallback_to_universal() {
        let mil = builtin_military_defense();
        // "citizen" has no biochemistry-specific bridge, should fall back to universal
        let b = mil.find_bridge("citizen", Some("biochemistry"));
        assert!(b.is_some());
        let b = b.unwrap_or_else(|| panic!("bridge not found"));
        assert_eq!(b.specific, "combatant");
        assert!(b.source_domain.is_none());
    }

    #[test]
    fn test_source_specific_overrides_universal() {
        let pv = builtin_pharmacovigilance();
        // With biochemistry source: "reaction" should map to "adverse drug reaction"
        let b_bio = pv.find_bridge("reaction", Some("biochemistry"));
        assert!(b_bio.is_some());
        let b_bio = b_bio.unwrap_or_else(|| panic!("bio bridge not found"));
        assert_eq!(b_bio.specific, "adverse drug reaction");

        // Without source: "reaction" has no universal bridge
        let b_none = pv.find_bridge("reaction", None);
        assert!(b_none.is_none());
    }

    #[test]
    fn test_source_domain_case_insensitive() {
        let mil = builtin_military_defense();
        let b1 = mil.find_bridge("enzyme", Some("Biochemistry"));
        let b2 = mil.find_bridge("enzyme", Some("BIOCHEMISTRY"));
        let b3 = mil.find_bridge("enzyme", Some("biochemistry"));
        assert!(b1.is_some());
        assert!(b2.is_some());
        assert!(b3.is_some());
        assert_eq!(
            b1.unwrap_or_else(|| panic!("b1")).specific,
            b3.unwrap_or_else(|| panic!("b3")).specific,
        );
    }

    #[test]
    fn test_known_source_domains() {
        let mil = builtin_military_defense();
        let domains = mil.known_source_domains();
        assert_eq!(domains, vec!["biochemistry"]);

        // Software architecture has no source-specific bridges
        let sw = builtin_software_architecture();
        assert!(sw.known_source_domains().is_empty());
    }
}
