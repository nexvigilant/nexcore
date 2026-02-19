//! # Claude Code Nervous System
//!
//! Claude Code specific nervous system types mapping hook events, context assembly,
//! tool chains, and event routing to biological nervous system components.

use serde::{Deserialize, Serialize};

/// Hook event types that trigger nervous system responses
///
/// **Biological mapping:** Types of nerve impulses
/// **Tier:** T1 (Σ sum + ν frequency)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    /// Session initialization
    SessionStart,
    /// Before tool execution
    PreToolUse,
    /// After successful tool execution
    PostToolUse,
    /// After failed tool execution
    PostToolUseFailure,
    /// Session termination
    Stop,
    /// Subagent spawned
    SubagentStart,
    /// Subagent completed
    SubagentEnd,
}

/// Hook dispatch record — the reflex arc in action
///
/// **Biological mapping:** Reflex arc execution
/// **Tier:** T2-C (→ causality + σ sequence + ν frequency)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookDispatch {
    /// Event that triggered this dispatch
    pub event: HookEvent,
    /// Optional pattern matcher
    pub matcher: Option<String>,
    /// Name of the hook being dispatched
    pub hook_name: String,
    /// Latency in milliseconds
    pub latency_ms: u64,
}

/// Chain of hooks executing in sequence
///
/// **Biological mapping:** Compound nerve pathway (polysynaptic reflex)
/// **Tier:** T2-C (σ sequence + → causality + N quantity)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookChain {
    /// Hooks in the chain
    pub hooks: Vec<String>,
    /// Whether hooks execute sequentially
    pub sequential: bool,
    /// Total latency across all hooks
    pub total_latency_ms: u64,
}

impl HookChain {
    /// Create a new sequential hook chain
    pub fn sequential(hooks: Vec<String>) -> Self {
        Self {
            hooks,
            sequential: true,
            total_latency_ms: 0,
        }
    }

    /// Create a new parallel hook chain
    pub fn parallel(hooks: Vec<String>) -> Self {
        Self {
            hooks,
            sequential: false,
            total_latency_ms: 0,
        }
    }

    /// Get the number of hooks in the chain
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

/// Event bus routing configuration
///
/// **Biological mapping:** Nerve pathway routing
/// **Tier:** T2-C (→ causality + λ location + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventBusRoute {
    /// Source of the event
    pub source: String,
    /// Destination for the event
    pub destination: String,
    /// Priority (0-255, higher = more urgent)
    pub priority: u8,
    /// Whether this route is myelinated (cached)
    pub myelinated: bool,
}

/// Context assembly from multiple sources
///
/// **Biological mapping:** Sensory integration in the brain
/// **Tier:** T2-C (Σ sum + μ mapping + N quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextAssembly {
    /// Sources contributing to the context
    pub sources: Vec<String>,
    /// Total token count
    pub total_tokens: usize,
    /// Integration quality (0.0-1.0)
    pub integration_quality: f64,
}

impl ContextAssembly {
    /// Create a new context assembly
    pub fn new(sources: Vec<String>, total_tokens: usize) -> Self {
        Self {
            sources,
            total_tokens,
            integration_quality: 0.8, // Default quality
        }
    }

    /// Get the number of sources
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

/// Tool chain execution tracking
///
/// **Biological mapping:** Motor neuron coordination (muscle synergy)
/// **Tier:** T2-C (σ sequence + → causality + N quantity)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolChain {
    /// Tools in the chain
    pub tools: Vec<String>,
    /// Whether tools execute in parallel
    pub parallel: bool,
    /// Number of completed tools
    pub completed: usize,
    /// Total number of tools
    pub total: usize,
}

impl ToolChain {
    /// Create a new tool chain
    pub fn new(tools: Vec<String>, parallel: bool) -> Self {
        let total = tools.len();
        Self {
            tools,
            parallel,
            completed: 0,
            total,
        }
    }

    /// Mark a tool as completed
    pub fn mark_completed(&mut self) {
        if self.completed < self.total {
            self.completed += 1;
        }
    }

    /// Check if all tools are completed
    pub fn is_complete(&self) -> bool {
        self.completed >= self.total
    }

    /// Get completion ratio (0.0-1.0)
    pub fn completion_ratio(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.completed as f64 / self.total as f64
        }
    }
}

/// Signal latency breakdown
///
/// **Biological mapping:** Nerve conduction time measurement
/// **Tier:** T2-C (N quantity + ν frequency + κ comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalLatency {
    /// Hook execution time
    pub hook_ms: u64,
    /// Event bus routing time
    pub event_bus_ms: u64,
    /// Tool dispatch time
    pub tool_dispatch_ms: u64,
}

impl SignalLatency {
    /// Calculate total latency
    pub fn total(&self) -> u64 {
        self.hook_ms + self.event_bus_ms + self.tool_dispatch_ms
    }
}

/// Myelination cache statistics
///
/// **Biological mapping:** Myelin health assessment
/// **Tier:** T2-C (π persistence + κ comparison + N quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MyelinationCache {
    /// Pattern being cached
    pub pattern: String,
    /// Hit rate (0.0-1.0)
    pub hit_rate: f64,
    /// Number of entries in cache
    pub entries: usize,
    /// Maximum cache capacity
    pub max_entries: usize,
}

impl MyelinationCache {
    /// Check if the cache is full
    pub fn is_full(&self) -> bool {
        self.entries >= self.max_entries
    }

    /// Get the fill ratio (0.0-1.0)
    pub fn fill_ratio(&self) -> f64 {
        if self.max_entries == 0 {
            0.0
        } else {
            self.entries as f64 / self.max_entries as f64
        }
    }
}

/// Reflex response configuration
///
/// **Biological mapping:** Spinal reflex that bypasses brain
/// **Tier:** T2-C (→ causality + ν frequency + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflexResponse {
    /// Trigger pattern
    pub trigger: String,
    /// Action to execute
    pub action: String,
    /// Whether this reflex bypasses the brain (model)
    pub bypasses_brain: bool,
}

impl ReflexResponse {
    /// Create a new reflex response that bypasses the brain
    pub fn bypass(trigger: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            trigger: trigger.into(),
            action: action.into(),
            bypasses_brain: true,
        }
    }

    /// Create a new reflex response that involves the brain
    pub fn with_brain(trigger: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            trigger: trigger.into(),
            action: action.into(),
            bypasses_brain: false,
        }
    }
}

/// Overall nervous system health
///
/// **Biological mapping:** Neurological examination results
/// **Tier:** T2-C (ς state + κ comparison + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NervousSystemHealth {
    /// Number of active hooks
    pub hook_count: usize,
    /// Whether event bus is active
    pub event_bus_active: bool,
    /// Average hook latency in milliseconds
    pub avg_hook_latency_ms: f64,
    /// Myelination cache hit rate
    pub myelination_hit_rate: f64,
    /// Number of active reflex arcs
    pub reflex_arcs_active: usize,
}

impl NervousSystemHealth {
    /// Check if the nervous system is healthy
    pub fn is_healthy(&self) -> bool {
        self.event_bus_active && self.avg_hook_latency_ms < 100.0 && self.myelination_hit_rate > 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_variants() {
        let events = vec![
            HookEvent::SessionStart,
            HookEvent::PreToolUse,
            HookEvent::PostToolUse,
            HookEvent::PostToolUseFailure,
            HookEvent::Stop,
            HookEvent::SubagentStart,
            HookEvent::SubagentEnd,
        ];
        assert_eq!(events.len(), 7);
    }

    #[test]
    fn test_hook_dispatch_creation() {
        let dispatch = HookDispatch {
            event: HookEvent::PreToolUse,
            matcher: Some("Bash".to_string()),
            hook_name: "timeout-enforcer".to_string(),
            latency_ms: 5,
        };
        assert_eq!(dispatch.hook_name, "timeout-enforcer");
    }

    #[test]
    fn test_hook_chain_sequential() {
        let chain = HookChain::sequential(vec!["hook1".to_string(), "hook2".to_string()]);
        assert!(chain.sequential);
        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_hook_chain_parallel() {
        let chain = HookChain::parallel(vec!["hook1".to_string()]);
        assert!(!chain.sequential);
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_event_bus_route() {
        let route = EventBusRoute {
            source: "guardian".to_string(),
            destination: "signal-detector".to_string(),
            priority: 100,
            myelinated: true,
        };
        assert!(route.myelinated);
    }

    #[test]
    fn test_context_assembly_new() {
        let ctx = ContextAssembly::new(vec!["source1".to_string(), "source2".to_string()], 1000);
        assert_eq!(ctx.source_count(), 2);
        assert_eq!(ctx.total_tokens, 1000);
    }

    #[test]
    fn test_tool_chain_new() {
        let chain = ToolChain::new(vec!["tool1".to_string(), "tool2".to_string()], false);
        assert_eq!(chain.total, 2);
        assert_eq!(chain.completed, 0);
        assert!(!chain.is_complete());
    }

    #[test]
    fn test_tool_chain_completion() {
        let mut chain = ToolChain::new(vec!["tool1".to_string()], false);
        assert_eq!(chain.completion_ratio(), 0.0);

        chain.mark_completed();
        assert_eq!(chain.completion_ratio(), 1.0);
        assert!(chain.is_complete());
    }

    #[test]
    fn test_signal_latency_total() {
        let latency = SignalLatency {
            hook_ms: 10,
            event_bus_ms: 5,
            tool_dispatch_ms: 15,
        };
        assert_eq!(latency.total(), 30);
    }

    #[test]
    fn test_myelination_cache_full() {
        let cache = MyelinationCache {
            pattern: "test".to_string(),
            hit_rate: 0.8,
            entries: 100,
            max_entries: 100,
        };
        assert!(cache.is_full());
        assert_eq!(cache.fill_ratio(), 1.0);
    }

    #[test]
    fn test_myelination_cache_not_full() {
        let cache = MyelinationCache {
            pattern: "test".to_string(),
            hit_rate: 0.8,
            entries: 50,
            max_entries: 100,
        };
        assert!(!cache.is_full());
        assert_eq!(cache.fill_ratio(), 0.5);
    }

    #[test]
    fn test_reflex_response_bypass() {
        let reflex = ReflexResponse::bypass("trigger", "action");
        assert!(reflex.bypasses_brain);
    }

    #[test]
    fn test_reflex_response_with_brain() {
        let reflex = ReflexResponse::with_brain("trigger", "action");
        assert!(!reflex.bypasses_brain);
    }

    #[test]
    fn test_nervous_system_health_healthy() {
        let health = NervousSystemHealth {
            hook_count: 10,
            event_bus_active: true,
            avg_hook_latency_ms: 50.0,
            myelination_hit_rate: 0.7,
            reflex_arcs_active: 5,
        };
        assert!(health.is_healthy());
    }

    #[test]
    fn test_nervous_system_health_unhealthy() {
        let health = NervousSystemHealth {
            hook_count: 10,
            event_bus_active: false,
            avg_hook_latency_ms: 50.0,
            myelination_hit_rate: 0.7,
            reflex_arcs_active: 5,
        };
        assert!(!health.is_healthy());
    }
}
