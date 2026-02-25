// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Command Palette — the universal AI-first interaction surface.
//!
//! ## Core Thesis
//!
//! > "Every action starts as a search."
//!
//! The command palette unifies natural language, voice, and structured
//! commands into a single interaction surface. It is the primary way
//! users interact with NexCore OS — whether opening apps, finding files,
//! changing settings, or asking the AI a question.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │              User Input                          │
//! │  "open browser"  │  "wifi on"  │  "what time?"  │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │           Command Palette (μ + σ + ∂)           │
//! │                                                  │
//! │  PaletteMode ──► Source Filter ──► Ranked Results│
//! │                                                  │
//! │  Modes:                                          │
//! │    Universal  → search everywhere                │
//! │    Apps       → only installed apps              │
//! │    Files      → only local files                 │
//! │    Settings   → only system settings             │
//! │    Commands   → only system commands             │
//! │    AiChat     → free-form AI conversation        │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │           AI Partner Integration                 │
//! │  Input → Intent parsing → Suggestions → Action   │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Form Factor Behavior
//!
//! | Device  | Activation        | Input      | Results  |
//! |---------|-------------------|------------|----------|
//! | Watch   | Crown press       | Voice      | 2-3 items|
//! | Phone   | Swipe down        | Text+Voice | 5-8 items|
//! | Desktop | Super/Ctrl+Space  | Keyboard   | 8-12 items|
//!
//! ## Primitive Grounding
//!
//! - μ Mapping: Query → ranked results
//! - σ Sequence: Results ordered by relevance
//! - ∂ Boundary: Mode constrains search scope
//! - κ Comparison: Score-based ranking
//! - ∃ Existence: Each result is a real action target

use crate::ai_partner::{AiPartner, Intent, NavigationTarget, SearchScope};
use nexcore_pal::FormFactor;
use serde::{Deserialize, Serialize};

/// Palette mode — constrains the search scope.
///
/// Tier: T2-P (∂ Boundary — search domain constraint)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaletteMode {
    /// Search everything (default).
    #[default]
    Universal,
    /// Search installed apps.
    Apps,
    /// Search local files.
    Files,
    /// Search system settings.
    Settings,
    /// Search system commands.
    Commands,
    /// Free-form AI conversation.
    AiChat,
}

impl PaletteMode {
    /// Whether this mode accepts free-form text.
    pub const fn accepts_freeform(&self) -> bool {
        matches!(self, Self::Universal | Self::AiChat)
    }

    /// Mode prefix shown in the palette input.
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Universal => "",
            Self::Apps => "app: ",
            Self::Files => "file: ",
            Self::Settings => "set: ",
            Self::Commands => "cmd: ",
            Self::AiChat => "ask: ",
        }
    }

    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Universal => "Search Everything",
            Self::Apps => "Apps",
            Self::Files => "Files",
            Self::Settings => "Settings",
            Self::Commands => "Commands",
            Self::AiChat => "Ask AI",
        }
    }

    /// Icon hint for renderers.
    pub const fn icon_hint(&self) -> &'static str {
        match self {
            Self::Universal => "search",
            Self::Apps => "grid",
            Self::Files => "folder",
            Self::Settings => "gear",
            Self::Commands => "terminal",
            Self::AiChat => "sparkle",
        }
    }

    /// All available modes.
    pub const ALL: [Self; 6] = [
        Self::Universal,
        Self::Apps,
        Self::Files,
        Self::Settings,
        Self::Commands,
        Self::AiChat,
    ];
}

/// A result entry in the command palette.
///
/// Tier: T2-C (∃ + κ + μ — existing, ranked, mapped result)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteEntry {
    /// Unique entry ID.
    pub id: u64,
    /// Display title.
    pub title: String,
    /// Subtitle / description.
    pub subtitle: String,
    /// Source category.
    pub source: EntrySource,
    /// Relevance score (0.0 = irrelevant, 1.0 = exact match).
    pub score: f64,
    /// Icon hint for renderers.
    pub icon: String,
    /// The intent that executing this entry would trigger.
    pub intent: Intent,
}

/// Source category for a palette entry.
///
/// Tier: T2-P (λ Location — where the result came from)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntrySource {
    /// An installed application.
    App,
    /// A local file or directory.
    File,
    /// A system setting.
    Setting,
    /// A system command.
    Command,
    /// An AI-generated suggestion.
    AiSuggestion,
    /// A navigation target.
    Navigation,
}

impl EntrySource {
    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::App => "App",
            Self::File => "File",
            Self::Setting => "Setting",
            Self::Command => "Command",
            Self::AiSuggestion => "AI",
            Self::Navigation => "Go to",
        }
    }
}

/// The Command Palette.
///
/// Tier: T3 (μ + σ + ∂ + κ + ∃ — full search-rank-act engine)
pub struct CommandPalette {
    /// Current search query.
    query: String,
    /// Current mode.
    mode: PaletteMode,
    /// Whether the palette is visible.
    visible: bool,
    /// Current results.
    results: Vec<PaletteEntry>,
    /// Selected result index.
    selected: usize,
    /// Form factor.
    form_factor: FormFactor,
    /// Maximum visible results.
    max_visible: usize,
    /// Entry ID counter.
    entry_counter: u64,
    /// Registered apps (name, id) for matching.
    known_apps: Vec<(String, String)>,
    /// Registered settings for matching.
    known_settings: Vec<(String, String)>,
    /// Navigation shortcuts.
    nav_shortcuts: Vec<(String, NavigationTarget)>,
}

impl CommandPalette {
    /// Create a new command palette for a form factor.
    pub fn new(form_factor: FormFactor) -> Self {
        let max_visible = match form_factor {
            FormFactor::Watch => 3,
            FormFactor::Phone => 6,
            FormFactor::Desktop => 10,
            _ => 10,
        };

        let nav_shortcuts = vec![
            ("home".to_string(), NavigationTarget::Home),
            ("back".to_string(), NavigationTarget::Back),
            ("launcher".to_string(), NavigationTarget::Launcher),
            ("settings".to_string(), NavigationTarget::Settings),
            ("lock".to_string(), NavigationTarget::Lock),
        ];

        Self {
            query: String::new(),
            mode: PaletteMode::default(),
            visible: false,
            results: Vec::new(),
            selected: 0,
            form_factor,
            max_visible,
            entry_counter: 0,
            known_apps: Vec::new(),
            known_settings: Vec::new(),
            nav_shortcuts,
        }
    }

    /// Show the palette.
    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.results.clear();
        self.selected = 0;
        self.mode = PaletteMode::default();
    }

    /// Hide the palette.
    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.results.clear();
        self.selected = 0;
    }

    /// Toggle visibility.
    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.open();
        }
    }

    /// Whether the palette is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the current query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get the current mode.
    pub fn mode(&self) -> PaletteMode {
        self.mode
    }

    /// Set the mode.
    pub fn set_mode(&mut self, mode: PaletteMode) {
        self.mode = mode;
        self.refresh_results();
    }

    /// Get current results.
    pub fn results(&self) -> &[PaletteEntry] {
        &self.results
    }

    /// Get visible results (capped by max_visible).
    pub fn visible_results(&self) -> &[PaletteEntry] {
        let end = self.results.len().min(self.max_visible);
        &self.results[..end]
    }

    /// Get the selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get the selected entry.
    pub fn selected_entry(&self) -> Option<&PaletteEntry> {
        self.results.get(self.selected)
    }

    /// Maximum visible results for this form factor.
    pub fn max_visible(&self) -> usize {
        self.max_visible
    }

    /// Form factor.
    pub fn form_factor(&self) -> FormFactor {
        self.form_factor
    }

    /// Register known apps for search matching.
    pub fn register_apps(&mut self, apps: Vec<(String, String)>) {
        self.known_apps = apps;
    }

    /// Register known settings for search matching.
    pub fn register_settings(&mut self, settings: Vec<(String, String)>) {
        self.known_settings = settings;
    }

    // ── Input handling ──

    /// Type a character into the query.
    pub fn type_char(&mut self, c: char) {
        self.query.push(c);
        self.refresh_results();
    }

    /// Set the full query text (e.g., from voice input).
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.refresh_results();
    }

    /// Delete the last character.
    pub fn backspace(&mut self) {
        self.query.pop();
        self.refresh_results();
    }

    /// Clear the query.
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected = 0;
    }

    // ── Selection navigation ──

    /// Select the next result.
    pub fn select_next(&mut self) {
        if !self.results.is_empty() && self.selected < self.results.len() - 1 {
            self.selected += 1;
        }
    }

    /// Select the previous result.
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Execute the selected entry, returning its intent.
    pub fn execute_selected(&mut self) -> Option<Intent> {
        let entry = self.results.get(self.selected)?;
        let intent = entry.intent.clone();
        self.close();
        Some(intent)
    }

    /// Execute the selected entry through the AI partner.
    ///
    /// Returns the intent and AI suggestions.
    pub fn execute_with_ai(
        &mut self,
        ai: &mut AiPartner,
    ) -> Option<(Intent, Vec<crate::ai_partner::Suggestion>)> {
        let entry = self.results.get(self.selected)?;
        let intent = entry.intent.clone();
        let suggestions = ai.process_intent(&intent);
        self.close();
        Some((intent, suggestions))
    }

    // ── Result generation ──

    /// Refresh results based on current query and mode.
    fn refresh_results(&mut self) {
        self.results.clear();
        self.selected = 0;

        let query_lower = self.query.to_lowercase();
        if query_lower.is_empty() {
            return;
        }

        // Check for mode prefix shortcuts
        let effective_mode = self.detect_mode_prefix(&query_lower);
        let search_query = Self::strip_mode_prefix(&query_lower);

        if search_query.is_empty() {
            return;
        }

        match effective_mode {
            PaletteMode::Universal => {
                self.add_nav_results(search_query);
                self.add_app_results(search_query);
                self.add_setting_results(search_query);
                self.add_command_results(search_query);
                self.add_ai_result(search_query);
            }
            PaletteMode::Apps => self.add_app_results(search_query),
            PaletteMode::Files => self.add_file_results(search_query),
            PaletteMode::Settings => self.add_setting_results(search_query),
            PaletteMode::Commands => self.add_command_results(search_query),
            PaletteMode::AiChat => self.add_ai_result(search_query),
        }

        // Sort by score descending
        self.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Detect if the query starts with a mode prefix.
    fn detect_mode_prefix(&self, query: &str) -> PaletteMode {
        if self.mode != PaletteMode::Universal {
            return self.mode;
        }
        if query.starts_with("app:") || query.starts_with("app ") {
            PaletteMode::Apps
        } else if query.starts_with("file:") || query.starts_with("file ") {
            PaletteMode::Files
        } else if query.starts_with("set:") || query.starts_with("set ") {
            PaletteMode::Settings
        } else if query.starts_with("cmd:") || query.starts_with("cmd ") {
            PaletteMode::Commands
        } else if query.starts_with("ask:") || query.starts_with("ask ") {
            PaletteMode::AiChat
        } else {
            PaletteMode::Universal
        }
    }

    /// Strip the mode prefix from a query.
    fn strip_mode_prefix(query: &str) -> &str {
        for prefix in [
            "app:", "app ", "file:", "file ", "set:", "set ", "cmd:", "cmd ", "ask:", "ask ",
        ] {
            if let Some(rest) = query.strip_prefix(prefix) {
                return rest.trim();
            }
        }
        query
    }

    /// Add navigation shortcut results.
    fn add_nav_results(&mut self, query: &str) {
        let matches: Vec<_> = self
            .nav_shortcuts
            .iter()
            .filter(|(name, _)| name.contains(query))
            .map(|(name, target)| {
                let score = if name == query { 1.0 } else { 0.85 };
                (
                    format!("Go to {name}"),
                    "Navigation".to_string(),
                    score,
                    Intent::Navigate {
                        target: target.clone(),
                    },
                )
            })
            .collect();
        for (title, subtitle, score, intent) in matches {
            self.push_entry(
                title,
                subtitle,
                EntrySource::Navigation,
                score,
                "arrow".to_string(),
                intent,
            );
        }
    }

    /// Add matching app results.
    fn add_app_results(&mut self, query: &str) {
        let matches: Vec<_> = self
            .known_apps
            .iter()
            .filter_map(|(name, id)| {
                let name_lower = name.to_lowercase();
                let id_lower = id.to_lowercase();
                if name_lower.contains(query) || id_lower.contains(query) {
                    let score = if name_lower == query {
                        0.95
                    } else if name_lower.starts_with(query) {
                        0.85
                    } else {
                        0.70
                    };
                    Some((
                        name.clone(),
                        format!("App — {id}"),
                        score,
                        Intent::LaunchApp { app: id.clone() },
                    ))
                } else {
                    None
                }
            })
            .collect();
        for (title, subtitle, score, intent) in matches {
            self.push_entry(
                title,
                subtitle,
                EntrySource::App,
                score,
                "app".to_string(),
                intent,
            );
        }
    }

    /// Add matching setting results.
    fn add_setting_results(&mut self, query: &str) {
        let matches: Vec<_> = self
            .known_settings
            .iter()
            .filter_map(|(name, category)| {
                let name_lower = name.to_lowercase();
                if name_lower.contains(query) {
                    let score = if name_lower == query {
                        0.90
                    } else if name_lower.starts_with(query) {
                        0.80
                    } else {
                        0.65
                    };
                    Some((
                        name.clone(),
                        format!("Setting — {category}"),
                        score,
                        Intent::ChangeSetting {
                            setting: name.clone(),
                            value: String::new(),
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();
        for (title, subtitle, score, intent) in matches {
            self.push_entry(
                title,
                subtitle,
                EntrySource::Setting,
                score,
                "gear".to_string(),
                intent,
            );
        }
    }

    /// Add file search results.
    fn add_file_results(&mut self, query: &str) {
        // Files are searched lazily — we create a search intent
        self.push_entry(
            format!("Search files: '{query}'"),
            "Find files matching query".to_string(),
            EntrySource::File,
            0.75,
            "folder".to_string(),
            Intent::Search {
                query: query.to_string(),
                scope: SearchScope::Files,
            },
        );
    }

    /// Add command results.
    fn add_command_results(&mut self, query: &str) {
        self.push_entry(
            format!("Run: {query}"),
            "Execute as system command".to_string(),
            EntrySource::Command,
            0.50,
            "terminal".to_string(),
            Intent::RunCommand {
                command: query.to_string(),
            },
        );
    }

    /// Add an AI suggestion result.
    fn add_ai_result(&mut self, query: &str) {
        self.push_entry(
            format!("Ask AI: \"{query}\""),
            "Get an AI answer".to_string(),
            EntrySource::AiSuggestion,
            0.40,
            "sparkle".to_string(),
            Intent::Ask {
                question: query.to_string(),
            },
        );
    }

    /// Push a new entry to results.
    fn push_entry(
        &mut self,
        title: String,
        subtitle: String,
        source: EntrySource,
        score: f64,
        icon: String,
        intent: Intent,
    ) {
        self.entry_counter += 1;
        self.results.push(PaletteEntry {
            id: self.entry_counter,
            title,
            subtitle,
            source,
            score,
            icon,
            intent,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_palette() -> CommandPalette {
        let mut p = CommandPalette::new(FormFactor::Desktop);
        p.register_apps(vec![
            ("Browser".to_string(), "com.nexcore.browser".to_string()),
            ("Terminal".to_string(), "com.nexcore.terminal".to_string()),
            ("Settings".to_string(), "com.nexcore.settings".to_string()),
            ("Files".to_string(), "com.nexcore.files".to_string()),
        ]);
        p.register_settings(vec![
            ("WiFi".to_string(), "Network".to_string()),
            ("Bluetooth".to_string(), "Network".to_string()),
            ("Display Brightness".to_string(), "Display".to_string()),
            ("Dark Mode".to_string(), "Display".to_string()),
            ("Volume".to_string(), "Audio".to_string()),
        ]);
        p
    }

    // ── Lifecycle tests ──

    #[test]
    fn palette_creation() {
        let p = CommandPalette::new(FormFactor::Desktop);
        assert!(!p.is_visible());
        assert!(p.query().is_empty());
        assert_eq!(p.mode(), PaletteMode::Universal);
        assert_eq!(p.form_factor(), FormFactor::Desktop);
        assert_eq!(p.max_visible(), 10);
    }

    #[test]
    fn watch_max_visible() {
        let p = CommandPalette::new(FormFactor::Watch);
        assert_eq!(p.max_visible(), 3);
    }

    #[test]
    fn phone_max_visible() {
        let p = CommandPalette::new(FormFactor::Phone);
        assert_eq!(p.max_visible(), 6);
    }

    #[test]
    fn open_close() {
        let mut p = make_palette();
        p.open();
        assert!(p.is_visible());
        assert!(p.query().is_empty());

        p.close();
        assert!(!p.is_visible());
    }

    #[test]
    fn toggle() {
        let mut p = make_palette();
        p.toggle();
        assert!(p.is_visible());
        p.toggle();
        assert!(!p.is_visible());
    }

    // ── Input tests ──

    #[test]
    fn type_char_builds_query() {
        let mut p = make_palette();
        p.open();
        p.type_char('b');
        p.type_char('r');
        assert_eq!(p.query(), "br");
    }

    #[test]
    fn set_query_from_voice() {
        let mut p = make_palette();
        p.open();
        p.set_query("open browser");
        assert_eq!(p.query(), "open browser");
    }

    #[test]
    fn backspace() {
        let mut p = make_palette();
        p.open();
        p.set_query("test");
        p.backspace();
        assert_eq!(p.query(), "tes");
    }

    #[test]
    fn clear_query() {
        let mut p = make_palette();
        p.open();
        p.set_query("test");
        p.clear_query();
        assert!(p.query().is_empty());
        assert!(p.results().is_empty());
    }

    // ── Search tests ──

    #[test]
    fn search_finds_apps() {
        let mut p = make_palette();
        p.open();
        p.set_query("browser");

        let app_results: Vec<_> = p
            .results()
            .iter()
            .filter(|e| e.source == EntrySource::App)
            .collect();
        assert_eq!(app_results.len(), 1);
        assert!(app_results[0].title.contains("Browser"));
    }

    #[test]
    fn search_finds_settings() {
        let mut p = make_palette();
        p.open();
        p.set_query("wifi");

        let setting_results: Vec<_> = p
            .results()
            .iter()
            .filter(|e| e.source == EntrySource::Setting)
            .collect();
        assert_eq!(setting_results.len(), 1);
        assert!(setting_results[0].title.contains("WiFi"));
    }

    #[test]
    fn search_finds_navigation() {
        let mut p = make_palette();
        p.open();
        p.set_query("home");

        let nav_results: Vec<_> = p
            .results()
            .iter()
            .filter(|e| e.source == EntrySource::Navigation)
            .collect();
        assert_eq!(nav_results.len(), 1);
        assert!(nav_results[0].title.contains("home"));
    }

    #[test]
    fn universal_includes_ai_fallback() {
        let mut p = make_palette();
        p.open();
        p.set_query("something random");

        let ai_results: Vec<_> = p
            .results()
            .iter()
            .filter(|e| e.source == EntrySource::AiSuggestion)
            .collect();
        assert_eq!(ai_results.len(), 1);
    }

    #[test]
    fn universal_includes_command_fallback() {
        let mut p = make_palette();
        p.open();
        p.set_query("something");

        let cmd_results: Vec<_> = p
            .results()
            .iter()
            .filter(|e| e.source == EntrySource::Command)
            .collect();
        assert_eq!(cmd_results.len(), 1);
    }

    #[test]
    fn results_sorted_by_score() {
        let mut p = make_palette();
        p.open();
        p.set_query("browser");

        // App result should rank higher than command/AI
        let results = p.results();
        assert!(results.len() >= 2);
        for i in 1..results.len() {
            assert!(
                results[i - 1].score >= results[i].score,
                "Results not sorted by score: {} < {}",
                results[i - 1].score,
                results[i].score,
            );
        }
    }

    #[test]
    fn empty_query_no_results() {
        let mut p = make_palette();
        p.open();
        assert!(p.results().is_empty());
    }

    // ── Mode prefix tests ──

    #[test]
    fn app_prefix_filters() {
        let mut p = make_palette();
        p.open();
        p.set_query("app:terminal");

        let results = p.results();
        assert!(!results.is_empty());
        // Should find Terminal app
        assert!(results.iter().any(|e| e.source == EntrySource::App));
        // Should NOT include command or AI results
        assert!(!results.iter().any(|e| e.source == EntrySource::Command));
    }

    #[test]
    fn ask_prefix_goes_to_ai() {
        let mut p = make_palette();
        p.open();
        p.set_query("ask:what is the weather");

        let results = p.results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, EntrySource::AiSuggestion);
    }

    #[test]
    fn cmd_prefix_goes_to_command() {
        let mut p = make_palette();
        p.open();
        p.set_query("cmd:ls -la");

        let results = p.results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, EntrySource::Command);
    }

    #[test]
    fn file_prefix_creates_search() {
        let mut p = make_palette();
        p.open();
        p.set_query("file:readme");

        let results = p.results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, EntrySource::File);
    }

    #[test]
    fn set_prefix_filters_settings() {
        let mut p = make_palette();
        p.open();
        p.set_query("set:dark");

        let results = p.results();
        assert!(!results.is_empty());
        assert!(results.iter().all(|e| e.source == EntrySource::Setting));
    }

    // ── Mode setting tests ──

    #[test]
    fn set_mode_constrains_search() {
        let mut p = make_palette();
        p.open();
        p.set_mode(PaletteMode::Apps);
        p.set_query("terminal");

        // Only app results
        assert!(p.results().iter().all(|e| e.source == EntrySource::App));
    }

    #[test]
    fn mode_labels() {
        assert_eq!(PaletteMode::Universal.label(), "Search Everything");
        assert_eq!(PaletteMode::Apps.label(), "Apps");
        assert_eq!(PaletteMode::AiChat.label(), "Ask AI");
    }

    #[test]
    fn mode_prefixes() {
        assert_eq!(PaletteMode::Universal.prefix(), "");
        assert_eq!(PaletteMode::Apps.prefix(), "app: ");
        assert_eq!(PaletteMode::AiChat.prefix(), "ask: ");
    }

    #[test]
    fn mode_icons() {
        assert_eq!(PaletteMode::Universal.icon_hint(), "search");
        assert_eq!(PaletteMode::AiChat.icon_hint(), "sparkle");
    }

    #[test]
    fn all_modes_count() {
        assert_eq!(PaletteMode::ALL.len(), 6);
    }

    #[test]
    fn freeform_modes() {
        assert!(PaletteMode::Universal.accepts_freeform());
        assert!(PaletteMode::AiChat.accepts_freeform());
        assert!(!PaletteMode::Apps.accepts_freeform());
        assert!(!PaletteMode::Files.accepts_freeform());
    }

    // ── Selection tests ──

    #[test]
    fn select_next_prev() {
        let mut p = make_palette();
        p.open();
        p.set_query("browser"); // should produce multiple results

        assert_eq!(p.selected_index(), 0);
        p.select_next();
        assert_eq!(p.selected_index(), 1);
        p.select_prev();
        assert_eq!(p.selected_index(), 0);
    }

    #[test]
    fn select_prev_at_zero() {
        let mut p = make_palette();
        p.open();
        p.set_query("browser");
        p.select_prev();
        assert_eq!(p.selected_index(), 0);
    }

    #[test]
    fn selected_entry() {
        let mut p = make_palette();
        p.open();
        p.set_query("browser");

        let entry = p.selected_entry();
        assert!(entry.is_some());
    }

    #[test]
    fn selected_entry_empty() {
        let p = CommandPalette::new(FormFactor::Desktop);
        assert!(p.selected_entry().is_none());
    }

    // ── Execution tests ──

    #[test]
    fn execute_selected_returns_intent() {
        let mut p = make_palette();
        p.open();
        p.set_query("browser");

        let intent = p.execute_selected();
        assert!(intent.is_some());
        // Palette should close after execution
        assert!(!p.is_visible());
    }

    #[test]
    fn execute_selected_empty() {
        let mut p = CommandPalette::new(FormFactor::Desktop);
        assert!(p.execute_selected().is_none());
    }

    #[test]
    fn execute_with_ai() {
        let mut p = make_palette();
        let mut ai = AiPartner::new(FormFactor::Desktop);
        p.open();
        p.set_query("browser");

        let result = p.execute_with_ai(&mut ai);
        assert!(result.is_some());
        if let Some((intent, suggestions)) = &result {
            assert!(matches!(intent, Intent::LaunchApp { .. }));
            assert!(!suggestions.is_empty());
        }
    }

    // ── Visible results cap ──

    #[test]
    fn visible_results_capped() {
        let mut p = CommandPalette::new(FormFactor::Watch); // max 3
        p.register_apps(vec![
            ("App1".to_string(), "a1".to_string()),
            ("App2".to_string(), "a2".to_string()),
            ("App3".to_string(), "a3".to_string()),
            ("App4".to_string(), "a4".to_string()),
            ("App5".to_string(), "a5".to_string()),
        ]);
        p.open();
        p.set_query("app");
        // Even if there are more results, visible is capped
        assert!(p.visible_results().len() <= 3);
    }

    // ── Entry source labels ──

    #[test]
    fn entry_source_labels() {
        assert_eq!(EntrySource::App.label(), "App");
        assert_eq!(EntrySource::AiSuggestion.label(), "AI");
        assert_eq!(EntrySource::Navigation.label(), "Go to");
    }

    // ── Case insensitive search ──

    #[test]
    fn search_case_insensitive() {
        let mut p = make_palette();
        p.open();
        p.set_query("BROWSER");

        let app_results: Vec<_> = p
            .results()
            .iter()
            .filter(|e| e.source == EntrySource::App)
            .collect();
        assert_eq!(app_results.len(), 1);
    }

    // ── App ID search ──

    #[test]
    fn search_by_app_id() {
        let mut p = make_palette();
        p.open();
        p.set_query("com.nexcore.terminal");

        let app_results: Vec<_> = p
            .results()
            .iter()
            .filter(|e| e.source == EntrySource::App)
            .collect();
        assert_eq!(app_results.len(), 1);
    }

    // ── Exact match scores higher ──

    #[test]
    fn exact_match_highest_score() {
        let mut p = make_palette();
        p.open();
        p.set_mode(PaletteMode::Apps);
        p.set_query("browser");

        let results = p.results();
        assert!(!results.is_empty());
        // Exact match should have score 0.95
        assert!(results[0].score >= 0.90);
    }

    // ── Open resets state ──

    #[test]
    fn open_resets_state() {
        let mut p = make_palette();
        p.open();
        p.set_query("test");
        p.set_mode(PaletteMode::Apps);

        p.open();
        assert!(p.query().is_empty());
        assert_eq!(p.mode(), PaletteMode::Universal);
        assert!(p.results().is_empty());
    }
}
