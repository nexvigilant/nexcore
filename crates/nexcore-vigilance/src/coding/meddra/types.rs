//! `MedDRA` type definitions.
//!
//! `MedDRA` (Medical Dictionary for Regulatory Activities) is a hierarchical
//! terminology for adverse event reporting with 5 levels:
//!
//! ```text
//! SOC (System Organ Class)           [27 classes]
//!  └── HLGT (High Level Group Term)  [~337 terms]
//!       └── HLT (High Level Term)    [~1,737 terms]
//!            └── PT (Preferred Term) [~23,000 terms]
//!                 └── LLT (Lowest Level Term) [~80,000 terms]
//! ```

use serde::{Deserialize, Serialize};

/// `MedDRA` hierarchy level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HierarchyLevel {
    /// Lowest Level Term - most specific, includes synonyms
    Llt,
    /// Preferred Term - standard reporting level
    Pt,
    /// High Level Term - grouped PTs
    Hlt,
    /// High Level Group Term - grouped HLTs
    Hlgt,
    /// System Organ Class - highest level, anatomical/physiological
    Soc,
}

impl HierarchyLevel {
    /// Return the level number (0 = LLT, 4 = SOC).
    #[must_use]
    pub const fn level_number(&self) -> u8 {
        match self {
            Self::Llt => 0,
            Self::Pt => 1,
            Self::Hlt => 2,
            Self::Hlgt => 3,
            Self::Soc => 4,
        }
    }

    /// Return the parent level, if any.
    #[must_use]
    pub const fn parent_level(&self) -> Option<Self> {
        match self {
            Self::Llt => Some(Self::Pt),
            Self::Pt => Some(Self::Hlt),
            Self::Hlt => Some(Self::Hlgt),
            Self::Hlgt => Some(Self::Soc),
            Self::Soc => None,
        }
    }
}

impl std::fmt::Display for HierarchyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Llt => write!(f, "LLT"),
            Self::Pt => write!(f, "PT"),
            Self::Hlt => write!(f, "HLT"),
            Self::Hlgt => write!(f, "HLGT"),
            Self::Soc => write!(f, "SOC"),
        }
    }
}

/// Lowest Level Term (LLT) - most specific term level.
///
/// LLTs include synonyms, lexical variants, and quasi-synonyms
/// that map to a single PT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Llt {
    /// LLT code (8 digits)
    pub code: u32,
    /// LLT name
    pub name: String,
    /// Parent PT code
    pub pt_code: u32,
    /// Currency status (Y = current, N = non-current)
    pub is_current: bool,
}

/// Preferred Term (PT) - standard reporting level.
///
/// PTs represent single medical concepts and are the primary
/// level used in safety reporting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pt {
    /// PT code (8 digits)
    pub code: u32,
    /// PT name
    pub name: String,
    /// Primary SOC code
    pub primary_soc_code: u32,
}

/// High Level Term (HLT) - grouping of related PTs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hlt {
    /// HLT code (8 digits)
    pub code: u32,
    /// HLT name
    pub name: String,
}

/// High Level Group Term (HLGT) - grouping of related HLTs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hlgt {
    /// HLGT code (8 digits)
    pub code: u32,
    /// HLGT name
    pub name: String,
}

/// System Organ Class (SOC) - highest level in hierarchy.
///
/// SOCs represent anatomical or physiological systems, etiology,
/// or purpose (e.g., "Cardiac disorders", "Infections").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Soc {
    /// SOC code (8 digits)
    pub code: u32,
    /// SOC name
    pub name: String,
    /// SOC abbreviation
    pub abbrev: String,
    /// International SOC order
    pub intl_order: u16,
}

/// Complete hierarchy path from LLT to SOC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchyPath {
    /// LLT code
    pub llt_code: u32,
    /// LLT name
    pub llt_name: String,
    /// PT code
    pub pt_code: u32,
    /// PT name
    pub pt_name: String,
    /// HLT code
    pub hlt_code: u32,
    /// HLT name
    pub hlt_name: String,
    /// HLGT code
    pub hlgt_code: u32,
    /// HLGT name
    pub hlgt_name: String,
    /// SOC code
    pub soc_code: u32,
    /// SOC name
    pub soc_name: String,
}

/// Titration-based provenance for a search result.
///
/// Supplements the opaque `score: f64` with auditable POM titration metadata
/// showing how semantic equivalence was measured between the query and matched term.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitrationProvenance {
    /// Titration equivalence score (0.0-1.0)
    pub equivalence_score: f64,
    /// Verdict: "Equivalent" (>0.90), "PartialOverlap" (0.60-0.90), "Distinct" (<0.60)
    pub verdict: String,
    /// Number of shared canonical atoms between query and matched term
    pub shared_atoms: usize,
}

/// Search result with match score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matched term
    pub term: String,
    /// `MedDRA` code
    pub code: u32,
    /// Hierarchy level
    pub level: HierarchyLevel,
    /// Match score (0.0 to 1.0)
    pub score: f64,
    /// Edit distance from query
    pub distance: usize,
    /// Optional POM titration provenance for audit trail.
    ///
    /// `None` for hot-path fuzzy search results where titration overhead
    /// is not justified. Populated for exact-match and encode results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub titration_provenance: Option<TitrationProvenance>,
}

/// `MedDRA` version information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeddraVersion {
    /// Major version (e.g., 26)
    pub major: u8,
    /// Minor version (e.g., 1)
    pub minor: u8,
    /// Language (e.g., "English")
    pub language: String,
}

impl MeddraVersion {
    /// Create a new version.
    #[must_use]
    pub fn new(major: u8, minor: u8, language: impl Into<String>) -> Self {
        Self {
            major,
            minor,
            language: language.into(),
        }
    }
}

impl std::fmt::Display for MeddraVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MedDRA v{}.{} ({})",
            self.major, self.minor, self.language
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_level_order() {
        assert!(HierarchyLevel::Llt.level_number() < HierarchyLevel::Pt.level_number());
        assert!(HierarchyLevel::Pt.level_number() < HierarchyLevel::Hlt.level_number());
        assert!(HierarchyLevel::Hlt.level_number() < HierarchyLevel::Hlgt.level_number());
        assert!(HierarchyLevel::Hlgt.level_number() < HierarchyLevel::Soc.level_number());
    }

    #[test]
    fn test_hierarchy_parent() {
        assert_eq!(HierarchyLevel::Llt.parent_level(), Some(HierarchyLevel::Pt));
        assert_eq!(HierarchyLevel::Pt.parent_level(), Some(HierarchyLevel::Hlt));
        assert_eq!(HierarchyLevel::Soc.parent_level(), None);
    }

    #[test]
    fn test_version_display() {
        let v = MeddraVersion::new(26, 1, "English");
        assert_eq!(format!("{v}"), "MedDRA v26.1 (English)");
    }

    #[test]
    fn test_search_result_without_titration() {
        let result = SearchResult {
            term: "headache".into(),
            code: 10019211,
            level: HierarchyLevel::Pt,
            score: 0.95,
            distance: 1,
            titration_provenance: None,
        };
        assert!(result.titration_provenance.is_none());
        assert!((result.score - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_search_result_with_titration() {
        let result = SearchResult {
            term: "cardiac adverse event".into(),
            code: 10007515,
            level: HierarchyLevel::Pt,
            score: 0.85,
            distance: 2,
            titration_provenance: Some(TitrationProvenance {
                equivalence_score: 0.78,
                verdict: "PartialOverlap".into(),
                shared_atoms: 2,
            }),
        };
        let prov = result
            .titration_provenance
            .as_ref()
            .expect("should have provenance");
        assert!((prov.equivalence_score - 0.78).abs() < f64::EPSILON);
        assert_eq!(prov.shared_atoms, 2);
    }

    #[test]
    fn test_titration_provenance_serde_roundtrip() {
        let prov = TitrationProvenance {
            equivalence_score: 0.92,
            verdict: "Equivalent".into(),
            shared_atoms: 3,
        };
        let json = serde_json::to_string(&prov).expect("serialize");
        let parsed: TitrationProvenance = serde_json::from_str(&json).expect("deserialize");
        assert!((parsed.equivalence_score - 0.92).abs() < f64::EPSILON);
        assert_eq!(parsed.verdict, "Equivalent");
    }

    #[test]
    fn test_search_result_none_provenance_skipped_in_json() {
        let result = SearchResult {
            term: "test".into(),
            code: 1,
            level: HierarchyLevel::Llt,
            score: 1.0,
            distance: 0,
            titration_provenance: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("titration_provenance"));
    }
}
