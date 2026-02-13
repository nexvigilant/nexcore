//! Agent Auto-Trigger Detection Module
//!
//! This module provides detection logic for automatically triggering
//! specialized Rust subagents based on:
//! - File content patterns (unsafe, async, macros, FFI)
//! - Compiler error codes (borrow, lifetime, trait errors)
//! - User intent keywords (migrations, optimizations)
//! - Agent completion chaining
//!
//! The detection is fully automatic - when patterns are detected,
//! agents are auto-triggered via context injection.

pub mod errors;
pub mod intents;
pub mod patterns;

use crate::protocol::HookOutput;

/// Result of a detection operation
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// The agent to trigger
    pub agent: &'static str,
    /// What was detected
    pub detected: String,
    /// Reason for triggering
    pub reason: &'static str,
    /// Auto-generated prompt for the agent
    pub auto_prompt: String,
}

impl DetectionResult {
    /// Create a new detection result
    pub fn new(
        agent: &'static str,
        detected: impl Into<String>,
        reason: &'static str,
        auto_prompt: impl Into<String>,
    ) -> Self {
        Self {
            agent,
            detected: detected.into(),
            reason,
            auto_prompt: auto_prompt.into(),
        }
    }

    /// Format as context injection output
    pub fn to_context(&self) -> String {
        format!(
            r#"
🤖 AUTO-TRIGGERING RUST AGENT ─────────────────────────────
   Detected: {}
   Agent: {}
   Reason: {}

   ⚡ AUTOMATIC ACTION: Invoking {} subagent now.

   Use Task tool with:
     subagent_type: "{}"
     prompt: "{}"
───────────────────────────────────────────────────────────"#,
            self.detected,
            self.agent,
            self.reason,
            self.agent,
            self.agent,
            self.auto_prompt.replace('"', r#"\""#)
        )
    }

    /// Convert to HookOutput for JSON serialization
    pub fn to_hook_output(&self) -> HookOutput {
        HookOutput::with_context(self.to_context())
    }
}

/// Agent chaining configuration
pub struct AgentChain {
    /// Agent that just completed
    pub completed: &'static str,
    /// Agent to trigger next
    pub next: &'static str,
    /// Condition (e.g., "with_errors" or "always")
    pub condition: &'static str,
}

/// Chaining map for agent completion
pub const AGENT_CHAINS: &[AgentChain] = &[
    AgentChain {
        completed: "rust-toolchain",
        next: "rust-compiler-doctor",
        condition: "with_errors",
    },
    AgentChain {
        completed: "rust-compiler-doctor",
        next: "rust-test-architect",
        condition: "always",
    },
    AgentChain {
        completed: "rust-borrow-doctor",
        next: "rust-test-architect",
        condition: "always",
    },
    AgentChain {
        completed: "rust-migrator",
        next: "rust-reviewer",
        condition: "always",
    },
    AgentChain {
        completed: "rust-c-migrator",
        next: "rust-reviewer",
        condition: "always",
    },
    AgentChain {
        completed: "rust-go-migrator",
        next: "rust-reviewer",
        condition: "always",
    },
    AgentChain {
        completed: "rust-js-migrator",
        next: "rust-reviewer",
        condition: "always",
    },
    AgentChain {
        completed: "rust-reviewer",
        next: "rust-test-architect",
        condition: "always",
    },
    AgentChain {
        completed: "rust-unsafe-specialist",
        next: "rust-ffi-bridge",
        condition: "ffi_detected",
    },
    AgentChain {
        completed: "rust-optimize",
        next: "rust-binary-optimizer",
        condition: "always",
    },
    AgentChain {
        completed: "rust-build-systems",
        next: "rust-release",
        condition: "always",
    },
    AgentChain {
        completed: "rust-architect",
        next: "rust-docs",
        condition: "always",
    },
    // ==========================================================================
    // FORGE: Autonomous Primitive Mining & Code Generation Loop
    // ==========================================================================
    // Entry: forge-primitive-miner extracts patterns
    // Loop: miner -> codifier -> validator -> (iterate or exit)
    // Exit: escape word detected or stability threshold reached
    AgentChain {
        completed: "forge-primitive-miner",
        next: "forge-codifier",
        condition: "always",
    },
    AgentChain {
        completed: "forge-codifier",
        next: "forge-validator",
        condition: "always",
    },
    AgentChain {
        completed: "forge-validator",
        next: "forge-primitive-miner",
        condition: "needs_refinement", // Loop back if not stable
    },
    AgentChain {
        completed: "forge-validator",
        next: "forge-distributor",
        condition: "stable", // Exit to distribution when stable
    },
    AgentChain {
        completed: "forge-distributor",
        next: "rust-test-architect",
        condition: "always", // Final verification
    },
];

/// Get the next agent in the chain
///
/// # Arguments
/// * `completed` - The agent that just completed
/// * `has_errors` - Whether the completed agent encountered errors
///
/// # Forge Loop Conditions
/// * `needs_refinement` - Validator found issues, loop back to miner
/// * `stable` - Validator confirmed stability, proceed to distribution
pub fn get_next_agent(completed: &str, has_errors: bool) -> Option<&'static AgentChain> {
    AGENT_CHAINS.iter().find(|chain| {
        chain.completed == completed
            && match chain.condition {
                "always" => true,
                "with_errors" => has_errors,
                "ffi_detected" => true, // Would need context to check
                "needs_refinement" => has_errors, // Errors = needs more work
                "stable" => !has_errors, // No errors = stable, can distribute
                _ => false,
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_result_format() {
        let result = DetectionResult::new(
            "rust-async-expert",
            "async fn detected",
            "Async code pattern found",
            "Analyze the async code for proper patterns",
        );

        let context = result.to_context();
        assert!(context.contains("rust-async-expert"));
        assert!(context.contains("AUTO-TRIGGERING"));
    }

    #[test]
    fn test_agent_chaining_with_errors() {
        // With errors, toolchain -> compiler-doctor
        let chain = get_next_agent("rust-toolchain", true);
        assert!(
            chain.is_some(),
            "Expected chain for rust-toolchain with errors"
        );
        assert_eq!(chain.map(|c| c.next), Some("rust-compiler-doctor"));
    }

    #[test]
    fn test_agent_chaining_without_errors() {
        // Without errors, no chain for toolchain
        let chain = get_next_agent("rust-toolchain", false);
        assert!(chain.is_none());
    }

    #[test]
    fn test_agent_chaining_migrator() {
        // Migrator -> reviewer (always)
        let chain = get_next_agent("rust-migrator", false);
        assert!(chain.is_some(), "Expected chain for rust-migrator");
        assert_eq!(chain.map(|c| c.next), Some("rust-reviewer"));
    }
}
