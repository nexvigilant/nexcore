// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Shell — the main launcher and home screen integration.
//!
//! ## Primitive Grounding
//!
//! - μ Mapping: Maps user input → app actions
//! - σ Sequence: Boot → init → display → event loop
//! - ς State: Shell lifecycle
//! - ∂ Boundary: Form-factor layout constraints

use nexcore_compositor::compositor::{Compositor, CompositorState};
use nexcore_compositor::surface::Rect;
use nexcore_pal::{FormFactor, InputEvent, Platform};

use crate::app::{AppId, AppRegistry};
use crate::input::{InputAction, InputProcessor};
use crate::layout::ShellLayout;
use crate::notification::{NotificationManager, NotificationPriority};

/// Shell lifecycle state.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellState {
    /// Not yet initialized.
    Idle,
    /// Booting — initializing compositor and layout.
    Booting,
    /// Running — displaying home screen.
    Running,
    /// Locked — awaiting authentication.
    Locked,
    /// Sleeping — display off, minimal processing.
    Sleeping,
    /// Shut down.
    Stopped,
}

/// The NexCore Shell — device launcher and home screen.
///
/// Tier: T3 (μ + σ + ς + ∂ — full shell integration)
///
/// Integrates the compositor, layout, and app registry into
/// a unified device experience.
pub struct Shell {
    /// Compositor for display output.
    compositor: Compositor,
    /// Screen layout.
    layout: ShellLayout,
    /// App registry.
    apps: AppRegistry,
    /// Input processor.
    input: InputProcessor,
    /// Notification manager.
    notifications: NotificationManager,
    /// Shell state.
    state: ShellState,
    /// Device form factor.
    form_factor: FormFactor,
}

impl Shell {
    /// Create a new shell for the given form factor.
    pub fn new(form_factor: FormFactor) -> Self {
        let layout = ShellLayout::for_form_factor(form_factor);
        let compositor = Compositor::new(form_factor, layout.width, layout.height);
        let input = InputProcessor::new(form_factor, layout.width, layout.height);

        Self {
            compositor,
            layout,
            apps: AppRegistry::new(),
            input,
            notifications: NotificationManager::new(),
            state: ShellState::Idle,
            form_factor,
        }
    }

    /// Create a shell from a platform instance.
    pub fn from_platform<P: Platform>(platform: &P) -> Self {
        Self::new(platform.form_factor())
    }

    /// Boot the shell — initialize compositor and register system apps.
    pub fn boot(&mut self) {
        self.state = ShellState::Booting;
        self.compositor.start();

        // Register built-in system apps
        self.apps.register("launcher", "Launcher");
        self.apps.register("settings", "Settings");

        self.state = ShellState::Running;
    }

    /// Lock the shell (require authentication to unlock).
    pub fn lock(&mut self) {
        if self.state == ShellState::Running {
            self.state = ShellState::Locked;
        }
    }

    /// Unlock the shell.
    pub fn unlock(&mut self) {
        if self.state == ShellState::Locked {
            self.state = ShellState::Running;
        }
    }

    /// Put the shell to sleep (display off).
    pub fn sleep(&mut self) {
        if self.state == ShellState::Running || self.state == ShellState::Locked {
            self.compositor.suspend();
            self.state = ShellState::Sleeping;
        }
    }

    /// Wake from sleep.
    pub fn wake(&mut self) {
        if self.state == ShellState::Sleeping {
            self.compositor.resume();
            self.state = ShellState::Locked; // Wake to lock screen
        }
    }

    /// Shut down the shell.
    pub fn shutdown(&mut self) {
        self.compositor.stop();
        self.state = ShellState::Stopped;
    }

    /// Launch an app by ID.
    pub fn launch_app(&mut self, app_id: &AppId) -> bool {
        if self.state != ShellState::Running {
            return false;
        }

        if !self.apps.launch(app_id) {
            return false;
        }

        // Create a compositor surface for the app
        let content = self.layout.content_bounds();
        let bounds =
            content.unwrap_or_else(|| Rect::full_screen(self.layout.width, self.layout.height));
        let surface_id = self.compositor.create_surface(app_id.as_str(), bounds);
        self.apps.set_running(app_id, surface_id);
        self.compositor.focus_surface(surface_id);
        true
    }

    /// Stop an app by ID.
    pub fn stop_app(&mut self, app_id: &AppId) {
        if let Some(app) = self.apps.get(app_id) {
            if let Some(sid) = app.surface_id {
                self.compositor.destroy_surface(sid);
            }
        }
        self.apps.stop(app_id);
    }

    /// Handle an input event — processes it and executes the resulting action.
    ///
    /// Returns the action taken (for logging/testing).
    pub fn handle_input(&mut self, event: &InputEvent) -> InputAction {
        // Get focused app ID for routing
        let focused = self.apps.focused().map(|a| a.id.as_str().to_string());
        let action = self.input.process(event, focused.as_deref());

        // Execute the action
        match &action {
            InputAction::GoHome => {
                // Stop focused app, return to home
                if let Some(id) = focused {
                    self.stop_app(&AppId::new(id));
                }
            }
            InputAction::LockDevice => {
                self.lock();
            }
            // Notifications, app switching, volume, routing — handled by subsystems
            InputAction::ShowNotifications
            | InputAction::FocusNext
            | InputAction::FocusPrev
            | InputAction::ShowAppSwitcher
            | InputAction::Volume(_)
            | InputAction::RouteToApp { .. }
            | InputAction::None => {}
        }

        action
    }

    /// Post a notification to the shell.
    pub fn notify(
        &mut self,
        source: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
        priority: NotificationPriority,
    ) -> u64 {
        self.notifications.post(source, title, body, priority)
    }

    /// Dismiss the current notification.
    pub fn dismiss_notification(&mut self) {
        self.notifications.dismiss();
    }

    /// Get the notification manager.
    pub fn notifications(&self) -> &NotificationManager {
        &self.notifications
    }

    /// Run one frame of the shell (composite + present).
    pub fn tick(&mut self) {
        if self.state == ShellState::Running {
            self.notifications.tick();
            self.compositor.composite();
        }
    }

    /// Get the shell state.
    pub fn state(&self) -> ShellState {
        self.state
    }

    /// Get the compositor state.
    pub fn compositor_state(&self) -> CompositorState {
        self.compositor.state()
    }

    /// Get the form factor.
    pub fn form_factor(&self) -> FormFactor {
        self.form_factor
    }

    /// Get the shell layout.
    pub fn layout(&self) -> &ShellLayout {
        &self.layout
    }

    /// Get the app registry.
    pub fn apps(&self) -> &AppRegistry {
        &self.apps
    }

    /// Get the composited framebuffer.
    pub fn framebuffer(&self) -> &[u8] {
        self.compositor.framebuffer()
    }

    /// Number of running apps.
    pub fn running_app_count(&self) -> usize {
        self.apps.running_count()
    }

    /// Get the frame count.
    pub fn frame_count(&self) -> u64 {
        self.compositor.frame_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_boot() {
        let mut shell = Shell::new(FormFactor::Watch);
        assert_eq!(shell.state(), ShellState::Idle);

        shell.boot();
        assert_eq!(shell.state(), ShellState::Running);
        assert_eq!(shell.compositor_state(), CompositorState::Running);
        assert!(shell.apps().count() >= 2); // launcher + settings
    }

    #[test]
    fn shell_lifecycle() {
        let mut shell = Shell::new(FormFactor::Phone);
        shell.boot();

        // Running → Locked
        shell.lock();
        assert_eq!(shell.state(), ShellState::Locked);

        // Locked → Running
        shell.unlock();
        assert_eq!(shell.state(), ShellState::Running);

        // Running → Sleeping
        shell.sleep();
        assert_eq!(shell.state(), ShellState::Sleeping);

        // Sleeping → Locked (wake goes to lock screen)
        shell.wake();
        assert_eq!(shell.state(), ShellState::Locked);

        shell.unlock();
        shell.shutdown();
        assert_eq!(shell.state(), ShellState::Stopped);
    }

    #[test]
    fn launch_app() {
        let mut shell = Shell::new(FormFactor::Desktop);
        shell.boot();

        let app_id = AppId::new("launcher");
        assert!(shell.launch_app(&app_id));
        assert_eq!(shell.running_app_count(), 1);
    }

    #[test]
    fn cannot_launch_when_locked() {
        let mut shell = Shell::new(FormFactor::Phone);
        shell.boot();
        shell.lock();

        let app_id = AppId::new("launcher");
        assert!(!shell.launch_app(&app_id));
    }

    #[test]
    fn stop_app() {
        let mut shell = Shell::new(FormFactor::Desktop);
        shell.boot();

        let app_id = AppId::new("launcher");
        shell.launch_app(&app_id);
        assert_eq!(shell.running_app_count(), 1);

        shell.stop_app(&app_id);
        assert_eq!(shell.running_app_count(), 0);
    }

    #[test]
    fn tick_composites() {
        let mut shell = Shell::new(FormFactor::Watch);
        shell.boot();

        assert_eq!(shell.frame_count(), 0);
        shell.tick();
        assert_eq!(shell.frame_count(), 1);
        shell.tick();
        assert_eq!(shell.frame_count(), 2);
    }

    #[test]
    fn watch_layout() {
        let shell = Shell::new(FormFactor::Watch);
        assert_eq!(shell.form_factor(), FormFactor::Watch);
        assert_eq!(shell.layout().width, 450);
    }

    #[test]
    fn from_platform() {
        let platform = nexcore_pal_linux::LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let shell = Shell::from_platform(&platform);
        assert_eq!(shell.form_factor(), FormFactor::Desktop);
    }
}
