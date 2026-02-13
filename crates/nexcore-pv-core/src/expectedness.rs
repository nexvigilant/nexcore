//! Expectedness Primitives for Pharmacovigilance.
//!
//! Determines whether adverse events are expected (listed) or unexpected
//! based on product labeling information:
//! - Label lookup and parsing
//! - Expectedness classification (listed/unlisted)
//! - Listedness by regulatory region
//! - Important Medical Events (IME) flagging

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

// =============================================================================
// Expectedness Classification
// =============================================================================

/// Expectedness classification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expectedness {
    /// Event is listed in product labeling.
    Listed,
    /// Event is not listed in product labeling.
    Unlisted,
    /// Cannot determine (no label data available).
    Unknown,
}

impl Expectedness {
    /// Whether this triggers expedited reporting requirements.
    #[must_use]
    pub fn requires_expedited(&self, is_serious: bool) -> bool {
        // Unlisted + Serious = Expedited (7/15 day reports)
        matches!(self, Self::Unlisted) && is_serious
    }

    /// Regulatory weight for signal prioritization.
    #[must_use]
    pub fn priority_weight(&self) -> f64 {
        match self {
            Self::Unlisted => 1.5, // Higher priority
            Self::Listed => 1.0,
            Self::Unknown => 1.2, // Treat with caution
        }
    }
}

// =============================================================================
// Label Information
// =============================================================================

/// Product label information for expectedness lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductLabel {
    /// Drug name (generic or brand).
    pub drug_name: String,
    /// Regulatory region (US, EU, JP, etc.).
    pub region: RegulatoryRegion,
    /// Listed adverse reactions (normalized terms).
    pub listed_reactions: HashSet<String>,
    /// Important Medical Events subset.
    pub ime_reactions: HashSet<String>,
    /// Label version date (YYYYMMDD).
    pub version_date: Option<String>,
    /// Source (USPI, SmPC, JPI, etc.).
    pub source: LabelSource,
}

/// Regulatory region for label lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegulatoryRegion {
    /// United States (FDA/USPI).
    US,
    /// European Union (EMA/SmPC).
    EU,
    /// Japan (PMDA/JPI).
    JP,
    /// United Kingdom (MHRA).
    UK,
    /// Canada (Health Canada).
    CA,
    /// Australia (TGA).
    AU,
    /// World Health Organization reference.
    WHO,
    /// Other/Unknown region.
    Other,
}

impl RegulatoryRegion {
    /// Reporting timeline for unexpected serious ADRs (days).
    #[must_use]
    pub fn expedited_timeline_days(&self) -> u32 {
        match self {
            Self::US => 15,  // FDA: 15 calendar days
            Self::EU => 15,  // EMA: 15 calendar days
            Self::JP => 15,  // PMDA: 15 days
            Self::UK => 15,  // MHRA: 15 days
            Self::CA => 15,  // HC: 15 days
            Self::AU => 15,  // TGA: 15 days
            Self::WHO => 15, // Standard
            Self::Other => 15,
        }
    }

    /// Fatal/life-threatening expedited timeline (days).
    #[must_use]
    pub fn fatal_timeline_days(&self) -> u32 {
        match self {
            Self::US => 7, // FDA: 7 days for fatal/life-threatening
            Self::EU => 7, // EMA: 7 days
            Self::JP => 7,
            Self::UK => 7,
            Self::CA => 7,
            Self::AU => 7,
            Self::WHO => 7,
            Self::Other => 7,
        }
    }
}

/// Label source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelSource {
    /// US Prescribing Information.
    USPI,
    /// EU Summary of Product Characteristics.
    SmPC,
    /// Japanese Package Insert.
    JPI,
    /// Company Core Data Sheet.
    CCDS,
    /// Reference Safety Information.
    RSI,
    /// Investigator's Brochure (for clinical trials).
    IB,
    /// Other source.
    Other,
}

impl ProductLabel {
    /// Create a new product label.
    #[must_use]
    pub fn new(drug_name: &str, region: RegulatoryRegion, source: LabelSource) -> Self {
        Self {
            drug_name: drug_name.to_string(),
            region,
            listed_reactions: HashSet::new(),
            ime_reactions: HashSet::new(),
            version_date: None,
            source,
        }
    }

    /// Add a listed reaction (normalized to uppercase).
    pub fn add_reaction(&mut self, reaction: &str) {
        self.listed_reactions.insert(reaction.to_uppercase());
    }

    /// Add an IME reaction.
    pub fn add_ime(&mut self, reaction: &str) {
        let normalized = reaction.to_uppercase();
        self.listed_reactions.insert(normalized.clone());
        self.ime_reactions.insert(normalized);
    }

    /// Check if a reaction is listed.
    #[must_use]
    pub fn is_listed(&self, reaction: &str) -> bool {
        self.listed_reactions.contains(&reaction.to_uppercase())
    }

    /// Check if a reaction is an IME.
    #[must_use]
    pub fn is_ime(&self, reaction: &str) -> bool {
        self.ime_reactions.contains(&reaction.to_uppercase())
    }
}

// =============================================================================
// Expectedness Lookup
// =============================================================================

/// Label registry for expectedness lookups.
#[derive(Debug, Clone, Default)]
pub struct LabelRegistry {
    /// Labels indexed by (drug_name_upper, region).
    labels: std::collections::HashMap<(String, RegulatoryRegion), ProductLabel>,
}

impl LabelRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a label to the registry.
    pub fn add_label(&mut self, label: ProductLabel) {
        let key = (label.drug_name.to_uppercase(), label.region);
        self.labels.insert(key, label);
    }

    /// Look up expectedness for a drug-event pair.
    #[must_use]
    pub fn lookup(&self, drug: &str, event: &str, region: RegulatoryRegion) -> ExpectednessResult {
        let key = (drug.to_uppercase(), region);

        match self.labels.get(&key) {
            Some(label) => {
                let is_listed = label.is_listed(event);
                let is_ime = label.is_ime(event);

                ExpectednessResult {
                    drug: drug.to_string(),
                    event: event.to_string(),
                    expectedness: if is_listed {
                        Expectedness::Listed
                    } else {
                        Expectedness::Unlisted
                    },
                    is_ime,
                    region,
                    label_version: label.version_date.clone(),
                    label_source: Some(label.source),
                }
            }
            None => ExpectednessResult {
                drug: drug.to_string(),
                event: event.to_string(),
                expectedness: Expectedness::Unknown,
                is_ime: false,
                region,
                label_version: None,
                label_source: None,
            },
        }
    }

    /// Look up across all regions, returning first match.
    #[must_use]
    pub fn lookup_any_region(&self, drug: &str, event: &str) -> ExpectednessResult {
        let drug_upper = drug.to_uppercase();

        for region in [
            RegulatoryRegion::US,
            RegulatoryRegion::EU,
            RegulatoryRegion::JP,
            RegulatoryRegion::UK,
            RegulatoryRegion::CA,
            RegulatoryRegion::AU,
        ] {
            let key = (drug_upper.clone(), region);
            if let Some(label) = self.labels.get(&key) {
                let is_listed = label.is_listed(event);
                if is_listed {
                    return ExpectednessResult {
                        drug: drug.to_string(),
                        event: event.to_string(),
                        expectedness: Expectedness::Listed,
                        is_ime: label.is_ime(event),
                        region,
                        label_version: label.version_date.clone(),
                        label_source: Some(label.source),
                    };
                }
            }
        }

        // No listing found in any region
        ExpectednessResult {
            drug: drug.to_string(),
            event: event.to_string(),
            expectedness: Expectedness::Unlisted,
            is_ime: false,
            region: RegulatoryRegion::Other,
            label_version: None,
            label_source: None,
        }
    }

    /// Number of labels in registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Whether registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

/// Result of expectedness lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectednessResult {
    /// Drug name queried.
    pub drug: String,
    /// Event name queried.
    pub event: String,
    /// Expectedness classification.
    pub expectedness: Expectedness,
    /// Whether event is an Important Medical Event.
    pub is_ime: bool,
    /// Region checked.
    pub region: RegulatoryRegion,
    /// Label version date if available.
    pub label_version: Option<String>,
    /// Label source if available.
    pub label_source: Option<LabelSource>,
}

impl ExpectednessResult {
    /// Determine if expedited reporting is required.
    #[must_use]
    pub fn requires_expedited(&self, is_serious: bool) -> bool {
        self.expectedness.requires_expedited(is_serious)
    }

    /// Get reporting deadline in days.
    #[must_use]
    pub fn reporting_deadline_days(&self, is_fatal: bool) -> u32 {
        if is_fatal {
            self.region.fatal_timeline_days()
        } else {
            self.region.expedited_timeline_days()
        }
    }
}

// =============================================================================
// Important Medical Events (IME) List
// =============================================================================

/// Common IME terms (EMA IME list subset).
/// In production, this would be loaded from official EMA IME list.
pub const COMMON_IME_TERMS: &[&str] = &[
    "AGRANULOCYTOSIS",
    "ANAPHYLACTIC REACTION",
    "ANAPHYLACTIC SHOCK",
    "APLASTIC ANAEMIA",
    "CARDIAC ARREST",
    "CEREBROVASCULAR ACCIDENT",
    "COMA",
    "DEATH",
    "HEPATIC FAILURE",
    "MALIGNANT NEOPLASM",
    "MYOCARDIAL INFARCTION",
    "PANCREATITIS ACUTE",
    "PULMONARY EMBOLISM",
    "RENAL FAILURE ACUTE",
    "RESPIRATORY FAILURE",
    "SEPSIS",
    "STEVENS-JOHNSON SYNDROME",
    "SUDDEN DEATH",
    "THROMBOCYTOPENIA",
    "TOXIC EPIDERMAL NECROLYSIS",
    "TORSADE DE POINTES",
    "VENTRICULAR FIBRILLATION",
];

/// Check if an event is on the IME list.
#[must_use]
pub fn is_ime_term(event: &str) -> bool {
    let upper = event.to_uppercase();
    COMMON_IME_TERMS.iter().any(|&ime| upper.contains(ime))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_label() -> ProductLabel {
        let mut label = ProductLabel::new("ASPIRIN", RegulatoryRegion::US, LabelSource::USPI);
        label.add_reaction("HEADACHE");
        label.add_reaction("NAUSEA");
        label.add_reaction("GASTROINTESTINAL HEMORRHAGE");
        label.add_ime("ANAPHYLACTIC REACTION");
        label.version_date = Some("20240101".to_string());
        label
    }

    #[test]
    fn test_expectedness_listed() {
        let label = sample_label();
        assert!(label.is_listed("headache")); // Case insensitive
        assert!(label.is_listed("NAUSEA"));
    }

    #[test]
    fn test_expectedness_unlisted() {
        let label = sample_label();
        assert!(!label.is_listed("HEPATOTOXICITY"));
    }

    #[test]
    fn test_ime_detection() {
        let label = sample_label();
        assert!(label.is_ime("anaphylactic reaction"));
        assert!(!label.is_ime("headache")); // Listed but not IME
    }

    #[test]
    fn test_registry_lookup() {
        let mut registry = LabelRegistry::new();
        registry.add_label(sample_label());

        let result = registry.lookup("ASPIRIN", "HEADACHE", RegulatoryRegion::US);
        assert_eq!(result.expectedness, Expectedness::Listed);
        assert!(!result.is_ime);

        let result = registry.lookup("ASPIRIN", "HEPATOTOXICITY", RegulatoryRegion::US);
        assert_eq!(result.expectedness, Expectedness::Unlisted);
    }

    #[test]
    fn test_registry_unknown_drug() {
        let registry = LabelRegistry::new();
        let result = registry.lookup("UNKNOWN_DRUG", "HEADACHE", RegulatoryRegion::US);
        assert_eq!(result.expectedness, Expectedness::Unknown);
    }

    #[test]
    fn test_expedited_reporting() {
        assert!(Expectedness::Unlisted.requires_expedited(true));
        assert!(!Expectedness::Unlisted.requires_expedited(false));
        assert!(!Expectedness::Listed.requires_expedited(true));
    }

    #[test]
    fn test_reporting_timeline() {
        assert_eq!(RegulatoryRegion::US.expedited_timeline_days(), 15);
        assert_eq!(RegulatoryRegion::US.fatal_timeline_days(), 7);
        assert_eq!(RegulatoryRegion::EU.expedited_timeline_days(), 15);
    }

    #[test]
    fn test_ime_list() {
        assert!(is_ime_term("anaphylactic shock"));
        assert!(is_ime_term("DEATH"));
        assert!(is_ime_term("Stevens-Johnson Syndrome"));
        assert!(!is_ime_term("headache"));
    }

    #[test]
    fn test_priority_weight() {
        assert!(Expectedness::Unlisted.priority_weight() > Expectedness::Listed.priority_weight());
    }

    #[test]
    fn test_result_expedited() {
        let result = ExpectednessResult {
            drug: "TEST".to_string(),
            event: "EVENT".to_string(),
            expectedness: Expectedness::Unlisted,
            is_ime: false,
            region: RegulatoryRegion::US,
            label_version: None,
            label_source: None,
        };

        assert!(result.requires_expedited(true));
        assert_eq!(result.reporting_deadline_days(false), 15);
        assert_eq!(result.reporting_deadline_days(true), 7);
    }

    #[test]
    fn test_lookup_any_region() {
        let mut registry = LabelRegistry::new();

        let mut us_label = ProductLabel::new("METFORMIN", RegulatoryRegion::US, LabelSource::USPI);
        us_label.add_reaction("DIARRHEA");
        registry.add_label(us_label);

        let mut eu_label = ProductLabel::new("METFORMIN", RegulatoryRegion::EU, LabelSource::SmPC);
        eu_label.add_reaction("LACTIC ACIDOSIS");
        registry.add_label(eu_label);

        // Should find US listing
        let result = registry.lookup_any_region("METFORMIN", "DIARRHEA");
        assert_eq!(result.expectedness, Expectedness::Listed);
        assert_eq!(result.region, RegulatoryRegion::US);

        // Should find EU listing
        let result = registry.lookup_any_region("METFORMIN", "LACTIC ACIDOSIS");
        assert_eq!(result.expectedness, Expectedness::Listed);
        assert_eq!(result.region, RegulatoryRegion::EU);
    }
}
