//! Shared EMA GVP module data and helper functions.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GvpModule {
    pub code: &'static str,
    pub title: &'static str,
    pub status: &'static str,
    pub note: &'static str,
    pub pathway: &'static str,
    pub ksb_domains: &'static [&'static str],
}

pub const EMA_GVP_URL: &str = "https://www.ema.europa.eu/en/human-regulatory-overview/post-authorisation/pharmacovigilance-post-authorisation/good-pharmacovigilance-practices-gvp";
pub const GUARDIAN_EVIDENCE_STORAGE_KEY: &str = "academy_guardian_writeback_v1";
pub const GVP_ASSESSMENT_STORAGE_KEY: &str = "academy_gvp_assessment_pass_v1";

pub const GVP_MODULES: [GvpModule; 16] = [
    GvpModule {
        code: "I",
        title: "Pharmacovigilance systems and their quality systems",
        status: "Final",
        note: "Core PV quality framework.",
        pathway: "PV Governance Foundations",
        ksb_domains: &["D01", "D02", "D05"],
    },
    GvpModule {
        code: "II",
        title: "Pharmacovigilance system master file",
        status: "Final",
        note: "PSMF structure and governance.",
        pathway: "PV Governance Foundations",
        ksb_domains: &["D02", "D04", "D06"],
    },
    GvpModule {
        code: "III",
        title: "Pharmacovigilance inspections",
        status: "Final",
        note: "Inspection readiness and conduct.",
        pathway: "Inspection Readiness",
        ksb_domains: &["D04", "D06", "D11"],
    },
    GvpModule {
        code: "IV",
        title: "Pharmacovigilance audits",
        status: "Final",
        note: "Audit strategy and lifecycle.",
        pathway: "Inspection Readiness",
        ksb_domains: &["D04", "D05", "D11"],
    },
    GvpModule {
        code: "V",
        title: "Risk management systems",
        status: "Final",
        note: "RMP planning and updates.",
        pathway: "Risk & Benefit Management",
        ksb_domains: &["D07", "D10", "D12"],
    },
    GvpModule {
        code: "VI",
        title: "Collection, management and submission of reports of suspected adverse reactions",
        status: "Final",
        note: "ICSR handling and EudraVigilance submission.",
        pathway: "Case Processing Excellence",
        ksb_domains: &["D03", "D06", "D08"],
    },
    GvpModule {
        code: "VII",
        title: "Periodic safety update report",
        status: "Final",
        note: "PSUR/PBRER requirements.",
        pathway: "Regulatory Reporting",
        ksb_domains: &["D04", "D09", "D10"],
    },
    GvpModule {
        code: "VIII",
        title: "Post-authorisation safety studies",
        status: "Final",
        note: "PASS design, conduct and reporting.",
        pathway: "Evidence & Studies",
        ksb_domains: &["D09", "D12", "D13"],
    },
    GvpModule {
        code: "IX",
        title: "Signal management",
        status: "Final",
        note: "Signal detection, validation and assessment.",
        pathway: "Signal Intelligence",
        ksb_domains: &["D08", "D10", "D12"],
    },
    GvpModule {
        code: "X",
        title: "Additional monitoring",
        status: "Final",
        note: "Black triangle and enhanced surveillance.",
        pathway: "Signal Intelligence",
        ksb_domains: &["D08", "D09", "D14"],
    },
    GvpModule {
        code: "XI",
        title: "Void",
        status: "Void",
        note: "Planned topic handled in other EMA guidance.",
        pathway: "Reserved / External Guidance",
        ksb_domains: &["D14"],
    },
    GvpModule {
        code: "XII",
        title: "Void",
        status: "Void",
        note: "Planned topic handled in other EMA guidance.",
        pathway: "Reserved / External Guidance",
        ksb_domains: &["D14"],
    },
    GvpModule {
        code: "XIII",
        title: "Void",
        status: "Void",
        note: "Planned topic handled in other EMA guidance.",
        pathway: "Reserved / External Guidance",
        ksb_domains: &["D14"],
    },
    GvpModule {
        code: "XIV",
        title: "Void",
        status: "Void",
        note: "Planned topic handled in other EMA guidance.",
        pathway: "Reserved / External Guidance",
        ksb_domains: &["D14"],
    },
    GvpModule {
        code: "XV",
        title: "Safety communication",
        status: "Final",
        note: "Safety communication planning and execution.",
        pathway: "Stakeholder Communication",
        ksb_domains: &["D10", "D11", "D15"],
    },
    GvpModule {
        code: "XVI",
        title: "Risk minimisation measures",
        status: "Final",
        note: "RMM selection and effectiveness evaluation.",
        pathway: "Risk & Benefit Management",
        ksb_domains: &["D07", "D10", "D15"],
    },
];

pub fn gvp_module_by_code(code: &str) -> Option<&'static GvpModule> {
    GVP_MODULES
        .iter()
        .find(|m| m.code.eq_ignore_ascii_case(code.trim()))
}

pub fn guardian_seed_for_module(code: &str) -> (&'static str, &'static str, u64) {
    match code.trim().to_ascii_uppercase().as_str() {
        "I" => ("adalimumab", "serious-infection", 18),
        "II" => ("atorvastatin", "liver-injury", 11),
        "III" => ("clopidogrel", "hemorrhage", 9),
        "IV" => ("methotrexate", "myelosuppression", 13),
        "V" => ("isotretinoin", "teratogenicity", 7),
        "VI" => ("warfarin", "bleeding", 42),
        "VII" => ("amoxicillin", "anaphylaxis", 6),
        "VIII" => ("semaglutide", "pancreatitis", 8),
        "IX" => ("clozapine", "agranulocytosis", 5),
        "X" => ("carbamazepine", "stevens-johnson-syndrome", 4),
        "XV" => ("valproate", "pregnancy-exposure", 10),
        "XVI" => ("codeine", "respiratory-depression", 12),
        _ => ("metformin", "lactic-acidosis", 3),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GuardianWritebackEvidence {
    pub module_code: String,
    pub drug_name: String,
    pub event_name: String,
    pub case_count: u64,
    pub risk_level: String,
    pub risk_score: f64,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GvpAssessmentPass {
    pub module_code: String,
    pub score_pct: u8,
    pub passed: bool,
    pub recorded_at: String,
}

pub fn module_evidence_count(module_code: &str, rows: &[GuardianWritebackEvidence]) -> usize {
    rows.iter()
        .filter(|r| r.module_code.eq_ignore_ascii_case(module_code))
        .count()
}

pub fn latest_module_evidence(
    module_code: &str,
    rows: &[GuardianWritebackEvidence],
) -> Option<GuardianWritebackEvidence> {
    rows.iter()
        .rev()
        .find(|r| r.module_code.eq_ignore_ascii_case(module_code))
        .cloned()
}

pub fn has_duplicate_evidence(
    rows: &[GuardianWritebackEvidence],
    row: &GuardianWritebackEvidence,
) -> bool {
    rows.iter().any(|r| {
        r.module_code.eq_ignore_ascii_case(&row.module_code)
            && r.drug_name.eq_ignore_ascii_case(&row.drug_name)
            && r.event_name.eq_ignore_ascii_case(&row.event_name)
            && r.case_count == row.case_count
            && r.risk_level.eq_ignore_ascii_case(&row.risk_level)
            && (r.risk_score - row.risk_score).abs() < 0.000_001
    })
}

pub fn has_assessment_pass(module_code: &str, rows: &[GvpAssessmentPass]) -> bool {
    rows.iter()
        .rev()
        .find(|r| r.module_code.eq_ignore_ascii_case(module_code))
        .map(|r| r.passed)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{GVP_MODULES, guardian_seed_for_module, gvp_module_by_code};

    #[test]
    fn has_all_ema_module_slots() {
        assert_eq!(GVP_MODULES.len(), 16);
    }

    #[test]
    fn has_expected_void_modules() {
        let void_codes: Vec<&str> = GVP_MODULES
            .iter()
            .filter(|m| m.status == "Void")
            .map(|m| m.code)
            .collect();
        assert_eq!(void_codes, vec!["XI", "XII", "XIII", "XIV"]);
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let m = gvp_module_by_code("ix").expect("module IX should exist");
        assert_eq!(m.code, "IX");
    }

    #[test]
    fn guardian_seed_defaults_for_unknown() {
        let (drug, event, count) = guardian_seed_for_module("unknown");
        assert_eq!(drug, "metformin");
        assert_eq!(event, "lactic-acidosis");
        assert_eq!(count, 3);
    }
}
