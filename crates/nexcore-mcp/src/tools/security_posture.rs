//! Security Posture Assessment — compliance scorecards, threat readiness, gap analysis.
//!
//! Inspired by AI Engineering Bible Section 35 (Securing AI Systems):
//! provides structured security posture evaluation against compliance
//! frameworks and AI-specific threat models.
//!
//! # Frameworks
//!
//! - **SOC 2**: Trust Services Criteria (security, availability, processing integrity)
//! - **HIPAA**: Protected health information safeguards
//! - **GDPR**: Data protection and privacy rights
//! - **EU AI Act**: Risk-based AI classification and requirements
//! - **NIST**: Cybersecurity framework (Identify, Protect, Detect, Respond, Recover)
//! - **ISO 27001**: Information security management system
//!
//! # T1 Grounding: ∂(Boundary) + ∝(Irreversibility) + κ(Comparison) + π(Persistence)
//! - ∂: Security boundaries and access controls
//! - ∝: Irreversible audit trails
//! - κ: Compliance comparison against requirements
//! - π: Persistent security posture over time

use crate::params::{
    SecurityComplianceGapParams, SecurityPostureAssessParams, SecurityThreatReadinessParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// Framework Definitions
// ============================================================================

struct FrameworkDef {
    name: &'static str,
    full_name: &'static str,
    controls: &'static [ControlDef],
}

struct ControlDef {
    id: &'static str,
    name: &'static str,
    category: &'static str,
    weight: f64, // 0.0-1.0 importance
}

const SOC2: FrameworkDef = FrameworkDef {
    name: "soc2",
    full_name: "SOC 2 Type II",
    controls: &[
        ControlDef {
            id: "CC1.1",
            name: "Security policies and procedures",
            category: "Security",
            weight: 0.8,
        },
        ControlDef {
            id: "CC2.1",
            name: "Internal communication of security",
            category: "Security",
            weight: 0.6,
        },
        ControlDef {
            id: "CC3.1",
            name: "Risk assessment process",
            category: "Security",
            weight: 0.9,
        },
        ControlDef {
            id: "CC5.1",
            name: "Logical and physical access controls",
            category: "Security",
            weight: 1.0,
        },
        ControlDef {
            id: "CC6.1",
            name: "Encryption of data at rest and in transit",
            category: "Security",
            weight: 1.0,
        },
        ControlDef {
            id: "CC7.1",
            name: "System monitoring and alerting",
            category: "Availability",
            weight: 0.8,
        },
        ControlDef {
            id: "CC8.1",
            name: "Change management process",
            category: "Processing Integrity",
            weight: 0.7,
        },
        ControlDef {
            id: "CC9.1",
            name: "Vendor risk management",
            category: "Security",
            weight: 0.6,
        },
        ControlDef {
            id: "A1.1",
            name: "System availability SLAs",
            category: "Availability",
            weight: 0.7,
        },
        ControlDef {
            id: "PI1.1",
            name: "Data processing accuracy",
            category: "Processing Integrity",
            weight: 0.8,
        },
    ],
};

const HIPAA: FrameworkDef = FrameworkDef {
    name: "hipaa",
    full_name: "HIPAA Security Rule",
    controls: &[
        ControlDef {
            id: "164.308(a)(1)",
            name: "Security management process",
            category: "Administrative",
            weight: 0.9,
        },
        ControlDef {
            id: "164.308(a)(3)",
            name: "Workforce security",
            category: "Administrative",
            weight: 0.8,
        },
        ControlDef {
            id: "164.308(a)(4)",
            name: "Information access management",
            category: "Administrative",
            weight: 0.9,
        },
        ControlDef {
            id: "164.308(a)(5)",
            name: "Security awareness training",
            category: "Administrative",
            weight: 0.7,
        },
        ControlDef {
            id: "164.310(a)(1)",
            name: "Facility access controls",
            category: "Physical",
            weight: 0.6,
        },
        ControlDef {
            id: "164.312(a)(1)",
            name: "Access control (unique IDs, emergency access)",
            category: "Technical",
            weight: 1.0,
        },
        ControlDef {
            id: "164.312(b)",
            name: "Audit controls",
            category: "Technical",
            weight: 0.9,
        },
        ControlDef {
            id: "164.312(c)(1)",
            name: "Data integrity controls",
            category: "Technical",
            weight: 0.8,
        },
        ControlDef {
            id: "164.312(d)",
            name: "Authentication",
            category: "Technical",
            weight: 1.0,
        },
        ControlDef {
            id: "164.312(e)(1)",
            name: "Transmission security (encryption)",
            category: "Technical",
            weight: 1.0,
        },
    ],
};

const GDPR: FrameworkDef = FrameworkDef {
    name: "gdpr",
    full_name: "EU General Data Protection Regulation",
    controls: &[
        ControlDef {
            id: "Art.5",
            name: "Data processing principles (lawfulness, fairness, transparency)",
            category: "Principles",
            weight: 1.0,
        },
        ControlDef {
            id: "Art.6",
            name: "Lawful basis for processing",
            category: "Principles",
            weight: 1.0,
        },
        ControlDef {
            id: "Art.13",
            name: "Information to data subjects",
            category: "Rights",
            weight: 0.8,
        },
        ControlDef {
            id: "Art.15",
            name: "Right of access",
            category: "Rights",
            weight: 0.9,
        },
        ControlDef {
            id: "Art.17",
            name: "Right to erasure",
            category: "Rights",
            weight: 0.9,
        },
        ControlDef {
            id: "Art.25",
            name: "Data protection by design and default",
            category: "Technical",
            weight: 0.9,
        },
        ControlDef {
            id: "Art.30",
            name: "Records of processing activities",
            category: "Administrative",
            weight: 0.7,
        },
        ControlDef {
            id: "Art.32",
            name: "Security of processing",
            category: "Technical",
            weight: 1.0,
        },
        ControlDef {
            id: "Art.33",
            name: "Breach notification (72 hours)",
            category: "Incident",
            weight: 0.8,
        },
        ControlDef {
            id: "Art.35",
            name: "Data Protection Impact Assessment",
            category: "Administrative",
            weight: 0.8,
        },
    ],
};

const EU_AI_ACT: FrameworkDef = FrameworkDef {
    name: "eu_ai_act",
    full_name: "EU Artificial Intelligence Act",
    controls: &[
        ControlDef {
            id: "Art.9",
            name: "Risk management system",
            category: "High-Risk",
            weight: 1.0,
        },
        ControlDef {
            id: "Art.10",
            name: "Data governance (training data quality)",
            category: "High-Risk",
            weight: 0.9,
        },
        ControlDef {
            id: "Art.11",
            name: "Technical documentation",
            category: "High-Risk",
            weight: 0.7,
        },
        ControlDef {
            id: "Art.12",
            name: "Record-keeping (logging)",
            category: "High-Risk",
            weight: 0.8,
        },
        ControlDef {
            id: "Art.13",
            name: "Transparency (user information)",
            category: "High-Risk",
            weight: 0.9,
        },
        ControlDef {
            id: "Art.14",
            name: "Human oversight measures",
            category: "High-Risk",
            weight: 1.0,
        },
        ControlDef {
            id: "Art.15",
            name: "Accuracy, robustness, cybersecurity",
            category: "High-Risk",
            weight: 1.0,
        },
        ControlDef {
            id: "Art.52",
            name: "Transparency for AI-generated content",
            category: "General",
            weight: 0.8,
        },
        ControlDef {
            id: "Art.62",
            name: "Post-market monitoring",
            category: "High-Risk",
            weight: 0.7,
        },
        ControlDef {
            id: "Art.65",
            name: "Serious incident reporting",
            category: "High-Risk",
            weight: 0.8,
        },
    ],
};

const NIST: FrameworkDef = FrameworkDef {
    name: "nist",
    full_name: "NIST Cybersecurity Framework",
    controls: &[
        ControlDef {
            id: "ID.AM",
            name: "Asset Management",
            category: "Identify",
            weight: 0.8,
        },
        ControlDef {
            id: "ID.RA",
            name: "Risk Assessment",
            category: "Identify",
            weight: 0.9,
        },
        ControlDef {
            id: "PR.AC",
            name: "Identity & Access Control",
            category: "Protect",
            weight: 1.0,
        },
        ControlDef {
            id: "PR.DS",
            name: "Data Security",
            category: "Protect",
            weight: 1.0,
        },
        ControlDef {
            id: "PR.IP",
            name: "Information Protection Processes",
            category: "Protect",
            weight: 0.8,
        },
        ControlDef {
            id: "PR.AT",
            name: "Awareness and Training",
            category: "Protect",
            weight: 0.6,
        },
        ControlDef {
            id: "DE.AE",
            name: "Anomaly and Event Detection",
            category: "Detect",
            weight: 0.9,
        },
        ControlDef {
            id: "DE.CM",
            name: "Continuous Monitoring",
            category: "Detect",
            weight: 0.8,
        },
        ControlDef {
            id: "RS.RP",
            name: "Incident Response Planning",
            category: "Respond",
            weight: 0.8,
        },
        ControlDef {
            id: "RC.RP",
            name: "Recovery Planning",
            category: "Recover",
            weight: 0.7,
        },
    ],
};

const ISO27001: FrameworkDef = FrameworkDef {
    name: "iso27001",
    full_name: "ISO/IEC 27001:2022",
    controls: &[
        ControlDef {
            id: "A.5",
            name: "Information security policies",
            category: "Organizational",
            weight: 0.8,
        },
        ControlDef {
            id: "A.6",
            name: "Organization of information security",
            category: "Organizational",
            weight: 0.7,
        },
        ControlDef {
            id: "A.7",
            name: "Human resource security",
            category: "People",
            weight: 0.6,
        },
        ControlDef {
            id: "A.8",
            name: "Asset management",
            category: "Organizational",
            weight: 0.7,
        },
        ControlDef {
            id: "A.9",
            name: "Access control",
            category: "Technical",
            weight: 1.0,
        },
        ControlDef {
            id: "A.10",
            name: "Cryptography",
            category: "Technical",
            weight: 0.9,
        },
        ControlDef {
            id: "A.12",
            name: "Operations security",
            category: "Technical",
            weight: 0.8,
        },
        ControlDef {
            id: "A.14",
            name: "System acquisition, development, maintenance",
            category: "Technical",
            weight: 0.8,
        },
        ControlDef {
            id: "A.16",
            name: "Incident management",
            category: "Organizational",
            weight: 0.8,
        },
        ControlDef {
            id: "A.18",
            name: "Compliance",
            category: "Organizational",
            weight: 0.9,
        },
    ],
};

fn get_framework(name: &str) -> Option<&'static FrameworkDef> {
    match name.to_lowercase().as_str() {
        "soc2" | "soc_2" => Some(&SOC2),
        "hipaa" => Some(&HIPAA),
        "gdpr" => Some(&GDPR),
        "eu_ai_act" | "euaiact" => Some(&EU_AI_ACT),
        "nist" => Some(&NIST),
        "iso27001" | "iso_27001" => Some(&ISO27001),
        _ => None,
    }
}

fn all_frameworks() -> Vec<&'static FrameworkDef> {
    vec![&SOC2, &HIPAA, &GDPR, &EU_AI_ACT, &NIST, &ISO27001]
}

// ============================================================================
// Threat Definitions
// ============================================================================

struct ThreatDef {
    id: &'static str,
    name: &'static str,
    severity: &'static str,
    defenses: &'static [&'static str],
}

const AI_THREATS: &[ThreatDef] = &[
    ThreatDef {
        id: "T1",
        name: "Data Poisoning",
        severity: "critical",
        defenses: &[
            "data_provenance",
            "input_validation",
            "anomaly_detection",
            "data_versioning",
        ],
    },
    ThreatDef {
        id: "T2",
        name: "Model Inversion",
        severity: "high",
        defenses: &[
            "differential_privacy",
            "output_perturbation",
            "access_controls",
            "rate_limiting",
        ],
    },
    ThreatDef {
        id: "T3",
        name: "Adversarial Attacks",
        severity: "high",
        defenses: &[
            "adversarial_training",
            "input_sanitization",
            "ensemble_models",
            "robustness_testing",
        ],
    },
    ThreatDef {
        id: "T4",
        name: "Prompt Injection",
        severity: "critical",
        defenses: &[
            "input_filtering",
            "output_guardrails",
            "sandboxing",
            "privilege_separation",
        ],
    },
    ThreatDef {
        id: "T5",
        name: "Supply Chain Compromise",
        severity: "critical",
        defenses: &[
            "dependency_scanning",
            "sbom_tracking",
            "hash_verification",
            "vendor_assessment",
        ],
    },
];

// ============================================================================
// MCP Tools
// ============================================================================

/// `security_posture_assess` — Multi-framework compliance scorecard.
///
/// Evaluates a system against selected compliance frameworks and returns
/// a weighted posture score with per-framework breakdown.
pub fn security_posture_assess(
    params: SecurityPostureAssessParams,
) -> Result<CallToolResult, McpError> {
    let frameworks_to_check: Vec<&FrameworkDef> = match params.frameworks {
        Some(ref names) if !names.is_empty() => {
            let mut fws = Vec::new();
            for name in names {
                match get_framework(name) {
                    Some(fw) => fws.push(fw),
                    None => {
                        return Err(McpError::invalid_params(
                            format!(
                                "Unknown framework: '{}'. Available: soc2, hipaa, gdpr, eu_ai_act, nist, iso27001",
                                name
                            ),
                            None,
                        ));
                    }
                }
            }
            fws
        }
        _ => all_frameworks(),
    };

    let existing: Vec<String> = params
        .existing_controls
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let mut framework_scores: Vec<serde_json::Value> = Vec::new();
    let mut total_score = 0.0;
    let mut total_weight = 0.0;

    for fw in &frameworks_to_check {
        let mut met_controls = Vec::new();
        let mut gap_controls = Vec::new();
        let mut fw_score = 0.0;
        let mut fw_max = 0.0;

        for control in fw.controls {
            fw_max += control.weight;
            let control_lower = control.name.to_lowercase();
            let is_met = existing.iter().any(|e| {
                control_lower.contains(e.as_str())
                    || control.id.to_lowercase() == *e
                    || e.contains(&control.id.to_lowercase())
            });

            if is_met {
                fw_score += control.weight;
                met_controls.push(json!({
                    "id": control.id,
                    "name": control.name,
                    "category": control.category,
                }));
            } else {
                gap_controls.push(json!({
                    "id": control.id,
                    "name": control.name,
                    "category": control.category,
                    "weight": control.weight,
                }));
            }
        }

        let pct = if fw_max > 0.0 {
            fw_score / fw_max * 100.0
        } else {
            0.0
        };
        total_score += pct;
        total_weight += 1.0;

        let grade = match pct as u64 {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        };

        framework_scores.push(json!({
            "framework": fw.full_name,
            "code": fw.name,
            "score_pct": (pct * 10.0).round() / 10.0,
            "grade": grade,
            "controls_met": met_controls.len(),
            "controls_total": fw.controls.len(),
            "gaps": gap_controls,
            "met": met_controls,
        }));
    }

    let overall_pct = if total_weight > 0.0 {
        total_score / total_weight
    } else {
        0.0
    };

    let overall_grade = match overall_pct as u64 {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    };

    let result = json!({
        "target": params.target,
        "overall_posture": {
            "score_pct": (overall_pct * 10.0).round() / 10.0,
            "grade": overall_grade,
            "frameworks_assessed": frameworks_to_check.len(),
            "existing_controls_declared": existing.len(),
        },
        "frameworks": framework_scores,
        "recommendation": match overall_grade {
            "A" => "Strong security posture. Maintain current controls and continue monitoring.",
            "B" => "Good posture with minor gaps. Address remaining controls in next quarter.",
            "C" => "Moderate gaps. Prioritize high-weight controls and create remediation plan.",
            "D" => "Significant gaps. Immediate action needed on critical controls.",
            _ => "Critical gaps. Security posture requires urgent remediation across all frameworks.",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `security_threat_readiness` — AI-specific threat readiness scoring.
///
/// Scores readiness against 5 AI threat categories with defense gap analysis.
pub fn security_threat_readiness(
    params: SecurityThreatReadinessParams,
) -> Result<CallToolResult, McpError> {
    let defenses: Vec<String> = params
        .defenses
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_lowercase().replace(' ', "_"))
        .collect();

    let threats_to_check: Vec<&ThreatDef> = match params.threats {
        Some(ref names) if !names.is_empty() => AI_THREATS
            .iter()
            .filter(|t| {
                names.iter().any(|n| {
                    t.id.to_lowercase() == n.to_lowercase()
                        || t.name.to_lowercase().contains(&n.to_lowercase())
                })
            })
            .collect(),
        _ => AI_THREATS.iter().collect(),
    };

    let mut threat_scores: Vec<serde_json::Value> = Vec::new();
    let mut total_readiness = 0.0;

    for threat in &threats_to_check {
        let mut deployed = Vec::new();
        let mut missing = Vec::new();

        for &defense in threat.defenses {
            let defense_lower = defense.to_lowercase();
            if defenses
                .iter()
                .any(|d| d.contains(&defense_lower) || defense_lower.contains(d.as_str()))
            {
                deployed.push(defense);
            } else {
                missing.push(defense);
            }
        }

        let readiness = if threat.defenses.is_empty() {
            0.0
        } else {
            deployed.len() as f64 / threat.defenses.len() as f64 * 100.0
        };
        total_readiness += readiness;

        let status = match readiness as u64 {
            75..=100 => "protected",
            50..=74 => "partial",
            25..=49 => "vulnerable",
            _ => "exposed",
        };

        threat_scores.push(json!({
            "threat_id": threat.id,
            "threat": threat.name,
            "severity": threat.severity,
            "readiness_pct": (readiness * 10.0).round() / 10.0,
            "status": status,
            "defenses_deployed": deployed,
            "defenses_missing": missing,
        }));
    }

    let avg_readiness = if threats_to_check.is_empty() {
        0.0
    } else {
        total_readiness / threats_to_check.len() as f64
    };

    let result = json!({
        "target": params.target,
        "overall_readiness_pct": (avg_readiness * 10.0).round() / 10.0,
        "threats_assessed": threats_to_check.len(),
        "defenses_declared": defenses.len(),
        "threats": threat_scores,
        "priority_actions": threat_scores.iter()
            .filter(|t| t["status"] == "exposed" || t["status"] == "vulnerable")
            .map(|t| json!({
                "threat": t["threat"],
                "severity": t["severity"],
                "deploy_next": t["defenses_missing"],
            }))
            .collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `security_compliance_gap` — Detailed gap analysis for a single framework.
///
/// Returns control-by-control status with remediation recommendations.
pub fn security_compliance_gap(
    params: SecurityComplianceGapParams,
) -> Result<CallToolResult, McpError> {
    let fw = get_framework(&params.framework).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown framework: '{}'. Available: soc2, hipaa, gdpr, eu_ai_act, nist, iso27001",
                params.framework
            ),
            None,
        )
    })?;

    let implemented: Vec<String> = params
        .implemented_controls
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let mut controls_status: Vec<serde_json::Value> = Vec::new();
    let mut gaps = 0;
    let mut met = 0;

    for control in fw.controls {
        let control_lower = control.name.to_lowercase();
        let is_met = implemented.iter().any(|e| {
            control_lower.contains(e.as_str())
                || control.id.to_lowercase() == *e
                || e.contains(&control.id.to_lowercase())
        });

        if is_met {
            met += 1;
        } else {
            gaps += 1;
        }

        controls_status.push(json!({
            "id": control.id,
            "name": control.name,
            "category": control.category,
            "weight": control.weight,
            "status": if is_met { "implemented" } else { "gap" },
            "priority": if !is_met && control.weight >= 0.9 { "critical" }
                       else if !is_met && control.weight >= 0.7 { "high" }
                       else if !is_met { "medium" }
                       else { "n/a" },
        }));
    }

    let result = json!({
        "framework": fw.full_name,
        "code": fw.name,
        "total_controls": fw.controls.len(),
        "implemented": met,
        "gaps": gaps,
        "compliance_pct": if fw.controls.is_empty() { 0.0 } else {
            (met as f64 / fw.controls.len() as f64 * 1000.0).round() / 10.0
        },
        "controls": controls_status,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
