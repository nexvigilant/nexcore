//! # Signal Detection Thresholds
//!
//! Standard criteria for signal detection (Evans, WHO-UMC, FDA FAERS).

use serde::{Deserialize, Serialize};

/// Signal detection criteria thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCriteria {
    /// PRR threshold
    pub prr_threshold: f64,
    /// ROR lower CI threshold
    pub ror_lower_threshold: f64,
    /// Chi-square threshold (df=1, p<0.05 = 3.841)
    pub chi_square_threshold: f64,
    /// IC025 threshold
    pub ic025_threshold: f64,
    /// EB05 threshold
    pub eb05_threshold: f64,
    /// Minimum case count
    pub min_cases: u32,
}

impl SignalCriteria {
    /// Evans criteria (classic, widely used)
    #[must_use]
    pub const fn evans() -> Self {
        Self {
            prr_threshold: 2.0,
            ror_lower_threshold: 1.0,
            chi_square_threshold: 3.841, // p < 0.05, df=1
            ic025_threshold: 0.0,
            eb05_threshold: 2.0,
            min_cases: 3,
        }
    }

    /// WHO-UMC criteria
    #[must_use]
    pub const fn who_umc() -> Self {
        Self {
            prr_threshold: 2.0,
            ror_lower_threshold: 1.0,
            chi_square_threshold: 4.0,
            ic025_threshold: 0.0,
            eb05_threshold: 2.0,
            min_cases: 3,
        }
    }

    /// FDA FAERS criteria
    #[must_use]
    pub const fn fda_faers() -> Self {
        Self {
            prr_threshold: 2.0,
            ror_lower_threshold: 1.0,
            chi_square_threshold: 4.0,
            ic025_threshold: 0.0,
            eb05_threshold: 2.0,
            min_cases: 3,
        }
    }

    /// EMA GVP-IX criteria
    #[must_use]
    pub const fn ema_gvp_ix() -> Self {
        Self {
            prr_threshold: 2.0,
            ror_lower_threshold: 1.0,
            chi_square_threshold: 3.841,
            ic025_threshold: 0.0,
            eb05_threshold: 2.0,
            min_cases: 3,
        }
    }

    /// Strict criteria (high confidence only)
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            prr_threshold: 3.0,
            ror_lower_threshold: 2.0,
            chi_square_threshold: 6.635, // p < 0.01, df=1
            ic025_threshold: 1.0,
            eb05_threshold: 3.0,
            min_cases: 5,
        }
    }

    /// Sensitive criteria (lower thresholds for early detection)
    #[must_use]
    pub const fn sensitive() -> Self {
        Self {
            prr_threshold: 1.5,
            ror_lower_threshold: 1.0,
            chi_square_threshold: 2.706, // p < 0.10, df=1
            ic025_threshold: -0.5,
            eb05_threshold: 1.5,
            min_cases: 2,
        }
    }

    /// Check if PRR meets criteria
    #[must_use]
    pub fn meets_prr(&self, prr: f64, chi_square: f64, n: u32) -> bool {
        prr >= self.prr_threshold && chi_square >= self.chi_square_threshold && n >= self.min_cases
    }

    /// Check if ROR meets criteria
    #[must_use]
    pub fn meets_ror(&self, ror_lower: f64, n: u32) -> bool {
        ror_lower > self.ror_lower_threshold && n >= self.min_cases
    }

    /// Check if IC meets criteria
    #[must_use]
    pub fn meets_ic(&self, ic025: f64, n: u32) -> bool {
        ic025 > self.ic025_threshold && n >= self.min_cases
    }

    /// Check if EBGM meets criteria
    #[must_use]
    pub fn meets_ebgm(&self, eb05: f64, n: u32) -> bool {
        eb05 >= self.eb05_threshold && n >= self.min_cases
    }
}

impl SignalCriteria {
    /// List all named preset identifiers.
    ///
    /// Useful for discovering valid values to pass to [`Self::by_name`].
    #[must_use]
    pub fn known_presets() -> &'static [&'static str] {
        &[
            "evans",
            "who-umc",
            "fda-faers",
            "ema-gvp-ix",
            "strict",
            "sensitive",
        ]
    }

    /// Look up a named preset (case-insensitive, kebab or underscore).
    ///
    /// Returns `None` for unrecognized names.
    #[must_use]
    pub fn by_name(name: &str) -> Option<Self> {
        let normalized: String = name.to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "evans" | "default" => Some(Self::evans()),
            "who-umc" | "who" | "umc" => Some(Self::who_umc()),
            "fda-faers" | "fda" | "faers" => Some(Self::fda_faers()),
            "ema-gvp-ix" | "ema" | "gvp-ix" => Some(Self::ema_gvp_ix()),
            "strict" | "high-confidence" => Some(Self::strict()),
            "sensitive" | "early-detection" => Some(Self::sensitive()),
            _ => None,
        }
    }
}

/// Context-qualified threshold: a named criteria set bound to an optional qualifier.
///
/// When `context` is `None`, the entry is universal (matches any query).
/// When set, it only fires for that specific context value, enabling
/// seriousness-aware, jurisdiction-aware, or drug-class-aware threshold selection.
///
/// Tier: T2-P | Dominant: mu (Mapping) × kappa (Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualThreshold {
    /// The context qualifier (e.g., "fatal", "oncology", "eu").
    /// `None` = universal.
    pub context: Option<String>,
    /// The criteria set to use when this context matches.
    pub criteria: SignalCriteria,
    /// Human-readable label for this threshold configuration.
    pub label: String,
}

/// Registry of context-qualified thresholds with two-pass lookup.
///
/// Lookup strategy (identical to `DomainProfile::find_bridge`):
///   1. Context-specific match (exact qualifier)
///   2. Universal fallback (context = None)
///
/// Tier: T2-C | Dominant: sigma (Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdRegistry {
    entries: Vec<ContextualThreshold>,
}

impl ThresholdRegistry {
    /// Create a registry with P0-aligned seriousness-based defaults.
    ///
    /// Fatal/Life-threatening → sensitive (early detection, P0 mandate)
    /// Disability → evans (standard)
    /// Hospitalization/Non-serious → evans (standard)
    /// Confirmed signals → strict (high confidence for action)
    #[must_use]
    pub fn with_seriousness_defaults() -> Self {
        Self {
            entries: vec![
                ContextualThreshold {
                    context: Some("fatal".to_string()),
                    criteria: SignalCriteria::sensitive(),
                    label: "Fatal — sensitive thresholds (P0 mandate)".to_string(),
                },
                ContextualThreshold {
                    context: Some("life-threatening".to_string()),
                    criteria: SignalCriteria::sensitive(),
                    label: "Life-threatening — sensitive thresholds (P0 mandate)".to_string(),
                },
                ContextualThreshold {
                    context: Some("disability".to_string()),
                    criteria: SignalCriteria::evans(),
                    label: "Disability — Evans standard thresholds".to_string(),
                },
                ContextualThreshold {
                    context: Some("hospitalization".to_string()),
                    criteria: SignalCriteria::evans(),
                    label: "Hospitalization — Evans standard thresholds".to_string(),
                },
                ContextualThreshold {
                    context: Some("confirmed".to_string()),
                    criteria: SignalCriteria::strict(),
                    label: "Confirmed signal — strict thresholds".to_string(),
                },
                // Universal fallback
                ContextualThreshold {
                    context: None,
                    criteria: SignalCriteria::evans(),
                    label: "Universal fallback — Evans criteria".to_string(),
                },
            ],
        }
    }

    /// Create an empty registry.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a context-qualified threshold entry.
    pub fn add(&mut self, entry: ContextualThreshold) {
        self.entries.push(entry);
    }

    /// Two-pass lookup: context-specific first, universal fallback second.
    ///
    /// Case-insensitive matching on context qualifier.
    #[must_use]
    pub fn resolve(&self, context: Option<&str>) -> Option<&ContextualThreshold> {
        let ctx_lower = context.map(|c| c.to_lowercase());

        // Pass 1: context-specific match
        if let Some(ref ctx) = ctx_lower {
            if let Some(entry) = self.entries.iter().find(|e| {
                e.context
                    .as_ref()
                    .is_some_and(|c| c.eq_ignore_ascii_case(ctx))
            }) {
                return Some(entry);
            }
        }

        // Pass 2: universal fallback
        self.entries.iter().find(|e| e.context.is_none())
    }

    /// Returns sorted list of known context qualifiers in this registry.
    #[must_use]
    pub fn known_contexts(&self) -> Vec<String> {
        let mut contexts: Vec<String> = self
            .entries
            .iter()
            .filter_map(|e| e.context.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        contexts.sort();
        contexts
    }

    /// Number of entries in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ThresholdRegistry {
    fn default() -> Self {
        Self::with_seriousness_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evans_criteria() {
        let criteria = SignalCriteria::evans();
        assert_eq!(criteria.prr_threshold, 2.0);
        assert_eq!(criteria.chi_square_threshold, 3.841);
        assert_eq!(criteria.min_cases, 3);
    }

    #[test]
    fn test_meets_prr() {
        let criteria = SignalCriteria::evans();
        assert!(criteria.meets_prr(2.5, 4.0, 5));
        assert!(!criteria.meets_prr(1.5, 4.0, 5)); // PRR too low
        assert!(!criteria.meets_prr(2.5, 3.0, 5)); // Chi-square too low
        assert!(!criteria.meets_prr(2.5, 4.0, 2)); // N too low
    }

    // ═══════════════════════════════════════════════════════════════
    // Named preset tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_known_presets() {
        let presets = SignalCriteria::known_presets();
        assert_eq!(presets.len(), 6);
        assert!(presets.contains(&"evans"));
        assert!(presets.contains(&"sensitive"));
    }

    #[test]
    fn test_by_name_case_insensitive() {
        assert!(SignalCriteria::by_name("Evans").is_some());
        assert!(SignalCriteria::by_name("EVANS").is_some());
        assert!(SignalCriteria::by_name("evans").is_some());
    }

    #[test]
    fn test_by_name_aliases() {
        assert!(SignalCriteria::by_name("who").is_some());
        assert!(SignalCriteria::by_name("fda").is_some());
        assert!(SignalCriteria::by_name("ema").is_some());
        assert!(SignalCriteria::by_name("default").is_some());
    }

    #[test]
    fn test_by_name_unknown() {
        assert!(SignalCriteria::by_name("nonexistent").is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // Context-aware threshold registry tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_registry_seriousness_defaults() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        assert_eq!(reg.len(), 6);
    }

    #[test]
    fn test_resolve_fatal_gets_sensitive() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        let entry = reg.resolve(Some("fatal"));
        assert!(entry.is_some());
        let entry = entry.unwrap_or_else(|| panic!("entry not found"));
        // Fatal should use sensitive thresholds (lower PRR = 1.5)
        assert_eq!(entry.criteria.prr_threshold, 1.5);
        assert_eq!(entry.criteria.min_cases, 2);
    }

    #[test]
    fn test_resolve_hospitalization_gets_evans() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        let entry = reg.resolve(Some("hospitalization"));
        assert!(entry.is_some());
        let entry = entry.unwrap_or_else(|| panic!("entry not found"));
        // Hospitalization should use Evans (standard PRR = 2.0)
        assert_eq!(entry.criteria.prr_threshold, 2.0);
        assert_eq!(entry.criteria.min_cases, 3);
    }

    #[test]
    fn test_resolve_unknown_falls_back_to_universal() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        let entry = reg.resolve(Some("unknown-context"));
        assert!(entry.is_some());
        let entry = entry.unwrap_or_else(|| panic!("fallback not found"));
        assert!(entry.context.is_none()); // Universal entry
        assert_eq!(entry.criteria.prr_threshold, 2.0);
    }

    #[test]
    fn test_resolve_none_gets_universal() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        let entry = reg.resolve(None);
        assert!(entry.is_some());
        let entry = entry.unwrap_or_else(|| panic!("universal not found"));
        assert!(entry.context.is_none());
    }

    #[test]
    fn test_resolve_case_insensitive() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        let e1 = reg.resolve(Some("Fatal"));
        let e2 = reg.resolve(Some("FATAL"));
        let e3 = reg.resolve(Some("fatal"));
        assert!(e1.is_some());
        assert!(e2.is_some());
        assert!(e3.is_some());
        assert_eq!(
            e1.unwrap_or_else(|| panic!("e1")).criteria.prr_threshold,
            e3.unwrap_or_else(|| panic!("e3")).criteria.prr_threshold,
        );
    }

    #[test]
    fn test_known_contexts() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        let contexts = reg.known_contexts();
        assert!(contexts.contains(&"fatal".to_string()));
        assert!(contexts.contains(&"life-threatening".to_string()));
        assert!(contexts.contains(&"confirmed".to_string()));
        // Universal entry should NOT appear
        assert!(!contexts.contains(&String::new()));
    }

    #[test]
    fn test_confirmed_gets_strict() {
        let reg = ThresholdRegistry::with_seriousness_defaults();
        let entry = reg.resolve(Some("confirmed"));
        assert!(entry.is_some());
        let entry = entry.unwrap_or_else(|| panic!("entry not found"));
        // Confirmed signals should use strict thresholds
        assert_eq!(entry.criteria.prr_threshold, 3.0);
        assert_eq!(entry.criteria.min_cases, 5);
    }

    #[test]
    fn test_empty_registry_resolve_returns_none() {
        let reg = ThresholdRegistry::empty();
        assert!(reg.resolve(Some("fatal")).is_none());
        assert!(reg.resolve(None).is_none());
    }
}
