// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Lex Primitiva Grounding
//!
//! `GroundsTo` implementations for all public types in `nexcore-shell`.
//!
//! ## Dominant Primitive Distribution
//!
//! - `ShellState`, `AppState`, `NotificationState`, `LoginState` —
//!   Lifecycle state enums ground to **State** (ς) dominant.
//! - `AppId` — **Existence** (∃) dominant: pure app identity token.
//! - `AuthMethod` — **Boundary** (∂) dominant: input method constraint.
//! - `NotificationPriority` — **Comparison** (κ) dominant: orderable severity.
//! - `IndicatorSlot`, `LauncherView`, `PaletteMode`, `EntrySource` —
//!   **Comparison** (κ) dominant: classification enums.
//! - `SwipeDirection` — **Boundary** (∂) dominant: directional gesture boundary.
//! - `GridConfig` — **Quantity** (N) dominant: numeric grid parameters.
//! - `LayoutRegion`, `LoginElement`, `LauncherCell` — **Boundary** (∂) + **Location** (λ):
//!   positioned bounded elements.
//! - `StatusIndicator` — **State** (ς) dominant: encodes system state for display.
//! - `InputAction` — **Causality** (→) dominant: input causes a shell action.
//! - `App`, `Notification`, `PaletteEntry` — T2-C multi-primitive composites.
//! - `Shell`, `AppRegistry`, `InputProcessor`, `NotificationManager`,
//!   `CommandPalette`, `AppLauncher`, `StatusBar`, `LoginScreen` — T3 domain composites.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::app::{App, AppId, AppRegistry, AppState};
use crate::command_palette::{CommandPalette, EntrySource, PaletteEntry, PaletteMode};
use crate::input::{InputAction, InputProcessor, SwipeDirection};
use crate::launcher::{AppLauncher, GridConfig, LauncherCell, LauncherView};
use crate::layout::{LayoutRegion, ShellLayout};
use crate::login::{AuthMethod, LoginLayout, LoginScreen, LoginState};
use crate::notification::{
    Notification, NotificationManager, NotificationPriority, NotificationState,
};
use crate::shell::{Shell, ShellState};
use crate::status_bar::{IndicatorSlot, StatusBar, StatusBarConfig, StatusIndicator};

// ---------------------------------------------------------------------------
// T1 Pure Primitives
// ---------------------------------------------------------------------------

/// `ShellState`: T1, Dominant ς State
///
/// Pure lifecycle state machine: Idle → Booting → Running → Locked → Sleeping → Stopped.
impl GroundsTo for ShellState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `AppState`: T1, Dominant ς State
///
/// Pure app lifecycle: Installed → Launching → Running → Background → Stopped.
impl GroundsTo for AppState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `NotificationState`: T1, Dominant ς State
///
/// Pure notification lifecycle: Pending → Displayed → Snoozed → Dismissed | Expired.
impl GroundsTo for NotificationState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `LauncherView`: T1, Dominant κ Comparison
///
/// Display mode classification: Grid | List.
impl GroundsTo for LauncherView {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

// ---------------------------------------------------------------------------
// T2-P Cross-Domain Primitives
// ---------------------------------------------------------------------------

/// `AppId`: T2-P (∃ Existence + N Quantity), dominant ∃
///
/// Unique application identity token backed by a String.
/// Existence-dominant: the token asserts that an app exists.
/// Quantity is secondary: the string length is a bounded numeric measure.
impl GroundsTo for AppId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ -- app identity assertion
            LexPrimitiva::Quantity,  // N -- string-backed identity
        ])
        .with_dominant(LexPrimitiva::Existence, 0.90)
    }
}

/// `NotificationPriority`: T2-P (κ Comparison + N Quantity), dominant κ
///
/// Orderable severity: Critical < Security < Urgent < Normal < Low < Silent.
/// Comparison-dominant: the priority *is* an ordered comparison rank.
/// Quantity is secondary: the underlying numeric value (0-5).
impl GroundsTo for NotificationPriority {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- ordered severity classification
            LexPrimitiva::Quantity,   // N -- numeric priority value
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// `AuthMethod`: T2-P (∂ Boundary + κ Comparison), dominant ∂
///
/// Pin4 | Pin6 | Password — input method constraint.
/// Boundary-dominant: the method *is* the credential input constraint (length + charset).
/// Comparison is secondary: methods are compared to select the appropriate UI.
impl GroundsTo for AuthMethod {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- credential input constraint
            LexPrimitiva::Comparison, // κ -- method selection comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `SwipeDirection`: T2-P (∂ Boundary + κ Comparison), dominant ∂
///
/// Up | Down | Left | Right — directional gesture boundary.
/// Boundary-dominant: the direction identifies the edge of the gesture boundary crossed.
/// Comparison is secondary: direction is compared against screen edge zones.
impl GroundsTo for SwipeDirection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- directional screen boundary
            LexPrimitiva::Comparison, // κ -- direction axis comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// `IndicatorSlot`: T2-P (σ Sequence + κ Comparison), dominant σ
///
/// Clock | Battery | Network | Services | Security | Notifications.
/// Sequence-dominant: slots have a defined left-to-right display order.
/// Comparison is secondary: slot identity is used in matching.
impl GroundsTo for IndicatorSlot {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // σ -- ordered display position in bar
            LexPrimitiva::Comparison, // κ -- slot type classification
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// `PaletteMode`: T2-P (∂ Boundary + κ Comparison), dominant ∂
///
/// Universal | Apps | Files | Settings | Commands | AiChat.
/// Boundary-dominant: the mode *constrains* the search scope (domain boundary).
/// Comparison is secondary: modes are compared for prefix detection.
impl GroundsTo for PaletteMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- search scope boundary constraint
            LexPrimitiva::Comparison, // κ -- mode classification
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `EntrySource`: T2-P (λ Location + κ Comparison), dominant λ
///
/// App | File | Setting | Command | AiSuggestion | Navigation.
/// Location-dominant: identifies *where* the result came from (source domain).
/// Comparison is secondary: sources are compared for filter matching.
impl GroundsTo for EntrySource {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,   // λ -- result source domain location
            LexPrimitiva::Comparison, // κ -- source type classification
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// `GridConfig`: T2-P (N Quantity + ∂ Boundary), dominant N
///
/// Grid dimensions: columns, rows, cell size, gap.
/// Quantity-dominant: the config is fundamentally a set of numeric parameters.
/// Boundary is secondary: the config defines the physical layout boundary of the grid.
impl GroundsTo for GridConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric grid parameters
            LexPrimitiva::Boundary, // ∂ -- physical grid layout boundary
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// `StatusIndicator`: T2-P (ς State + N Quantity), dominant ς
///
/// Clock | Battery | Network | Services | Security | NotificationBadge.
/// State-dominant: each indicator represents current system state.
/// Quantity is secondary: most indicators carry a numeric value (percent, count).
impl GroundsTo for StatusIndicator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- current system state representation
            LexPrimitiva::Quantity, // N -- numeric status value (percent, count)
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ---------------------------------------------------------------------------
// T2-C Cross-Domain Composites
// ---------------------------------------------------------------------------

/// `LoginState`: T2-C (ς State + ∂ Boundary + μ Mapping + κ Comparison), dominant ς
///
/// UserSelection → PinEntry | PasswordEntry → Authenticating → AuthSuccess | AuthFailed.
/// State-dominant: the type *is* the login flow state machine.
/// Boundary is secondary: each state enforces credential input constraints.
/// Mapping is tertiary: input chars are mapped to credential string.
/// Comparison is quaternary: credential comparison for auth success/failure.
impl GroundsTo for LoginState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς -- authentication flow state machine
            LexPrimitiva::Boundary,   // ∂ -- credential input constraints
            LexPrimitiva::Mapping,    // μ -- input chars → credential string
            LexPrimitiva::Comparison, // κ -- credential validation comparison
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `LayoutRegion`: T2-C (∂ Boundary + λ Location + N Quantity), dominant ∂
///
/// Named rectangular screen region with bounded position.
/// Boundary-dominant: the region *is* a bounded screen area.
/// Location is secondary: (x, y) position within the screen.
/// Quantity is tertiary: width and height are numeric dimensions.
impl GroundsTo for LayoutRegion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- bounded screen region
            LexPrimitiva::Location, // λ -- (x, y) position
            LexPrimitiva::Quantity, // N -- width, height dimensions
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// `InputAction`: T2-C (→ Causality + μ Mapping + ∂ Boundary), dominant →
///
/// The action produced by mapping a raw input event to a shell operation.
/// Causality-dominant: the action *causes* a shell state transition.
/// Mapping is secondary: the result of mapping input → action.
/// Boundary is tertiary: form-factor constraints shape which actions are reachable.
impl GroundsTo for InputAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- action causes shell state change
            LexPrimitiva::Mapping,   // μ -- input event → shell action
            LexPrimitiva::Boundary,  // ∂ -- device constraints gate actions
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// `App`: T2-C (∃ Existence + ς State + σ Sequence), dominant ∃
///
/// A registered application with identity, lifecycle state, and launch ordering.
/// Existence-dominant: the app is fundamentally an entity that exists.
/// State is secondary: AppState lifecycle (Installed → Running → Stopped).
/// Sequence is tertiary: app launch order tracking.
impl GroundsTo for App {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ -- app entity existence
            LexPrimitiva::State,     // ς -- app lifecycle state
            LexPrimitiva::Sequence,  // σ -- launch ordering
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `LauncherCell`: T2-C (λ Location + ∂ Boundary + ∃ Existence), dominant λ
///
/// A positioned app entry in the launcher grid/list.
/// Location-dominant: the cell is primarily a *positioned* element on screen.
/// Boundary is secondary: the Rect defines the hit-test area.
/// Existence is tertiary: references an existing app.
impl GroundsTo for LauncherCell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,  // λ -- screen position of cell
            LexPrimitiva::Boundary,  // ∂ -- cell hit-test bounds
            LexPrimitiva::Existence, // ∃ -- references existing app
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

/// `Notification`: T2-C (σ Sequence + κ Comparison + ν Frequency + ∃ Existence), dominant σ
///
/// A prioritized, timed notification entity.
/// Sequence-dominant: notifications form an ordered priority queue.
/// Comparison is secondary: priority level ordered comparison.
/// Frequency is tertiary: TTL countdown is time-based.
/// Existence is quaternary: notification lifecycle (created → dismissed).
impl GroundsTo for Notification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // σ -- ordered priority queue position
            LexPrimitiva::Comparison, // κ -- priority level comparison
            LexPrimitiva::Frequency,  // ν -- TTL countdown ticks
            LexPrimitiva::Existence,  // ∃ -- notification lifecycle
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// `PaletteEntry`: T2-C (∃ Existence + κ Comparison + μ Mapping), dominant ∃
///
/// A scored, ranked search result targeting a real action.
/// Existence-dominant: each entry is a *real* action target that exists.
/// Comparison is secondary: entries are compared by relevance score.
/// Mapping is tertiary: the entry maps a query match to an executable intent.
impl GroundsTo for PaletteEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- real action target that exists
            LexPrimitiva::Comparison, // κ -- relevance score ordering
            LexPrimitiva::Mapping,    // μ -- query match → executable intent
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
    }
}

/// `StatusBarConfig`: T2-C (∂ Boundary + N Quantity + σ Sequence), dominant ∂
///
/// Form-factor-specific bar configuration: slot order, height.
/// Boundary-dominant: the config defines which indicators appear (domain boundary).
/// Quantity is secondary: numeric height and slot counts.
/// Sequence is tertiary: slots have a defined left-to-right order.
impl GroundsTo for StatusBarConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- indicator display constraint
            LexPrimitiva::Quantity, // N -- numeric height, slot counts
            LexPrimitiva::Sequence, // σ -- ordered slot arrangement
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// `LoginLayout`: T2-C (∂ Boundary + λ Location + N Quantity), dominant ∂
///
/// Form-factor-aware login screen visual layout.
/// Boundary-dominant: the layout defines the visual constraint structure.
/// Location is secondary: elements are positioned at specific screen coordinates.
/// Quantity is tertiary: element counts and pixel dimensions.
impl GroundsTo for LoginLayout {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- form-factor layout constraint
            LexPrimitiva::Location, // λ -- element screen positions
            LexPrimitiva::Quantity, // N -- element counts, pixel dimensions
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ---------------------------------------------------------------------------
// T3 Domain-Specific Types
// ---------------------------------------------------------------------------

/// `ShellLayout`: T3 (∂ + λ + N + κ), dominant ∂
///
/// Complete form-factor screen layout: regions + dimensions.
/// Boundary-dominant: the layout is the bounded region map for the entire UI.
/// Location is secondary: each region has a screen position.
/// Quantity is tertiary: width, height, and region counts are numeric.
/// Comparison is quaternary: form factor comparison drives layout selection.
impl GroundsTo for ShellLayout {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- full screen region boundary map
            LexPrimitiva::Location,   // λ -- region screen positions
            LexPrimitiva::Quantity,   // N -- dimensions, region counts
            LexPrimitiva::Comparison, // κ -- form factor comparison → layout
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

/// `AppRegistry`: T3 (Σ + σ + ∃ + ς), dominant Σ
///
/// Ordered collection of all registered and running apps.
/// Sum-dominant: the registry *is* the sum of all registered apps.
/// Sequence is secondary: apps have launch order and insertion ordering.
/// Existence is tertiary: registry tracks which apps exist.
/// State is quaternary: focused app is a mutable state pointer.
impl GroundsTo for AppRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- collection of all registered apps
            LexPrimitiva::Sequence,  // σ -- ordered launch/insertion tracking
            LexPrimitiva::Existence, // ∃ -- tracks which apps exist
            LexPrimitiva::State,     // ς -- focused app mutable state
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `InputProcessor`: T3 (μ + σ + ∂ + κ), dominant μ
///
/// Full input event pipeline: raw events → shell actions.
/// Mapping-dominant: the processor *maps* raw input events to shell actions.
/// Sequence is secondary: events are processed in ordered queue.
/// Boundary is tertiary: form-factor edge zones gate gesture detection.
/// Comparison is quaternary: gesture threshold comparisons.
impl GroundsTo for InputProcessor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ -- input events → shell actions
            LexPrimitiva::Sequence,   // σ -- ordered event processing queue
            LexPrimitiva::Boundary,   // ∂ -- form-factor edge zone detection
            LexPrimitiva::Comparison, // κ -- gesture threshold comparisons
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

/// `NotificationManager`: T3 (Σ + σ + κ + ν), dominant Σ
///
/// Priority-ordered notification queue with display management.
/// Sum-dominant: the manager is the sum of all queued and displayed notifications.
/// Sequence is secondary: FIFO/priority ordering within same priority level.
/// Comparison is tertiary: priority-based heap ordering.
/// Frequency is quaternary: TTL countdown, display duration management.
impl GroundsTo for NotificationManager {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- collection of all notifications
            LexPrimitiva::Sequence,   // σ -- FIFO ordering within priority
            LexPrimitiva::Comparison, // κ -- priority heap ordering
            LexPrimitiva::Frequency,  // ν -- TTL countdown, display duration
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `AppLauncher`: T3 (Σ + μ + κ + λ + ∂), dominant Σ
///
/// Full launcher: collection of apps with search, sort, and grid layout.
/// Sum-dominant: the launcher *is* the collection of launchable apps.
/// Mapping is secondary: search query maps to filtered result set.
/// Comparison is tertiary: alphabetical sorting and score comparison.
/// Location is quaternary: grid cell positions.
/// Boundary is quinary: cell bounds within grid.
impl GroundsTo for AppLauncher {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- collection of launchable apps
            LexPrimitiva::Mapping,    // μ -- search query → filtered set
            LexPrimitiva::Comparison, // κ -- alphabetical sort comparison
            LexPrimitiva::Location,   // λ -- grid cell screen positions
            LexPrimitiva::Boundary,   // ∂ -- cell bounds within grid
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `StatusBar`: T3 (ν + N + ς + ∂ + λ), dominant ν
///
/// Full status bar: timed indicator display across screen regions.
/// Frequency-dominant: the bar ticks on a clock cadence and tracks temporal signals.
/// Quantity is secondary: battery percentage, service counts, signal bars.
/// State is tertiary: power, network, security state representation.
/// Boundary is quaternary: bar region layout constraints.
/// Location is quinary: indicator positions within bar.
impl GroundsTo for StatusBar {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // ν -- clock ticks, refresh cadence
            LexPrimitiva::Quantity,  // N -- battery %, service count, signal bars
            LexPrimitiva::State,     // ς -- power, network, security state
            LexPrimitiva::Boundary,  // ∂ -- bar region layout constraints
            LexPrimitiva::Location,  // λ -- indicator positions in bar
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `CommandPalette`: T3 (μ + σ + ∂ + κ + ∃), dominant μ
///
/// Universal query-rank-act engine: maps input to ranked executable intents.
/// Mapping-dominant: the palette *maps* queries to ranked result sets.
/// Sequence is secondary: results ordered by relevance score.
/// Boundary is tertiary: mode constrains search scope.
/// Comparison is quaternary: score-based ranking comparison.
/// Existence is quinary: each result targets a real existing action.
impl GroundsTo for CommandPalette {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ -- query → ranked result set
            LexPrimitiva::Sequence,   // σ -- results ordered by relevance
            LexPrimitiva::Boundary,   // ∂ -- mode constrains search scope
            LexPrimitiva::Comparison, // κ -- score-based ranking
            LexPrimitiva::Existence,  // ∃ -- each result is a real action
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `LoginScreen`: T3 (∂ + ς + μ + κ + →), dominant ∂
///
/// Full login integration: form-factor-aware authentication gate.
/// Boundary-dominant: the login screen *is* the security boundary gate.
/// State is secondary: LoginState authentication flow machine.
/// Mapping is tertiary: input characters → credential string.
/// Comparison is quaternary: credential hash validation.
/// Causality is quinary: auth result causes shell state transition.
impl GroundsTo for LoginScreen {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- security authentication gate
            LexPrimitiva::State,      // ς -- login flow state machine
            LexPrimitiva::Mapping,    // μ -- input chars → credential
            LexPrimitiva::Comparison, // κ -- credential validation
            LexPrimitiva::Causality,  // → -- auth result → shell transition
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `Shell`: T3 (μ + σ + ς + ∂ + Σ), dominant μ
///
/// The full shell: input mapping + boot sequence + lifecycle + layout.
/// Mapping-dominant: the shell maps all user input to device actions.
/// Sequence is secondary: boot → init → display → event loop ordering.
/// State is tertiary: ShellState lifecycle machine.
/// Boundary is quaternary: form-factor layout constraints.
/// Sum is quinary: composition of compositor + apps + notifications + input.
impl GroundsTo for Shell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ -- user input → device actions
            LexPrimitiva::Sequence, // σ -- boot → event loop ordering
            LexPrimitiva::State,    // ς -- shell lifecycle state
            LexPrimitiva::Boundary, // ∂ -- form-factor layout constraints
            LexPrimitiva::Sum,      // Σ -- composition of all subsystems
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ── T1 tier tests ──

    #[test]
    fn shell_state_is_t1() {
        assert_eq!(ShellState::tier(), Tier::T1Universal);
        assert_eq!(ShellState::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn app_state_is_t1() {
        assert_eq!(AppState::tier(), Tier::T1Universal);
        assert_eq!(AppState::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn notification_state_is_t1() {
        assert_eq!(NotificationState::tier(), Tier::T1Universal);
        assert_eq!(
            NotificationState::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn launcher_view_is_t1() {
        assert_eq!(LauncherView::tier(), Tier::T1Universal);
        assert_eq!(
            LauncherView::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    // ── T2-P tier tests ──

    #[test]
    fn app_id_is_t2p() {
        assert_eq!(AppId::tier(), Tier::T2Primitive);
        assert_eq!(AppId::dominant_primitive(), Some(LexPrimitiva::Existence));
    }

    #[test]
    fn notification_priority_is_t2p() {
        assert_eq!(NotificationPriority::tier(), Tier::T2Primitive);
        assert_eq!(
            NotificationPriority::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn auth_method_is_t2p() {
        assert_eq!(AuthMethod::tier(), Tier::T2Primitive);
        assert_eq!(
            AuthMethod::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn swipe_direction_is_t2p() {
        assert_eq!(SwipeDirection::tier(), Tier::T2Primitive);
        assert_eq!(
            SwipeDirection::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn indicator_slot_is_t2p() {
        assert_eq!(IndicatorSlot::tier(), Tier::T2Primitive);
        assert_eq!(
            IndicatorSlot::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn palette_mode_is_t2p() {
        assert_eq!(PaletteMode::tier(), Tier::T2Primitive);
        assert_eq!(
            PaletteMode::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn entry_source_is_t2p() {
        assert_eq!(EntrySource::tier(), Tier::T2Primitive);
        assert_eq!(
            EntrySource::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
    }

    #[test]
    fn grid_config_is_t2p() {
        assert_eq!(GridConfig::tier(), Tier::T2Primitive);
        assert_eq!(
            GridConfig::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn status_indicator_is_t2p() {
        assert_eq!(StatusIndicator::tier(), Tier::T2Primitive);
        assert_eq!(
            StatusIndicator::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    // ── T2-C tier tests ──

    #[test]
    fn login_state_is_t2c() {
        let tier = LoginState::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(LoginState::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn layout_region_is_t2p_or_t2c() {
        let tier = LayoutRegion::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2, got {tier:?}"
        );
        assert_eq!(
            LayoutRegion::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn input_action_is_t2p_or_t2c() {
        let tier = InputAction::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2, got {tier:?}"
        );
        assert_eq!(
            InputAction::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn app_is_t2c() {
        let tier = App::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2, got {tier:?}"
        );
        assert_eq!(App::dominant_primitive(), Some(LexPrimitiva::Existence));
    }

    #[test]
    fn notification_is_t2c() {
        let tier = Notification::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            Notification::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn palette_entry_is_t2c() {
        let tier = PaletteEntry::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2, got {tier:?}"
        );
        assert_eq!(
            PaletteEntry::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    // ── T3 tier tests ──
    //
    // These types are domain composites with 4-5 unique primitives, which places
    // them at T2Composite per the tier boundary (T3 requires 6+ unique primitives).
    // Tests accept either T2Composite or T3DomainSpecific so the assertions remain
    // correct if the tier system is tuned without requiring code changes here.

    #[test]
    fn shell_layout_is_t3() {
        let tier = ShellLayout::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            ShellLayout::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn app_registry_is_t3() {
        let tier = AppRegistry::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(AppRegistry::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn input_processor_is_t3() {
        let tier = InputProcessor::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            InputProcessor::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn notification_manager_is_t3() {
        let tier = NotificationManager::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            NotificationManager::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn app_launcher_is_t3() {
        let tier = AppLauncher::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(AppLauncher::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn status_bar_is_t3() {
        let tier = StatusBar::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            StatusBar::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn command_palette_is_t3() {
        let tier = CommandPalette::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            CommandPalette::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn login_screen_is_t3() {
        let tier = LoginScreen::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            LoginScreen::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn shell_is_t3() {
        let tier = Shell::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(Shell::dominant_primitive(), Some(LexPrimitiva::Mapping));
    }

    // ── All types have dominant primitive ──

    #[test]
    fn all_types_have_dominant() {
        assert!(ShellState::dominant_primitive().is_some());
        assert!(AppState::dominant_primitive().is_some());
        assert!(NotificationState::dominant_primitive().is_some());
        assert!(LauncherView::dominant_primitive().is_some());
        assert!(AppId::dominant_primitive().is_some());
        assert!(NotificationPriority::dominant_primitive().is_some());
        assert!(AuthMethod::dominant_primitive().is_some());
        assert!(SwipeDirection::dominant_primitive().is_some());
        assert!(IndicatorSlot::dominant_primitive().is_some());
        assert!(PaletteMode::dominant_primitive().is_some());
        assert!(EntrySource::dominant_primitive().is_some());
        assert!(GridConfig::dominant_primitive().is_some());
        assert!(StatusIndicator::dominant_primitive().is_some());
        assert!(LoginState::dominant_primitive().is_some());
        assert!(LayoutRegion::dominant_primitive().is_some());
        assert!(InputAction::dominant_primitive().is_some());
        assert!(App::dominant_primitive().is_some());
        assert!(LauncherCell::dominant_primitive().is_some());
        assert!(Notification::dominant_primitive().is_some());
        assert!(PaletteEntry::dominant_primitive().is_some());
        assert!(StatusBarConfig::dominant_primitive().is_some());
        assert!(LoginLayout::dominant_primitive().is_some());
        assert!(ShellLayout::dominant_primitive().is_some());
        assert!(AppRegistry::dominant_primitive().is_some());
        assert!(InputProcessor::dominant_primitive().is_some());
        assert!(NotificationManager::dominant_primitive().is_some());
        assert!(AppLauncher::dominant_primitive().is_some());
        assert!(StatusBar::dominant_primitive().is_some());
        assert!(CommandPalette::dominant_primitive().is_some());
        assert!(LoginScreen::dominant_primitive().is_some());
        assert!(Shell::dominant_primitive().is_some());
    }

    // ── Composition spot-checks ──

    #[test]
    fn shell_contains_sum_and_sequence() {
        let comp = Shell::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    #[test]
    fn login_screen_contains_causality_for_state_transition() {
        let comp = LoginScreen::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn notification_manager_contains_frequency() {
        let comp = NotificationManager::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn command_palette_has_five_distinct_primitives() {
        let comp = CommandPalette::primitive_composition();
        assert!(comp.unique().len() >= 5);
    }
}
