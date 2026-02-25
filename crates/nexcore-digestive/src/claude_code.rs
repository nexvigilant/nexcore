//! # Claude Code Skill Execution Pipeline — Digestive System Mapping
//!
//! Maps the biological digestive system to Claude Code's skill execution pipeline
//! per Biological Alignment v2.0 Section 7.
//!
//! ## Organ-to-Skill Mapping
//!
//! | Biological Organ | Skill Pipeline Stage | Function |
//! |------------------|---------------------|----------|
//! | Mouth | Skill trigger detection | "Chewing" the user's request to detect skill invocations |
//! | Esophagus | Skill file loading | Peristaltic transport of SKILL.md to processing |
//! | Stomach | SKILL.md instruction parsing | Aggressive breakdown of frontmatter + content |
//! | Small Intestine | Skill execution | WHERE 90% OF VALUE IS EXTRACTED |
//! | Large Intestine | Output formatting | Consolidation before delivery to the user |
//! | Sphincters | Gate flags | `disable-model-invocation` (lower) and `user-invocable` (upper) |
//! | Microbiome | `!command` substitutions | External processes producing essential nutrients |
//! | Chyme | `$ARGUMENTS` | Broken-down user input flowing through the skill |
//! | Enzymes | String substitutions | `$ARGUMENTS`, `${SESSION_ID}`, `!git status` |
//! | Unidirectional flow | Forward failure | Skills fail FORWARD (error output), never backward |
//!
//! ## Key Principle: Unidirectional Flow
//!
//! Like the biological digestive tract, skills fail FORWARD. If execution fails,
//! the error is emitted as output — there is no retry from the trigger stage.
//! Peristalsis only moves one direction.

#![allow(
    dead_code,
    reason = "Digestive-system model includes reference types not yet consumed by runtime paths"
)]

use serde::{Deserialize, Serialize};

// ============================================================================
// SkillTrigger — The Mouth
// ============================================================================

/// The Mouth of the skill pipeline: detects skill invocations in user input.
///
/// Per Biological Alignment v2.0 Section 7, the mouth "chews" the user's request,
/// breaking it down to identify whether a skill invocation pattern is present.
/// This is the first stage: no processing happens until a trigger is detected.
///
/// Tier: T2-C (kappa + partial + varsigma + mu) — Comparison-dominant
/// (the entire purpose is matching patterns against input)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    /// The pattern that activates this skill (e.g., "/commit", "/review-pr")
    pub trigger_pattern: String,
    /// Whether the trigger pattern was detected in the current input
    pub matched: bool,
    /// Whether this skill was invoked by a user (explicit slash command)
    pub user_invoked: bool,
    /// Whether this skill was invoked by the model (automatic detection)
    pub model_invoked: bool,
}

impl SkillTrigger {
    /// Create a new skill trigger with the given pattern.
    pub fn new(trigger_pattern: impl Into<String>) -> Self {
        Self {
            trigger_pattern: trigger_pattern.into(),
            matched: false,
            user_invoked: false,
            model_invoked: false,
        }
    }

    /// Attempt to match this trigger against user input (the "chewing" action).
    pub fn chew(&mut self, input: &str) -> bool {
        self.matched = input.contains(&self.trigger_pattern);
        self.matched
    }
}

// ============================================================================
// SkillLoad — The Esophagus
// ============================================================================

/// The Esophagus of the skill pipeline: loads the skill file from disk.
///
/// Per Biological Alignment v2.0 Section 7, the esophagus provides peristaltic
/// transport — moving the identified skill file to the processing stage (stomach).
/// This is a transport-only stage; no transformation occurs here.
///
/// Tier: T2-C (sigma + pi + N + partial) — Sequence-dominant
/// (ordered transport from detection to parsing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLoad {
    /// Name of the skill being loaded
    pub skill_name: String,
    /// Filesystem path to the SKILL.md file
    pub file_path: String,
    /// Size of the loaded content in bytes
    pub content_size: usize,
    /// Time taken to load the file in milliseconds
    pub load_time_ms: u64,
}

impl SkillLoad {
    /// Create a new skill load record.
    pub fn new(
        skill_name: impl Into<String>,
        file_path: impl Into<String>,
        content_size: usize,
        load_time_ms: u64,
    ) -> Self {
        Self {
            skill_name: skill_name.into(),
            file_path: file_path.into(),
            content_size,
            load_time_ms,
        }
    }
}

// ============================================================================
// ContextMode — Fork vs Inherit
// ============================================================================

/// Context mode for skill execution.
///
/// Per Biological Alignment v2.0 Section 7, context mode determines whether the
/// skill executes in a forked context (separate "lungs" — isolated environment)
/// or inherits the parent context (shared state).
///
/// Tier: T2-P (Sigma + varsigma) — Sum-dominant (binary choice)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextMode {
    /// Separate context — skill runs in isolation ("separate lungs")
    Fork,
    /// Shared context — skill inherits the parent conversation state
    Inherit,
}

impl Default for ContextMode {
    fn default() -> Self {
        Self::Fork
    }
}

// ============================================================================
// SkillFrontmatter — The Stomach
// ============================================================================

/// The Stomach of the skill pipeline: parses SKILL.md frontmatter.
///
/// Per Biological Alignment v2.0 Section 7, the stomach performs "aggressive
/// breakdown" of the skill file content, extracting the structured frontmatter
/// fields that control execution. Just as gastric acid denatures proteins,
/// the frontmatter parser strips raw markdown into typed configuration.
///
/// Tier: T2-C (mu + partial + sigma + varsigma + Sigma) — Mapping-dominant
/// (transforms raw SKILL.md text into structured configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    /// The skill's display name
    pub name: String,
    /// Human-readable description of what the skill does
    pub description: String,
    /// List of tools this skill is allowed to invoke
    pub allowed_tools: Vec<String>,
    /// Whether the skill forks or inherits context
    pub context_mode: ContextMode,
    /// If true, the model cannot invoke this skill autonomously (lower esophageal sphincter)
    pub disable_model_invocation: bool,
    /// If true, users can invoke this skill directly (upper esophageal sphincter)
    pub user_invocable: bool,
}

impl SkillFrontmatter {
    /// Create a new frontmatter with the given name and description.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            allowed_tools: Vec::new(),
            context_mode: ContextMode::default(),
            disable_model_invocation: false,
            user_invocable: true,
        }
    }
}

// ============================================================================
// Sphincter — Gate Control
// ============================================================================

/// Sphincter gate for controlling skill flow.
///
/// Per Biological Alignment v2.0 Section 7, sphincters control the flow
/// between digestive stages. In the skill pipeline:
/// - Upper esophageal sphincter = `user_invocable` flag (controls user entry)
/// - Lower esophageal sphincter = `disable_model_invocation` flag (controls model entry)
///
/// Tier: T2-P (partial + varsigma) — Boundary-dominant (gate/barrier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sphincter {
    /// Gate is open — flow is permitted
    Open,
    /// Gate is closed — flow is blocked
    Closed,
}

impl Sphincter {
    /// Construct the upper esophageal sphincter from the `user_invocable` frontmatter flag.
    ///
    /// When `user_invocable` is true, the sphincter is Open (users can trigger the skill).
    /// When false, it is Closed (skill cannot be user-triggered).
    pub fn upper_esophageal(frontmatter: &SkillFrontmatter) -> Self {
        if frontmatter.user_invocable {
            Self::Open
        } else {
            Self::Closed
        }
    }

    /// Construct the lower esophageal sphincter from the `disable_model_invocation` flag.
    ///
    /// When `disable_model_invocation` is true, the sphincter is Closed (model cannot trigger).
    /// When false, it is Open (model is allowed to trigger the skill).
    pub fn lower_esophageal(frontmatter: &SkillFrontmatter) -> Self {
        if frontmatter.disable_model_invocation {
            Self::Closed
        } else {
            Self::Open
        }
    }

    /// Check whether this sphincter allows passage.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}

// ============================================================================
// SkillArguments — The Chyme
// ============================================================================

/// The Chyme: broken-down user input flowing through the skill pipeline.
///
/// Per Biological Alignment v2.0 Section 7, chyme is `$ARGUMENTS` — the user's
/// input after it has been broken down by the trigger detection (mouth) and loaded
/// (esophagus). These arguments flow unidirectionally through the remaining stages.
///
/// Tier: T2-C (sigma + mu + varsigma + partial) — Sequence-dominant
/// (ordered flow of arguments through stages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillArguments {
    /// The raw, unparsed argument string from the user
    pub raw_args: String,
    /// Parsed individual argument tokens
    pub parsed: Vec<String>,
    /// Optional session identifier for substitution
    pub session_id: Option<String>,
}

impl SkillArguments {
    /// Create new skill arguments from a raw string.
    pub fn new(raw_args: impl Into<String>) -> Self {
        let raw = raw_args.into();
        let parsed = raw.split_whitespace().map(|s| s.to_string()).collect();
        Self {
            raw_args: raw,
            parsed,
            session_id: None,
        }
    }

    /// Set the session ID for `${SESSION_ID}` enzyme substitution.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

// ============================================================================
// EnzymeType — Substitution Classification
// ============================================================================

/// Classification of enzyme substitutions in the skill pipeline.
///
/// Per Biological Alignment v2.0 Section 7, enzymes are the string substitution
/// mechanisms that transform chyme (arguments) as it flows through the pipeline.
/// Each enzyme type corresponds to a biological digestive enzyme:
///
/// - Amylase breaks down starches (simple carbs) = `$ARGUMENTS` (simple substitution)
/// - Pepsin breaks down proteins (complex structures) = `${SESSION_ID}` (context-aware)
/// - Lipase breaks down fats (stored energy) = `!command` (external process invocation)
///
/// Tier: T2-P (Sigma + mu) — Sum-dominant (3-variant classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnzymeType {
    /// `$ARGUMENTS` substitution — simple variable replacement (like amylase on starch)
    Amylase,
    /// `${SESSION_ID}` substitution — context-aware replacement (like pepsin on proteins)
    Pepsin,
    /// `!command` substitution — external process invocation (like lipase on fats)
    Lipase,
}

// ============================================================================
// EnzymeSubstitution — String Transformation
// ============================================================================

/// A single enzyme substitution: a pattern-to-replacement transformation.
///
/// Per Biological Alignment v2.0 Section 7, enzymes are the string substitution
/// mechanisms applied during skill execution. Each substitution has a pattern
/// (what to find), a replacement (what to substitute), and a type classification.
///
/// Tier: T2-C (mu + arrow + partial + Sigma) — Mapping-dominant
/// (pattern -> replacement is a mapping operation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnzymeSubstitution {
    /// The pattern to search for (e.g., "$ARGUMENTS", "${SESSION_ID}", "!`git status`")
    pub pattern: String,
    /// The replacement value after substitution
    pub replacement: String,
    /// The type of enzyme performing this substitution
    pub enzyme_type: EnzymeType,
}

impl EnzymeSubstitution {
    /// Create a new enzyme substitution.
    pub fn new(
        pattern: impl Into<String>,
        replacement: impl Into<String>,
        enzyme_type: EnzymeType,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            enzyme_type,
        }
    }
}

// ============================================================================
// Microbiome — Symbiotic Processes
// ============================================================================

/// The Microbiome: dynamic `!command` substitutions that produce essential nutrients.
///
/// Per Biological Alignment v2.0 Section 7, the microbiome represents the
/// external processes (shell commands) that are invoked during skill execution
/// via `!command` syntax. Just as gut microbiota produce vitamins and short-chain
/// fatty acids that the host cannot synthesize alone, these external commands
/// produce context (git status, file listings, etc.) that the skill cannot
/// generate internally.
///
/// Tier: T2-C (rho + mu + sigma + partial) — Recursion-dominant
/// (external commands are recursive calls out to the shell and back)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Microbiome {
    /// Shell commands that provide dynamic context (e.g., "git status", "ls -la")
    pub commands: Vec<String>,
}

impl Microbiome {
    /// Create an empty microbiome.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a symbiotic command to the microbiome.
    pub fn add_command(&mut self, command: impl Into<String>) {
        self.commands.push(command.into());
    }

    /// Check microbiome diversity (number of distinct command producers).
    pub fn diversity(&self) -> usize {
        self.commands.len()
    }
}

impl Default for Microbiome {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SkillResult — Forward Failure
// ============================================================================

/// The result of skill execution: success or forward failure.
///
/// Per Biological Alignment v2.0 Section 7, skills fail FORWARD. If execution
/// produces an error, that error is emitted as output — there is no retry from
/// the trigger stage. This mirrors the unidirectional flow of the digestive tract:
/// peristalsis only moves content in one direction.
///
/// Tier: T2-P (Sigma + arrow) — Sum-dominant (binary outcome classification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillResult {
    /// Skill executed successfully, producing output
    Success(String),
    /// Skill failed FORWARD — error is the output, no retry
    Failure(String),
}

impl SkillResult {
    /// Check if the skill result is a success.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Extract the output string regardless of success or failure.
    pub fn output(&self) -> &str {
        match self {
            Self::Success(s) | Self::Failure(s) => s,
        }
    }
}

// ============================================================================
// SkillExecution — The Small Intestine (90% of value extraction)
// ============================================================================

/// The Small Intestine of the skill pipeline: WHERE 90% OF VALUE IS EXTRACTED.
///
/// Per Biological Alignment v2.0 Section 7, the small intestine is where the
/// actual skill execution happens. Just as the biological small intestine absorbs
/// 90% of nutrients from digested food, the skill execution stage extracts 90%
/// of the value from the parsed skill instructions and arguments.
///
/// Tier: T3 (sigma + mu + arrow + N + partial + varsigma) — Causality-dominant
/// (the transformation from input to output is the primary operation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecution {
    /// Name of the skill being executed
    pub skill_name: String,
    /// The chyme: processed arguments flowing through execution
    pub arguments: SkillArguments,
    /// The result of execution (success or forward failure)
    pub result: SkillResult,
    /// Number of tokens consumed during execution
    pub tokens_consumed: u64,
    /// Estimated value extracted (0.0-1.0 scale, target: 0.90 for small intestine)
    pub value_extracted: f64,
}

impl SkillExecution {
    /// Create a new skill execution record.
    pub fn new(
        skill_name: impl Into<String>,
        arguments: SkillArguments,
        result: SkillResult,
        tokens_consumed: u64,
        value_extracted: f64,
    ) -> Self {
        Self {
            skill_name: skill_name.into(),
            arguments,
            result,
            tokens_consumed,
            value_extracted,
        }
    }
}

// ============================================================================
// DigestiveHealth — System Diagnostics
// ============================================================================

/// Diagnostic health check for the entire skill digestive pipeline.
///
/// Per Biological Alignment v2.0 Section 7, this provides a holistic view of
/// the skill pipeline's health, analogous to a GI tract health assessment.
///
/// Tier: T2-C (kappa + N + partial + varsigma) — Comparison-dominant
/// (health check is fundamentally a comparison against expected baselines)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestiveHealth {
    /// Number of skills available (target: 94, per the 94 skills in ~/nexcore/skills/)
    pub skill_count: usize,
    /// Whether sphincter control is correctly configured (upper and lower gates)
    pub sphincter_control_correct: bool,
    /// Whether the microbiome is diverse (has multiple command producers)
    pub microbiome_diverse: bool,
    /// Whether flow is unidirectional (skills fail forward, not backward)
    pub unidirectional_flow: bool,
}

impl DigestiveHealth {
    /// Diagnose the health of the skill digestive pipeline.
    ///
    /// Returns a diagnostic report with health indicators.
    ///
    /// ## Checks
    /// - `skill_count`: compared against target of 94 skills
    /// - `sphincter_control_correct`: both gates must be properly configured
    /// - `microbiome_diverse`: at least one command substitution available
    /// - `unidirectional_flow`: must be true (skills always fail forward)
    pub fn diagnose(
        skill_count: usize,
        frontmatter: &SkillFrontmatter,
        microbiome: &Microbiome,
    ) -> Self {
        let upper = Sphincter::upper_esophageal(frontmatter);
        let lower = Sphincter::lower_esophageal(frontmatter);

        // Sphincter control is correct when:
        // - A user-invocable skill has its upper sphincter open
        // - A model-disabled skill has its lower sphincter closed
        // Both constructors already enforce this, so correctness = consistency check
        let sphincter_control_correct = (frontmatter.user_invocable == upper.is_open())
            && (frontmatter.disable_model_invocation != lower.is_open());

        let microbiome_diverse = microbiome.diversity() > 0;

        Self {
            skill_count,
            sphincter_control_correct,
            microbiome_diverse,
            // Unidirectional flow is architecturally enforced by SkillResult:
            // there is no "Retry" variant, only Success and Failure (forward).
            unidirectional_flow: true,
        }
    }

    /// Check if the pipeline is healthy overall.
    pub fn is_healthy(&self) -> bool {
        self.sphincter_control_correct && self.microbiome_diverse && self.unidirectional_flow
    }

    /// Check if the skill count meets the target of 94.
    pub fn skill_count_at_target(&self) -> bool {
        self.skill_count >= 94
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- SkillTrigger (Mouth) tests ---

    #[test]
    fn trigger_detects_slash_command() {
        let mut trigger = SkillTrigger::new("/commit");
        let matched = trigger.chew("Please /commit my changes");
        assert!(matched);
        assert!(trigger.matched);
    }

    #[test]
    fn trigger_does_not_match_absent_pattern() {
        let mut trigger = SkillTrigger::new("/review-pr");
        let matched = trigger.chew("Just a normal message");
        assert!(!matched);
        assert!(!trigger.matched);
    }

    #[test]
    fn trigger_default_is_unmatched() {
        let trigger = SkillTrigger::new("/test");
        assert!(!trigger.matched);
        assert!(!trigger.user_invoked);
        assert!(!trigger.model_invoked);
    }

    // --- SkillLoad (Esophagus) tests ---

    #[test]
    fn skill_load_captures_metadata() {
        let load = SkillLoad::new(
            "commit",
            "/home/user/nexcore/skills/commit/SKILL.md",
            2048,
            5,
        );
        assert_eq!(load.skill_name, "commit");
        assert_eq!(load.content_size, 2048);
        assert_eq!(load.load_time_ms, 5);
    }

    // --- Sphincter (Gate) tests ---

    #[test]
    fn upper_sphincter_open_when_user_invocable() {
        let fm = SkillFrontmatter {
            name: "test".to_string(),
            description: "test skill".to_string(),
            allowed_tools: vec![],
            context_mode: ContextMode::Fork,
            disable_model_invocation: false,
            user_invocable: true,
        };
        assert_eq!(Sphincter::upper_esophageal(&fm), Sphincter::Open);
    }

    #[test]
    fn lower_sphincter_closed_when_model_disabled() {
        let fm = SkillFrontmatter {
            name: "test".to_string(),
            description: "test skill".to_string(),
            allowed_tools: vec![],
            context_mode: ContextMode::Fork,
            disable_model_invocation: true,
            user_invocable: true,
        };
        assert_eq!(Sphincter::lower_esophageal(&fm), Sphincter::Closed);
    }

    #[test]
    fn sphincter_is_open_method() {
        assert!(Sphincter::Open.is_open());
        assert!(!Sphincter::Closed.is_open());
    }

    // --- SkillArguments (Chyme) tests ---

    #[test]
    fn arguments_parse_whitespace_tokens() {
        let args = SkillArguments::new("-m fix-bug --no-verify");
        assert_eq!(args.parsed.len(), 3);
        assert_eq!(args.raw_args, "-m fix-bug --no-verify");
    }

    #[test]
    fn arguments_with_session_id() {
        let args = SkillArguments::new("test").with_session_id("abc-123");
        assert_eq!(args.session_id.as_deref(), Some("abc-123"));
    }

    // --- EnzymeSubstitution tests ---

    #[test]
    fn enzyme_substitution_types() {
        let amylase = EnzymeSubstitution::new("$ARGUMENTS", "hello world", EnzymeType::Amylase);
        assert_eq!(amylase.enzyme_type, EnzymeType::Amylase);

        let pepsin = EnzymeSubstitution::new("${SESSION_ID}", "abc-123", EnzymeType::Pepsin);
        assert_eq!(pepsin.enzyme_type, EnzymeType::Pepsin);

        let lipase = EnzymeSubstitution::new("!`git status`", "On branch main", EnzymeType::Lipase);
        assert_eq!(lipase.enzyme_type, EnzymeType::Lipase);
    }

    // --- Microbiome tests ---

    #[test]
    fn microbiome_diversity() {
        let mut microbiome = Microbiome::new();
        assert_eq!(microbiome.diversity(), 0);

        microbiome.add_command("git status");
        microbiome.add_command("ls -la");
        assert_eq!(microbiome.diversity(), 2);
    }

    // --- SkillResult (Forward Failure) tests ---

    #[test]
    fn skill_result_success() {
        let result = SkillResult::Success("output data".to_string());
        assert!(result.is_success());
        assert_eq!(result.output(), "output data");
    }

    #[test]
    fn skill_result_failure_is_forward() {
        let result = SkillResult::Failure("error: file not found".to_string());
        assert!(!result.is_success());
        // Failure output is still accessible — it fails FORWARD, not silently
        assert_eq!(result.output(), "error: file not found");
    }

    // --- SkillExecution (Small Intestine) tests ---

    #[test]
    fn skill_execution_captures_value() {
        let exec = SkillExecution::new(
            "commit",
            SkillArguments::new("-m 'fix'"),
            SkillResult::Success("committed abc123".to_string()),
            1500,
            0.92,
        );
        assert_eq!(exec.skill_name, "commit");
        assert!(exec.result.is_success());
        // Small intestine target: 90% value extraction
        assert!(exec.value_extracted >= 0.90);
    }

    // --- DigestiveHealth tests ---

    #[test]
    fn health_diagnose_healthy_pipeline() {
        let fm = SkillFrontmatter::new("test", "a test skill");
        let mut microbiome = Microbiome::new();
        microbiome.add_command("git status");

        let health = DigestiveHealth::diagnose(94, &fm, &microbiome);
        assert!(health.is_healthy());
        assert!(health.skill_count_at_target());
        assert!(health.unidirectional_flow);
        assert!(health.sphincter_control_correct);
        assert!(health.microbiome_diverse);
    }

    #[test]
    fn health_diagnose_unhealthy_no_microbiome() {
        let fm = SkillFrontmatter::new("test", "a test skill");
        let microbiome = Microbiome::new(); // empty — no symbiotic processes

        let health = DigestiveHealth::diagnose(94, &fm, &microbiome);
        assert!(!health.is_healthy());
        assert!(!health.microbiome_diverse);
    }

    #[test]
    fn health_skill_count_below_target() {
        let fm = SkillFrontmatter::new("test", "a test skill");
        let mut microbiome = Microbiome::new();
        microbiome.add_command("git status");

        let health = DigestiveHealth::diagnose(50, &fm, &microbiome);
        assert!(!health.skill_count_at_target());
        // Pipeline can still be healthy even with fewer skills
        assert!(health.is_healthy());
    }

    #[test]
    fn context_mode_default_is_fork() {
        assert_eq!(ContextMode::default(), ContextMode::Fork);
    }

    #[test]
    fn frontmatter_new_defaults() {
        let fm = SkillFrontmatter::new("my-skill", "does things");
        assert_eq!(fm.name, "my-skill");
        assert_eq!(fm.description, "does things");
        assert!(fm.allowed_tools.is_empty());
        assert_eq!(fm.context_mode, ContextMode::Fork);
        assert!(!fm.disable_model_invocation);
        assert!(fm.user_invocable);
    }
}
