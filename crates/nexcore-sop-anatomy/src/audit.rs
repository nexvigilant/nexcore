//! # Codebase Audit Against SOP Governance Sections
//!
//! Checks a project directory against the 18 SOP governance sections by
//! detecting structural code patterns (file presence, directory existence).
//!
//! Scoring mirrors the SOP validator: Critical sections weighted 2x, max 25.

use crate::mapping::{Priority, SopSection};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ─── Audit Report ──────────────────────────────────────────────────────────

/// Audit result for a single SOP section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionAudit {
    pub number: u8,
    pub name: &'static str,
    pub priority: Priority,
    pub weight: u8,
    pub status: AuditStatus,
    pub detected_artifacts: Vec<String>,
    pub missing_hints: Vec<&'static str>,
}

/// Pass/Fail/Warn status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Pass,
    Fail,
    Warn,
}

/// Full audit report across all 18 sections.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditReport {
    pub path: String,
    pub score: u8,
    pub max_score: u8,
    pub percentage: f64,
    pub rating: &'static str,
    pub sections: Vec<SectionAudit>,
    pub critical_failures: Vec<u8>,
}

// ─── Audit Logic ───────────────────────────────────────────────────────────

/// Audit a project directory against all 18 SOP governance sections.
pub fn audit_path(path: &Path) -> AuditReport {
    let sections: Vec<SectionAudit> = SopSection::ALL
        .iter()
        .map(|section| audit_section(path, *section))
        .collect();

    let score: u8 = sections
        .iter()
        .filter(|s| s.status == AuditStatus::Pass)
        .map(|s| s.weight)
        .sum();

    let max_score = SopSection::max_score();
    let percentage = if max_score > 0 {
        (f64::from(score) / f64::from(max_score)) * 100.0
    } else {
        0.0
    };

    let rating = match percentage as u8 {
        90..=100 => "Compliant",
        75..=89 => "Minor Gaps",
        50..=74 => "Major Gaps",
        _ => "Non-Compliant",
    };

    let critical_failures: Vec<u8> = sections
        .iter()
        .filter(|s| s.priority == Priority::Critical && s.status == AuditStatus::Fail)
        .map(|s| s.number)
        .collect();

    AuditReport {
        path: path.display().to_string(),
        score,
        max_score,
        percentage,
        rating,
        sections,
        critical_failures,
    }
}

/// Audit one SOP section against the filesystem.
fn audit_section(path: &Path, section: SopSection) -> SectionAudit {
    let m = section.mapping();
    let mut detected = Vec::new();
    let mut missing = Vec::new();

    for hint in m.code.detection {
        let check_path = path.join(hint);
        if check_path.exists() {
            detected.push(hint.to_string());
        } else {
            missing.push(*hint);
        }
    }

    let status = if detected.is_empty() {
        AuditStatus::Fail
    } else if !missing.is_empty() {
        AuditStatus::Warn
    } else {
        AuditStatus::Pass
    };

    SectionAudit {
        number: m.number,
        name: m.name,
        priority: m.priority,
        weight: section.weight(),
        status,
        detected_artifacts: detected,
        missing_hints: missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_nonexistent_path() {
        let report = audit_path(Path::new("/nonexistent/path"));
        assert_eq!(report.score, 0);
        assert_eq!(report.rating, "Non-Compliant");
    }

    #[test]
    fn audit_status_values() {
        assert_eq!(
            serde_json::to_string(&AuditStatus::Pass).ok(),
            Some("\"pass\"".to_string())
        );
        assert_eq!(
            serde_json::to_string(&AuditStatus::Fail).ok(),
            Some("\"fail\"".to_string())
        );
    }

    #[test]
    fn audit_this_crate() {
        // This crate has Cargo.toml (S1) and src/ (S7) at minimum
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = audit_path(crate_dir);
        // Should at least detect Cargo.toml for S1
        let s1 = report.sections.iter().find(|s| s.number == 1);
        assert!(s1.is_some());
        let s1 = s1.map(|s| &s.status);
        // Cargo.toml should be detected
        assert!(matches!(s1, Some(AuditStatus::Pass | AuditStatus::Warn)));
    }

    #[test]
    fn max_score_consistent() {
        let report = audit_path(Path::new("/tmp"));
        assert_eq!(report.max_score, 25);
    }
}
