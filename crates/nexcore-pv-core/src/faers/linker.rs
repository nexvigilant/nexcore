//! FAERS record linking across files.
//!
//! Links demographics, drugs, reactions, and outcomes by primary ID.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Linked FAERS report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedReport {
    /// Primary ID
    pub primary_id: String,
    /// Case version
    pub case_version: u32,
    /// Age in years
    pub age_years: Option<f64>,
    /// Age group
    pub age_group: Option<String>,
    /// Sex (M, F, UNK)
    pub sex: Option<String>,
    /// Weight (kg)
    pub weight_kg: Option<f64>,
    /// Initial/Followup code
    pub i_f_code: Option<String>,
    /// Reporter country
    pub reporter_country: Option<String>,
    /// Occurrence country
    pub occr_country: Option<String>,
    /// Event date (YYYYMMDD)
    pub event_dt: Option<String>,
    /// FDA receive date
    pub fda_dt: Option<String>,
    /// Manufacturer/sender
    pub mfr_sndr: Option<String>,
    /// Occupation code
    pub occp_cod: Option<String>,
    /// Manufacturer control number (for deduplication)
    pub mfr_num: Option<String>,
    /// Drugs (name, role, drug_seq)
    pub drugs: Vec<(String, String, u32)>,
    /// Reactions (PT)
    pub reactions: Vec<String>,
    /// Outcomes (DE, LT, etc.)
    pub outcomes: Vec<String>,
    /// Indications (indi_pt, drug_seq)
    pub indications: Vec<(String, u32)>,
    /// Therapy (start_dt, end_dt, drug_seq)
    pub therapy: Vec<(Option<String>, Option<String>, u32)>,
    /// Report sources (RPSR codes)
    pub report_sources: Vec<String>,
}

impl LinkedReport {
    /// Create new linked report
    #[must_use]
    pub fn new(primary_id: String) -> Self {
        Self {
            primary_id,
            case_version: 0,
            age_years: None,
            age_group: None,
            sex: None,
            weight_kg: None,
            i_f_code: None,
            reporter_country: None,
            occr_country: None,
            event_dt: None,
            fda_dt: None,
            mfr_sndr: None,
            occp_cod: None,
            mfr_num: None,
            drugs: vec![],
            reactions: vec![],
            outcomes: vec![],
            indications: vec![],
            therapy: vec![],
            report_sources: vec![],
        }
    }

    /// Check if report has a specific drug
    #[must_use]
    pub fn has_drug(&self, name: &str) -> bool {
        self.drugs
            .iter()
            .any(|(d, _, _)| d.eq_ignore_ascii_case(name))
    }

    /// Check if report has a specific reaction
    #[must_use]
    pub fn has_reaction(&self, pt: &str) -> bool {
        self.reactions.iter().any(|r| r.eq_ignore_ascii_case(pt))
    }

    /// Get suspect drugs only
    #[must_use]
    pub fn suspect_drugs(&self) -> Vec<&str> {
        self.drugs
            .iter()
            .filter(|(_, role, _)| role == "PS" || role == "SS")
            .map(|(d, _, _)| d.as_str())
            .collect()
    }
}

/// Linker for building linked reports
#[derive(Debug, Clone, Default)]
pub struct FaersLinker {
    reports: HashMap<String, LinkedReport>,
}

impl FaersLinker {
    /// Create new linker
    #[must_use]
    pub fn new() -> Self {
        Self {
            reports: HashMap::new(),
        }
    }

    /// Get or create report
    fn get_or_create(&mut self, id: &str) -> &mut LinkedReport {
        self.reports
            .entry(id.into())
            .or_insert_with(|| LinkedReport::new(id.into()))
    }

    /// Add statistics to the linker (to be implemented if needed)

    /// Add demographics from FAERSDemo
    pub fn add_faers_demo(&mut self, demo: &super::FAERSDemo) {
        let r = self.get_or_create(&demo.primaryid.to_string());
        r.age_years = demo
            .age
            .and_then(|a| demo.age_cod.as_ref().map(|c| super::normalize_age(a, c)));
        r.age_group = demo.age_grp.clone();
        r.sex = demo.sex.clone();
        r.weight_kg = demo.wt;
        r.i_f_code = Some(demo.i_f_code.clone());
        r.reporter_country = demo.reporter_country.clone();
        r.occr_country = demo.occr_country.clone();
        r.event_dt = demo.event_dt.clone();
        r.fda_dt = demo.fda_dt.clone();
        r.mfr_sndr = demo.mfr_sndr.clone();
        r.occp_cod = demo.occp_cod.clone();
        r.mfr_num = demo.mfr_num.clone();
        r.case_version = demo.caseversion;
    }

    /// Add demographics (legacy method - kept for backward compatibility)
    pub fn add_demo(
        &mut self,
        id: &str,
        age: Option<f64>,
        sex: Option<&str>,
        wt: Option<f64>,
        country: Option<&str>,
    ) {
        let r = self.get_or_create(id);
        r.age_years = age;
        r.sex = sex.map(String::from);
        r.weight_kg = wt;
        r.occr_country = country.map(String::from);
    }

    /// Add drug
    pub fn add_drug(&mut self, id: &str, name: &str, role: &str, seq: u32) {
        let r = self.get_or_create(id);
        r.drugs
            .push((name.to_uppercase(), role.to_uppercase(), seq));
    }

    /// Add reaction
    pub fn add_reaction(&mut self, id: &str, pt: &str) {
        let r = self.get_or_create(id);
        if !r.reactions.contains(&pt.to_uppercase()) {
            r.reactions.push(pt.to_uppercase());
        }
    }

    /// Add outcome
    pub fn add_outcome(&mut self, id: &str, code: &str) {
        let r = self.get_or_create(id);
        if !r.outcomes.contains(&code.to_uppercase()) {
            r.outcomes.push(code.to_uppercase());
        }
    }

    /// Add indication
    pub fn add_indication(&mut self, id: &str, ind: &str, drug_seq: u32) {
        let r = self.get_or_create(id);
        r.indications.push((ind.to_uppercase(), drug_seq));
    }

    /// Add therapy
    pub fn add_therapy(
        &mut self,
        id: &str,
        start_dt: Option<String>,
        end_dt: Option<String>,
        drug_seq: u32,
    ) {
        let r = self.get_or_create(id);
        r.therapy.push((start_dt, end_dt, drug_seq));
    }

    /// Add report source
    pub fn add_rpsr(&mut self, id: &str, code: &str) {
        let r = self.get_or_create(id);
        if !r.report_sources.contains(&code.to_uppercase()) {
            r.report_sources.push(code.to_uppercase());
        }
    }

    /// Remove specific reports (e.g., retracted IDs)
    pub fn remove_reports(&mut self, ids: &[u64]) {
        for id in ids {
            self.reports.remove(&id.to_string());
        }
    }

    /// Get all reports
    #[must_use]
    pub fn reports(&self) -> Vec<&LinkedReport> {
        self.reports.values().collect()
    }

    /// Get reports with drug-event pair
    #[must_use]
    pub fn filter_drug_event(&self, drug: &str, event: &str) -> Vec<&LinkedReport> {
        self.reports
            .values()
            .filter(|r| r.has_drug(drug) && r.has_reaction(event))
            .collect()
    }

    /// Get report count
    #[must_use]
    pub fn len(&self) -> usize {
        self.reports.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }

    /// Consume and return all reports
    #[must_use]
    pub fn into_reports(self) -> Vec<LinkedReport> {
        self.reports.into_values().collect()
    }
}

/// Build contingency table from linked reports
#[must_use]
pub fn build_contingency(
    reports: &[LinkedReport],
    drug: &str,
    event: &str,
) -> crate::types::ContingencyTable {
    let mut a = 0;
    let mut b = 0;
    let mut c = 0;
    let mut d = 0;

    for r in reports {
        match (r.has_drug(drug), r.has_reaction(event)) {
            (true, true) => a += 1,
            (true, false) => b += 1,
            (false, true) => c += 1,
            (false, false) => d += 1,
        }
    }

    crate::types::ContingencyTable::new(a, b, c, d)
}

/// Build contingency tables for multiple pairs in parallel
#[must_use]
pub fn build_contingency_parallel(
    reports: &[LinkedReport],
    pairs: &[(String, String)],
) -> Vec<crate::types::ContingencyTable> {
    pairs
        .par_iter()
        .map(|(d, e)| build_contingency(reports, d, e))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker() {
        let mut l = FaersLinker::new();
        l.add_demo("1", Some(45.0), Some("M"), Some(80.0), Some("US"));
        l.add_drug("1", "Aspirin", "PS", 1);
        l.add_reaction("1", "Headache");
        l.add_outcome("1", "DE");
        assert_eq!(l.len(), 1);
        let r = l.reports();
        assert!(r[0].has_drug("ASPIRIN"));
        assert!(r[0].has_reaction("HEADACHE"));
    }

    #[test]
    fn test_filter() {
        let mut l = FaersLinker::new();
        l.add_drug("1", "Aspirin", "PS", 1);
        l.add_reaction("1", "Headache");
        l.add_drug("2", "Ibuprofen", "PS", 1);
        l.add_reaction("2", "Nausea");
        let f = l.filter_drug_event("Aspirin", "Headache");
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn test_contingency() {
        let reports = vec![
            {
                let mut r = LinkedReport::new("1".into());
                r.drugs.push(("A".into(), "PS".into(), 1));
                r.reactions.push("X".into());
                r
            },
            {
                let mut r = LinkedReport::new("2".into());
                r.drugs.push(("A".into(), "PS".into(), 1));
                r.reactions.push("Y".into());
                r
            },
            {
                let mut r = LinkedReport::new("3".into());
                r.drugs.push(("B".into(), "PS".into(), 1));
                r.reactions.push("X".into());
                r
            },
        ];
        let t = build_contingency(&reports, "A", "X");
        assert_eq!(t.a, 1);
        assert_eq!(t.b, 1);
        assert_eq!(t.c, 1);
    }
}
