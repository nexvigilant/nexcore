//! Cascade order from dependency topology.
//!
//! Precomputes the failure cascade ordering for the 4-layer NexCore
//! architecture. Layers with more incoming dependencies fail first when
//! foundational components go down, so they carry the highest fragility score.
//!
//! ## T1 Grounding
//!
//! - `ArchitectureLayer` → Σ (Sum) — discrete layer taxonomy
//! - `fragility` → κ (Comparison) — ordered by failure risk
//! - `compute_cascade_order` → σ (Sequence) — ordered failure chain

use serde::{Deserialize, Serialize};

// =============================================================================
// ArchitectureLayer
// =============================================================================

/// One of the four architectural layers in the NexCore dependency hierarchy.
///
/// Dependency direction: `Service → Orchestration → Domain → Foundation`.
/// Failure propagation is the reverse: a `Foundation` failure cascades up
/// through all layers; a `Service` failure is contained.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchitectureLayer {
    /// Service layer — binary targets, 5–76 internal dependencies (most fragile).
    Service,
    /// Orchestration layer — workflow coordination, 3–5 internal dependencies.
    Orchestration,
    /// Domain layer — business logic, 2–25 internal dependencies.
    Domain,
    /// Foundation layer — pure primitives, 0–3 internal dependencies (most resilient).
    Foundation,
}

impl std::fmt::Display for ArchitectureLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service => write!(f, "Service"),
            Self::Orchestration => write!(f, "Orchestration"),
            Self::Domain => write!(f, "Domain"),
            Self::Foundation => write!(f, "Foundation"),
        }
    }
}

// =============================================================================
// CascadeOrder
// =============================================================================

/// Precomputed failure cascade ordering for the NexCore 4-layer architecture.
///
/// `layers` is ordered from highest failure risk (most deps → fails first on
/// upstream outage) to lowest (most resilient, can survive in isolation).
///
/// `fragility` maps each layer to a normalized score in `[0.0, 1.0]`:
/// - `1.0` = maximally fragile (Service: 76 deps)
/// - `0.0` = perfectly resilient (hypothetical zero-dep Foundation)
///
/// ```
/// use nexcore_homeostasis_primitives::cascade_order::{ArchitectureLayer, compute_cascade_order};
///
/// let order = compute_cascade_order();
/// assert_eq!(order.layers[0], ArchitectureLayer::Service);
/// assert_eq!(order.layers[3], ArchitectureLayer::Foundation);
///
/// let (_, service_fragility) = &order.fragility[0];
/// assert!((*service_fragility - 1.0).abs() < f64::EPSILON);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeOrder {
    /// Layers ordered from most fragile (index 0) to most resilient (last).
    pub layers: Vec<ArchitectureLayer>,
    /// Per-layer fragility scores: `(layer, score)` in `[0.0, 1.0]`.
    ///
    /// `fragility = incoming_deps / max_incoming_deps` (normalized).
    pub fragility: Vec<(ArchitectureLayer, f64)>,
}

// =============================================================================
// compute_cascade_order
// =============================================================================

/// Compute the failure cascade order from the NexCore dependency topology.
///
/// Fragility is normalized by the maximum observed incoming-dependency count
/// (`nexcore-mcp` Service layer: 76 deps).
///
/// | Layer | Deps | Fragility |
/// |-------|------|-----------|
/// | Service | 76 | 1.000 |
/// | Domain | 25 | 0.329 |
/// | Orchestration | 5 | 0.066 |
/// | Foundation | 3 | 0.039 |
pub fn compute_cascade_order() -> CascadeOrder {
    // Anchor: nexcore-mcp has 76 internal deps (the global maximum).
    const MAX_DEPS: f64 = 76.0;

    let fragility = vec![
        (ArchitectureLayer::Service, 76.0 / MAX_DEPS),       // 1.000
        (ArchitectureLayer::Domain, 25.0 / MAX_DEPS),        // 0.329
        (ArchitectureLayer::Orchestration, 5.0 / MAX_DEPS),  // 0.066
        (ArchitectureLayer::Foundation, 3.0 / MAX_DEPS),     // 0.039
    ];

    let layers = fragility.iter().map(|(l, _)| l.clone()).collect();

    CascadeOrder { layers, fragility }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_order_is_service_first() {
        let order = compute_cascade_order();
        assert_eq!(order.layers[0], ArchitectureLayer::Service);
        assert_eq!(order.layers[3], ArchitectureLayer::Foundation);
    }

    #[test]
    fn service_fragility_is_one() {
        let order = compute_cascade_order();
        let svc = order
            .fragility
            .iter()
            .find(|(l, _)| l == &ArchitectureLayer::Service)
            .map(|(_, f)| *f);
        assert!(svc.is_some());
        assert!((svc.unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn foundation_fragility_is_lowest() {
        let order = compute_cascade_order();
        let scores: Vec<f64> = order.fragility.iter().map(|(_, f)| *f).collect();
        let min_score = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let (last_layer, last_score) = &order.fragility[order.fragility.len() - 1];
        assert_eq!(last_layer, &ArchitectureLayer::Foundation);
        assert!((last_score - min_score).abs() < f64::EPSILON);
    }

    #[test]
    fn fragility_scores_monotonically_decreasing() {
        let order = compute_cascade_order();
        let scores: Vec<f64> = order.fragility.iter().map(|(_, f)| *f).collect();
        for pair in scores.windows(2) {
            assert!(pair[0] >= pair[1], "scores must be non-increasing: {} >= {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn all_four_layers_present() {
        let order = compute_cascade_order();
        assert_eq!(order.layers.len(), 4);
        assert!(order.layers.contains(&ArchitectureLayer::Service));
        assert!(order.layers.contains(&ArchitectureLayer::Orchestration));
        assert!(order.layers.contains(&ArchitectureLayer::Domain));
        assert!(order.layers.contains(&ArchitectureLayer::Foundation));
    }

    #[test]
    fn display_impl() {
        assert_eq!(ArchitectureLayer::Service.to_string(), "Service");
        assert_eq!(ArchitectureLayer::Foundation.to_string(), "Foundation");
    }
}
