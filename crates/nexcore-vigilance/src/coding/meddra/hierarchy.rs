#![allow(
    clippy::disallowed_types,
    reason = "Meddra dictionary requires O(1) code/name lookup tables."
)]
#![allow(clippy::unreadable_literal)] // MedDRA numeric IDs are canonical fixture values.

//! `MedDRA` hierarchy traversal and lookup.
//!
//! Provides efficient traversal up and down the `MedDRA` hierarchy:
//! LLT -> PT -> HLT -> HLGT -> SOC
//!
//! # Complexity
//!
//! - Code lookup: O(1) using `HashMap`
//! - Hierarchy traversal: O(1) per level (O(5) total for full path)
//! - Name search: O(1) for exact match, O(n) for fuzzy search

use std::collections::HashMap;

use super::super::error::CodingError;
use super::super::fuzzy::BkTree;
use super::types::{
    HierarchyLevel, HierarchyPath, Hlgt, Hlt, Llt, MeddraVersion, Pt, SearchResult, Soc,
};

/// `MedDRA` dictionary with indexed hierarchy.
///
/// Provides O(1) lookups by code and efficient fuzzy search via BK-tree.
#[derive(Debug)]
pub struct MeddraDictionary {
    /// `MedDRA` version
    pub version: MeddraVersion,

    /// LLT lookup by code
    llt_by_code: HashMap<u32, Llt>,
    /// PT lookup by code
    pt_by_code: HashMap<u32, Pt>,
    /// HLT lookup by code
    hlt_by_code: HashMap<u32, Hlt>,
    /// HLGT lookup by code
    hlgt_by_code: HashMap<u32, Hlgt>,
    /// SOC lookup by code
    soc_by_code: HashMap<u32, Soc>,

    /// PT to HLT mapping (many-to-many via `hlt_pt.asc`)
    pt_to_hlts: HashMap<u32, Vec<u32>>,
    /// HLT to HLGT mapping (many-to-many via `hlgt_hlt.asc`)
    hlt_to_hlgts: HashMap<u32, Vec<u32>>,
    /// HLGT to SOC mapping (many-to-many via `soc_hlgt.asc`)
    hlgt_to_socs: HashMap<u32, Vec<u32>>,

    /// BK-tree for LLT fuzzy search
    llt_bktree: BkTree,
    /// BK-tree for PT fuzzy search
    pt_bktree: BkTree,

    /// Lowercase name to code mappings for exact match
    llt_name_to_code: HashMap<String, u32>,
    pt_name_to_code: HashMap<String, u32>,
}

impl MeddraDictionary {
    /// Create an empty dictionary with specified version.
    #[must_use]
    pub fn new(version: MeddraVersion) -> Self {
        Self {
            version,
            llt_by_code: HashMap::new(),
            pt_by_code: HashMap::new(),
            hlt_by_code: HashMap::new(),
            hlgt_by_code: HashMap::new(),
            soc_by_code: HashMap::new(),
            pt_to_hlts: HashMap::new(),
            hlt_to_hlgts: HashMap::new(),
            hlgt_to_socs: HashMap::new(),
            llt_bktree: BkTree::new(),
            pt_bktree: BkTree::new(),
            llt_name_to_code: HashMap::new(),
            pt_name_to_code: HashMap::new(),
        }
    }

    /// Load LLT terms into dictionary.
    pub fn load_llts(&mut self, llts: Vec<Llt>) {
        for llt in llts {
            let name_lower = llt.name.to_lowercase();
            self.llt_bktree.insert(&name_lower, llt.code);
            self.llt_name_to_code.insert(name_lower, llt.code);
            self.llt_by_code.insert(llt.code, llt);
        }
    }

    /// Load PT terms into dictionary.
    pub fn load_pts(&mut self, pts: Vec<Pt>) {
        for pt in pts {
            let name_lower = pt.name.to_lowercase();
            self.pt_bktree.insert(&name_lower, pt.code);
            self.pt_name_to_code.insert(name_lower, pt.code);
            self.pt_by_code.insert(pt.code, pt);
        }
    }

    /// Load HLT terms into dictionary.
    pub fn load_hlts(&mut self, hlts: Vec<Hlt>) {
        for hlt in hlts {
            self.hlt_by_code.insert(hlt.code, hlt);
        }
    }

    /// Load HLGT terms into dictionary.
    pub fn load_hlgts(&mut self, hlgts: Vec<Hlgt>) {
        for hlgt in hlgts {
            self.hlgt_by_code.insert(hlgt.code, hlgt);
        }
    }

    /// Load SOC terms into dictionary.
    pub fn load_socs(&mut self, socs: Vec<Soc>) {
        for soc in socs {
            self.soc_by_code.insert(soc.code, soc);
        }
    }

    /// Load HLT-PT relationships.
    pub fn load_hlt_pt_relationships(&mut self, relationships: Vec<(u32, u32)>) {
        for (hlt_code, pt_code) in relationships {
            self.pt_to_hlts.entry(pt_code).or_default().push(hlt_code);
        }
    }

    /// Load HLGT-HLT relationships.
    #[allow(clippy::similar_names)] // Standard MedDRA hierarchy names
    pub fn load_hlgt_hlt_relationships(&mut self, relationships: Vec<(u32, u32)>) {
        for (hlgt_code, hlt_code) in relationships {
            self.hlt_to_hlgts
                .entry(hlt_code)
                .or_default()
                .push(hlgt_code);
        }
    }

    /// Load SOC-HLGT relationships.
    pub fn load_soc_hlgt_relationships(&mut self, relationships: Vec<(u32, u32)>) {
        for (soc_code, hlgt_code) in relationships {
            self.hlgt_to_socs
                .entry(hlgt_code)
                .or_default()
                .push(soc_code);
        }
    }

    /// Get LLT by code.
    ///
    /// # Complexity
    ///
    /// - TIME: O(1)
    #[must_use]
    pub fn get_llt(&self, code: u32) -> Option<&Llt> {
        self.llt_by_code.get(&code)
    }

    /// Get PT by code.
    #[must_use]
    pub fn get_pt(&self, code: u32) -> Option<&Pt> {
        self.pt_by_code.get(&code)
    }

    /// Get HLT by code.
    #[must_use]
    pub fn get_hlt(&self, code: u32) -> Option<&Hlt> {
        self.hlt_by_code.get(&code)
    }

    /// Get HLGT by code.
    #[must_use]
    pub fn get_hlgt(&self, code: u32) -> Option<&Hlgt> {
        self.hlgt_by_code.get(&code)
    }

    /// Get SOC by code.
    #[must_use]
    pub fn get_soc(&self, code: u32) -> Option<&Soc> {
        self.soc_by_code.get(&code)
    }

    /// Get full hierarchy path for an LLT code.
    ///
    /// Traverses: LLT -> PT -> HLT -> HLGT -> SOC
    ///
    /// # Errors
    ///
    /// Returns `CodingError::MeddraCodeNotFound` if any level of the hierarchy is missing.
    ///
    /// # Complexity
    ///
    /// - TIME: O(1) - fixed number of hashmap lookups
    #[allow(clippy::similar_names)] // Standard MedDRA hierarchy names
    pub fn get_hierarchy_path(&self, llt_code: u32) -> Result<HierarchyPath, CodingError> {
        let llt = self
            .get_llt(llt_code)
            .ok_or_else(|| CodingError::MeddraCodeNotFound(format!("LLT {llt_code}")))?;

        let pt = self
            .get_pt(llt.pt_code)
            .ok_or_else(|| CodingError::MeddraCodeNotFound(format!("PT {}", llt.pt_code)))?;

        // Get first HLT for this PT (primary path)
        let hlt_code = self
            .pt_to_hlts
            .get(&pt.code)
            .and_then(|hlts| hlts.first())
            .copied()
            .ok_or_else(|| CodingError::MeddraCodeNotFound(format!("HLT for PT {}", pt.code)))?;

        let hlt = self
            .get_hlt(hlt_code)
            .ok_or_else(|| CodingError::MeddraCodeNotFound(format!("HLT {hlt_code}")))?;

        // Get first HLGT for this HLT (primary path)
        let hlgt_code = self
            .hlt_to_hlgts
            .get(&hlt_code)
            .and_then(|hlgts| hlgts.first())
            .copied()
            .ok_or_else(|| CodingError::MeddraCodeNotFound(format!("HLGT for HLT {hlt_code}")))?;

        let hlgt = self
            .get_hlgt(hlgt_code)
            .ok_or_else(|| CodingError::MeddraCodeNotFound(format!("HLGT {hlgt_code}")))?;

        // Get SOC from PT's primary_soc_code (more reliable) or via HLGT
        let soc_code = pt.primary_soc_code;
        let soc = self
            .get_soc(soc_code)
            .ok_or_else(|| CodingError::MeddraCodeNotFound(format!("SOC {soc_code}")))?;

        Ok(HierarchyPath {
            llt_code: llt.code,
            llt_name: llt.name.clone(),
            pt_code: pt.code,
            pt_name: pt.name.clone(),
            hlt_code: hlt.code,
            hlt_name: hlt.name.clone(),
            hlgt_code: hlgt.code,
            hlgt_name: hlgt.name.clone(),
            soc_code: soc.code,
            soc_name: soc.name.clone(),
        })
    }

    /// Search for terms by name (fuzzy match).
    ///
    /// # Parameters
    ///
    /// - `query`: Search string
    /// - `max_distance`: Maximum Levenshtein distance (default: 2)
    /// - `limit`: Maximum results to return
    ///
    /// # Complexity
    ///
    /// - TIME: O(n^(1-1/d)) average using BK-tree, where d is `max_distance`
    #[must_use]
    pub fn search(&self, query: &str, max_distance: usize, limit: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Search LLTs
        for (term, code, distance) in self.llt_bktree.search(&query_lower, max_distance) {
            #[allow(clippy::cast_precision_loss)]
            let distance_f = distance as f64;
            #[allow(clippy::cast_precision_loss)]
            let max_len_f = query_lower.len().max(term.len()) as f64;

            let score = 1.0 - (distance_f / max_len_f);
            // TODO: titration_provenance omitted in fuzzy search hot path
            results.push(SearchResult {
                term,
                code,
                level: HierarchyLevel::Llt,
                score,
                distance,
                titration_provenance: None,
            });
        }

        // Search PTs
        for (term, code, distance) in self.pt_bktree.search(&query_lower, max_distance) {
            #[allow(clippy::cast_precision_loss)]
            let distance_f = distance as f64;
            #[allow(clippy::cast_precision_loss)]
            let max_len_f = query_lower.len().max(term.len()) as f64;

            let score = 1.0 - (distance_f / max_len_f);
            // TODO: titration_provenance omitted in fuzzy search hot path
            results.push(SearchResult {
                term,
                code,
                level: HierarchyLevel::Pt,
                score,
                distance,
                titration_provenance: None,
            });
        }

        // Sort by score (highest first), then by level (PT preferred over LLT)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.level.level_number().cmp(&a.level.level_number()))
        });

        results.truncate(limit);
        results
    }

    /// Encode a term to `MedDRA` (find best match).
    ///
    /// Returns the best matching code at PT level if possible.
    ///
    /// # Errors
    ///
    /// Returns `CodingError::NoMatch` if no term is found within the default fuzzy distance.
    ///
    /// # Complexity
    ///
    /// - TIME: O(1) for exact match, O(n^(1-1/d)) for fuzzy
    pub fn encode(&self, term: &str) -> Result<SearchResult, CodingError> {
        let term_lower = term.to_lowercase();

        // Try exact PT match first
        if let Some(&code) = self.pt_name_to_code.get(&term_lower) {
            if let Some(pt) = self.get_pt(code) {
                return Ok(SearchResult {
                    term: pt.name.clone(),
                    code,
                    level: HierarchyLevel::Pt,
                    score: 1.0,
                    distance: 0,
                    // Exact match — titration is trivially 1.0, omit for efficiency
                    titration_provenance: None,
                });
            }
        }

        // Try exact LLT match
        if let Some(&code) = self.llt_name_to_code.get(&term_lower) {
            if let Some(llt) = self.get_llt(code) {
                return Ok(SearchResult {
                    term: llt.name.clone(),
                    code,
                    level: HierarchyLevel::Llt,
                    score: 1.0,
                    distance: 0,
                    // Exact match — titration is trivially 1.0, omit for efficiency
                    titration_provenance: None,
                });
            }
        }

        // Fall back to fuzzy search
        let results = self.search(term, 2, 1);
        results
            .into_iter()
            .next()
            .ok_or_else(|| CodingError::NoMatch(term.to_string()))
    }

    /// Get statistics about loaded dictionary.
    #[must_use]
    pub fn stats(&self) -> DictionaryStats {
        DictionaryStats {
            llt_count: self.llt_by_code.len(),
            pt_count: self.pt_by_code.len(),
            hlt_count: self.hlt_by_code.len(),
            hlgt_count: self.hlgt_by_code.len(),
            soc_count: self.soc_by_code.len(),
        }
    }
}

/// Dictionary statistics.
#[derive(Debug, Clone, Copy)]
pub struct DictionaryStats {
    /// Number of LLT terms
    pub llt_count: usize,
    /// Number of PT terms
    pub pt_count: usize,
    /// Number of HLT terms
    pub hlt_count: usize,
    /// Number of HLGT terms
    pub hlgt_count: usize,
    /// Number of SOC terms
    pub soc_count: usize,
}

impl DictionaryStats {
    /// Total number of terms across all levels.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.llt_count + self.pt_count + self.hlt_count + self.hlgt_count + self.soc_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dictionary() -> MeddraDictionary {
        let mut dict = MeddraDictionary::new(MeddraVersion::new(26, 1, "English"));

        // Add test data
        dict.load_llts(vec![
            Llt {
                code: 10019211,
                name: "Headache".to_string(),
                pt_code: 10019231,
                is_current: true,
            },
            Llt {
                code: 10019212,
                name: "Head pain".to_string(),
                pt_code: 10019231,
                is_current: true,
            },
        ]);

        dict.load_pts(vec![Pt {
            code: 10019231,
            name: "Headache".to_string(),
            primary_soc_code: 10029205,
        }]);

        dict.load_hlts(vec![Hlt {
            code: 10019233,
            name: "Headaches".to_string(),
        }]);

        dict.load_hlgts(vec![Hlgt {
            code: 10019234,
            name: "Headaches NEC".to_string(),
        }]);

        dict.load_socs(vec![Soc {
            code: 10029205,
            name: "Nervous system disorders".to_string(),
            abbrev: "Nerv".to_string(),
            intl_order: 1,
        }]);

        // Load relationships
        dict.load_hlt_pt_relationships(vec![(10019233, 10019231)]);
        dict.load_hlgt_hlt_relationships(vec![(10019234, 10019233)]);
        dict.load_soc_hlgt_relationships(vec![(10029205, 10019234)]);

        dict
    }

    #[test]
    fn test_get_llt() {
        let dict = create_test_dictionary();
        let llt = dict.get_llt(10019211).expect("LLT not found");
        assert_eq!(llt.name, "Headache");
        assert_eq!(llt.pt_code, 10019231);
    }

    #[test]
    fn test_get_hierarchy_path() {
        let dict = create_test_dictionary();
        let path = dict.get_hierarchy_path(10019211).expect("path failed");

        assert_eq!(path.llt_name, "Headache");
        assert_eq!(path.pt_name, "Headache");
        assert_eq!(path.hlt_name, "Headaches");
        assert_eq!(path.hlgt_name, "Headaches NEC");
        assert_eq!(path.soc_name, "Nervous system disorders");
    }

    #[test]
    fn test_exact_encode() {
        let dict = create_test_dictionary();
        let result = dict.encode("Headache").expect("encode failed");

        assert_eq!(result.code, 10019231); // PT code
        assert_eq!(result.level, HierarchyLevel::Pt);
        assert!((result.score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fuzzy_search() {
        let dict = create_test_dictionary();
        let results = dict.search("headahce", 2, 10); // typo

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.term.to_lowercase() == "headache"));
    }

    #[test]
    fn test_stats() {
        let dict = create_test_dictionary();
        let stats = dict.stats();

        assert_eq!(stats.llt_count, 2);
        assert_eq!(stats.pt_count, 1);
        assert_eq!(stats.hlt_count, 1);
        assert_eq!(stats.hlgt_count, 1);
        assert_eq!(stats.soc_count, 1);
        assert_eq!(stats.total(), 6);
    }
}
