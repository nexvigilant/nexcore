//! Patient safety triage commands
//!
//! Tier: T3 (wires nexcore-guardian-engine patient_safety to terminal output)
//! Dominant primitive: ∂ Boundary (patient safety is ultimate boundary-guarding)

use colored::Colorize;
use nexcore_guardian_engine::patient_safety::{
    PatientSafetyPriority, SafetyEscalationMatrix, SeriousnessCategory, TriageResult,
    resolve_priority_conflict,
};

/// Parse seriousness string to enum
pub fn parse_seriousness(s: &str) -> Option<SeriousnessCategory> {
    match s.to_lowercase().as_str() {
        "fatal" | "death" => Some(SeriousnessCategory::Fatal),
        "lifethreatening" | "life-threatening" | "lt" => Some(SeriousnessCategory::LifeThreatening),
        "disability" | "dis" => Some(SeriousnessCategory::Disability),
        "hospitalization" | "hosp" => Some(SeriousnessCategory::Hospitalization),
        "congenital" | "anomaly" | "ca" => Some(SeriousnessCategory::CongenitalAnomaly),
        "medically-important" | "mi" | "medical" => Some(SeriousnessCategory::MedicallyImportant),
        "nonserious" | "non-serious" | "ns" => Some(SeriousnessCategory::NonSerious),
        _ => None,
    }
}

/// Parse priority level string to enum
pub fn parse_priority(s: &str) -> Option<PatientSafetyPriority> {
    match s.to_lowercase().as_str() {
        "p0" | "safety" | "patient-safety" => Some(PatientSafetyPriority::PatientSafety),
        "p1" | "signal" | "signal-integrity" => Some(PatientSafetyPriority::SignalIntegrity),
        "p2" | "regulatory" | "compliance" => Some(PatientSafetyPriority::RegulatoryCompliance),
        "p3" | "quality" | "data-quality" => Some(PatientSafetyPriority::DataQuality),
        "p4" | "operational" | "efficiency" => Some(PatientSafetyPriority::OperationalEfficiency),
        "p5" | "cost" | "optimization" => Some(PatientSafetyPriority::CostOptimization),
        _ => None,
    }
}

/// Handle triage command — run safety escalation matrix
pub fn handle_triage(seriousness: &str) -> String {
    let Some(cat) = parse_seriousness(seriousness) else {
        return format!(
            "{}: '{}'\n{}: fatal, life-threatening, disability, hospitalization, congenital, medical, nonserious\n",
            "Unknown seriousness".red(),
            seriousness,
            "Valid values".bold()
        );
    };

    let matrix = SafetyEscalationMatrix::new();
    // Use default signal strength and case count for triage assessment
    let result = matrix.triage_signal(cat, 2.0, 3);
    format_triage(&result)
}

/// Handle priority conflict resolution
pub fn handle_priority(a_str: &str, b_str: &str) -> String {
    let Some(a) = parse_priority(a_str) else {
        return format!(
            "{}: '{}'\n{}: p0-p5, safety, signal, regulatory, quality, operational, cost\n",
            "Unknown priority".red(),
            a_str,
            "Valid values".bold()
        );
    };
    let Some(b) = parse_priority(b_str) else {
        return format!(
            "{}: '{}'\n{}: p0-p5, safety, signal, regulatory, quality, operational, cost\n",
            "Unknown priority".red(),
            b_str,
            "Valid values".bold()
        );
    };

    let (winner, reason) = resolve_priority_conflict(a, b);

    format!(
        "\n{}\n{}: {:?} vs {:?}\n{}: {:?} (P{})\n{}: {}\n",
        section_header("PRIORITY RESOLUTION"),
        "Conflict".bold(),
        a,
        b,
        "Winner".bold(),
        winner,
        winner.level(),
        "Reason".bold(),
        reason,
    )
}

/// Handle escalation timeline query
pub fn handle_escalation(seriousness: &str) -> String {
    let Some(cat) = parse_seriousness(seriousness) else {
        return format!(
            "{}: '{}'\n{}: fatal, life-threatening, disability, hospitalization, congenital, medical, nonserious\n",
            "Unknown seriousness".red(),
            seriousness,
            "Valid values".bold()
        );
    };

    let matrix = SafetyEscalationMatrix::new();
    let result = matrix.triage_signal(cat, 2.0, 3);

    let hours = if result.escalation_hours == u32::MAX {
        "No escalation required".to_string()
    } else if result.escalation_hours == 0 {
        "IMMEDIATE (0h)".to_string()
    } else {
        format!("{}h", result.escalation_hours)
    };

    format!(
        "\n{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n",
        section_header("ESCALATION TIMELINE"),
        "Seriousness".bold(),
        cat.label(),
        "Escalation".bold(),
        hours,
        "Human review".bold(),
        bool_icon(result.requires_human_review),
        "Sensitive thresholds".bold(),
        bool_icon(result.use_sensitive_thresholds),
        "Rule".bold(),
        result.rule_description,
    )
}

fn format_triage(result: &TriageResult) -> String {
    let urgency = if result.is_emergency() {
        "EMERGENCY".red().bold()
    } else if result.is_critical() {
        "CRITICAL".yellow().bold()
    } else {
        "ROUTINE".green().normal()
    };

    let hours = if result.escalation_hours == u32::MAX {
        "N/A".to_string()
    } else if result.escalation_hours == 0 {
        "IMMEDIATE".to_string()
    } else {
        format!("{}h", result.escalation_hours)
    };

    format!(
        "\n{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n",
        section_header("TRIAGE RESULT"),
        "Seriousness".bold(),
        result.seriousness.label(),
        "Urgency".bold(),
        urgency,
        "Triage weight".bold(),
        result.triage_weight,
        "Escalation".bold(),
        hours,
        "Human review".bold(),
        bool_icon(result.requires_human_review),
        "Sensitive thresholds".bold(),
        bool_icon(result.use_sensitive_thresholds),
        "Rule".bold(),
        result.rule_description,
    )
}

fn bool_icon(v: bool) -> colored::ColoredString {
    if v {
        "YES".green().bold()
    } else {
        "no".dimmed()
    }
}

fn section_header(title: &str) -> colored::ColoredString {
    format!("=== {title} ===").cyan().bold()
}
