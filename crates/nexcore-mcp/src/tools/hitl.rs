//! Human-in-the-Loop Pipeline — decision approval queue with review workflow.
//!
//! Inspired by AI Engineering Bible Section 11 (Human Oversight):
//! AI decisions above risk thresholds are queued for human review before
//! execution. Reviewers can approve, reject, or modify recommendations.
//!
//! # Architecture
//!
//! ```text
//! Tool Decision → Risk Check → [Auto-execute if low risk]
//!                      ↓
//!                 [Queue if high risk]
//!                      ↓
//!              HITL Approval Queue
//!                      ↓
//!              Human Reviewer
//!              ├─ Approve → Execute recommendation
//!              ├─ Reject  → Log + archive
//!              └─ Modify  → Execute modified action
//!                      ↓
//!              Feedback → Decision Audit Trail
//! ```
//!
//! # T1 Grounding: ∂(Boundary) + κ(Comparison) + ς(State) + π(Persistence) + →(Causality)
//! - ∂: Risk threshold boundary between auto-execute and human review
//! - κ: Reviewer comparison of recommendation vs evidence
//! - ς: Decision state machine (Pending → Approved/Rejected/Modified/Expired)
//! - π: Persistent audit trail with full provenance
//! - →: Causal chain from signal → recommendation → review → execution

use crate::params::{HitlQueueParams, HitlReviewParams, HitlStatsParams, HitlSubmitParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Data Model
// ============================================================================

/// A decision pending human review.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingDecision {
    /// Unique decision ID
    id: String,
    /// Tool that generated the decision
    tool: String,
    /// Recommended action
    recommendation: String,
    /// Target entity
    target: String,
    /// Risk level string
    risk_level: String,
    /// Numeric risk score (0-10)
    risk_score: f64,
    /// Supporting evidence
    evidence: Option<serde_json::Value>,
    /// Current status: pending, approved, rejected, modified, expired
    status: String,
    /// Assigned reviewer
    assigned_to: Option<String>,
    /// When submitted
    submitted_at: String,
    /// When it expires
    expires_at: String,
    /// Review metadata (populated after review)
    reviewed_by: Option<String>,
    reviewed_at: Option<String>,
    review_comments: Option<String>,
    /// If modified, the new action
    modified_action: Option<String>,
    /// Priority rank (derived from risk_level + risk_score)
    priority: u8,
}

/// Aggregate metrics for the HITL pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HitlMetrics {
    total_submitted: u64,
    total_approved: u64,
    total_rejected: u64,
    total_modified: u64,
    total_expired: u64,
    /// Per-tool submission counts
    by_tool: HashMap<String, u64>,
    /// Per-reviewer activity
    by_reviewer: HashMap<String, ReviewerStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReviewerStats {
    approved: u64,
    rejected: u64,
    modified: u64,
    total_reviews: u64,
}

/// Full HITL state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HitlStore {
    decisions: Vec<PendingDecision>,
    metrics: HitlMetrics,
    next_id: u64,
}

// ============================================================================
// Persistence
// ============================================================================

fn store_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    PathBuf::from(format!("{home}/.claude/hitl"))
}

fn store_path() -> PathBuf {
    store_dir().join("store.json")
}

fn load_store() -> HitlStore {
    let path = store_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        HitlStore::default()
    }
}

fn save_store(store: &HitlStore) -> Result<(), McpError> {
    let dir = store_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| McpError::internal_error(format!("Cannot create hitl dir: {e}"), None))?;
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| McpError::internal_error(format!("Serialize error: {e}"), None))?;
    std::fs::write(store_path(), json)
        .map_err(|e| McpError::internal_error(format!("Write error: {e}"), None))?;
    Ok(())
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn parse_iso(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn hours_since(iso: &str) -> f64 {
    parse_iso(iso)
        .map(|dt| {
            let elapsed = chrono::Utc::now().signed_duration_since(dt);
            elapsed.num_minutes() as f64 / 60.0
        })
        .unwrap_or(999.0)
}

/// Derive priority (lower = more urgent): critical=0, high=1, medium=2, low=3.
fn priority_from_risk(risk_level: &str, risk_score: f64) -> u8 {
    match risk_level.to_lowercase().as_str() {
        "critical" => 0,
        "high" => {
            if risk_score >= 8.0 {
                0
            } else {
                1
            }
        }
        "medium" => 2,
        _ => 3,
    }
}

/// Generate ISO timestamp for N hours in the future.
fn expires_iso(hours: u64) -> String {
    let expires = chrono::Utc::now() + chrono::Duration::hours(hours as i64);
    expires.to_rfc3339()
}

/// Expire any decisions past their expiration timestamp.
fn expire_stale(store: &mut HitlStore) {
    let now = chrono::Utc::now();
    for decision in &mut store.decisions {
        if decision.status == "pending" {
            if let Some(exp) = parse_iso(&decision.expires_at) {
                if now > exp {
                    decision.status = "expired".to_string();
                    store.metrics.total_expired += 1;
                }
            }
        }
    }
}

// ============================================================================
// Audit Log (append-only JSONL)
// ============================================================================

fn append_audit(event: &serde_json::Value) {
    let dir = store_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("audit.jsonl");
    let mut line = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
    line.push('\n');
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// `hitl_submit` — Submit a decision for human review.
///
/// Creates a pending approval entry in the HITL queue. Decisions are prioritized
/// by risk level and score. Each entry has an expiration (default 72 hours).
pub fn hitl_submit(params: HitlSubmitParams) -> Result<CallToolResult, McpError> {
    let mut store = load_store();

    // Expire stale entries
    expire_stale(&mut store);

    let priority = priority_from_risk(&params.risk_level, params.risk_score);
    let expires_hours = params.expires_hours.unwrap_or(72);

    store.next_id += 1;
    let id = format!("HITL-{:04}", store.next_id);
    let now = now_iso();

    let decision = PendingDecision {
        id: id.clone(),
        tool: params.tool.clone(),
        recommendation: params.recommendation.clone(),
        target: params.target.clone(),
        risk_level: params.risk_level.clone(),
        risk_score: params.risk_score,
        evidence: params.evidence.clone(),
        status: "pending".to_string(),
        assigned_to: params.assign_to.clone(),
        submitted_at: now.clone(),
        expires_at: expires_iso(expires_hours),
        reviewed_by: None,
        reviewed_at: None,
        review_comments: None,
        modified_action: None,
        priority,
    };

    // Update metrics
    store.metrics.total_submitted += 1;
    *store
        .metrics
        .by_tool
        .entry(params.tool.clone())
        .or_insert(0) += 1;

    store.decisions.push(decision);

    // Sort by priority (most urgent first), then by submission time
    store.decisions.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then(a.submitted_at.cmp(&b.submitted_at))
    });

    save_store(&store)?;

    // Audit log
    append_audit(&json!({
        "event": "submit",
        "decision_id": id,
        "tool": params.tool,
        "target": params.target,
        "risk_level": params.risk_level,
        "risk_score": params.risk_score,
        "recommendation": params.recommendation,
        "assigned_to": params.assign_to,
        "timestamp": now,
    }));

    // Count pending
    let pending_count = store
        .decisions
        .iter()
        .filter(|d| d.status == "pending")
        .count();

    let urgency = match priority {
        0 => "CRITICAL — immediate review required",
        1 => "HIGH — review within 4 hours",
        2 => "MEDIUM — review within 24 hours",
        _ => "LOW — review at convenience",
    };

    let result = json!({
        "status": "submitted",
        "decision_id": id,
        "priority": priority,
        "urgency": urgency,
        "risk_level": params.risk_level,
        "risk_score": params.risk_score,
        "recommendation": params.recommendation,
        "target": params.target,
        "assigned_to": params.assign_to,
        "expires_hours": expires_hours,
        "queue_depth": pending_count,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `hitl_queue` — View the HITL approval queue.
///
/// Lists pending (or filtered) decisions sorted by priority.
/// Automatically expires stale entries.
pub fn hitl_queue(params: HitlQueueParams) -> Result<CallToolResult, McpError> {
    let mut store = load_store();
    expire_stale(&mut store);
    let _ = save_store(&store);

    let status_filter = params.status.as_deref().unwrap_or("pending");
    let limit = params.limit.unwrap_or(20);

    let filtered: Vec<&PendingDecision> = store
        .decisions
        .iter()
        .filter(|d| {
            if status_filter != "all" && d.status != status_filter {
                return false;
            }
            if let Some(ref rl) = params.risk_level {
                if d.risk_level.to_lowercase() != rl.to_lowercase() {
                    return false;
                }
            }
            if let Some(ref assigned) = params.assigned_to {
                match &d.assigned_to {
                    Some(a) if a == assigned => {}
                    _ => return false,
                }
            }
            true
        })
        .take(limit)
        .collect();

    let entries: Vec<serde_json::Value> = filtered
        .iter()
        .map(|d| {
            let age_hours = hours_since(&d.submitted_at);
            let remaining_hours = if d.status == "pending" {
                let exp_h = hours_since(&d.expires_at);
                if exp_h < 0.0 {
                    Some((-exp_h * 10.0).round() / 10.0)
                } else {
                    Some(0.0)
                }
            } else {
                None
            };

            json!({
                "id": d.id,
                "status": d.status,
                "priority": d.priority,
                "tool": d.tool,
                "target": d.target,
                "recommendation": d.recommendation,
                "risk_level": d.risk_level,
                "risk_score": d.risk_score,
                "assigned_to": d.assigned_to,
                "age_hours": (age_hours * 10.0).round() / 10.0,
                "remaining_hours": remaining_hours,
                "reviewed_by": d.reviewed_by,
                "reviewed_at": d.reviewed_at,
                "review_comments": d.review_comments,
                "modified_action": d.modified_action,
            })
        })
        .collect();

    // Summary counts
    let pending = store
        .decisions
        .iter()
        .filter(|d| d.status == "pending")
        .count();
    let critical_pending = store
        .decisions
        .iter()
        .filter(|d| d.status == "pending" && d.priority == 0)
        .count();

    let result = json!({
        "status": "success",
        "filter": {
            "status": status_filter,
            "risk_level": params.risk_level,
            "assigned_to": params.assigned_to,
        },
        "summary": {
            "total_pending": pending,
            "critical_pending": critical_pending,
            "shown": entries.len(),
        },
        "decisions": entries,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `hitl_review` — Approve, reject, or modify a pending decision.
///
/// Transitions a decision from "pending" to the review outcome.
/// Records reviewer identity, timestamp, comments, and any modifications.
/// Writes to append-only audit log.
pub fn hitl_review(params: HitlReviewParams) -> Result<CallToolResult, McpError> {
    let mut store = load_store();
    expire_stale(&mut store);

    let decision = store
        .decisions
        .iter_mut()
        .find(|d| d.id == params.decision_id)
        .ok_or_else(|| {
            McpError::invalid_params(format!("Decision not found: {}", params.decision_id), None)
        })?;

    // Validate current state
    if decision.status != "pending" {
        return Err(McpError::invalid_params(
            format!(
                "Decision {} is '{}', not 'pending' — cannot review",
                decision.id, decision.status
            ),
            None,
        ));
    }

    let action = params.action.to_lowercase();
    let now = now_iso();
    let original_recommendation = decision.recommendation.clone();
    let target = decision.target.clone();
    let tool = decision.tool.clone();

    match action.as_str() {
        "approve" => {
            decision.status = "approved".to_string();
            store.metrics.total_approved += 1;
        }
        "reject" => {
            decision.status = "rejected".to_string();
            store.metrics.total_rejected += 1;
        }
        "modify" => {
            let modified = params.modified_action.as_deref().ok_or_else(|| {
                McpError::invalid_params(
                    "modified_action is required when action is 'modify'".to_string(),
                    None,
                )
            })?;
            decision.status = "modified".to_string();
            decision.modified_action = Some(modified.to_string());
            store.metrics.total_modified += 1;
        }
        _ => {
            return Err(McpError::invalid_params(
                format!(
                    "Invalid review action: '{}'. Use 'approve', 'reject', or 'modify'",
                    action
                ),
                None,
            ));
        }
    }

    decision.reviewed_by = Some(params.reviewer.clone());
    decision.reviewed_at = Some(now.clone());
    decision.review_comments = params.comments.clone();

    // Update reviewer stats
    let reviewer_stats = store
        .metrics
        .by_reviewer
        .entry(params.reviewer.clone())
        .or_default();
    reviewer_stats.total_reviews += 1;
    match action.as_str() {
        "approve" => reviewer_stats.approved += 1,
        "reject" => reviewer_stats.rejected += 1,
        "modify" => reviewer_stats.modified += 1,
        _ => {}
    }

    // Clone what we need before save
    let decision_id = decision.id.clone();
    let final_status = decision.status.clone();
    let final_action = decision
        .modified_action
        .clone()
        .unwrap_or_else(|| original_recommendation.clone());

    save_store(&store)?;

    // Audit log
    append_audit(&json!({
        "event": "review",
        "decision_id": decision_id,
        "action": action,
        "reviewer": params.reviewer,
        "comments": params.comments,
        "original_recommendation": original_recommendation,
        "final_action": final_action,
        "target": target,
        "tool": tool,
        "timestamp": now,
    }));

    let result = json!({
        "status": "reviewed",
        "decision_id": decision_id,
        "review_action": action,
        "final_status": final_status,
        "reviewer": params.reviewer,
        "comments": params.comments,
        "original_recommendation": original_recommendation,
        "final_action": final_action,
        "target": target,
        "feedback": match action.as_str() {
            "approve" => format!("Decision {} APPROVED. Recommendation '{}' cleared for execution on '{}'.", decision_id, final_action, target),
            "reject" => format!("Decision {} REJECTED. Recommendation '{}' will NOT be executed on '{}'.", decision_id, original_recommendation, target),
            "modify" => format!("Decision {} MODIFIED. Original '{}' changed to '{}' for '{}'.", decision_id, original_recommendation, final_action, target),
            _ => String::new(),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `hitl_stats` — HITL pipeline statistics and health metrics.
///
/// Reports approval rates, review times, per-tool and per-reviewer breakdowns.
pub fn hitl_stats(params: HitlStatsParams) -> Result<CallToolResult, McpError> {
    let mut store = load_store();
    expire_stale(&mut store);
    let _ = save_store(&store);

    let include_tools = params.include_tool_breakdown.unwrap_or(true);
    let include_reviewers = params.include_reviewer_activity.unwrap_or(true);

    let m = &store.metrics;
    let total_reviewed = m.total_approved + m.total_rejected + m.total_modified;
    let approval_rate = if total_reviewed > 0 {
        (m.total_approved + m.total_modified) as f64 / total_reviewed as f64
    } else {
        0.0
    };

    // Compute average review time from reviewed decisions
    let mut review_times: Vec<f64> = Vec::new();
    for d in &store.decisions {
        let is_reviewed = matches!(d.status.as_str(), "approved" | "rejected" | "modified");
        if let (Some(reviewed_at), true) = (&d.reviewed_at, is_reviewed) {
            if let (Some(sub), Some(rev)) = (parse_iso(&d.submitted_at), parse_iso(reviewed_at)) {
                let hours = rev.signed_duration_since(sub).num_minutes() as f64 / 60.0;
                if hours >= 0.0 {
                    review_times.push(hours);
                }
            }
        }
    }

    let avg_review_hours = if review_times.is_empty() {
        0.0
    } else {
        review_times.iter().sum::<f64>() / review_times.len() as f64
    };

    // Status distribution
    let mut status_counts: HashMap<String, usize> = HashMap::new();
    for d in &store.decisions {
        *status_counts.entry(d.status.clone()).or_insert(0) += 1;
    }

    // Risk level distribution of pending
    let mut risk_pending: HashMap<String, usize> = HashMap::new();
    for d in store.decisions.iter().filter(|d| d.status == "pending") {
        *risk_pending.entry(d.risk_level.clone()).or_insert(0) += 1;
    }

    let result = json!({
        "pipeline_health": {
            "total_submitted": m.total_submitted,
            "total_reviewed": total_reviewed,
            "total_expired": m.total_expired,
            "approval_rate_pct": (approval_rate * 10000.0).round() / 100.0,
            "avg_review_time_hours": (avg_review_hours * 100.0).round() / 100.0,
        },
        "outcomes": {
            "approved": m.total_approved,
            "rejected": m.total_rejected,
            "modified": m.total_modified,
            "expired": m.total_expired,
        },
        "queue_status": {
            "status_distribution": status_counts,
            "pending_by_risk": risk_pending,
        },
        "tool_breakdown": if include_tools { Some(&m.by_tool) } else { None },
        "reviewer_activity": if include_reviewers { Some(&m.by_reviewer) } else { None },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
