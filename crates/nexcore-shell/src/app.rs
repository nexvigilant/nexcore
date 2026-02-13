// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! App registry — tracks installed and running applications.
//!
//! ## Primitive Grounding
//!
//! - ∃ Existence: App registration (installed/available)
//! - ς State: App lifecycle (installed → launched → running → stopped)
//! - σ Sequence: App launch order, recent apps list

use serde::{Deserialize, Serialize};

use nexcore_compositor::SurfaceId;

/// Unique application identifier.
///
/// Tier: T2-P (∃ Existence — app identity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(String);

impl AppId {
    /// Create a new app ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the raw ID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// App lifecycle state.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppState {
    /// Installed but not running.
    Installed,
    /// Currently launching.
    Launching,
    /// Running and visible.
    Running,
    /// Running but in background.
    Background,
    /// Stopped (can be re-launched).
    Stopped,
}

/// A registered application.
///
/// Tier: T3 (∃ + ς + σ — identity, state, and ordering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    /// Unique identifier.
    pub id: AppId,
    /// Display name.
    pub name: String,
    /// Current state.
    pub state: AppState,
    /// Associated compositor surface (when running).
    #[serde(skip)]
    pub surface_id: Option<SurfaceId>,
}

impl App {
    /// Create a new app registration.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: AppId::new(id),
            name: name.into(),
            state: AppState::Installed,
            surface_id: None,
        }
    }

    /// Whether the app is currently active (launching or running).
    pub fn is_active(&self) -> bool {
        matches!(self.state, AppState::Launching | AppState::Running)
    }
}

/// App registry — manages installed and running apps.
///
/// Tier: T3 (Σ Sum + σ Sequence — ordered collection of apps)
pub struct AppRegistry {
    /// All registered apps.
    apps: Vec<App>,
    /// Currently focused app.
    focused: Option<AppId>,
}

impl AppRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            focused: None,
        }
    }

    /// Register a new app.
    pub fn register(&mut self, id: impl Into<String>, name: impl Into<String>) -> AppId {
        let app = App::new(id, name);
        let app_id = app.id.clone();
        self.apps.push(app);
        app_id
    }

    /// Get an app by ID.
    pub fn get(&self, id: &AppId) -> Option<&App> {
        self.apps.iter().find(|a| &a.id == id)
    }

    /// Get a mutable app by ID.
    pub fn get_mut(&mut self, id: &AppId) -> Option<&mut App> {
        self.apps.iter_mut().find(|a| &a.id == id)
    }

    /// Launch an app (transitions to Launching state).
    pub fn launch(&mut self, id: &AppId) -> bool {
        if let Some(app) = self.get_mut(id) {
            if app.state == AppState::Installed || app.state == AppState::Stopped {
                app.state = AppState::Launching;
                return true;
            }
        }
        false
    }

    /// Mark an app as running with its compositor surface.
    pub fn set_running(&mut self, id: &AppId, surface_id: SurfaceId) {
        if let Some(app) = self.get_mut(id) {
            app.state = AppState::Running;
            app.surface_id = Some(surface_id);
            self.focused = Some(id.clone());
        }
    }

    /// Stop an app.
    pub fn stop(&mut self, id: &AppId) {
        if let Some(app) = self.get_mut(id) {
            app.state = AppState::Stopped;
            app.surface_id = None;
            if self.focused.as_ref() == Some(id) {
                self.focused = None;
            }
        }
    }

    /// Get the currently focused app.
    pub fn focused(&self) -> Option<&App> {
        self.focused.as_ref().and_then(|id| self.get(id))
    }

    /// Number of registered apps.
    pub fn count(&self) -> usize {
        self.apps.len()
    }

    /// Number of running apps.
    pub fn running_count(&self) -> usize {
        self.apps.iter().filter(|a| a.is_active()).count()
    }

    /// List all apps.
    pub fn list(&self) -> &[App] {
        &self.apps
    }
}

impl Default for AppRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_app() {
        let mut reg = AppRegistry::new();
        let id = reg.register("guardian", "Guardian");
        assert_eq!(reg.count(), 1);
        let app = reg.get(&id);
        assert!(app.is_some());
        if let Some(a) = app {
            assert_eq!(a.name, "Guardian");
            assert_eq!(a.state, AppState::Installed);
        }
    }

    #[test]
    fn app_lifecycle() {
        let mut reg = AppRegistry::new();
        let id = reg.register("test-app", "Test App");

        // Install → Launch
        assert!(reg.launch(&id));
        assert_eq!(reg.get(&id).map(|a| a.state), Some(AppState::Launching));

        // Launch → Running
        let surface = SurfaceId::new(1);
        reg.set_running(&id, surface);
        assert_eq!(reg.get(&id).map(|a| a.state), Some(AppState::Running));
        assert!(reg.focused().is_some());

        // Running → Stopped
        reg.stop(&id);
        assert_eq!(reg.get(&id).map(|a| a.state), Some(AppState::Stopped));
        assert!(reg.focused().is_none());
    }

    #[test]
    fn cannot_launch_running_app() {
        let mut reg = AppRegistry::new();
        let id = reg.register("app", "App");
        assert!(reg.launch(&id));

        let surface = SurfaceId::new(1);
        reg.set_running(&id, surface);

        // Cannot launch an already-running app
        assert!(!reg.launch(&id));
    }

    #[test]
    fn running_count() {
        let mut reg = AppRegistry::new();
        let id1 = reg.register("app1", "App 1");
        let id2 = reg.register("app2", "App 2");

        assert_eq!(reg.running_count(), 0);

        reg.launch(&id1);
        reg.set_running(&id1, SurfaceId::new(1));
        assert_eq!(reg.running_count(), 1);

        reg.launch(&id2);
        reg.set_running(&id2, SurfaceId::new(2));
        assert_eq!(reg.running_count(), 2);
    }

    #[test]
    fn restart_stopped_app() {
        let mut reg = AppRegistry::new();
        let id = reg.register("app", "App");

        reg.launch(&id);
        reg.set_running(&id, SurfaceId::new(1));
        reg.stop(&id);

        // Can re-launch a stopped app
        assert!(reg.launch(&id));
        assert_eq!(reg.get(&id).map(|a| a.state), Some(AppState::Launching));
    }
}
