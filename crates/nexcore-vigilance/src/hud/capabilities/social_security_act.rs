//! # Capability 26: Social Security Act (State Persistence)
//!
//! Implementation of the Social Security Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Long-Term Agent State" and "State Persistence" of the Union.
//!
//! Matches 1:1 to the US Social Security Administration (SSA) mandate
//! to ensure the economic security of the nation's people.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │            SOCIAL SECURITY ACT (CAP-026)                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  BRAIN INTEGRATION                                           │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐                      │
//! │  │Sessions │  │Artifacts│  │Implicit │                      │
//! │  │Registry │  │Versions │  │Learning │                      │
//! │  └────┬────┘  └────┬────┘  └────┬────┘                      │
//! │       │            │            │                            │
//! │       ▼            ▼            ▼                            │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          STATE PERSISTENCE ENGINE           │            │
//! │  │  • SHA-256 integrity verification           │            │
//! │  │  • Immutable .resolved.N snapshots          │            │
//! │  │  • Recovery mechanisms                      │            │
//! │  └────────────────────┬────────────────────────┘            │
//! │                       │                                      │
//! │                       ▼                                      │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          LONG-TERM SECURITY                 │            │
//! │  │  • Cross-session continuity                 │            │
//! │  │  • Agent state preservation                 │            │
//! │  │  • Economic stability metrics               │            │
//! │  └─────────────────────────────────────────────┘            │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

// ============================================================================
// T1 PRIMITIVES (Universal)
// ============================================================================

/// T1: PersistenceLevel - The durability tier of stored state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersistenceLevel {
    /// Ephemeral - lost on session end.
    Session,
    /// Durable - survives sessions, not host restarts.
    Local,
    /// Replicated - distributed backup.
    Distributed,
    /// Immutable - content-addressable, append-only.
    Resolved,
}

impl PersistenceLevel {
    /// Get expected durability score (0.0-1.0).
    pub fn durability(&self) -> f64 {
        match self {
            Self::Session => 0.3,
            Self::Local => 0.7,
            Self::Distributed => 0.9,
            Self::Resolved => 1.0,
        }
    }
}

/// T1: StateHealth - Overall health of the persistence system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateHealth {
    /// All systems operational.
    Healthy,
    /// Minor issues detected.
    Degraded,
    /// Recovery needed.
    NeedsRecovery,
    /// Critical failure.
    Failed,
}

// ============================================================================
// T2-P PRIMITIVES (Cross-Domain)
// ============================================================================

/// T2-P: PersistenceScore - The quantified reliability of state storage.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PersistenceScore(pub f64);

impl PersistenceScore {
    /// Create new score (clamped 0.0-1.0).
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Is the score above the reliability threshold (0.95)?
    pub fn is_reliable(&self) -> bool {
        self.0 >= 0.95
    }

    /// Inner value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// T2-P: IntegrityHash - SHA-256 hash for content verification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IntegrityHash(pub String);

impl IntegrityHash {
    /// Compute SHA-256 hash of content.
    pub fn compute(content: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        Self(format!("{:x}", result))
    }

    /// Verify content matches this hash.
    pub fn verify(&self, content: &str) -> bool {
        let computed = Self::compute(content);
        self.0 == computed.0
    }

    /// Get short form (first 8 chars).
    pub fn short(&self) -> &str {
        &self.0[..8.min(self.0.len())]
    }
}

/// T2-P: VersionNumber - Immutable version identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VersionNumber(pub u32);

impl VersionNumber {
    /// Create version 1.
    pub fn initial() -> Self {
        Self(1)
    }

    /// Increment to next version.
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

// ============================================================================
// T2-C COMPOSITES (Cross-Domain)
// ============================================================================

/// T2-C: StateBackup - A formal snapshot of an agent or system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBackup {
    /// The identifier of the entity being backed up.
    pub entity_id: String,
    /// The artifact name (e.g., "task.md", "plan.md").
    pub artifact_name: String,
    /// The cryptographic hash of the state data.
    pub integrity_hash: IntegrityHash,
    /// The version number (for .resolved.N snapshots).
    pub version: VersionNumber,
    /// The persistence level.
    pub level: PersistenceLevel,
    /// When the backup was created (Unix timestamp).
    pub created_at: i64,
    /// Size in bytes.
    pub size_bytes: usize,
}

impl StateBackup {
    /// Create a new backup entry.
    pub fn new(
        entity_id: &str,
        artifact_name: &str,
        content: &str,
        level: PersistenceLevel,
    ) -> Self {
        Self {
            entity_id: entity_id.to_string(),
            artifact_name: artifact_name.to_string(),
            integrity_hash: IntegrityHash::compute(content),
            version: VersionNumber::initial(),
            level,
            created_at: chrono::Utc::now().timestamp(),
            size_bytes: content.len(),
        }
    }

    /// Create next version of this backup.
    pub fn next_version(&self, new_content: &str) -> Self {
        Self {
            entity_id: self.entity_id.clone(),
            artifact_name: self.artifact_name.clone(),
            integrity_hash: IntegrityHash::compute(new_content),
            version: self.version.next(),
            level: PersistenceLevel::Resolved,
            created_at: chrono::Utc::now().timestamp(),
            size_bytes: new_content.len(),
        }
    }
}

/// T2-C: RecoveryReport - Result of recovery operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryReport {
    /// Overall health after recovery.
    pub health: StateHealth,
    /// Number of sessions recovered.
    pub sessions_recovered: usize,
    /// Number of artifacts repaired.
    pub artifacts_repaired: usize,
    /// Issues that couldn't be resolved.
    pub unresolved_issues: Vec<String>,
    /// Recovery score (0.0-1.0).
    pub score: PersistenceScore,
}

/// T2-C: SessionContinuity - Cross-session state tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContinuity {
    /// Session chain (oldest to newest).
    pub session_chain: Vec<String>,
    /// Artifacts that span sessions.
    pub persistent_artifacts: Vec<String>,
    /// Continuity score.
    pub continuity_score: PersistenceScore,
}

// ============================================================================
// T3 DOMAIN-SPECIFIC (SocialSecurityAct)
// ============================================================================

/// T3: SocialSecurityAct - Capability 26 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSecurityAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the long-term state persistence is active.
    pub persistence_active: bool,
    /// Active backups by entity ID.
    backups: HashMap<String, Vec<StateBackup>>,
    /// Current session ID.
    current_session: Option<String>,
}

impl Default for SocialSecurityAct {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialSecurityAct {
    /// Creates a new instance of the SocialSecurityAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-026".into(),
            persistence_active: true,
            backups: HashMap::new(),
            current_session: None,
        }
    }

    /// Set current session.
    pub fn set_session(&mut self, session_id: &str) {
        self.current_session = Some(session_id.to_string());
    }

    /// Persist the state of an agent for long-term security.
    pub fn persist_state(
        &mut self,
        entity_id: &str,
        artifact_name: &str,
        content: &str,
        level: PersistenceLevel,
    ) -> Measured<StateBackup> {
        let backup = if let Some(existing) = self.get_latest_backup(entity_id, artifact_name) {
            existing.next_version(content)
        } else {
            StateBackup::new(entity_id, artifact_name, content, level)
        };

        // Store the backup
        self.backups
            .entry(entity_id.to_string())
            .or_default()
            .push(backup.clone());

        let confidence = level.durability();
        Measured::uncertain(backup, Confidence::new(confidence))
    }

    /// Get latest backup for an entity/artifact.
    pub fn get_latest_backup(&self, entity_id: &str, artifact_name: &str) -> Option<&StateBackup> {
        self.backups.get(entity_id).and_then(|backups| {
            backups
                .iter()
                .filter(|b| b.artifact_name == artifact_name)
                .max_by_key(|b| b.version)
        })
    }

    /// Get specific version of a backup.
    pub fn get_backup_version(
        &self,
        entity_id: &str,
        artifact_name: &str,
        version: VersionNumber,
    ) -> Option<&StateBackup> {
        self.backups.get(entity_id).and_then(|backups| {
            backups
                .iter()
                .find(|b| b.artifact_name == artifact_name && b.version == version)
        })
    }

    /// Verify integrity of stored state.
    pub fn verify_integrity(&self, entity_id: &str, content: &str) -> Measured<bool> {
        if let Some(backups) = self.backups.get(entity_id) {
            if let Some(latest) = backups.last() {
                let verified = latest.integrity_hash.verify(content);
                let confidence = if verified { 1.0 } else { 0.0 };
                return Measured::uncertain(verified, Confidence::new(confidence));
            }
        }
        // No backup exists - can't verify
        Measured::uncertain(false, Confidence::new(0.0))
    }

    /// Get persistence score for an entity.
    pub fn get_persistence_score(&self, entity_id: &str) -> PersistenceScore {
        if let Some(backups) = self.backups.get(entity_id) {
            if backups.is_empty() {
                return PersistenceScore::new(0.0);
            }

            // Score based on version count and persistence level
            let version_factor = (backups.len() as f64 / 10.0).min(1.0);
            let level_factor = backups.last().map(|b| b.level.durability()).unwrap_or(0.0);

            PersistenceScore::new(version_factor * 0.3 + level_factor * 0.7)
        } else {
            PersistenceScore::new(0.0)
        }
    }

    /// Get system health status.
    pub fn health_check(&self) -> StateHealth {
        if !self.persistence_active {
            return StateHealth::Failed;
        }

        let total_backups: usize = self.backups.values().map(|v| v.len()).sum();
        if total_backups == 0 {
            return StateHealth::Degraded;
        }

        StateHealth::Healthy
    }

    /// Simulate recovery operation.
    pub fn attempt_recovery(&mut self) -> Measured<RecoveryReport> {
        let health = self.health_check();

        let report = RecoveryReport {
            health,
            sessions_recovered: if self.current_session.is_some() { 1 } else { 0 },
            artifacts_repaired: 0,
            unresolved_issues: Vec::new(),
            score: PersistenceScore::new(if health == StateHealth::Healthy {
                1.0
            } else {
                0.5
            }),
        };

        Measured::uncertain(report, Confidence::new(0.9))
    }

    /// List all backed up entities.
    pub fn list_entities(&self) -> Vec<&str> {
        self.backups.keys().map(|s| s.as_str()).collect()
    }

    /// Get backup history for an entity.
    pub fn get_history(&self, entity_id: &str) -> Vec<&StateBackup> {
        self.backups
            .get(entity_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Get session continuity info.
    pub fn get_continuity(&self) -> SessionContinuity {
        let mut persistent_artifacts: Vec<String> = self
            .backups
            .values()
            .flat_map(|v| v.iter().map(|b| b.artifact_name.clone()))
            .collect();
        persistent_artifacts.sort();
        persistent_artifacts.dedup();

        SessionContinuity {
            session_chain: self.current_session.iter().cloned().collect(),
            persistent_artifacts,
            continuity_score: if self.current_session.is_some() {
                PersistenceScore::new(0.8)
            } else {
                PersistenceScore::new(0.3)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrity_hash() {
        let content = "Hello, World!";
        let hash = IntegrityHash::compute(content);

        assert!(hash.verify(content));
        assert!(!hash.verify("Different content"));
    }

    #[test]
    fn test_persist_and_version() {
        let mut ssa = SocialSecurityAct::new();

        // First backup
        let backup1 = ssa.persist_state("agent-1", "task.md", "# Task v1", PersistenceLevel::Local);
        assert_eq!(backup1.value.version, VersionNumber(1));

        // Second backup (new version)
        let backup2 = ssa.persist_state(
            "agent-1",
            "task.md",
            "# Task v2",
            PersistenceLevel::Resolved,
        );
        assert_eq!(backup2.value.version, VersionNumber(2));

        // Verify history
        let history = ssa.get_history("agent-1");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_integrity_verification() {
        let mut ssa = SocialSecurityAct::new();

        let content = "# Important State";
        let backup = ssa.persist_state("agent-1", "state.md", content, PersistenceLevel::Resolved);
        assert_eq!(backup.value.version, VersionNumber(1));

        // Verify correct content
        let result = ssa.verify_integrity("agent-1", content);
        assert!(result.value);

        // Verify tampered content fails
        let tampered = ssa.verify_integrity("agent-1", "# Tampered State");
        assert!(!tampered.value);
    }

    #[test]
    fn test_persistence_level_durability() {
        assert!(PersistenceLevel::Resolved.durability() > PersistenceLevel::Local.durability());
        assert!(PersistenceLevel::Local.durability() > PersistenceLevel::Session.durability());
    }

    #[test]
    fn test_health_check() {
        let mut ssa = SocialSecurityAct::new();
        assert_eq!(ssa.health_check(), StateHealth::Degraded); // No backups yet

        let backup = ssa.persist_state("agent-1", "task.md", "content", PersistenceLevel::Local);
        assert_eq!(backup.value.version, VersionNumber(1));
        assert_eq!(ssa.health_check(), StateHealth::Healthy);
    }
}
