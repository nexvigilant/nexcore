//! MedDRA hierarchy operations.
//!
//! PT to SOC mapping, multi-axiality handling, and SMQ queries.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// MedDRA term with full hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedDRATerm {
    /// Term code
    pub code: u32,
    /// Term name
    pub name: String,
    /// Preferred Term code (for LLT)
    pub pt_code: Option<u32>,
    /// High Level Term codes
    pub hlt_codes: Vec<u32>,
    /// High Level Group Term codes
    pub hlgt_codes: Vec<u32>,
    /// System Organ Class codes
    pub soc_codes: Vec<u32>,
    /// Primary SOC code
    pub primary_soc: Option<u32>,
}

impl MedDRATerm {
    /// Create PT with SOC
    #[must_use]
    pub fn pt(code: u32, name: &str, soc: u32) -> Self {
        Self {
            code,
            name: name.into(),
            pt_code: Some(code),
            hlt_codes: vec![],
            hlgt_codes: vec![],
            soc_codes: vec![soc],
            primary_soc: Some(soc),
        }
    }
    /// Is multi-axial (multiple SOCs)
    #[must_use]
    pub fn is_multiaxial(&self) -> bool {
        self.soc_codes.len() > 1
    }
}

/// MedDRA hierarchy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MedDRAHierarchy {
    /// PT to term map
    pts: HashMap<u32, MedDRATerm>,
    /// LLT to PT map
    llt_to_pt: HashMap<u32, u32>,
    /// Name to code map (lowercase)
    name_to_code: HashMap<String, u32>,
    /// SOC names
    soc_names: HashMap<u32, String>,
}

impl MedDRAHierarchy {
    /// Create new hierarchy
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a PT
    pub fn add_pt(&mut self, term: MedDRATerm) {
        let name_lower = term.name.to_lowercase();
        self.name_to_code.insert(name_lower, term.code);
        self.pts.insert(term.code, term);
    }

    /// Add LLT to PT mapping
    pub fn add_llt(&mut self, llt_code: u32, pt_code: u32) {
        self.llt_to_pt.insert(llt_code, pt_code);
    }

    /// Add SOC name
    pub fn add_soc(&mut self, code: u32, name: &str) {
        self.soc_names.insert(code, name.into());
    }

    /// Get PT by code
    #[must_use]
    pub fn get_pt(&self, code: u32) -> Option<&MedDRATerm> {
        self.pts.get(&code)
    }

    /// Get PT for LLT
    #[must_use]
    pub fn llt_to_pt(&self, llt_code: u32) -> Option<u32> {
        self.llt_to_pt.get(&llt_code).copied()
    }

    /// Get all SOCs for a PT
    #[must_use]
    pub fn get_socs(&self, pt_code: u32) -> Vec<u32> {
        self.pts
            .get(&pt_code)
            .map(|t| t.soc_codes.clone())
            .unwrap_or_default()
    }

    /// Get primary SOC for a PT
    #[must_use]
    pub fn get_primary_soc(&self, pt_code: u32) -> Option<u32> {
        self.pts.get(&pt_code).and_then(|t| t.primary_soc)
    }

    /// Find PT by name (case-insensitive)
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&MedDRATerm> {
        self.name_to_code
            .get(&name.to_lowercase())
            .and_then(|c| self.pts.get(c))
    }

    /// Get SOC name
    #[must_use]
    pub fn soc_name(&self, code: u32) -> Option<&str> {
        self.soc_names.get(&code).map(String::as_str)
    }

    /// Count PTs
    #[must_use]
    pub fn pt_count(&self) -> usize {
        self.pts.len()
    }
}

/// Standardised MedDRA Query (SMQ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMQ {
    /// SMQ code
    pub code: u32,
    /// SMQ name
    pub name: String,
    /// PT codes (narrow scope)
    pub narrow: HashSet<u32>,
    /// PT codes (broad scope)
    pub broad: HashSet<u32>,
    /// Child SMQ codes
    pub children: Vec<u32>,
    /// Is algorithmic
    pub algorithmic: bool,
}

impl SMQ {
    /// Create new SMQ
    #[must_use]
    pub fn new(code: u32, name: &str) -> Self {
        Self {
            code,
            name: name.into(),
            narrow: HashSet::new(),
            broad: HashSet::new(),
            children: vec![],
            algorithmic: false,
        }
    }

    /// Add narrow term
    pub fn add_narrow(&mut self, pt: u32) {
        self.narrow.insert(pt);
    }

    /// Add broad term
    pub fn add_broad(&mut self, pt: u32) {
        self.broad.insert(pt);
    }

    /// Check if PT matches (narrow)
    #[must_use]
    pub fn matches_narrow(&self, pt: u32) -> bool {
        self.narrow.contains(&pt)
    }

    /// Check if PT matches (broad)
    #[must_use]
    pub fn matches_broad(&self, pt: u32) -> bool {
        self.broad.contains(&pt) || self.narrow.contains(&pt)
    }

    /// Get all PTs (narrow + broad)
    #[must_use]
    pub fn all_pts(&self) -> HashSet<u32> {
        self.narrow.union(&self.broad).copied().collect()
    }
}

/// SMQ registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SMQRegistry {
    smqs: HashMap<u32, SMQ>,
    name_to_code: HashMap<String, u32>,
}

impl SMQRegistry {
    /// Create new registry
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add SMQ
    pub fn add(&mut self, smq: SMQ) {
        self.name_to_code.insert(smq.name.to_lowercase(), smq.code);
        self.smqs.insert(smq.code, smq);
    }

    /// Get SMQ by code
    #[must_use]
    pub fn get(&self, code: u32) -> Option<&SMQ> {
        self.smqs.get(&code)
    }

    /// Find by name
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&SMQ> {
        self.name_to_code
            .get(&name.to_lowercase())
            .and_then(|c| self.smqs.get(c))
    }

    /// Get SMQs matching a PT (parallel)
    #[must_use]
    pub fn matching_smqs(&self, pt: u32, broad: bool) -> Vec<&SMQ> {
        self.smqs
            .values()
            .filter(|s| {
                if broad {
                    s.matches_broad(pt)
                } else {
                    s.matches_narrow(pt)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy() {
        let mut h = MedDRAHierarchy::new();
        h.add_pt(MedDRATerm::pt(10019211, "Headache", 10029205));
        h.add_soc(10029205, "Nervous system disorders");
        assert!(h.find_by_name("headache").is_some());
        assert_eq!(h.get_primary_soc(10019211), Some(10029205));
    }

    #[test]
    fn test_smq() {
        let mut smq = SMQ::new(20000001, "Test SMQ");
        smq.add_narrow(10019211);
        smq.add_broad(10033371);
        assert!(smq.matches_narrow(10019211));
        assert!(smq.matches_broad(10033371));
        assert!(!smq.matches_narrow(10033371));
    }

    #[test]
    fn test_registry() {
        let mut reg = SMQRegistry::new();
        let mut smq = SMQ::new(1, "Hepatic disorders");
        smq.add_narrow(100);
        reg.add(smq);
        assert!(reg.find("Hepatic disorders").is_some());
    }
}
