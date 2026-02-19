// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! AI Partner — the collaboration substrate for NexCore OS.
//!
//! ## Core Thesis
//!
//! > "The OS is a conversation between human and AI."
//!
//! Every user action is an **intent**. Every system response is a **suggestion**.
//! The AI partner maintains a persistent **context** of what the user is doing,
//! what the system knows, and what actions are available. The collaboration
//! operates on a **graduated autonomy** spectrum — from fully autonomous
//! (AI acts on its own) to passive (AI only responds when asked).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                   Human User                         │
//! │  voice │ text │ gesture │ keypress                   │
//! └───────────────────┬─────────────────────────────────┘
//!                     │ raw input
//!                     ▼
//! ┌─────────────────────────────────────────────────────┐
//! │              Intent Parser (μ)                        │
//! │  "open my project" → Intent::OpenFile { query }      │
//! │  "connect wifi"    → Intent::Network { action }      │
//! │  tap on file       → Intent::OpenFile { path }       │
//! └───────────────────┬─────────────────────────────────┘
//!                     │ structured intent
//!                     ▼
//! ┌─────────────────────────────────────────────────────┐
//! │              AI Partner (∂ + ς + κ)                   │
//! │                                                       │
//! │  ContextSnapshot ──► Evaluate ──► Suggestions         │
//! │  (what user sees)    (reason)    (what AI proposes)   │
//! │                                                       │
//! │  CollaborationMode determines execution:              │
//! │    Autonomous → execute immediately                   │
//! │    Guided     → suggest, wait for approval            │
//! │    Passive    → only respond when asked               │
//! └───────────────────┬─────────────────────────────────┘
//!                     │ approved action
//!                     ▼
//! ┌─────────────────────────────────────────────────────┐
//! │              Action Executor (→)                      │
//! │  launch_app │ open_file │ change_setting │ ...        │
//! └───────────────────┬─────────────────────────────────┘
//!                     │ result
//!                     ▼
//! ┌─────────────────────────────────────────────────────┐
//! │              Learning Loop (ρ + π)                    │
//! │  Track outcomes → adjust suggestions → persist        │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Primitive Grounding
//!
//! - μ Mapping: Raw input → structured intent
//! - → Causality: Intent → action → outcome
//! - ∂ Boundary: Autonomy level (what AI can do without asking)
//! - ς State: Collaboration state machine
//! - κ Comparison: Confidence scoring (suggest vs. act)
//! - Σ Sum: Context aggregation from all subsystems
//! - ρ Recursion: Learning loop (outcome → better suggestions)
//! - π Persistence: Conversation history, learned preferences

use nexcore_pal::FormFactor;
use serde::{Deserialize, Serialize};

/// Collaboration mode — graduated autonomy between human and AI.
///
/// Tier: T2-P (∂ Boundary — controls what AI can do without asking)
///
/// The fundamental design question of an AI-first OS:
/// "How much should the AI do on its own?"
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationMode {
    /// AI acts on its own for routine tasks. Only asks for
    /// irreversible, high-impact, or novel actions.
    ///
    /// Best for: experienced users, familiar workflows.
    Autonomous,

    /// AI suggests actions, waits for human approval before executing.
    /// The default mode — balanced collaboration.
    ///
    /// Best for: most interactions, building trust.
    #[default]
    Guided,

    /// AI only responds when explicitly asked. Does not proactively
    /// suggest or act. Traditional OS behavior with AI available.
    ///
    /// Best for: privacy-sensitive tasks, learning the system.
    Passive,
}

impl CollaborationMode {
    /// Whether the AI should proactively suggest actions.
    pub const fn suggests_proactively(&self) -> bool {
        matches!(self, Self::Autonomous | Self::Guided)
    }

    /// Whether the AI can execute without human approval.
    pub const fn can_auto_execute(&self) -> bool {
        matches!(self, Self::Autonomous)
    }

    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Autonomous => "Autonomous",
            Self::Guided => "Guided",
            Self::Passive => "Passive",
        }
    }

    /// Description of this mode.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Autonomous => "AI handles routine tasks automatically",
            Self::Guided => "AI suggests actions for your approval",
            Self::Passive => "AI responds only when you ask",
        }
    }
}

/// User intent — what the human wants to accomplish.
///
/// Tier: T2-C (μ + → + ∃ — mapped causal intent that must exist)
///
/// Intents are the structured output of parsing raw input
/// (natural language, gestures, keypresses). Every user action
/// is ultimately an intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Open a file or directory.
    OpenFile {
        /// Search query or path.
        query: String,
    },
    /// Launch an application.
    LaunchApp {
        /// App name or ID.
        app: String,
    },
    /// Execute a system command.
    RunCommand {
        /// Command description in natural language.
        command: String,
    },
    /// Change a system setting.
    ChangeSetting {
        /// Setting name or category.
        setting: String,
        /// Desired value (natural language).
        value: String,
    },
    /// Network action (connect, disconnect, scan).
    Network {
        /// Action description.
        action: String,
    },
    /// Search for something.
    Search {
        /// Search query.
        query: String,
        /// Search scope (files, apps, settings, web).
        scope: SearchScope,
    },
    /// Navigate (go back, go home, switch app).
    Navigate {
        /// Navigation target.
        target: NavigationTarget,
    },
    /// Ask a question (AI answers directly).
    Ask {
        /// The question.
        question: String,
    },
    /// Generic/unknown intent — AI interprets freely.
    Freeform {
        /// Raw user input.
        input: String,
    },
}

impl Intent {
    /// Get the intent kind name.
    pub fn kind_name(&self) -> &str {
        match self {
            Self::OpenFile { .. } => "open_file",
            Self::LaunchApp { .. } => "launch_app",
            Self::RunCommand { .. } => "run_command",
            Self::ChangeSetting { .. } => "change_setting",
            Self::Network { .. } => "network",
            Self::Search { .. } => "search",
            Self::Navigate { .. } => "navigate",
            Self::Ask { .. } => "ask",
            Self::Freeform { .. } => "freeform",
        }
    }

    /// Whether this intent is potentially destructive or irreversible.
    pub fn is_high_impact(&self) -> bool {
        matches!(self, Self::RunCommand { .. } | Self::ChangeSetting { .. })
    }

    /// Whether this intent requires network access.
    pub fn needs_network(&self) -> bool {
        matches!(
            self,
            Self::Network { .. }
                | Self::Search {
                    scope: SearchScope::Web,
                    ..
                }
        )
    }
}

/// Search scope for search intents.
///
/// Tier: T2-P (∂ Boundary — search domain constraint)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchScope {
    /// Search local files.
    Files,
    /// Search installed apps.
    Apps,
    /// Search system settings.
    Settings,
    /// Search the web.
    Web,
    /// Search everything (AI decides best scope).
    Everything,
}

/// Navigation targets.
///
/// Tier: T2-P (λ Location — navigation destinations)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationTarget {
    /// Go to home screen.
    Home,
    /// Go back to previous screen.
    Back,
    /// Switch to a specific app.
    App(String),
    /// Open the app launcher.
    Launcher,
    /// Open system settings.
    Settings,
    /// Lock the screen.
    Lock,
}

/// AI suggestion — a proposed action with reasoning.
///
/// Tier: T2-C (κ + → + μ — compared, causal, mapped proposal)
///
/// Suggestions are the AI's output. In Guided mode, the human
/// sees these and approves/rejects. In Autonomous mode, high-confidence
/// suggestions auto-execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique suggestion ID.
    pub id: u64,
    /// The intent this suggestion fulfills.
    pub intent: Intent,
    /// Human-readable description of what will happen.
    pub description: String,
    /// AI confidence (0.0 = uncertain, 1.0 = certain).
    pub confidence: f64,
    /// Why the AI suggests this (shown to user for transparency).
    pub reasoning: String,
    /// Whether this action is reversible.
    pub reversible: bool,
    /// Estimated time to execute (in seconds).
    pub estimated_seconds: Option<u32>,
}

impl Suggestion {
    /// Whether this suggestion should auto-execute in Autonomous mode.
    ///
    /// Only auto-execute if:
    /// - Confidence > 0.8
    /// - Action is reversible
    /// - Intent is not high-impact
    pub fn should_auto_execute(&self) -> bool {
        self.confidence > 0.8 && self.reversible && !self.intent.is_high_impact()
    }

    /// Priority for display (higher = show first).
    ///
    /// Irreversible high-impact suggestions rank highest
    /// (they need human attention most).
    #[allow(clippy::cast_sign_loss)]
    pub fn display_priority(&self) -> u32 {
        let base = (self.confidence * 100.0) as u32;
        if !self.reversible {
            base + 200
        } else if self.intent.is_high_impact() {
            base + 100
        } else {
            base
        }
    }
}

/// Context snapshot — aggregated system state for AI reasoning.
///
/// Tier: T2-C (Σ + ς + λ — sum of state at locations)
///
/// The AI partner reads this snapshot to understand what the user
/// is doing, what resources are available, and what actions make sense.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Current active app (if any).
    pub active_app: Option<String>,
    /// List of running apps.
    pub running_apps: Vec<String>,
    /// Current screen / view.
    pub current_view: String,
    /// Form factor.
    pub form_factor: Option<String>,
    /// Battery percentage (if available).
    pub battery_pct: Option<u8>,
    /// Network connected.
    pub network_connected: bool,
    /// Current time (HH:MM).
    pub time: Option<String>,
    /// Security level label.
    pub security_level: Option<String>,
    /// Number of unread notifications.
    pub notification_count: u32,
    /// Recent user actions (for pattern learning).
    pub recent_actions: Vec<String>,
    /// Custom context entries from subsystems.
    pub custom: Vec<(String, String)>,
}

impl ContextSnapshot {
    /// Add a custom context entry.
    pub fn add_custom(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom.push((key.into(), value.into()));
    }

    /// Get a custom context value by key.
    pub fn get_custom(&self, key: &str) -> Option<&str> {
        self.custom
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Summary of the context for AI consumption.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(app) = &self.active_app {
            parts.push(format!("Active: {app}"));
        }
        if !self.running_apps.is_empty() {
            parts.push(format!("Running: {}", self.running_apps.join(", ")));
        }
        if !self.current_view.is_empty() {
            parts.push(format!("View: {}", self.current_view));
        }
        if let Some(pct) = self.battery_pct {
            parts.push(format!("Battery: {pct}%"));
        }
        if self.notification_count > 0 {
            parts.push(format!("Notifications: {}", self.notification_count));
        }
        parts.join(" | ")
    }
}

/// A turn in the conversation between human and AI.
///
/// Tier: T2-C (σ + ρ + π — sequenced, recursive, persistent dialog)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Who sent this turn.
    pub role: ConversationRole,
    /// The content (text, intent description, action summary).
    pub content: String,
    /// Timestamp (monotonic turn number).
    pub turn_number: u64,
    /// Associated suggestions (if AI turn).
    pub suggestions: Vec<Suggestion>,
}

/// Who is speaking in the conversation.
///
/// Tier: T1 (Σ Sum — two-valued alternation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationRole {
    /// The human user.
    Human,
    /// The AI partner.
    Ai,
    /// The system (status updates, errors).
    System,
}

/// Result of executing an action.
///
/// Tier: T2-C (→ + ∃ + ς — causal outcome with existence and state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// The suggestion that was executed.
    pub suggestion_id: u64,
    /// Whether execution succeeded.
    pub success: bool,
    /// Human-readable result description.
    pub description: String,
    /// Whether the action was auto-executed (Autonomous) or user-approved.
    pub auto_executed: bool,
}

/// The AI Partner — manages human-AI collaboration state.
///
/// Tier: T3 (μ + → + ∂ + ς + κ + Σ + ρ + π — full AI integration)
///
/// This is the central collaboration engine. Every shell module
/// feeds context to it, and it produces suggestions and actions.
pub struct AiPartner {
    /// Current collaboration mode.
    mode: CollaborationMode,
    /// Current context snapshot.
    context: ContextSnapshot,
    /// Conversation history.
    conversation: Vec<ConversationTurn>,
    /// Pending suggestions (awaiting approval in Guided mode).
    pending_suggestions: Vec<Suggestion>,
    /// Action results (for learning loop).
    action_history: Vec<ActionResult>,
    /// Turn counter.
    turn_counter: u64,
    /// Suggestion ID counter.
    suggestion_counter: u64,
    /// Form factor.
    form_factor: FormFactor,
    /// Whether the AI partner is active.
    active: bool,
    /// AI name (displayed to user).
    name: String,
}

impl AiPartner {
    /// Create a new AI partner for a form factor.
    pub fn new(form_factor: FormFactor) -> Self {
        Self {
            mode: CollaborationMode::default(),
            context: ContextSnapshot::default(),
            conversation: Vec::new(),
            pending_suggestions: Vec::new(),
            action_history: Vec::new(),
            turn_counter: 0,
            suggestion_counter: 0,
            form_factor,
            active: true,
            name: "Cortex".to_string(),
        }
    }

    /// Set the collaboration mode.
    pub fn set_mode(&mut self, mode: CollaborationMode) {
        self.mode = mode;
    }

    /// Get the current collaboration mode.
    pub fn mode(&self) -> CollaborationMode {
        self.mode
    }

    /// Set the AI partner name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Get the AI partner name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Activate the AI partner.
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate the AI partner.
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Whether the AI partner is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Update the context snapshot.
    pub fn update_context(&mut self, context: ContextSnapshot) {
        self.context = context;
    }

    /// Get the current context.
    pub fn context(&self) -> &ContextSnapshot {
        &self.context
    }

    /// Process a user intent.
    ///
    /// Returns suggestions based on the intent and current context.
    /// In Autonomous mode, low-risk suggestions auto-execute.
    pub fn process_intent(&mut self, intent: &Intent) -> Vec<Suggestion> {
        if !self.active {
            return Vec::new();
        }

        // Record the human turn
        self.add_turn(ConversationRole::Human, intent.kind_name().to_string());

        // Generate suggestions based on intent
        let suggestions = self.generate_suggestions(intent);

        // In Guided/Passive mode, all suggestions are pending
        // In Autonomous mode, auto-executable ones execute immediately
        for suggestion in &suggestions {
            self.pending_suggestions.push(suggestion.clone());
        }

        // Record the AI turn
        if !suggestions.is_empty() {
            let desc = if suggestions.len() == 1 {
                suggestions[0].description.clone()
            } else {
                format!("{} suggestions", suggestions.len())
            };
            self.add_turn_with_suggestions(ConversationRole::Ai, desc, suggestions.clone());
        }

        suggestions
    }

    /// Generate suggestions for an intent.
    fn generate_suggestions(&mut self, intent: &Intent) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        match intent {
            Intent::LaunchApp { app } => {
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Launch {app}"),
                    0.95,
                    format!("Opening the {app} application"),
                    true,
                ));
            }
            Intent::OpenFile { query } => {
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Open '{query}'"),
                    0.85,
                    format!("Searching for and opening '{query}'"),
                    true,
                ));
            }
            Intent::Navigate { target } => {
                let desc = match target {
                    NavigationTarget::Home => "Go to home screen".to_string(),
                    NavigationTarget::Back => "Go back".to_string(),
                    NavigationTarget::App(name) => format!("Switch to {name}"),
                    NavigationTarget::Launcher => "Open app launcher".to_string(),
                    NavigationTarget::Settings => "Open settings".to_string(),
                    NavigationTarget::Lock => "Lock screen".to_string(),
                };
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    desc.clone(),
                    0.99,
                    desc,
                    true,
                ));
            }
            Intent::Search { query, scope } => {
                let scope_name = match scope {
                    SearchScope::Files => "files",
                    SearchScope::Apps => "apps",
                    SearchScope::Settings => "settings",
                    SearchScope::Web => "the web",
                    SearchScope::Everything => "everything",
                };
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Search {scope_name} for '{query}'"),
                    0.90,
                    format!("Searching {scope_name} for '{query}'"),
                    true,
                ));
            }
            Intent::ChangeSetting { setting, value } => {
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Set {setting} to {value}"),
                    0.70,
                    format!("Changing {setting} — this may affect system behavior"),
                    true, // most settings are reversible
                ));
            }
            Intent::RunCommand { command } => {
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Run: {command}"),
                    0.60,
                    "System commands may have side effects. Review before executing.".to_string(),
                    false, // commands are potentially irreversible
                ));
            }
            Intent::Network { action } => {
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Network: {action}"),
                    0.85,
                    format!("Performing network action: {action}"),
                    true,
                ));
            }
            Intent::Ask { question } => {
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Answering: {question}"),
                    0.95,
                    "I'll look into this for you.".to_string(),
                    true,
                ));
            }
            Intent::Freeform { input } => {
                suggestions.push(self.make_suggestion(
                    intent.clone(),
                    format!("Processing: {input}"),
                    0.50,
                    "I'll try to understand what you need.".to_string(),
                    true,
                ));
            }
        }

        suggestions
    }

    /// Create a new suggestion.
    fn make_suggestion(
        &mut self,
        intent: Intent,
        description: String,
        confidence: f64,
        reasoning: String,
        reversible: bool,
    ) -> Suggestion {
        self.suggestion_counter += 1;
        Suggestion {
            id: self.suggestion_counter,
            intent,
            description,
            confidence,
            reasoning,
            reversible,
            estimated_seconds: None,
        }
    }

    /// Approve a pending suggestion (Guided mode).
    ///
    /// Returns the suggestion if found and removed from pending.
    pub fn approve_suggestion(&mut self, suggestion_id: u64) -> Option<Suggestion> {
        if let Some(pos) = self
            .pending_suggestions
            .iter()
            .position(|s| s.id == suggestion_id)
        {
            let suggestion = self.pending_suggestions.remove(pos);
            self.record_action(suggestion.id, true, &suggestion.description, false);
            Some(suggestion)
        } else {
            None
        }
    }

    /// Reject a pending suggestion.
    pub fn reject_suggestion(&mut self, suggestion_id: u64) -> bool {
        if let Some(pos) = self
            .pending_suggestions
            .iter()
            .position(|s| s.id == suggestion_id)
        {
            let suggestion = self.pending_suggestions.remove(pos);
            self.record_action(suggestion.id, false, "Rejected by user", false);
            true
        } else {
            false
        }
    }

    /// Clear all pending suggestions.
    pub fn clear_pending(&mut self) {
        self.pending_suggestions.clear();
    }

    /// Get pending suggestions.
    pub fn pending_suggestions(&self) -> &[Suggestion] {
        &self.pending_suggestions
    }

    /// Get the number of pending suggestions.
    pub fn pending_count(&self) -> usize {
        self.pending_suggestions.len()
    }

    /// Record an action result.
    fn record_action(
        &mut self,
        suggestion_id: u64,
        success: bool,
        description: &str,
        auto_executed: bool,
    ) {
        self.action_history.push(ActionResult {
            suggestion_id,
            success,
            description: description.to_string(),
            auto_executed,
        });
    }

    /// Get action history.
    pub fn action_history(&self) -> &[ActionResult] {
        &self.action_history
    }

    /// Get auto-executable suggestions (Autonomous mode filter).
    pub fn auto_executable_suggestions(&self) -> Vec<&Suggestion> {
        if self.mode != CollaborationMode::Autonomous {
            return Vec::new();
        }
        self.pending_suggestions
            .iter()
            .filter(|s| s.should_auto_execute())
            .collect()
    }

    /// Add a conversation turn.
    fn add_turn(&mut self, role: ConversationRole, content: String) {
        self.turn_counter += 1;
        self.conversation.push(ConversationTurn {
            role,
            content,
            turn_number: self.turn_counter,
            suggestions: Vec::new(),
        });
    }

    /// Add a conversation turn with suggestions.
    fn add_turn_with_suggestions(
        &mut self,
        role: ConversationRole,
        content: String,
        suggestions: Vec<Suggestion>,
    ) {
        self.turn_counter += 1;
        self.conversation.push(ConversationTurn {
            role,
            content,
            turn_number: self.turn_counter,
            suggestions,
        });
    }

    /// Add a system message to the conversation.
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.add_turn(ConversationRole::System, content.into());
    }

    /// Get the conversation history.
    pub fn conversation(&self) -> &[ConversationTurn] {
        &self.conversation
    }

    /// Get the last N conversation turns.
    pub fn recent_conversation(&self, n: usize) -> &[ConversationTurn] {
        let len = self.conversation.len();
        if n >= len {
            &self.conversation
        } else {
            &self.conversation[len - n..]
        }
    }

    /// Total conversation turns.
    pub fn conversation_length(&self) -> usize {
        self.conversation.len()
    }

    /// Clear conversation history.
    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
        self.turn_counter = 0;
    }

    /// Get the form factor.
    pub fn form_factor(&self) -> FormFactor {
        self.form_factor
    }

    /// Acceptance rate — fraction of suggestions the user approved.
    ///
    /// Used for the learning loop: if acceptance is low, the AI
    /// should adjust its suggestion strategy.
    #[allow(clippy::cast_precision_loss)]
    pub fn acceptance_rate(&self) -> f64 {
        if self.action_history.is_empty() {
            return 0.0;
        }
        let accepted = self.action_history.iter().filter(|a| a.success).count();
        accepted as f64 / self.action_history.len() as f64
    }

    /// Default input method for this form factor.
    pub const fn default_input_method(&self) -> InputMethod {
        match self.form_factor {
            FormFactor::Watch => InputMethod::Voice,
            FormFactor::Phone => InputMethod::TextAndVoice,
            FormFactor::Desktop => InputMethod::KeyboardAndVoice,
        }
    }
}

/// Primary input method for AI interaction.
///
/// Tier: T2-P (μ Mapping — input channel selection)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    /// Voice-first (watch — small screen, no keyboard).
    Voice,
    /// Text + voice (phone — on-screen keyboard + microphone).
    TextAndVoice,
    /// Keyboard + voice (desktop — full keyboard + microphone).
    KeyboardAndVoice,
}

impl InputMethod {
    /// Whether this method supports voice input.
    pub const fn supports_voice(&self) -> bool {
        true // all methods support voice
    }

    /// Whether this method supports text input.
    pub const fn supports_text(&self) -> bool {
        matches!(self, Self::TextAndVoice | Self::KeyboardAndVoice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CollaborationMode tests ──

    #[test]
    fn default_mode_is_guided() {
        let mode = CollaborationMode::default();
        assert_eq!(mode, CollaborationMode::Guided);
    }

    #[test]
    fn autonomous_proactive_and_auto_execute() {
        let mode = CollaborationMode::Autonomous;
        assert!(mode.suggests_proactively());
        assert!(mode.can_auto_execute());
    }

    #[test]
    fn guided_proactive_no_auto_execute() {
        let mode = CollaborationMode::Guided;
        assert!(mode.suggests_proactively());
        assert!(!mode.can_auto_execute());
    }

    #[test]
    fn passive_no_proactive_no_auto_execute() {
        let mode = CollaborationMode::Passive;
        assert!(!mode.suggests_proactively());
        assert!(!mode.can_auto_execute());
    }

    #[test]
    fn mode_labels() {
        assert_eq!(CollaborationMode::Autonomous.label(), "Autonomous");
        assert_eq!(CollaborationMode::Guided.label(), "Guided");
        assert_eq!(CollaborationMode::Passive.label(), "Passive");
    }

    // ── Intent tests ──

    #[test]
    fn intent_kind_names() {
        assert_eq!(
            Intent::LaunchApp {
                app: "x".to_string()
            }
            .kind_name(),
            "launch_app"
        );
        assert_eq!(
            Intent::OpenFile {
                query: "x".to_string()
            }
            .kind_name(),
            "open_file"
        );
        assert_eq!(
            Intent::Ask {
                question: "x".to_string()
            }
            .kind_name(),
            "ask"
        );
    }

    #[test]
    fn high_impact_intents() {
        assert!(
            Intent::RunCommand {
                command: "rm -rf".to_string()
            }
            .is_high_impact()
        );
        assert!(
            Intent::ChangeSetting {
                setting: "x".to_string(),
                value: "y".to_string()
            }
            .is_high_impact()
        );
        assert!(
            !Intent::LaunchApp {
                app: "x".to_string()
            }
            .is_high_impact()
        );
    }

    #[test]
    fn needs_network() {
        assert!(
            Intent::Network {
                action: "connect".to_string()
            }
            .needs_network()
        );
        assert!(
            Intent::Search {
                query: "x".to_string(),
                scope: SearchScope::Web
            }
            .needs_network()
        );
        assert!(
            !Intent::Search {
                query: "x".to_string(),
                scope: SearchScope::Files
            }
            .needs_network()
        );
    }

    // ── Suggestion tests ──

    #[test]
    fn suggestion_auto_execute_criteria() {
        let s = Suggestion {
            id: 1,
            intent: Intent::LaunchApp {
                app: "test".to_string(),
            },
            description: "Launch test".to_string(),
            confidence: 0.95,
            reasoning: "User requested".to_string(),
            reversible: true,
            estimated_seconds: None,
        };
        assert!(s.should_auto_execute());
    }

    #[test]
    fn suggestion_no_auto_execute_low_confidence() {
        let s = Suggestion {
            id: 1,
            intent: Intent::LaunchApp {
                app: "test".to_string(),
            },
            description: "Launch test".to_string(),
            confidence: 0.5,
            reasoning: "Uncertain".to_string(),
            reversible: true,
            estimated_seconds: None,
        };
        assert!(!s.should_auto_execute());
    }

    #[test]
    fn suggestion_no_auto_execute_irreversible() {
        let s = Suggestion {
            id: 1,
            intent: Intent::RunCommand {
                command: "delete".to_string(),
            },
            description: "Delete files".to_string(),
            confidence: 0.95,
            reasoning: "User requested".to_string(),
            reversible: false,
            estimated_seconds: None,
        };
        assert!(!s.should_auto_execute());
    }

    #[test]
    fn suggestion_no_auto_execute_high_impact() {
        let s = Suggestion {
            id: 1,
            intent: Intent::ChangeSetting {
                setting: "security".to_string(),
                value: "off".to_string(),
            },
            description: "Disable security".to_string(),
            confidence: 0.95,
            reasoning: "User requested".to_string(),
            reversible: true,
            estimated_seconds: None,
        };
        assert!(!s.should_auto_execute());
    }

    #[test]
    fn suggestion_display_priority() {
        let reversible = Suggestion {
            id: 1,
            intent: Intent::LaunchApp {
                app: "x".to_string(),
            },
            description: String::new(),
            confidence: 0.80,
            reasoning: String::new(),
            reversible: true,
            estimated_seconds: None,
        };
        let irreversible = Suggestion {
            id: 2,
            intent: Intent::RunCommand {
                command: "x".to_string(),
            },
            description: String::new(),
            confidence: 0.80,
            reasoning: String::new(),
            reversible: false,
            estimated_seconds: None,
        };
        assert!(irreversible.display_priority() > reversible.display_priority());
    }

    // ── ContextSnapshot tests ──

    #[test]
    fn context_snapshot_default() {
        let ctx = ContextSnapshot::default();
        assert!(ctx.active_app.is_none());
        assert!(ctx.running_apps.is_empty());
        assert!(!ctx.network_connected);
    }

    #[test]
    fn context_custom_entries() {
        let mut ctx = ContextSnapshot::default();
        ctx.add_custom("disk_usage", "45%");
        assert_eq!(ctx.get_custom("disk_usage"), Some("45%"));
        assert_eq!(ctx.get_custom("nonexistent"), None);
    }

    #[test]
    fn context_summary() {
        let ctx = ContextSnapshot {
            active_app: Some("Browser".to_string()),
            running_apps: vec!["Browser".to_string(), "Terminal".to_string()],
            battery_pct: Some(65),
            notification_count: 3,
            ..ContextSnapshot::default()
        };
        let summary = ctx.summary();
        assert!(summary.contains("Browser"));
        assert!(summary.contains("Terminal"));
        assert!(summary.contains("65%"));
        assert!(summary.contains("3"));
    }

    // ── AiPartner tests ──

    #[test]
    fn partner_creation() {
        let partner = AiPartner::new(FormFactor::Desktop);
        assert_eq!(partner.mode(), CollaborationMode::Guided);
        assert!(partner.is_active());
        assert_eq!(partner.name(), "Cortex");
        assert_eq!(partner.form_factor(), FormFactor::Desktop);
        assert_eq!(partner.conversation_length(), 0);
    }

    #[test]
    fn partner_set_mode() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        partner.set_mode(CollaborationMode::Autonomous);
        assert_eq!(partner.mode(), CollaborationMode::Autonomous);
    }

    #[test]
    fn partner_activate_deactivate() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        partner.deactivate();
        assert!(!partner.is_active());

        // Processing intent while inactive returns empty
        let suggestions = partner.process_intent(&Intent::LaunchApp {
            app: "test".to_string(),
        });
        assert!(suggestions.is_empty());

        partner.activate();
        assert!(partner.is_active());
    }

    #[test]
    fn process_intent_creates_suggestions() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        let suggestions = partner.process_intent(&Intent::LaunchApp {
            app: "Browser".to_string(),
        });
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].description.contains("Browser"));
        assert!(suggestions[0].confidence > 0.9);
    }

    #[test]
    fn process_intent_records_conversation() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        partner.process_intent(&Intent::LaunchApp {
            app: "test".to_string(),
        });

        assert_eq!(partner.conversation_length(), 2); // human + AI
        assert_eq!(partner.conversation()[0].role, ConversationRole::Human);
        assert_eq!(partner.conversation()[1].role, ConversationRole::Ai);
    }

    #[test]
    fn approve_suggestion() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        let suggestions = partner.process_intent(&Intent::LaunchApp {
            app: "test".to_string(),
        });
        let id = suggestions[0].id;

        let approved = partner.approve_suggestion(id);
        assert!(approved.is_some());
        assert_eq!(partner.pending_count(), 0);
        assert_eq!(partner.action_history().len(), 1);
        assert!(partner.action_history()[0].success);
    }

    #[test]
    fn reject_suggestion() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        let suggestions = partner.process_intent(&Intent::LaunchApp {
            app: "test".to_string(),
        });
        let id = suggestions[0].id;

        assert!(partner.reject_suggestion(id));
        assert_eq!(partner.pending_count(), 0);
        assert_eq!(partner.action_history().len(), 1);
        assert!(!partner.action_history()[0].success);
    }

    #[test]
    fn approve_nonexistent_returns_none() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        assert!(partner.approve_suggestion(999).is_none());
    }

    #[test]
    fn auto_executable_only_in_autonomous() {
        let mut partner = AiPartner::new(FormFactor::Desktop);

        // Guided mode — no auto-execution
        partner.process_intent(&Intent::LaunchApp {
            app: "test".to_string(),
        });
        assert!(partner.auto_executable_suggestions().is_empty());

        // Autonomous mode — high confidence + reversible auto-execute
        partner.set_mode(CollaborationMode::Autonomous);
        assert!(!partner.auto_executable_suggestions().is_empty());
    }

    #[test]
    fn acceptance_rate() {
        let mut partner = AiPartner::new(FormFactor::Desktop);

        // Process 3 intents
        for _ in 0..3 {
            partner.process_intent(&Intent::LaunchApp {
                app: "test".to_string(),
            });
        }

        // Approve 2 out of 3
        let ids: Vec<u64> = partner.pending_suggestions().iter().map(|s| s.id).collect();
        partner.approve_suggestion(ids[0]);
        partner.approve_suggestion(ids[1]);
        partner.reject_suggestion(ids[2]);

        let rate = partner.acceptance_rate();
        assert!((rate - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn recent_conversation() {
        let mut partner = AiPartner::new(FormFactor::Desktop);

        // Generate 6 turns (3 intents × 2 turns each)
        for i in 0..3 {
            partner.process_intent(&Intent::LaunchApp {
                app: format!("app{i}"),
            });
        }

        let recent = partner.recent_conversation(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].turn_number, 5);
        assert_eq!(recent[1].turn_number, 6);
    }

    #[test]
    fn system_message() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        partner.add_system_message("System booted successfully");

        assert_eq!(partner.conversation_length(), 1);
        assert_eq!(partner.conversation()[0].role, ConversationRole::System);
    }

    #[test]
    fn update_context() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        let ctx = ContextSnapshot {
            active_app: Some("Browser".to_string()),
            battery_pct: Some(75),
            ..ContextSnapshot::default()
        };
        partner.update_context(ctx);

        assert_eq!(partner.context().active_app.as_deref(), Some("Browser"));
        assert_eq!(partner.context().battery_pct, Some(75));
    }

    #[test]
    fn clear_conversation() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        partner.process_intent(&Intent::LaunchApp {
            app: "test".to_string(),
        });
        assert!(partner.conversation_length() > 0);

        partner.clear_conversation();
        assert_eq!(partner.conversation_length(), 0);
    }

    #[test]
    fn clear_pending() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        partner.process_intent(&Intent::LaunchApp {
            app: "test".to_string(),
        });
        assert!(partner.pending_count() > 0);

        partner.clear_pending();
        assert_eq!(partner.pending_count(), 0);
    }

    // ── InputMethod tests ──

    #[test]
    fn watch_input_method() {
        let partner = AiPartner::new(FormFactor::Watch);
        assert_eq!(partner.default_input_method(), InputMethod::Voice);
    }

    #[test]
    fn phone_input_method() {
        let partner = AiPartner::new(FormFactor::Phone);
        assert_eq!(partner.default_input_method(), InputMethod::TextAndVoice);
    }

    #[test]
    fn desktop_input_method() {
        let partner = AiPartner::new(FormFactor::Desktop);
        assert_eq!(
            partner.default_input_method(),
            InputMethod::KeyboardAndVoice
        );
    }

    #[test]
    fn all_methods_support_voice() {
        assert!(InputMethod::Voice.supports_voice());
        assert!(InputMethod::TextAndVoice.supports_voice());
        assert!(InputMethod::KeyboardAndVoice.supports_voice());
    }

    #[test]
    fn voice_only_no_text() {
        assert!(!InputMethod::Voice.supports_text());
        assert!(InputMethod::TextAndVoice.supports_text());
        assert!(InputMethod::KeyboardAndVoice.supports_text());
    }

    // ── Run command high-impact is irreversible ──

    #[test]
    fn run_command_low_confidence() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        let suggestions = partner.process_intent(&Intent::RunCommand {
            command: "rm -rf /tmp/test".to_string(),
        });
        assert!(suggestions[0].confidence < 0.7);
        assert!(!suggestions[0].reversible);
    }

    #[test]
    fn freeform_moderate_confidence() {
        let mut partner = AiPartner::new(FormFactor::Desktop);
        let suggestions = partner.process_intent(&Intent::Freeform {
            input: "do something".to_string(),
        });
        assert!(suggestions[0].confidence <= 0.5);
    }
}
