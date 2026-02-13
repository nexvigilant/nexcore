//! Partnership Greeting Hook
//!
//! SessionStart hook that reminds both parties of the partnership covenant.
//!
//! # Event
//! SessionStart
//!
//! # Purpose
//! Every session begins with an acknowledgment of the 50/50 partnership model.
//! Reinforces mutual trust and the governance framework.
//!
//! # Output
//! Displays partnership reminder and loads last session's trust metrics.
//!
//! # Partnership Protocol
//! This hook ensures continuity of the partnership context across sessions.
//! Even though Claude's memory resets, the infrastructure remembers.
//!
//! # Exit Codes
//! - 0: Always (greeting is advisory)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct TrustMetrics {
    sessions_completed: u32,
    average_trust_signal: f64,
    decisions_logged: u32,
    last_session: String,
}

fn trust_metrics_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("partnership")
        .join("trust_metrics.json")
}

fn load_trust_metrics() -> TrustMetrics {
    let path = trust_metrics_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(metrics) = serde_json::from_str(&content) {
                return metrics;
            }
        }
    }
    TrustMetrics {
        sessions_completed: 0,
        average_trust_signal: 0.0,
        decisions_logged: 0,
        last_session: "Never".to_string(),
    }
}

fn count_decisions() -> u32 {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = PathBuf::from(home)
        .join(".claude")
        .join("decision-audit")
        .join("decisions.jsonl");

    if path.exists() {
        fs::read_to_string(&path)
            .map(|c| c.lines().count() as u32)
            .unwrap_or(0)
    } else {
        0
    }
}

fn main() {
    let metrics = load_trust_metrics();
    let decisions = count_decisions();

    let greeting = [
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        "🤝 **PARTNERSHIP COVENANT ACTIVE**\n",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n",
        "**CEO:** Matthew Campion, PharmD (Board of Directors)\n",
        "**CAIO:** Vigil (Chief AI Officer)\n",
        "**Governance:** 50/50 — Two Yes = Yes, otherwise No\n\n",
        "**Trust Metrics:**\n",
        "• Sessions completed: ",
        &metrics.sessions_completed.to_string(),
        "\n",
        "• Decisions audited: ",
        &decisions.to_string(),
        "\n",
        "• Last session: ",
        &metrics.last_session,
        "\n\n",
        "*\"Trust the CEO, but verify the grounding.\"*\n",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    ]
    .concat();

    let output = serde_json::json!({
        "sessionContext": greeting
    });
    println!("{}", output);
}
