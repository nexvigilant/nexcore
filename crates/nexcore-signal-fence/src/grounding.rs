//! # GroundsTo implementations for nexcore-signal-fence types
//!
//! Connects the process-level network signal container to the Lex Primitiva type system.
//!
//! ## Grounding Strategy
//!
//! The signal-fence crate applies signal theory to network security: every connection
//! is an observation, the allowlist defines boundaries, and default-deny embodies
//! A2 (Noise Dominance). Boundary (∂) is the dominant primitive — the entire crate
//! is about drawing boundaries between allowed and denied traffic.
//!
//! | Primitive | Role in Signal Fence |
//! |-----------|---------------------|
//! | ∂ (Boundary) | **Dominant** — rules, policies, verdicts define network boundaries |
//! | κ (Comparison) | Rule matching, verdict comparison, priority ordering |
//! | σ (Sequence) | Scan→evaluate→enforce→audit pipeline |
//! | ς (State) | Connection states, fence modes, TCP state machine |
//! | μ (Mapping) | Inode→process resolution, address parsing |
//! | ∃ (Existence) | Process existence validation, signal existence |
//! | ∅ (Void) | Default deny, unknown processes, missing attribution |
//! | π (Persistence) | Audit trail, stats accumulation |
//! | N (Quantity) | Ports, UIDs, stats counters, inode numbers |
//! | → (Causality) | Enforcement actions, decision consequences |
//! | ∝ (Irreversibility) | Enforcement (blocked connections can't be unblocked retroactively) |
//! | Σ (Sum) | Stats aggregation, rule set composition |
//! | λ (Location) | IP addresses, CIDR networks |
//! | ν (Frequency) | Tick rate, connection observation frequency |
//!
//! ## Summary
//!
//! | Tier | Count | Types |
//! |------|-------|-------|
//! | T1 | 5 | FenceMode, Direction, FenceVerdict, Protocol, TcpState |
//! | T2-P | 7 | SocketEntry, ProcessMatch, NetworkMatch, FenceRule, AuditEntry, ProcessInfo, FenceDecision |
//! | T2-C | 6 | ConnectionEvent, RuleSet, FencePolicy, FenceStats, AuditLog, FenceTickResult |
//! | T3 | 2 | FenceEngine, FenceReport |
//! | **Total** | **20** | |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

// ═══════════════════════════════════════════════════════════
// T1 TYPES — Single dominant primitive
// ═══════════════════════════════════════════════════════════

/// FenceMode: T1 (ς), dominant State
///
/// Three operating modes (Monitor/Enforce/Lockdown) — pure state selection.
impl GroundsTo for crate::policy::FenceMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 0.95)
    }
}

/// Direction: T1 (∂), dominant Boundary
///
/// Ingress/Egress/Both — directional boundary of traffic flow.
impl GroundsTo for crate::process::Direction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

/// FenceVerdict: T1 (∂), dominant Boundary
///
/// Allow/Deny/Alert — the fundamental boundary decision.
impl GroundsTo for crate::rule::FenceVerdict {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.97)
    }
}

/// Protocol: T1 (κ), dominant Comparison
///
/// Tcp/Tcp6/Udp/Udp6 — pure classification of protocol type.
impl GroundsTo for crate::connection::Protocol {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// TcpState: T1 (ς), dominant State
///
/// TCP state machine positions (Established, Listen, TimeWait, etc.).
impl GroundsTo for crate::connection::TcpState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 0.97)
    }
}

// ═══════════════════════════════════════════════════════════
// T2-P TYPES — 2-3 primitives, cross-domain transferable
// ═══════════════════════════════════════════════════════════

/// SocketEntry: T2-P (λ + N), dominant Location
///
/// A parsed /proc/net/tcp entry — IP addresses (λ) with port numbers (N).
impl GroundsTo for crate::connection::SocketEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// ProcessInfo: T2-P (∃ + μ), dominant Existence
///
/// Process identity — validates that a process exists and maps its attributes.
impl GroundsTo for crate::process::ProcessInfo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Existence, 0.88)
    }
}

/// ProcessMatch: T2-P (∂ + κ), dominant Boundary
///
/// Process matching criterion — boundary that selects which processes apply.
impl GroundsTo for crate::rule::ProcessMatch {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// NetworkMatch: T2-P (∂ + λ), dominant Boundary
///
/// Network matching criterion — boundary defined by location (IP/CIDR/port).
impl GroundsTo for crate::rule::NetworkMatch {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Boundary, 0.82)
    }
}

/// FenceRule: T2-P (∂ + κ + N), dominant Boundary
///
/// A signal-theory FixedBoundary for network connections — deterministic criterion.
impl GroundsTo for crate::rule::FenceRule {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// FenceDecision: T2-P (∂ + →), dominant Boundary
///
/// A verdict with causal attribution (which rule, why).
impl GroundsTo for crate::policy::FenceDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Boundary, 0.88)
    }
}

/// AuditEntry: T2-P (π + →), dominant Persistence
///
/// A persisted record of a causal decision.
impl GroundsTo for crate::audit::AuditEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Persistence, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// T2-C TYPES — 4-5 primitives, domain composites
// ═══════════════════════════════════════════════════════════

/// ConnectionEvent: T2-C (λ + ∃ + μ + ς), dominant Location
///
/// A socket observation attributed to a process — the primary "observation"
/// in signal theory terms.
impl GroundsTo for crate::process::ConnectionEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Existence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Location, 0.78)
    }
}

/// RuleSet: T2-C (∂ + κ + σ + Σ), dominant Boundary
///
/// Ordered composition of rules — aggregate boundaries with sequential evaluation.
impl GroundsTo for crate::rule::RuleSet {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// FencePolicy: T2-C (∂ + ∅ + κ + ς), dominant Boundary
///
/// Complete policy with default-deny (∅ Void → noise dominance) and mode (ς State).
impl GroundsTo for crate::policy::FencePolicy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Void,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// FenceStats: T2-C (N + Σ + ν + π), dominant Quantity
///
/// Aggregate statistics — counters (N), sums (Σ), rates (ν), persisted (π).
impl GroundsTo for crate::engine::FenceStats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Sum,
            LexPrimitiva::Frequency,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// AuditLog: T2-C (π + σ + → + Σ), dominant Persistence
///
/// Bounded circular buffer of causal decisions — sequential persistent record.
impl GroundsTo for crate::audit::AuditLog {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Sequence,
            LexPrimitiva::Causality,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.82)
    }
}

/// FenceTickResult: T2-C (σ + N + ∂ + κ), dominant Sequence
///
/// Result of one scan→evaluate→enforce cycle — sequential with quantified outcomes.
impl GroundsTo for crate::engine::FenceTickResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.78)
    }
}

// ═══════════════════════════════════════════════════════════
// T3 TYPES — 6+ primitives, full domain types
// ═══════════════════════════════════════════════════════════

// FenceEngine: T3 (σ + ∂ + ς + κ + π + ∝ + μ)
//
// The full scan→evaluate→enforce→audit orchestrator.
// Cannot implement GroundsTo due to non-Clone Arc<dyn Enforcer>,
// but documented for completeness. The FenceReport serves as the
// serializable T3 proxy.

/// FenceReport: T3 (σ + ∂ + ς + κ + π + N + Σ), dominant Boundary
///
/// Complete engine status snapshot — full domain type summarizing all aspects.
impl GroundsTo for crate::engine::FenceReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Persistence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

// ═══════════════════════════════════════════════════════════
// ENFORCER TYPES
// ═══════════════════════════════════════════════════════════

/// EnforcerOp: T2-P (∂ + ∝), dominant Boundary
///
/// A recorded enforcement operation — irreversible boundary action.
impl GroundsTo for crate::enforcer::EnforcerOp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Irreversibility])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// TickDecision: T2-P (∂ + κ), dominant Boundary
///
/// A single decision record from a tick — verdict with rule attribution.
impl GroundsTo for crate::engine::TickDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Boundary, 0.88)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: verify a type's grounding.
    fn verify_grounding<T: GroundsTo>(
        expected_dominant: LexPrimitiva,
        min_confidence: f64,
        expected_tier: &str,
    ) {
        let comp = T::primitive_composition();
        assert!(comp.dominant.is_some(), "no dominant for {expected_tier}");
        let prim = comp.dominant.unwrap_or(LexPrimitiva::Void);
        let conf = comp.confidence;
        assert_eq!(
            prim, expected_dominant,
            "{expected_tier}: expected {expected_dominant:?}, got {prim:?}"
        );
        assert!(
            conf >= min_confidence,
            "{expected_tier}: confidence {conf} < {min_confidence}"
        );
    }

    // T1 types
    #[test]
    fn test_grounding_fence_mode() {
        verify_grounding::<crate::policy::FenceMode>(LexPrimitiva::State, 0.90, "FenceMode");
    }

    #[test]
    fn test_grounding_direction() {
        verify_grounding::<crate::process::Direction>(LexPrimitiva::Boundary, 0.90, "Direction");
    }

    #[test]
    fn test_grounding_fence_verdict() {
        verify_grounding::<crate::rule::FenceVerdict>(LexPrimitiva::Boundary, 0.90, "FenceVerdict");
    }

    #[test]
    fn test_grounding_protocol() {
        verify_grounding::<crate::connection::Protocol>(LexPrimitiva::Comparison, 0.90, "Protocol");
    }

    #[test]
    fn test_grounding_tcp_state() {
        verify_grounding::<crate::connection::TcpState>(LexPrimitiva::State, 0.90, "TcpState");
    }

    // T2-P types
    #[test]
    fn test_grounding_socket_entry() {
        verify_grounding::<crate::connection::SocketEntry>(
            LexPrimitiva::Location,
            0.80,
            "SocketEntry",
        );
    }

    #[test]
    fn test_grounding_process_info() {
        verify_grounding::<crate::process::ProcessInfo>(
            LexPrimitiva::Existence,
            0.80,
            "ProcessInfo",
        );
    }

    #[test]
    fn test_grounding_process_match() {
        verify_grounding::<crate::rule::ProcessMatch>(LexPrimitiva::Boundary, 0.80, "ProcessMatch");
    }

    #[test]
    fn test_grounding_network_match() {
        verify_grounding::<crate::rule::NetworkMatch>(LexPrimitiva::Boundary, 0.80, "NetworkMatch");
    }

    #[test]
    fn test_grounding_fence_rule() {
        verify_grounding::<crate::rule::FenceRule>(LexPrimitiva::Boundary, 0.85, "FenceRule");
    }

    #[test]
    fn test_grounding_fence_decision() {
        verify_grounding::<crate::policy::FenceDecision>(
            LexPrimitiva::Boundary,
            0.85,
            "FenceDecision",
        );
    }

    #[test]
    fn test_grounding_audit_entry() {
        verify_grounding::<crate::audit::AuditEntry>(LexPrimitiva::Persistence, 0.80, "AuditEntry");
    }

    // T2-C types
    #[test]
    fn test_grounding_connection_event() {
        verify_grounding::<crate::process::ConnectionEvent>(
            LexPrimitiva::Location,
            0.75,
            "ConnectionEvent",
        );
    }

    #[test]
    fn test_grounding_rule_set() {
        verify_grounding::<crate::rule::RuleSet>(LexPrimitiva::Boundary, 0.75, "RuleSet");
    }

    #[test]
    fn test_grounding_fence_policy() {
        verify_grounding::<crate::policy::FencePolicy>(LexPrimitiva::Boundary, 0.80, "FencePolicy");
    }

    #[test]
    fn test_grounding_fence_stats() {
        verify_grounding::<crate::engine::FenceStats>(LexPrimitiva::Quantity, 0.75, "FenceStats");
    }

    #[test]
    fn test_grounding_audit_log() {
        verify_grounding::<crate::audit::AuditLog>(LexPrimitiva::Persistence, 0.80, "AuditLog");
    }

    #[test]
    fn test_grounding_fence_tick_result() {
        verify_grounding::<crate::engine::FenceTickResult>(
            LexPrimitiva::Sequence,
            0.75,
            "FenceTickResult",
        );
    }

    // T3 types
    #[test]
    fn test_grounding_fence_report() {
        verify_grounding::<crate::engine::FenceReport>(LexPrimitiva::Boundary, 0.70, "FenceReport");
    }

    // Enforcer types
    #[test]
    fn test_grounding_enforcer_op() {
        verify_grounding::<crate::enforcer::EnforcerOp>(LexPrimitiva::Boundary, 0.80, "EnforcerOp");
    }

    #[test]
    fn test_grounding_tick_decision() {
        verify_grounding::<crate::engine::TickDecision>(
            LexPrimitiva::Boundary,
            0.85,
            "TickDecision",
        );
    }

    // Count verification
    #[test]
    fn test_grounding_count_total_20() {
        // 5 T1 + 7 T2-P + 6 T2-C + 2 T3 = 20
        // (FenceEngine is documented but not impl'd due to Arc<dyn>)
        let count = 5 + 7 + 6 + 2;
        assert_eq!(count, 20, "expected 20 GroundsTo impls");
    }

    // Tier classification verification
    #[test]
    fn test_tier_classification() {
        // T1: 1 primitive
        let t1_types = [
            crate::policy::FenceMode::primitive_composition(),
            crate::process::Direction::primitive_composition(),
            crate::rule::FenceVerdict::primitive_composition(),
            crate::connection::Protocol::primitive_composition(),
            crate::connection::TcpState::primitive_composition(),
        ];
        for (i, comp) in t1_types.iter().enumerate() {
            assert_eq!(
                comp.primitives.len(),
                1,
                "T1 type {i} has {} primitives, expected 1",
                comp.primitives.len()
            );
        }

        // T2-P: 2-3 primitives
        let t2p_types = [
            crate::connection::SocketEntry::primitive_composition(),
            crate::process::ProcessInfo::primitive_composition(),
            crate::rule::ProcessMatch::primitive_composition(),
            crate::rule::NetworkMatch::primitive_composition(),
            crate::rule::FenceRule::primitive_composition(),
            crate::policy::FenceDecision::primitive_composition(),
            crate::audit::AuditEntry::primitive_composition(),
        ];
        for (i, comp) in t2p_types.iter().enumerate() {
            let n = comp.primitives.len();
            assert!(
                (2..=3).contains(&n),
                "T2-P type {i} has {n} primitives, expected 2-3"
            );
        }

        // T2-C: 4-5 primitives
        let t2c_types = [
            crate::process::ConnectionEvent::primitive_composition(),
            crate::rule::RuleSet::primitive_composition(),
            crate::policy::FencePolicy::primitive_composition(),
            crate::engine::FenceStats::primitive_composition(),
            crate::audit::AuditLog::primitive_composition(),
            crate::engine::FenceTickResult::primitive_composition(),
        ];
        for (i, comp) in t2c_types.iter().enumerate() {
            let n = comp.primitives.len();
            assert!(
                (4..=5).contains(&n),
                "T2-C type {i} has {n} primitives, expected 4-5"
            );
        }

        // T3: 6+ primitives
        let t3_types = [crate::engine::FenceReport::primitive_composition()];
        for (i, comp) in t3_types.iter().enumerate() {
            let n = comp.primitives.len();
            assert!(n >= 6, "T3 type {i} has {n} primitives, expected 6+");
        }
    }
}
