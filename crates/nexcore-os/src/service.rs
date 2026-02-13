// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Service manager — NexCore OS service lifecycle.
//!
//! Each system service (Guardian, Brain, Vigil, etc.) runs as a managed
//! service with defined state transitions.
//!
//! ## Primitive Grounding
//!
//! - ς State: Service lifecycle states
//! - σ Sequence: Ordered startup/shutdown
//! - Σ Sum: Service enumeration
//! - ∃ Existence: Service availability checks

use crate::error::ServiceError;

/// Unique service identifier.
///
/// Tier: T2-P (∃ Existence — service identity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ServiceId(u32);

impl ServiceId {
    /// Create a new service ID.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Service lifecycle state.
///
/// Tier: T2-P (ς State — service state machine)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Service is registered but not started.
    Registered,
    /// Service is starting up.
    Starting,
    /// Service is running normally.
    Running,
    /// Service is in degraded mode (partial functionality).
    Degraded,
    /// Service is shutting down.
    Stopping,
    /// Service has stopped.
    Stopped,
    /// Service has failed.
    Failed,
}

impl ServiceState {
    /// Whether the service is considered "alive" (accepting requests).
    pub const fn is_alive(&self) -> bool {
        matches!(self, Self::Running | Self::Degraded)
    }

    /// Whether the service can be started.
    pub const fn can_start(&self) -> bool {
        matches!(self, Self::Registered | Self::Stopped | Self::Failed)
    }
}

/// Service priority level (determines startup order).
///
/// Tier: T2-P (σ Sequence — ordering within boot sequence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServicePriority {
    /// Critical OS services (STOS, Clearance) — start first.
    Critical = 0,
    /// Core services (Guardian, Energy) — start second.
    Core = 1,
    /// Standard services (Brain, Vigil) — start third.
    Standard = 2,
    /// User services (Shell, apps) — start last.
    User = 3,
}

/// A managed system service.
///
/// Tier: T3 (ς State + σ Sequence + ∃ Existence + Σ Sum)
#[derive(Debug, Clone)]
pub struct Service {
    /// Unique identifier.
    pub id: ServiceId,
    /// Human-readable name.
    pub name: String,
    /// Current state.
    pub state: ServiceState,
    /// Startup priority.
    pub priority: ServicePriority,
    /// Services this one depends on (must start first).
    pub dependencies: Vec<ServiceId>,
    /// STOS machine ID (if tracked by state kernel).
    pub machine_id: Option<u64>,
}

impl Service {
    /// Create a new service definition.
    pub fn new(id: ServiceId, name: impl Into<String>, priority: ServicePriority) -> Self {
        Self {
            id,
            name: name.into(),
            state: ServiceState::Registered,
            priority,
            dependencies: Vec::new(),
            machine_id: None,
        }
    }

    /// Add a dependency.
    #[must_use]
    pub fn depends_on(mut self, dep: ServiceId) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Transition to a new state.
    pub fn transition(&mut self, new_state: ServiceState) -> Result<(), ServiceError> {
        // Validate transition
        let valid = matches!(
            (&self.state, &new_state),
            (
                ServiceState::Registered | ServiceState::Stopped | ServiceState::Failed,
                ServiceState::Starting,
            ) | (
                ServiceState::Starting | ServiceState::Degraded,
                ServiceState::Running,
            ) | (
                ServiceState::Starting
                    | ServiceState::Running
                    | ServiceState::Degraded
                    | ServiceState::Stopping,
                ServiceState::Failed,
            ) | (
                ServiceState::Running | ServiceState::Degraded,
                ServiceState::Stopping,
            ) | (ServiceState::Running, ServiceState::Degraded)
                | (ServiceState::Stopping, ServiceState::Stopped)
        );

        if valid {
            self.state = new_state;
            Ok(())
        } else {
            Err(ServiceError::StartFailed(format!(
                "{}: invalid transition {:?} -> {:?}",
                self.name, self.state, new_state
            )))
        }
    }
}

/// Service manager — tracks and orchestrates system services.
///
/// Tier: T3 (σ Sequence + ς State + Σ Sum)
pub struct ServiceManager {
    /// Registered services.
    services: Vec<Service>,
    /// Next service ID counter.
    next_id: u32,
}

impl ServiceManager {
    /// Create a new service manager.
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            next_id: 1,
        }
    }

    /// Register a new service and return its ID.
    pub fn register(&mut self, name: impl Into<String>, priority: ServicePriority) -> ServiceId {
        let id = ServiceId::new(self.next_id);
        self.next_id += 1;
        let service = Service::new(id, name, priority);
        self.services.push(service);
        id
    }

    /// Get a service by ID.
    pub fn get(&self, id: ServiceId) -> Option<&Service> {
        self.services.iter().find(|s| s.id == id)
    }

    /// Get a mutable service by ID.
    pub fn get_mut(&mut self, id: ServiceId) -> Option<&mut Service> {
        self.services.iter_mut().find(|s| s.id == id)
    }

    /// Get services sorted by startup priority.
    pub fn startup_order(&self) -> Vec<&Service> {
        let mut sorted: Vec<&Service> = self.services.iter().collect();
        sorted.sort_by_key(|s| s.priority);
        sorted
    }

    /// Count services in a given state.
    pub fn count_in_state(&self, state: ServiceState) -> usize {
        self.services.iter().filter(|s| s.state == state).count()
    }

    /// Total number of registered services.
    pub fn count(&self) -> usize {
        self.services.len()
    }

    /// List all services with their names and current states.
    pub fn list(&self) -> Vec<(&str, ServiceState)> {
        let mut result: Vec<_> = self
            .services
            .iter()
            .map(|s| (s.name.as_str(), s.state))
            .collect();
        result.sort_by_key(|(name, _)| *name);
        result
    }

    /// Check if all dependencies for a service are satisfied (running).
    pub fn dependencies_met(&self, id: ServiceId) -> bool {
        self.get(id).is_some_and(|service| {
            service
                .dependencies
                .iter()
                .all(|dep_id| self.get(*dep_id).is_some_and(|dep| dep.state.is_alive()))
        })
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_creation() {
        let svc = Service::new(ServiceId::new(1), "guardian", ServicePriority::Core);
        assert_eq!(svc.state, ServiceState::Registered);
        assert_eq!(svc.name, "guardian");
    }

    #[test]
    fn service_valid_transitions() {
        let mut svc = Service::new(ServiceId::new(1), "test", ServicePriority::Standard);
        assert!(svc.transition(ServiceState::Starting).is_ok());
        assert!(svc.transition(ServiceState::Running).is_ok());
        assert!(svc.transition(ServiceState::Stopping).is_ok());
        assert!(svc.transition(ServiceState::Stopped).is_ok());
    }

    #[test]
    fn service_invalid_transition() {
        let mut svc = Service::new(ServiceId::new(1), "test", ServicePriority::Standard);
        // Can't go directly from Registered to Running
        assert!(svc.transition(ServiceState::Running).is_err());
    }

    #[test]
    fn service_recovery() {
        let mut svc = Service::new(ServiceId::new(1), "test", ServicePriority::Standard);
        assert!(svc.transition(ServiceState::Starting).is_ok());
        assert!(svc.transition(ServiceState::Failed).is_ok());
        // Can restart from failed
        assert!(svc.transition(ServiceState::Starting).is_ok());
    }

    #[test]
    fn service_manager_register() {
        let mut mgr = ServiceManager::new();
        let id1 = mgr.register("guardian", ServicePriority::Core);
        let id2 = mgr.register("brain", ServicePriority::Standard);
        assert_ne!(id1, id2);
        assert_eq!(mgr.count(), 2);
    }

    #[test]
    fn startup_order() {
        let mut mgr = ServiceManager::new();
        mgr.register("shell", ServicePriority::User);
        mgr.register("guardian", ServicePriority::Core);
        mgr.register("stos", ServicePriority::Critical);
        mgr.register("brain", ServicePriority::Standard);

        let order = mgr.startup_order();
        assert_eq!(order[0].name, "stos");
        assert_eq!(order[1].name, "guardian");
        assert_eq!(order[2].name, "brain");
        assert_eq!(order[3].name, "shell");
    }

    #[test]
    fn state_alive_check() {
        assert!(ServiceState::Running.is_alive());
        assert!(ServiceState::Degraded.is_alive());
        assert!(!ServiceState::Stopped.is_alive());
        assert!(!ServiceState::Failed.is_alive());
    }

    #[test]
    fn dependencies_check() {
        let mut mgr = ServiceManager::new();
        let guardian_id = mgr.register("guardian", ServicePriority::Core);
        let brain_id = mgr.register("brain", ServicePriority::Standard);

        // Brain depends on Guardian
        if let Some(brain) = mgr.get_mut(brain_id) {
            brain.dependencies.push(guardian_id);
        }

        // Guardian not running yet — deps not met
        assert!(!mgr.dependencies_met(brain_id));

        // Start Guardian
        if let Some(guardian) = mgr.get_mut(guardian_id) {
            let _ = guardian.transition(ServiceState::Starting);
            let _ = guardian.transition(ServiceState::Running);
        }

        // Now deps are met
        assert!(mgr.dependencies_met(brain_id));
    }
}
