//! Career Transitions MCP tool — compute career transition graph from KSB corpus.
//!
//! Uses cosine similarity over binary KSB component vectors to determine
//! transition probability between pharmacovigilance career roles.
//!
//! Data source: static seed data with 15 PV career roles, each mapped to
//! a subset of the 1,286 KSB components by domain/competency tags.

use crate::params::career::CareerTransitionsParams;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::collections::HashSet;

/// A career role with its KSB component set
struct CareerRole {
    id: &'static str,
    label: &'static str,
    ksb_ids: &'static [u16],
    salary_median: u32,
    salary_trend: f64,
}

/// Static seed data: 15 PV career roles with KSB component indices.
/// KSB indices are synthetic subsets representing domain expertise.
/// In production, these would load from Firestore `career_pathways`.
fn seed_roles() -> Vec<CareerRole> {
    vec![
        CareerRole {
            id: "drug-safety-assoc",
            label: "Drug Safety Associate",
            ksb_ids: &[
                1, 2, 3, 10, 11, 12, 50, 51, 52, 100, 101, 200, 201, 202, 300, 301, 400, 401, 500,
                501,
            ],
            salary_median: 55_000,
            salary_trend: 0.04,
        },
        CareerRole {
            id: "pv-scientist",
            label: "PV Scientist",
            ksb_ids: &[
                1, 2, 3, 10, 11, 12, 13, 50, 51, 52, 53, 100, 101, 102, 200, 201, 202, 203, 300,
                301, 302, 400, 401, 402, 500, 501, 502, 600, 601, 700,
            ],
            salary_median: 75_000,
            salary_trend: 0.05,
        },
        CareerRole {
            id: "signal-analyst",
            label: "Signal Detection Analyst",
            ksb_ids: &[
                1, 2, 3, 10, 11, 12, 50, 51, 52, 53, 54, 55, 100, 101, 102, 103, 200, 201, 300,
                500, 501, 502, 503, 600, 601, 602, 700, 701,
            ],
            salary_median: 80_000,
            salary_trend: 0.06,
        },
        CareerRole {
            id: "qppv",
            label: "QPPV",
            ksb_ids: &[
                1, 2, 3, 4, 5, 10, 11, 12, 13, 14, 50, 51, 52, 53, 100, 101, 102, 103, 104, 200,
                201, 202, 203, 204, 300, 301, 302, 303, 400, 401, 402, 403, 500, 501, 502, 503,
                504, 600, 601, 602, 700, 701, 702, 800, 801, 802, 900, 901,
            ],
            salary_median: 120_000,
            salary_trend: 0.03,
        },
        CareerRole {
            id: "medical-reviewer",
            label: "Medical Reviewer",
            ksb_ids: &[
                1, 2, 3, 10, 11, 50, 51, 100, 101, 102, 200, 201, 202, 203, 204, 205, 300, 301,
                302, 400, 401, 500, 501, 600, 800, 801,
            ],
            salary_median: 90_000,
            salary_trend: 0.04,
        },
        CareerRole {
            id: "aggregate-writer",
            label: "Aggregate Report Writer",
            ksb_ids: &[
                1, 2, 3, 10, 11, 12, 50, 51, 52, 100, 101, 200, 201, 300, 301, 302, 303, 304, 400,
                401, 402, 500, 501, 502, 600, 601,
            ],
            salary_median: 70_000,
            salary_trend: 0.03,
        },
        CareerRole {
            id: "risk-mgmt-specialist",
            label: "Risk Management Specialist",
            ksb_ids: &[
                1, 2, 3, 10, 11, 12, 13, 50, 51, 52, 100, 101, 102, 103, 200, 201, 202, 300, 301,
                400, 401, 402, 403, 500, 501, 502, 503, 504, 600, 601, 602, 700, 701, 800,
            ],
            salary_median: 85_000,
            salary_trend: 0.05,
        },
        CareerRole {
            id: "reg-affairs-pv",
            label: "Regulatory Affairs PV",
            ksb_ids: &[
                1, 2, 3, 10, 11, 50, 51, 100, 101, 200, 201, 300, 301, 302, 400, 401, 402, 403,
                404, 405, 500, 501, 600, 700, 800, 801, 802, 900,
            ],
            salary_median: 82_000,
            salary_trend: 0.04,
        },
        CareerRole {
            id: "pv-ops-manager",
            label: "PV Operations Manager",
            ksb_ids: &[
                1, 2, 3, 4, 10, 11, 12, 50, 51, 52, 100, 101, 102, 200, 201, 202, 300, 301, 302,
                400, 401, 402, 500, 501, 502, 600, 601, 700, 701, 800, 801, 900, 901, 902, 903,
            ],
            salary_median: 95_000,
            salary_trend: 0.04,
        },
        CareerRole {
            id: "pv-director",
            label: "PV Director",
            ksb_ids: &[
                1, 2, 3, 4, 5, 10, 11, 12, 13, 14, 15, 50, 51, 52, 53, 100, 101, 102, 103, 200,
                201, 202, 203, 300, 301, 302, 303, 400, 401, 402, 403, 500, 501, 502, 503, 600,
                601, 602, 700, 701, 702, 800, 801, 802, 900, 901, 902,
            ],
            salary_median: 140_000,
            salary_trend: 0.03,
        },
        CareerRole {
            id: "chief-safety-officer",
            label: "Chief Safety Officer",
            ksb_ids: &[
                1, 2, 3, 4, 5, 6, 10, 11, 12, 13, 14, 15, 16, 50, 51, 52, 53, 54, 100, 101, 102,
                103, 104, 200, 201, 202, 203, 204, 300, 301, 302, 303, 304, 400, 401, 402, 403,
                404, 500, 501, 502, 503, 504, 600, 601, 602, 603, 700, 701, 702, 703, 800, 801,
                802, 803, 900, 901, 902, 903,
            ],
            salary_median: 180_000,
            salary_trend: 0.02,
        },
        CareerRole {
            id: "clinical-safety-sci",
            label: "Clinical Safety Scientist",
            ksb_ids: &[
                1, 2, 3, 10, 11, 12, 50, 51, 52, 53, 100, 101, 102, 103, 200, 201, 202, 203, 204,
                205, 206, 300, 301, 400, 401, 500, 501, 502, 600, 601, 700, 800,
            ],
            salary_median: 95_000,
            salary_trend: 0.05,
        },
        CareerRole {
            id: "benefit-risk-analyst",
            label: "Benefit-Risk Analyst",
            ksb_ids: &[
                1, 2, 3, 10, 11, 12, 50, 51, 52, 53, 54, 100, 101, 102, 103, 200, 201, 300, 301,
                500, 501, 502, 503, 504, 505, 600, 601, 602, 603, 700, 701, 702,
            ],
            salary_median: 88_000,
            salary_trend: 0.06,
        },
        CareerRole {
            id: "pv-systems-specialist",
            label: "PV Systems Specialist",
            ksb_ids: &[
                1, 2, 3, 10, 11, 50, 51, 100, 101, 300, 301, 302, 400, 401, 900, 901, 902, 903,
                904, 905, 906, 1000, 1001, 1002, 1003, 1004, 1005,
            ],
            salary_median: 78_000,
            salary_trend: 0.07,
        },
        CareerRole {
            id: "inspection-readiness",
            label: "Inspection Readiness Lead",
            ksb_ids: &[
                1, 2, 3, 4, 10, 11, 12, 50, 51, 52, 100, 101, 102, 200, 201, 300, 301, 302, 303,
                400, 401, 402, 403, 404, 405, 500, 501, 600, 700, 800, 801, 802, 900, 901,
            ],
            salary_median: 92_000,
            salary_trend: 0.04,
        },
    ]
}

/// Compute cosine similarity between two sets of KSB IDs.
/// Uses binary vector representation (1 if present, 0 otherwise).
fn cosine_similarity(a: &HashSet<u16>, b: &HashSet<u16>) -> f64 {
    let intersection = a.intersection(b).count() as f64;
    let mag_a = (a.len() as f64).sqrt();
    let mag_b = (b.len() as f64).sqrt();
    let denominator = mag_a * mag_b;
    if denominator < f64::EPSILON {
        return 0.0;
    }
    intersection / denominator
}

/// `career_transitions` — Compute career transition graph from KSB corpus.
///
/// Returns nodes (career roles) and edges (transitions) with similarity-based
/// probability scores. Uses cosine similarity over binary KSB component vectors.
pub fn transitions(params: CareerTransitionsParams) -> Result<CallToolResult, McpError> {
    let threshold = params.threshold.unwrap_or(0.15);
    let include_salary = params.include_salary.unwrap_or(false);
    let all_roles = seed_roles();

    // Filter roles if specific IDs requested
    let roles: Vec<&CareerRole> = if let Some(ref role_ids) = params.roles {
        let id_set: HashSet<&str> = role_ids.iter().map(|s| s.as_str()).collect();
        all_roles.iter().filter(|r| id_set.contains(r.id)).collect()
    } else {
        all_roles.iter().collect()
    };

    if roles.is_empty() {
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json!({"error": "No matching roles found", "available_roles": seed_roles().iter().map(|r| r.id).collect::<Vec<_>>()}).to_string(),
        )]));
    }

    // Build KSB sets for each role
    let ksb_sets: Vec<HashSet<u16>> = roles
        .iter()
        .map(|r| r.ksb_ids.iter().copied().collect())
        .collect();

    // Build nodes
    let nodes: Vec<serde_json::Value> = roles
        .iter()
        .map(|r| {
            let mut node = json!({
                "id": r.id,
                "label": r.label,
                "ksb_count": r.ksb_ids.len(),
            });
            if include_salary {
                node["salary_median"] = json!(r.salary_median);
                node["salary_trend"] = json!(r.salary_trend);
            }
            node
        })
        .collect();

    // Compute pairwise edges
    let mut edges = Vec::new();
    for i in 0..roles.len() {
        for j in (i + 1)..roles.len() {
            let sim = cosine_similarity(&ksb_sets[i], &ksb_sets[j]);
            if sim >= threshold {
                let difficulty = 1.0 - sim;
                // Round for cleaner output
                let sim_rounded = (sim * 1000.0).round() / 1000.0;
                let diff_rounded = (difficulty * 1000.0).round() / 1000.0;

                edges.push(json!({
                    "source": roles[i].id,
                    "target": roles[j].id,
                    "probability": sim_rounded,
                    "difficulty": diff_rounded,
                }));
            }
        }
    }

    let result = json!({
        "nodes": nodes,
        "edges": edges,
        "similarity_matrix_size": roles.len(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
