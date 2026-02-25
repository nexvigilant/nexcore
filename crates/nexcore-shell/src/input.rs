// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Input handling — routes input events to the correct destination.
//!
//! ## Primitive Grounding
//!
//! - μ Mapping: Input event → action mapping
//! - σ Sequence: Event queue processing order
//! - ∂ Boundary: Form-factor-specific input zones
//! - κ Comparison: Gesture recognition (swipe distance thresholds)

use nexcore_pal::{
    CrownEvent, FormFactor, InputEvent, KeyCode, KeyEvent, KeyState, Modifiers, PointerEvent,
    TouchEvent, TouchPhase,
};

/// Action produced by input processing.
///
/// Tier: T2-C (μ Mapping + ∂ Boundary — mapped from input within device constraints)
#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    /// Route event to the focused application.
    RouteToApp {
        /// App ID receiving the event.
        app_id: String,
        /// The input event to deliver.
        event: InputEvent,
    },
    /// Launch the app switcher / recents.
    ShowAppSwitcher,
    /// Go to home screen.
    GoHome,
    /// Lock the device.
    LockDevice,
    /// Adjust volume (delta: positive = up, negative = down).
    Volume(f32),
    /// Open notification shade.
    ShowNotifications,
    /// Switch focus to next app (Alt+Tab).
    FocusNext,
    /// Switch focus to previous app (Alt+Shift+Tab).
    FocusPrev,
    /// No action (event consumed or ignored).
    None,
}

/// Swipe direction for gesture recognition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Touch tracking for gesture recognition.
///
/// Tracks the start position of each touch to detect gestures.
#[derive(Debug, Clone)]
struct TouchTracker {
    /// Starting position of the touch.
    start_x: f32,
    start_y: f32,
    /// Current position.
    current_x: f32,
    current_y: f32,
    /// Touch ID.
    id: u32,
}

impl TouchTracker {
    fn new(event: &TouchEvent) -> Self {
        Self {
            start_x: event.x,
            start_y: event.y,
            current_x: event.x,
            current_y: event.y,
            id: event.id,
        }
    }

    fn update(&mut self, event: &TouchEvent) {
        self.current_x = event.x;
        self.current_y = event.y;
    }

    /// Detect swipe direction (returns None if below threshold).
    fn detect_swipe(&self, threshold: f32) -> Option<SwipeDirection> {
        let dx = self.current_x - self.start_x;
        let dy = self.current_y - self.start_y;

        if dx.abs() > dy.abs() && dx.abs() > threshold {
            if dx > 0.0 {
                Some(SwipeDirection::Right)
            } else {
                Some(SwipeDirection::Left)
            }
        } else if dy.abs() > dx.abs() && dy.abs() > threshold {
            if dy > 0.0 {
                Some(SwipeDirection::Down)
            } else {
                Some(SwipeDirection::Up)
            }
        } else {
            Option::None
        }
    }

    /// Whether the touch started in the top edge zone.
    fn started_in_top_zone(&self, zone_height: f32) -> bool {
        self.start_y < zone_height
    }

    /// Whether the touch started in the bottom edge zone.
    fn started_in_bottom_zone(&self, screen_height: f32, zone_height: f32) -> bool {
        self.start_y > screen_height - zone_height
    }
}

/// Input processor — maps raw input events to shell actions.
///
/// Tier: T3 (μ + σ + ∂ + κ — full input processing pipeline)
pub struct InputProcessor {
    /// Device form factor (determines gesture rules).
    form_factor: FormFactor,
    /// Screen height for edge detection.
    screen_height: f32,
    /// Active touch trackers.
    active_touches: Vec<TouchTracker>,
    /// Swipe distance threshold (pixels).
    swipe_threshold: f32,
    /// Edge zone height (pixels from edge).
    edge_zone: f32,
}

impl InputProcessor {
    /// Create a new input processor for the given form factor and screen size.
    #[allow(clippy::cast_precision_loss)]
    pub fn new(form_factor: FormFactor, _screen_width: u32, screen_height: u32) -> Self {
        let (swipe_threshold, edge_zone) = match form_factor {
            FormFactor::Watch => (30.0, 40.0), // Small screen — short gestures
            FormFactor::Phone => (80.0, 60.0), // Standard mobile gestures
            FormFactor::Desktop => (100.0, 48.0), // Desktop — larger thresholds
            _ => (100.0, 48.0),                // Unknown form factor: conservative desktop defaults
        };

        Self {
            form_factor,
            screen_height: screen_height as f32,
            active_touches: Vec::new(),
            swipe_threshold,
            edge_zone,
        }
    }

    /// Process an input event and return the resulting action.
    ///
    /// The `focused_app` parameter is the ID of the currently focused app (if any).
    pub fn process(&mut self, event: &InputEvent, focused_app: Option<&str>) -> InputAction {
        match event {
            InputEvent::Touch(touch) => self.process_touch(touch, focused_app),
            InputEvent::Key(key) => Self::process_key(key, focused_app),
            InputEvent::Pointer(ptr) => Self::process_pointer(ptr, focused_app),
            InputEvent::Crown(crown) => self.process_crown(*crown, focused_app),
            _ => InputAction::None,
        }
    }

    /// Process a touch event.
    fn process_touch(&mut self, touch: &TouchEvent, focused_app: Option<&str>) -> InputAction {
        match touch.phase {
            TouchPhase::Started => {
                self.active_touches.push(TouchTracker::new(touch));
                // Touch start — don't produce action yet (wait for gesture or release)
                InputAction::None
            }
            TouchPhase::Moved => {
                // Update tracker
                if let Some(tracker) = self.active_touches.iter_mut().find(|t| t.id == touch.id) {
                    tracker.update(touch);
                }
                // Route move events to app for scrolling etc.
                Self::route_to_app(focused_app, InputEvent::Touch(*touch))
            }
            TouchPhase::Ended => {
                let action = self.resolve_touch_gesture(touch, focused_app);
                self.active_touches.retain(|t| t.id != touch.id);
                action
            }
            TouchPhase::Cancelled => {
                self.active_touches.retain(|t| t.id != touch.id);
                InputAction::None
            }
            _ => InputAction::None,
        }
    }

    /// Resolve a completed touch gesture.
    fn resolve_touch_gesture(&self, touch: &TouchEvent, focused_app: Option<&str>) -> InputAction {
        let tracker = self.active_touches.iter().find(|t| t.id == touch.id);

        let Some(tracker) = tracker else {
            // No tracker — just route the touch end to the app
            return Self::route_to_app(focused_app, InputEvent::Touch(*touch));
        };

        // Check for system gestures based on form factor
        match self.form_factor {
            FormFactor::Phone => {
                // Swipe down from top → notifications
                if tracker.started_in_top_zone(self.edge_zone)
                    && tracker.detect_swipe(self.swipe_threshold) == Some(SwipeDirection::Down)
                {
                    return InputAction::ShowNotifications;
                }
                // Swipe up from bottom → home / app switcher
                if tracker.started_in_bottom_zone(self.screen_height, self.edge_zone)
                    && tracker.detect_swipe(self.swipe_threshold) == Some(SwipeDirection::Up)
                {
                    return InputAction::GoHome;
                }
            }
            FormFactor::Watch => {
                // Swipe right → back / home on watch
                if tracker.detect_swipe(self.swipe_threshold) == Some(SwipeDirection::Right) {
                    return InputAction::GoHome;
                }
                // Swipe down from top → notifications
                if tracker.started_in_top_zone(self.edge_zone)
                    && tracker.detect_swipe(self.swipe_threshold) == Some(SwipeDirection::Down)
                {
                    return InputAction::ShowNotifications;
                }
            }
            FormFactor::Desktop => {
                // Desktop touch — route to app (desktop uses pointer primarily)
            }
            _ => {}
        }

        // No system gesture — route to app
        Self::route_to_app(focused_app, InputEvent::Touch(*touch))
    }

    /// Process a keyboard event.
    fn process_key(key: &KeyEvent, focused_app: Option<&str>) -> InputAction {
        // Only act on key press/repeat, not release
        if key.state == KeyState::Released {
            return Self::route_to_app(focused_app, InputEvent::Key(*key));
        }

        // System shortcuts (all form factors that have keyboards)
        match key.code {
            // Super key → home
            KeyCode::LeftSuper | KeyCode::RightSuper
                if !key.modifiers.contains(Modifiers::ALT)
                    && !key.modifiers.contains(Modifiers::CTRL) =>
            {
                InputAction::GoHome
            }

            // Alt+Tab → focus next
            KeyCode::Tab if key.modifiers.contains(Modifiers::ALT) => {
                if key.modifiers.contains(Modifiers::SHIFT) {
                    InputAction::FocusPrev
                } else {
                    InputAction::FocusNext
                }
            }

            // Super+L → lock
            KeyCode::L if key.modifiers.contains(Modifiers::SUPER) => InputAction::LockDevice,

            // Everything else → route to app
            _ => Self::route_to_app(focused_app, InputEvent::Key(*key)),
        }
    }

    /// Process a pointer (mouse/trackpad) event.
    fn process_pointer(ptr: &PointerEvent, focused_app: Option<&str>) -> InputAction {
        // Pointer events always route to the app (hit testing happens at compositor level)
        Self::route_to_app(focused_app, InputEvent::Pointer(*ptr))
    }

    /// Process a crown/bezel event (watch only).
    fn process_crown(&self, crown: CrownEvent, focused_app: Option<&str>) -> InputAction {
        // Crown press → home on watch
        if crown.pressed && self.form_factor == FormFactor::Watch {
            return InputAction::GoHome;
        }

        // Crown rotation → route to app (scrolling)
        Self::route_to_app(focused_app, InputEvent::Crown(crown))
    }

    /// Route an event to the focused app (if any).
    fn route_to_app(focused_app: Option<&str>, event: InputEvent) -> InputAction {
        focused_app.map_or(InputAction::None, |app_id| InputAction::RouteToApp {
            app_id: app_id.to_string(),
            event,
        })
    }

    /// Get the form factor.
    pub fn form_factor(&self) -> FormFactor {
        self.form_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_touch(id: u32, x: f32, y: f32, phase: TouchPhase) -> InputEvent {
        InputEvent::Touch(TouchEvent::new(id, x, y, 1.0, phase))
    }

    fn make_key(code: KeyCode, state: KeyState, modifiers: Modifiers) -> InputEvent {
        InputEvent::Key(KeyEvent::new(code, state, modifiers))
    }

    fn make_crown(delta: f32, pressed: bool) -> InputEvent {
        InputEvent::Crown(CrownEvent::new(delta, pressed))
    }

    #[test]
    fn touch_routes_to_focused_app() {
        let mut proc = InputProcessor::new(FormFactor::Phone, 1080, 2400);
        let focused = Some("launcher");

        // Touch start → None (waiting for gesture)
        let action = proc.process(&make_touch(0, 500.0, 500.0, TouchPhase::Started), focused);
        assert_eq!(action, InputAction::None);

        // Touch end (short tap) → route to app
        let action = proc.process(&make_touch(0, 500.0, 500.0, TouchPhase::Ended), focused);
        assert!(matches!(action, InputAction::RouteToApp { .. }));
    }

    #[test]
    fn phone_swipe_down_from_top_shows_notifications() {
        let mut proc = InputProcessor::new(FormFactor::Phone, 1080, 2400);

        // Swipe down from top edge
        proc.process(&make_touch(0, 500.0, 10.0, TouchPhase::Started), None);
        proc.process(&make_touch(0, 500.0, 200.0, TouchPhase::Moved), None);
        let action = proc.process(&make_touch(0, 500.0, 200.0, TouchPhase::Ended), None);
        assert_eq!(action, InputAction::ShowNotifications);
    }

    #[test]
    fn phone_swipe_up_from_bottom_goes_home() {
        let mut proc = InputProcessor::new(FormFactor::Phone, 1080, 2400);

        // Swipe up from bottom edge
        proc.process(
            &make_touch(0, 500.0, 2380.0, TouchPhase::Started),
            Some("app"),
        );
        proc.process(
            &make_touch(0, 500.0, 2200.0, TouchPhase::Moved),
            Some("app"),
        );
        let action = proc.process(
            &make_touch(0, 500.0, 2200.0, TouchPhase::Ended),
            Some("app"),
        );
        assert_eq!(action, InputAction::GoHome);
    }

    #[test]
    fn watch_swipe_right_goes_home() {
        let mut proc = InputProcessor::new(FormFactor::Watch, 450, 450);

        proc.process(
            &make_touch(0, 50.0, 200.0, TouchPhase::Started),
            Some("app"),
        );
        proc.process(&make_touch(0, 200.0, 200.0, TouchPhase::Moved), Some("app"));
        let action = proc.process(&make_touch(0, 200.0, 200.0, TouchPhase::Ended), Some("app"));
        assert_eq!(action, InputAction::GoHome);
    }

    #[test]
    fn watch_crown_press_goes_home() {
        let mut proc = InputProcessor::new(FormFactor::Watch, 450, 450);
        let action = proc.process(&make_crown(0.0, true), Some("app"));
        assert_eq!(action, InputAction::GoHome);
    }

    #[test]
    fn watch_crown_rotation_routes_to_app() {
        let mut proc = InputProcessor::new(FormFactor::Watch, 450, 450);
        let action = proc.process(&make_crown(0.5, false), Some("guardian"));
        assert!(matches!(action, InputAction::RouteToApp { app_id, .. } if app_id == "guardian"));
    }

    #[test]
    fn alt_tab_focuses_next() {
        let mut proc = InputProcessor::new(FormFactor::Desktop, 1920, 1080);
        let action = proc.process(
            &make_key(KeyCode::Tab, KeyState::Pressed, Modifiers::ALT),
            Some("editor"),
        );
        assert_eq!(action, InputAction::FocusNext);
    }

    #[test]
    fn alt_shift_tab_focuses_prev() {
        let mut proc = InputProcessor::new(FormFactor::Desktop, 1920, 1080);
        let action = proc.process(
            &make_key(
                KeyCode::Tab,
                KeyState::Pressed,
                Modifiers::ALT.union(Modifiers::SHIFT),
            ),
            Some("editor"),
        );
        assert_eq!(action, InputAction::FocusPrev);
    }

    #[test]
    fn super_l_locks_device() {
        let mut proc = InputProcessor::new(FormFactor::Desktop, 1920, 1080);
        let action = proc.process(
            &make_key(KeyCode::L, KeyState::Pressed, Modifiers::SUPER),
            Some("app"),
        );
        assert_eq!(action, InputAction::LockDevice);
    }

    #[test]
    fn super_key_goes_home() {
        let mut proc = InputProcessor::new(FormFactor::Desktop, 1920, 1080);
        let action = proc.process(
            &make_key(KeyCode::LeftSuper, KeyState::Pressed, Modifiers::NONE),
            Some("app"),
        );
        assert_eq!(action, InputAction::GoHome);
    }

    #[test]
    fn regular_key_routes_to_app() {
        let mut proc = InputProcessor::new(FormFactor::Desktop, 1920, 1080);
        let action = proc.process(
            &make_key(KeyCode::A, KeyState::Pressed, Modifiers::NONE),
            Some("editor"),
        );
        assert!(matches!(action, InputAction::RouteToApp { app_id, .. } if app_id == "editor"));
    }

    #[test]
    fn no_app_focused_returns_none() {
        let mut proc = InputProcessor::new(FormFactor::Desktop, 1920, 1080);
        let action = proc.process(
            &make_key(KeyCode::A, KeyState::Pressed, Modifiers::NONE),
            None,
        );
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn touch_cancel_clears_tracker() {
        let mut proc = InputProcessor::new(FormFactor::Phone, 1080, 2400);
        proc.process(&make_touch(0, 500.0, 10.0, TouchPhase::Started), None);
        let action = proc.process(&make_touch(0, 500.0, 10.0, TouchPhase::Cancelled), None);
        assert_eq!(action, InputAction::None);
        // After cancel, no gesture should be detected
        assert!(proc.active_touches.is_empty());
    }
}
