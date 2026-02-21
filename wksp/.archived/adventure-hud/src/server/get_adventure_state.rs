//! Adventure state server functions
//!
//! Communicates with nexcore-brain for session persistence.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Current adventure/session state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdventureState {
    pub session_id: String,
    pub session_name: String,
    pub started_at: String,
    pub duration_mins: u32,
    pub tasks: Vec<TaskInfo>,
    pub skills_used: Vec<SkillInfo>,
    pub tools_called: u32,
    pub tokens_used: u64,
    pub milestones: Vec<Milestone>,
    // Meta-Game Mechanics
    pub level: u32,
    pub basis_xp: u64,
    pub reuse_prestige: u64,
    pub compound_velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub subject: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub invocations: u32,
    pub last_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub achieved_at: String,
    pub description: String,
}

/// Fetch current adventure state from nexcore-brain
#[server]
pub async fn get_adventure_state() -> Result<AdventureState, ServerFnError> {
    // Connect to nexcore-brain via localhost API
    let client = reqwest::Client::new();

    // Try to fetch from nexcore-api brain endpoints
    let response = client
        .get("http://localhost:3030/api/v1/brain/session/current")
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<AdventureState>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            // Fallback to mock data - Level 5 Evolution (matches SAVE_STATE.md)
            Ok(AdventureState {
                session_id: "evolution-session".into(),
                session_name: "NexVigilant: Immune System Evolution".into(),
                started_at: chrono::Utc::now().to_rfc3339(),
                duration_mins: 180,
                tasks: vec![
                    TaskInfo { id: "1".into(), subject: "Create Antitransformer".into(), status: "completed".into(), created_at: "2026-02-09T00:00:00Z".into() },
                    TaskInfo { id: "2".into(), subject: "Wire Adversarial PAMPs".into(), status: "completed".into(), created_at: "2026-02-09T01:00:00Z".into() },
                    TaskInfo { id: "3".into(), subject: "Implement Engram DAMPs".into(), status: "completed".into(), created_at: "2026-02-09T02:00:00Z".into() },
                    TaskInfo { id: "4".into(), subject: "Build Signal Cascade UI".into(), status: "completed".into(), created_at: "2026-02-10T00:00:00Z".into() },
                    TaskInfo { id: "5".into(), subject: "Connect Hormone Feedback".into(), status: "completed".into(), created_at: "2026-02-10T01:00:00Z".into() },
                    TaskInfo { id: "6".into(), subject: "Test Adrenalized Response".into(), status: "completed".into(), created_at: "2026-02-10T02:00:00Z".into() },
                    TaskInfo { id: "7".into(), subject: "Implement SELF-SYNTH Tool".into(), status: "completed".into(), created_at: "2026-02-11T00:00:00Z".into() },
                ],
                skills_used: vec![
                    SkillInfo { name: "vigilance-dev".into(), invocations: 12, last_used: "2026-02-11T05:00:00Z".into() },
                    SkillInfo { name: "guardian-orchestrator".into(), invocations: 8, last_used: "2026-02-11T04:30:00Z".into() },
                    SkillInfo { name: "primitive-extractor".into(), invocations: 6, last_used: "2026-02-11T03:00:00Z".into() },
                    SkillInfo { name: "trust-suite".into(), invocations: 4, last_used: "2026-02-11T02:00:00Z".into() },
                    SkillInfo { name: "chemistry-dev".into(), invocations: 3, last_used: "2026-02-11T01:00:00Z".into() },
                ],
                tools_called: 847,
                tokens_used: 128500,
                milestones: vec![
                    Milestone { name: "Ignition".into(), achieved_at: "2026-02-09T01:00:00Z".into(), description: "Basic PAMP sensing online".into() },
                    Milestone { name: "Acceleration".into(), achieved_at: "2026-02-09T03:00:00Z".into(), description: "DAMP monitoring + Real-time UI".into() },
                    Milestone { name: "Sustain".into(), achieved_at: "2026-02-10T02:00:00Z".into(), description: "Dynamic Hormone Modulation active".into() },
                    Milestone { name: "Evolution".into(), achieved_at: "2026-02-11T00:00:00Z".into(), description: "Self-Synthesizing Primitives unlocked. SUPER-ORGANISM achieved.".into() },
                ],
                level: 5,
                basis_xp: 1800,
                reuse_prestige: 850,
                compound_velocity: 62.40,
            })
        }
    }
}
