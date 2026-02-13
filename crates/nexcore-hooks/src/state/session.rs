//! Session state management for cognitive integrity hooks.
//!
//! This module provides persistent state tracking across Claude Code sessions,
//! enabling cognitive hooks to:
//! - Track verified crates, APIs, and paths
//! - Manage assumptions with confidence levels
//! - Enforce incremental verification thresholds
//! - Track requirements and scope boundaries
//!
//! State is persisted to `~/.claude/session_state.json` and logs are written
//! to `~/.claude/verified/` for audit trails.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp as f64
pub fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Get the session state file path
pub fn state_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("session_state.json")
}

/// Get the verified evidence directory path
pub fn verified_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("verified")
}

/// Ensure verified directory exists
fn ensure_verified_dir() -> std::io::Result<PathBuf> {
    let dir = verified_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// An assumption made during development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    /// The assumption text
    pub assumption: String,
    /// Confidence level
    pub confidence: String,
    /// Status: assumed, verified, disproven
    pub status: String,
    /// How to verify
    pub verification_method: String,
    /// Timestamp
    pub timestamp: f64,
    /// Evidence
    pub evidence: Option<String>,
}

/// Session state for cognitive integrity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Verified crates
    pub verified_crates: Vec<String>,
    /// Verified APIs
    pub verified_apis: Vec<String>,
    /// Verified paths
    pub verified_paths: Vec<String>,
    /// Assumptions
    pub assumptions: Vec<Assumption>,
    /// Lines since verification
    pub lines_since_verification: u32,
    /// Files since verification
    pub files_since_verification: u32,
    /// Last verification time
    pub last_verification_timestamp: f64,
    /// Last verification result
    pub last_verification_result: String,
    /// Unverified constructs
    pub unverified_constructs: Vec<String>,
    /// Type-level edits (struct/trait/impl) since last verification
    #[serde(default)]
    pub type_level_edits: u32,
    /// Requirements verified flag
    pub requirements_verified: bool,
    /// Explicit requirements
    pub explicit_requirements: Vec<String>,
    /// Implicit requirements
    pub implicit_requirements: Vec<String>,
    /// In scope items
    pub scope_in: Vec<String>,
    /// Out of scope items
    pub scope_out: Vec<String>,
    /// Session start time
    pub session_start: f64,
    /// Last update time
    pub last_update: f64,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            verified_crates: Vec::new(),
            verified_apis: Vec::new(),
            verified_paths: Vec::new(),
            assumptions: Vec::new(),
            lines_since_verification: 0,
            files_since_verification: 0,
            last_verification_timestamp: 0.0,
            last_verification_result: String::new(),
            unverified_constructs: Vec::new(),
            requirements_verified: false,
            explicit_requirements: Vec::new(),
            implicit_requirements: Vec::new(),
            scope_in: Vec::new(),
            scope_out: Vec::new(),
            type_level_edits: 0,
            session_start: now(), // Initialize to current time, not 0!
            last_update: now(),
        }
    }
}

impl SessionState {
    /// Load from file
    pub fn load() -> Self {
        let path = state_file();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    /// Save to file
    pub fn save(&mut self) -> std::io::Result<()> {
        self.last_update = now();
        let path = state_file();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Get session ID (integer timestamp)
    pub fn session_id(&self) -> i64 {
        self.session_start as i64
    }

    /// Add verified crate
    pub fn add_verified_crate(&mut self, name: &str) {
        if !self.verified_crates.contains(&name.to_string()) {
            self.verified_crates.push(name.to_string());
        }
    }

    /// Check if a crate is already verified
    pub fn is_crate_verified(&self, name: &str) -> bool {
        self.verified_crates.contains(&name.to_string())
    }

    /// Add verified API
    pub fn add_verified_api(&mut self, api: &str) {
        if !self.verified_apis.contains(&api.to_string()) {
            self.verified_apis.push(api.to_string());
        }
    }

    /// Add verified path
    pub fn add_verified_path(&mut self, path: &str) {
        if !self.verified_paths.contains(&path.to_string()) {
            self.verified_paths.push(path.to_string());
        }
    }

    /// Add an unverified assumption
    pub fn add_assumption(&mut self, assumption: &str, confidence: &str, method: &str) {
        let entry = Assumption {
            assumption: assumption.to_string(),
            confidence: confidence.to_string(),
            status: "assumed".to_string(),
            verification_method: method.to_string(),
            timestamp: now(),
            evidence: None,
        };
        self.assumptions.push(entry.clone());
        let _ = self.log_assumption(&entry);
    }

    /// Verify an assumption with evidence
    pub fn verify_assumption(&mut self, assumption: &str, evidence: &str) {
        for a in &mut self.assumptions {
            if a.assumption == assumption {
                a.status = "verified".to_string();
                a.evidence = Some(evidence.to_string());
            }
        }
    }

    /// Disprove an assumption with evidence
    pub fn disprove_assumption(&mut self, assumption: &str, evidence: &str) {
        for a in &mut self.assumptions {
            if a.assumption == assumption {
                a.status = "disproven".to_string();
                a.evidence = Some(evidence.to_string());
            }
        }
    }

    /// Get all unverified assumptions
    pub fn get_unverified_assumptions(&self) -> Vec<&Assumption> {
        self.assumptions
            .iter()
            .filter(|a| a.status == "assumed")
            .collect()
    }

    /// Count low-confidence unverified assumptions
    pub fn get_uncertain_count(&self) -> usize {
        self.assumptions
            .iter()
            .filter(|a| a.status == "assumed" && a.confidence == "low")
            .count()
    }

    /// Increment lines
    pub fn increment_lines(&mut self, count: u32) {
        self.lines_since_verification += count;
    }

    /// Increment files
    pub fn increment_files(&mut self) {
        self.files_since_verification += 1;
    }

    /// Increment type-level edit counter (struct/trait/impl changes)
    pub fn increment_type_level_edits(&mut self) {
        self.type_level_edits += 1;
    }

    /// Whether type-level constructs have been modified since last verification
    pub fn has_type_level_edits(&self) -> bool {
        self.type_level_edits > 0
    }

    /// Add unverified construct
    pub fn add_unverified_construct(&mut self, c: &str) {
        if !self.unverified_constructs.contains(&c.to_string()) {
            self.unverified_constructs.push(c.to_string());
        }
    }

    /// Record verification event
    pub fn record_verification(&mut self, result: &str) {
        let ts = now();
        self.lines_since_verification = 0;
        self.files_since_verification = 0;
        self.type_level_edits = 0;
        self.last_verification_timestamp = ts;
        self.last_verification_result = result.to_string();
        if result == "success" {
            let evidence = format!("Verified by compilation at {}", ts);
            for a in &mut self.assumptions {
                if a.status == "assumed" {
                    a.status = "verified".to_string();
                    a.evidence = Some(evidence.clone());
                }
            }
        }
        let _ = self.log_verification(result);
        self.unverified_constructs.clear();
    }

    /// Set requirements verified with scope
    pub fn set_requirements_verified(
        &mut self,
        explicit: Vec<String>,
        implicit: Vec<String>,
        scope_in: Vec<String>,
        scope_out: Vec<String>,
    ) {
        self.requirements_verified = true;
        self.explicit_requirements = explicit.clone();
        self.implicit_requirements = implicit.clone();
        self.scope_in = scope_in.clone();
        self.scope_out = scope_out.clone();
        let _ = self.save_requirements(&explicit, &implicit, &scope_in, &scope_out);
    }

    /// Reset state
    pub fn reset(&mut self) {
        *self = Self {
            session_start: now(),
            ..Default::default()
        };
    }

    fn log_assumption(&self, a: &Assumption) -> std::io::Result<()> {
        let dir = ensure_verified_dir()?;
        let p = dir.join(format!("assumptions_{}.jsonl", self.session_id()));
        let mut f = OpenOptions::new().create(true).append(true).open(p)?;
        writeln!(f, "{}", serde_json::to_string(a)?)?;
        Ok(())
    }

    fn log_verification(&self, result: &str) -> std::io::Result<()> {
        let dir = ensure_verified_dir()?;
        let p = dir.join("verification_chain.jsonl");
        let mut f = OpenOptions::new().create(true).append(true).open(p)?;
        let e = serde_json::json!({"session_id": self.session_id(), "result": result, "timestamp": now()});
        writeln!(f, "{}", e)?;
        Ok(())
    }

    fn save_requirements(
        &self,
        exp: &[String],
        imp: &[String],
        si: &[String],
        so: &[String],
    ) -> std::io::Result<()> {
        let dir = ensure_verified_dir()?;
        let p = dir.join(format!("requirements_{}.md", now() as i64));
        let fmt = |v: &[String], d: &str| {
            if v.is_empty() {
                d.into()
            } else {
                v.iter()
                    .map(|x| format!("- {}", x))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };
        let c = format!(
            "# Requirements\n\n## Explicit\n{}\n\n## Implicit\n{}\n\n## In Scope\n{}\n\n## Out of Scope\n{}\n",
            fmt(exp, "- None"),
            fmt(imp, "- None"),
            fmt(si, "- Not defined"),
            fmt(so, "- Not defined")
        );
        fs::write(p, c)
    }

    /// Generate audit report
    pub fn generate_audit_report(&self) -> String {
        let v = self
            .assumptions
            .iter()
            .filter(|a| a.status == "verified")
            .count();
        let d = self
            .assumptions
            .iter()
            .filter(|a| a.status == "disproven")
            .count();
        let u = self
            .assumptions
            .iter()
            .filter(|a| a.status == "assumed")
            .count();
        format!(
            "# Audit\nSession: {}\nVerified: {}, Disproven: {}, Assumed: {}\nRequirements: {}\n",
            self.session_id(),
            v,
            d,
            u,
            self.requirements_verified
        )
    }
}
