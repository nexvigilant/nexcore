//! Usage metering — track compute, AI tokens, and MCP calls per session.
//!
//! Usage counters are maintained per-session and aggregated monthly per tenant
//! for billing purposes.

use serde::{Deserialize, Serialize};
use vr_core::ids::{TenantId, TerminalSessionId};

/// Usage counters for a single terminal session.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionUsage {
    /// Session this usage belongs to.
    pub session_id: TerminalSessionId,
    /// Owning tenant (for aggregation).
    pub tenant_id: TenantId,
    /// Wall-clock compute seconds consumed.
    pub compute_seconds: u64,
    /// AI input tokens consumed.
    pub ai_input_tokens: u64,
    /// AI output tokens consumed.
    pub ai_output_tokens: u64,
    /// MCP tool invocations made.
    pub mcp_calls: u64,
}

impl SessionUsage {
    /// Create a new zero-usage record for a session.
    #[must_use]
    pub fn new(session_id: TerminalSessionId, tenant_id: TenantId) -> Self {
        Self {
            session_id,
            tenant_id,
            compute_seconds: 0,
            ai_input_tokens: 0,
            ai_output_tokens: 0,
            mcp_calls: 0,
        }
    }

    /// Record AI token usage.
    pub fn record_ai_tokens(&mut self, input: u64, output: u64) {
        self.ai_input_tokens = self.ai_input_tokens.saturating_add(input);
        self.ai_output_tokens = self.ai_output_tokens.saturating_add(output);
    }

    /// Record an MCP tool call.
    pub fn record_mcp_call(&mut self) {
        self.mcp_calls = self.mcp_calls.saturating_add(1);
    }

    /// Record compute time.
    pub fn record_compute(&mut self, seconds: u64) {
        self.compute_seconds = self.compute_seconds.saturating_add(seconds);
    }

    /// Total AI tokens (input + output).
    #[must_use]
    pub fn total_ai_tokens(&self) -> u64 {
        self.ai_input_tokens.saturating_add(self.ai_output_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_usage_is_zero() {
        let usage = SessionUsage::new(TerminalSessionId::new(), TenantId::new());
        assert_eq!(usage.compute_seconds, 0);
        assert_eq!(usage.total_ai_tokens(), 0);
        assert_eq!(usage.mcp_calls, 0);
    }

    #[test]
    fn record_ai_tokens_accumulates() {
        let mut usage = SessionUsage::new(TerminalSessionId::new(), TenantId::new());
        usage.record_ai_tokens(100, 200);
        usage.record_ai_tokens(50, 75);
        assert_eq!(usage.ai_input_tokens, 150);
        assert_eq!(usage.ai_output_tokens, 275);
        assert_eq!(usage.total_ai_tokens(), 425);
    }

    #[test]
    fn saturating_prevents_overflow() {
        let mut usage = SessionUsage::new(TerminalSessionId::new(), TenantId::new());
        usage.mcp_calls = u64::MAX;
        usage.record_mcp_call();
        assert_eq!(usage.mcp_calls, u64::MAX);
    }
}
