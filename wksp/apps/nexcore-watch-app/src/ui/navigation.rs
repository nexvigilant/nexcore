#![allow(dead_code)]
//! Screen navigation state machine.
//!
//! ## Primitive Grounding
//! - ς (State): current screen in the navigation stack
//! - σ (Sequence): screen ordering (Guardian → Signal → Alerts)
//! - ∂ (Boundary): wrap-around at edges
//!
//! ## Tier: T2-C (ς + σ + ∂)
//!
//! ## Grammar: Type-3 (regular)
//! Three-state circular automaton: Guardian ↔ Signal ↔ Alerts

use serde::{Deserialize, Serialize};

/// Active screen on the watch.
///
/// ## Primitive: ς (State) — current screen
/// ## Tier: T1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    /// Guardian homeostasis status — ς (State) display
    Guardian,
    /// Signal detection metrics — κ (Comparison) display
    Signal,
    /// P0-P5 alert queue — ∂ (Boundary) display
    Alerts,
}

impl Screen {
    /// Navigate to next screen (wraps around).
    ///
    /// ## Primitive: σ (Sequence) + ∂ (Boundary)
    /// ## Tier: T2-P
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Guardian => Self::Signal,
            Self::Signal => Self::Alerts,
            Self::Alerts => Self::Guardian,
        }
    }

    /// Navigate to previous screen (wraps around).
    ///
    /// ## Primitive: σ (Sequence) + ∂ (Boundary)
    /// ## Tier: T2-P
    #[must_use]
    pub fn prev(self) -> Self {
        match self {
            Self::Guardian => Self::Alerts,
            Self::Signal => Self::Guardian,
            Self::Alerts => Self::Signal,
        }
    }

    /// Screen title for header display.
    ///
    /// ## Primitive: μ (Mapping)
    /// ## Tier: T1
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Guardian => "Guardian",
            Self::Signal => "Signal",
            Self::Alerts => "Alerts",
        }
    }

    /// Screen index for page indicator.
    ///
    /// ## Primitive: μ (Mapping) — enum → ordinal
    /// ## Tier: T1
    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            Self::Guardian => 0,
            Self::Signal => 1,
            Self::Alerts => 2,
        }
    }

    /// Total number of screens.
    pub const COUNT: u8 = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_wraps_around() {
        assert_eq!(Screen::Guardian.next(), Screen::Signal);
        assert_eq!(Screen::Signal.next(), Screen::Alerts);
        assert_eq!(Screen::Alerts.next(), Screen::Guardian);
    }

    #[test]
    fn prev_wraps_around() {
        assert_eq!(Screen::Guardian.prev(), Screen::Alerts);
        assert_eq!(Screen::Signal.prev(), Screen::Guardian);
        assert_eq!(Screen::Alerts.prev(), Screen::Signal);
    }

    #[test]
    fn next_prev_inverse() {
        let screen = Screen::Guardian;
        assert_eq!(screen.next().prev(), screen);
        assert_eq!(screen.prev().next(), screen);
    }

    #[test]
    fn indices_sequential() {
        assert_eq!(Screen::Guardian.index(), 0);
        assert_eq!(Screen::Signal.index(), 1);
        assert_eq!(Screen::Alerts.index(), 2);
    }
}
