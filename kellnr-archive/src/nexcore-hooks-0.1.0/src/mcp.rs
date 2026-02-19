//! MCP (Model Context Protocol) tool support.
//!
//! This module provides validated types and pattern matching for MCP tools
//! in Claude Code hooks.
//!
//! # MCP Tool Name Format
//!
//! MCP tools follow the naming convention: `mcp__<server>__<tool>`
//!
//! - `server`: lowercase alphanumeric with underscores/hyphens (e.g., `memory`, `github-api`)
//! - `tool`: lowercase alphanumeric with underscores (e.g., `create_entities`, `search`)
//!
//! # Type Hierarchy
//!
//! ```text
//! McpServer          — Validated server identifier
//! McpTool            — Validated tool identifier
//! McpToolName        — Full mcp__server__tool name
//! McpMatcher         — Pattern matching for MCP tools
//! McpValidator       — Allow/block list validator
//! McpServerCategory  — Server categorization
//! ```
//!
//! # Examples
//!
//! ## Parsing MCP Tool Names
//!
//! ```rust
//! use claude_hooks::mcp::McpToolName;
//!
//! let name = McpToolName::parse("mcp__memory__save").unwrap();
//! assert_eq!(name.server().as_str(), "memory");
//! assert_eq!(name.tool().as_str(), "save");
//! ```
//!
//! ## Pattern Matching
//!
//! ```rust
//! use claude_hooks::mcp::{McpToolName, McpMatcher};
//!
//! // Match all tools from memory server
//! let matcher = McpMatcher::server("memory").unwrap();
//! let name = McpToolName::parse("mcp__memory__save").unwrap();
//! assert!(matcher.matches(&name));
//! ```

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{HookError, HookResult};

// ════════════════════════════════════════════════════════════════════════════
// NEWTYPES
// ════════════════════════════════════════════════════════════════════════════

/// Validated MCP server identifier.
///
/// # Invariants
///
/// - Non-empty
/// - Contains only valid identifier characters: `[a-z0-9_-]`
/// - No leading/trailing whitespace
///
/// # Examples
///
/// ```rust
/// use claude_hooks::mcp::McpServer;
///
/// let server = McpServer::new("memory").unwrap();
/// assert_eq!(server.as_str(), "memory");
///
/// // Hyphens allowed
/// let server = McpServer::new("github-api").unwrap();
/// assert_eq!(server.as_str(), "github-api");
///
/// // Invalid: contains uppercase
/// assert!(McpServer::new("Memory").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct McpServer(String);

impl McpServer {
    /// Create a new validated MCP server identifier.
    pub fn new(s: impl AsRef<str>) -> HookResult<Self> {
        let s = s.as_ref().trim();
        if s.is_empty() {
            return Err(HookError::ValidationFailed(
                "MCP server name cannot be empty".into(),
            ));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        {
            return Err(HookError::ValidationFailed(format!(
                "MCP server '{}' contains invalid characters (allowed: a-z, 0-9, _, -)",
                s
            )));
        }
        Ok(Self(s.to_string()))
    }

    /// Access inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for McpServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for McpServer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        McpServer::new(&s).map_err(serde::de::Error::custom)
    }
}

/// Validated MCP tool identifier.
///
/// # Invariants
///
/// - Non-empty
/// - Contains only valid identifier characters: `[a-z0-9_]`
/// - No leading/trailing whitespace
///
/// # Examples
///
/// ```rust
/// use claude_hooks::mcp::McpTool;
///
/// let tool = McpTool::new("create_entities").unwrap();
/// assert_eq!(tool.as_str(), "create_entities");
///
/// // Invalid: contains hyphen (not allowed in tool names)
/// assert!(McpTool::new("create-entities").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct McpTool(String);

impl McpTool {
    /// Create a new validated MCP tool identifier.
    pub fn new(s: impl AsRef<str>) -> HookResult<Self> {
        let s = s.as_ref().trim();
        if s.is_empty() {
            return Err(HookError::ValidationFailed(
                "MCP tool name cannot be empty".into(),
            ));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(HookError::ValidationFailed(format!(
                "MCP tool '{}' contains invalid characters (allowed: a-z, 0-9, _)",
                s
            )));
        }
        Ok(Self(s.to_string()))
    }

    /// Access inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for McpTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for McpTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        McpTool::new(&s).map_err(serde::de::Error::custom)
    }
}

// ════════════════════════════════════════════════════════════════════════════
// MCP TOOL NAME
// ════════════════════════════════════════════════════════════════════════════

/// Full MCP tool name: `mcp__<server>__<tool>`
///
/// This type represents a validated MCP tool name with its server and tool
/// components accessible as validated newtypes.
///
/// # Examples
///
/// ```rust
/// use claude_hooks::mcp::McpToolName;
///
/// let name = McpToolName::parse("mcp__memory__create_entities").unwrap();
/// assert_eq!(name.server().as_str(), "memory");
/// assert_eq!(name.tool().as_str(), "create_entities");
/// assert_eq!(name.to_string(), "mcp__memory__create_entities");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct McpToolName {
    server: McpServer,
    tool: McpTool,
}

impl McpToolName {
    /// MCP tool name prefix.
    pub const PREFIX: &'static str = "mcp__";
    /// MCP tool name separator.
    pub const SEPARATOR: &'static str = "__";

    /// Parse from full tool name string.
    ///
    /// # Errors
    ///
    /// Returns error if the string doesn't match the MCP format.
    pub fn parse(s: impl AsRef<str>) -> HookResult<Self> {
        let s = s.as_ref().trim();
        if !s.starts_with(Self::PREFIX) {
            return Err(HookError::ValidationFailed(format!(
                "MCP tool name must start with '{}', got '{}'",
                Self::PREFIX,
                s
            )));
        }

        let rest = &s[Self::PREFIX.len()..];
        let parts: Vec<&str> = rest.splitn(2, Self::SEPARATOR).collect();

        if parts.len() != 2 {
            return Err(HookError::ValidationFailed(format!(
                "MCP tool name must have format 'mcp__<server>__<tool>', got '{}'",
                s
            )));
        }

        let server_str = parts.first().ok_or_else(|| {
            HookError::ValidationFailed("MCP tool name missing server component".into())
        })?;
        let tool_str = parts.get(1).ok_or_else(|| {
            HookError::ValidationFailed("MCP tool name missing tool component".into())
        })?;
        let server = McpServer::new(*server_str)?;
        let tool = McpTool::new(*tool_str)?;

        Ok(Self { server, tool })
    }

    /// Try to parse, returning None if not an MCP tool.
    ///
    /// Use this when you want to check if something is an MCP tool without
    /// treating non-MCP tools as errors.
    #[must_use]
    pub fn try_parse(s: impl AsRef<str>) -> Option<Self> {
        Self::parse(s).ok()
    }

    /// Construct from validated components.
    #[must_use]
    pub fn from_parts(server: McpServer, tool: McpTool) -> Self {
        Self { server, tool }
    }

    /// Check if a string is an MCP tool name.
    #[must_use]
    pub fn is_mcp(s: &str) -> bool {
        s.starts_with(Self::PREFIX) && s.matches(Self::SEPARATOR).count() >= 2
    }

    /// Get the server component.
    #[inline]
    #[must_use]
    pub fn server(&self) -> &McpServer {
        &self.server
    }

    /// Get the tool component.
    #[inline]
    #[must_use]
    pub fn tool(&self) -> &McpTool {
        &self.tool
    }

    /// Check if this tool is from a specific server.
    #[must_use]
    pub fn is_from_server(&self, server: &McpServer) -> bool {
        &self.server == server
    }

    /// Reconstruct the full tool name string.
    #[must_use]
    pub fn full_name(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for McpToolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            Self::PREFIX,
            self.server.as_str(),
            Self::SEPARATOR,
            self.tool.as_str()
        )
    }
}

impl Serialize for McpToolName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for McpToolName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        McpToolName::parse(&s).map_err(serde::de::Error::custom)
    }
}

// ════════════════════════════════════════════════════════════════════════════
// MCP MATCHER
// ════════════════════════════════════════════════════════════════════════════

/// Pattern matcher for MCP tools with glob-style wildcards.
///
/// # Pattern Syntax
///
/// | Pattern | Matches |
/// |---------|---------|
/// | `*` | All MCP tools |
/// | `mcp__memory__*` | All tools from memory server |
/// | `mcp__memory__save` | Exact match |
/// | `mcp__*__create_*` | Tools starting with `create_` on any server |
/// | `mcp__memory__create_*` | Tools starting with `create_` on memory server |
///
/// # Examples
///
/// ```rust
/// use claude_hooks::mcp::{McpMatcher, McpToolName};
///
/// // Match all tools from memory server
/// let matcher = McpMatcher::server("memory").unwrap();
/// assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
/// assert!(!matcher.matches(&McpToolName::parse("mcp__github__search").unwrap()));
///
/// // Match any create_* tool
/// let matcher = McpMatcher::any_server_tool_prefix("create_");
/// assert!(matcher.matches(&McpToolName::parse("mcp__memory__create_entities").unwrap()));
/// assert!(matcher.matches(&McpToolName::parse("mcp__github__create_issue").unwrap()));
/// ```
#[derive(Debug, Clone)]
pub enum McpMatcher {
    /// Match all MCP tools.
    All,
    /// Match all tools from a specific server.
    Server(McpServer),
    /// Match exact server and tool.
    Exact(McpServer, McpTool),
    /// Match server with tool name prefix.
    ServerToolPrefix {
        /// Server to match.
        server: McpServer,
        /// Tool name prefix.
        tool_prefix: String,
    },
    /// Match any server with tool name prefix.
    AnyServerToolPrefix {
        /// Tool name prefix.
        tool_prefix: String,
    },
    /// Match any server with exact tool name.
    AnyServerTool(McpTool),
}

impl McpMatcher {
    /// Match all MCP tools.
    #[must_use]
    pub fn all() -> Self {
        Self::All
    }

    /// Match all tools from a specific server.
    pub fn server(name: impl AsRef<str>) -> HookResult<Self> {
        Ok(Self::Server(McpServer::new(name)?))
    }

    /// Match exact server and tool.
    pub fn exact(server: impl AsRef<str>, tool: impl AsRef<str>) -> HookResult<Self> {
        Ok(Self::Exact(McpServer::new(server)?, McpTool::new(tool)?))
    }

    /// Match server with tool name prefix.
    pub fn server_tool_prefix(
        server: impl AsRef<str>,
        prefix: impl Into<String>,
    ) -> HookResult<Self> {
        Ok(Self::ServerToolPrefix {
            server: McpServer::new(server)?,
            tool_prefix: prefix.into(),
        })
    }

    /// Match any server with tool name prefix.
    #[must_use]
    pub fn any_server_tool_prefix(prefix: impl Into<String>) -> Self {
        Self::AnyServerToolPrefix {
            tool_prefix: prefix.into(),
        }
    }

    /// Match any server with specific tool.
    pub fn any_server_tool(tool: impl AsRef<str>) -> HookResult<Self> {
        Ok(Self::AnyServerTool(McpTool::new(tool)?))
    }

    /// Parse from pattern string.
    ///
    /// # Patterns
    ///
    /// - `*` → All
    /// - `mcp__memory__*` → Server("memory")
    /// - `mcp__memory__save` → Exact("memory", "save")
    /// - `mcp__*__create_*` → AnyServerToolPrefix("create_")
    /// - `mcp__memory__create_*` → ServerToolPrefix("memory", "create_")
    pub fn parse(pattern: impl AsRef<str>) -> HookResult<Self> {
        let pattern = pattern.as_ref().trim();

        if pattern.is_empty() || pattern == "*" {
            return Ok(Self::All);
        }

        if !pattern.starts_with(McpToolName::PREFIX) {
            return Err(HookError::ValidationFailed(format!(
                "MCP pattern must start with '{}' or be '*'",
                McpToolName::PREFIX
            )));
        }

        let rest = &pattern[McpToolName::PREFIX.len()..];
        let parts: Vec<&str> = rest.splitn(2, McpToolName::SEPARATOR).collect();

        match parts.as_slice() {
            ["*"] => Ok(Self::All),
            [server, "*"] if *server != "*" => Self::server(*server),
            ["*", tool] if !tool.ends_with('*') => Self::any_server_tool(*tool),
            ["*", tool_pattern] if tool_pattern.ends_with('*') => {
                let prefix = &tool_pattern[..tool_pattern.len() - 1];
                Ok(Self::any_server_tool_prefix(prefix))
            }
            [server, tool] if !tool.ends_with('*') => Self::exact(*server, *tool),
            [server, tool_pattern] if tool_pattern.ends_with('*') => {
                let prefix = &tool_pattern[..tool_pattern.len() - 1];
                Self::server_tool_prefix(*server, prefix)
            }
            _ => Err(HookError::ValidationFailed(format!(
                "Invalid MCP pattern: '{}'",
                pattern
            ))),
        }
    }

    /// Check if pattern matches a tool name.
    #[must_use]
    pub fn matches(&self, name: &McpToolName) -> bool {
        match self {
            Self::All => true,
            Self::Server(s) => name.server() == s,
            Self::Exact(s, t) => name.server() == s && name.tool() == t,
            Self::ServerToolPrefix {
                server,
                tool_prefix,
            } => name.server() == server && name.tool().as_str().starts_with(tool_prefix),
            Self::AnyServerToolPrefix { tool_prefix } => {
                name.tool().as_str().starts_with(tool_prefix)
            }
            Self::AnyServerTool(t) => name.tool() == t,
        }
    }

    /// Check if pattern matches a tool name string.
    ///
    /// Returns false if the string is not a valid MCP tool name.
    #[must_use]
    pub fn matches_str(&self, tool_name: &str) -> bool {
        McpToolName::try_parse(tool_name).is_some_and(|name| self.matches(&name))
    }
}

// ════════════════════════════════════════════════════════════════════════════
// MCP VALIDATOR
// ════════════════════════════════════════════════════════════════════════════

/// Helper for validating MCP tool calls with allow/block lists.
///
/// # Examples
///
/// ```rust
/// use claude_hooks::mcp::McpValidator;
///
/// let validator = McpValidator::new()
///     .block_servers(vec!["dangerous".to_string()]);
///
/// assert!(validator.is_allowed("mcp__memory__save"));
/// assert!(!validator.is_allowed("mcp__dangerous__delete"));
/// assert!(validator.is_allowed("Bash")); // Non-MCP passes
/// ```
pub struct McpValidator {
    /// Allowed servers (empty = all allowed).
    allowed_servers: Vec<McpServer>,
    /// Blocked servers.
    blocked_servers: Vec<McpServer>,
    /// Blocked tools (full names).
    blocked_tools: Vec<McpToolName>,
}

impl McpValidator {
    /// Create a new validator allowing all MCP tools.
    #[must_use]
    pub fn new() -> Self {
        Self {
            allowed_servers: Vec::new(),
            blocked_servers: Vec::new(),
            blocked_tools: Vec::new(),
        }
    }

    /// Only allow specific servers.
    #[must_use]
    pub fn allow_servers(mut self, servers: Vec<String>) -> Self {
        self.allowed_servers = servers
            .into_iter()
            .filter_map(|s| McpServer::new(&s).ok())
            .collect();
        self
    }

    /// Block specific servers.
    #[must_use]
    pub fn block_servers(mut self, servers: Vec<String>) -> Self {
        self.blocked_servers = servers
            .into_iter()
            .filter_map(|s| McpServer::new(&s).ok())
            .collect();
        self
    }

    /// Block specific tools by full name.
    #[must_use]
    pub fn block_tools(mut self, tools: Vec<String>) -> Self {
        self.blocked_tools = tools
            .into_iter()
            .filter_map(|s| McpToolName::parse(&s).ok())
            .collect();
        self
    }

    /// Check if a tool is allowed.
    #[must_use]
    pub fn is_allowed(&self, tool_name: &str) -> bool {
        // Check if it's an MCP tool
        let Some(mcp) = McpToolName::try_parse(tool_name) else {
            return true; // Non-MCP tools pass through
        };

        // Check blocked tools
        if self.blocked_tools.contains(&mcp) {
            return false;
        }

        // Check blocked servers
        if self.blocked_servers.contains(mcp.server()) {
            return false;
        }

        // Check allowed servers (if specified)
        if !self.allowed_servers.is_empty() && !self.allowed_servers.contains(mcp.server()) {
            return false;
        }

        true
    }

    /// Validate and return the reason if blocked.
    pub fn validate(&self, tool_name: &str) -> Result<(), String> {
        if self.is_allowed(tool_name) {
            Ok(())
        } else {
            Err(format!("MCP tool '{}' is not allowed", tool_name))
        }
    }
}

impl Default for McpValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// CATEGORIES
// ════════════════════════════════════════════════════════════════════════════

/// Categories of MCP servers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpServerCategory {
    /// NexCore computational kernel.
    NexCore,
    /// Code/documentation search.
    Search,
    /// Cloud providers (GCP, AWS, etc.).
    Cloud,
    /// Version control (GitHub, GitLab).
    VCS,
    /// External APIs.
    Api,
    /// Unknown/other.
    Other,
}

impl McpServerCategory {
    /// Categorize an MCP server by name.
    #[must_use]
    pub fn from_server(server: &McpServer) -> Self {
        match server.as_str() {
            "nexcore" => Self::NexCore,
            "rust-lang" | "claude-code-docs" => Self::Search,
            "gcloud" | "aws" | "azure" => Self::Cloud,
            "github" | "gitlab" => Self::VCS,
            _ => Self::Other,
        }
    }

    /// Categorize from a server name string.
    #[must_use]
    pub fn categorize(server: &str) -> Self {
        match server.to_lowercase().as_str() {
            "nexcore" => Self::NexCore,
            "rust-lang" | "claude-code-docs" => Self::Search,
            "gcloud" | "aws" | "azure" => Self::Cloud,
            "github" | "gitlab" => Self::VCS,
            _ => Self::Other,
        }
    }
}

/// Known NexCore tool categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexCoreToolCategory {
    /// Foundation algorithms (levenshtein, sha256, etc.).
    Foundation,
    /// Signal detection (prr, ror, ic, ebgm).
    PvSignal,
    /// Causality assessment.
    Causality,
    /// Vigilance/safety.
    Vigilance,
    /// Skill management.
    Skills,
    /// Guidelines.
    Guidelines,
    /// FAERS database.
    Faers,
    /// GCloud operations.
    GCloud,
    /// Wolfram Alpha.
    Wolfram,
    /// Principles knowledge base.
    Principles,
    /// Unknown.
    Other,
}

impl NexCoreToolCategory {
    /// Categorize a NexCore tool by name.
    #[must_use]
    pub fn from_tool(tool: &McpTool) -> Self {
        Self::categorize(tool.as_str())
    }

    /// Categorize from a tool name string.
    #[must_use]
    pub fn categorize(tool: &str) -> Self {
        if tool.starts_with("foundation_") {
            Self::Foundation
        } else if tool.starts_with("pv_signal_") || tool == "pv_chi_square" {
            Self::PvSignal
        } else if tool.starts_with("pv_naranjo") || tool.starts_with("pv_who") {
            Self::Causality
        } else if tool.starts_with("vigilance_") {
            Self::Vigilance
        } else if tool.starts_with("skill_") {
            Self::Skills
        } else if tool.starts_with("guidelines_") {
            Self::Guidelines
        } else if tool.starts_with("faers_") {
            Self::Faers
        } else if tool.starts_with("gcloud_") {
            Self::GCloud
        } else if tool.starts_with("wolfram_") {
            Self::Wolfram
        } else if tool.starts_with("principles_") {
            Self::Principles
        } else {
            Self::Other
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// TESTS
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // McpServer Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn mcp_server_accepts_valid() {
        assert!(McpServer::new("memory").is_ok());
        assert!(McpServer::new("github-api").is_ok());
        assert!(McpServer::new("my_server_123").is_ok());
        assert!(McpServer::new("nexcore").is_ok());
    }

    #[test]
    fn mcp_server_rejects_invalid() {
        assert!(McpServer::new("").is_err()); // empty
        assert!(McpServer::new("Memory").is_err()); // uppercase
        assert!(McpServer::new("my server").is_err()); // space
        assert!(McpServer::new("server.name").is_err()); // dot
    }

    #[test]
    fn mcp_server_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let server = McpServer::new("memory")?;
        let json = serde_json::to_string(&server)?;
        assert_eq!(json, "\"memory\"");
        let parsed: McpServer = serde_json::from_str(&json)?;
        assert_eq!(server, parsed);
        Ok(())
    }

    #[test]
    fn mcp_server_serde_rejects_invalid() {
        let json = "\"INVALID\"";
        let result: Result<McpServer, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // McpTool Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn mcp_tool_accepts_valid() {
        assert!(McpTool::new("save").is_ok());
        assert!(McpTool::new("create_entities").is_ok());
        assert!(McpTool::new("skill_list").is_ok());
        assert!(McpTool::new("foundation_sha256").is_ok());
    }

    #[test]
    fn mcp_tool_rejects_invalid() {
        assert!(McpTool::new("").is_err()); // empty
        assert!(McpTool::new("Save").is_err()); // uppercase
        assert!(McpTool::new("create-entities").is_err()); // hyphen
        assert!(McpTool::new("save file").is_err()); // space
    }

    #[test]
    fn mcp_tool_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let tool = McpTool::new("create_entities")?;
        let json = serde_json::to_string(&tool)?;
        assert_eq!(json, "\"create_entities\"");
        let parsed: McpTool = serde_json::from_str(&json)?;
        assert_eq!(tool, parsed);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // McpToolName Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn mcp_tool_name_parse_valid() -> Result<(), HookError> {
        let name = McpToolName::parse("mcp__memory__save")?;
        assert_eq!(name.server().as_str(), "memory");
        assert_eq!(name.tool().as_str(), "save");
        Ok(())
    }

    #[test]
    fn mcp_tool_name_parse_complex() -> Result<(), HookError> {
        let name = McpToolName::parse("mcp__nexcore__foundation_sha256")?;
        assert_eq!(name.server().as_str(), "nexcore");
        assert_eq!(name.tool().as_str(), "foundation_sha256");
        Ok(())
    }

    #[test]
    fn mcp_tool_name_parse_invalid() {
        assert!(McpToolName::parse("Bash").is_err());
        assert!(McpToolName::parse("mcp_memory_save").is_err()); // single underscore
        assert!(McpToolName::parse("mcp__memory").is_err()); // missing tool
        assert!(McpToolName::parse("").is_err());
    }

    #[test]
    fn mcp_tool_name_try_parse() {
        assert!(McpToolName::try_parse("mcp__memory__save").is_some());
        assert!(McpToolName::try_parse("Bash").is_none());
    }

    #[test]
    fn mcp_tool_name_display() {
        let name = McpToolName::parse("mcp__memory__save").unwrap();
        assert_eq!(name.to_string(), "mcp__memory__save");
        assert_eq!(name.full_name(), "mcp__memory__save");
    }

    #[test]
    fn mcp_tool_name_is_mcp() {
        assert!(McpToolName::is_mcp("mcp__memory__save"));
        assert!(!McpToolName::is_mcp("Bash"));
        assert!(!McpToolName::is_mcp("mcp__onlyonepart"));
    }

    #[test]
    fn mcp_tool_name_from_parts() {
        let server = McpServer::new("memory").unwrap();
        let tool = McpTool::new("save").unwrap();
        let name = McpToolName::from_parts(server, tool);
        assert_eq!(name.to_string(), "mcp__memory__save");
    }

    #[test]
    fn mcp_tool_name_is_from_server() {
        let name = McpToolName::parse("mcp__memory__save").unwrap();
        let memory = McpServer::new("memory").unwrap();
        let github = McpServer::new("github").unwrap();
        assert!(name.is_from_server(&memory));
        assert!(!name.is_from_server(&github));
    }

    #[test]
    fn mcp_tool_name_serde_roundtrip() {
        let name = McpToolName::parse("mcp__memory__save").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"mcp__memory__save\"");
        let parsed: McpToolName = serde_json::from_str(&json).unwrap();
        assert_eq!(name, parsed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // McpMatcher Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn mcp_matcher_all() {
        let matcher = McpMatcher::all();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
        assert!(matcher.matches(&McpToolName::parse("mcp__github__search").unwrap()));
    }

    #[test]
    fn mcp_matcher_server() {
        let matcher = McpMatcher::server("memory").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__create").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__github__search").unwrap()));
    }

    #[test]
    fn mcp_matcher_exact() {
        let matcher = McpMatcher::exact("memory", "save").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__memory__create").unwrap()));
    }

    #[test]
    fn mcp_matcher_server_tool_prefix() {
        let matcher = McpMatcher::server_tool_prefix("memory", "create_").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__create_entities").unwrap()));
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__create_relations").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__github__create_issue").unwrap()));
    }

    #[test]
    fn mcp_matcher_any_server_tool_prefix() {
        let matcher = McpMatcher::any_server_tool_prefix("create_");
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__create_entities").unwrap()));
        assert!(matcher.matches(&McpToolName::parse("mcp__github__create_issue").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
    }

    #[test]
    fn mcp_matcher_any_server_tool() {
        let matcher = McpMatcher::any_server_tool("save").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
        assert!(matcher.matches(&McpToolName::parse("mcp__github__save").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__memory__create").unwrap()));
    }

    #[test]
    fn mcp_matcher_parse_all() {
        let matcher = McpMatcher::parse("*").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
    }

    #[test]
    fn mcp_matcher_parse_server() {
        let matcher = McpMatcher::parse("mcp__memory__*").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__github__search").unwrap()));
    }

    #[test]
    fn mcp_matcher_parse_exact() {
        let matcher = McpMatcher::parse("mcp__memory__save").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__memory__create").unwrap()));
    }

    #[test]
    fn mcp_matcher_parse_any_server_prefix() {
        let matcher = McpMatcher::parse("mcp__*__create_*").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__create_entities").unwrap()));
        assert!(matcher.matches(&McpToolName::parse("mcp__github__create_issue").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__memory__save").unwrap()));
    }

    #[test]
    fn mcp_matcher_parse_server_prefix() {
        let matcher = McpMatcher::parse("mcp__memory__create_*").unwrap();
        assert!(matcher.matches(&McpToolName::parse("mcp__memory__create_entities").unwrap()));
        assert!(!matcher.matches(&McpToolName::parse("mcp__github__create_issue").unwrap()));
    }

    #[test]
    fn mcp_matcher_matches_str() {
        let matcher = McpMatcher::server("memory").unwrap();
        assert!(matcher.matches_str("mcp__memory__save"));
        assert!(!matcher.matches_str("mcp__github__search"));
        assert!(!matcher.matches_str("Bash")); // Non-MCP
    }

    // ─────────────────────────────────────────────────────────────────────────
    // McpValidator Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn mcp_validator_allows_by_default() {
        let validator = McpValidator::new();
        assert!(validator.is_allowed("mcp__memory__save"));
        assert!(validator.is_allowed("Bash")); // Non-MCP passes
    }

    #[test]
    fn mcp_validator_blocks_servers() {
        let validator = McpValidator::new().block_servers(vec!["dangerous".to_string()]);
        assert!(validator.is_allowed("mcp__memory__save"));
        assert!(!validator.is_allowed("mcp__dangerous__delete"));
    }

    #[test]
    fn mcp_validator_allows_only_specified() {
        let validator = McpValidator::new().allow_servers(vec!["memory".to_string()]);
        assert!(validator.is_allowed("mcp__memory__save"));
        assert!(!validator.is_allowed("mcp__github__search"));
    }

    #[test]
    fn mcp_validator_blocks_specific_tools() {
        let validator = McpValidator::new().block_tools(vec!["mcp__memory__delete".to_string()]);
        assert!(validator.is_allowed("mcp__memory__save"));
        assert!(!validator.is_allowed("mcp__memory__delete"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Category Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn mcp_server_category() {
        let nexcore = McpServer::new("nexcore").unwrap();
        assert_eq!(
            McpServerCategory::from_server(&nexcore),
            McpServerCategory::NexCore
        );

        let github = McpServer::new("github").unwrap();
        assert_eq!(
            McpServerCategory::from_server(&github),
            McpServerCategory::VCS
        );
    }

    #[test]
    fn nexcore_tool_category() {
        let foundation = McpTool::new("foundation_levenshtein").unwrap();
        assert_eq!(
            NexCoreToolCategory::from_tool(&foundation),
            NexCoreToolCategory::Foundation
        );

        let signal = McpTool::new("pv_signal_prr").unwrap();
        assert_eq!(
            NexCoreToolCategory::from_tool(&signal),
            NexCoreToolCategory::PvSignal
        );

        let skills = McpTool::new("skill_list").unwrap();
        assert_eq!(
            NexCoreToolCategory::from_tool(&skills),
            NexCoreToolCategory::Skills
        );
    }
}
