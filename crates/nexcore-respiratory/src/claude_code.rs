//! # Claude Code Infrastructure Mapping — Respiratory System
//!
//! Maps biological respiratory concepts to Claude Code's **context window management**
//! per Biological Alignment v2.0 §10.
//!
//! ```text
//! Lungs        = Context window (exchange surface for information processing)
//! Inhalation   = Tool invocations pulling data INTO context (Read, Grep, Glob, MCP)
//! Exhalation   = AUTOCOMPACT (passive expiration of stale context)
//! Gas exchange = Reasoning (fresh input tokens → processed output tokens)
//! Dead space   = System prompt overhead, irrelevant CLAUDE.md entries, verbose skills
//! Context fork = Subagent lungs (prevents emphysema from cross-task pollution)
//! Tidal volume = Working context (compacted and refreshed each cycle)
//! Vital cap.   = Maximum context window size
//! Residual vol = System prompt + CLAUDE.md (cannot be exhaled)
//! ```
//!
//! ## Mapping Table (§10)
//!
//! | Biological Concept | Claude Code Mechanism | Type |
//! |-------------------|----------------------|------|
//! | Inhalation (pull) | Tool invocations (Read, Grep, Glob, MCP) | [`Inhalation`] |
//! | Exhalation (passive) | AUTOCOMPACT expiring stale context | [`Exhalation`] |
//! | Gas exchange | Reasoning: input tokens → output tokens | [`GasExchange`] |
//! | Dead space | System prompt + CLAUDE.md overhead | [`DeadSpace`] |
//! | Context fork | Subagent isolation (separate lungs) | [`ContextFork`] |
//! | Tidal volume | Working context per compaction cycle | [`TidalVolume`] |
//! | Vital capacity | Max context window | [`VitalCapacity`] |
//! | Breath cycle | Full inhalation → exchange → exhalation | [`BreathCycle`] |
//! | Respiratory health | Session-level diagnostic | [`RespiratoryHealth`] |

use serde::{Deserialize, Serialize};

// ============================================================================
// Context Source — What kind of tool pulled data into context (§10: inhalation)
// ============================================================================

/// Classification of how data enters the context window.
/// Biology: different airways (nose, mouth) carry different air qualities.
/// Claude Code: different tools pull different kinds of context.
///
/// Alignment doc §10: Inhalation is pull-based — the agent actively requests data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextSource {
    /// File reading tools: Read, Grep, Glob
    Tool,
    /// MCP server tool invocations (nexcore, claude-fs, compendious)
    McpCall,
    /// System prompt injected at session start (residual volume)
    SystemPrompt,
    /// User-provided messages (the breath trigger)
    UserMessage,
    /// Skill output from executed skills
    SkillOutput,
}

// ============================================================================
// Inhalation — Data pulled into context (§10: pull-based intake)
// ============================================================================

/// A single inhalation event: data pulled into the context window via a tool.
///
/// Biology: each breath draws air (data) through the trachea (tool pipeline)
/// into the alveoli (context window) for gas exchange (reasoning).
///
/// Alignment doc §10: Inhalation is active — the agent chooses what to pull.
/// Priority determines what survives compaction (exhalation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inhalation {
    /// How this data entered context
    pub source: ContextSource,
    /// Description of the content pulled in
    pub content: String,
    /// Token count consumed by this inhalation
    pub tokens_consumed: u64,
    /// Priority for compaction survival (lower = more important, survives longer)
    pub priority: u8,
}

// ============================================================================
// Exhalation — Context expiration via AUTOCOMPACT (§10: passive expiry)
// ============================================================================

/// An exhalation event: stale context expired via AUTOCOMPACT.
///
/// Biology: exhalation is passive — the diaphragm relaxes and CO2 leaves.
/// Claude Code: AUTOCOMPACT passively removes stale context when the window fills.
///
/// Alignment doc §10: Exhalation is NOT active deletion — it is passive expiry
/// triggered by context pressure (filling the lungs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exhalation {
    /// Tokens freed by this compaction event
    pub tokens_freed: u64,
    /// What triggered compaction (e.g., "context_limit_reached", "manual_compact")
    pub compaction_trigger: String,
    /// Number of stale items removed in this exhalation
    pub stale_items_count: u32,
}

// ============================================================================
// Gas Exchange — The actual reasoning (§10: input → output transformation)
// ============================================================================

/// A gas exchange event: the reasoning step where input tokens produce output tokens.
///
/// Biology: O2 diffuses into blood, CO2 diffuses out — passive gradient-driven exchange.
/// Claude Code: fresh context (O2) is processed into reasoning output (CO2 is waste context).
///
/// Alignment doc §10: Exchange ratio < 1.0 means the model produces fewer output tokens
/// than input tokens consumed — typical for analysis tasks. Ratio > 1.0 indicates
/// generative tasks (more output than input, like code generation).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GasExchange {
    /// Input tokens consumed (O2 equivalent)
    pub input_tokens: u64,
    /// Output tokens produced (CO2 equivalent — useful waste)
    pub output_tokens: u64,
    /// Exchange ratio: output / input (efficiency metric)
    pub exchange_ratio: f64,
}

impl GasExchange {
    /// Create a new gas exchange record, computing the ratio automatically.
    ///
    /// Returns exchange_ratio of 0.0 if input_tokens is zero (no breathing occurred).
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        let exchange_ratio = if input_tokens == 0 {
            0.0
        } else {
            output_tokens as f64 / input_tokens as f64
        };
        Self {
            input_tokens,
            output_tokens,
            exchange_ratio,
        }
    }
}

// ============================================================================
// Dead Space — Overhead that consumes context without contributing (§10)
// ============================================================================

/// Dead space: context tokens consumed by overhead that does not contribute
/// to the current reasoning task.
///
/// Biology: anatomical dead space is airway volume that doesn't participate in
/// gas exchange (trachea, bronchi). Increasing dead space = less efficient breathing.
///
/// Alignment doc §10: System prompt, irrelevant CLAUDE.md entries, and verbose
/// skill descriptions all consume context without aiding the current task.
/// Dead space ratio > 0.3 indicates inefficiency (emphysema-like condition).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DeadSpace {
    /// Tokens consumed by the system prompt
    pub system_prompt_tokens: u64,
    /// Tokens consumed by CLAUDE.md content
    pub claude_md_tokens: u64,
    /// Tokens consumed by skill descriptions in context
    pub skill_description_tokens: u64,
    /// Total dead space = sum of all overhead
    pub total_dead_space: u64,
    /// Dead space ratio = dead_space / total_context (0.0 to 1.0)
    pub dead_space_ratio: f64,
}

impl DeadSpace {
    /// Compute dead space metrics from component token counts and total context size.
    ///
    /// Returns dead_space_ratio of 0.0 if total_context is zero.
    pub fn compute(
        system_prompt_tokens: u64,
        claude_md_tokens: u64,
        skill_description_tokens: u64,
        total_context: u64,
    ) -> Self {
        let total_dead_space = system_prompt_tokens + claude_md_tokens + skill_description_tokens;
        let dead_space_ratio = if total_context == 0 {
            0.0
        } else {
            total_dead_space as f64 / total_context as f64
        };
        Self {
            system_prompt_tokens,
            claude_md_tokens,
            skill_description_tokens,
            total_dead_space,
            dead_space_ratio,
        }
    }

    /// Whether dead space is excessive (> 30% of total context).
    /// Biology: high dead space ratio indicates obstructive pulmonary disease.
    /// Claude Code: too much overhead means less room for actual reasoning.
    pub fn is_excessive(&self) -> bool {
        self.dead_space_ratio > 0.3
    }
}

// ============================================================================
// Context Fork — Subagent isolation (§10: separate lungs)
// ============================================================================

/// A context fork: a subagent spawned with its own isolated context window.
///
/// Biology: each lung operates semi-independently. If one collapses (pneumothorax),
/// the other continues. Cross-contamination between lungs = emphysema.
///
/// Alignment doc §10: Subagent context forks prevent emphysema (cross-task pollution).
/// `isolated=true` means the fork shares NO mutable context with the parent.
/// `shared_prompt_only=true` means only the system prompt crosses the boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFork {
    /// Parent session identifier
    pub parent_session: String,
    /// Fork identifier (subagent ID)
    pub fork_id: String,
    /// Full isolation: no shared mutable context
    pub isolated: bool,
    /// Only the system prompt is shared (residual volume only)
    pub shared_prompt_only: bool,
}

impl ContextFork {
    /// Whether this fork is properly isolated (no cross-task pollution risk).
    /// Biology: healthy lungs have intact pleural membranes separating them.
    pub fn is_properly_isolated(&self) -> bool {
        self.isolated || self.shared_prompt_only
    }
}

// ============================================================================
// Tidal Volume — Working context per compaction cycle (§10)
// ============================================================================

/// Tidal volume: the working context that gets compacted and refreshed each cycle.
///
/// Biology: tidal volume is the air moved in a normal breath (~500mL).
/// Not the full lung capacity — just the working portion.
///
/// Alignment doc §10: Working context = what AUTOCOMPACT refreshes each cycle.
/// High compaction_cycle_count with stable avg_tokens_per_cycle = healthy breathing.
/// Increasing avg_tokens_per_cycle over time = context bloat (approaching vital capacity).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TidalVolume {
    /// Current working context size in tokens
    pub working_context_tokens: u64,
    /// Number of compaction cycles completed this session
    pub compaction_cycle_count: u32,
    /// Average tokens processed per compaction cycle
    pub avg_tokens_per_cycle: f64,
}

impl TidalVolume {
    /// Whether tidal volume is stable (not bloating over time).
    /// A stable tidal volume means compaction is keeping pace with inhalation.
    ///
    /// Heuristic: if average tokens per cycle is less than 60% of working context,
    /// the system is breathing efficiently.
    pub fn is_stable(&self) -> bool {
        if self.working_context_tokens == 0 {
            return true;
        }
        self.avg_tokens_per_cycle < (self.working_context_tokens as f64 * 0.6)
    }
}

// ============================================================================
// Vital Capacity — Maximum context window (§10)
// ============================================================================

/// Vital capacity: the maximum usable context window.
///
/// Biology: vital capacity = total lung capacity - residual volume.
/// You can never exhale ALL the air — residual volume remains.
///
/// Alignment doc §10: Max context tokens minus the residual volume (system prompt +
/// CLAUDE.md that can never be compacted away). The remaining space is what the
/// agent can actually use for reasoning.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VitalCapacity {
    /// Maximum context window in tokens (total lung capacity)
    pub max_context_tokens: u64,
    /// Residual volume: system prompt + CLAUDE.md tokens (cannot be exhaled)
    pub residual_volume: u64,
    /// Current tidal volume snapshot
    pub tidal_volume: u64,
    /// Current dead space ratio
    pub dead_space_ratio: f64,
}

impl VitalCapacity {
    /// Usable capacity = max - residual.
    /// Biology: the air you can actually exchange during breathing.
    pub fn usable_capacity(&self) -> u64 {
        self.max_context_tokens.saturating_sub(self.residual_volume)
    }

    /// Utilization ratio: tidal_volume / usable_capacity.
    /// High utilization (>0.9) means the agent is near capacity — handoff may be needed.
    pub fn utilization(&self) -> f64 {
        let usable = self.usable_capacity();
        if usable == 0 {
            return 1.0;
        }
        self.tidal_volume as f64 / usable as f64
    }

    /// Whether a handoff (new session) should be triggered.
    /// Biology: when you can't get enough O2, you need supplemental breathing (ventilator).
    /// Claude Code: when context is >90% full, hand off to a fresh session.
    pub fn should_handoff(&self) -> bool {
        self.utilization() > 0.9
    }
}

// ============================================================================
// Breath Cycle — Full inhalation → exchange → exhalation (§10)
// ============================================================================

/// A complete breath cycle: all inhalations, the gas exchange, and optional exhalation.
///
/// Biology: one breath = inhale → gas exchange → exhale.
/// Claude Code: one turn = tool calls (inhale) → reasoning (exchange) → compaction (exhale).
///
/// Alignment doc §10: Not every breath triggers exhalation — AUTOCOMPACT only fires
/// when context pressure is high. `handoff_triggered` indicates the session was
/// handed off to a fresh context (the biological equivalent of switching to a ventilator).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathCycle {
    /// All inhalation events in this cycle (tool calls pulling data in)
    pub inhalations: Vec<Inhalation>,
    /// The gas exchange event (reasoning step)
    pub gas_exchange: GasExchange,
    /// Optional exhalation (only if AUTOCOMPACT fired)
    pub exhalation: Option<Exhalation>,
    /// Whether this cycle triggered a handoff to fresh context
    pub handoff_triggered: bool,
}

impl BreathCycle {
    /// Total tokens inhaled in this cycle.
    pub fn total_inhaled_tokens(&self) -> u64 {
        self.inhalations.iter().map(|i| i.tokens_consumed).sum()
    }

    /// Whether this cycle included an exhalation (compaction event).
    pub fn had_exhalation(&self) -> bool {
        self.exhalation.is_some()
    }
}

// ============================================================================
// Respiratory Health — Session-level diagnostic (§10 design table)
// ============================================================================

/// Session-level respiratory health diagnostic per alignment doc §10.
///
/// Biology: spirometry measures lung function. Reduced capacity, high dead space,
/// or hyperventilation all indicate disease.
///
/// Claude Code: too many handoffs, high dead space ratio, or context pollution
/// all indicate the agent is struggling with its context window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespiratoryHealth {
    /// Number of handoffs (session transfers) in this session
    pub handoffs_per_session: u32,
    /// Current dead space ratio (overhead / total context)
    pub dead_space_ratio: f64,
    /// Whether context pollution has been detected (cross-task contamination)
    pub is_emphysemic: bool,
    /// Whether handoff rate is normal (< 20 per session)
    pub breathing_rate_normal: bool,
}

impl RespiratoryHealth {
    /// Diagnose respiratory health from session metrics.
    ///
    /// Checks:
    /// - Handoff rate: <20 per session is normal (biology: 12-20 breaths/min normal)
    /// - Dead space ratio: <0.3 is healthy (biology: VD/VT ratio ~0.2-0.35 normal)
    /// - Emphysema: context pollution detected = cross-task contamination in subagents
    ///
    /// Alignment doc §10: A healthy respiratory system means the agent can reason
    /// effectively within its context window without excessive handoffs or overhead.
    pub fn diagnose(handoffs_per_session: u32, dead_space_ratio: f64, is_emphysemic: bool) -> Self {
        let breathing_rate_normal = handoffs_per_session < 20;
        Self {
            handoffs_per_session,
            dead_space_ratio,
            is_emphysemic,
            breathing_rate_normal,
        }
    }

    /// Is the respiratory system healthy?
    ///
    /// Healthy = normal handoff rate AND acceptable dead space AND no emphysema.
    pub fn is_healthy(&self) -> bool {
        self.breathing_rate_normal && self.dead_space_ratio < 0.3 && !self.is_emphysemic
    }

    /// Severity assessment: how many health indicators are failing?
    /// 0 = healthy, 1 = mild, 2 = moderate, 3 = severe.
    pub fn severity(&self) -> u8 {
        let mut score: u8 = 0;
        if !self.breathing_rate_normal {
            score += 1;
        }
        if self.dead_space_ratio >= 0.3 {
            score += 1;
        }
        if self.is_emphysemic {
            score += 1;
        }
        score
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_source_variants() {
        let sources = [
            ContextSource::Tool,
            ContextSource::McpCall,
            ContextSource::SystemPrompt,
            ContextSource::UserMessage,
            ContextSource::SkillOutput,
        ];
        assert_eq!(sources.len(), 5);
        // Each variant is distinct
        assert_ne!(ContextSource::Tool, ContextSource::McpCall);
        assert_ne!(ContextSource::SystemPrompt, ContextSource::UserMessage);
    }

    #[test]
    fn inhalation_creation() {
        let inhalation = Inhalation {
            source: ContextSource::Tool,
            content: "Read(/home/matthew/nexcore/src/lib.rs)".to_string(),
            tokens_consumed: 1500,
            priority: 1,
        };
        assert_eq!(inhalation.tokens_consumed, 1500);
        assert_eq!(inhalation.source, ContextSource::Tool);
        assert_eq!(inhalation.priority, 1);
    }

    #[test]
    fn exhalation_records_compaction() {
        let exhalation = Exhalation {
            tokens_freed: 50000,
            compaction_trigger: "context_limit_reached".to_string(),
            stale_items_count: 12,
        };
        assert_eq!(exhalation.tokens_freed, 50000);
        assert_eq!(exhalation.stale_items_count, 12);
    }

    #[test]
    fn gas_exchange_ratio_calculation() {
        let exchange = GasExchange::new(10000, 3000);
        assert!((exchange.exchange_ratio - 0.3).abs() < f64::EPSILON);

        // Zero input produces zero ratio
        let zero_input = GasExchange::new(0, 100);
        assert!((zero_input.exchange_ratio - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gas_exchange_generative_vs_analytical() {
        // Analytical task: more input than output (ratio < 1.0)
        let analytical = GasExchange::new(10000, 2000);
        assert!(analytical.exchange_ratio < 1.0);

        // Generative task: more output than input (ratio > 1.0)
        let generative = GasExchange::new(2000, 8000);
        assert!(generative.exchange_ratio > 1.0);
    }

    #[test]
    fn dead_space_computation() {
        let ds = DeadSpace::compute(2000, 3000, 500, 20000);
        assert_eq!(ds.total_dead_space, 5500);
        assert!((ds.dead_space_ratio - 0.275).abs() < f64::EPSILON);
        assert!(!ds.is_excessive()); // 27.5% < 30%
    }

    #[test]
    fn dead_space_excessive_detection() {
        let ds = DeadSpace::compute(5000, 4000, 2000, 20000);
        assert_eq!(ds.total_dead_space, 11000);
        assert!((ds.dead_space_ratio - 0.55).abs() < f64::EPSILON);
        assert!(ds.is_excessive()); // 55% > 30%
    }

    #[test]
    fn dead_space_zero_context() {
        let ds = DeadSpace::compute(100, 200, 50, 0);
        assert!((ds.dead_space_ratio - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn context_fork_isolation() {
        let isolated_fork = ContextFork {
            parent_session: "session-abc".to_string(),
            fork_id: "subagent-1".to_string(),
            isolated: true,
            shared_prompt_only: false,
        };
        assert!(isolated_fork.is_properly_isolated());

        let shared_prompt_fork = ContextFork {
            parent_session: "session-abc".to_string(),
            fork_id: "subagent-2".to_string(),
            isolated: false,
            shared_prompt_only: true,
        };
        assert!(shared_prompt_fork.is_properly_isolated());

        let polluted_fork = ContextFork {
            parent_session: "session-abc".to_string(),
            fork_id: "subagent-3".to_string(),
            isolated: false,
            shared_prompt_only: false,
        };
        assert!(!polluted_fork.is_properly_isolated());
    }

    #[test]
    fn tidal_volume_stability() {
        let stable = TidalVolume {
            working_context_tokens: 100_000,
            compaction_cycle_count: 5,
            avg_tokens_per_cycle: 40_000.0,
        };
        assert!(stable.is_stable()); // 40k < 60k (60% of 100k)

        let bloated = TidalVolume {
            working_context_tokens: 100_000,
            compaction_cycle_count: 5,
            avg_tokens_per_cycle: 80_000.0,
        };
        assert!(!bloated.is_stable()); // 80k > 60k

        // Zero working context is trivially stable
        let empty = TidalVolume {
            working_context_tokens: 0,
            compaction_cycle_count: 0,
            avg_tokens_per_cycle: 0.0,
        };
        assert!(empty.is_stable());
    }

    #[test]
    fn vital_capacity_usable_and_utilization() {
        let vc = VitalCapacity {
            max_context_tokens: 200_000,
            residual_volume: 20_000,
            tidal_volume: 90_000,
            dead_space_ratio: 0.1,
        };
        assert_eq!(vc.usable_capacity(), 180_000);
        assert!((vc.utilization() - 0.5).abs() < f64::EPSILON);
        assert!(!vc.should_handoff());
    }

    #[test]
    fn vital_capacity_handoff_trigger() {
        let vc = VitalCapacity {
            max_context_tokens: 200_000,
            residual_volume: 20_000,
            tidal_volume: 170_000, // 170k / 180k usable = 94.4%
            dead_space_ratio: 0.1,
        };
        assert!(vc.utilization() > 0.9);
        assert!(vc.should_handoff());
    }

    #[test]
    fn vital_capacity_zero_usable() {
        let vc = VitalCapacity {
            max_context_tokens: 1000,
            residual_volume: 2000, // residual > max (pathological)
            tidal_volume: 0,
            dead_space_ratio: 1.0,
        };
        assert_eq!(vc.usable_capacity(), 0); // saturating_sub
        assert!((vc.utilization() - 1.0).abs() < f64::EPSILON);
        assert!(vc.should_handoff());
    }

    #[test]
    fn breath_cycle_aggregation() {
        let cycle = BreathCycle {
            inhalations: vec![
                Inhalation {
                    source: ContextSource::Tool,
                    content: "Read(lib.rs)".to_string(),
                    tokens_consumed: 1500,
                    priority: 1,
                },
                Inhalation {
                    source: ContextSource::McpCall,
                    content: "skill_list()".to_string(),
                    tokens_consumed: 800,
                    priority: 3,
                },
            ],
            gas_exchange: GasExchange::new(2300, 500),
            exhalation: Some(Exhalation {
                tokens_freed: 10000,
                compaction_trigger: "context_limit_reached".to_string(),
                stale_items_count: 5,
            }),
            handoff_triggered: false,
        };
        assert_eq!(cycle.total_inhaled_tokens(), 2300);
        assert!(cycle.had_exhalation());
        assert!(!cycle.handoff_triggered);
    }

    #[test]
    fn breath_cycle_no_exhalation() {
        let cycle = BreathCycle {
            inhalations: vec![Inhalation {
                source: ContextSource::UserMessage,
                content: "hello".to_string(),
                tokens_consumed: 10,
                priority: 1,
            }],
            gas_exchange: GasExchange::new(10, 50),
            exhalation: None,
            handoff_triggered: false,
        };
        assert!(!cycle.had_exhalation());
        assert_eq!(cycle.total_inhaled_tokens(), 10);
    }

    #[test]
    fn respiratory_health_healthy() {
        let health = RespiratoryHealth::diagnose(5, 0.15, false);
        assert!(health.is_healthy());
        assert!(health.breathing_rate_normal);
        assert_eq!(health.severity(), 0);
    }

    #[test]
    fn respiratory_health_emphysemic() {
        let health = RespiratoryHealth::diagnose(3, 0.1, true);
        assert!(!health.is_healthy());
        assert_eq!(health.severity(), 1); // only emphysema flag
    }

    #[test]
    fn respiratory_health_severe() {
        let health = RespiratoryHealth::diagnose(25, 0.5, true);
        assert!(!health.is_healthy());
        assert!(!health.breathing_rate_normal);
        assert_eq!(health.severity(), 3); // all three indicators failing
    }

    #[test]
    fn respiratory_health_high_dead_space_only() {
        let health = RespiratoryHealth::diagnose(10, 0.4, false);
        assert!(!health.is_healthy());
        assert!(health.breathing_rate_normal);
        assert_eq!(health.severity(), 1);
    }

    #[test]
    fn serde_round_trip() {
        let health = RespiratoryHealth::diagnose(5, 0.15, false);
        let json = serde_json::to_string(&health);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_else(|_| String::new());
        let parsed: Result<RespiratoryHealth, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok());
        let restored = parsed.unwrap_or_else(|_| RespiratoryHealth::diagnose(0, 0.0, false));
        assert_eq!(restored.handoffs_per_session, 5);
        assert!(restored.is_healthy());
    }
}
