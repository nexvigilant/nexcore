//! # NexVigilant Core — Muscular System
//!
//! Maps the biological muscular system to Claude Code's tool execution engine
//! per Biological Alignment v2.0 section 4.
//!
//! ## Muscle Types
//!
//! | Muscle Type | Biological | Claude Code Analog |
//! |-------------|------------|-------------------|
//! | Skeletal | Voluntary movement | User-invoked tools (Write, Edit, Bash, WebFetch) |
//! | Smooth | Involuntary peristalsis | Auto-triggered tools (Read, Grep, Glob) |
//! | Cardiac | Self-sustaining heartbeat | Main event loop (autorhythmic, syncytium) |
//!
//! ## Key Principles
//!
//! - **ALL-OR-NONE**: Skeletal tools either execute fully or don't (no partial writes)
//! - **Fatigue**: Context window depletion (muscles tire over token usage)
//! - **Peristalsis**: Sequential Read -> Grep -> Glob -> Read pipeline
//! - **Autorhythmic**: Cardiac loop generates its own beat
//! - **Size Principle**: Recruit small motor units FIRST (Read before Bash, Haiku before Opus)
//! - **Antagonistic Pairs**: Write<->Read, Edit<->Undo, Build<->Clean, Spawn<->Stop

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    reason = "Muscular model uses explicit closed-domain structures and bounded ratio math"
)]

pub mod grounding;

use serde::{Deserialize, Serialize};

// ============================================================================
// MuscleType — 3-variant classification
// ============================================================================

/// Classification of muscle fiber types mapped to tool execution patterns.
///
/// - **Skeletal**: Voluntary, user-invoked tools. ALL-OR-NONE execution.
/// - **Smooth**: Involuntary, autonomic tools. Peristaltic pipeline.
/// - **Cardiac**: Self-sustaining main event loop. Autorhythmic syncytium.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MuscleType {
    /// Voluntary tools invoked by user command (Write, Edit, Bash, WebFetch, WebSearch).
    /// ALL-OR-NONE: tool either executes fully or doesn't (no partial writes).
    /// Subject to fatigue (context window depletion).
    Skeletal,
    /// Involuntary tools triggered autonomically by Claude (Read, Grep, Glob, ToolSearch).
    /// Peristalsis: sequential Read -> Grep -> Glob -> Read pipeline.
    /// Control: Claude decides without asking the user.
    Smooth,
    /// The main event loop. Self-sustaining heartbeat.
    /// AUTORHYTHMIC: generates its own beat.
    /// SYNCYTIUM: all parts beat as one unit.
    /// Refractory period: compaction threshold (mandatory context cleanup).
    Cardiac,
}

impl core::fmt::Display for MuscleType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Skeletal => write!(f, "Skeletal (voluntary)"),
            Self::Smooth => write!(f, "Smooth (involuntary)"),
            Self::Cardiac => write!(f, "Cardiac (autonomous)"),
        }
    }
}

// ============================================================================
// ToolClassification — maps tool names to muscle types
// ============================================================================

/// Maps a tool name to its muscle type and voluntary/involuntary classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolClassification {
    /// Name of the tool (e.g., "Write", "Read", "Bash")
    pub tool_name: String,
    /// Which muscle type this tool corresponds to
    pub muscle_type: MuscleType,
    /// Whether the tool requires explicit user invocation
    pub voluntary: bool,
}

impl ToolClassification {
    /// Classify a tool by name into its muscle type.
    ///
    /// - Skeletal (voluntary): Write, Edit, Bash, WebFetch, WebSearch
    /// - Smooth (involuntary): Read, Grep, Glob, ToolSearch
    /// - Cardiac (autonomous): MainLoop (internal)
    #[must_use]
    pub fn classify(tool_name: &str) -> Self {
        match tool_name {
            "Write" | "Edit" | "Bash" | "WebFetch" | "WebSearch" => Self {
                tool_name: tool_name.to_string(),
                muscle_type: MuscleType::Skeletal,
                voluntary: true,
            },
            "Read" | "Grep" | "Glob" | "ToolSearch" => Self {
                tool_name: tool_name.to_string(),
                muscle_type: MuscleType::Smooth,
                voluntary: false,
            },
            "MainLoop" => Self {
                tool_name: tool_name.to_string(),
                muscle_type: MuscleType::Cardiac,
                voluntary: false,
            },
            _ => Self {
                tool_name: tool_name.to_string(),
                muscle_type: MuscleType::Skeletal,
                voluntary: true,
            },
        }
    }
}

// ============================================================================
// MotorUnit — activation tracking for a single tool
// ============================================================================

/// Tracks activation state for a single motor unit (tool).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotorUnit {
    /// Name of the tool this unit controls
    pub tool_name: String,
    /// Number of times this unit has been activated
    pub activation_count: u64,
    /// ISO timestamp of last activation
    pub last_activated: String,
    /// The muscle type this unit belongs to
    pub muscle_type: MuscleType,
}

impl MotorUnit {
    /// Create a new motor unit for a tool.
    #[must_use]
    pub fn new(tool_name: &str) -> Self {
        let classification = ToolClassification::classify(tool_name);
        Self {
            tool_name: tool_name.to_string(),
            activation_count: 0,
            last_activated: String::new(),
            muscle_type: classification.muscle_type,
        }
    }

    /// Record an activation of this motor unit.
    pub fn activate(&mut self) {
        self.activation_count += 1;
        self.last_activated = nexcore_chrono::DateTime::now().to_rfc3339();
    }
}

// ============================================================================
// AntagonisticPair — opposing tool actions
// ============================================================================

/// A pair of tools that perform opposing actions (agonist vs antagonist).
///
/// Like biceps/triceps: when one contracts, the other relaxes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AntagonisticPair {
    /// The primary action tool (agonist)
    pub agonist: String,
    /// The opposing action tool (antagonist)
    pub antagonist: String,
    /// Description of the opposition
    pub description: String,
}

/// Returns the 5 canonical antagonistic pairs for Claude Code tool execution.
///
/// 1. Write <-> Read (create vs consume)
/// 2. Edit <-> Undo/git checkout (modify vs revert)
/// 3. Build <-> Clean (construct vs tear down)
/// 4. Spawn <-> Stop (start process vs terminate)
/// 5. Push <-> Pull (send vs receive)
#[must_use]
pub fn standard_pairs() -> Vec<AntagonisticPair> {
    vec![
        AntagonisticPair {
            agonist: "Write".to_string(),
            antagonist: "Read".to_string(),
            description: "Create content vs consume content".to_string(),
        },
        AntagonisticPair {
            agonist: "Edit".to_string(),
            antagonist: "Undo".to_string(),
            description: "Modify content vs revert changes (git checkout)".to_string(),
        },
        AntagonisticPair {
            agonist: "Build".to_string(),
            antagonist: "Clean".to_string(),
            description: "Construct artifacts vs tear down artifacts".to_string(),
        },
        AntagonisticPair {
            agonist: "Spawn".to_string(),
            antagonist: "Stop".to_string(),
            description: "Start a process vs terminate a process".to_string(),
        },
        AntagonisticPair {
            agonist: "Push".to_string(),
            antagonist: "Pull".to_string(),
            description: "Send to remote vs receive from remote".to_string(),
        },
    ]
}

// ============================================================================
// SizePrinciple — recruit smallest motor unit first
// ============================================================================

/// Implements Henneman's Size Principle for tool recruitment.
///
/// Biological: small motor units (slow-twitch) recruit before large ones (fast-twitch).
/// Claude Code: Read/Grep BEFORE Bash, Haiku BEFORE Sonnet BEFORE Opus.
/// Escalate compute only as load demands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizePrinciple {
    /// Tools ordered from smallest (cheapest) to largest (most expensive)
    pub tool_order: Vec<String>,
}

impl Default for SizePrinciple {
    fn default() -> Self {
        Self {
            tool_order: vec![
                "Read".to_string(),
                "Grep".to_string(),
                "Glob".to_string(),
                "ToolSearch".to_string(),
                "WebSearch".to_string(),
                "WebFetch".to_string(),
                "Edit".to_string(),
                "Write".to_string(),
                "Bash".to_string(),
            ],
        }
    }
}

impl SizePrinciple {
    /// Check if a sequence of tool usages follows the Size Principle
    /// (smallest motor units recruited first).
    ///
    /// Returns true if tools were used in non-decreasing order of size,
    /// or if the sequence has fewer than 2 tools.
    #[must_use]
    pub fn is_compliant(&self, tool_sequence: &[&str]) -> bool {
        if tool_sequence.len() < 2 {
            return true;
        }

        let index_of = |name: &str| -> usize {
            self.tool_order
                .iter()
                .position(|t| t == name)
                .unwrap_or(self.tool_order.len())
        };

        let mut max_seen = 0usize;
        for tool in tool_sequence {
            let idx = index_of(tool);
            if idx < max_seen {
                return false;
            }
            if idx > max_seen {
                max_seen = idx;
            }
        }

        true
    }

    /// Returns the minimum (smallest) sufficient tool for a given complexity level.
    ///
    /// Complexity 0-2: Read (cheapest observation)
    /// Complexity 3-4: Grep (targeted search)
    /// Complexity 5-6: Edit (targeted modification)
    /// Complexity 7-8: Bash (general execution)
    /// Complexity 9+: Bash (heaviest tool)
    #[must_use]
    pub fn smallest_sufficient(&self, task_complexity: u8) -> &str {
        match task_complexity {
            0..=2 => self.tool_order.first().map_or("Read", |s| s.as_str()),
            3..=4 => self.tool_order.get(1).map_or("Grep", |s| s.as_str()),
            5..=6 => self.tool_order.get(6).map_or("Edit", |s| s.as_str()),
            7..=8 => self.tool_order.get(8).map_or("Bash", |s| s.as_str()),
            _ => self.tool_order.last().map_or("Bash", |s| s.as_str()),
        }
    }
}

// ============================================================================
// ModelEscalation — Haiku -> Sonnet -> Opus ladder
// ============================================================================

/// A rung on the model escalation ladder.
///
/// Size Principle applied to model selection: start with cheapest model,
/// escalate only when task complexity demands it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelEscalation {
    /// Model name (e.g., "Haiku", "Sonnet", "Opus")
    pub model_name: String,
    /// Relative compute cost (Haiku=1.0, Sonnet=3.0, Opus=10.0)
    pub compute_cost: f64,
    /// Minimum task complexity to justify this model (0.0 - 1.0)
    pub complexity_threshold: f64,
}

/// Returns the standard model escalation ladder.
///
/// - Haiku (cost 1.0): for complexity < 0.3
/// - Sonnet (cost 3.0): for complexity 0.3 - 0.6
/// - Opus (cost 10.0): for complexity > 0.6
#[must_use]
pub fn escalation_ladder() -> Vec<ModelEscalation> {
    vec![
        ModelEscalation {
            model_name: "Haiku".to_string(),
            compute_cost: 1.0,
            complexity_threshold: 0.3,
        },
        ModelEscalation {
            model_name: "Sonnet".to_string(),
            compute_cost: 3.0,
            complexity_threshold: 0.6,
        },
        ModelEscalation {
            model_name: "Opus".to_string(),
            compute_cost: 10.0,
            complexity_threshold: 0.9,
        },
    ]
}

// ============================================================================
// Fatigue — context window depletion tracking
// ============================================================================

/// Tracks context window fatigue, analogous to muscular fatigue.
///
/// As muscles deplete glycogen, Claude depletes context tokens.
/// Fatigue > 0.80 triggers refractory period (compaction).
/// Fatigue > 0.90 indicates exhaustion (handoff needed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fatigue {
    /// Total available context tokens
    pub total_context_tokens: u64,
    /// Currently used context tokens
    pub used_context_tokens: u64,
    /// Number of handoffs (compaction/continuation events)
    pub handoff_count: u32,
}

impl Fatigue {
    /// Create a new fatigue tracker with given context window size.
    #[must_use]
    pub fn new(total_tokens: u64) -> Self {
        Self {
            total_context_tokens: total_tokens,
            used_context_tokens: 0,
            handoff_count: 0,
        }
    }

    /// Returns the current fatigue level as a ratio (0.0 - 1.0).
    ///
    /// 0.0 = fully rested, 1.0 = completely exhausted.
    #[must_use]
    pub fn fatigue_level(&self) -> f64 {
        if self.total_context_tokens == 0 {
            return 1.0;
        }
        self.used_context_tokens as f64 / self.total_context_tokens as f64
    }

    /// Returns true if fatigue exceeds 0.90 (exhaustion threshold).
    ///
    /// At this level, the agent needs to hand off to a fresh context.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.fatigue_level() > 0.90
    }

    /// Returns true if fatigue exceeds 0.80 (compaction threshold).
    ///
    /// At this level, a refractory period is needed: compact context,
    /// summarize, or offload to subagents.
    #[must_use]
    pub fn needs_refractory(&self) -> bool {
        self.fatigue_level() > 0.80
    }

    /// Record token usage.
    pub fn consume(&mut self, tokens: u64) {
        self.used_context_tokens = self.used_context_tokens.saturating_add(tokens);
    }
}

// ============================================================================
// MotorActivation — recruitment ratio tracking
// ============================================================================

/// Tracks tool activation counts to measure recruitment balance.
///
/// Healthy recruitment follows approximately:
/// - Bash: ~64% (most frequent, general purpose)
/// - Edit: ~26% (targeted modifications)
/// - Write: ~10% (new file creation, least frequent)
///
/// Acceptable ranges: Bash 50-75%, Edit 15-35%, Write 5-20%.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotorActivation {
    /// Number of Bash tool invocations
    pub bash_count: u64,
    /// Number of Edit tool invocations
    pub edit_count: u64,
    /// Number of Write tool invocations
    pub write_count: u64,
}

impl MotorActivation {
    /// Create a new activation tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bash_count: 0,
            edit_count: 0,
            write_count: 0,
        }
    }

    /// Returns the recruitment ratio as (bash%, edit%, write%) each in 0.0-1.0.
    ///
    /// Target: approximately (0.64, 0.26, 0.10).
    #[must_use]
    pub fn recruitment_ratio(&self) -> (f64, f64, f64) {
        let total = self.bash_count + self.edit_count + self.write_count;
        if total == 0 {
            return (0.0, 0.0, 0.0);
        }
        let t = total as f64;
        (
            self.bash_count as f64 / t,
            self.edit_count as f64 / t,
            self.write_count as f64 / t,
        )
    }

    /// Returns true if the recruitment ratio is within healthy bounds.
    ///
    /// Healthy: Bash 50-75%, Edit 15-35%, Write 5-20%.
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        let total = self.bash_count + self.edit_count + self.write_count;
        if total == 0 {
            return true;
        }
        let (bash, edit, write) = self.recruitment_ratio();
        (0.50..=0.75).contains(&bash)
            && (0.15..=0.35).contains(&edit)
            && (0.05..=0.20).contains(&write)
    }
}

impl Default for MotorActivation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MuscularHealth — overall system health snapshot
// ============================================================================

/// Snapshot of the muscular system's overall health.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MuscularHealth {
    /// Whether recent tool sequences followed the Size Principle
    pub size_principle_compliant: bool,
    /// Whether antagonistic pairs are properly defined
    pub antagonistic_pairs_defined: bool,
    /// Whether recruitment ratios are within healthy bounds
    pub recruitment_balanced: bool,
    /// Current fatigue level (0.0 - 1.0)
    pub fatigue_level: f64,
    /// Whether the cardiac (main loop) is running
    pub cardiac_running: bool,
}

impl MuscularHealth {
    /// Create a health snapshot from current system state.
    #[must_use]
    pub fn assess(
        size_principle: &SizePrinciple,
        recent_tools: &[&str],
        activation: &MotorActivation,
        fatigue: &Fatigue,
        cardiac_running: bool,
    ) -> Self {
        Self {
            size_principle_compliant: size_principle.is_compliant(recent_tools),
            antagonistic_pairs_defined: !standard_pairs().is_empty(),
            recruitment_balanced: activation.is_balanced(),
            fatigue_level: fatigue.fatigue_level(),
            cardiac_running,
        }
    }

    /// Returns true if all health indicators are positive.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.size_principle_compliant
            && self.antagonistic_pairs_defined
            && self.recruitment_balanced
            && self.fatigue_level <= 0.80
            && self.cardiac_running
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- MuscleType ---

    #[test]
    fn muscle_type_display() {
        assert_eq!(format!("{}", MuscleType::Skeletal), "Skeletal (voluntary)");
        assert_eq!(format!("{}", MuscleType::Smooth), "Smooth (involuntary)");
        assert_eq!(format!("{}", MuscleType::Cardiac), "Cardiac (autonomous)");
    }

    // --- ToolClassification ---

    #[test]
    fn classify_skeletal_tools() {
        for tool in &["Write", "Edit", "Bash", "WebFetch", "WebSearch"] {
            let c = ToolClassification::classify(tool);
            assert_eq!(
                c.muscle_type,
                MuscleType::Skeletal,
                "expected Skeletal for {tool}"
            );
            assert!(c.voluntary, "expected voluntary for {tool}");
        }
    }

    #[test]
    fn classify_smooth_tools() {
        for tool in &["Read", "Grep", "Glob", "ToolSearch"] {
            let c = ToolClassification::classify(tool);
            assert_eq!(
                c.muscle_type,
                MuscleType::Smooth,
                "expected Smooth for {tool}"
            );
            assert!(!c.voluntary, "expected involuntary for {tool}");
        }
    }

    #[test]
    fn classify_cardiac_tool() {
        let c = ToolClassification::classify("MainLoop");
        assert_eq!(c.muscle_type, MuscleType::Cardiac);
        assert!(!c.voluntary);
    }

    #[test]
    fn classify_unknown_defaults_to_skeletal() {
        let c = ToolClassification::classify("CustomTool");
        assert_eq!(c.muscle_type, MuscleType::Skeletal);
        assert!(c.voluntary);
    }

    // --- MotorUnit ---

    #[test]
    fn motor_unit_activation() {
        let mut unit = MotorUnit::new("Read");
        assert_eq!(unit.activation_count, 0);
        assert_eq!(unit.muscle_type, MuscleType::Smooth);

        unit.activate();
        assert_eq!(unit.activation_count, 1);
        assert!(!unit.last_activated.is_empty());

        unit.activate();
        assert_eq!(unit.activation_count, 2);
    }

    // --- AntagonisticPair ---

    #[test]
    fn standard_pairs_has_five() {
        let pairs = standard_pairs();
        assert_eq!(pairs.len(), 5);
    }

    #[test]
    fn standard_pairs_write_read() {
        let pairs = standard_pairs();
        let first = &pairs[0];
        assert_eq!(first.agonist, "Write");
        assert_eq!(first.antagonist, "Read");
    }

    #[test]
    fn standard_pairs_all_have_descriptions() {
        let pairs = standard_pairs();
        for pair in &pairs {
            assert!(!pair.description.is_empty());
            assert!(!pair.agonist.is_empty());
            assert!(!pair.antagonist.is_empty());
        }
    }

    // --- SizePrinciple ---

    #[test]
    fn size_principle_compliant_order() {
        let sp = SizePrinciple::default();
        assert!(sp.is_compliant(&["Read", "Grep", "Edit", "Bash"]));
    }

    #[test]
    fn size_principle_non_compliant_order() {
        let sp = SizePrinciple::default();
        assert!(!sp.is_compliant(&["Bash", "Read"]));
    }

    #[test]
    fn size_principle_empty_sequence() {
        let sp = SizePrinciple::default();
        assert!(sp.is_compliant(&[]));
    }

    #[test]
    fn size_principle_single_tool() {
        let sp = SizePrinciple::default();
        assert!(sp.is_compliant(&["Bash"]));
    }

    #[test]
    fn size_principle_same_tool_repeated() {
        let sp = SizePrinciple::default();
        assert!(sp.is_compliant(&["Read", "Read", "Read"]));
    }

    #[test]
    fn smallest_sufficient_low_complexity() {
        let sp = SizePrinciple::default();
        assert_eq!(sp.smallest_sufficient(0), "Read");
        assert_eq!(sp.smallest_sufficient(1), "Read");
        assert_eq!(sp.smallest_sufficient(2), "Read");
    }

    #[test]
    fn smallest_sufficient_mid_complexity() {
        let sp = SizePrinciple::default();
        assert_eq!(sp.smallest_sufficient(3), "Grep");
        assert_eq!(sp.smallest_sufficient(4), "Grep");
    }

    #[test]
    fn smallest_sufficient_high_complexity() {
        let sp = SizePrinciple::default();
        assert_eq!(sp.smallest_sufficient(5), "Edit");
        assert_eq!(sp.smallest_sufficient(6), "Edit");
        assert_eq!(sp.smallest_sufficient(7), "Bash");
        assert_eq!(sp.smallest_sufficient(9), "Bash");
    }

    // --- ModelEscalation ---

    #[test]
    fn escalation_ladder_has_three_rungs() {
        let ladder = escalation_ladder();
        assert_eq!(ladder.len(), 3);
        assert_eq!(ladder[0].model_name, "Haiku");
        assert_eq!(ladder[1].model_name, "Sonnet");
        assert_eq!(ladder[2].model_name, "Opus");
    }

    #[test]
    fn escalation_ladder_cost_increases() {
        let ladder = escalation_ladder();
        assert!(ladder[0].compute_cost < ladder[1].compute_cost);
        assert!(ladder[1].compute_cost < ladder[2].compute_cost);
    }

    // --- Fatigue ---

    #[test]
    fn fatigue_level_fresh() {
        let f = Fatigue::new(200_000);
        assert!((f.fatigue_level() - 0.0).abs() < f64::EPSILON);
        assert!(!f.is_exhausted());
        assert!(!f.needs_refractory());
    }

    #[test]
    fn fatigue_level_moderate() {
        let mut f = Fatigue::new(100_000);
        f.consume(50_000);
        assert!((f.fatigue_level() - 0.5).abs() < f64::EPSILON);
        assert!(!f.is_exhausted());
        assert!(!f.needs_refractory());
    }

    #[test]
    fn fatigue_needs_refractory() {
        let mut f = Fatigue::new(100_000);
        f.consume(85_000);
        assert!(f.needs_refractory());
        assert!(!f.is_exhausted());
    }

    #[test]
    fn fatigue_is_exhausted() {
        let mut f = Fatigue::new(100_000);
        f.consume(95_000);
        assert!(f.is_exhausted());
        assert!(f.needs_refractory());
    }

    #[test]
    fn fatigue_zero_total_is_exhausted() {
        let f = Fatigue::new(0);
        assert!((f.fatigue_level() - 1.0).abs() < f64::EPSILON);
        assert!(f.is_exhausted());
    }

    #[test]
    fn fatigue_consume_saturates() {
        let mut f = Fatigue::new(100);
        f.consume(u64::MAX);
        assert!(f.fatigue_level() >= 1.0);
    }

    // --- MotorActivation ---

    #[test]
    fn recruitment_ratio_empty() {
        let a = MotorActivation::new();
        let (b, e, w) = a.recruitment_ratio();
        assert!((b - 0.0).abs() < f64::EPSILON);
        assert!((e - 0.0).abs() < f64::EPSILON);
        assert!((w - 0.0).abs() < f64::EPSILON);
        assert!(a.is_balanced()); // empty is considered balanced
    }

    #[test]
    fn recruitment_ratio_balanced() {
        let a = MotorActivation {
            bash_count: 64,
            edit_count: 26,
            write_count: 10,
        };
        let (b, e, w) = a.recruitment_ratio();
        assert!(b > 0.50 && b < 0.75);
        assert!(e > 0.15 && e < 0.35);
        assert!(w > 0.05 && w < 0.20);
        assert!(a.is_balanced());
    }

    #[test]
    fn recruitment_ratio_unbalanced() {
        let a = MotorActivation {
            bash_count: 10,
            edit_count: 10,
            write_count: 80,
        };
        assert!(!a.is_balanced());
    }

    // --- MuscularHealth ---

    #[test]
    fn muscular_health_healthy() {
        let sp = SizePrinciple::default();
        let tools = ["Read", "Grep", "Edit"];
        let activation = MotorActivation {
            bash_count: 64,
            edit_count: 26,
            write_count: 10,
        };
        let fatigue = Fatigue::new(200_000);
        let health = MuscularHealth::assess(&sp, &tools, &activation, &fatigue, true);
        assert!(health.is_healthy());
    }

    #[test]
    fn muscular_health_fatigued() {
        let sp = SizePrinciple::default();
        let tools = ["Read", "Grep"];
        let activation = MotorActivation {
            bash_count: 64,
            edit_count: 26,
            write_count: 10,
        };
        let mut fatigue = Fatigue::new(100_000);
        fatigue.consume(90_000);
        let health = MuscularHealth::assess(&sp, &tools, &activation, &fatigue, true);
        assert!(!health.is_healthy());
    }

    #[test]
    fn muscular_health_cardiac_stopped() {
        let sp = SizePrinciple::default();
        let tools = ["Read"];
        let activation = MotorActivation {
            bash_count: 64,
            edit_count: 26,
            write_count: 10,
        };
        let fatigue = Fatigue::new(200_000);
        let health = MuscularHealth::assess(&sp, &tools, &activation, &fatigue, false);
        assert!(!health.is_healthy());
    }

    // --- Serde round-trip ---

    #[test]
    fn serde_muscle_type_round_trip() {
        let mt = MuscleType::Skeletal;
        let json = serde_json::to_string(&mt).unwrap_or_default();
        let back: MuscleType = serde_json::from_str(&json).unwrap_or(MuscleType::Cardiac);
        assert_eq!(back, mt);
    }

    #[test]
    fn serde_tool_classification_round_trip() {
        let tc = ToolClassification::classify("Bash");
        let json = serde_json::to_string(&tc).unwrap_or_default();
        assert!(json.contains("Bash"));
    }
}
