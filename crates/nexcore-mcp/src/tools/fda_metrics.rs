//! FDA Credibility Metrics MCP Tools
//!
//! ## T1 Primitive Foundation
//!
//! | Tool | T1 Grounding | Purpose |
//! |------|:------------:|---------|
//! | fda_calculate_score | N | Compute credibility score from inputs |
//! | fda_metrics_summary | Σ | Get aggregate assessment metrics |
//! | fda_evidence_distribution | N+ν | Evidence type/quality breakdown |
//! | fda_risk_distribution | N+κ | Risk level frequency analysis |
//! | fda_drift_trend | ν+σ | Drift pattern analysis |

use nexcore_vigilance::fda::{
    AssessmentMetrics, CredibilityInput, CredibilityRating, CredibilityScore, DriftHistory,
    DriftMeasurement, EvidenceDistribution, EvidenceQuality, EvidenceType, RiskDistribution,
    RiskLevel,
};
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Calculate credibility score from component inputs
///
/// T1 Grounding: N (Weighted linear combination)
pub fn fda_calculate_score(
    evidence_quality: f64,
    fit_for_use: f64,
    risk_mitigation: f64,
    documentation: f64,
) -> CallToolResult {
    // Validate inputs are 0-1
    if evidence_quality < 0.0
        || evidence_quality > 1.0
        || fit_for_use < 0.0
        || fit_for_use > 1.0
        || risk_mitigation < 0.0
        || risk_mitigation > 1.0
        || documentation < 0.0
        || documentation > 1.0
    {
        return error_result("All inputs must be between 0.0 and 1.0");
    }

    let input = CredibilityInput {
        evidence_quality,
        fit_for_use,
        risk_mitigation,
        documentation,
    };

    let score = input.calculate_score();
    let rating = score.rating();

    success_json(json!({
        "status": "success",
        "t1_grounding": "N (Weighted linear combination)",
        "inputs": {
            "evidence_quality": evidence_quality,
            "fit_for_use": fit_for_use,
            "risk_mitigation": risk_mitigation,
            "documentation": documentation
        },
        "formula": "score = (evidence × 0.30) + (fit_for_use × 0.25) + (risk × 0.25) + (docs × 0.20) × 10",
        "result": {
            "score": score.value(),
            "score_display": score,
            "rating": rating.to_string(),
            "is_acceptable": rating.is_acceptable(),
            "recommended_action": rating.recommended_action()
        },
        "thresholds": {
            "Critical": "0-2 (reject)",
            "Weak": "2-4 (revision needed)",
            "Adequate": "4-6 (minimum acceptable)",
            "Good": "6-8 (above requirements)",
            "Excellent": "8-10 (exemplary)"
        }
    }))
}

/// Get assessment metrics summary
///
/// T1 Grounding: Σ (Aggregation)
pub fn fda_metrics_summary(
    started: usize,
    completed: usize,
    approved: usize,
    rejected: usize,
    revision: usize,
    drift_alerts: usize,
) -> CallToolResult {
    let mut metrics = AssessmentMetrics::new();
    metrics.assessments_started = started;
    metrics.assessments_completed = completed;
    metrics.assessments_approved = approved;
    metrics.assessments_rejected = rejected;
    metrics.assessments_revision = revision;
    metrics.drift_alerts = drift_alerts;

    let health = metrics.health_score();

    success_json(json!({
        "status": "success",
        "t1_grounding": "Σ (Aggregation)",
        "counts": {
            "started": started,
            "completed": completed,
            "approved": approved,
            "rejected": rejected,
            "needs_revision": revision,
            "drift_alerts": drift_alerts
        },
        "rates": {
            "approval_rate": format!("{:.1}%", metrics.approval_rate() * 100.0),
            "completion_rate": format!("{:.1}%", metrics.completion_rate() * 100.0),
            "rejection_rate": if completed > 0 {
                format!("{:.1}%", rejected as f64 / completed as f64 * 100.0)
            } else {
                "N/A".to_string()
            }
        },
        "health": {
            "score": health.value(),
            "rating": health.rating().to_string(),
            "is_acceptable": health.rating().is_acceptable()
        }
    }))
}

/// Analyze evidence distribution
///
/// T1 Grounding: N (Quantity) + ν (Frequency)
pub fn fda_evidence_distribution(evidence_items: Vec<(String, String)>) -> CallToolResult {
    let mut dist = EvidenceDistribution::new();

    for (type_str, quality_str) in &evidence_items {
        let ev_type = parse_evidence_type(type_str);
        let quality = parse_evidence_quality(quality_str);
        dist.record(&ev_type, quality);
    }

    success_json(json!({
        "status": "success",
        "t1_grounding": "N (Quantity) + ν (Frequency)",
        "summary": {
            "total": dist.total(),
            "high_quality_count": dist.high_quality_count(),
            "high_quality_ratio": format!("{:.1}%", dist.high_quality_ratio() * 100.0)
        },
        "by_type": dist.by_type(),
        "by_quality": dist.by_quality(),
        "assessment": if dist.high_quality_ratio() >= 0.7 {
            "Strong evidence portfolio"
        } else if dist.high_quality_ratio() >= 0.5 {
            "Adequate evidence quality"
        } else {
            "Evidence quality improvement needed"
        }
    }))
}

/// Analyze risk distribution
///
/// T1 Grounding: N (Quantity) + κ (Comparison)
pub fn fda_risk_distribution(risk_levels: Vec<String>) -> CallToolResult {
    let mut dist = RiskDistribution::new();

    for level_str in &risk_levels {
        let level = parse_risk_level(level_str);
        dist.record(level);
    }

    success_json(json!({
        "status": "success",
        "t1_grounding": "N (Quantity) + κ (Comparison)",
        "summary": {
            "total": dist.total(),
            "high_risk_ratio": format!("{:.1}%", dist.high_risk_ratio() * 100.0)
        },
        "distribution": dist.distribution(),
        "assessment": if dist.high_risk_ratio() > 0.5 {
            "High-risk portfolio — additional validation required"
        } else if dist.high_risk_ratio() > 0.25 {
            "Mixed risk portfolio — prioritize high-risk assessments"
        } else {
            "Low-risk portfolio — standard validation acceptable"
        }
    }))
}

/// Analyze drift trends
///
/// T1 Grounding: ν (Frequency) + σ (Sequence)
pub fn fda_drift_trend(
    measurements: Vec<(u64, f64, String, String)>,
    trend_threshold: f64,
) -> CallToolResult {
    let mut history = DriftHistory::new();

    for (timestamp, drift_percent, severity, model_id) in measurements {
        history.record(DriftMeasurement {
            timestamp,
            drift_percent,
            severity,
            model_id,
        });
    }

    let recent_trend = history.trend(5);
    let is_worsening = history.is_worsening(trend_threshold);

    success_json(json!({
        "status": "success",
        "t1_grounding": "ν (Frequency) + σ (Sequence)",
        "summary": {
            "total_measurements": history.measurement_count(),
            "total_alerts": history.total_alerts(),
            "minor_alerts": history.alert_count("MINOR"),
            "major_alerts": history.alert_count("MAJOR"),
            "critical_alerts": history.alert_count("CRITICAL")
        },
        "trend": {
            "recent_average_drift": format!("{:.1}%", recent_trend),
            "threshold": trend_threshold,
            "is_worsening": is_worsening,
            "direction": if is_worsening { "INCREASING" } else { "STABLE" }
        },
        "recommendation": if is_worsening {
            "Drift increasing — consider revalidation"
        } else if recent_trend > 15.0 {
            "High drift — monitor closely"
        } else if recent_trend > 5.0 {
            "Moderate drift — within acceptable bounds"
        } else {
            "Low drift — model stable"
        }
    }))
}

/// Get rating thresholds and descriptions
///
/// T1 Grounding: κ (Comparison thresholds)
pub fn fda_rating_thresholds() -> CallToolResult {
    success_json(json!({
        "status": "success",
        "t1_grounding": "κ (Comparison thresholds)",
        "ratings": [
            {
                "rating": "Critical",
                "range": "0.0 - 2.0",
                "description": "Major deficiencies — reject",
                "action": CredibilityRating::Critical.recommended_action(),
                "is_acceptable": false
            },
            {
                "rating": "Weak",
                "range": "2.0 - 4.0",
                "description": "Significant gaps — needs revision",
                "action": CredibilityRating::Weak.recommended_action(),
                "is_acceptable": false
            },
            {
                "rating": "Adequate",
                "range": "4.0 - 6.0",
                "description": "Minimum acceptable — proceed with caution",
                "action": CredibilityRating::Adequate.recommended_action(),
                "is_acceptable": true
            },
            {
                "rating": "Good",
                "range": "6.0 - 8.0",
                "description": "Above requirements — strong credibility",
                "action": CredibilityRating::Good.recommended_action(),
                "is_acceptable": true
            },
            {
                "rating": "Excellent",
                "range": "8.0 - 10.0",
                "description": "Exemplary — model submission",
                "action": CredibilityRating::Excellent.recommended_action(),
                "is_acceptable": true
            }
        ],
        "formula": {
            "weights": {
                "evidence_quality": 0.30,
                "fit_for_use": 0.25,
                "risk_mitigation": 0.25,
                "documentation": 0.20
            },
            "equation": "score = (E × 0.30 + F × 0.25 + R × 0.25 + D × 0.20) × 10"
        }
    }))
}

// ============================================================================
// Helpers
// ============================================================================

fn parse_evidence_type(s: &str) -> EvidenceType {
    match s.to_lowercase().as_str() {
        "validation_metrics" | "metrics" => EvidenceType::ValidationMetrics,
        "test_results" | "test" => EvidenceType::TestResults,
        "training_data" | "data" => EvidenceType::TrainingData,
        "architecture" | "arch" => EvidenceType::Architecture,
        "bias_analysis" | "bias" => EvidenceType::BiasAnalysis,
        "explainability" | "explain" => EvidenceType::Explainability,
        "prior_knowledge" | "literature" => EvidenceType::PriorKnowledge,
        "precedent" => EvidenceType::Precedent,
        other => EvidenceType::Other(other.to_string()),
    }
}

fn parse_evidence_quality(s: &str) -> EvidenceQuality {
    match s.to_lowercase().as_str() {
        "high" => EvidenceQuality::High,
        "medium" => EvidenceQuality::Medium,
        _ => EvidenceQuality::Low,
    }
}

fn parse_risk_level(s: &str) -> RiskLevel {
    match s.to_lowercase().as_str() {
        "high" => RiskLevel::High,
        "medium" => RiskLevel::Medium,
        _ => RiskLevel::Low,
    }
}

fn success_json(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string()),
    )])
}

fn error_result(message: &str) -> CallToolResult {
    CallToolResult::error(vec![Content::text(message)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_text(result: &CallToolResult) -> String {
        result
            .content
            .first()
            .map(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => t.text.clone(),
                _ => String::new(),
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_calculate_score_perfect() {
        let result = fda_calculate_score(1.0, 1.0, 1.0, 1.0);
        let text = get_text(&result);
        assert!(
            text.contains("10.00/10"),
            "Should be perfect score: {}",
            text
        );
        assert!(text.contains("Excellent"), "Should be Excellent: {}", text);
    }

    #[test]
    fn test_calculate_score_half() {
        let result = fda_calculate_score(0.5, 0.5, 0.5, 0.5);
        let text = get_text(&result);
        assert!(text.contains("5.00/10"), "Should be 5.0: {}", text);
        assert!(text.contains("Adequate"), "Should be Adequate: {}", text);
    }

    #[test]
    fn test_calculate_score_invalid() {
        let result = fda_calculate_score(1.5, 0.5, 0.5, 0.5);
        let text = get_text(&result);
        assert!(text.contains("must be between"), "Should error: {}", text);
    }

    #[test]
    fn test_metrics_summary() {
        let result = fda_metrics_summary(100, 80, 60, 10, 10, 5);
        let text = get_text(&result);
        assert!(text.contains("75.0%"), "Should show 75% approval: {}", text);
        assert!(
            text.contains("80.0%"),
            "Should show 80% completion: {}",
            text
        );
    }

    #[test]
    fn test_evidence_distribution() {
        let items = vec![
            ("metrics".to_string(), "high".to_string()),
            ("test".to_string(), "high".to_string()),
            ("docs".to_string(), "low".to_string()),
        ];
        let result = fda_evidence_distribution(items);
        let text = get_text(&result);
        assert!(
            text.contains("66.7%"),
            "Should show 66.7% high quality: {}",
            text
        );
    }

    #[test]
    fn test_risk_distribution() {
        let levels = vec![
            "high".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "low".to_string(),
        ];
        let result = fda_risk_distribution(levels);
        let text = get_text(&result);
        assert!(
            text.contains("25.0%"),
            "Should show 25% high risk: {}",
            text
        );
    }

    #[test]
    fn test_rating_thresholds() {
        let result = fda_rating_thresholds();
        let text = get_text(&result);
        assert!(text.contains("Critical"), "Should list Critical: {}", text);
        assert!(
            text.contains("Excellent"),
            "Should list Excellent: {}",
            text
        );
        assert!(text.contains("0.30"), "Should show weights: {}", text);
    }
}
