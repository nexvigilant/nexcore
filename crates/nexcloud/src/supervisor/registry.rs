use dashmap::DashMap;
use nexcore_chrono::DateTime;

/// Process lifecycle state machine.
///
/// Tier: T2-P (ς State) — the core FSM for service processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Manifest parsed, dependencies not yet ready.
    Pending,
    /// Process spawned, waiting for health check.
    Starting,
    /// Health checks passing.
    Healthy,
    /// Health check failed.
    Unhealthy,
    /// Backoff wait before respawn.
    Restarting,
    /// SIGTERM sent, awaiting exit.
    Stopping,
    /// Clean exit.
    Stopped,
    /// Exceeded max_restarts or unrecoverable crash.
    Failed,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Starting => write!(f, "starting"),
            Self::Healthy => write!(f, "healthy"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Restarting => write!(f, "restarting"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

/// Record for a single managed service.
///
/// Tier: T2-C (ς State + π Persistence + N Quantity)
/// Persists service state with quantified restart counts.
#[derive(Debug, Clone)]
pub struct ServiceRecord {
    pub name: String,
    pub state: ProcessState,
    pub pid: Option<u32>,
    pub port: u16,
    pub restarts: u32,
    pub started_at: Option<DateTime>,
    pub last_health: Option<DateTime>,
}

/// Concurrent service registry backed by DashMap.
///
/// Tier: T2-C (μ Mapping + ς State + π Persistence)
/// Maps service names to their mutable state records.
pub struct ServiceRegistry {
    inner: DashMap<String, ServiceRecord>,
}

impl ServiceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    /// Register a new service.
    pub fn register(&self, name: String, port: u16) {
        self.inner.insert(
            name.clone(),
            ServiceRecord {
                name,
                state: ProcessState::Pending,
                pid: None,
                port,
                restarts: 0,
                started_at: None,
                last_health: None,
            },
        );
    }

    /// Update the state of a service.
    pub fn update_state(&self, name: &str, state: ProcessState) {
        if let Some(mut record) = self.inner.get_mut(name) {
            record.state = state;
        }
    }

    /// Update the PID of a service.
    pub fn update_pid(&self, name: &str, pid: Option<u32>) {
        if let Some(mut record) = self.inner.get_mut(name) {
            record.pid = pid;
        }
    }

    /// Record a successful health check.
    pub fn record_healthy(&self, name: &str) {
        if let Some(mut record) = self.inner.get_mut(name) {
            record.state = ProcessState::Healthy;
            record.last_health = Some(DateTime::now());
        }
    }

    /// Increment restart counter.
    pub fn increment_restarts(&self, name: &str) {
        if let Some(mut record) = self.inner.get_mut(name) {
            record.restarts += 1;
        }
    }

    /// Mark service as started.
    pub fn mark_started(&self, name: &str, pid: u32) {
        if let Some(mut record) = self.inner.get_mut(name) {
            record.state = ProcessState::Starting;
            record.pid = Some(pid);
            record.started_at = Some(DateTime::now());
        }
    }

    /// Get a snapshot of a service record.
    pub fn get(&self, name: &str) -> Option<ServiceRecord> {
        self.inner.get(name).map(|r| r.clone())
    }

    /// Get a snapshot of all service records.
    pub fn snapshot(&self) -> Vec<ServiceRecord> {
        self.inner.iter().map(|r| r.value().clone()).collect()
    }

    /// Number of registered services.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get() {
        let reg = ServiceRegistry::new();
        reg.register("web".to_string(), 8080);

        let record = reg.get("web");
        assert!(record.is_some());
        let r = record.unwrap_or_else(|| panic!("expected record"));
        assert_eq!(r.name, "web");
        assert_eq!(r.port, 8080);
        assert_eq!(r.state, ProcessState::Pending);
        assert_eq!(r.restarts, 0);
    }

    #[test]
    fn update_state_and_pid() {
        let reg = ServiceRegistry::new();
        reg.register("web".to_string(), 8080);
        reg.mark_started("web", 1234);

        let r = reg.get("web").unwrap_or_else(|| panic!("expected record"));
        assert_eq!(r.state, ProcessState::Starting);
        assert_eq!(r.pid, Some(1234));
        assert!(r.started_at.is_some());
    }

    #[test]
    fn record_healthy() {
        let reg = ServiceRegistry::new();
        reg.register("web".to_string(), 8080);
        reg.record_healthy("web");

        let r = reg.get("web").unwrap_or_else(|| panic!("expected record"));
        assert_eq!(r.state, ProcessState::Healthy);
        assert!(r.last_health.is_some());
    }

    #[test]
    fn increment_restarts() {
        let reg = ServiceRegistry::new();
        reg.register("web".to_string(), 8080);
        reg.increment_restarts("web");
        reg.increment_restarts("web");

        let r = reg.get("web").unwrap_or_else(|| panic!("expected record"));
        assert_eq!(r.restarts, 2);
    }

    #[test]
    fn snapshot_all() {
        let reg = ServiceRegistry::new();
        reg.register("a".to_string(), 3000);
        reg.register("b".to_string(), 3001);
        reg.register("c".to_string(), 3002);

        let snap = reg.snapshot();
        assert_eq!(snap.len(), 3);
        assert_eq!(reg.len(), 3);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let reg = ServiceRegistry::new();
        assert!(reg.get("nope").is_none());
    }

    #[test]
    fn process_state_display() {
        assert_eq!(format!("{}", ProcessState::Healthy), "healthy");
        assert_eq!(format!("{}", ProcessState::Failed), "FAILED");
    }
}
