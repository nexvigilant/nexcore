// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Lex Primitiva Grounding
//!
//! `GroundsTo` implementations for all public types in `nexcore-compositor`.
//!
//! ## Dominant Primitive Distribution
//!
//! - `CompositorState`, `Visibility` — lifecycle state enums ground to
//!   **State** (ς) dominant.
//! - `CompositorMode` — **Comparison** (κ) dominant: mode is a form-factor
//!   classification.
//! - `SurfaceId` — **Existence** (∃) dominant: pure surface identity.
//! - `Rect`, `SplitDirection`, `ResizeEdge`, `DecorationZone` —
//!   **Boundary** (∂) dominant: spatial boundary descriptors.
//! - `TilingLayout` — **Mapping** (μ) dominant: windows → tile positions.
//! - `RenderCommand` — **Mapping** (μ) dominant: maps surface data to draw ops.
//! - `GlobalAction`, `InputTarget` — **Causality** (→) dominant: input causes
//!   a compositor action.
//! - `Surface`, `Compositor`, `RenderPipeline`, `TilingEngine` — T3 domain
//!   composites.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::compositor::{Compositor, CompositorState};
use crate::decoration::{DecorationRenderer, DecorationTheme};
use crate::input::{DecorationZone, GlobalAction, InputRouter, InputTarget, ResizeEdge};
use crate::mode::CompositorMode;
use crate::render::{FrameStats, RenderCommand, RenderPipeline};
use crate::surface::{Rect, Surface, SurfaceId, Visibility};
use crate::tiling::{SplitDirection, SplitNode, TilingEngine, TilingLayout};

// ---------------------------------------------------------------------------
// T1 Pure Primitives
// ---------------------------------------------------------------------------

/// `CompositorState`: T1, Dominant ς State
///
/// Pure lifecycle: Idle → Running → Suspended | Stopped.
impl GroundsTo for CompositorState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `Visibility`: T1, Dominant ς State
///
/// Pure surface visibility state: Visible | Occluded | Hidden.
impl GroundsTo for Visibility {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ---------------------------------------------------------------------------
// T2-P Cross-Domain Primitives
// ---------------------------------------------------------------------------

/// `SurfaceId`: T2-P (∃ Existence + N Quantity), dominant ∃
///
/// Pure surface identity token backed by a u32.
/// Existence-dominant: the token asserts that a surface exists.
/// Quantity is secondary: raw u32 value.
impl GroundsTo for SurfaceId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ -- surface identity assertion
            LexPrimitiva::Quantity,  // N -- raw u32 id value
        ])
        .with_dominant(LexPrimitiva::Existence, 0.90)
    }
}

/// `CompositorMode`: T2-P (κ Comparison + ∂ Boundary), dominant κ
///
/// SingleApp | AppStack | WindowManager — form-factor classification.
/// Comparison-dominant: mode is selected by comparing form factor.
/// Boundary is secondary: each mode defines surface layout constraints.
impl GroundsTo for CompositorMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- form factor classification
            LexPrimitiva::Boundary,   // ∂ -- layout constraint boundaries
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// `Rect`: T2-P (∂ Boundary + λ Location), dominant ∂
///
/// Screen-space rectangle: (x, y, width, height).
/// Boundary-dominant: the rectangle *defines* the spatial boundary.
/// Location is secondary: (x, y) is a position.
impl GroundsTo for Rect {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- spatial extent boundary
            LexPrimitiva::Location, // λ -- (x, y) screen position
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `SplitDirection`: T2-P (∂ Boundary + κ Comparison), dominant ∂
///
/// Vertical | Horizontal — identifies the split axis.
/// Boundary-dominant: the direction *is* the boundary axis.
/// Comparison is secondary: axes are compared for layout decisions.
impl GroundsTo for SplitDirection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- split axis boundary
            LexPrimitiva::Comparison, // κ -- axis comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// `ResizeEdge`: T2-P (∂ Boundary + λ Location), dominant ∂
///
/// Top | Bottom | Left | Right | corner variants.
/// Boundary-dominant: identifies which window edge is being resized.
/// Location is secondary: the edge has a spatial position.
impl GroundsTo for ResizeEdge {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- window edge being resized
            LexPrimitiva::Location, // λ -- edge spatial position
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// `DecorationZone`: T2-P (λ Location + ∂ Boundary), dominant λ
///
/// TitleBar | CloseButton | MinimizeButton | MaximizeButton | ResizeHandle.
/// Location-dominant: identifies a spatial zone within the decoration chrome.
/// Boundary is secondary: each zone has a bounded hit-test region.
impl GroundsTo for DecorationZone {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- zone within decoration chrome
            LexPrimitiva::Boundary, // ∂ -- hit-test boundary of zone
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// `GlobalAction`: T2-P (→ Causality + κ Comparison), dominant →
///
/// Compositor-level action triggered by input: SwitchApp | OpenLauncher.
/// Causality-dominant: the action causes a compositor state change.
/// Comparison is secondary: actions are matched by type to handler.
impl GroundsTo for GlobalAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- input causes compositor action
            LexPrimitiva::Comparison, // κ -- action type classification
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// `TilingLayout`: T2-P (μ Mapping + σ Sequence + ∂ Boundary), dominant μ
///
/// Monocle | MasterStack | VerticalSplit | HorizontalSplit | Grid | Bsp.
/// Mapping-dominant: the layout *maps* windows to tile positions.
/// Sequence is secondary: insertion order determines position.
/// Boundary is tertiary: tiles have strict non-overlapping bounds.
impl GroundsTo for TilingLayout {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ -- window list → tile positions
            LexPrimitiva::Sequence, // σ -- insertion order determines position
            LexPrimitiva::Boundary, // ∂ -- non-overlapping tile bounds
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// `DecorationTheme`: T2-P (μ Mapping + κ Comparison), dominant μ
///
/// Color scheme mapping surface state to visual appearance.
/// Mapping-dominant: maps active/inactive state to color values.
/// Comparison is secondary: active vs inactive window comparison.
impl GroundsTo for DecorationTheme {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ -- surface state → color values
            LexPrimitiva::Comparison, // κ -- active vs inactive comparison
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// `FrameStats`: T2-P (N Quantity + ν Frequency), dominant N
///
/// Frame rendering statistics: frame count, timing.
/// Quantity-dominant: the stats are numeric counters and durations.
/// Frequency is secondary: frames arrive at a temporal frequency.
impl GroundsTo for FrameStats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- frame counts, timing values
            LexPrimitiva::Frequency, // ν -- frame arrival rate
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T2-C Cross-Domain Composites
// ---------------------------------------------------------------------------

/// `InputTarget`: T2-C (μ Mapping + λ Location + → Causality), dominant μ
///
/// Result of routing an input event: Surface | Decoration | Desktop | Global.
/// Mapping-dominant: the target is the *output* of the input→surface mapping.
/// Location is secondary: hit-testing uses screen coordinates.
/// Causality is tertiary: input causes the target to receive the event.
impl GroundsTo for InputTarget {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // μ -- input event → target surface mapping
            LexPrimitiva::Location,  // λ -- hit-test screen coordinates
            LexPrimitiva::Causality, // → -- input causes surface event receipt
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// `RenderCommand`: T2-C (μ Mapping + σ Sequence + ∂ Boundary), dominant μ
///
/// Clear | BlitSurface | BlitDecoration | Present.
/// Mapping-dominant: each command maps source data to a display operation.
/// Sequence is secondary: commands execute in pipeline order.
/// Boundary is tertiary: Rect bounds constrain the blit region.
impl GroundsTo for RenderCommand {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ -- source data → display operation
            LexPrimitiva::Sequence, // σ -- ordered pipeline execution
            LexPrimitiva::Boundary, // ∂ -- blit region clipping bounds
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// `SplitNode`: T2-C (ρ Recursion + ∂ Boundary + μ Mapping), dominant ρ
///
/// BSP tree node: Leaf(SurfaceId) | Split(direction, ratio, box, box).
/// Recursion-dominant: the BSP tree is recursively defined.
/// Boundary is secondary: each node partitions a screen region.
/// Mapping is tertiary: maps surface to its allocated rect.
impl GroundsTo for SplitNode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ -- recursive BSP tree structure
            LexPrimitiva::Boundary,  // ∂ -- partitioned screen region
            LexPrimitiva::Mapping,   // μ -- surface → allocated rect
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.80)
    }
}

// ---------------------------------------------------------------------------
// T3 Domain-Specific Types
// ---------------------------------------------------------------------------

/// `Surface`: T3 (∂ + λ + ∃ + π + ς), dominant ∂
///
/// Full compositing surface: bounded, located, existent, persistent, stateful.
/// Boundary-dominant: the surface *is* a bounded pixel region.
/// Location is secondary: the surface has a screen position.
/// Existence is tertiary: surface lifecycle (created/destroyed).
/// Persistence is quaternary: framebuffer content survives redraws.
/// State is quinary: Visibility and focus state.
impl GroundsTo for Surface {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // ∂ -- bounded pixel region
            LexPrimitiva::Location,    // λ -- screen position
            LexPrimitiva::Existence,   // ∃ -- surface lifecycle
            LexPrimitiva::Persistence, // π -- framebuffer content persistence
            LexPrimitiva::State,       // ς -- visibility and focus state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `RenderPipeline`: T3 (μ + σ + ∂ + Σ), dominant μ
///
/// Ordered command-based render pipeline.
/// Mapping-dominant: maps surface framebuffers to display output.
/// Sequence is secondary: pipeline stages execute in order.
/// Boundary is tertiary: clipping regions constrain rendering.
/// Sum is quaternary: aggregates multiple surface outputs into one.
impl GroundsTo for RenderPipeline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ -- surface framebuffers → display output
            LexPrimitiva::Sequence, // σ -- ordered pipeline stages
            LexPrimitiva::Boundary, // ∂ -- clipping region constraints
            LexPrimitiva::Sum,      // Σ -- aggregates multiple surface outputs
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

/// `TilingEngine`: T3 (ρ + ∂ + μ + σ + λ), dominant ρ
///
/// BSP-based automatic window tiling.
/// Recursion-dominant: the BSP tree recursively partitions the workspace.
/// Boundary is secondary: tiles have strict non-overlapping screen regions.
/// Mapping is tertiary: assigns windows to tile rects.
/// Sequence is quaternary: insertion order determines initial layout.
/// Location is quinary: each tile has a screen-space position.
impl GroundsTo for TilingEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ -- recursive BSP tree partitioning
            LexPrimitiva::Boundary,  // ∂ -- non-overlapping tile boundaries
            LexPrimitiva::Mapping,   // μ -- window → tile rect assignment
            LexPrimitiva::Sequence,  // σ -- insertion order → initial position
            LexPrimitiva::Location,  // λ -- tile screen-space position
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.75)
    }
}

/// `InputRouter`: T3 (μ + λ + → + ∂ + ∃), dominant μ
///
/// Hit-test-based input event router.
/// Mapping-dominant: routes input events to the correct target surface.
/// Location is secondary: hit-testing uses screen coordinates.
/// Causality is tertiary: routed input causes surface state change.
/// Boundary is quaternary: surface bounds constrain hit regions.
/// Existence is quinary: focus surface may or may not exist.
impl GroundsTo for InputRouter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // μ -- input event → target surface
            LexPrimitiva::Location,  // λ -- hit-test screen coordinates
            LexPrimitiva::Causality, // → -- routing causes surface state change
            LexPrimitiva::Boundary,  // ∂ -- surface bounds constrain hit regions
            LexPrimitiva::Existence, // ∃ -- focused surface may not exist
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

/// `DecorationRenderer`: T3 (∂ + λ + μ + κ), dominant ∂
///
/// Window chrome generator: title bars, borders, buttons.
/// Boundary-dominant: decorations define and visualize window edges.
/// Location is secondary: buttons and title are positioned within chrome.
/// Mapping is tertiary: surface state → decoration appearance.
/// Comparison is quaternary: active vs inactive window styling.
impl GroundsTo for DecorationRenderer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- window chrome edge definition
            LexPrimitiva::Location,   // λ -- button/title positioning
            LexPrimitiva::Mapping,    // μ -- surface state → appearance
            LexPrimitiva::Comparison, // κ -- active vs inactive styling
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

/// `Compositor`: T3 (Σ + μ + σ + ς + ∂), dominant Σ
///
/// The complete display server: surfaces + input + rendering + mode.
/// Sum-dominant: the compositor *is* the sum of all surfaces mapped to display.
/// Mapping is secondary: input events routed to focused surface.
/// Sequence is tertiary: frame rendering pipeline ordering.
/// State is quaternary: CompositorState lifecycle.
/// Boundary is quinary: resolution defines the total framebuffer boundary.
impl GroundsTo for Compositor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- sum of all surfaces on display
            LexPrimitiva::Mapping,  // μ -- input events → focused surface
            LexPrimitiva::Sequence, // σ -- frame render pipeline
            LexPrimitiva::State,    // ς -- compositor lifecycle state
            LexPrimitiva::Boundary, // ∂ -- display resolution boundary
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
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

    // T1 tier tests

    #[test]
    fn compositor_state_is_t1() {
        assert_eq!(CompositorState::tier(), Tier::T1Universal);
        assert_eq!(
            CompositorState::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn visibility_is_t1() {
        assert_eq!(Visibility::tier(), Tier::T1Universal);
        assert_eq!(
            Visibility::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    // T2-P tier tests

    #[test]
    fn surface_id_is_t2p() {
        assert_eq!(SurfaceId::tier(), Tier::T2Primitive);
        assert_eq!(
            SurfaceId::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn compositor_mode_is_t2p() {
        assert_eq!(CompositorMode::tier(), Tier::T2Primitive);
        assert_eq!(
            CompositorMode::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn rect_is_t2p() {
        assert_eq!(Rect::tier(), Tier::T2Primitive);
        assert_eq!(Rect::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn split_direction_is_t2p() {
        assert_eq!(SplitDirection::tier(), Tier::T2Primitive);
        assert_eq!(
            SplitDirection::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn tiling_layout_is_t2p() {
        let tier = TilingLayout::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2-P or T2-C, got {tier:?}"
        );
        assert_eq!(
            TilingLayout::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    // T2-C tier tests

    #[test]
    fn render_command_is_t2p_or_t2c() {
        let tier = RenderCommand::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2, got {tier:?}"
        );
        assert_eq!(
            RenderCommand::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn input_target_is_t2p_or_t2c() {
        let tier = InputTarget::tier();
        assert!(
            tier == Tier::T2Primitive || tier == Tier::T2Composite,
            "expected T2, got {tier:?}"
        );
        assert_eq!(
            InputTarget::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn split_node_uses_recursion() {
        let comp = SplitNode::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
    }

    // T2Composite tier tests

    #[test]
    fn surface_is_t3() {
        assert_eq!(Surface::tier(), Tier::T2Composite);
        assert_eq!(Surface::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn compositor_is_t3() {
        assert_eq!(Compositor::tier(), Tier::T2Composite);
        assert_eq!(Compositor::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn tiling_engine_is_t3() {
        assert_eq!(TilingEngine::tier(), Tier::T2Composite);
        assert_eq!(
            TilingEngine::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
    }

    #[test]
    fn render_pipeline_is_t3() {
        assert_eq!(RenderPipeline::tier(), Tier::T2Composite);
        assert_eq!(
            RenderPipeline::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    // All types have dominant primitive

    #[test]
    fn all_types_have_dominant() {
        assert!(CompositorState::dominant_primitive().is_some());
        assert!(Visibility::dominant_primitive().is_some());
        assert!(SurfaceId::dominant_primitive().is_some());
        assert!(CompositorMode::dominant_primitive().is_some());
        assert!(Rect::dominant_primitive().is_some());
        assert!(SplitDirection::dominant_primitive().is_some());
        assert!(ResizeEdge::dominant_primitive().is_some());
        assert!(DecorationZone::dominant_primitive().is_some());
        assert!(GlobalAction::dominant_primitive().is_some());
        assert!(TilingLayout::dominant_primitive().is_some());
        assert!(InputTarget::dominant_primitive().is_some());
        assert!(RenderCommand::dominant_primitive().is_some());
        assert!(SplitNode::dominant_primitive().is_some());
        assert!(Surface::dominant_primitive().is_some());
        assert!(Compositor::dominant_primitive().is_some());
        assert!(TilingEngine::dominant_primitive().is_some());
        assert!(RenderPipeline::dominant_primitive().is_some());
        assert!(InputRouter::dominant_primitive().is_some());
        assert!(DecorationRenderer::dominant_primitive().is_some());
    }

    // Composition content spot-checks

    #[test]
    fn surface_has_five_distinct_primitives() {
        let comp = Surface::primitive_composition();
        assert!(comp.unique().len() >= 5);
    }

    #[test]
    fn compositor_contains_sum_and_mapping() {
        let comp = Compositor::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
    }

    #[test]
    fn tiling_engine_contains_recursion_and_boundary() {
        let comp = TilingEngine::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }
}
