//! # FAERS Module
//!
//! FDA Adverse Event Reporting System data processing.
//!
//! ## Submodules
//!
//! - **types** - Core enums (DrugRole, OutcomeCode, AgeUnit)
//! - **parser** - Field parsing utilities
//! - **demo** - Demographics file parsing
//! - **drug** - Drug file parsing
//! - **reaction** - Reaction file parsing
//! - **outcome** - Outcome file parsing
//! - **indication** - Indication file parsing
//! - **therapy** - Therapy file parsing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod counter;
pub mod delete;
pub mod demo;
pub mod drug;
pub mod indication;
pub mod linker;
pub mod outcome;
pub mod parser;
pub mod pipeline;
pub mod reaction;
pub mod rpsr;
pub mod therapy;
pub mod types;
pub mod validation;

// Re-export types
pub use types::{AgeUnit, DrugRole, OutcomeCode};

// Re-export delete support
pub use delete::{FAERSDelete, load_deleted_ids};

// Re-export counter (P2)
pub use counter::{
    AgeAdjustedCount, BatchCountResult, StratifiedCounts, StratumKey, age_adjust, batch_count,
    count_pairs_parallel,
};

// Re-export linker (P2)
pub use linker::{FaersLinker, LinkedReport, build_contingency, build_contingency_parallel};

// Re-export entity structs
pub use demo::{FAERSDemo, parse_demo_line};
pub use drug::{FAERSDrug, parse_drug_line};
pub use indication::{FAERSIndication, parse_indication_line};
pub use outcome::{FAERSOutcome, parse_outcome_line};
pub use reaction::{FAERSReaction, parse_reaction_line};
pub use rpsr::{FAERSRpsr, parse_rpsr_line};
pub use therapy::{FAERSTherapy, parse_therapy_line};

// Re-export parser utilities
pub use parser::{
    get_field, is_suspect_drug, normalize_age_to_years, normalize_drug_name, parse_faers_date,
    parse_float_field, parse_int_field, parse_role_code, parse_str_field, split_faers_line,
};

// Re-export pipeline types
pub use pipeline::{
    DisproportionalityResult, DrugEventSignal, FaersPipelineConfig, FaersPipelineConfigBuilder,
    FaersPipelineResult, SignalStrength,
};

// Re-export validation types
pub use validation::{
    ValidationCategory, ValidationIssue, ValidationResult, ValidationSeverity, ValidationSummary,
    validate_age, validate_contingency_table, validate_country_code, validate_drug_role,
    validate_sex, validate_weight,
};

/// Drug-event pair count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEventCount {
    /// Drug name
    pub drug: String,
    /// Event (MedDRA PT)
    pub event: String,
    /// Count of co-occurrences
    pub count: u32,
}

/// FAERS contingency counts for signal detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaersContingencyCounts {
    /// Drug-event pair counts keyed by (drug, event)
    pub pair_counts: HashMap<(String, String), u32>,
    /// Total drug counts
    pub drug_totals: HashMap<String, u32>,
    /// Total event counts
    pub event_totals: HashMap<String, u32>,
    /// Total report count
    pub total_reports: u32,
}

impl FaersContingencyCounts {
    /// Create new empty counts
    #[must_use]
    pub fn new() -> Self {
        Self {
            pair_counts: HashMap::new(),
            drug_totals: HashMap::new(),
            event_totals: HashMap::new(),
            total_reports: 0,
        }
    }

    /// Add a drug-event pair
    pub fn add_pair(&mut self, drug: &str, event: &str) {
        let key = (drug.to_string(), event.to_string());
        *self.pair_counts.entry(key).or_insert(0) += 1;

        // Update totals
        *self.drug_totals.entry(drug.to_string()).or_insert(0) += 1;
        *self.event_totals.entry(event.to_string()).or_insert(0) += 1;
        self.total_reports += 1;
    }

    /// Get contingency table for a drug-event pair
    #[must_use]
    pub fn get_contingency(
        &self,
        drug: &str,
        event: &str,
    ) -> Option<crate::types::ContingencyTable> {
        let a = *self
            .pair_counts
            .get(&(drug.to_string(), event.to_string()))?;
        let drug_total = *self.drug_totals.get(drug)?;
        let event_total = *self.event_totals.get(event)?;

        let b = drug_total - a;
        let c = event_total - a;
        let d = self.total_reports.saturating_sub(a + b + c);

        Some(crate::types::ContingencyTable::new(
            a.into(),
            b.into(),
            c.into(),
            d.into(),
        ))
    }
}

impl Default for FaersContingencyCounts {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize age to years
#[must_use]
pub fn normalize_age(value: f64, unit: &str) -> f64 {
    match unit.to_uppercase().as_str() {
        "YR" | "YEAR" | "YEARS" => value,
        "MON" | "MONTH" | "MONTHS" => value / 12.0,
        "WK" | "WEEK" | "WEEKS" => value / 52.0,
        "DY" | "DAY" | "DAYS" => value / 365.0,
        "DEC" | "DECADE" | "DECADES" => value * 10.0,
        _ => value,
    }
}

/// Get age group from age in years
#[must_use]
pub fn age_group(age_years: f64) -> &'static str {
    match age_years as u32 {
        0..=1 => "Infant",
        2..=12 => "Child",
        13..=17 => "Adolescent",
        18..=64 => "Adult",
        _ => "Elderly",
    }
}

// =============================================================================
// Batch Quarterly Parsing (P4)
// =============================================================================

use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Result of parsing a FAERS quarterly release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyParseResult {
    /// Year (e.g., 2024)
    pub year: u32,
    /// Quarter (1-4)
    pub quarter: u8,
    /// Linked reports
    pub report_count: usize,
    /// Drug count
    pub drug_count: usize,
    /// Reaction count
    pub reaction_count: usize,
    /// Parse errors
    pub errors: Vec<String>,
}

/// Parse a single FAERS file with field-based parser.
fn parse_faers_file_fields<P, F, T>(
    path: P,
    mut parser: F,
    skip_header: bool,
) -> (Vec<T>, Vec<String>)
where
    P: AsRef<Path>,
    F: FnMut(&[&str]) -> T,
{
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let file = match File::open(path.as_ref()) {
        Ok(f) => f,
        Err(e) => {
            errors.push(format!("Failed to open file: {e}"));
            return (results, errors);
        }
    };
    let reader = BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        if skip_header && i == 0 {
            continue;
        }
        match line {
            Ok(l) => {
                let fields = split_faers_line(&l);
                results.push(parser(&fields));
            }
            Err(e) => errors.push(format!("Line {i}: {e}")),
        }
    }
    (results, errors)
}

/// Find a `.txt` file in directory matching prefix (case-insensitive).
fn find_faers_file(files: &[std::path::PathBuf], prefix: &str) -> Option<std::path::PathBuf> {
    files
        .iter()
        .find(|p| {
            p.file_name()
                .map(|n| {
                    let name = n.to_string_lossy();
                    let upper = name.to_uppercase();
                    upper.starts_with(prefix) && upper.ends_with(".TXT")
                })
                .unwrap_or(false)
        })
        .cloned()
}

/// Parse a quarterly FAERS release from a directory.
///
/// Expects standard FAERS file naming: DEMO*.txt, DRUG*.txt, REAC*.txt, etc.
pub fn parse_quarterly<P: AsRef<Path>>(dir: P, year: u32, quarter: u8) -> QuarterlyParseResult {
    let dir = dir.as_ref();
    let mut linker = FaersLinker::new();
    let mut errors = Vec::new();

    let files: Vec<_> = std::fs::read_dir(dir)
        .map(|r| r.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();

    // Parse demographics
    if let Some(path) = find_faers_file(&files, "DEMO") {
        let (demos, errs) = parse_faers_file_fields(&path, FAERSDemo::parse, true);
        errors.extend(errs);
        for d in demos {
            linker.add_faers_demo(&d);
        }
    }

    // Parse drugs
    if let Some(path) = find_faers_file(&files, "DRUG") {
        let (drugs, errs) = parse_faers_file_fields(&path, FAERSDrug::parse, true);
        errors.extend(errs);
        for d in drugs {
            linker.add_drug(
                &d.primaryid.to_string(),
                &d.drugname,
                &d.role_cod,
                d.drug_seq,
            );
        }
    }

    // Parse reactions
    if let Some(path) = find_faers_file(&files, "REAC") {
        let (reactions, errs) = parse_faers_file_fields(&path, FAERSReaction::parse, true);
        errors.extend(errs);
        for r in reactions {
            linker.add_reaction(&r.primaryid.to_string(), &r.pt);
        }
    }

    // Parse outcomes
    if let Some(path) = find_faers_file(&files, "OUTC") {
        let (outcomes, errs) = parse_faers_file_fields(&path, FAERSOutcome::parse, true);
        errors.extend(errs);
        for o in outcomes {
            linker.add_outcome(&o.primaryid.to_string(), &o.outc_cod);
        }
    }

    // Parse indications
    if let Some(path) = find_faers_file(&files, "INDI") {
        let (indications, errs) = parse_faers_file_fields(&path, FAERSIndication::parse, true);
        errors.extend(errs);
        for i in indications {
            linker.add_indication(&i.primaryid.to_string(), &i.indi_pt, i.indi_drug_seq);
        }
    }

    // Parse therapy
    if let Some(path) = find_faers_file(&files, "THER") {
        let (therapies, errs) = parse_faers_file_fields(&path, FAERSTherapy::parse, true);
        errors.extend(errs);
        for t in therapies {
            linker.add_therapy(
                &t.primaryid.to_string(),
                t.start_dt,
                t.end_dt,
                t.dsg_drug_seq,
            );
        }
    }

    // Parse report source
    if let Some(path) = find_faers_file(&files, "RPSR") {
        let (sources, errs) = parse_faers_file_fields(&path, FAERSRpsr::parse, true);
        errors.extend(errs);
        for s in sources {
            linker.add_rpsr(&s.primaryid.to_string(), &s.rpsr_cod);
        }
    }

    let reports = linker.reports();
    let drug_count: usize = reports.iter().map(|r| r.drugs.len()).sum();
    let reaction_count: usize = reports.iter().map(|r| r.reactions.len()).sum();

    QuarterlyParseResult {
        year,
        quarter,
        report_count: reports.len(),
        drug_count,
        reaction_count,
        errors,
    }
}

/// Batch parse multiple quarterly releases in parallel.
pub fn batch_parse_quarterly(
    quarters: &[(std::path::PathBuf, u32, u8)],
) -> Vec<QuarterlyParseResult> {
    quarters
        .par_iter()
        .map(|(dir, year, quarter)| parse_quarterly(dir, *year, *quarter))
        .collect()
}

/// Parse quarterly and build linked reports.
pub fn parse_quarterly_linked<P: AsRef<Path>>(dir: P) -> (Vec<LinkedReport>, Vec<String>) {
    let dir = dir.as_ref();
    let mut linker = FaersLinker::new();
    let mut errors = Vec::new();

    let files: Vec<_> = std::fs::read_dir(dir)
        .map(|r| r.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();

    if let Some(p) = find_faers_file(&files, "DEMO") {
        let (d, e) = parse_faers_file_fields(&p, FAERSDemo::parse, true);
        errors.extend(e);
        for x in d {
            linker.add_faers_demo(&x);
        }
    }
    if let Some(p) = find_faers_file(&files, "DRUG") {
        let (d, e) = parse_faers_file_fields(&p, FAERSDrug::parse, true);
        errors.extend(e);
        for x in d {
            linker.add_drug(
                &x.primaryid.to_string(),
                &x.drugname,
                &x.role_cod,
                x.drug_seq,
            );
        }
    }
    if let Some(p) = find_faers_file(&files, "REAC") {
        let (d, e) = parse_faers_file_fields(&p, FAERSReaction::parse, true);
        errors.extend(e);
        for x in d {
            linker.add_reaction(&x.primaryid.to_string(), &x.pt);
        }
    }
    if let Some(p) = find_faers_file(&files, "OUTC") {
        let (d, e) = parse_faers_file_fields(&p, FAERSOutcome::parse, true);
        errors.extend(e);
        for x in d {
            linker.add_outcome(&x.primaryid.to_string(), &x.outc_cod);
        }
    }
    if let Some(p) = find_faers_file(&files, "INDI") {
        let (d, e) = parse_faers_file_fields(&p, FAERSIndication::parse, true);
        errors.extend(e);
        for x in d {
            linker.add_indication(&x.primaryid.to_string(), &x.indi_pt, x.indi_drug_seq);
        }
    }
    if let Some(p) = find_faers_file(&files, "THER") {
        let (d, e) = parse_faers_file_fields(&p, FAERSTherapy::parse, true);
        errors.extend(e);
        for x in d {
            linker.add_therapy(
                &x.primaryid.to_string(),
                x.start_dt,
                x.end_dt,
                x.dsg_drug_seq,
            );
        }
    }
    if let Some(p) = find_faers_file(&files, "RPSR") {
        let (d, e) = parse_faers_file_fields(&p, FAERSRpsr::parse, true);
        errors.extend(e);
        for x in d {
            linker.add_rpsr(&x.primaryid.to_string(), &x.rpsr_cod);
        }
    }

    // Handle Deleted records
    let mut deleted_ids = Vec::new();
    // Check current dir for DELETE file
    if let Some(p) = find_faers_file(&files, "DELETE") {
        let ids = load_deleted_ids(&p);
        deleted_ids.extend(ids);
    }
    // Check sibling 'Deleted' directory if current dir is 'ASCII'
    if dir.file_name().and_then(|n| n.to_str()) == Some("ASCII") {
        if let Some(parent) = dir.parent() {
            let deleted_dir = parent.join("Deleted");
            if deleted_dir.exists() {
                if let Ok(rd) = std::fs::read_dir(deleted_dir) {
                    let d_files: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
                    if let Some(p) = find_faers_file(&d_files, "DELETE") {
                        let ids = load_deleted_ids(&p);
                        deleted_ids.extend(ids);
                    }
                }
            }
        }
    }

    if !deleted_ids.is_empty() {
        linker.remove_reports(&deleted_ids);
    }

    (linker.into_reports(), errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_age() {
        assert!((normalize_age(24.0, "MON") - 2.0).abs() < 0.01);
        assert!((normalize_age(365.0, "DAY") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_age_group() {
        assert_eq!(age_group(0.5), "Infant");
        assert_eq!(age_group(10.0), "Child");
        assert_eq!(age_group(30.0), "Adult");
        assert_eq!(age_group(75.0), "Elderly");
    }

    #[test]
    fn test_contingency_counts() {
        let mut counts = FaersContingencyCounts::new();
        counts.add_pair("Aspirin", "Headache");
        counts.add_pair("Aspirin", "Headache");
        counts.add_pair("Aspirin", "Nausea");
        counts.add_pair("Ibuprofen", "Headache");

        let table = counts.get_contingency("Aspirin", "Headache");
        assert!(table.is_some());
        assert_eq!(table.map(|t| t.a), Some(2));
    }
}
