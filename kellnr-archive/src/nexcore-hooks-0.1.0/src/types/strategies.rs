//! Proptest strategies for generating valid and invalid type instances.
//!
//! # Design Philosophy
//!
//! Each type has strategy categories:
//! 1. **Valid** — Generates instances that pass validation
//! 2. **Invalid** — Generates instances that fail validation
//!
//! # Usage
//!
//! ```rust,ignore
//! use proptest::prelude::*;
//! use claude_hooks::types::strategies::*;
//!
//! proptest! {
//!     #[test]
//!     fn valid_tool_names_parse(name in valid_tool_name()) {
//!         prop_assert!(ToolName::new(&name).is_ok());
//!     }
//! }
//! ```

// Allow expect() in test-only strategies module - these are static regexes known to be valid.
#![allow(clippy::expect_used)]

use proptest::prelude::*;

// ════════════════════════════════════════════════════════════════════════════
// STRING STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for non-empty strings (1-100 chars, printable ASCII for simplicity).
pub fn non_empty_string() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zA-Z0-9 ]{1,100}").expect("valid regex")
}

/// Strategy for empty string (invalid for NonEmptyString).
pub fn empty_string() -> impl Strategy<Value = String> {
    Just(String::new())
}

/// Strategy for bounded strings within limit.
pub fn bounded_string(max_len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::range('a', 'z'), 0..=max_len)
        .prop_map(|chars| chars.into_iter().collect())
}

/// Strategy for strings exceeding bound (invalid).
pub fn oversized_string(min_len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::range('a', 'z'), min_len..min_len + 50)
        .prop_map(|chars| chars.into_iter().collect())
}

// ════════════════════════════════════════════════════════════════════════════
// TOOL NAME STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for valid standard tool names (PascalCase).
/// Must start with uppercase, contain at least one lowercase, all alphabetic.
pub fn valid_standard_tool_name() -> impl Strategy<Value = String> {
    // First char uppercase A-Z, then at least one lowercase, rest alphabetic
    proptest::string::string_regex("[A-Z][a-z][a-zA-Z]{0,14}").expect("valid regex")
}

/// Strategy for valid MCP tool names (mcp__server__tool format).
/// Server/tool parts: start with lowercase, then lowercase/digits, no trailing underscore.
pub fn valid_mcp_tool_name() -> impl Strategy<Value = String> {
    (
        proptest::string::string_regex("[a-z][a-z0-9]{0,10}").expect("valid regex"),
        proptest::string::string_regex("[a-z][a-z0-9]{0,5}(_[a-z0-9]+)?").expect("valid regex"),
    )
        .prop_map(|(server, tool)| format!("mcp__{}__{}", server, tool))
}

/// Strategy for any valid tool name.
pub fn valid_tool_name() -> impl Strategy<Value = String> {
    prop_oneof![valid_standard_tool_name(), valid_mcp_tool_name(),]
}

/// Strategy for invalid tool names.
pub fn invalid_tool_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),                                                 // Empty
        proptest::string::string_regex("[a-z]{1,10}").expect("valid regex"), // All lowercase
        proptest::string::string_regex("[A-Z]{2,10}").expect("valid regex"), // All uppercase after first
        Just("Web_Fetch".to_string()),                                       // Contains underscore
        Just("mcp_memory_create".to_string()), // Single underscore MCP
        Just("MCP__memory__create".to_string()), // Uppercase MCP prefix
        Just("123Tool".to_string()),           // Starts with digit
        Just("A".to_string()),                 // Single uppercase (no lowercase)
        Just("BASH".to_string()),              // All uppercase
    ]
}

// ════════════════════════════════════════════════════════════════════════════
// HOOK EVENT STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for valid hook event names.
pub fn valid_hook_event_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("PreToolUse".to_string()),
        Just("PostToolUse".to_string()),
        Just("PostToolUseFailure".to_string()),
        Just("PermissionRequest".to_string()),
        Just("UserPromptSubmit".to_string()),
        Just("Stop".to_string()),
        Just("SubagentStop".to_string()),
        Just("SubagentStart".to_string()),
        Just("SessionStart".to_string()),
        Just("SessionEnd".to_string()),
        Just("Setup".to_string()),
        Just("Notification".to_string()),
        Just("PreCompact".to_string()),
    ]
}

/// Strategy for invalid hook event names.
pub fn invalid_hook_event_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("preToolUse".to_string()),   // Wrong case
        Just("PRE_TOOL_USE".to_string()), // Snake case
        Just("UnknownEvent".to_string()), // Not a real event
        proptest::string::string_regex("[a-z]{5,20}").expect("valid regex"),
    ]
}

// ════════════════════════════════════════════════════════════════════════════
// TIMEOUT STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for valid timeout values (1-3600).
pub fn valid_timeout() -> impl Strategy<Value = u16> {
    1u16..=3600u16
}

/// Strategy for invalid timeout values.
pub fn invalid_timeout() -> impl Strategy<Value = u16> {
    prop_oneof![
        Just(0u16),         // Below minimum
        3601u16..=u16::MAX, // Above maximum
    ]
}

// ════════════════════════════════════════════════════════════════════════════
// PATH STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for valid relative paths.
pub fn valid_relative_path() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        proptest::string::string_regex("[a-zA-Z0-9_.-]{1,20}").expect("valid regex"),
        1..5,
    )
    .prop_map(|segments| segments.join("/"))
}

/// Strategy for valid absolute paths.
pub fn valid_absolute_path() -> impl Strategy<Value = String> {
    valid_relative_path().prop_map(|p| format!("/{}", p))
}

/// Strategy for path traversal attempts (invalid for ProjectPath).
pub fn traversal_path() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("../../../etc/passwd".to_string()),
        Just("..".to_string()),
        Just("foo/../../bar".to_string()),
    ]
}

/// Strategy for absolute paths outside project (invalid for ProjectPath).
pub fn absolute_escape_path() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/etc/passwd".to_string()),
        Just("/absolute/path".to_string()),
        Just("/tmp/outside".to_string()),
    ]
}

/// Strategy for empty path (invalid).
pub fn empty_path() -> impl Strategy<Value = String> {
    Just(String::new())
}

// ════════════════════════════════════════════════════════════════════════════
// SESSION ID STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for valid session IDs (non-empty).
pub fn valid_session_id() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}")
        .expect("valid regex")
}

/// Strategy for invalid session IDs (empty).
pub fn invalid_session_id() -> impl Strategy<Value = String> {
    Just(String::new())
}

// ════════════════════════════════════════════════════════════════════════════
// JSON INPUT STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for malformed JSON (invalid).
pub fn malformed_json() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("{".to_string()),               // Unclosed brace
        Just("}{".to_string()),              // Inverted braces
        Just("{\"key\":}".to_string()),      // Missing value
        Just("not json at all".to_string()), // Plain text
        Just(String::new()),                 // Empty
        Just("[1,2,".to_string()),           // Unclosed array
    ]
}

/// Strategy for valid JSON but wrong schema.
pub fn wrong_schema_json() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r#"{"wrong": "schema"}"#.to_string()),
        Just(r#"{"session_id": 123}"#.to_string()), // Wrong type
        Just("[]".to_string()),                     // Array not object
        Just("null".to_string()),                   // Null
    ]
}

// ════════════════════════════════════════════════════════════════════════════
// COMPOSITE STRATEGIES
// ════════════════════════════════════════════════════════════════════════════

/// Strategy for arbitrary bytes (for fuzz-like testing).
pub fn arbitrary_bytes() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 0..1000)
}

/// Strategy for arbitrary UTF-8 strings.
pub fn arbitrary_string() -> impl Strategy<Value = String> {
    ".*"
}
