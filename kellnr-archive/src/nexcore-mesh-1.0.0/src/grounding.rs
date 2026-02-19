//! GroundsTo implementations for all mesh types.
//!
//! 28 implementations grounding mesh types to T1 Lex Primitiva.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::chemotaxis::{ChemotacticRouteSelector, ChemotacticRouter};
use crate::discovery::{DiscoveryAction, DiscoveryLoop, DiscoveryMessage};
use crate::error::MeshError;
use crate::gossip::{GossipLoop, GossipMessage};
use crate::neighbor::{Neighbor, NeighborRegistry};
use crate::node::{MeshMessage, Node, NodeState};
use crate::persistence::{MeshSnapshot, SnapshotStore};
use crate::resilience::{ResilienceAction, ResilienceLoop, ResilienceState};
use crate::routing::{Route, RoutingTable};
use crate::runtime::{MeshEvent, MeshHandle, MeshRuntime};
use crate::security::{PeerIdentity, SecurityPolicy, TlsTier};
use crate::topology::{Path, RouteQuality};

// ============================================================================
// T1 — Pure primitives
// ============================================================================

/// NodeState: pure ς (State)
impl GroundsTo for NodeState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

// ============================================================================
// T2-P — 2-3 primitives
// ============================================================================

/// Path: σ (Sequence) + ∂ (Boundary) — hop sequence with TTL boundary
impl GroundsTo for Path {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// MeshError: ∂ (Boundary) + ∅ (Void) — error boundaries delimit valid/invalid
impl GroundsTo for MeshError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Void])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// DiscoveryMessage: Σ (Sum) + ∃ (Existence) — announce/response sum type
impl GroundsTo for DiscoveryMessage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// DiscoveryAction: Σ (Sum) + → (Causality) — action sum type with causal consequence
impl GroundsTo for DiscoveryAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

/// MeshEvent: Σ (Sum) + → (Causality) — observable event sum type
impl GroundsTo for MeshEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// ResilienceAction: Σ (Sum) + → (Causality) — corrective action sum type
impl GroundsTo for ResilienceAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

/// TlsTier: pure ς (State) — each tier is a distinct security state
impl GroundsTo for TlsTier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

/// PeerIdentity: λ (Location) + ∂ (Boundary) — who and where with boundary
impl GroundsTo for PeerIdentity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

// ============================================================================
// T2-C — 4-5 primitives
// ============================================================================

/// RouteQuality: κ (Comparison) + N (Quantity) + ν (Frequency) + ∂ (Boundary)
impl GroundsTo for RouteQuality {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.70)
    }
}

/// MeshMessage: σ (Sequence) + λ (Location) + μ (Mapping) + → (Causality)
impl GroundsTo for MeshMessage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Location,
            LexPrimitiva::Mapping,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.65)
    }
}

/// Neighbor: ς (State) + ν (Frequency) + ∃ (Existence) + κ (Comparison)
impl GroundsTo for Neighbor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Frequency,
            LexPrimitiva::Existence,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::State, 0.60)
    }
}

/// Route: λ (Location) + σ (Sequence) + κ (Comparison) + μ (Mapping)
impl GroundsTo for Route {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Location, 0.60)
    }
}

/// ResilienceState: κ (Comparison) + N (Quantity) + ν (Frequency) + ∃ (Existence)
impl GroundsTo for ResilienceState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.65)
    }
}

/// SecurityPolicy: ∂ (Boundary) + κ (Comparison) + ς (State) + → (Causality)
impl GroundsTo for SecurityPolicy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.65)
    }
}

/// ChemotacticRouteSelector: κ (Comparison) + N (Quantity) + Σ (Sum) + → (Causality)
impl GroundsTo for ChemotacticRouteSelector {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Sum,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.60)
    }
}

/// MeshSnapshot: π (Persistence) + ς (State) + μ (Mapping) + σ (Sequence)
impl GroundsTo for MeshSnapshot {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.65)
    }
}

/// MeshHandle: μ (Mapping) + → (Causality) + ∂ (Boundary) + ς (State)
impl GroundsTo for MeshHandle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.65)
    }
}

/// GossipMessage: μ (Mapping) + σ (Sequence) + ∂ (Boundary) + ν (Frequency)
impl GroundsTo for GossipMessage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.60)
    }
}

// ============================================================================
// T3 — 6+ primitives (full domain types)
// ============================================================================

/// NeighborRegistry: μ (Mapping) + ∃ (Existence) + κ (Comparison) + ∂ (Boundary) + ς (State) + N (Quantity)
impl GroundsTo for NeighborRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.55)
    }
}

/// RoutingTable: μ (Mapping) + λ (Location) + κ (Comparison) + σ (Sequence) + ∂ (Boundary) + N (Quantity)
impl GroundsTo for RoutingTable {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Location,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.50)
    }
}

/// DiscoveryLoop: ν (Frequency) + ∃ (Existence) + σ (Sequence) + μ (Mapping) + ∂ (Boundary) + → (Causality)
impl GroundsTo for DiscoveryLoop {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Existence,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.50)
    }
}

/// GossipLoop: ν (Frequency) + μ (Mapping) + σ (Sequence) + ∂ (Boundary) + → (Causality) + κ (Comparison)
impl GroundsTo for GossipLoop {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.50)
    }
}

/// ResilienceLoop: ρ (Recursion) + κ (Comparison) + ∂ (Boundary) + ν (Frequency) + ς (State) + → (Causality)
impl GroundsTo for ResilienceLoop {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
            LexPrimitiva::State,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.50)
    }
}

/// MeshRuntime: ρ (Recursion) + σ (Sequence) + μ (Mapping) + ν (Frequency) + ∂ (Boundary) + → (Causality) + ς (State)
impl GroundsTo for MeshRuntime {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.50)
    }
}

/// Node: ς (State) + μ (Mapping) + σ (Sequence) + ∂ (Boundary) + → (Causality) + ρ (Recursion) + λ (Location)
impl GroundsTo for Node {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Recursion,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::State, 0.45)
    }
}

/// SnapshotStore: π (Persistence) + μ (Mapping) + ∂ (Boundary) + → (Causality) + λ (Location) + ς (State)
impl GroundsTo for SnapshotStore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.55)
    }
}

/// ChemotacticRouter: ρ (Recursion) + → (Causality) + λ (Location) + κ (Comparison) + μ (Mapping) + Σ (Sum)
impl GroundsTo for ChemotacticRouter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // --- T1 ---

    #[test]
    fn node_state_is_t1() {
        assert_eq!(NodeState::tier(), Tier::T1Universal);
        assert_eq!(NodeState::dominant_primitive(), Some(LexPrimitiva::State));
        assert!(NodeState::is_pure_primitive());
    }

    #[test]
    fn tls_tier_is_t1() {
        assert_eq!(TlsTier::tier(), Tier::T1Universal);
        assert_eq!(TlsTier::dominant_primitive(), Some(LexPrimitiva::State));
    }

    // --- T2-P ---

    #[test]
    fn path_is_t2p() {
        assert_eq!(Path::tier(), Tier::T2Primitive);
        assert_eq!(Path::dominant_primitive(), Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn mesh_error_is_t2p() {
        assert_eq!(MeshError::tier(), Tier::T2Primitive);
        assert_eq!(
            MeshError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn discovery_message_is_t2p() {
        assert_eq!(DiscoveryMessage::tier(), Tier::T2Primitive);
        assert_eq!(
            DiscoveryMessage::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn discovery_action_is_t2p() {
        assert_eq!(DiscoveryAction::tier(), Tier::T2Primitive);
        assert_eq!(
            DiscoveryAction::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn mesh_event_is_t2p() {
        assert_eq!(MeshEvent::tier(), Tier::T2Primitive);
        assert_eq!(MeshEvent::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn resilience_action_is_t2p() {
        assert_eq!(ResilienceAction::tier(), Tier::T2Primitive);
        assert_eq!(
            ResilienceAction::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn peer_identity_is_t2p() {
        assert_eq!(PeerIdentity::tier(), Tier::T2Primitive);
        assert_eq!(
            PeerIdentity::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
    }

    // --- T2-C ---

    #[test]
    fn route_quality_is_t2c() {
        assert_eq!(RouteQuality::tier(), Tier::T2Composite);
        assert_eq!(
            RouteQuality::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn mesh_message_is_t2c() {
        assert_eq!(MeshMessage::tier(), Tier::T2Composite);
        assert_eq!(
            MeshMessage::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn neighbor_is_t2c() {
        assert_eq!(Neighbor::tier(), Tier::T2Composite);
        assert_eq!(Neighbor::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn route_is_t2c() {
        assert_eq!(Route::tier(), Tier::T2Composite);
        assert_eq!(Route::dominant_primitive(), Some(LexPrimitiva::Location));
    }

    #[test]
    fn gossip_message_is_t2c() {
        assert_eq!(GossipMessage::tier(), Tier::T2Composite);
        assert_eq!(
            GossipMessage::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn resilience_state_is_t2c() {
        assert_eq!(ResilienceState::tier(), Tier::T2Composite);
        assert_eq!(
            ResilienceState::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn mesh_handle_is_t2c() {
        assert_eq!(MeshHandle::tier(), Tier::T2Composite);
        assert_eq!(
            MeshHandle::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn security_policy_is_t2c() {
        assert_eq!(SecurityPolicy::tier(), Tier::T2Composite);
        assert_eq!(
            SecurityPolicy::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn chemotactic_route_selector_is_t2c() {
        assert_eq!(ChemotacticRouteSelector::tier(), Tier::T2Composite);
        assert_eq!(
            ChemotacticRouteSelector::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn mesh_snapshot_is_t2c() {
        assert_eq!(MeshSnapshot::tier(), Tier::T2Composite);
        assert_eq!(
            MeshSnapshot::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    // --- T3 ---

    #[test]
    fn neighbor_registry_is_t3() {
        assert_eq!(NeighborRegistry::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            NeighborRegistry::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn routing_table_is_t3() {
        assert_eq!(RoutingTable::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            RoutingTable::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn discovery_loop_is_t3() {
        assert_eq!(DiscoveryLoop::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            DiscoveryLoop::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn gossip_loop_is_t3() {
        assert_eq!(GossipLoop::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            GossipLoop::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn resilience_loop_is_t3() {
        assert_eq!(ResilienceLoop::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            ResilienceLoop::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
    }

    #[test]
    fn mesh_runtime_is_t3() {
        assert_eq!(MeshRuntime::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            MeshRuntime::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
    }

    #[test]
    fn node_is_t3() {
        assert_eq!(Node::tier(), Tier::T3DomainSpecific);
        assert_eq!(Node::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn snapshot_store_is_t3() {
        assert_eq!(SnapshotStore::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            SnapshotStore::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn chemotactic_router_is_t3() {
        assert_eq!(ChemotacticRouter::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            ChemotacticRouter::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
    }

    // --- Coverage ---

    #[test]
    fn all_28_groundings_have_nonzero_confidence() {
        let types: Vec<PrimitiveComposition> = vec![
            // T1 (2)
            NodeState::primitive_composition(),
            TlsTier::primitive_composition(),
            // T2-P (8)
            Path::primitive_composition(),
            MeshError::primitive_composition(),
            DiscoveryMessage::primitive_composition(),
            DiscoveryAction::primitive_composition(),
            MeshEvent::primitive_composition(),
            ResilienceAction::primitive_composition(),
            PeerIdentity::primitive_composition(),
            // T2-C (10)
            RouteQuality::primitive_composition(),
            MeshMessage::primitive_composition(),
            Neighbor::primitive_composition(),
            Route::primitive_composition(),
            GossipMessage::primitive_composition(),
            ResilienceState::primitive_composition(),
            MeshHandle::primitive_composition(),
            SecurityPolicy::primitive_composition(),
            ChemotacticRouteSelector::primitive_composition(),
            MeshSnapshot::primitive_composition(),
            // T3 (9)
            NeighborRegistry::primitive_composition(),
            RoutingTable::primitive_composition(),
            DiscoveryLoop::primitive_composition(),
            GossipLoop::primitive_composition(),
            ResilienceLoop::primitive_composition(),
            MeshRuntime::primitive_composition(),
            Node::primitive_composition(),
            SnapshotStore::primitive_composition(),
            ChemotacticRouter::primitive_composition(),
        ];

        assert_eq!(types.len(), 28);

        for comp in &types {
            assert!(comp.dominant.is_some());
            assert!(comp.confidence > 0.0);
            assert!(!comp.primitives.is_empty());
        }
    }

    #[test]
    fn mesh_covers_14_of_16_primitives() {
        let mut all_primitives = std::collections::HashSet::new();
        let types: Vec<PrimitiveComposition> = vec![
            // Original 21
            NodeState::primitive_composition(),
            Path::primitive_composition(),
            MeshError::primitive_composition(),
            RouteQuality::primitive_composition(),
            MeshMessage::primitive_composition(),
            Neighbor::primitive_composition(),
            Route::primitive_composition(),
            NeighborRegistry::primitive_composition(),
            RoutingTable::primitive_composition(),
            DiscoveryLoop::primitive_composition(),
            GossipLoop::primitive_composition(),
            ResilienceLoop::primitive_composition(),
            Node::primitive_composition(),
            DiscoveryMessage::primitive_composition(),
            DiscoveryAction::primitive_composition(),
            GossipMessage::primitive_composition(),
            MeshEvent::primitive_composition(),
            ResilienceAction::primitive_composition(),
            ResilienceState::primitive_composition(),
            MeshHandle::primitive_composition(),
            MeshRuntime::primitive_composition(),
            // New 7
            TlsTier::primitive_composition(),
            PeerIdentity::primitive_composition(),
            SecurityPolicy::primitive_composition(),
            ChemotacticRouteSelector::primitive_composition(),
            MeshSnapshot::primitive_composition(),
            SnapshotStore::primitive_composition(),
            ChemotacticRouter::primitive_composition(),
        ];

        for comp in &types {
            for p in &comp.primitives {
                all_primitives.insert(*p);
            }
        }

        // Covered: λ μ σ ∂ ρ ν ∃ ς κ → N Σ ∅ π = 14/16
        // Missing: ∝ (Irreversibility), × (Product)
        assert!(
            all_primitives.len() >= 14,
            "Expected >= 14 primitives, got {}",
            all_primitives.len()
        );
    }

    #[test]
    fn tier_distribution() {
        // 2 T1, 7 T2-P, 10 T2-C, 9 T3 = 28 total
        let t1_count = [NodeState::tier(), TlsTier::tier()]
            .iter()
            .filter(|t| **t == Tier::T1Universal)
            .count();
        assert_eq!(t1_count, 2);

        let t2p_count = [
            Path::tier(),
            MeshError::tier(),
            DiscoveryMessage::tier(),
            DiscoveryAction::tier(),
            MeshEvent::tier(),
            ResilienceAction::tier(),
            PeerIdentity::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();
        assert_eq!(t2p_count, 7);

        let t2c_count = [
            RouteQuality::tier(),
            MeshMessage::tier(),
            Neighbor::tier(),
            Route::tier(),
            GossipMessage::tier(),
            ResilienceState::tier(),
            MeshHandle::tier(),
            SecurityPolicy::tier(),
            ChemotacticRouteSelector::tier(),
            MeshSnapshot::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Composite)
        .count();
        assert_eq!(t2c_count, 10);

        let t3_count = [
            NeighborRegistry::tier(),
            RoutingTable::tier(),
            DiscoveryLoop::tier(),
            GossipLoop::tier(),
            ResilienceLoop::tier(),
            MeshRuntime::tier(),
            Node::tier(),
            SnapshotStore::tier(),
            ChemotacticRouter::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T3DomainSpecific)
        .count();
        assert_eq!(t3_count, 9);
    }
}
