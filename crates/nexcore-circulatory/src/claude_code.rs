//! # Claude Code MCP Transport Network — Circulatory Mapping
//!
//! Maps the biological circulatory system to Claude Code's MCP server transport
//! network per Biological Alignment v2.0 section 6.
//!
//! ## Mapping Overview
//!
//! | Biological Structure | Claude Code Analog | Function |
//! |---------------------|--------------------|----------|
//! | Heart | MCP Server Manager | Central pump orchestrating transport |
//! | Right heart | stdio transport | Local process communication |
//! | Left heart | HTTP transport | Remote API calls |
//! | Arteries | Outbound MCP tool calls | Commands flowing to external services |
//! | Capillaries | MCP tool execution | Where actual exchange happens |
//! | Veins | MCP tool results | Data flowing back to the agent |
//! | Red blood cells | Tool results | Carry useful data (oxygen) |
//! | White blood cells | Permission checks | Patrol for threats in transit |
//! | Platelets | Error recovery | Seal breaches when MCP calls fail |
//! | Plasma | JSON/Protocol | The transport medium itself |
//! | Portal filtration | MAX_MCP_OUTPUT_TOKENS | Liver filtering external data |
//! | Selective perfusion | Tool Search / deferred loading | On-demand tool loading |
//! | Frank-Starling law | MCP_TIMEOUT adjusts to load | Timeout scales with demand |
//!
//! ## MCP Scopes as Circulatory Regions
//!
//! - **Local** (capillary bed): Tools scoped to a single project directory
//! - **Project** (organ-level): Tools scoped to a project workspace
//! - **User** (systemic circulation): Tools available across all projects

#![allow(
    dead_code,
    reason = "Transport model includes reference structures that are not yet invoked in all runtime paths"
)]

use serde::{Deserialize, Serialize};

// ============================================================================
// McpTransport — Dual circulation (right heart = stdio, left heart = HTTP)
// ============================================================================

/// MCP transport type modeling dual circulation.
///
/// Per Biological Alignment v2.0 section 6: the heart has two sides.
/// The right side pumps blood to the lungs (local stdio transport — short
/// loop, same machine). The left side pumps blood systemically (HTTP
/// transport — remote API calls across the network).
///
/// Tier: T2-P (Sum + Mapping), dominant Sum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpTransport {
    /// Right heart: stdio transport for local process communication.
    /// Blood flows through the pulmonary circuit — never leaves the body.
    Stdio,
    /// Left heart: HTTP transport for remote API calls.
    /// Blood flows through the systemic circuit — reaches distant organs.
    Http,
}

impl McpTransport {
    /// Returns the biological analog name for this transport type.
    pub fn biological_analog(&self) -> &'static str {
        match self {
            Self::Stdio => "pulmonary circulation (right heart)",
            Self::Http => "systemic circulation (left heart)",
        }
    }
}

// ============================================================================
// McpScope — Circulatory region granularity
// ============================================================================

/// MCP tool scope as a circulatory region.
///
/// Per Biological Alignment v2.0 section 6: tool scopes map to circulatory
/// regions of increasing breadth.
///
/// Tier: T2-P (Sum + Location), dominant Sum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpScope {
    /// Capillary bed: tools scoped to a single project directory.
    Local,
    /// Organ-level: tools scoped to a project workspace.
    Project,
    /// Systemic circulation: tools available across all projects.
    User,
}

impl McpScope {
    /// Returns the biological analog for this scope level.
    pub fn biological_analog(&self) -> &'static str {
        match self {
            Self::Local => "capillary bed",
            Self::Project => "organ-level circulation",
            Self::User => "systemic circulation",
        }
    }
}

// ============================================================================
// FlowDirection — Arterial (outbound) vs Venous (return)
// ============================================================================

/// Direction of data flow through the MCP transport network.
///
/// Per Biological Alignment v2.0 section 6: arteries carry oxygenated blood
/// away from the heart (outbound tool calls), veins carry deoxygenated blood
/// back (tool results returning to the agent).
///
/// Tier: T2-P (Sum + Causality), dominant Causality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlowDirection {
    /// Arterial: outbound MCP tool calls (commands to external services).
    Arterial,
    /// Venous: inbound MCP tool results (data returning to the agent).
    Venous,
}

// ============================================================================
// McpServer — A registered MCP server (one circulatory organ)
// ============================================================================

/// An MCP server registered with Claude Code.
///
/// Per Biological Alignment v2.0 section 6: each MCP server is an organ
/// connected to the circulatory network. It receives blood (tool calls)
/// through arteries and returns blood (results) through veins.
///
/// Tier: T2-C (Product + Mapping + State + Quantity), dominant Mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    /// Server name (e.g., "nexcore", "claude-fs", "compendious").
    pub name: String,
    /// Transport type: stdio (local) or HTTP (remote).
    pub transport: McpTransport,
    /// Number of tools this server exposes.
    pub tool_count: usize,
    /// Scope at which this server's tools are available.
    pub scope: McpScope,
    /// Whether the server is currently active and responsive.
    pub active: bool,
}

// ============================================================================
// McpHeartbeat — One circulatory cycle's health check
// ============================================================================

/// A heartbeat representing one circulatory cycle to an MCP server.
///
/// Per Biological Alignment v2.0 section 6: each heartbeat pumps blood
/// through the circuit. Latency is the time for one complete cycle
/// (call sent, result received). An unresponsive server is like an
/// organ with blocked perfusion.
///
/// Tier: T2-C (Frequency + Mapping + Quantity + State), dominant Frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHeartbeat {
    /// Which server was checked.
    pub server_name: String,
    /// Transport used for this heartbeat.
    pub transport: McpTransport,
    /// Round-trip latency in milliseconds.
    pub latency_ms: u64,
    /// Whether the server responded to the heartbeat.
    pub is_responsive: bool,
}

// ============================================================================
// ToolCall — Arterial flow (outbound command)
// ============================================================================

/// An outbound MCP tool call (arterial flow).
///
/// Per Biological Alignment v2.0 section 6: arteries carry oxygenated blood
/// from the heart to the organs. A tool call is an arterial pulse —
/// the agent sends a command with parameters to an MCP server.
///
/// Tier: T2-C (Causality + Mapping + Quantity + Sequence), dominant Causality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Target MCP server.
    pub server: String,
    /// Tool being invoked.
    pub tool_name: String,
    /// Size of parameters in tokens.
    pub params_size_tokens: usize,
    /// Direction of flow (always Arterial for calls).
    pub direction: FlowDirection,
}

impl ToolCall {
    /// Create a new outbound tool call.
    pub fn new(
        server: impl Into<String>,
        tool_name: impl Into<String>,
        params_size_tokens: usize,
    ) -> Self {
        Self {
            server: server.into(),
            tool_name: tool_name.into(),
            params_size_tokens,
            direction: FlowDirection::Arterial,
        }
    }
}

// ============================================================================
// ToolResult — Venous flow (return data)
// ============================================================================

/// An inbound MCP tool result (venous flow).
///
/// Per Biological Alignment v2.0 section 6: veins carry deoxygenated blood
/// back to the heart. A tool result is a venous return — data flowing
/// back from the MCP server to the agent. Large results may be truncated
/// (portal filtration) to prevent flooding the agent context.
///
/// Tier: T2-C (Causality + Mapping + Quantity + Boundary), dominant Causality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Source MCP server.
    pub server: String,
    /// Tool that produced this result.
    pub tool_name: String,
    /// Size of result in tokens.
    pub result_size_tokens: usize,
    /// Whether the result was truncated due to size limits.
    pub truncated: bool,
    /// Whether portal filtration (MAX_MCP_OUTPUT_TOKENS) was applied.
    pub portal_filtered: bool,
}

// ============================================================================
// PortalFiltration — MAX_MCP_OUTPUT_TOKENS (liver filtering)
// ============================================================================

/// Portal filtration: the liver filtering external data before it enters
/// the agent's context window.
///
/// Per Biological Alignment v2.0 section 6: the hepatic portal system
/// filters blood from the gut before it enters systemic circulation.
/// Similarly, MAX_MCP_OUTPUT_TOKENS caps how much raw tool output
/// can enter the agent's context, preventing context flooding.
///
/// Tier: T2-C (Boundary + Quantity + Sequence + Irreversibility), dominant Boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalFiltration {
    /// Maximum output tokens allowed per tool result (default: 25000).
    pub max_output_tokens: usize,
    /// Number of results that were filtered (truncated).
    pub filtered_count: usize,
    /// Total tokens removed by filtration.
    pub total_tokens_filtered: usize,
}

impl Default for PortalFiltration {
    fn default() -> Self {
        Self {
            max_output_tokens: 25_000,
            filtered_count: 0,
            total_tokens_filtered: 0,
        }
    }
}

impl PortalFiltration {
    /// Create a portal filtration tracker with the given token limit.
    pub fn with_limit(max_output_tokens: usize) -> Self {
        Self {
            max_output_tokens,
            filtered_count: 0,
            total_tokens_filtered: 0,
        }
    }

    /// Apply filtration to a result size. Returns the allowed size and
    /// whether filtration was applied.
    pub fn filter(&mut self, result_tokens: usize) -> (usize, bool) {
        if result_tokens > self.max_output_tokens {
            let excess = result_tokens.saturating_sub(self.max_output_tokens);
            self.filtered_count = self.filtered_count.saturating_add(1);
            self.total_tokens_filtered = self.total_tokens_filtered.saturating_add(excess);
            (self.max_output_tokens, true)
        } else {
            (result_tokens, false)
        }
    }

    /// Filtration ratio: fraction of results that required filtering.
    /// Returns 0.0 if no results have been processed.
    pub fn filtration_ratio(&self, total_results: usize) -> f64 {
        if total_results == 0 {
            return 0.0;
        }
        self.filtered_count as f64 / total_results as f64
    }
}

// ============================================================================
// SelectivePerfusion — On-demand tool loading (ToolSearch)
// ============================================================================

/// Selective perfusion: on-demand tool loading via ToolSearch.
///
/// Per Biological Alignment v2.0 section 6: blood does not flow equally
/// to all organs. Active organs receive more blood (selective perfusion).
/// Similarly, Claude Code's deferred tool loading only activates tools
/// when they are needed — idle tools consume no context bandwidth.
///
/// Tier: T2-C (Quantity + Existence + Boundary + Comparison), dominant Quantity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectivePerfusion {
    /// Total number of tools across all MCP servers.
    pub total_tools: usize,
    /// Number of tools in deferred (unloaded) state.
    pub deferred_tools: usize,
    /// Number of tools currently active (loaded and available).
    pub active_tools: usize,
    /// Perfusion ratio: active_tools / total_tools.
    /// Low ratio = efficient (most tools dormant until needed).
    pub perfusion_ratio: f64,
}

impl SelectivePerfusion {
    /// Create a new perfusion state from tool counts.
    pub fn new(total_tools: usize, active_tools: usize) -> Self {
        let deferred_tools = total_tools.saturating_sub(active_tools);
        let perfusion_ratio = if total_tools == 0 {
            0.0
        } else {
            active_tools as f64 / total_tools as f64
        };
        Self {
            total_tools,
            deferred_tools,
            active_tools,
            perfusion_ratio,
        }
    }

    /// Whether perfusion is efficient (low ratio = most tools dormant).
    /// A ratio below 0.3 means less than 30% of tools are active.
    pub fn is_efficient(&self) -> bool {
        self.perfusion_ratio < 0.3
    }
}

// ============================================================================
// BloodPayload — The composite data in transit
// ============================================================================

/// The composite blood payload flowing through MCP transport.
///
/// Per Biological Alignment v2.0 section 6: blood is not a single substance
/// but a composite. Red cells carry oxygen (tool results with useful data),
/// white cells patrol for threats (permission checks), platelets seal
/// breaches (error recovery), and plasma is the medium (JSON protocol).
///
/// Tier: T2-C (Product + Mapping + Boundary + State), dominant Product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodPayload {
    /// Red blood cells: tool results carrying useful data.
    pub red_cells: usize,
    /// White blood cells: permission checks performed.
    pub white_cells: usize,
    /// Platelets: error recoveries triggered.
    pub platelets: usize,
    /// Plasma format: the transport medium protocol.
    pub plasma_format: String,
}

impl Default for BloodPayload {
    fn default() -> Self {
        Self {
            red_cells: 0,
            white_cells: 0,
            platelets: 0,
            plasma_format: "JSON".to_string(),
        }
    }
}

// ============================================================================
// FrankStarling — MCP_TIMEOUT adjusts to load
// ============================================================================

/// Frank-Starling mechanism: MCP timeout adjusts to load.
///
/// Per Biological Alignment v2.0 section 6: the Frank-Starling law states
/// that the heart's stroke volume increases with preload. When more blood
/// returns to the heart, it pumps harder. Similarly, when MCP load
/// increases, the timeout adjusts upward to accommodate the demand,
/// preventing premature disconnections under heavy tool usage.
///
/// Tier: T2-C (Quantity + Frequency + Causality + Boundary), dominant Quantity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrankStarling {
    /// Base timeout in milliseconds (resting heart rate).
    pub base_timeout_ms: u64,
    /// Current load factor (0.0 = idle, 1.0 = full capacity).
    pub current_load: f64,
    /// Adjusted timeout after applying Frank-Starling compensation.
    pub adjusted_timeout_ms: u64,
}

impl Default for FrankStarling {
    fn default() -> Self {
        Self {
            base_timeout_ms: 30_000,
            current_load: 0.0,
            adjusted_timeout_ms: 30_000,
        }
    }
}

impl FrankStarling {
    /// Create a new Frank-Starling regulator with the given base timeout.
    pub fn new(base_timeout_ms: u64) -> Self {
        Self {
            base_timeout_ms,
            current_load: 0.0,
            adjusted_timeout_ms: base_timeout_ms,
        }
    }

    /// Adjust the timeout based on current load.
    ///
    /// The Frank-Starling law: as preload (load factor) increases,
    /// the stroke volume (timeout) increases proportionally, up to
    /// a maximum of 4x the base timeout (physiological limit).
    ///
    /// Formula: adjusted = base * (1.0 + load * 3.0)
    /// - At load 0.0: timeout = base (resting)
    /// - At load 0.5: timeout = base * 2.5
    /// - At load 1.0: timeout = base * 4.0 (maximum contractility)
    pub fn adjust(&mut self, load: f64) {
        // Clamp load to [0.0, 1.0]
        let clamped = load.clamp(0.0, 1.0);
        self.current_load = clamped;

        // Frank-Starling: stroke volume scales with preload
        let multiplier = 1.0 + clamped * 3.0;
        let adjusted = (self.base_timeout_ms as f64 * multiplier) as u64;
        self.adjusted_timeout_ms = adjusted;
    }

    /// The Frank-Starling ratio: adjusted_timeout / base_timeout.
    /// 1.0 = resting, 4.0 = maximum contractility.
    pub fn ratio(&self) -> f64 {
        if self.base_timeout_ms == 0 {
            return 0.0;
        }
        self.adjusted_timeout_ms as f64 / self.base_timeout_ms as f64
    }
}

// ============================================================================
// CirculatoryHealth — System-wide health assessment
// ============================================================================

/// System-wide circulatory health assessment for the MCP transport network.
///
/// Per Biological Alignment v2.0 section 6: overall cardiovascular health
/// requires all subsystems functioning — heart pumping, vessels clear,
/// blood composition balanced, portal filtration active. This struct
/// aggregates the health of the entire MCP transport network.
///
/// Tier: T3 (Product + Quantity + State + Comparison + Boundary + Existence),
/// dominant Product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirculatoryHealth {
    /// Number of registered MCP servers (target: 5 for the standard roster).
    pub server_count: usize,
    /// Whether all registered servers responded to heartbeat.
    pub all_responsive: bool,
    /// Whether portal filtration (MAX_MCP_OUTPUT_TOKENS) is active.
    pub portal_filtration_active: bool,
    /// Whether selective perfusion (deferred tool loading) is active.
    pub selective_perfusion_active: bool,
    /// Frank-Starling ratio (adjusted_timeout / base_timeout).
    /// 1.0 = resting, up to 4.0 = maximum load compensation.
    pub frank_starling_ratio: f64,
}

impl CirculatoryHealth {
    /// Diagnose the health of the MCP circulatory network.
    ///
    /// Produces a summary assessment based on:
    /// - Server count (healthy: >= 3, target: 5)
    /// - All servers responsive
    /// - Portal filtration active (prevents context flooding)
    /// - Selective perfusion active (efficient tool loading)
    /// - Frank-Starling ratio in normal range (1.0 to 4.0)
    ///
    /// Returns a tuple of (is_healthy, Vec of diagnostic messages).
    pub fn diagnose(&self) -> (bool, Vec<String>) {
        let mut messages = Vec::new();
        let mut healthy = true;

        // Check server count
        if self.server_count == 0 {
            messages.push("CRITICAL: no MCP servers registered — circulatory arrest".to_string());
            healthy = false;
        } else if self.server_count < 3 {
            messages.push(format!(
                "WARNING: only {} MCP servers (target: 5) — reduced perfusion",
                self.server_count
            ));
        } else {
            messages.push(format!("OK: {} MCP servers registered", self.server_count));
        }

        // Check responsiveness
        if !self.all_responsive {
            messages.push("WARNING: not all servers responsive — partial ischemia".to_string());
            healthy = false;
        } else {
            messages.push("OK: all servers responsive — full perfusion".to_string());
        }

        // Check portal filtration
        if !self.portal_filtration_active {
            messages
                .push("WARNING: portal filtration inactive — risk of context flooding".to_string());
            healthy = false;
        } else {
            messages.push("OK: portal filtration active — context protected".to_string());
        }

        // Check selective perfusion
        if !self.selective_perfusion_active {
            messages.push(
                "INFO: selective perfusion inactive — all tools loaded (higher memory usage)"
                    .to_string(),
            );
        } else {
            messages.push("OK: selective perfusion active — efficient tool loading".to_string());
        }

        // Check Frank-Starling ratio
        if self.frank_starling_ratio < 1.0 {
            messages.push(format!(
                "WARNING: Frank-Starling ratio {:.2} below baseline — timeout undercompensation",
                self.frank_starling_ratio
            ));
            healthy = false;
        } else if self.frank_starling_ratio > 4.0 {
            messages.push(format!(
                "WARNING: Frank-Starling ratio {:.2} above maximum — timeout overcompensation",
                self.frank_starling_ratio
            ));
        } else {
            messages.push(format!(
                "OK: Frank-Starling ratio {:.2} — normal cardiac output",
                self.frank_starling_ratio
            ));
        }

        (healthy, messages)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- McpTransport tests ---

    #[test]
    fn transport_biological_analogs() {
        assert_eq!(
            McpTransport::Stdio.biological_analog(),
            "pulmonary circulation (right heart)"
        );
        assert_eq!(
            McpTransport::Http.biological_analog(),
            "systemic circulation (left heart)"
        );
    }

    #[test]
    fn transport_equality() {
        assert_eq!(McpTransport::Stdio, McpTransport::Stdio);
        assert_ne!(McpTransport::Stdio, McpTransport::Http);
    }

    // --- McpScope tests ---

    #[test]
    fn scope_biological_analogs() {
        assert_eq!(McpScope::Local.biological_analog(), "capillary bed");
        assert_eq!(
            McpScope::Project.biological_analog(),
            "organ-level circulation"
        );
        assert_eq!(McpScope::User.biological_analog(), "systemic circulation");
    }

    // --- McpServer tests ---

    #[test]
    fn mcp_server_creation() {
        let server = McpServer {
            name: "nexcore".to_string(),
            transport: McpTransport::Stdio,
            tool_count: 260,
            scope: McpScope::User,
            active: true,
        };
        assert_eq!(server.name, "nexcore");
        assert_eq!(server.transport, McpTransport::Stdio);
        assert_eq!(server.tool_count, 260);
        assert!(server.active);
    }

    // --- ToolCall tests ---

    #[test]
    fn tool_call_is_always_arterial() {
        let call = ToolCall::new("nexcore", "pv_signal_complete", 128);
        assert_eq!(call.direction, FlowDirection::Arterial);
        assert_eq!(call.server, "nexcore");
        assert_eq!(call.tool_name, "pv_signal_complete");
        assert_eq!(call.params_size_tokens, 128);
    }

    // --- PortalFiltration tests ---

    #[test]
    fn portal_filtration_default_limit() {
        let pf = PortalFiltration::default();
        assert_eq!(pf.max_output_tokens, 25_000);
        assert_eq!(pf.filtered_count, 0);
    }

    #[test]
    fn portal_filtration_filters_large_results() {
        let mut pf = PortalFiltration::with_limit(1000);

        // Small result passes through
        let (size, filtered) = pf.filter(500);
        assert_eq!(size, 500);
        assert!(!filtered);

        // Large result gets truncated
        let (size, filtered) = pf.filter(5000);
        assert_eq!(size, 1000);
        assert!(filtered);
        assert_eq!(pf.filtered_count, 1);
        assert_eq!(pf.total_tokens_filtered, 4000);
    }

    #[test]
    fn portal_filtration_ratio() {
        let mut pf = PortalFiltration::with_limit(100);
        let _ = pf.filter(50); // not filtered
        let _ = pf.filter(200); // filtered
        let _ = pf.filter(300); // filtered

        let ratio = pf.filtration_ratio(3);
        let diff = (ratio - (2.0 / 3.0)).abs();
        assert!(diff < 0.01);
    }

    #[test]
    fn portal_filtration_ratio_zero_results() {
        let pf = PortalFiltration::default();
        let ratio = pf.filtration_ratio(0);
        assert!(ratio.abs() < f64::EPSILON);
    }

    // --- SelectivePerfusion tests ---

    #[test]
    fn selective_perfusion_calculation() {
        let sp = SelectivePerfusion::new(300, 30);
        assert_eq!(sp.total_tools, 300);
        assert_eq!(sp.deferred_tools, 270);
        assert_eq!(sp.active_tools, 30);
        let diff = (sp.perfusion_ratio - 0.1).abs();
        assert!(diff < 0.01);
        assert!(sp.is_efficient());
    }

    #[test]
    fn selective_perfusion_zero_tools() {
        let sp = SelectivePerfusion::new(0, 0);
        assert!(sp.perfusion_ratio.abs() < f64::EPSILON);
    }

    #[test]
    fn selective_perfusion_high_ratio_not_efficient() {
        let sp = SelectivePerfusion::new(100, 50);
        assert!(!sp.is_efficient());
    }

    // --- FrankStarling tests ---

    #[test]
    fn frank_starling_resting() {
        let fs = FrankStarling::new(30_000);
        assert_eq!(fs.adjusted_timeout_ms, 30_000);
        let diff = (fs.ratio() - 1.0).abs();
        assert!(diff < f64::EPSILON);
    }

    #[test]
    fn frank_starling_adjust_scales_timeout() {
        let mut fs = FrankStarling::new(30_000);

        // Half load: 30000 * (1 + 0.5 * 3) = 30000 * 2.5 = 75000
        fs.adjust(0.5);
        assert_eq!(fs.adjusted_timeout_ms, 75_000);

        // Full load: 30000 * (1 + 1.0 * 3) = 30000 * 4.0 = 120000
        fs.adjust(1.0);
        assert_eq!(fs.adjusted_timeout_ms, 120_000);
    }

    #[test]
    fn frank_starling_clamps_load() {
        let mut fs = FrankStarling::new(10_000);

        // Negative load clamps to 0
        fs.adjust(-0.5);
        assert_eq!(fs.adjusted_timeout_ms, 10_000);
        let diff = (fs.current_load - 0.0).abs();
        assert!(diff < f64::EPSILON);

        // Load > 1.0 clamps to 1.0
        fs.adjust(5.0);
        let diff = (fs.current_load - 1.0).abs();
        assert!(diff < f64::EPSILON);
        assert_eq!(fs.adjusted_timeout_ms, 40_000);
    }

    #[test]
    fn frank_starling_zero_base() {
        let fs = FrankStarling::new(0);
        assert!(fs.ratio().abs() < f64::EPSILON);
    }

    // --- BloodPayload tests ---

    #[test]
    fn blood_payload_defaults_to_json() {
        let bp = BloodPayload::default();
        assert_eq!(bp.plasma_format, "JSON");
        assert_eq!(bp.red_cells, 0);
        assert_eq!(bp.white_cells, 0);
        assert_eq!(bp.platelets, 0);
    }

    // --- CirculatoryHealth::diagnose tests ---

    #[test]
    fn diagnose_healthy_system() {
        let health = CirculatoryHealth {
            server_count: 5,
            all_responsive: true,
            portal_filtration_active: true,
            selective_perfusion_active: true,
            frank_starling_ratio: 1.0,
        };
        let (is_healthy, messages) = health.diagnose();
        assert!(is_healthy);
        assert_eq!(messages.len(), 5);
        // All messages should start with "OK:"
        for msg in &messages {
            assert!(msg.starts_with("OK:"), "expected OK, got: {msg}");
        }
    }

    #[test]
    fn diagnose_no_servers_is_critical() {
        let health = CirculatoryHealth {
            server_count: 0,
            all_responsive: false,
            portal_filtration_active: true,
            selective_perfusion_active: true,
            frank_starling_ratio: 1.0,
        };
        let (is_healthy, messages) = health.diagnose();
        assert!(!is_healthy);
        let has_critical = messages.iter().any(|m| m.contains("CRITICAL"));
        assert!(has_critical);
    }

    #[test]
    fn diagnose_unresponsive_servers() {
        let health = CirculatoryHealth {
            server_count: 3,
            all_responsive: false,
            portal_filtration_active: true,
            selective_perfusion_active: true,
            frank_starling_ratio: 2.0,
        };
        let (is_healthy, _messages) = health.diagnose();
        assert!(!is_healthy);
    }

    #[test]
    fn diagnose_no_portal_filtration() {
        let health = CirculatoryHealth {
            server_count: 5,
            all_responsive: true,
            portal_filtration_active: false,
            selective_perfusion_active: true,
            frank_starling_ratio: 1.5,
        };
        let (is_healthy, messages) = health.diagnose();
        assert!(!is_healthy);
        let has_filtration_warning = messages
            .iter()
            .any(|m| m.contains("portal filtration inactive"));
        assert!(has_filtration_warning);
    }

    #[test]
    fn diagnose_frank_starling_undercompensation() {
        let health = CirculatoryHealth {
            server_count: 5,
            all_responsive: true,
            portal_filtration_active: true,
            selective_perfusion_active: true,
            frank_starling_ratio: 0.5,
        };
        let (is_healthy, messages) = health.diagnose();
        assert!(!is_healthy);
        let has_fs_warning = messages.iter().any(|m| m.contains("below baseline"));
        assert!(has_fs_warning);
    }

    // --- Serialization round-trip tests ---

    #[test]
    fn serde_round_trip_mcp_server() {
        let server = McpServer {
            name: "nexcore".to_string(),
            transport: McpTransport::Stdio,
            tool_count: 260,
            scope: McpScope::User,
            active: true,
        };
        let json = serde_json::to_string(&server);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        let parsed: Result<McpServer, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok());
    }

    #[test]
    fn serde_round_trip_circulatory_health() {
        let health = CirculatoryHealth {
            server_count: 5,
            all_responsive: true,
            portal_filtration_active: true,
            selective_perfusion_active: true,
            frank_starling_ratio: 1.0,
        };
        let json = serde_json::to_string(&health);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        let parsed: Result<CirculatoryHealth, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok());
    }
}
