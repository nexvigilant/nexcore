// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexCore Shell — Device Launcher & Home Screen
//!
//! The user-facing shell for NexCore OS across three form factors.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │              Shell                            │
//! │  Boot │ Lock/Unlock │ Sleep/Wake │ App Launch │
//! ├──────────────────────────────────────────────┤
//! │              Layout Engine                    │
//! │  Watch: 2 regions │ Phone: 3 │ Desktop: 2    │
//! ├──────────────────────────────────────────────┤
//! │              App Registry                     │
//! │  Install │ Launch │ Focus │ Stop              │
//! ├──────────────────────────────────────────────┤
//! │              Compositor                       │
//! │  Surface creation │ Compositing │ Display     │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! ## Form Factor Layouts
//!
//! | Device | Regions | Navigation |
//! |--------|---------|------------|
//! | Watch | status_bar + content | Touch/crown |
//! | Phone | status_bar + content + nav_bar | Gesture |
//! | Desktop | content + taskbar | Mouse/keyboard |
//!
//! ## Primitive Grounding
//!
//! | Component | Primitives | Role |
//! |-----------|------------|------|
//! | Layout | ∂ + λ | Bounded regions at positions |
//! | App | ∃ + ς | Existing entity with lifecycle |
//! | Shell | μ + σ | Input mapping + boot sequence |
//! | Lock | ∂ + ς | Security boundary + state |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod ai_partner;
pub mod app;
pub mod command_palette;
pub mod composites;
pub mod grounding;
pub mod input;
pub mod launcher;
pub mod layout;
pub mod login;
pub mod notification;
pub mod prelude;
pub mod primitives;
pub mod shell;
pub mod status_bar;
pub mod transfer;

// Re-export main types
pub use ai_partner::{
    ActionResult, AiPartner, CollaborationMode, ContextSnapshot, ConversationRole,
    ConversationTurn, InputMethod, Intent, NavigationTarget, SearchScope, Suggestion,
};
pub use app::{App, AppId, AppRegistry, AppState};
pub use command_palette::{CommandPalette, EntrySource, PaletteEntry, PaletteMode};
pub use input::{InputAction, InputProcessor, SwipeDirection};
pub use launcher::{AppLauncher, GridConfig, LauncherCell, LauncherView};
pub use layout::{LayoutRegion, ShellLayout};
pub use login::{AuthMethod, LoginLayout, LoginScreen, LoginState};
pub use notification::{
    Notification, NotificationManager, NotificationPriority, NotificationState,
};
pub use shell::{Shell, ShellState};
pub use status_bar::{IndicatorSlot, StatusBar, StatusBarConfig, StatusIndicator};
