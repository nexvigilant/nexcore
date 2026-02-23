// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Shell Prelude
//!
//! Convenience re-exports of the most-used types from `nexcore-shell`.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_shell::prelude::*;
//! ```
//!
//! Brings into scope the main shell, all app lifecycle types, the input
//! processor, notification manager, launcher, status bar, login screen,
//! command palette, AI partner, and the Lex Primitiva grounding infrastructure.

// Core shell
pub use crate::shell::{Shell, ShellState};

// App lifecycle
pub use crate::app::{App, AppId, AppRegistry, AppState};

// Layout
pub use crate::layout::{LayoutRegion, ShellLayout};

// Input
pub use crate::input::{InputAction, InputProcessor, SwipeDirection};

// Launcher
pub use crate::launcher::{AppLauncher, GridConfig, LauncherCell, LauncherView};

// Status bar
pub use crate::status_bar::{IndicatorSlot, StatusBar, StatusBarConfig, StatusIndicator};

// Login
pub use crate::login::{AuthMethod, LoginLayout, LoginScreen, LoginState};

// Notifications
pub use crate::notification::{
    Notification, NotificationManager, NotificationPriority, NotificationState,
};

// Command palette
pub use crate::command_palette::{CommandPalette, EntrySource, PaletteEntry, PaletteMode};

// AI partner
pub use crate::ai_partner::{
    AiPartner, CollaborationMode, ContextSnapshot, Intent, NavigationTarget, SearchScope,
    Suggestion,
};

// Grounding
pub use crate::primitives::{GroundsTo, LexPrimitiva, PrimitiveComposition, Tier};
