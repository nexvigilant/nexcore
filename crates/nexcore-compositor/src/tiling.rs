// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Tiling window manager — automatic window layout via BSP tree.
//!
//! ## Primitive Grounding
//!
//! - σ Sequence: Window insertion order determines tile position
//! - ∂ Boundary: Tiles have strict non-overlapping bounds
//! - μ Mapping: Window list → tile position assignment
//! - λ Location: Screen region allocation per tile
//! - κ Comparison: Split ratios between tiles
//! - Σ Sum: Total tile area = display area (conservation)

use crate::surface::{Rect, SurfaceId};

/// Tiling layout preset.
///
/// Tier: T2-C (σ + μ — ordered mapping strategy)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilingLayout {
    /// Single window fills the entire workspace.
    Monocle,

    /// Master window on the left, stack on the right.
    /// The ratio is the master width fraction (0.0..1.0).
    MasterStack,

    /// Even vertical split (side by side).
    VerticalSplit,

    /// Even horizontal split (top and bottom).
    HorizontalSplit,

    /// Grid layout (N columns, computed rows).
    Grid,

    /// Manual BSP tree (user-controlled splits).
    Bsp,
}

/// Split direction for BSP nodes.
///
/// Tier: T2-P (∂ Boundary — split axis)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// Split into left and right halves.
    Vertical,
    /// Split into top and bottom halves.
    Horizontal,
}

/// A node in the BSP layout tree.
///
/// Tier: T2-C (ρ + ∂ — recursive boundary partitioning)
#[derive(Debug, Clone)]
pub enum SplitNode {
    /// Leaf node — contains a window.
    Leaf(SurfaceId),

    /// Internal node — splits into two children.
    Split {
        /// Split direction.
        direction: SplitDirection,
        /// Split ratio (0.0..1.0) — fraction for the first child.
        ratio: f32,
        /// First child (left or top).
        first: Box<Self>,
        /// Second child (right or bottom).
        second: Box<Self>,
    },
}

impl SplitNode {
    /// Create a leaf node.
    pub fn leaf(id: SurfaceId) -> Self {
        Self::Leaf(id)
    }

    /// Create a split node.
    pub fn split(direction: SplitDirection, ratio: f32, first: Self, second: Self) -> Self {
        Self::Split {
            direction,
            ratio: ratio.clamp(0.1, 0.9),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Count the number of windows in this subtree.
    pub fn window_count(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Split { first, second, .. } => first.window_count() + second.window_count(),
        }
    }

    /// Collect all surface IDs in tree order (left-to-right / top-to-bottom).
    pub fn surface_ids(&self) -> Vec<SurfaceId> {
        let mut ids = Vec::new();
        self.collect_ids(&mut ids);
        ids
    }

    fn collect_ids(&self, out: &mut Vec<SurfaceId>) {
        match self {
            Self::Leaf(id) => out.push(*id),
            Self::Split { first, second, .. } => {
                first.collect_ids(out);
                second.collect_ids(out);
            }
        }
    }

    /// Check if a surface exists in this tree.
    pub fn contains(&self, target: SurfaceId) -> bool {
        match self {
            Self::Leaf(id) => *id == target,
            Self::Split { first, second, .. } => first.contains(target) || second.contains(target),
        }
    }

    /// Remove a surface from the tree. Returns the simplified tree, or None if empty.
    pub fn remove(self, target: SurfaceId) -> Option<Self> {
        match self {
            Self::Leaf(id) if id == target => None,
            Self::Leaf(_) => Some(self),
            Self::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let first_removed = first.remove(target);
                let second_removed = second.remove(target);
                match (first_removed, second_removed) {
                    (None, None) => None,
                    (Some(node), None) | (None, Some(node)) => Some(node),
                    (Some(f), Some(s)) => Some(Self::Split {
                        direction,
                        ratio,
                        first: Box::new(f),
                        second: Box::new(s),
                    }),
                }
            }
        }
    }

    /// Compute the rectangle for each window given a bounding area.
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub fn compute_rects(&self, area: &Rect, gap: u32) -> Vec<(SurfaceId, Rect)> {
        let mut result = Vec::new();
        self.compute_rects_inner(area, gap, &mut result);
        result
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn compute_rects_inner(&self, area: &Rect, gap: u32, out: &mut Vec<(SurfaceId, Rect)>) {
        match self {
            Self::Leaf(id) => {
                // Apply inner gap (shrink the tile area)
                let tile = Rect::new(
                    area.x + gap as i32,
                    area.y + gap as i32,
                    area.width.saturating_sub(gap * 2),
                    area.height.saturating_sub(gap * 2),
                );
                out.push((*id, tile));
            }
            Self::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let (first_area, second_area) = split_rect(area, *direction, *ratio);
                first.compute_rects_inner(&first_area, gap, out);
                second.compute_rects_inner(&second_area, gap, out);
            }
        }
    }
}

/// Split a rectangle into two along the given direction.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn split_rect(area: &Rect, direction: SplitDirection, ratio: f32) -> (Rect, Rect) {
    match direction {
        SplitDirection::Vertical => {
            let first_w = (area.width as f32 * ratio) as u32;
            let second_w = area.width.saturating_sub(first_w);
            (
                Rect::new(area.x, area.y, first_w, area.height),
                Rect::new(area.x + first_w as i32, area.y, second_w, area.height),
            )
        }
        SplitDirection::Horizontal => {
            let first_h = (area.height as f32 * ratio) as u32;
            let second_h = area.height.saturating_sub(first_h);
            (
                Rect::new(area.x, area.y, area.width, first_h),
                Rect::new(area.x, area.y + first_h as i32, area.width, second_h),
            )
        }
    }
}

/// Tiling engine — manages window layout for desktop mode.
///
/// Tier: T3 (σ + ∂ + μ + λ + κ + Σ — full tiling pipeline)
///
/// Computes window positions based on the active layout and
/// the list of tiled windows. Non-tiled (floating) windows
/// are excluded from tiling calculations.
pub struct TilingEngine {
    /// Active layout preset.
    layout: TilingLayout,
    /// BSP tree (used when layout is Bsp, or as computed tree for presets).
    tree: Option<SplitNode>,
    /// Ordered list of tiled surface IDs.
    tiled_surfaces: Vec<SurfaceId>,
    /// Master/stack ratio for MasterStack layout.
    master_ratio: f32,
    /// Gap between tiles in pixels.
    gap: u32,
    /// Workspace area (display area minus status bar, etc.).
    workspace: Rect,
}

impl TilingEngine {
    /// Create a new tiling engine.
    pub fn new(workspace: Rect) -> Self {
        Self {
            layout: TilingLayout::MasterStack,
            tree: None,
            tiled_surfaces: Vec::new(),
            master_ratio: 0.55,
            gap: 4,
            workspace,
        }
    }

    /// Set the active layout.
    pub fn set_layout(&mut self, layout: TilingLayout) {
        self.layout = layout;
        self.rebuild_tree();
    }

    /// Get the active layout.
    pub fn layout(&self) -> TilingLayout {
        self.layout
    }

    /// Cycle to the next layout.
    pub fn cycle_layout(&mut self) {
        self.layout = match self.layout {
            TilingLayout::MasterStack => TilingLayout::VerticalSplit,
            TilingLayout::VerticalSplit => TilingLayout::HorizontalSplit,
            TilingLayout::HorizontalSplit => TilingLayout::Grid,
            TilingLayout::Grid => TilingLayout::Monocle,
            TilingLayout::Monocle => TilingLayout::Bsp,
            TilingLayout::Bsp => TilingLayout::MasterStack,
        };
        self.rebuild_tree();
    }

    /// Set the gap between tiles.
    pub fn set_gap(&mut self, gap: u32) {
        self.gap = gap;
        // No rebuild needed — gap is applied during compute_rects
    }

    /// Get the current gap.
    pub fn gap(&self) -> u32 {
        self.gap
    }

    /// Set the master ratio (for MasterStack layout).
    pub fn set_master_ratio(&mut self, ratio: f32) {
        self.master_ratio = ratio.clamp(0.1, 0.9);
        self.rebuild_tree();
    }

    /// Get the master ratio.
    pub fn master_ratio(&self) -> f32 {
        self.master_ratio
    }

    /// Update the workspace area.
    pub fn set_workspace(&mut self, workspace: Rect) {
        self.workspace = workspace;
        // No rebuild needed — workspace is applied during compute_rects
    }

    /// Add a surface to the tiling layout.
    pub fn add_surface(&mut self, id: SurfaceId) {
        if !self.tiled_surfaces.contains(&id) {
            self.tiled_surfaces.push(id);
            self.rebuild_tree();
        }
    }

    /// Remove a surface from the tiling layout.
    pub fn remove_surface(&mut self, id: SurfaceId) {
        self.tiled_surfaces.retain(|s| *s != id);
        self.rebuild_tree();
    }

    /// Swap two surfaces in the tiling order.
    pub fn swap_surfaces(&mut self, a: SurfaceId, b: SurfaceId) {
        let pos_a = self.tiled_surfaces.iter().position(|s| *s == a);
        let pos_b = self.tiled_surfaces.iter().position(|s| *s == b);
        if let (Some(ia), Some(ib)) = (pos_a, pos_b) {
            self.tiled_surfaces.swap(ia, ib);
            self.rebuild_tree();
        }
    }

    /// Promote a surface to the master position (index 0).
    pub fn promote_to_master(&mut self, id: SurfaceId) {
        if let Some(pos) = self.tiled_surfaces.iter().position(|s| *s == id) {
            if pos > 0 {
                let surface = self.tiled_surfaces.remove(pos);
                self.tiled_surfaces.insert(0, surface);
                self.rebuild_tree();
            }
        }
    }

    /// Number of tiled surfaces.
    pub fn tiled_count(&self) -> usize {
        self.tiled_surfaces.len()
    }

    /// Compute tile rectangles for all tiled surfaces.
    ///
    /// Returns a list of (SurfaceId, Rect) pairs with the computed
    /// bounds for each tiled surface.
    pub fn compute_layout(&self) -> Vec<(SurfaceId, Rect)> {
        if self.tiled_surfaces.is_empty() {
            return Vec::new();
        }

        self.tree.as_ref().map_or_else(Vec::new, |tree| {
            tree.compute_rects(&self.workspace, self.gap)
        })
    }

    /// Rebuild the BSP tree from the current layout and surface list.
    fn rebuild_tree(&mut self) {
        if self.tiled_surfaces.is_empty() {
            self.tree = None;
            return;
        }

        self.tree = Some(match self.layout {
            TilingLayout::Monocle => self.build_monocle(),
            TilingLayout::MasterStack => self.build_master_stack(),
            TilingLayout::VerticalSplit => self.build_even_split(SplitDirection::Vertical),
            TilingLayout::HorizontalSplit => self.build_even_split(SplitDirection::Horizontal),
            TilingLayout::Grid => self.build_grid(),
            TilingLayout::Bsp => self.build_bsp(),
        });
    }

    /// Monocle: only the first (focused) window is visible.
    fn build_monocle(&self) -> SplitNode {
        // In monocle, we only tile the first surface full-screen
        SplitNode::leaf(self.tiled_surfaces[0])
    }

    /// Master-stack: master on left, rest stacked vertically on right.
    fn build_master_stack(&self) -> SplitNode {
        let surfaces = &self.tiled_surfaces;
        if surfaces.len() == 1 {
            return SplitNode::leaf(surfaces[0]);
        }

        let master = SplitNode::leaf(surfaces[0]);
        let stack = build_vertical_stack_recursive(&surfaces[1..]);

        SplitNode::split(SplitDirection::Vertical, self.master_ratio, master, stack)
    }

    /// Even split: all windows share space equally.
    fn build_even_split(&self, direction: SplitDirection) -> SplitNode {
        build_even_recursive(&self.tiled_surfaces, direction)
    }

    /// Grid: arrange windows in columns and rows.
    #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
    fn build_grid(&self) -> SplitNode {
        let n = self.tiled_surfaces.len();
        if n == 1 {
            return SplitNode::leaf(self.tiled_surfaces[0]);
        }

        // Compute grid dimensions (ceil(sqrt(n)) columns)
        let cols = (n as f32).sqrt().ceil() as usize;
        let rows = n.div_ceil(cols);

        // Build row by row, then combine rows horizontally
        let mut row_nodes = Vec::new();
        for row in 0..rows {
            let start = row * cols;
            let end = (start + cols).min(n);
            if start >= n {
                break;
            }
            let row_surfaces = &self.tiled_surfaces[start..end];
            let row_node = build_even_recursive(row_surfaces, SplitDirection::Vertical);
            row_nodes.push(row_node);
        }

        // Combine rows with even horizontal splits
        combine_nodes_recursive(&row_nodes, SplitDirection::Horizontal)
    }

    /// BSP: alternate vertical/horizontal splits.
    fn build_bsp(&self) -> SplitNode {
        build_bsp_recursive(&self.tiled_surfaces, SplitDirection::Vertical)
    }
}

/// Build a vertical stack recursively (free function to satisfy clippy).
#[allow(clippy::cast_precision_loss)]
fn build_vertical_stack_recursive(surfaces: &[SurfaceId]) -> SplitNode {
    if surfaces.len() == 1 {
        return SplitNode::leaf(surfaces[0]);
    }

    let n = surfaces.len();
    let ratio = 1.0 / n as f32;
    let first = SplitNode::leaf(surfaces[0]);
    let rest = build_vertical_stack_recursive(&surfaces[1..]);

    SplitNode::split(SplitDirection::Horizontal, ratio, first, rest)
}

/// Even-split recursive builder (free function to satisfy clippy).
#[allow(clippy::cast_precision_loss)]
fn build_even_recursive(surfaces: &[SurfaceId], direction: SplitDirection) -> SplitNode {
    if surfaces.len() == 1 {
        return SplitNode::leaf(surfaces[0]);
    }

    let mid = surfaces.len() / 2;
    let (left, right) = surfaces.split_at(mid);
    let ratio = mid as f32 / surfaces.len() as f32;

    SplitNode::split(
        direction,
        ratio,
        build_even_recursive(left, direction),
        build_even_recursive(right, direction),
    )
}

/// Combine nodes into a balanced tree (free function to satisfy clippy).
#[allow(clippy::cast_precision_loss)]
fn combine_nodes_recursive(nodes: &[SplitNode], direction: SplitDirection) -> SplitNode {
    if nodes.len() == 1 {
        return nodes[0].clone();
    }

    let mid = nodes.len() / 2;
    let ratio = mid as f32 / nodes.len() as f32;

    SplitNode::split(
        direction,
        ratio,
        combine_nodes_recursive(&nodes[..mid], direction),
        combine_nodes_recursive(&nodes[mid..], direction),
    )
}

/// BSP recursive builder (free function to satisfy clippy).
#[allow(clippy::cast_precision_loss)]
fn build_bsp_recursive(surfaces: &[SurfaceId], direction: SplitDirection) -> SplitNode {
    if surfaces.len() == 1 {
        return SplitNode::leaf(surfaces[0]);
    }

    let mid = surfaces.len() / 2;
    let (left, right) = surfaces.split_at(mid);
    let ratio = mid as f32 / surfaces.len() as f32;
    let next_dir = match direction {
        SplitDirection::Vertical => SplitDirection::Horizontal,
        SplitDirection::Horizontal => SplitDirection::Vertical,
    };

    SplitNode::split(
        direction,
        ratio,
        build_bsp_recursive(left, next_dir),
        build_bsp_recursive(right, next_dir),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    fn ids(n: u32) -> Vec<SurfaceId> {
        (1..=n).map(SurfaceId::new).collect()
    }

    // --- SplitNode tests ---

    #[test]
    fn split_node_leaf() {
        let node = SplitNode::leaf(SurfaceId::new(1));
        assert_eq!(node.window_count(), 1);
        assert!(node.contains(SurfaceId::new(1)));
        assert!(!node.contains(SurfaceId::new(2)));
    }

    #[test]
    fn split_node_tree() {
        let tree = SplitNode::split(
            SplitDirection::Vertical,
            0.5,
            SplitNode::leaf(SurfaceId::new(1)),
            SplitNode::leaf(SurfaceId::new(2)),
        );
        assert_eq!(tree.window_count(), 2);
        assert!(tree.contains(SurfaceId::new(1)));
        assert!(tree.contains(SurfaceId::new(2)));
        assert!(!tree.contains(SurfaceId::new(3)));
    }

    #[test]
    fn split_node_surface_ids() {
        let tree = SplitNode::split(
            SplitDirection::Vertical,
            0.5,
            SplitNode::leaf(SurfaceId::new(1)),
            SplitNode::split(
                SplitDirection::Horizontal,
                0.5,
                SplitNode::leaf(SurfaceId::new(2)),
                SplitNode::leaf(SurfaceId::new(3)),
            ),
        );
        let ids = tree.surface_ids();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], SurfaceId::new(1));
        assert_eq!(ids[1], SurfaceId::new(2));
        assert_eq!(ids[2], SurfaceId::new(3));
    }

    #[test]
    fn split_node_remove_leaf() {
        let tree = SplitNode::split(
            SplitDirection::Vertical,
            0.5,
            SplitNode::leaf(SurfaceId::new(1)),
            SplitNode::leaf(SurfaceId::new(2)),
        );

        // Remove one child — tree collapses to the remaining leaf
        let result = tree.remove(SurfaceId::new(1));
        assert!(result.is_some());
        if let Some(node) = result {
            assert_eq!(node.window_count(), 1);
            assert!(node.contains(SurfaceId::new(2)));
        }
    }

    #[test]
    fn split_node_remove_last() {
        let node = SplitNode::leaf(SurfaceId::new(1));
        let result = node.remove(SurfaceId::new(1));
        assert!(result.is_none());
    }

    #[test]
    fn split_node_compute_rects_leaf() {
        let node = SplitNode::leaf(SurfaceId::new(1));
        let rects = node.compute_rects(&workspace(), 0);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].0, SurfaceId::new(1));
        assert_eq!(rects[0].1, workspace());
    }

    #[test]
    fn split_node_compute_rects_with_gap() {
        let node = SplitNode::leaf(SurfaceId::new(1));
        let rects = node.compute_rects(&workspace(), 4);
        assert_eq!(rects.len(), 1);
        let tile = rects[0].1;
        assert_eq!(tile.x, 4);
        assert_eq!(tile.y, 4);
        assert_eq!(tile.width, 1912); // 1920 - 8
        assert_eq!(tile.height, 1072); // 1080 - 8
    }

    #[test]
    fn split_node_vertical_split() {
        let tree = SplitNode::split(
            SplitDirection::Vertical,
            0.5,
            SplitNode::leaf(SurfaceId::new(1)),
            SplitNode::leaf(SurfaceId::new(2)),
        );
        let rects = tree.compute_rects(&workspace(), 0);
        assert_eq!(rects.len(), 2);

        // Left half
        assert_eq!(rects[0].1.x, 0);
        assert_eq!(rects[0].1.width, 960);
        // Right half
        assert_eq!(rects[1].1.x, 960);
        assert_eq!(rects[1].1.width, 960);
        // Both full height
        assert_eq!(rects[0].1.height, 1080);
        assert_eq!(rects[1].1.height, 1080);
    }

    #[test]
    fn split_node_horizontal_split() {
        let tree = SplitNode::split(
            SplitDirection::Horizontal,
            0.5,
            SplitNode::leaf(SurfaceId::new(1)),
            SplitNode::leaf(SurfaceId::new(2)),
        );
        let rects = tree.compute_rects(&workspace(), 0);
        assert_eq!(rects.len(), 2);

        // Top half
        assert_eq!(rects[0].1.y, 0);
        assert_eq!(rects[0].1.height, 540);
        // Bottom half
        assert_eq!(rects[1].1.y, 540);
        assert_eq!(rects[1].1.height, 540);
    }

    #[test]
    fn ratio_clamped() {
        let node = SplitNode::split(
            SplitDirection::Vertical,
            0.0, // Too small — clamped to 0.1
            SplitNode::leaf(SurfaceId::new(1)),
            SplitNode::leaf(SurfaceId::new(2)),
        );
        if let SplitNode::Split { ratio, .. } = node {
            assert!((ratio - 0.1).abs() < f32::EPSILON);
        }
    }

    // --- TilingEngine tests ---

    #[test]
    fn engine_empty() {
        let engine = TilingEngine::new(workspace());
        assert_eq!(engine.tiled_count(), 0);
        assert!(engine.compute_layout().is_empty());
    }

    #[test]
    fn engine_single_window() {
        let mut engine = TilingEngine::new(workspace());
        engine.add_surface(SurfaceId::new(1));

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 1);
        // Single window fills workspace minus gap
        let tile = layout[0].1;
        assert_eq!(tile.x, 4);
        assert_eq!(tile.y, 4);
    }

    #[test]
    fn engine_master_stack_two() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);
        engine.set_master_ratio(0.5);
        engine.add_surface(SurfaceId::new(1));
        engine.add_surface(SurfaceId::new(2));

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 2);

        // Master (left half)
        assert_eq!(layout[0].0, SurfaceId::new(1));
        assert_eq!(layout[0].1.x, 0);
        assert_eq!(layout[0].1.width, 960);

        // Stack (right half)
        assert_eq!(layout[1].0, SurfaceId::new(2));
        assert_eq!(layout[1].1.x, 960);
        assert_eq!(layout[1].1.width, 960);
    }

    #[test]
    fn engine_master_stack_three() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);
        engine.set_master_ratio(0.5);
        engine.add_surface(SurfaceId::new(1));
        engine.add_surface(SurfaceId::new(2));
        engine.add_surface(SurfaceId::new(3));

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 3);

        // Master fills left half
        assert_eq!(layout[0].1.width, 960);
        assert_eq!(layout[0].1.height, 1080);

        // Stack windows share the right half vertically
        assert_eq!(layout[1].1.x, 960);
        assert_eq!(layout[2].1.x, 960);
        // Together they should fill 1080 height
        assert_eq!(layout[1].1.height + layout[2].1.height, 1080);
    }

    #[test]
    fn engine_vertical_split() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_layout(TilingLayout::VerticalSplit);
        engine.set_gap(0);

        for id in ids(3) {
            engine.add_surface(id);
        }

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 3);

        // All should have full height
        for (_, rect) in &layout {
            assert_eq!(rect.height, 1080);
        }
    }

    #[test]
    fn engine_horizontal_split() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_layout(TilingLayout::HorizontalSplit);
        engine.set_gap(0);

        for id in ids(2) {
            engine.add_surface(id);
        }

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 2);

        // All should have full width
        for (_, rect) in &layout {
            assert_eq!(rect.width, 1920);
        }
    }

    #[test]
    fn engine_grid_four() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_layout(TilingLayout::Grid);
        engine.set_gap(0);

        for id in ids(4) {
            engine.add_surface(id);
        }

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 4);

        // 4 windows → 2x2 grid, each 960x540
        for (_, rect) in &layout {
            assert_eq!(rect.width, 960);
            assert_eq!(rect.height, 540);
        }
    }

    #[test]
    fn engine_monocle() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_layout(TilingLayout::Monocle);
        engine.set_gap(0);

        for id in ids(3) {
            engine.add_surface(id);
        }

        let layout = engine.compute_layout();
        // Monocle only tiles the first window
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].0, SurfaceId::new(1));
        assert_eq!(layout[0].1, workspace());
    }

    #[test]
    fn engine_bsp() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_layout(TilingLayout::Bsp);
        engine.set_gap(0);

        for id in ids(4) {
            engine.add_surface(id);
        }

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 4);

        // BSP alternates V/H splits
        // 4 windows: V-split → left: H-split(1,2), right: H-split(3,4)
        // Window 1: top-left quarter
        // Window 2: bottom-left quarter
        // Window 3: top-right quarter
        // Window 4: bottom-right quarter
        let total_area: u64 = layout.iter().map(|(_, r)| r.area()).sum();
        assert_eq!(total_area, workspace().area()); // Area conservation
    }

    #[test]
    fn engine_add_remove() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);

        engine.add_surface(SurfaceId::new(1));
        engine.add_surface(SurfaceId::new(2));
        assert_eq!(engine.tiled_count(), 2);

        engine.remove_surface(SurfaceId::new(1));
        assert_eq!(engine.tiled_count(), 1);

        let layout = engine.compute_layout();
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].0, SurfaceId::new(2));
    }

    #[test]
    fn engine_swap() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);
        engine.set_master_ratio(0.5);

        engine.add_surface(SurfaceId::new(1));
        engine.add_surface(SurfaceId::new(2));

        // Before swap: 1 is master, 2 is stack
        let layout = engine.compute_layout();
        assert_eq!(layout[0].0, SurfaceId::new(1));

        // After swap: 2 is master, 1 is stack
        engine.swap_surfaces(SurfaceId::new(1), SurfaceId::new(2));
        let layout = engine.compute_layout();
        assert_eq!(layout[0].0, SurfaceId::new(2));
    }

    #[test]
    fn engine_promote_to_master() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);

        for id in ids(3) {
            engine.add_surface(id);
        }

        // Promote window 3 to master
        engine.promote_to_master(SurfaceId::new(3));
        let layout = engine.compute_layout();
        assert_eq!(layout[0].0, SurfaceId::new(3));
    }

    #[test]
    fn engine_cycle_layout() {
        let mut engine = TilingEngine::new(workspace());
        assert_eq!(engine.layout(), TilingLayout::MasterStack);

        engine.cycle_layout();
        assert_eq!(engine.layout(), TilingLayout::VerticalSplit);

        engine.cycle_layout();
        assert_eq!(engine.layout(), TilingLayout::HorizontalSplit);

        engine.cycle_layout();
        assert_eq!(engine.layout(), TilingLayout::Grid);

        engine.cycle_layout();
        assert_eq!(engine.layout(), TilingLayout::Monocle);

        engine.cycle_layout();
        assert_eq!(engine.layout(), TilingLayout::Bsp);

        engine.cycle_layout();
        assert_eq!(engine.layout(), TilingLayout::MasterStack);
    }

    #[test]
    fn engine_duplicate_add_ignored() {
        let mut engine = TilingEngine::new(workspace());
        engine.add_surface(SurfaceId::new(1));
        engine.add_surface(SurfaceId::new(1)); // duplicate
        assert_eq!(engine.tiled_count(), 1);
    }

    #[test]
    fn area_conservation_all_layouts() {
        let ws = workspace();
        let layouts = [
            TilingLayout::MasterStack,
            TilingLayout::VerticalSplit,
            TilingLayout::HorizontalSplit,
            TilingLayout::Grid,
            TilingLayout::Bsp,
        ];

        for layout in &layouts {
            let mut engine = TilingEngine::new(ws);
            engine.set_layout(*layout);
            engine.set_gap(0);

            for id in ids(4) {
                engine.add_surface(id);
            }

            let tiles = engine.compute_layout();
            let total_area: u64 = tiles.iter().map(|(_, r)| r.area()).sum();
            assert_eq!(
                total_area,
                ws.area(),
                "Area conservation violated for layout {layout:?}"
            );
        }
    }

    #[test]
    fn no_overlap_master_stack() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);
        for id in ids(4) {
            engine.add_surface(id);
        }

        let tiles = engine.compute_layout();

        // Check no two tiles overlap
        for i in 0..tiles.len() {
            for j in (i + 1)..tiles.len() {
                let a = &tiles[i].1;
                let b = &tiles[j].1;
                let overlap = rects_overlap(a, b);
                assert!(!overlap, "Tiles {i} and {j} overlap: {a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn gap_reduces_tile_size() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);
        engine.add_surface(SurfaceId::new(1));
        let no_gap = engine.compute_layout();

        engine.set_gap(8);
        let with_gap = engine.compute_layout();

        assert!(with_gap[0].1.width < no_gap[0].1.width);
        assert!(with_gap[0].1.height < no_gap[0].1.height);
    }

    #[test]
    fn master_ratio_affects_layout() {
        let mut engine = TilingEngine::new(workspace());
        engine.set_gap(0);
        engine.add_surface(SurfaceId::new(1));
        engine.add_surface(SurfaceId::new(2));

        engine.set_master_ratio(0.7);
        let layout70 = engine.compute_layout();

        engine.set_master_ratio(0.3);
        let layout30 = engine.compute_layout();

        // 70% master should be wider than 30% master
        assert!(layout70[0].1.width > layout30[0].1.width);
    }

    /// Helper: check if two rectangles overlap.
    #[allow(clippy::cast_possible_wrap)]
    fn rects_overlap(a: &Rect, b: &Rect) -> bool {
        let a_right = a.x + a.width as i32;
        let a_bottom = a.y + a.height as i32;
        let b_right = b.x + b.width as i32;
        let b_bottom = b.y + b.height as i32;

        a.x < b_right && a_right > b.x && a.y < b_bottom && a_bottom > b.y
    }
}
