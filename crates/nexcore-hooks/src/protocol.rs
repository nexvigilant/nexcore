//! Claude Code hook protocol types.
//!
//! This module implements the complete Claude Code hooks specification,
//! enabling hooks to receive rich context and provide structured responses.
//!
//! # Exit Codes
//!
//! - `0`: Success (allow tool execution)
//! - `1`: Warning (allow but show feedback)
//! - `2`: Block (prevent tool execution, stderr shown to Claude)

use serde::{Deserialize, Serialize};

/// All supported hook event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookEvent {
    // ─────────────────────────────────────────────────────────────────────────
    // Claude Code Standard Events
    // ─────────────────────────────────────────────────────────────────────────
    /// Session begins or resumes
    SessionStart,
    /// User submits a prompt
    UserPromptSubmit,
    /// Before tool execution
    PreToolUse,
    /// When permission dialog appears
    PermissionRequest,
    /// After tool succeeds
    PostToolUse,
    /// After tool fails
    PostToolUseFailure,
    /// When spawning a subagent
    SubagentStart,
    /// When subagent finishes
    SubagentStop,
    /// Claude finishes responding
    Stop,
    /// Before context compaction
    PreCompact,
    /// On --init or --maintenance flags
    Setup,
    /// Session terminates
    SessionEnd,
    /// Claude Code sends notifications
    Notification,

    // ─────────────────────────────────────────────────────────────────────────
    // Pharmacovigilance Domain Events (NexVigilant)
    // ─────────────────────────────────────────────────────────────────────────
    /// A new PV case (ICSR) has been created
    CaseCreated,
    /// An existing PV case has been updated
    CaseUpdated,
    /// Signal detection algorithm detected a potential signal
    SignalDetected,
    /// Regulatory submission has been prepared (ICSRs, PSURs, etc.)
    SubmissionPrepared,
    /// Causality assessment completed (Naranjo, WHO-UMC)
    CausalityAssessed,
    /// MedDRA coding completed for adverse event terms
    MedDRACoded,
}

impl HookEvent {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // Claude Code standard events
            "SessionStart" => Some(Self::SessionStart),
            "UserPromptSubmit" => Some(Self::UserPromptSubmit),
            "PreToolUse" => Some(Self::PreToolUse),
            "PermissionRequest" => Some(Self::PermissionRequest),
            "PostToolUse" => Some(Self::PostToolUse),
            "PostToolUseFailure" => Some(Self::PostToolUseFailure),
            "SubagentStart" => Some(Self::SubagentStart),
            "SubagentStop" => Some(Self::SubagentStop),
            "Stop" => Some(Self::Stop),
            "PreCompact" => Some(Self::PreCompact),
            "Setup" => Some(Self::Setup),
            "SessionEnd" => Some(Self::SessionEnd),
            "Notification" => Some(Self::Notification),
            // Pharmacovigilance domain events
            "CaseCreated" => Some(Self::CaseCreated),
            "CaseUpdated" => Some(Self::CaseUpdated),
            "SignalDetected" => Some(Self::SignalDetected),
            "SubmissionPrepared" => Some(Self::SubmissionPrepared),
            "CausalityAssessed" => Some(Self::CausalityAssessed),
            "MedDRACoded" => Some(Self::MedDRACoded),
            _ => None,
        }
    }

    /// Returns true if this is a pharmacovigilance domain event
    pub fn is_pv_event(&self) -> bool {
        matches!(
            self,
            Self::CaseCreated
                | Self::CaseUpdated
                | Self::SignalDetected
                | Self::SubmissionPrepared
                | Self::CausalityAssessed
                | Self::MedDRACoded
        )
    }
}

/// Hook input from Claude Code (received via stdin)
#[derive(Debug, Clone, Deserialize)]
pub struct HookInput {
    /// Unique session identifier
    pub session_id: String,
    /// Current working directory
    pub cwd: String,
    /// Hook event type (e.g., "PreToolUse", "SessionStart")
    pub hook_event_name: String,
    /// Path to conversation JSON transcript
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current permission mode
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Tool being used
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Tool input parameters
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    /// Tool response
    #[serde(default)]
    pub tool_response: Option<serde_json::Value>,
    /// Unique identifier for this tool use
    #[serde(default)]
    pub tool_use_id: Option<String>,
    /// User's prompt text
    #[serde(default)]
    pub prompt: Option<String>,
    /// Notification message text
    #[serde(default)]
    pub message: Option<String>,
    /// Notification type
    #[serde(default)]
    pub notification_type: Option<String>,
    /// True when Claude is already continuing due to a stop hook
    #[serde(default)]
    pub stop_hook_active: Option<bool>,
    /// Unique identifier for the subagent
    #[serde(default)]
    pub agent_id: Option<String>,
    /// Agent type
    #[serde(default)]
    pub agent_type: Option<String>,
    /// Path to subagent's transcript
    #[serde(default)]
    pub agent_transcript_path: Option<String>,
    /// How session started
    #[serde(default)]
    pub source: Option<String>,
    /// Model identifier
    #[serde(default)]
    pub model: Option<String>,
    /// Trigger type
    #[serde(default)]
    pub trigger: Option<String>,
    /// Custom instructions
    #[serde(default)]
    pub custom_instructions: Option<String>,
    /// End reason
    #[serde(default)]
    pub reason: Option<String>,

    // ─────────────────────────────────────────────────────────────────────────
    // Pharmacovigilance Domain Fields (NexVigilant)
    // ─────────────────────────────────────────────────────────────────────────
    /// ICSR case identifier (for CaseCreated/CaseUpdated events)
    #[serde(default)]
    pub case_id: Option<String>,
    /// Signal identifier (for SignalDetected events)
    #[serde(default)]
    pub signal_id: Option<String>,
    /// Drug/product name
    #[serde(default)]
    pub drug_name: Option<String>,
    /// Adverse event term (PT level)
    #[serde(default)]
    pub event_term: Option<String>,
    /// MedDRA preferred term code
    #[serde(default)]
    pub meddra_pt_code: Option<u32>,
    /// Signal detection method (PRR, ROR, IC, EBGM)
    #[serde(default)]
    pub signal_method: Option<String>,
    /// Signal score/statistic value
    #[serde(default)]
    pub signal_score: Option<f64>,
    /// Causality assessment result
    #[serde(default)]
    pub causality_result: Option<String>,
    /// Submission type (ICSR, PSUR, DSUR, etc.)
    #[serde(default)]
    pub submission_type: Option<String>,
}

impl HookInput {
    /// Parse the hook event type
    pub fn event(&self) -> Option<HookEvent> {
        HookEvent::from_str(&self.hook_event_name)
    }
    /// Get the user's prompt text
    pub fn get_prompt(&self) -> Option<&str> {
        self.prompt
            .as_deref()
            .or(self.message.as_deref())
            .filter(|s| !s.is_empty())
    }
    /// Get file path from tool_input
    pub fn get_file_path(&self) -> Option<&str> {
        self.tool_input.as_ref()?.get("file_path")?.as_str()
    }
    /// Get content from tool_input
    pub fn get_content(&self) -> Option<&str> {
        self.tool_input.as_ref()?.get("content")?.as_str()
    }
    /// Get new_string from tool_input
    pub fn get_new_string(&self) -> Option<&str> {
        self.tool_input.as_ref()?.get("new_string")?.as_str()
    }
    /// Get command from tool_input
    pub fn get_command(&self) -> Option<&str> {
        self.tool_input.as_ref()?.get("command")?.as_str()
    }
    /// Get any content being written
    pub fn get_written_content(&self) -> Option<&str> {
        self.get_content().or_else(|| self.get_new_string())
    }
    /// Check if running in plan mode
    pub fn is_plan_mode(&self) -> bool {
        self.permission_mode.as_deref() == Some("plan")
    }
    /// Check if running in bypass permissions mode
    pub fn is_bypass_permissions(&self) -> bool {
        self.permission_mode.as_deref() == Some("bypassPermissions")
    }
    /// Check if stop hook is active
    pub fn is_stop_hook_active(&self) -> bool {
        self.stop_hook_active.unwrap_or(false)
    }
    /// Check if this is a write operation (Write or Edit tool)
    pub fn is_write_tool(&self) -> bool {
        matches!(self.tool_name.as_deref(), Some("Write") | Some("Edit"))
    }
    /// Check if this is a shell command (Bash tool)
    pub fn is_shell_tool(&self) -> bool {
        matches!(self.tool_name.as_deref(), Some("Bash"))
    }
    /// Check if file is a Rust file
    pub fn is_rust_file(&self) -> bool {
        self.get_file_path().is_some_and(|p| p.ends_with(".rs"))
    }
    /// Get tool_use_id or fall back to session_id
    pub fn tool_use_id_or_session(&self) -> &str {
        self.tool_use_id.as_deref().unwrap_or(&self.session_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Pharmacovigilance Helpers
    // ─────────────────────────────────────────────────────────────────────────
    /// Check if this is a PV domain event
    pub fn is_pv_event(&self) -> bool {
        self.event().map(|e| e.is_pv_event()).unwrap_or(false)
    }
    /// Get case ID for PV events
    pub fn get_case_id(&self) -> Option<&str> {
        self.case_id.as_deref()
    }
    /// Get signal ID for SignalDetected events
    pub fn get_signal_id(&self) -> Option<&str> {
        self.signal_id.as_deref()
    }
    /// Get drug name for PV events
    pub fn get_drug_name(&self) -> Option<&str> {
        self.drug_name.as_deref()
    }
    /// Get adverse event term
    pub fn get_event_term(&self) -> Option<&str> {
        self.event_term.as_deref()
    }
    /// Get signal detection method
    pub fn get_signal_method(&self) -> Option<&str> {
        self.signal_method.as_deref()
    }
}

/// Hook output to Claude Code
#[derive(Debug, Clone, Serialize, Default)]
pub struct HookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<HookDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub should_continue: Option<bool>,
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(rename = "suppressOutput", skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(rename = "systemMessage", skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(rename = "hookSpecificOutput", skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

/// Hook decision type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookDecision {
    #[serde(rename = "approve")]
    Allow,
    Block,
    Warn,
}

/// Hook-specific output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    pub hook_event_name: String,
    #[serde(rename = "permissionDecision", skip_serializing_if = "Option::is_none")]
    pub permission_decision: Option<String>,
    #[serde(
        rename = "permissionDecisionReason",
        skip_serializing_if = "Option::is_none"
    )]
    pub permission_decision_reason: Option<String>,
    #[serde(rename = "updatedInput", skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
    #[serde(rename = "additionalContext", skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<PermissionDecision>,
}

/// Permission decision for PermissionRequest hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecision {
    pub behavior: String,
    #[serde(rename = "updatedInput", skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt: Option<bool>,
}

impl HookOutput {
    /// Create a skip response for UserPromptSubmit
    pub fn skip_prompt() -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::user_prompt_submit("")),
            ..Default::default()
        }
    }
    /// Create a skip response for SessionStart
    pub fn skip_session() -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::session_start_context("")),
            ..Default::default()
        }
    }
    /// Create a context injection response for UserPromptSubmit
    pub fn with_context(context: impl Into<String>) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::user_prompt_submit(context)),
            ..Default::default()
        }
    }
    /// Create a context injection response for SessionStart
    pub fn with_session_context(context: impl Into<String>) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::session_start_context(context)),
            ..Default::default()
        }
    }
    /// Create an allow response
    pub fn allow() -> Self {
        Self {
            decision: Some(HookDecision::Allow),
            ..Default::default()
        }
    }
    /// Create a block response
    pub fn block(reason: impl Into<String>) -> Self {
        Self {
            decision: Some(HookDecision::Block),
            reason: Some(reason.into()),
            ..Default::default()
        }
    }
    /// Create a warn response
    pub fn warn(reason: impl Into<String>) -> Self {
        Self {
            decision: Some(HookDecision::Warn),
            reason: Some(reason.into()),
            ..Default::default()
        }
    }
    /// Create a PreToolUse allow response
    pub fn pre_tool_use_allow(reason: impl Into<String>) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::pre_tool_use_allow(reason)),
            ..Default::default()
        }
    }
    /// Create a PreToolUse deny response
    pub fn pre_tool_use_deny(reason: impl Into<String>) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::pre_tool_use_deny(reason)),
            ..Default::default()
        }
    }
    /// Create a PreToolUse allow with modified input
    pub fn pre_tool_use_allow_with_update(
        reason: impl Into<String>,
        updated_input: serde_json::Value,
    ) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::pre_tool_use_allow_with_update(
                reason,
                updated_input,
            )),
            ..Default::default()
        }
    }
    /// Create a PermissionRequest allow response
    pub fn permission_allow() -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::permission_allow()),
            ..Default::default()
        }
    }
    /// Create a PermissionRequest allow with modified input
    pub fn permission_allow_with_update(updated_input: serde_json::Value) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::permission_allow_with_update(
                updated_input,
            )),
            ..Default::default()
        }
    }
    /// Create a PermissionRequest deny response
    pub fn permission_deny(message: impl Into<String>) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput::permission_deny(message)),
            ..Default::default()
        }
    }
    /// Create a Stop block response
    pub fn stop_block(reason: impl Into<String>) -> Self {
        Self {
            decision: Some(HookDecision::Block),
            reason: Some(reason.into()),
            ..Default::default()
        }
    }
    /// Create a response that stops Claude processing
    pub fn stop_claude(reason: impl Into<String>) -> Self {
        Self {
            should_continue: Some(false),
            stop_reason: Some(reason.into()),
            ..Default::default()
        }
    }
    /// Add a system message
    pub fn with_system_message(mut self, message: impl Into<String>) -> Self {
        self.system_message = Some(message.into());
        self
    }
    /// Suppress output from transcript mode
    pub fn suppressed(mut self) -> Self {
        self.suppress_output = Some(true);
        self
    }
    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    /// Print JSON to stdout
    pub fn emit(&self) {
        if let Ok(json) = self.to_json() {
            println!("{json}");
        }
    }
}

impl HookSpecificOutput {
    /// Create UserPromptSubmit context injection
    pub fn user_prompt_submit(context: impl Into<String>) -> Self {
        Self {
            hook_event_name: "UserPromptSubmit".into(),
            additional_context: Some(context.into()),
            permission_decision: None,
            permission_decision_reason: None,
            updated_input: None,
            decision: None,
        }
    }
    /// Create PreToolUse allow
    pub fn pre_tool_use_allow(reason: impl Into<String>) -> Self {
        Self {
            hook_event_name: "PreToolUse".into(),
            permission_decision: Some("allow".into()),
            permission_decision_reason: Some(reason.into()),
            additional_context: None,
            updated_input: None,
            decision: None,
        }
    }
    /// Create PreToolUse deny
    pub fn pre_tool_use_deny(reason: impl Into<String>) -> Self {
        Self {
            hook_event_name: "PreToolUse".into(),
            permission_decision: Some("deny".into()),
            permission_decision_reason: Some(reason.into()),
            additional_context: None,
            updated_input: None,
            decision: None,
        }
    }
    /// Create PreToolUse allow with modified input
    pub fn pre_tool_use_allow_with_update(
        reason: impl Into<String>,
        updated_input: serde_json::Value,
    ) -> Self {
        Self {
            hook_event_name: "PreToolUse".into(),
            permission_decision: Some("allow".into()),
            permission_decision_reason: Some(reason.into()),
            updated_input: Some(updated_input),
            additional_context: None,
            decision: None,
        }
    }
    /// Create PreToolUse with context
    pub fn pre_tool_use_with_context(context: impl Into<String>) -> Self {
        Self {
            hook_event_name: "PreToolUse".into(),
            additional_context: Some(context.into()),
            permission_decision: None,
            permission_decision_reason: None,
            updated_input: None,
            decision: None,
        }
    }
    /// Create PermissionRequest allow
    pub fn permission_allow() -> Self {
        Self {
            hook_event_name: "PermissionRequest".into(),
            decision: Some(PermissionDecision::allow()),
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            updated_input: None,
        }
    }
    /// Create PermissionRequest allow with modified input
    pub fn permission_allow_with_update(updated_input: serde_json::Value) -> Self {
        Self {
            hook_event_name: "PermissionRequest".into(),
            decision: Some(PermissionDecision::allow_with_input(updated_input)),
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            updated_input: None,
        }
    }
    /// Create PermissionRequest deny
    pub fn permission_deny(message: impl Into<String>) -> Self {
        Self {
            hook_event_name: "PermissionRequest".into(),
            decision: Some(PermissionDecision::deny(message)),
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            updated_input: None,
        }
    }
    /// Create PermissionRequest deny with interrupt
    pub fn permission_deny_interrupt(message: impl Into<String>) -> Self {
        Self {
            hook_event_name: "PermissionRequest".into(),
            decision: Some(PermissionDecision::deny_interrupt(message)),
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            updated_input: None,
        }
    }
    /// Create PostToolUse context
    pub fn post_tool_use_context(context: impl Into<String>) -> Self {
        Self {
            hook_event_name: "PostToolUse".into(),
            additional_context: Some(context.into()),
            permission_decision: None,
            permission_decision_reason: None,
            updated_input: None,
            decision: None,
        }
    }
    /// Create SessionStart context
    pub fn session_start_context(context: impl Into<String>) -> Self {
        Self {
            hook_event_name: "SessionStart".into(),
            additional_context: Some(context.into()),
            permission_decision: None,
            permission_decision_reason: None,
            updated_input: None,
            decision: None,
        }
    }
    /// Create Setup context
    pub fn setup_context(context: impl Into<String>) -> Self {
        Self {
            hook_event_name: "Setup".into(),
            additional_context: Some(context.into()),
            permission_decision: None,
            permission_decision_reason: None,
            updated_input: None,
            decision: None,
        }
    }
}

impl PermissionDecision {
    /// Create an allow decision
    pub fn allow() -> Self {
        Self {
            behavior: "allow".into(),
            updated_input: None,
            message: None,
            interrupt: None,
        }
    }
    /// Create an allow decision with modified input
    pub fn allow_with_input(updated_input: serde_json::Value) -> Self {
        Self {
            behavior: "allow".into(),
            updated_input: Some(updated_input),
            message: None,
            interrupt: None,
        }
    }
    /// Create a deny decision
    pub fn deny(message: impl Into<String>) -> Self {
        Self {
            behavior: "deny".into(),
            updated_input: None,
            message: Some(message.into()),
            interrupt: None,
        }
    }
    /// Create a deny decision that interrupts Claude
    pub fn deny_interrupt(message: impl Into<String>) -> Self {
        Self {
            behavior: "deny".into(),
            updated_input: None,
            message: Some(message.into()),
            interrupt: Some(true),
        }
    }
}

// =============================================================================
// Hook Orchestration Types
// =============================================================================

/// Decision type for hook aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Warn,
    Block,
}

impl Decision {
    /// Get the severity ordering (higher = more severe)
    pub fn severity(&self) -> u8 {
        match self {
            Self::Allow => 0,
            Self::Warn => 1,
            Self::Block => 2,
        }
    }

    /// Combine two decisions, taking the more severe one
    pub fn combine(self, other: Self) -> Self {
        if self.severity() >= other.severity() {
            self
        } else {
            other
        }
    }
}

impl Default for Decision {
    fn default() -> Self {
        Self::Allow
    }
}

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// Should this severity level block the operation?
    pub fn should_block(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Should this severity level warn?
    pub fn should_warn(&self) -> bool {
        matches!(self, Self::Medium | Self::Low)
    }
}

impl Default for Severity {
    fn default() -> Self {
        Self::Info
    }
}

/// Location within a file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Location {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl Location {
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line: None,
            column: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

/// A finding/evidence from a hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub message: String,
    pub severity: Severity,
    pub location: Location,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl Finding {
    pub fn new(message: impl Into<String>, severity: Severity, location: Location) -> Self {
        Self {
            message: message.into(),
            severity,
            location,
            code: None,
            suggestion: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Programming language detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    C,
    Cpp,
    Toml,
    Yaml,
    Json,
    Markdown,
    Shell,
    #[default]
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".rs") {
            Self::Rust
        } else if path.ends_with(".py") {
            Self::Python
        } else if path.ends_with(".ts") || path.ends_with(".tsx") {
            Self::TypeScript
        } else if path.ends_with(".js") || path.ends_with(".jsx") {
            Self::JavaScript
        } else if path.ends_with(".go") {
            Self::Go
        } else if path.ends_with(".c") || path.ends_with(".h") {
            Self::C
        } else if path.ends_with(".cpp") || path.ends_with(".hpp") || path.ends_with(".cc") {
            Self::Cpp
        } else if path.ends_with(".toml") {
            Self::Toml
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Self::Yaml
        } else if path.ends_with(".json") {
            Self::Json
        } else if path.ends_with(".md") {
            Self::Markdown
        } else if path.ends_with(".sh") || path.ends_with(".bash") {
            Self::Shell
        } else {
            Self::Unknown
        }
    }
}

/// Shared file context built once and used by all hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// Detected language
    pub language: Language,
    /// Is this a Rust file?
    pub is_rust: bool,
    /// Is this a Cargo.toml file?
    pub is_cargo: bool,
    /// Is this a test file?
    pub is_test: bool,
    /// Number of lines
    pub line_count: usize,
    /// SHA256 hash of content for cache validation
    pub content_hash: String,
    /// Tool use ID this context belongs to
    pub tool_use_id: String,
    /// Timestamp when context was built
    pub created_at: f64,
}

impl FileContext {
    /// Build file context from path and content
    pub fn build(path: &str, content: &str, tool_use_id: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let language = Language::from_path(path);
        let is_rust = language == Language::Rust;
        let is_cargo = path.ends_with("Cargo.toml");
        let is_test = path.contains("/tests/")
            || path.contains("/test/")
            || path.contains("_test.rs")
            || path.contains("/benches/")
            || path.starts_with("tests/")
            || path.starts_with("benches/");
        let line_count = content.lines().count();
        let content_hash = Self::hash_content(content);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            path: path.to_string(),
            content: content.to_string(),
            language,
            is_rust,
            is_cargo,
            is_test,
            line_count,
            content_hash,
            tool_use_id: tool_use_id.to_string(),
            created_at,
        }
    }

    /// Compute SHA256 hash of content (first 16 hex chars)
    fn hash_content(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Get the context file path for a given tool_use_id
    pub fn context_file_path(tool_use_id: &str) -> std::path::PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("nexcore-hooks")
            .join(format!("ctx_{}.json", tool_use_id))
    }

    /// Save context to file
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::context_file_path(&self.tool_use_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }

    /// Load context from file
    pub fn load(tool_use_id: &str) -> Option<Self> {
        let path = Self::context_file_path(tool_use_id);
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

/// Hook group for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookGroup {
    Security,
    Quality,
    Safety,
    Dependencies,
    Policy,
    Context,
    Aggregation,
}

impl HookGroup {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Security => "Security",
            Self::Quality => "Quality",
            Self::Safety => "Safety",
            Self::Dependencies => "Dependencies",
            Self::Policy => "Policy",
            Self::Context => "Context",
            Self::Aggregation => "Aggregation",
        }
    }
}

/// A single hook's result (stored in the registry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Hook name (binary name)
    pub hook_name: String,
    /// Hook group
    pub group: HookGroup,
    /// Decision made
    pub decision: Decision,
    /// Findings from this hook
    pub findings: Vec<Finding>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp when hook completed
    pub completed_at: f64,
}

impl HookResult {
    pub fn new(hook_name: impl Into<String>, group: HookGroup) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        Self {
            hook_name: hook_name.into(),
            group,
            decision: Decision::Allow,
            findings: Vec::new(),
            duration_ms: 0,
            completed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
        }
    }

    pub fn with_decision(mut self, decision: Decision) -> Self {
        self.decision = decision;
        self
    }

    pub fn with_findings(mut self, findings: Vec<Finding>) -> Self {
        self.findings = findings;
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }
}

/// Aggregated result from all hooks in a tool use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    /// Final decision (most severe wins)
    pub decision: Decision,
    /// Results grouped by hook group
    pub by_group: std::collections::HashMap<String, GroupResult>,
    /// Total findings across all hooks
    pub total_findings: usize,
    /// Count of critical findings
    pub critical_count: usize,
    /// Count of high findings
    pub high_count: usize,
    /// Total hook execution time
    pub total_duration_ms: u64,
    /// Tool use ID
    pub tool_use_id: String,
}

/// Results for a single hook group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResult {
    /// Group name
    pub group_name: String,
    /// Aggregated decision for this group
    pub decision: Decision,
    /// All findings from hooks in this group
    pub findings: Vec<Finding>,
    /// Hook names in this group
    pub hooks: Vec<String>,
}

impl AggregatedResult {
    /// Create empty aggregated result
    pub fn new(tool_use_id: impl Into<String>) -> Self {
        Self {
            decision: Decision::Allow,
            by_group: std::collections::HashMap::new(),
            total_findings: 0,
            critical_count: 0,
            high_count: 0,
            total_duration_ms: 0,
            tool_use_id: tool_use_id.into(),
        }
    }

    /// Aggregate multiple hook results
    pub fn from_results(tool_use_id: &str, results: &[HookResult]) -> Self {
        let mut aggregated = Self::new(tool_use_id);

        for result in results {
            // Combine decisions
            aggregated.decision = aggregated.decision.combine(result.decision);
            aggregated.total_duration_ms += result.duration_ms;

            // Count findings by severity
            for finding in &result.findings {
                aggregated.total_findings += 1;
                match finding.severity {
                    Severity::Critical => aggregated.critical_count += 1,
                    Severity::High => aggregated.high_count += 1,
                    _ => {}
                }
            }

            // Group results
            let group_name = result.group.display_name().to_string();
            let group_result = aggregated
                .by_group
                .entry(group_name.clone())
                .or_insert_with(|| GroupResult {
                    group_name: group_name.clone(),
                    decision: Decision::Allow,
                    findings: Vec::new(),
                    hooks: Vec::new(),
                });

            group_result.decision = group_result.decision.combine(result.decision);
            group_result.findings.extend(result.findings.clone());
            group_result.hooks.push(result.hook_name.clone());
        }

        aggregated
    }

    /// Format as a human-readable report
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        // Header based on decision
        match self.decision {
            Decision::Block => {
                report.push_str("🚫 BLOCKED: Hook checks failed\n\n");
            }
            Decision::Warn => {
                report.push_str("⚠️ WARNINGS: Issues detected\n\n");
            }
            Decision::Allow => {
                report.push_str("✅ PASSED: All checks passed\n\n");
            }
        }

        // Summary
        report.push_str(&format!(
            "Findings: {} total ({} critical, {} high)\n",
            self.total_findings, self.critical_count, self.high_count
        ));
        report.push_str(&format!("Duration: {}ms\n\n", self.total_duration_ms));

        // Group details (only for non-Allow decisions)
        if self.decision != Decision::Allow {
            for (group_name, group_result) in &self.by_group {
                if group_result.decision == Decision::Allow {
                    continue;
                }

                let icon = match group_result.decision {
                    Decision::Block => "🚫",
                    Decision::Warn => "⚠️",
                    Decision::Allow => "✅",
                };

                report.push_str(&format!("{} {}\n", icon, group_name));

                for finding in &group_result.findings {
                    let sev = match finding.severity {
                        Severity::Critical => "[CRITICAL]",
                        Severity::High => "[HIGH]",
                        Severity::Medium => "[MEDIUM]",
                        Severity::Low => "[LOW]",
                        Severity::Info => "[INFO]",
                    };
                    report.push_str(&format!("  {} {}\n", sev, finding.message));

                    if let Some(line) = finding.location.line {
                        report.push_str(&format!("    at {}:{}\n", finding.location.file, line));
                    }

                    if let Some(suggestion) = &finding.suggestion {
                        report.push_str(&format!("    💡 {}\n", suggestion));
                    }
                }
                report.push('\n');
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_parsing() {
        assert_eq!(
            HookEvent::from_str("SessionStart"),
            Some(HookEvent::SessionStart)
        );
        assert_eq!(HookEvent::from_str("Invalid"), None);
    }

    #[test]
    fn test_pv_event_parsing() {
        assert_eq!(
            HookEvent::from_str("CaseCreated"),
            Some(HookEvent::CaseCreated)
        );
        assert_eq!(
            HookEvent::from_str("SignalDetected"),
            Some(HookEvent::SignalDetected)
        );
        assert_eq!(
            HookEvent::from_str("CausalityAssessed"),
            Some(HookEvent::CausalityAssessed)
        );
        assert_eq!(
            HookEvent::from_str("MedDRACoded"),
            Some(HookEvent::MedDRACoded)
        );
    }

    #[test]
    fn test_is_pv_event() {
        assert!(HookEvent::SignalDetected.is_pv_event());
        assert!(HookEvent::CaseCreated.is_pv_event());
        assert!(!HookEvent::PreToolUse.is_pv_event());
        assert!(!HookEvent::SessionStart.is_pv_event());
    }

    #[test]
    fn test_hook_input_deserialization() {
        let json = r#"{"session_id":"abc","cwd":"/","hook_event_name":"PreToolUse","tool_input":{"command":"test"}}"#;
        if let Ok(input) = serde_json::from_str::<HookInput>(json) {
            assert_eq!(input.session_id, "abc");
            assert_eq!(input.get_command(), Some("test"));
        }
    }

    #[test]
    fn test_hook_output_serialization() {
        let output = HookOutput::pre_tool_use_allow("ok");
        if let Ok(json) = output.to_json() {
            assert!(json.contains("PreToolUse"));
        }
    }

    #[test]
    fn test_permission_decision() {
        let allow = PermissionDecision::allow();
        assert_eq!(allow.behavior, "allow");
        let deny = PermissionDecision::deny_interrupt("no");
        assert_eq!(deny.interrupt, Some(true));
    }

    // =========================================================================
    // Hook Orchestration Tests
    // =========================================================================

    #[test]
    fn test_decision_combine() {
        assert_eq!(Decision::Allow.combine(Decision::Allow), Decision::Allow);
        assert_eq!(Decision::Allow.combine(Decision::Warn), Decision::Warn);
        assert_eq!(Decision::Allow.combine(Decision::Block), Decision::Block);
        assert_eq!(Decision::Warn.combine(Decision::Allow), Decision::Warn);
        assert_eq!(Decision::Warn.combine(Decision::Block), Decision::Block);
        assert_eq!(Decision::Block.combine(Decision::Allow), Decision::Block);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }

    #[test]
    fn test_severity_should_block() {
        assert!(Severity::Critical.should_block());
        assert!(Severity::High.should_block());
        assert!(!Severity::Medium.should_block());
        assert!(!Severity::Low.should_block());
        assert!(!Severity::Info.should_block());
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_path("src/main.rs"), Language::Rust);
        assert_eq!(Language::from_path("script.py"), Language::Python);
        assert_eq!(Language::from_path("app.tsx"), Language::TypeScript);
        assert_eq!(Language::from_path("Cargo.toml"), Language::Toml);
        assert_eq!(Language::from_path("config.yaml"), Language::Yaml);
        assert_eq!(Language::from_path("data.json"), Language::Json);
        assert_eq!(Language::from_path("README.md"), Language::Markdown);
        assert_eq!(Language::from_path("unknown.xyz"), Language::Unknown);
    }

    #[test]
    fn test_file_context_build() {
        let ctx = FileContext::build("src/lib.rs", "fn main() {}\n", "test-id-123");
        assert_eq!(ctx.path, "src/lib.rs");
        assert_eq!(ctx.language, Language::Rust);
        assert!(ctx.is_rust);
        assert!(!ctx.is_cargo);
        assert!(!ctx.is_test);
        assert_eq!(ctx.line_count, 1);
        assert_eq!(ctx.tool_use_id, "test-id-123");
        assert!(!ctx.content_hash.is_empty());
    }

    #[test]
    fn test_file_context_test_detection() {
        assert!(FileContext::build("tests/unit.rs", "", "id").is_test);
        assert!(FileContext::build("src/lib_test.rs", "", "id").is_test);
        assert!(FileContext::build("benches/bench.rs", "", "id").is_test);
        assert!(!FileContext::build("src/lib.rs", "", "id").is_test);
    }

    #[test]
    fn test_hook_result_builder() {
        let result = HookResult::new("secret_scanner", HookGroup::Security)
            .with_decision(Decision::Block)
            .with_duration(42);

        assert_eq!(result.hook_name, "secret_scanner");
        assert_eq!(result.group, HookGroup::Security);
        assert_eq!(result.decision, Decision::Block);
        assert_eq!(result.duration_ms, 42);
    }

    #[test]
    fn test_aggregated_result() {
        let results = vec![
            HookResult::new("secret_scanner", HookGroup::Security)
                .with_decision(Decision::Warn)
                .with_duration(10),
            HookResult::new("unsafe_gatekeeper", HookGroup::Security)
                .with_decision(Decision::Block)
                .with_findings(vec![Finding::new(
                    "Unsafe block without SAFETY comment",
                    Severity::High,
                    Location::new("src/lib.rs").with_line(42),
                )])
                .with_duration(5),
            HookResult::new("clone_detector", HookGroup::Quality)
                .with_decision(Decision::Allow)
                .with_duration(8),
        ];

        let agg = AggregatedResult::from_results("tool-123", &results);

        assert_eq!(agg.decision, Decision::Block);
        assert_eq!(agg.total_findings, 1);
        assert_eq!(agg.high_count, 1);
        assert_eq!(agg.total_duration_ms, 23);
        assert!(agg.by_group.contains_key("Security"));
        assert!(agg.by_group.contains_key("Quality"));
    }

    #[test]
    fn test_finding_builder() {
        let finding = Finding::new(
            "Secret detected",
            Severity::Critical,
            Location::new("config.yaml").with_line(10).with_column(5),
        )
        .with_code("api_key")
        .with_suggestion("Use environment variable instead");

        assert_eq!(finding.message, "Secret detected");
        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.location.file, "config.yaml");
        assert_eq!(finding.location.line, Some(10));
        assert_eq!(finding.location.column, Some(5));
        assert_eq!(finding.code, Some("api_key".to_string()));
        assert!(finding.suggestion.is_some());
    }
}
