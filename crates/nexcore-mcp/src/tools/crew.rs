//! Crew-Mode Orchestration — multi-agent task decomposition and decision fusion.
//!
//! Inspired by CrewAI's multi-agent paradigm (AI Engineering Bible Section 4):
//! instead of sequential pipeline execution, decompose tasks into role-based
//! subtasks that can run in parallel with DAG-constrained dependencies.
//!
//! # Architecture
//!
//! ```text
//! Task → Decompose → [Agent(Analyzer), Agent(Guardian), Agent(Synthesizer)]
//!                          ↓                  ↓                 ↓
//!                     partial results    partial results    (depends on ↑)
//!                          ↓                  ↓                 ↓
//!                          └──────── Fusion Engine ──────────────┘
//!                                          ↓
//!                                   CrewDecision
//! ```
//!
//! # T1 Grounding: σ(Sequence) + μ(Mapping) + Σ(Sum) + ×(Product)
//! - σ: DAG execution order
//! - μ: Role → tool set mapping
//! - Σ: Decision fusion (one-of-N strategies)
//! - ×: Conjunctive combination of partial results

use crate::params::{CrewAssignParams, CrewFuseDecisionsParams, CrewTaskStatusParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// Agent Role Definitions
// ============================================================================

/// Predefined agent roles with their tool permissions and responsibilities.
const ROLE_DEFINITIONS: &[RoleDef] = &[
    RoleDef {
        name: "analyzer",
        description: "Consumes events and data, extracts signals and patterns",
        tools: &[
            "pv_signal_complete",
            "faers_search",
            "faers_drug_events",
            "faers_disproportionality",
            "foundation_fuzzy_search",
            "foundation_concept_grep",
            "guidelines_search",
            "mesh_lookup",
        ],
    },
    RoleDef {
        name: "guardian",
        description: "Assesses risk, enforces safety boundaries, validates decisions",
        tools: &[
            "guardian_evaluate_pv",
            "guardian_homeostasis_tick",
            "vigilance_risk_score",
            "vigilance_safety_margin",
            "adversarial_decision_probe",
            "compliance_assess",
        ],
    },
    RoleDef {
        name: "synthesizer",
        description: "Combines partial results into unified assessments",
        tools: &[
            "pv_signal_complete",
            "pv_naranjo_quick",
            "pv_who_umc_quick",
            "stem_confidence_combine",
            "chemistry_pv_mappings",
        ],
    },
    RoleDef {
        name: "learner",
        description: "Updates beliefs, patterns, and thresholds from outcomes",
        tools: &[
            "synapse_observe",
            "brain_implicit_set",
            "brain_artifact_save",
            "immunity_propose",
        ],
    },
    RoleDef {
        name: "executor",
        description: "Performs remediation actions and system operations",
        tools: &[
            "vigil_emit_event",
            "cytokine_emit",
            "phase4_surveillance_tick",
            "monitoring_health_check",
        ],
    },
];

struct RoleDef {
    name: &'static str,
    description: &'static str,
    tools: &'static [&'static str],
}

// ============================================================================
// Persistent State
// ============================================================================

fn crew_state_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.claude/crew", home)
}

fn crew_task_path(task_id: &str) -> String {
    format!("{}/{}.json", crew_state_dir(), task_id)
}

fn crew_index_path() -> String {
    format!("{}/index.json", crew_state_dir())
}

/// A crew task with agent assignments and DAG dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrewTask {
    id: String,
    description: String,
    created_at: f64,
    status: String, // "pending", "in_progress", "complete", "failed"
    agents: Vec<AgentAssignment>,
    /// DAG: agent_id → list of agent_ids it depends on
    dependencies: HashMap<String, Vec<String>>,
    /// Partial results from each agent
    results: HashMap<String, serde_json::Value>,
    /// Fused decision (if complete)
    fused_decision: Option<serde_json::Value>,
}

/// An agent assigned to a crew task.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentAssignment {
    agent_id: String,
    role: String,
    tools: Vec<String>,
    status: String, // "pending", "running", "complete", "failed"
    result: Option<serde_json::Value>,
}

/// Task index for listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CrewIndex {
    tasks: Vec<CrewIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrewIndexEntry {
    id: String,
    description: String,
    status: String,
    agent_count: usize,
    created_at: f64,
}

impl CrewTask {
    fn save(&self) {
        let dir = crew_state_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(crew_task_path(&self.id), json);
        }
        // Update index
        let mut index = CrewIndex::load();
        index.tasks.retain(|e| e.id != self.id);
        index.tasks.push(CrewIndexEntry {
            id: self.id.clone(),
            description: self.description.clone(),
            status: self.status.clone(),
            agent_count: self.agents.len(),
            created_at: self.created_at,
        });
        index.save();
    }

    fn load(task_id: &str) -> Option<Self> {
        let path_str = crew_task_path(task_id);
        std::fs::read_to_string(&path_str)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    }

    fn execution_order(&self) -> Vec<Vec<String>> {
        // Simple topological sort into parallel levels
        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut resolved: Vec<String> = Vec::new();
        let all_ids: Vec<String> = self.agents.iter().map(|a| a.agent_id.clone()).collect();
        let mut remaining: Vec<String> = all_ids.clone();

        for _ in 0..all_ids.len() + 1 {
            if remaining.is_empty() {
                break;
            }
            let mut level = Vec::new();
            for id in &remaining {
                let deps = self.dependencies.get(id).cloned().unwrap_or_default();
                if deps.iter().all(|d| resolved.contains(d)) {
                    level.push(id.clone());
                }
            }
            if level.is_empty() && !remaining.is_empty() {
                // Cycle detected — just push remaining
                levels.push(remaining.clone());
                break;
            }
            for id in &level {
                remaining.retain(|r| r != id);
                resolved.push(id.clone());
            }
            if !level.is_empty() {
                levels.push(level);
            }
        }
        levels
    }
}

impl CrewIndex {
    fn load() -> Self {
        let path_str = crew_index_path();
        std::fs::read_to_string(&path_str)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    fn save(&self) {
        let dir = crew_state_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(crew_index_path(), json);
        }
    }
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

// ============================================================================
// MCP Tools
// ============================================================================

/// `crew_assign` — create a crew task with role-based agent assignments.
///
/// Decomposes a task into role-based subtasks, assigns tools per role,
/// and computes DAG execution order.
pub fn crew_assign(params: CrewAssignParams) -> Result<CallToolResult, McpError> {
    let task_id = format!("crew-{}", now_secs() as u64);

    // Build agent assignments from requested roles
    let mut agents = Vec::new();
    let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

    for (i, role_name) in params.roles.iter().enumerate() {
        let role_lower = role_name.to_lowercase();
        let role_def = ROLE_DEFINITIONS.iter().find(|r| r.name == role_lower);

        let tools: Vec<String> = match role_def {
            Some(def) => def.tools.iter().map(|t| t.to_string()).collect(),
            None => Vec::new(), // Custom role with no predefined tools
        };

        let agent_id = format!("{}-{}", role_lower, i);

        // Synthesizer depends on analyzer and guardian by convention
        if role_lower == "synthesizer" {
            let dep_ids: Vec<String> = agents
                .iter()
                .filter(|a: &&AgentAssignment| a.role == "analyzer" || a.role == "guardian")
                .map(|a| a.agent_id.clone())
                .collect();
            if !dep_ids.is_empty() {
                dependencies.insert(agent_id.clone(), dep_ids);
            }
        }

        // Learner depends on synthesizer
        if role_lower == "learner" {
            let dep_ids: Vec<String> = agents
                .iter()
                .filter(|a: &&AgentAssignment| a.role == "synthesizer")
                .map(|a| a.agent_id.clone())
                .collect();
            if !dep_ids.is_empty() {
                dependencies.insert(agent_id.clone(), dep_ids);
            }
        }

        // Executor depends on guardian
        if role_lower == "executor" {
            let dep_ids: Vec<String> = agents
                .iter()
                .filter(|a: &&AgentAssignment| a.role == "guardian")
                .map(|a| a.agent_id.clone())
                .collect();
            if !dep_ids.is_empty() {
                dependencies.insert(agent_id.clone(), dep_ids);
            }
        }

        agents.push(AgentAssignment {
            agent_id,
            role: role_lower,
            tools,
            status: "pending".to_string(),
            result: None,
        });
    }

    let task = CrewTask {
        id: task_id.clone(),
        description: params.description.clone(),
        created_at: now_secs(),
        status: "pending".to_string(),
        agents: agents.clone(),
        dependencies: dependencies.clone(),
        results: HashMap::new(),
        fused_decision: None,
    };

    let execution_order = task.execution_order();
    task.save();

    // Build role summaries
    let role_summaries: Vec<serde_json::Value> = agents
        .iter()
        .map(|a| {
            let role_def = ROLE_DEFINITIONS.iter().find(|r| r.name == a.role);
            json!({
                "agent_id": a.agent_id,
                "role": a.role,
                "description": role_def.map(|r| r.description).unwrap_or("custom role"),
                "tools": a.tools,
                "depends_on": dependencies.get(&a.agent_id).unwrap_or(&Vec::new()),
            })
        })
        .collect();

    let result = json!({
        "task_id": task_id,
        "description": params.description,
        "status": "pending",
        "agents": role_summaries,
        "execution_order": execution_order,
        "available_roles": ROLE_DEFINITIONS.iter().map(|r| json!({
            "name": r.name,
            "description": r.description,
            "tool_count": r.tools.len(),
        })).collect::<Vec<_>>(),
        "instructions": "Use the tools listed for each agent to gather results, then call crew_fuse_decisions with partial results to get a unified verdict.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// `crew_task_status` — check progress of a crew task.
pub fn crew_task_status(params: CrewTaskStatusParams) -> Result<CallToolResult, McpError> {
    // If no task_id provided, list all tasks
    if params.task_id.is_none() {
        let index = CrewIndex::load();
        let result = json!({
            "tasks": index.tasks,
            "total": index.tasks.len(),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]));
    }

    let task_id = params.task_id.as_deref().unwrap_or("");
    let task = CrewTask::load(task_id)
        .ok_or_else(|| McpError::invalid_params(format!("Task not found: {}", task_id), None))?;

    let agent_statuses: Vec<serde_json::Value> = task
        .agents
        .iter()
        .map(|a| {
            json!({
                "agent_id": a.agent_id,
                "role": a.role,
                "status": a.status,
                "has_result": a.result.is_some(),
            })
        })
        .collect();

    let completed = task
        .agents
        .iter()
        .filter(|a| a.status == "complete")
        .count();
    let total = task.agents.len();

    let result = json!({
        "task_id": task.id,
        "description": task.description,
        "status": task.status,
        "progress": format!("{}/{}", completed, total),
        "agents": agent_statuses,
        "execution_order": task.execution_order(),
        "has_fused_decision": task.fused_decision.is_some(),
        "fused_decision": task.fused_decision,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// `crew_fuse_decisions` — combine multiple agent partial results into a unified decision.
///
/// Fusion strategies:
/// - `majority_vote`: Most common verdict wins
/// - `weighted_score`: Weighted average of numeric scores
/// - `guardian_veto`: Guardian can override other agents if risk exceeds threshold
/// - `unanimous`: All agents must agree
pub fn crew_fuse_decisions(params: CrewFuseDecisionsParams) -> Result<CallToolResult, McpError> {
    let mut task = CrewTask::load(&params.task_id).ok_or_else(|| {
        McpError::invalid_params(format!("Task not found: {}", params.task_id), None)
    })?;

    // Record partial results
    for (agent_id, result_val) in &params.agent_results {
        if let Some(agent) = task.agents.iter_mut().find(|a| &a.agent_id == agent_id) {
            agent.result = Some(result_val.clone());
            agent.status = "complete".to_string();
        }
        task.results.insert(agent_id.clone(), result_val.clone());
    }

    let strategy = params
        .fusion_strategy
        .as_deref()
        .unwrap_or("weighted_score");

    // Extract verdicts and scores from agent results
    let mut verdicts: Vec<(String, String)> = Vec::new(); // (agent_id, verdict)
    let mut scores: Vec<(String, f64, f64)> = Vec::new(); // (agent_id, score, weight)

    for (agent_id, result_val) in &task.results {
        // Try to extract verdict string
        if let Some(verdict) = result_val
            .get("verdict")
            .or_else(|| result_val.get("level"))
            .or_else(|| result_val.get("status"))
            .and_then(|v| v.as_str())
        {
            verdicts.push((agent_id.clone(), verdict.to_string()));
        }

        // Try to extract numeric score
        if let Some(score) = result_val
            .get("score")
            .or_else(|| result_val.get("severity_score"))
            .or_else(|| result_val.get("risk_score"))
            .and_then(|v| v.as_f64())
        {
            // Guardian gets higher weight
            let agent = task.agents.iter().find(|a| &a.agent_id == agent_id);
            let weight = match agent.map(|a| a.role.as_str()) {
                Some("guardian") => 1.5,
                Some("analyzer") => 1.2,
                Some("synthesizer") => 1.3,
                _ => 1.0,
            };
            scores.push((agent_id.clone(), score, weight));
        }
    }

    // Apply fusion strategy
    let fused = match strategy {
        "majority_vote" => {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for (_, verdict) in &verdicts {
                *counts.entry(verdict.clone()).or_insert(0) += 1;
            }
            let winner = counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(v, c)| (v.clone(), *c));

            json!({
                "strategy": "majority_vote",
                "verdict": winner.as_ref().map(|(v, _)| v.as_str()),
                "votes": counts,
                "total_voters": verdicts.len(),
                "consensus": winner.as_ref().map(|(_, c)| *c == verdicts.len()).unwrap_or(false),
            })
        }
        "weighted_score" => {
            if scores.is_empty() {
                json!({
                    "strategy": "weighted_score",
                    "error": "No numeric scores found in agent results",
                })
            } else {
                let total_weight: f64 = scores.iter().map(|(_, _, w)| w).sum();
                let weighted_sum: f64 = scores.iter().map(|(_, s, w)| s * w).sum();
                let weighted_avg = if total_weight > 0.0 {
                    weighted_sum / total_weight
                } else {
                    0.0
                };

                // Determine verdict from weighted average
                let verdict = if weighted_avg >= 60.0 {
                    "critical"
                } else if weighted_avg >= 35.0 {
                    "high"
                } else if weighted_avg >= 15.0 {
                    "medium"
                } else {
                    "low"
                };

                let contributions: Vec<serde_json::Value> = scores
                    .iter()
                    .map(|(id, score, weight)| {
                        json!({
                            "agent_id": id,
                            "score": score,
                            "weight": weight,
                            "weighted_contribution": score * weight / total_weight,
                        })
                    })
                    .collect();

                json!({
                    "strategy": "weighted_score",
                    "weighted_average": (weighted_avg * 100.0).round() / 100.0,
                    "verdict": verdict,
                    "contributions": contributions,
                    "total_weight": total_weight,
                })
            }
        }
        "guardian_veto" => {
            // Guardian can override if risk is high
            let guardian_result = task
                .results
                .iter()
                .find(|(id, _)| {
                    task.agents
                        .iter()
                        .any(|a| &a.agent_id == *id && a.role == "guardian")
                })
                .map(|(_, v)| v);

            let guardian_score = guardian_result.and_then(|r| {
                r.get("score")
                    .or_else(|| r.get("severity_score"))
                    .and_then(|v| v.as_f64())
            });

            let veto_threshold = 50.0;
            let vetoed = guardian_score.map_or(false, |s| s >= veto_threshold);

            let base_verdict = verdicts
                .iter()
                .find(|(id, _)| {
                    task.agents
                        .iter()
                        .any(|a| a.agent_id == *id && a.role == "synthesizer")
                })
                .or(verdicts.first())
                .map(|(_, v)| v.as_str());

            json!({
                "strategy": "guardian_veto",
                "guardian_score": guardian_score,
                "veto_threshold": veto_threshold,
                "vetoed": vetoed,
                "original_verdict": base_verdict,
                "final_verdict": if vetoed { "blocked_by_guardian" } else { base_verdict.unwrap_or("unknown") },
                "explanation": if vetoed {
                    format!("Guardian score {:.1} exceeds veto threshold ({:.1}) — action blocked", guardian_score.unwrap_or(0.0), veto_threshold)
                } else {
                    format!("Guardian score {:.1} within threshold ({:.1}) — proceeding", guardian_score.unwrap_or(0.0), veto_threshold)
                },
            })
        }
        "unanimous" => {
            let unique_verdicts: Vec<String> = verdicts
                .iter()
                .map(|(_, v)| v.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let unanimous = unique_verdicts.len() == 1;

            json!({
                "strategy": "unanimous",
                "unanimous": unanimous,
                "verdict": if unanimous { unique_verdicts.first().map(|v| v.as_str()) } else { None },
                "dissenting_verdicts": if !unanimous { Some(&unique_verdicts) } else { None },
                "all_verdicts": verdicts,
            })
        }
        _ => {
            json!({
                "error": format!("Unknown fusion strategy: {}", strategy),
                "available_strategies": ["majority_vote", "weighted_score", "guardian_veto", "unanimous"],
            })
        }
    };

    // Mark task complete if all agents have results
    let all_complete = task.agents.iter().all(|a| a.status == "complete");
    if all_complete {
        task.status = "complete".to_string();
    } else {
        task.status = "in_progress".to_string();
    }

    task.fused_decision = Some(fused.clone());
    task.save();

    let result = json!({
        "task_id": task.id,
        "task_status": task.status,
        "agents_complete": task.agents.iter().filter(|a| a.status == "complete").count(),
        "agents_total": task.agents.len(),
        "fusion": fused,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}
