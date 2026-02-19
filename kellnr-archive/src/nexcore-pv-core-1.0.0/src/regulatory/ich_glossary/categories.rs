//! Categories Module
//!
//! ICH guideline category metadata and lookup functions.

use crate::regulatory::ich_glossary::types::IchCategory;
use phf::phf_map;
use serde::Serialize;

// ============================================================================
// Category Metadata
// ============================================================================

/// Extended metadata for an ICH category
#[derive(Debug, Clone, Serialize)]
pub struct CategoryInfo {
    /// Category enum value
    pub category: IchCategory,
    /// Full name
    pub name: &'static str,
    /// Single-letter prefix
    pub prefix: &'static str,
    /// Description
    pub description: &'static str,
    /// Guideline ID ranges
    pub id_range: &'static str,
    /// Key topics covered
    pub topics: &'static [&'static str],
    /// Number of guidelines
    pub guideline_count: usize,
    /// Number of terms
    pub term_count: usize,
}

// ============================================================================
// Category Lookup Table (PHF)
// ============================================================================

/// Compile-time category lookup
pub static CATEGORIES: phf::Map<&'static str, CategoryInfo> = phf_map! {
    "quality" => CategoryInfo {
        category: IchCategory::Quality,
        name: "Quality",
        prefix: "Q",
        description: "Guidelines on pharmaceutical quality, stability, specifications, manufacturing, and risk management",
        id_range: "Q1-Q14",
        topics: &[
            "Stability testing",
            "Analytical validation",
            "Impurities control",
            "Specifications",
            "GMP for APIs",
            "Quality by Design",
            "Risk management",
            "Continuous manufacturing",
        ],
        guideline_count: 35,
        term_count: 312,
    },
    "safety" => CategoryInfo {
        category: IchCategory::Safety,
        name: "Safety",
        prefix: "S",
        description: "Guidelines on nonclinical safety assessment, toxicology, pharmacology, and carcinogenicity",
        id_range: "S1-S12",
        topics: &[
            "Carcinogenicity testing",
            "Genotoxicity testing",
            "Toxicokinetics",
            "Reproductive toxicology",
            "Safety pharmacology",
            "Immunotoxicology",
            "Photosafety",
            "Pediatric safety",
            "Gene therapy biodistribution",
        ],
        guideline_count: 18,
        term_count: 156,
    },
    "efficacy" => CategoryInfo {
        category: IchCategory::Efficacy,
        name: "Efficacy",
        prefix: "E",
        description: "Guidelines on clinical trial design, safety reporting, biostatistics, and GCP",
        id_range: "E1-E22",
        topics: &[
            "Clinical safety data management",
            "Expedited reporting",
            "ICSR transmission",
            "Periodic safety reports",
            "Good Clinical Practice",
            "Statistical principles",
            "Pediatric extrapolation",
            "Adaptive designs",
            "Rare diseases",
        ],
        guideline_count: 32,
        term_count: 485,
    },
    "multidisciplinary" => CategoryInfo {
        category: IchCategory::Multidisciplinary,
        name: "Multidisciplinary",
        prefix: "M",
        description: "Cross-cutting guidelines on electronic standards, CTD format, bioanalysis, and drug interactions",
        id_range: "M1-M15",
        topics: &[
            "Electronic standards",
            "Common Technical Document",
            "Mutagenic impurities",
            "Bioanalytical validation",
            "Clinical protocols",
            "Drug interactions",
            "Bioequivalence",
            "Real-world data",
            "Model-informed development",
        ],
        guideline_count: 22,
        term_count: 294,
    },
    "q" => CategoryInfo {
        category: IchCategory::Quality,
        name: "Quality",
        prefix: "Q",
        description: "Guidelines on pharmaceutical quality, stability, specifications, manufacturing, and risk management",
        id_range: "Q1-Q14",
        topics: &["Stability", "Impurities", "GMP", "QbD"],
        guideline_count: 35,
        term_count: 312,
    },
    "s" => CategoryInfo {
        category: IchCategory::Safety,
        name: "Safety",
        prefix: "S",
        description: "Guidelines on nonclinical safety assessment, toxicology, pharmacology, and carcinogenicity",
        id_range: "S1-S12",
        topics: &["Toxicology", "Pharmacology", "Carcinogenicity"],
        guideline_count: 18,
        term_count: 156,
    },
    "e" => CategoryInfo {
        category: IchCategory::Efficacy,
        name: "Efficacy",
        prefix: "E",
        description: "Guidelines on clinical trial design, safety reporting, biostatistics, and GCP",
        id_range: "E1-E22",
        topics: &["Clinical trials", "Safety reporting", "GCP"],
        guideline_count: 32,
        term_count: 485,
    },
    "m" => CategoryInfo {
        category: IchCategory::Multidisciplinary,
        name: "Multidisciplinary",
        prefix: "M",
        description: "Cross-cutting guidelines on electronic standards, CTD format, bioanalysis, and drug interactions",
        id_range: "M1-M15",
        topics: &["eCTD", "Bioanalysis", "Drug interactions"],
        guideline_count: 22,
        term_count: 294,
    },
};

// ============================================================================
// Lookup Functions
// ============================================================================

/// O(1) lookup of category info (case-insensitive, allocation-free)
pub fn lookup_category(name: &str) -> Option<&'static CategoryInfo> {
    // PHF map has 4 entries with lowercase keys; iterate to avoid allocating a lowercase copy
    CATEGORIES.entries().find_map(|(k, v)| {
        if k.eq_ignore_ascii_case(name) {
            Some(v)
        } else {
            None
        }
    })
}

/// Get category by enum value.
///
/// Returns `None` only if the PHF map is missing an entry (build error).
pub fn category_info(category: IchCategory) -> Option<&'static CategoryInfo> {
    let key = match category {
        IchCategory::Quality => "quality",
        IchCategory::Safety => "safety",
        IchCategory::Efficacy => "efficacy",
        IchCategory::Multidisciplinary => "multidisciplinary",
    };
    CATEGORIES.get(key)
}

/// Get all categories (returns only those present in the PHF map).
pub fn all_categories() -> Vec<&'static CategoryInfo> {
    ["quality", "safety", "efficacy", "multidisciplinary"]
        .iter()
        .filter_map(|k| CATEGORIES.get(*k))
        .collect()
}

/// Parse category from string (flexible matching)
pub fn parse_category(s: &str) -> Option<IchCategory> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "q" | "quality" => Some(IchCategory::Quality),
        "s" | "safety" => Some(IchCategory::Safety),
        "e" | "efficacy" => Some(IchCategory::Efficacy),
        "m" | "multidisciplinary" | "multi" => Some(IchCategory::Multidisciplinary),
        _ => None,
    }
}

// ============================================================================
// Statistics
// ============================================================================

/// Get total counts across all categories
pub fn total_counts() -> (usize, usize) {
    let categories = all_categories();
    let guidelines: usize = categories.iter().map(|c| c.guideline_count).sum();
    let terms: usize = categories.iter().map(|c| c.term_count).sum();
    (guidelines, terms)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_category_full_name() {
        let cat = lookup_category("quality").unwrap();
        assert_eq!(cat.prefix, "Q");
        assert_eq!(cat.category, IchCategory::Quality);
    }

    #[test]
    fn test_lookup_category_prefix() {
        let cat = lookup_category("q").unwrap();
        assert_eq!(cat.name, "Quality");
    }

    #[test]
    fn test_lookup_category_case_insensitive() {
        let c1 = lookup_category("QUALITY").unwrap();
        let c2 = lookup_category("quality").unwrap();
        assert_eq!(c1.prefix, c2.prefix);
    }

    #[test]
    fn test_category_info() {
        let info = category_info(IchCategory::Efficacy)
            .expect("test: efficacy category must exist in PHF map");
        assert_eq!(info.prefix, "E");
        assert!(!info.topics.is_empty());
    }

    #[test]
    fn test_all_categories() {
        let cats = all_categories();
        assert_eq!(cats.len(), 4);
    }

    #[test]
    fn test_parse_category() {
        assert_eq!(parse_category("Q"), Some(IchCategory::Quality));
        assert_eq!(parse_category("safety"), Some(IchCategory::Safety));
        assert_eq!(parse_category("E"), Some(IchCategory::Efficacy));
        assert_eq!(
            parse_category("multi"),
            Some(IchCategory::Multidisciplinary)
        );
        assert_eq!(parse_category("invalid"), None);
    }

    #[test]
    fn test_total_counts() {
        let (guidelines, terms) = total_counts();
        assert!(guidelines > 100);
        assert!(terms > 1000);
    }

    #[test]
    fn test_category_has_topics() {
        for cat in all_categories() {
            assert!(
                !cat.topics.is_empty(),
                "Category {} should have topics",
                cat.name
            );
        }
    }
}
