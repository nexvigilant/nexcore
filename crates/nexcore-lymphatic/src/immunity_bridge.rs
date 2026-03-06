//! # Immunity Bridge
//!
//! Inter-crate pipeline: Immunity → Lymphatic.
//!
//! Converts immunity `ScanResult` and `ThreatMatch` data into lymphatic
//! `OverflowItem`s for drainage and inspection.
//!
//! ```text
//! Immunity::ScanResult → OverflowItem → LymphNode/OverflowHandler
//! Immunity::ThreatMatch → InspectionResult (threat classification)
//! ```
//!
//! **Biological mapping**: Immune cells travel through the lymphatic system.
//! When the immunity scanner detects threats (PAMPs/DAMPs), those findings
//! become overflow items that the lymphatic drainage system must process —
//! analogous to how activated immune cells migrate to lymph nodes for
//! antigen presentation and immune coordination.

use nexcore_immunity::{ScanResult, ThreatLevel, ThreatMatch};

use crate::{InspectionResult, OverflowItem};

/// Convert an immunity `ThreatMatch` into a lymphatic `OverflowItem`.
///
/// **Biological mapping**: Antigen presentation — an activated immune cell
/// carrying threat information migrates to the nearest lymph node via
/// the lymphatic vessels for inspection and coordination.
pub fn threat_to_overflow(threat: &ThreatMatch) -> OverflowItem {
    let priority = threat_level_to_priority(&threat.severity);
    let content = format!(
        "immunity-threat:{}:{}",
        threat.antibody_name, threat.severity
    );
    OverflowItem::new(content, "immunity", priority)
}

/// Convert an immunity `ScanResult` into a batch of lymphatic `OverflowItem`s.
///
/// Clean scans produce no overflow items — only detected threats generate
/// lymphatic traffic. Each threat becomes an overflow item with priority
/// proportional to its severity.
///
/// **Biological mapping**: Post-scan immune response — after scanning code
/// for antipatterns, each detected threat is packaged and sent through
/// the lymphatic system for distributed processing.
pub fn scan_result_to_overflow(result: &ScanResult) -> Vec<OverflowItem> {
    if result.clean {
        return Vec::new();
    }
    result.threats.iter().map(|t| threat_to_overflow(t)).collect()
}

/// Map a `ThreatMatch` to a lymphatic `InspectionResult`.
///
/// Translates immunity threat classification into the lymph node's own
/// inspection vocabulary.
///
/// **Biological mapping**: Lymph node triage — when an immune cell arrives
/// at a lymph node, the node classifies the threat for further action.
pub fn threat_to_inspection(threat: &ThreatMatch) -> InspectionResult {
    match threat.severity {
        ThreatLevel::Critical | ThreatLevel::High => InspectionResult::Threat(format!(
            "immunity[{}]: {} ({})",
            threat.severity, threat.antibody_name, threat.threat_type
        )),
        ThreatLevel::Medium => InspectionResult::Suspicious(format!(
            "immunity[{}]: {} ({})",
            threat.severity, threat.antibody_name, threat.threat_type
        )),
        ThreatLevel::Low => InspectionResult::Clear,
    }
}

/// Compute the throughput metric for an immunity scan result.
///
/// Returns the number of threats detected — a scalar value representing
/// the immune load being placed on the lymphatic system.
///
/// **Biological mapping**: Immune burden — higher threat counts indicate
/// greater demand on lymphatic drainage capacity.
pub fn scan_throughput(result: &ScanResult) -> usize {
    result.threats.len()
}

/// Map immunity `ThreatLevel` to lymphatic priority (0-255).
///
/// **Biological mapping**: Triage urgency — critical threats (like
/// anaphylaxis) demand immediate lymphatic response, while low-severity
/// findings can wait for routine drainage.
fn threat_level_to_priority(level: &ThreatLevel) -> u8 {
    match level {
        ThreatLevel::Critical => 1, // Highest urgency
        ThreatLevel::High => 2,
        ThreatLevel::Medium => 5,
        ThreatLevel::Low => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_immunity::{ResponseStrategy, ThreatType};

    fn make_threat(name: &str, severity: ThreatLevel, threat_type: ThreatType) -> ThreatMatch {
        ThreatMatch {
            antibody_id: format!("TEST-{name}"),
            antibody_name: name.to_string(),
            threat_type,
            severity,
            location: None,
            matched_content: String::new(),
            confidence: 0.9,
            response: ResponseStrategy::Warn,
        }
    }

    fn make_scan_result(threats: Vec<ThreatMatch>) -> ScanResult {
        let clean = threats.is_empty();
        ScanResult {
            threats,
            clean,
            antibodies_applied: vec!["test-antibody".to_string()],
            metrics: Default::default(),
        }
    }

    #[test]
    fn test_threat_to_overflow_critical() {
        let threat = make_threat("test-unwrap", ThreatLevel::Critical, ThreatType::Damp);
        let item = threat_to_overflow(&threat);

        assert_eq!(item.source, "immunity");
        assert_eq!(item.priority, 1);
        assert!(item.content.contains("test-unwrap"));
        assert!(!item.drained);
    }

    #[test]
    fn test_threat_to_overflow_low() {
        let threat = make_threat("style-issue", ThreatLevel::Low, ThreatType::Pamp);
        let item = threat_to_overflow(&threat);

        assert_eq!(item.priority, 10);
        assert!(item.content.contains("style-issue"));
    }

    #[test]
    fn test_scan_result_to_overflow_clean() {
        let result = make_scan_result(Vec::new());
        let items = scan_result_to_overflow(&result);
        assert!(items.is_empty());
    }

    #[test]
    fn test_scan_result_to_overflow_with_threats() {
        let result = make_scan_result(vec![
            make_threat("unwrap-usage", ThreatLevel::Critical, ThreatType::Damp),
            make_threat("todo-comment", ThreatLevel::Low, ThreatType::Damp),
        ]);
        let items = scan_result_to_overflow(&result);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].priority, 1); // Critical
        assert_eq!(items[1].priority, 10); // Low
    }

    #[test]
    fn test_threat_to_inspection_critical() {
        let threat = make_threat("panic-call", ThreatLevel::Critical, ThreatType::Damp);
        let result = threat_to_inspection(&threat);
        assert!(matches!(result, InspectionResult::Threat(_)));
    }

    #[test]
    fn test_threat_to_inspection_high() {
        let threat = make_threat("expect-call", ThreatLevel::High, ThreatType::Damp);
        let result = threat_to_inspection(&threat);
        assert!(matches!(result, InspectionResult::Threat(_)));
    }

    #[test]
    fn test_threat_to_inspection_medium() {
        let threat = make_threat("style-warn", ThreatLevel::Medium, ThreatType::Pamp);
        let result = threat_to_inspection(&threat);
        assert!(matches!(result, InspectionResult::Suspicious(_)));
    }

    #[test]
    fn test_threat_to_inspection_low() {
        let threat = make_threat("info-note", ThreatLevel::Low, ThreatType::Pamp);
        let result = threat_to_inspection(&threat);
        assert_eq!(result, InspectionResult::Clear);
    }

    #[test]
    fn test_scan_throughput() {
        let result = make_scan_result(vec![
            make_threat("a", ThreatLevel::High, ThreatType::Damp),
            make_threat("b", ThreatLevel::Low, ThreatType::Pamp),
            make_threat("c", ThreatLevel::Medium, ThreatType::Damp),
        ]);
        assert_eq!(scan_throughput(&result), 3);
    }

    #[test]
    fn test_scan_throughput_clean() {
        let result = make_scan_result(Vec::new());
        assert_eq!(scan_throughput(&result), 0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(threat_level_to_priority(&ThreatLevel::Critical) < threat_level_to_priority(&ThreatLevel::High));
        assert!(threat_level_to_priority(&ThreatLevel::High) < threat_level_to_priority(&ThreatLevel::Medium));
        assert!(threat_level_to_priority(&ThreatLevel::Medium) < threat_level_to_priority(&ThreatLevel::Low));
    }
}
