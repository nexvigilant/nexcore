//! # MCP Failure Tests
//!
//! CTVP Phase 1: Test behavior when MCP server is unreachable.
//!
//! ## Test Scenarios
//!
//! 1. Connection refused - Server not running
//! 2. Connection timeout - Server unresponsive
//! 3. Partial response - Server crashes mid-response
//! 4. Invalid response format - Malformed JSON

use std::time::Duration;

use super::{ChaosTestResult, FaultInjector};

/// Simulated MCP connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpConnectionState {
    /// Connected and healthy
    Connected,
    /// Connection refused (server not running)
    ConnectionRefused,
    /// Connection timed out
    Timeout,
    /// Server returned invalid response
    InvalidResponse,
    /// Server crashed mid-response
    PartialResponse,
}

/// Error types for MCP operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpError {
    /// Connection was refused
    ConnectionRefused(String),
    /// Operation timed out
    Timeout {
        operation: String,
        elapsed: Duration,
    },
    /// Invalid response from server
    InvalidResponse { expected: String, received: String },
    /// Server returned partial data
    PartialResponse {
        bytes_received: usize,
        bytes_expected: usize,
    },
    /// Server is unavailable for graceful degradation
    Unavailable(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionRefused(msg) => write!(f, "Connection refused: {msg}"),
            Self::Timeout { operation, elapsed } => {
                write!(f, "Timeout after {elapsed:?} on operation: {operation}")
            }
            Self::InvalidResponse { expected, received } => {
                write!(f, "Invalid response: expected {expected}, got {received}")
            }
            Self::PartialResponse {
                bytes_received,
                bytes_expected,
            } => {
                write!(
                    f,
                    "Partial response: received {bytes_received}/{bytes_expected} bytes"
                )
            }
            Self::Unavailable(msg) => write!(f, "MCP unavailable: {msg}"),
        }
    }
}

/// Simulated MCP client for testing
pub struct SimulatedMcpClient {
    state: McpConnectionState,
    fault_injector: Option<FaultInjector>,
}

impl SimulatedMcpClient {
    /// Create a new simulated client
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: McpConnectionState::Connected,
            fault_injector: None,
        }
    }

    /// Set the connection state
    pub fn set_state(&mut self, state: McpConnectionState) {
        self.state = state;
    }

    /// Attach a fault injector
    pub fn with_fault_injector(mut self, injector: FaultInjector) -> Self {
        self.fault_injector = Some(injector);
        self
    }

    /// Check if fault is currently active
    fn is_fault_active(&self) -> bool {
        self.fault_injector
            .as_ref()
            .map_or(false, FaultInjector::is_active)
    }

    /// Simulate a tool call
    pub fn call_tool(&self, tool_name: &str, _params: &str) -> Result<String, McpError> {
        // Check for injected faults
        if self.is_fault_active() {
            return Err(McpError::Unavailable("fault injected".to_string()));
        }

        match &self.state {
            McpConnectionState::Connected => {
                Ok(format!(r#"{{"result":"success","tool":"{tool_name}"}}"#))
            }
            McpConnectionState::ConnectionRefused => {
                Err(McpError::ConnectionRefused("localhost:8080".to_string()))
            }
            McpConnectionState::Timeout => Err(McpError::Timeout {
                operation: format!("call_tool({tool_name})"),
                elapsed: Duration::from_secs(30),
            }),
            McpConnectionState::InvalidResponse => Err(McpError::InvalidResponse {
                expected: "JSON object".to_string(),
                received: "<!DOCTYPE html>".to_string(),
            }),
            McpConnectionState::PartialResponse => Err(McpError::PartialResponse {
                bytes_received: 42,
                bytes_expected: 1024,
            }),
        }
    }

    /// Simulate tool call with graceful fallback
    pub fn call_tool_with_fallback(
        &self,
        tool_name: &str,
        params: &str,
        fallback: impl FnOnce() -> String,
    ) -> String {
        match self.call_tool(tool_name, params) {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("MCP call failed: {}. Using fallback.", e);
                fallback()
            }
        }
    }
}

impl Default for SimulatedMcpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper that provides graceful degradation for MCP operations
pub struct GracefulMcpClient {
    client: SimulatedMcpClient,
    degradation_enabled: bool,
}

impl GracefulMcpClient {
    /// Create a new graceful client
    #[must_use]
    pub fn new(client: SimulatedMcpClient) -> Self {
        Self {
            client,
            degradation_enabled: true,
        }
    }

    /// Disable graceful degradation (for strict mode testing)
    pub fn disable_degradation(&mut self) {
        self.degradation_enabled = false;
    }

    /// Call tool with graceful degradation
    pub fn call_tool(&self, tool_name: &str, params: &str) -> Result<String, McpError> {
        let result = self.client.call_tool(tool_name, params);

        match &result {
            Ok(_) => result,
            Err(e) if self.should_degrade(e) => {
                tracing::warn!("Degrading gracefully from MCP error: {}", e);
                Ok(self.degraded_response(tool_name))
            }
            Err(_) => result,
        }
    }

    /// Check if we should degrade for this error type
    fn should_degrade(&self, error: &McpError) -> bool {
        if !self.degradation_enabled {
            return false;
        }

        matches!(
            error,
            McpError::ConnectionRefused(_) | McpError::Timeout { .. } | McpError::Unavailable(_)
        )
    }

    /// Generate a degraded response
    fn degraded_response(&self, tool_name: &str) -> String {
        format!(
            r#"{{"result":"degraded","tool":"{tool_name}","message":"MCP unavailable, using cached/default response"}}"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ========== Connection Refused Tests ==========

    #[test]
    fn test_connection_refused_error_propagation() {
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::ConnectionRefused);

        let result = client.call_tool("test_tool", "{}");
        assert!(result.is_err());

        let err = result.err().expect("Already checked");
        match err {
            McpError::ConnectionRefused(addr) => {
                assert!(addr.contains("localhost"));
            }
            _ => panic!("Expected ConnectionRefused error"),
        }
    }

    #[test]
    fn test_connection_refused_graceful_degradation() {
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::ConnectionRefused);

        let graceful = GracefulMcpClient::new(client);
        let result = graceful.call_tool("test_tool", "{}");

        // Should succeed with degraded response
        assert!(result.is_ok());
        let response = result.expect("Already checked");
        assert!(response.contains("degraded"));
    }

    // ========== Timeout Tests ==========

    #[test]
    fn test_timeout_error_propagation() {
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::Timeout);

        let result = client.call_tool("slow_tool", "{}");
        assert!(result.is_err());

        let err = result.err().expect("Already checked");
        match err {
            McpError::Timeout { operation, elapsed } => {
                assert!(operation.contains("slow_tool"));
                assert!(elapsed >= Duration::from_secs(1));
            }
            _ => panic!("Expected Timeout error"),
        }
    }

    #[test]
    fn test_timeout_graceful_degradation() {
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::Timeout);

        let graceful = GracefulMcpClient::new(client);
        let result = graceful.call_tool("slow_tool", "{}");

        assert!(result.is_ok());
        assert!(result.expect("Already checked").contains("degraded"));
    }

    // ========== Invalid Response Tests ==========

    #[test]
    fn test_invalid_response_error_propagation() {
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::InvalidResponse);

        let result = client.call_tool("test_tool", "{}");
        assert!(result.is_err());

        let err = result.err().expect("Already checked");
        match err {
            McpError::InvalidResponse { expected, received } => {
                assert_eq!(expected, "JSON object");
                assert!(received.contains("DOCTYPE"));
            }
            _ => panic!("Expected InvalidResponse error"),
        }
    }

    #[test]
    fn test_invalid_response_no_graceful_degradation() {
        // Invalid responses should NOT trigger degradation (indicates bug, not transient failure)
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::InvalidResponse);

        let graceful = GracefulMcpClient::new(client);
        let result = graceful.call_tool("test_tool", "{}");

        // Should fail - invalid response is not a degradable condition
        assert!(result.is_err());
    }

    // ========== Partial Response Tests ==========

    #[test]
    fn test_partial_response_error() {
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::PartialResponse);

        let result = client.call_tool("test_tool", "{}");
        assert!(result.is_err());

        let err = result.err().expect("Already checked");
        match err {
            McpError::PartialResponse {
                bytes_received,
                bytes_expected,
            } => {
                assert!(bytes_received < bytes_expected);
            }
            _ => panic!("Expected PartialResponse error"),
        }
    }

    // ========== Fallback Tests ==========

    #[test]
    fn test_fallback_on_connection_refused() {
        let mut client = SimulatedMcpClient::new();
        client.set_state(McpConnectionState::ConnectionRefused);

        let result = client
            .call_tool_with_fallback("test_tool", "{}", || r#"{"result":"fallback"}"#.to_string());

        assert!(result.contains("fallback"));
    }

    #[test]
    fn test_fallback_not_used_when_connected() {
        let client = SimulatedMcpClient::new();

        let result = client.call_tool_with_fallback("test_tool", "{}", || {
            panic!("Fallback should not be called")
        });

        assert!(result.contains("success"));
    }

    // ========== Fault Injector Integration ==========

    #[test]
    fn test_fault_injector_mcp_scenario() {
        let injector = FaultInjector::new("mcp unavailable");
        let mut result = ChaosTestResult::new("mcp_connection_failure");

        let client = SimulatedMcpClient::new().with_fault_injector(injector.clone());

        // Phase 1: Normal operation
        let call_result = client.call_tool("test_tool", "{}");
        assert!(call_result.is_ok());
        result.add_degradation("Normal call succeeded");

        // Phase 2: Inject fault
        injector.inject();

        // Phase 3: Verify error propagation
        let call_result = client.call_tool("test_tool", "{}");
        assert!(call_result.is_err());
        result.add_propagated_error(format!("{:?}", call_result.err().expect("Already checked")));

        // Phase 4: Verify graceful degradation wrapper works
        let graceful =
            GracefulMcpClient::new(SimulatedMcpClient::new().with_fault_injector(injector.clone()));
        let graceful_result = graceful.call_tool("test_tool", "{}");
        assert!(graceful_result.is_ok());
        result.add_degradation("Graceful degradation worked");

        // Phase 5: Clear fault
        injector.clear();

        // Phase 6: Verify recovery
        let call_result = client.call_tool("test_tool", "{}");
        assert!(call_result.is_ok());
        result.add_recovery("Normal operation restored");

        assert!(result.passed);
    }

    // ========== No Panic Tests ==========

    #[test]
    fn test_no_panic_on_connection_refused() {
        let result = std::panic::catch_unwind(|| {
            let mut client = SimulatedMcpClient::new();
            client.set_state(McpConnectionState::ConnectionRefused);
            let _ = client.call_tool("any_tool", "{}");
        });
        assert!(result.is_ok(), "Should not panic on connection refused");
    }

    #[test]
    fn test_no_panic_on_timeout() {
        let result = std::panic::catch_unwind(|| {
            let mut client = SimulatedMcpClient::new();
            client.set_state(McpConnectionState::Timeout);
            let _ = client.call_tool("any_tool", "{}");
        });
        assert!(result.is_ok(), "Should not panic on timeout");
    }

    // ========== Property-Based Tests ==========

    proptest! {
        #[test]
        fn prop_error_display_never_panics(
            msg in ".*",
            op in ".*",
            secs in 0u64..1000,
        ) {
            let errors = vec![
                McpError::ConnectionRefused(msg.clone()),
                McpError::Timeout {
                    operation: op.clone(),
                    elapsed: Duration::from_secs(secs),
                },
                McpError::InvalidResponse {
                    expected: msg.clone(),
                    received: op.clone(),
                },
                McpError::PartialResponse {
                    bytes_received: 42,
                    bytes_expected: 1024,
                },
                McpError::Unavailable(msg),
            ];

            for error in errors {
                // Should never panic when formatting
                let _ = format!("{}", error);
            }
        }

        #[test]
        fn prop_graceful_degradation_never_panics(
            tool_name in "[a-zA-Z_][a-zA-Z0-9_]*",
            params in "\\{.*\\}",
        ) {
            let mut client = SimulatedMcpClient::new();
            client.set_state(McpConnectionState::ConnectionRefused);

            let graceful = GracefulMcpClient::new(client);

            // Should never panic regardless of input
            let _ = graceful.call_tool(&tool_name, &params);
        }
    }
}
