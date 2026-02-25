//! Session registry — tracks active terminal sessions and enforces limits.
//!
//! Thread-safe registry scoped to the terminal subsystem. Enforces per-tenant
//! session concurrency limits based on `SubscriptionTier` via `SandboxConfig`.
//!
//! ## Primitive Grounding
//!
//! `∂(Boundary) + N(Quantity) + ς(State) + μ(Mapping)`
#![allow(
    clippy::disallowed_types,
    reason = "session lookup is O(1) by ID; TerminalSessionId lacks Ord"
)]

use std::collections::HashMap;
use tokio::sync::RwLock;
use vr_core::ids::{TenantId, TerminalSessionId, UserId};
use vr_core::tenant::SubscriptionTier;

use crate::config::SandboxConfig;
use crate::session::{SessionStatus, TerminalMode, TerminalSession};

/// Thread-safe registry of active terminal sessions.
///
/// Enforces per-tenant concurrency limits and provides lookup by
/// session ID, tenant, or user.
pub struct SessionRegistry {
    sessions: RwLock<HashMap<TerminalSessionId, TerminalSession>>,
}

impl SessionRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a tenant can create a new session given their tier limits.
    ///
    /// Returns `true` if the tenant has not yet reached their max concurrent sessions.
    pub async fn can_create_session(&self, tenant_id: &TenantId, tier: &SubscriptionTier) -> bool {
        let config = SandboxConfig::from_tier(tier);
        let sessions = self.sessions.read().await;
        let active_count = sessions
            .values()
            .filter(|s| s.tenant_id == *tenant_id && s.is_alive())
            .count();

        active_count < usize::try_from(config.max_concurrent_sessions).unwrap_or(usize::MAX)
    }

    /// Register a new session. Returns an error if the tenant's limit is exceeded.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::SessionLimitExceeded` if the tenant has reached
    /// their maximum concurrent sessions.
    pub async fn register(
        &self,
        session: TerminalSession,
        tier: &SubscriptionTier,
    ) -> Result<TerminalSessionId, RegistryError> {
        if !self.can_create_session(&session.tenant_id, tier).await {
            return Err(RegistryError::SessionLimitExceeded {
                tenant_id: session.tenant_id,
                limit: SandboxConfig::from_tier(tier).max_concurrent_sessions,
            });
        }

        let id = session.id;
        let mut sessions = self.sessions.write().await;
        sessions.insert(id, session);
        Ok(id)
    }

    /// Get a snapshot of a session by ID.
    pub async fn get(&self, id: &TerminalSessionId) -> Option<TerminalSession> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }

    /// Get all active sessions for a tenant.
    pub async fn sessions_for_tenant(&self, tenant_id: &TenantId) -> Vec<TerminalSession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.tenant_id == *tenant_id && s.is_alive())
            .cloned()
            .collect()
    }

    /// Get all active sessions for a specific user within a tenant.
    pub async fn sessions_for_user(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Vec<TerminalSession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.tenant_id == *tenant_id && s.user_id == *user_id && s.is_alive())
            .cloned()
            .collect()
    }

    /// Update a session's status (activate, idle, terminate).
    ///
    /// Returns `true` if the session was found and updated.
    pub async fn update_status(&self, id: &TerminalSessionId, status: SessionStatus) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(id) {
            match status {
                SessionStatus::Active => session.activate(),
                SessionStatus::Idle => session.mark_idle(),
                SessionStatus::Terminated => session.terminate(),
                SessionStatus::Creating | SessionStatus::Suspended => {
                    session.status = status;
                }
            }
            true
        } else {
            false
        }
    }

    /// Record activity on a session (resets idle timer).
    pub async fn touch(&self, id: &TerminalSessionId) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(id) {
            session.touch();
            true
        } else {
            false
        }
    }

    /// Terminate a session and return it.
    pub async fn terminate(&self, id: &TerminalSessionId) -> Option<TerminalSession> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(id) {
            session.terminate();
            Some(session.clone())
        } else {
            None
        }
    }

    /// Remove all terminated sessions older than the given duration.
    /// Returns the IDs of cleaned-up sessions.
    pub async fn cleanup_terminated(&self) -> Vec<TerminalSessionId> {
        let mut sessions = self.sessions.write().await;
        let to_remove: Vec<TerminalSessionId> = sessions
            .iter()
            .filter(|(_, s)| s.status == SessionStatus::Terminated)
            .map(|(id, _)| *id)
            .collect();

        for id in &to_remove {
            sessions.remove(id);
        }

        to_remove
    }

    /// Find sessions that have exceeded their idle timeout.
    pub async fn find_idle_expired(&self, tier: &SubscriptionTier) -> Vec<TerminalSessionId> {
        let config = SandboxConfig::from_tier(tier);
        let idle_secs = i64::try_from(config.idle_timeout_secs).unwrap_or(i64::MAX);
        let idle_timeout = nexcore_chrono::Duration::seconds(idle_secs);
        let now = nexcore_chrono::DateTime::now();

        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .filter(|(_, s)| {
                s.is_alive() && now.signed_duration_since(s.last_activity).gt(&idle_timeout)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Total number of sessions in the registry (all states).
    pub async fn total_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Number of active (alive) sessions.
    pub async fn active_count(&self) -> usize {
        self.sessions
            .read()
            .await
            .values()
            .filter(|s| s.is_alive())
            .count()
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from session registry operations.
#[non_exhaustive]
#[derive(Debug)]
pub enum RegistryError {
    /// Tenant has reached their maximum concurrent session limit.
    SessionLimitExceeded {
        /// The tenant that hit the limit.
        tenant_id: TenantId,
        /// The maximum allowed sessions.
        limit: u32,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionLimitExceeded { tenant_id, limit } => {
                write!(f, "tenant {tenant_id} has reached session limit of {limit}")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(tenant: TenantId, user: UserId) -> TerminalSession {
        TerminalSession::new(tenant, user, TerminalMode::Hybrid)
    }

    #[tokio::test]
    async fn register_within_limit() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();
        let user = UserId::new();
        let session = make_session(tenant, user);

        let result = registry
            .register(session, &SubscriptionTier::Explorer)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn register_exceeds_limit() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();

        // Explorer tier allows 1 concurrent session
        let s1 = make_session(tenant, UserId::new());
        let s2 = make_session(tenant, UserId::new());

        let r1 = registry.register(s1, &SubscriptionTier::Explorer).await;
        assert!(r1.is_ok());

        let r2 = registry.register(s2, &SubscriptionTier::Explorer).await;
        assert!(r2.is_err());
    }

    #[tokio::test]
    async fn terminated_session_frees_slot() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();

        let s1 = make_session(tenant, UserId::new());
        let id1 = registry
            .register(s1, &SubscriptionTier::Explorer)
            .await
            .expect("first session should succeed");

        // Terminate s1 to free the slot
        registry.terminate(&id1).await;

        // Now a new session should be allowed
        let s2 = make_session(tenant, UserId::new());
        let r2 = registry.register(s2, &SubscriptionTier::Explorer).await;
        assert!(r2.is_ok());
    }

    #[tokio::test]
    async fn get_session() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();
        let user = UserId::new();
        let session = make_session(tenant, user);
        let id = session.id;

        registry
            .register(session, &SubscriptionTier::Enterprise)
            .await
            .expect("register should succeed");

        let retrieved = registry.get(&id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("session should exist").id, id);
    }

    #[tokio::test]
    async fn sessions_for_tenant() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();
        let other_tenant = TenantId::new();

        let s1 = make_session(tenant, UserId::new());
        let s2 = make_session(tenant, UserId::new());
        let s3 = make_session(other_tenant, UserId::new());

        // Enterprise allows 5 concurrent
        registry
            .register(s1, &SubscriptionTier::Enterprise)
            .await
            .ok();
        registry
            .register(s2, &SubscriptionTier::Enterprise)
            .await
            .ok();
        registry
            .register(s3, &SubscriptionTier::Enterprise)
            .await
            .ok();

        let tenant_sessions = registry.sessions_for_tenant(&tenant).await;
        assert_eq!(tenant_sessions.len(), 2);

        let other_sessions = registry.sessions_for_tenant(&other_tenant).await;
        assert_eq!(other_sessions.len(), 1);
    }

    #[tokio::test]
    async fn cleanup_removes_terminated() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();

        let session = make_session(tenant, UserId::new());
        let id = registry
            .register(session, &SubscriptionTier::Enterprise)
            .await
            .expect("register should succeed");

        registry.terminate(&id).await;
        assert_eq!(registry.total_count().await, 1);

        let cleaned = registry.cleanup_terminated().await;
        assert_eq!(cleaned.len(), 1);
        assert_eq!(registry.total_count().await, 0);
    }

    #[tokio::test]
    async fn touch_reactivates_idle() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();

        let session = make_session(tenant, UserId::new());
        let id = registry
            .register(session, &SubscriptionTier::Enterprise)
            .await
            .expect("register should succeed");

        registry.update_status(&id, SessionStatus::Active).await;
        registry.update_status(&id, SessionStatus::Idle).await;

        let s = registry.get(&id).await.expect("session should exist");
        assert_eq!(s.status, SessionStatus::Idle);

        registry.touch(&id).await;
        let s = registry.get(&id).await.expect("session should exist");
        assert_eq!(s.status, SessionStatus::Active);
    }

    #[tokio::test]
    async fn active_count_tracks_alive() {
        let registry = SessionRegistry::new();
        let tenant = TenantId::new();

        let s1 = make_session(tenant, UserId::new());
        let s2 = make_session(tenant, UserId::new());
        let id1 = registry
            .register(s1, &SubscriptionTier::Enterprise)
            .await
            .expect("register");
        registry
            .register(s2, &SubscriptionTier::Enterprise)
            .await
            .expect("register");

        assert_eq!(registry.active_count().await, 2);

        registry.terminate(&id1).await;
        assert_eq!(registry.active_count().await, 1);
        assert_eq!(registry.total_count().await, 2);
    }
}
