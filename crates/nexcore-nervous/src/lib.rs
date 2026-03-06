//! # nexcore-nervous
//!
//! The Nervous System for NexCore — event routing, reflex arcs, myelination,
//! sensory integration, and motor coordination.
//!
//! Maps biological nervous system components to Claude Code's event bus, hook dispatch,
//! signaling, and routing infrastructure.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "Nervous-system mapping uses closed biological DTOs and bounded ratio/state counters"
)]

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

pub mod claude_code;
pub mod grounding;

/// Type of neuron in the nervous system
///
/// **Biological mapping:** Neuron classification by function
/// **Tier:** T1 (Σ sum + μ mapping)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeuronType {
    /// Sensory neuron — detects input
    Sensory,
    /// Motor neuron — executes output
    Motor,
    /// Interneuron — relay processing
    Inter,
}

/// Nerve impulse being routed through the system
///
/// **Biological mapping:** Action potential traveling along nerve pathway
/// **Tier:** T2-C (→ causality + σ sequence + ν frequency)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Impulse {
    /// Source of the impulse
    pub source: String,
    /// Target destination
    pub target: String,
    /// Payload data
    pub payload: String,
    /// Priority (0-255, higher = more urgent)
    pub priority: u8,
    /// When the impulse was generated
    pub timestamp: DateTime,
}

/// Fast local hook dispatch without brain involvement
///
/// **Biological mapping:** Spinal reflex arc (e.g., withdrawal reflex)
/// **Tier:** T2-C (→ causality + σ sequence + ν frequency)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflexArc {
    /// Pattern that triggers this reflex
    pub trigger_pattern: String,
    /// Action to execute when triggered
    pub response_action: String,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Number of times this reflex has fired
    pub invocation_count: u64,
}

impl ReflexArc {
    /// Create a new reflex arc
    pub fn new(trigger: impl Into<String>, response: impl Into<String>) -> Self {
        Self {
            trigger_pattern: trigger.into(),
            response_action: response.into(),
            latency_ms: 10, // Reflex speed ~0.01ms simulated as 10ms
            invocation_count: 0,
        }
    }

    /// Invoke this reflex arc (increments invocation count)
    pub fn invoke(&mut self) {
        self.invocation_count += 1;
    }
}

/// Hot path caching for frequently accessed patterns
///
/// **Biological mapping:** Myelin sheath that speeds up signal transmission
/// **Tier:** T2-C (π persistence + ν frequency + κ comparison)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Myelination {
    /// Pattern being cached
    pub pattern: String,
    /// Cached response
    pub cached_response: String,
    /// Number of cache hits
    pub hit_count: u64,
    /// Number of cache misses
    pub miss_count: u64,
}

impl Myelination {
    /// Create a new myelination cache entry
    pub fn new(pattern: impl Into<String>, cached: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            cached_response: cached.into(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Lookup a pattern in the cache
    pub fn lookup(&mut self, pattern: &str) -> Option<&str> {
        if self.pattern == pattern {
            self.hit_count += 1;
            Some(&self.cached_response)
        } else {
            self.miss_count += 1;
            None
        }
    }

    /// Get the hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

/// Input detected from environment
///
/// **Biological mapping:** Sensory receptor activation
/// **Tier:** T2-P (∂ boundary + N quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensoryInput {
    /// Source of the input
    pub source: String,
    /// Raw data
    pub raw_data: String,
    /// Intensity (clamped 0.0-1.0)
    pub intensity: f64,
}

impl SensoryInput {
    /// Create a new sensory input with intensity clamping
    pub fn new(source: impl Into<String>, raw_data: impl Into<String>, intensity: f64) -> Self {
        Self {
            source: source.into(),
            raw_data: raw_data.into(),
            intensity: intensity.clamp(0.0, 1.0),
        }
    }
}

/// Command to execute via motor neuron
///
/// **Biological mapping:** Motor neuron activation of muscle
/// **Tier:** T2-C (→ causality + μ mapping)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotorCommand {
    /// Target tool to invoke
    pub target_tool: String,
    /// Parameters as JSON string
    pub params_json: String,
    /// Urgency (0-255, higher = more urgent)
    pub urgency: u8,
}

/// Speed of signal transmission
///
/// **Biological mapping:** Signal conduction velocity in different nerve types
/// **Tier:** T1 (Σ sum + ν frequency)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalSpeed {
    /// Unmyelinated nerve (slow ~1ms)
    Unmyelinated,
    /// Myelinated nerve (fast ~0.1ms)
    Myelinated,
    /// Reflex arc (fastest ~0.01ms)
    Reflex,
}

impl SignalSpeed {
    /// Get the approximate latency in milliseconds
    pub fn latency_ms(&self) -> f64 {
        match self {
            SignalSpeed::Unmyelinated => 1.0,
            SignalSpeed::Myelinated => 0.1,
            SignalSpeed::Reflex => 0.01,
        }
    }
}

/// Health status of the nervous system
///
/// **Biological mapping:** Neurological health assessment
/// **Tier:** T2-C (ς state + κ comparison + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NervousHealth {
    /// Number of active neurons
    pub neuron_count: usize,
    /// Number of active reflex arcs
    pub reflex_arcs_active: usize,
    /// Ratio of myelinated to unmyelinated pathways
    pub myelination_ratio: f64,
    /// Average signal latency in milliseconds
    pub avg_signal_latency_ms: f64,
    /// Whether sensory integration is functioning
    pub sensory_integration_ok: bool,
}

impl NervousHealth {
    /// Check if the nervous system is healthy
    pub fn is_healthy(&self) -> bool {
        self.neuron_count > 0 && self.myelination_ratio > 0.3 && self.avg_signal_latency_ms < 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_type_variants() {
        let sensory = NeuronType::Sensory;
        let motor = NeuronType::Motor;
        let inter = NeuronType::Inter;

        assert!(sensory != motor);
        assert!(motor != inter);
        assert!(sensory != inter);
    }

    #[test]
    fn test_impulse_creation() {
        let impulse = Impulse {
            source: "sensor".to_string(),
            target: "motor".to_string(),
            payload: "data".to_string(),
            priority: 100,
            timestamp: DateTime::now(),
        };

        assert_eq!(impulse.source, "sensor");
        assert_eq!(impulse.priority, 100);
    }

    #[test]
    fn test_reflex_arc_new() {
        let arc = ReflexArc::new("pain", "withdraw");
        assert_eq!(arc.trigger_pattern, "pain");
        assert_eq!(arc.response_action, "withdraw");
        assert_eq!(arc.invocation_count, 0);
    }

    #[test]
    fn test_reflex_arc_invoke() {
        let mut arc = ReflexArc::new("trigger", "response");
        assert_eq!(arc.invocation_count, 0);

        arc.invoke();
        assert_eq!(arc.invocation_count, 1);

        arc.invoke();
        assert_eq!(arc.invocation_count, 2);
    }

    #[test]
    fn test_myelination_new() {
        let myelin = Myelination::new("pattern", "cached");
        assert_eq!(myelin.pattern, "pattern");
        assert_eq!(myelin.cached_response, "cached");
        assert_eq!(myelin.hit_count, 0);
        assert_eq!(myelin.miss_count, 0);
    }

    #[test]
    fn test_myelination_lookup_hit() {
        let mut myelin = Myelination::new("test", "result");
        let result = myelin.lookup("test");
        assert_eq!(result, Some("result"));
        assert_eq!(myelin.hit_count, 1);
        assert_eq!(myelin.miss_count, 0);
    }

    #[test]
    fn test_myelination_lookup_miss() {
        let mut myelin = Myelination::new("test", "result");
        let result = myelin.lookup("other");
        assert_eq!(result, None);
        assert_eq!(myelin.hit_count, 0);
        assert_eq!(myelin.miss_count, 1);
    }

    #[test]
    fn test_myelination_hit_rate() {
        let mut myelin = Myelination::new("test", "result");
        assert_eq!(myelin.hit_rate(), 0.0);

        myelin.lookup("test"); // hit
        assert_eq!(myelin.hit_rate(), 1.0);

        myelin.lookup("other"); // miss
        assert_eq!(myelin.hit_rate(), 0.5);
    }

    #[test]
    fn test_sensory_input_new() {
        let input = SensoryInput::new("sensor", "data", 0.8);
        assert_eq!(input.source, "sensor");
        assert_eq!(input.intensity, 0.8);
    }

    #[test]
    fn test_sensory_input_intensity_clamping() {
        let input = SensoryInput::new("sensor", "data", 1.5);
        assert_eq!(input.intensity, 1.0);

        let input2 = SensoryInput::new("sensor", "data", -0.5);
        assert_eq!(input2.intensity, 0.0);
    }

    #[test]
    fn test_motor_command_creation() {
        let cmd = MotorCommand {
            target_tool: "bash".to_string(),
            params_json: r#"{"cmd":"ls"}"#.to_string(),
            urgency: 200,
        };

        assert_eq!(cmd.target_tool, "bash");
        assert_eq!(cmd.urgency, 200);
    }

    #[test]
    fn test_signal_speed_latency() {
        assert_eq!(SignalSpeed::Unmyelinated.latency_ms(), 1.0);
        assert_eq!(SignalSpeed::Myelinated.latency_ms(), 0.1);
        assert_eq!(SignalSpeed::Reflex.latency_ms(), 0.01);
    }

    #[test]
    fn test_nervous_health_is_healthy() {
        let healthy = NervousHealth {
            neuron_count: 10,
            reflex_arcs_active: 5,
            myelination_ratio: 0.5,
            avg_signal_latency_ms: 50.0,
            sensory_integration_ok: true,
        };
        assert!(healthy.is_healthy());

        let unhealthy = NervousHealth {
            neuron_count: 0,
            reflex_arcs_active: 0,
            myelination_ratio: 0.1,
            avg_signal_latency_ms: 200.0,
            sensory_integration_ok: false,
        };
        assert!(!unhealthy.is_healthy());
    }

    #[test]
    fn test_nervous_health_low_myelination() {
        let health = NervousHealth {
            neuron_count: 10,
            reflex_arcs_active: 5,
            myelination_ratio: 0.2,
            avg_signal_latency_ms: 50.0,
            sensory_integration_ok: true,
        };
        assert!(!health.is_healthy());
    }
}
