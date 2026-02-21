#![allow(dead_code)]
//! Touch and rotary crown input handling.
//!
//! ## Primitive Grounding
//! - ∂ (Boundary): touch regions, gesture thresholds
//! - ν (Frequency): rotary crown tick events
//! - ς (State): gesture state machine (Idle → Touching → Swiped)
//! - κ (Comparison): direction detection (left/right/up/down)
//!
//! ## Tier: T2-C (∂ + ν + ς + κ)
//!
//! ## Grammar: Type-3 (regular)
//! Gesture FSM: Idle → Touching → (Tap | Swipe | LongPress) → Idle

use serde::{Deserialize, Serialize};

/// Input event from watch hardware.
///
/// ## Primitive: Σ (Sum) — alternation of input types
/// ## Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    /// Single tap at (x, y) — λ (Location)
    Tap { x: f32, y: f32 },
    /// Swipe gesture — κ (Comparison) of start/end
    Swipe { direction: SwipeDirection },
    /// Rotary crown rotation — ν (Frequency)
    RotaryCrown { delta: f32 },
    /// Long press at (x, y) — ν (Frequency) + λ (Location)
    LongPress { x: f32, y: f32 },
}

/// Swipe direction.
///
/// ## Primitive: κ (Comparison) — directional comparison
/// ## Tier: T1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Input action that the UI should respond to.
///
/// ## Primitive: → (Causality) — input → action
/// ## Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputAction {
    /// Navigate to next screen
    NextScreen,
    /// Navigate to previous screen
    PrevScreen,
    /// Select/confirm current item
    Select,
    /// Scroll content
    Scroll { direction: SwipeDirection },
    /// Dismiss alert
    DismissAlert,
    /// No action
    None,
}

/// Map raw input event to semantic action.
///
/// ## Primitive: μ (Mapping) — raw → semantic
/// ## Tier: T2-P
pub fn map_input(event: InputEvent) -> InputAction {
    match event {
        InputEvent::Tap { .. } => InputAction::Select,
        InputEvent::Swipe {
            direction: SwipeDirection::Left,
        } => InputAction::NextScreen,
        InputEvent::Swipe {
            direction: SwipeDirection::Right,
        } => InputAction::PrevScreen,
        InputEvent::Swipe { direction } => InputAction::Scroll { direction },
        InputEvent::RotaryCrown { delta } => {
            if delta > 0.0 {
                InputAction::Scroll {
                    direction: SwipeDirection::Down,
                }
            } else if delta < 0.0 {
                InputAction::Scroll {
                    direction: SwipeDirection::Up,
                }
            } else {
                InputAction::None
            }
        }
        InputEvent::LongPress { .. } => InputAction::DismissAlert,
    }
}
