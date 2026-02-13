// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! App launcher — grid/list views for discovering and launching apps.
//!
//! ## Architecture
//!
//! ```text
//! ┌───────────────────────────────────┐
//! │          AppLauncher               │
//! │                                    │
//! │  ┌──────────┐   ┌──────────────┐  │
//! │  │ SearchBar │   │  View Mode   │  │
//! │  │ [filter] │   │ Grid / List  │  │
//! │  └──────────┘   └──────┬───────┘  │
//! │                        │          │
//! │  ┌─────────────────────▼────────┐ │
//! │  │        AppGrid / AppList     │ │
//! │  │  ┌────┐ ┌────┐ ┌────┐       │ │
//! │  │  │Icon│ │Icon│ │Icon│  ...   │ │
//! │  │  │Name│ │Name│ │Name│        │ │
//! │  │  └────┘ └────┘ └────┘       │ │
//! │  └──────────────────────────────┘ │
//! └───────────────────────────────────┘
//! ```
//!
//! ## Form Factor Grid Sizes
//!
//! | Device | Grid | Cell Size | View Default |
//! |--------|------|-----------|-------------|
//! | Watch | 2x3 | 200x130 | List |
//! | Phone | 4x5 | 240x400 | Grid |
//! | Desktop | 6x4 | 280x220 | Grid |
//!
//! ## Primitive Grounding
//!
//! - Σ Sum: Collection of launchable apps
//! - μ Mapping: Search query → filtered set
//! - κ Comparison: Alphabetical sorting
//! - λ Location: Grid cell positions
//! - ∂ Boundary: Cell bounds within grid

use nexcore_compositor::surface::Rect;
use nexcore_pal::FormFactor;

use crate::app::{App, AppId, AppState};

/// Launcher view mode.
///
/// Tier: T2-P (ς State — display mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherView {
    /// Icon grid — 2D grid of app icons with labels.
    Grid,
    /// Vertical list — single column with icon + name + description.
    List,
}

/// Grid dimensions for a form factor.
///
/// Tier: T2-P (N Quantity — grid parameters)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridConfig {
    /// Number of columns.
    pub columns: u32,
    /// Number of visible rows (before scrolling).
    pub visible_rows: u32,
    /// Cell width in pixels.
    pub cell_width: u32,
    /// Cell height in pixels.
    pub cell_height: u32,
    /// Gap between cells in pixels.
    pub gap: u32,
}

impl GridConfig {
    /// Grid config for watch (2x3, compact).
    pub const fn watch() -> Self {
        Self {
            columns: 2,
            visible_rows: 3,
            cell_width: 200,
            cell_height: 130,
            gap: 10,
        }
    }

    /// Grid config for phone (4x5, touch-friendly).
    pub const fn phone() -> Self {
        Self {
            columns: 4,
            visible_rows: 5,
            cell_width: 240,
            cell_height: 400,
            gap: 15,
        }
    }

    /// Grid config for desktop (6x4, spacious).
    pub const fn desktop() -> Self {
        Self {
            columns: 6,
            visible_rows: 4,
            cell_width: 280,
            cell_height: 220,
            gap: 20,
        }
    }

    /// Select grid config for a form factor.
    pub const fn for_form_factor(ff: FormFactor) -> Self {
        match ff {
            FormFactor::Watch => Self::watch(),
            FormFactor::Phone => Self::phone(),
            FormFactor::Desktop => Self::desktop(),
        }
    }

    /// Total cells visible without scrolling.
    pub const fn visible_cells(&self) -> u32 {
        self.columns * self.visible_rows
    }

    /// Total grid width (all columns + gaps).
    pub const fn total_width(&self) -> u32 {
        self.columns * self.cell_width + (self.columns.saturating_sub(1)) * self.gap
    }

    /// Total grid height for visible rows.
    pub const fn total_height(&self) -> u32 {
        self.visible_rows * self.cell_height + (self.visible_rows.saturating_sub(1)) * self.gap
    }
}

/// A positioned app cell in the launcher.
///
/// Tier: T2-C (λ + ∂ + ∃ — positioned app entry)
#[derive(Debug, Clone)]
pub struct LauncherCell {
    /// App ID.
    pub app_id: AppId,
    /// App display name.
    pub name: String,
    /// App state.
    pub state: AppState,
    /// Cell position on screen.
    pub bounds: Rect,
    /// Whether this cell is selected/highlighted.
    pub selected: bool,
}

/// The app launcher — shows available apps and launches them.
///
/// Tier: T3 (Σ + μ + κ + λ + ∂ — full launcher integration)
pub struct AppLauncher {
    /// Current view mode.
    view: LauncherView,
    /// Grid configuration.
    grid_config: GridConfig,
    /// Form factor.
    form_factor: FormFactor,
    /// Screen bounds for the launcher.
    bounds: Rect,
    /// Search/filter query.
    search_query: String,
    /// Currently selected index.
    selected_index: usize,
    /// Scroll offset (in rows for grid, items for list).
    scroll_offset: usize,
    /// Whether the launcher is visible.
    visible: bool,
}

impl AppLauncher {
    /// Create a new launcher for a form factor.
    pub fn new(form_factor: FormFactor, bounds: Rect) -> Self {
        let default_view = match form_factor {
            FormFactor::Watch => LauncherView::List,
            FormFactor::Phone | FormFactor::Desktop => LauncherView::Grid,
        };

        Self {
            view: default_view,
            grid_config: GridConfig::for_form_factor(form_factor),
            form_factor,
            bounds,
            search_query: String::new(),
            selected_index: 0,
            scroll_offset: 0,
            visible: false,
        }
    }

    /// Show the launcher.
    pub fn show(&mut self) {
        self.visible = true;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.search_query.clear();
    }

    /// Hide the launcher.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Whether the launcher is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggle between Grid and List view.
    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            LauncherView::Grid => LauncherView::List,
            LauncherView::List => LauncherView::Grid,
        };
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Set the view mode.
    pub fn set_view(&mut self, view: LauncherView) {
        if self.view != view {
            self.view = view;
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    /// Get the current view mode.
    pub fn view(&self) -> LauncherView {
        self.view
    }

    /// Set the search/filter query.
    pub fn set_search(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Append a character to the search query.
    pub fn search_append(&mut self, ch: char) {
        self.search_query.push(ch);
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Remove the last character from the search query.
    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.selected_index = 0;
    }

    /// Clear the search query.
    pub fn search_clear(&mut self) {
        self.search_query.clear();
        self.selected_index = 0;
    }

    /// Get the current search query.
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Filter apps by current search query.
    fn filter_apps<'a>(&self, apps: &'a [App]) -> Vec<&'a App> {
        let mut filtered: Vec<&App> = if self.search_query.is_empty() {
            apps.iter().collect()
        } else {
            let query_lower = self.search_query.to_lowercase();
            apps.iter()
                .filter(|app| {
                    app.name.to_lowercase().contains(&query_lower)
                        || app.id.as_str().to_lowercase().contains(&query_lower)
                })
                .collect()
        };

        // Sort alphabetically by name
        filtered.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        filtered
    }

    /// Move selection up.
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down (requires app count to bound).
    pub fn select_next(&mut self, app_count: usize) {
        if app_count > 0 && self.selected_index < app_count - 1 {
            self.selected_index += 1;
        }
    }

    /// Move selection left in grid mode.
    pub fn select_left(&mut self) {
        let cols = self.grid_config.columns as usize;
        if self.view == LauncherView::Grid && self.selected_index % cols > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection right in grid mode.
    pub fn select_right(&mut self, app_count: usize) {
        let cols = self.grid_config.columns as usize;
        if self.view == LauncherView::Grid
            && self.selected_index % cols < cols - 1
            && self.selected_index + 1 < app_count
        {
            self.selected_index += 1;
        }
    }

    /// Get the selected index.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get the AppId of the currently selected app.
    pub fn selected_app<'a>(&self, apps: &'a [App]) -> Option<&'a App> {
        let filtered = self.filter_apps(apps);
        filtered.get(self.selected_index).copied()
    }

    /// Scroll down.
    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    /// Scroll up.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Compute the positioned cells for rendering.
    ///
    /// Returns a vector of `LauncherCell` with screen positions.
    #[allow(clippy::cast_possible_wrap)]
    pub fn compute_cells(&self, apps: &[App]) -> Vec<LauncherCell> {
        let filtered = self.filter_apps(apps);
        if filtered.is_empty() {
            return Vec::new();
        }

        match self.view {
            LauncherView::Grid => self.compute_grid_cells(&filtered),
            LauncherView::List => self.compute_list_cells(&filtered),
        }
    }

    /// Compute grid-mode cell positions.
    #[allow(clippy::cast_possible_wrap)]
    fn compute_grid_cells(&self, apps: &[&App]) -> Vec<LauncherCell> {
        let gc = &self.grid_config;
        let cols = gc.columns as usize;
        let visible = gc.visible_cells() as usize;
        let skip = self.scroll_offset * cols;

        // Center grid within bounds
        let grid_w = gc.total_width();
        let offset_x = if self.bounds.width > grid_w {
            (self.bounds.width - grid_w) / 2
        } else {
            0
        };

        apps.iter()
            .skip(skip)
            .take(visible)
            .enumerate()
            .map(|(idx, app)| {
                let row = idx / cols;
                let col = idx % cols;
                let x = self.bounds.x
                    + offset_x as i32
                    + (col as u32 * (gc.cell_width + gc.gap)) as i32;
                let y = self.bounds.y + (row as u32 * (gc.cell_height + gc.gap)) as i32;

                LauncherCell {
                    app_id: app.id.clone(),
                    name: app.name.clone(),
                    state: app.state,
                    bounds: Rect::new(x, y, gc.cell_width, gc.cell_height),
                    selected: idx + skip == self.selected_index,
                }
            })
            .collect()
    }

    /// Compute list-mode cell positions.
    #[allow(clippy::cast_possible_wrap)]
    fn compute_list_cells(&self, apps: &[&App]) -> Vec<LauncherCell> {
        let row_height = 60_u32;
        let row_gap = 4_u32;
        let visible_rows = (self.bounds.height / (row_height + row_gap)) as usize;

        apps.iter()
            .skip(self.scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(idx, app)| {
                let y = self.bounds.y + (idx as u32 * (row_height + row_gap)) as i32;

                LauncherCell {
                    app_id: app.id.clone(),
                    name: app.name.clone(),
                    state: app.state,
                    bounds: Rect::new(self.bounds.x, y, self.bounds.width, row_height),
                    selected: idx + self.scroll_offset == self.selected_index,
                }
            })
            .collect()
    }

    /// Get the grid configuration.
    pub fn grid_config(&self) -> &GridConfig {
        &self.grid_config
    }

    /// Get the form factor.
    pub fn form_factor(&self) -> FormFactor {
        self.form_factor
    }

    /// Get the launcher bounds.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Get the total number of apps matching current filter.
    pub fn filtered_count(&self, apps: &[App]) -> usize {
        self.filter_apps(apps).len()
    }

    /// Whether any apps match the current search.
    pub fn has_results(&self, apps: &[App]) -> bool {
        self.filtered_count(apps) > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_apps() -> Vec<App> {
        vec![
            App::new("guardian", "Guardian"),
            App::new("settings", "Settings"),
            App::new("browser", "Browser"),
            App::new("terminal", "Terminal"),
            App::new("files", "Files"),
            App::new("music", "Music Player"),
            App::new("camera", "Camera"),
            App::new("launcher", "Launcher"),
        ]
    }

    // ── GridConfig tests ──

    #[test]
    fn grid_config_watch() {
        let gc = GridConfig::watch();
        assert_eq!(gc.columns, 2);
        assert_eq!(gc.visible_rows, 3);
        assert_eq!(gc.visible_cells(), 6);
    }

    #[test]
    fn grid_config_phone() {
        let gc = GridConfig::phone();
        assert_eq!(gc.columns, 4);
        assert_eq!(gc.visible_rows, 5);
        assert_eq!(gc.visible_cells(), 20);
    }

    #[test]
    fn grid_config_desktop() {
        let gc = GridConfig::desktop();
        assert_eq!(gc.columns, 6);
        assert_eq!(gc.visible_rows, 4);
        assert_eq!(gc.visible_cells(), 24);
    }

    #[test]
    fn grid_config_dimensions() {
        let gc = GridConfig::desktop();
        // 6 * 280 + 5 * 20 = 1680 + 100 = 1780
        assert_eq!(gc.total_width(), 1780);
        // 4 * 220 + 3 * 20 = 880 + 60 = 940
        assert_eq!(gc.total_height(), 940);
    }

    // ── Launcher creation tests ──

    #[test]
    fn launcher_watch_defaults_to_list() {
        let launcher = AppLauncher::new(FormFactor::Watch, Rect::new(0, 0, 450, 410));
        assert_eq!(launcher.view(), LauncherView::List);
        assert!(!launcher.is_visible());
    }

    #[test]
    fn launcher_phone_defaults_to_grid() {
        let launcher = AppLauncher::new(FormFactor::Phone, Rect::new(0, 0, 1080, 2200));
        assert_eq!(launcher.view(), LauncherView::Grid);
    }

    #[test]
    fn launcher_desktop_defaults_to_grid() {
        let launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        assert_eq!(launcher.view(), LauncherView::Grid);
    }

    // ── Visibility tests ──

    #[test]
    fn show_hide() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        assert!(!launcher.is_visible());

        launcher.show();
        assert!(launcher.is_visible());

        launcher.hide();
        assert!(!launcher.is_visible());
    }

    #[test]
    fn show_resets_state() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.set_search("test");
        launcher.select_next(5);

        launcher.show();
        assert_eq!(launcher.search_query(), "");
        assert_eq!(launcher.selected_index(), 0);
    }

    // ── View toggle tests ──

    #[test]
    fn toggle_view() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        assert_eq!(launcher.view(), LauncherView::Grid);

        launcher.toggle_view();
        assert_eq!(launcher.view(), LauncherView::List);

        launcher.toggle_view();
        assert_eq!(launcher.view(), LauncherView::Grid);
    }

    #[test]
    fn set_view() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.set_view(LauncherView::List);
        assert_eq!(launcher.view(), LauncherView::List);
    }

    // ── Search tests ──

    #[test]
    fn search_filters_apps() {
        let launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        let apps = test_apps();
        assert_eq!(launcher.filtered_count(&apps), 8);
    }

    #[test]
    fn search_by_name() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        let apps = test_apps();

        launcher.set_search("Guard");
        assert_eq!(launcher.filtered_count(&apps), 1);
        assert!(launcher.has_results(&apps));
    }

    #[test]
    fn search_by_id() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        let apps = test_apps();

        launcher.set_search("terminal");
        assert_eq!(launcher.filtered_count(&apps), 1);
    }

    #[test]
    fn search_case_insensitive() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        let apps = test_apps();

        launcher.set_search("BROWSER");
        assert_eq!(launcher.filtered_count(&apps), 1);
    }

    #[test]
    fn search_no_results() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        let apps = test_apps();

        launcher.set_search("nonexistent");
        assert_eq!(launcher.filtered_count(&apps), 0);
        assert!(!launcher.has_results(&apps));
    }

    #[test]
    fn search_append_and_backspace() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));

        launcher.search_append('h');
        launcher.search_append('e');
        assert_eq!(launcher.search_query(), "he");

        launcher.search_backspace();
        assert_eq!(launcher.search_query(), "h");

        launcher.search_clear();
        assert_eq!(launcher.search_query(), "");
    }

    // ── Selection tests ──

    #[test]
    fn select_next_prev() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        assert_eq!(launcher.selected_index(), 0);

        launcher.select_next(8);
        assert_eq!(launcher.selected_index(), 1);

        launcher.select_next(8);
        assert_eq!(launcher.selected_index(), 2);

        launcher.select_prev();
        assert_eq!(launcher.selected_index(), 1);
    }

    #[test]
    fn select_prev_at_zero() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.select_prev();
        assert_eq!(launcher.selected_index(), 0); // cannot go below 0
    }

    #[test]
    fn select_next_at_end() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        for _ in 0..20 {
            launcher.select_next(3);
        }
        assert_eq!(launcher.selected_index(), 2); // bounded at count - 1
    }

    #[test]
    fn select_left_right_grid() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        // Grid: 6 columns

        launcher.select_right(8);
        assert_eq!(launcher.selected_index(), 1);

        launcher.select_right(8);
        assert_eq!(launcher.selected_index(), 2);

        launcher.select_left();
        assert_eq!(launcher.selected_index(), 1);
    }

    #[test]
    fn select_left_at_column_zero() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.select_left();
        assert_eq!(launcher.selected_index(), 0); // already at col 0
    }

    #[test]
    fn selected_app() {
        let launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        let apps = test_apps();

        let selected = launcher.selected_app(&apps);
        assert!(selected.is_some());
        // Apps are sorted alphabetically, so first is "Browser"
        if let Some(app) = selected {
            assert_eq!(app.name, "Browser");
        }
    }

    // ── Grid cell computation tests ──

    #[test]
    fn compute_grid_cells() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.show();
        let apps = test_apps();

        let cells = launcher.compute_cells(&apps);
        assert_eq!(cells.len(), 8); // all 8 fit in 24-cell grid

        // Verify cells have valid bounds
        for cell in &cells {
            assert!(cell.bounds.width > 0);
            assert!(cell.bounds.height > 0);
        }
    }

    #[test]
    fn compute_grid_cells_with_selection() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.show();
        let apps = test_apps();

        let cells = launcher.compute_cells(&apps);
        // First cell should be selected
        assert!(cells[0].selected);
        assert!(!cells[1].selected);
    }

    #[test]
    fn compute_list_cells() {
        let mut launcher = AppLauncher::new(FormFactor::Watch, Rect::new(0, 40, 450, 410));
        launcher.show();
        let apps = test_apps();

        let cells = launcher.compute_cells(&apps);
        assert!(!cells.is_empty());

        // List cells span full width
        for cell in &cells {
            assert_eq!(cell.bounds.width, 450);
        }
    }

    #[test]
    fn cells_sorted_alphabetically() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.show();
        let apps = test_apps();

        let cells = launcher.compute_cells(&apps);
        assert!(cells.len() >= 2);
        // First app alphabetically should be "Browser"
        assert_eq!(cells[0].name, "Browser");
    }

    #[test]
    fn cells_filtered_by_search() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.show();
        launcher.set_search("er");
        let apps = test_apps();

        let cells = launcher.compute_cells(&apps);
        // "Browser", "Launcher", "Music Player" contain "er"
        assert!(cells.len() >= 2);
        for cell in &cells {
            let name_lower = cell.name.to_lowercase();
            let id_lower = cell.app_id.as_str().to_lowercase();
            assert!(
                name_lower.contains("er") || id_lower.contains("er"),
                "Cell '{}' (id='{}') should contain 'er'",
                cell.name,
                cell.app_id.as_str()
            );
        }
    }

    #[test]
    fn empty_app_list() {
        let mut launcher = AppLauncher::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 1032));
        launcher.show();
        let apps: Vec<App> = Vec::new();

        let cells = launcher.compute_cells(&apps);
        assert!(cells.is_empty());
    }

    // ── Scroll tests ──

    #[test]
    fn scroll_down_up() {
        let mut launcher = AppLauncher::new(FormFactor::Watch, Rect::new(0, 0, 450, 410));
        assert_eq!(launcher.scroll_offset, 0);

        launcher.scroll_down();
        assert_eq!(launcher.scroll_offset, 1);

        launcher.scroll_up();
        assert_eq!(launcher.scroll_offset, 0);

        // Cannot scroll above 0
        launcher.scroll_up();
        assert_eq!(launcher.scroll_offset, 0);
    }

    // ── Accessors ──

    #[test]
    fn form_factor_preserved() {
        let launcher = AppLauncher::new(FormFactor::Phone, Rect::new(0, 0, 1080, 2200));
        assert_eq!(launcher.form_factor(), FormFactor::Phone);
    }

    #[test]
    fn bounds_preserved() {
        let bounds = Rect::new(10, 20, 800, 600);
        let launcher = AppLauncher::new(FormFactor::Desktop, bounds);
        assert_eq!(launcher.bounds(), bounds);
    }

    #[test]
    fn grid_config_accessible() {
        let launcher = AppLauncher::new(FormFactor::Phone, Rect::new(0, 0, 1080, 2200));
        assert_eq!(launcher.grid_config().columns, 4);
    }
}
