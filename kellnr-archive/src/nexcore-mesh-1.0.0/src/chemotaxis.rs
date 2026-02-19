//! Chemotaxis routing — gradient-weighted path selection for mesh nodes.
//!
//! ## Primitive Foundation
//! - `ChemotacticRouter`: T3, ρ (Recursion) + → (Causality) + λ (Location) + κ (Comparison) + μ (Mapping) + Σ (Sum)
//! - `ChemotacticRouteSelector`: T2-C, κ (Comparison) + N (Quantity) + Σ (Sum) + → (Causality)
//!
//! ## Design
//!
//! Integrates `nexcore-cytokine` chemotaxis types with mesh routing.
//! Each node maintains a `GradientField` updated by neighbor health metrics.
//! Route selection is weighted by gradient pull:
//!
//! ```text
//! effective_quality = route_quality.score() * (1.0 + gradient_pull)
//! ```
//!
//! This allows mesh routing to prefer paths toward healthy, active neighbors
//! while avoiding degraded links (negative tropism).

use crate::neighbor::NeighborRegistry;
use crate::routing::{Route, RoutingTable};
use crate::topology::RouteQuality;
use nexcore_cytokine::{
    ChemotacticAgent, ChemotacticDecision, CytokineFamily, Gradient, GradientField, Tropism,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// MeshGradientConfig — Configuration for gradient-based routing
// ============================================================================

/// Configuration for chemotactic routing behavior.
///
/// Tier: T2-P | Dominant: ∂ (Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshGradientConfig {
    /// Minimum effective gradient strength to influence routing.
    /// Below this, the gradient is ignored and standard quality-based routing is used.
    pub sensitivity_threshold: f64,
    /// Maximum gradient pull multiplier. Caps the influence of gradients
    /// to prevent extreme bias.
    pub max_gradient_influence: f64,
    /// Cytokine family to use for health gradients.
    pub health_family: CytokineFamily,
}

impl Default for MeshGradientConfig {
    fn default() -> Self {
        Self {
            sensitivity_threshold: 0.01,
            max_gradient_influence: 2.0,
            health_family: CytokineFamily::Il6, // IL-6: acute phase response
        }
    }
}

// ============================================================================
// ChemotacticRouteSelector — Gradient-weighted route scoring
// ============================================================================

/// Selects routes by combining quality scores with gradient pull.
///
/// Tier: T2-C | Dominant: κ (Comparison)
///
/// Standard route quality scoring is augmented by gradient information:
/// routes toward healthier neighbors get a boost, routes toward
/// degraded neighbors get penalized.
#[derive(Debug, Clone)]
pub struct ChemotacticRouteSelector {
    /// Gradient field from sensed neighbor health.
    field: GradientField,
    /// Configuration.
    config: MeshGradientConfig,
}

impl ChemotacticRouteSelector {
    /// Create a new selector with the given gradient field and config.
    pub fn new(field: GradientField, config: MeshGradientConfig) -> Self {
        Self { field, config }
    }

    /// Compute effective quality for a route, weighted by gradient pull.
    ///
    /// `effective_quality = quality.score() * (1.0 + clamped_gradient_pull)`
    ///
    /// The gradient pull for the route's next_hop is looked up in the field's
    /// per-source pulls. If no gradient exists for that hop, standard scoring is used.
    pub fn effective_quality(&self, route: &Route) -> f64 {
        let base = route.quality.score();
        let pulls = self.field.source_pulls();

        let gradient_pull = pulls.get(&route.next_hop).copied().unwrap_or(0.0);

        // Ignore weak gradients
        if gradient_pull.abs() < self.config.sensitivity_threshold {
            return base;
        }

        // Clamp the influence
        let clamped = gradient_pull.clamp(
            -self.config.max_gradient_influence,
            self.config.max_gradient_influence,
        );

        base * (1.0 + clamped)
    }

    /// Select the best route from a set, considering gradient weights.
    pub fn select_best<'a>(&self, routes: &'a [Route]) -> Option<&'a Route> {
        if routes.is_empty() {
            return None;
        }

        routes.iter().max_by(|a, b| {
            self.effective_quality(a)
                .partial_cmp(&self.effective_quality(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get the underlying gradient field.
    pub fn field(&self) -> &GradientField {
        &self.field
    }
}

// ============================================================================
// ChemotacticRouter — Full gradient-aware routing for mesh nodes
// ============================================================================

/// A mesh node routing component that uses cytokine gradients for path selection.
///
/// Tier: T3 | Dominant: ρ (Recursion)
///
/// Wraps `NeighborRegistry` and `RoutingTable` with gradient-based selection.
/// Implements `ChemotacticAgent` from nexcore-cytokine for standardized
/// gradient sensing and navigation.
pub struct ChemotacticRouter {
    /// Node ID of the node this router belongs to.
    node_id: String,
    /// Configuration for gradient behavior.
    config: MeshGradientConfig,
    /// Reference to the node's neighbor registry (for health sensing).
    neighbors: NeighborRegistry,
    /// Reference to the node's routing table.
    routing: RoutingTable,
}

impl ChemotacticRouter {
    /// Create a new chemotactic router.
    pub fn new(
        node_id: impl Into<String>,
        config: MeshGradientConfig,
        neighbors: NeighborRegistry,
        routing: RoutingTable,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            config,
            neighbors,
            routing,
        }
    }

    /// Build a gradient field from current neighbor health metrics.
    ///
    /// Each neighbor contributes a gradient sample:
    /// - **concentration** = `neighbor.quality.reliability` (health proxy)
    /// - **distance** = `neighbor.quality.hop_count` (topological distance)
    /// - **tropism** = Positive if reachable, Negative if circuit breaker open
    pub fn build_gradient_field(&self) -> GradientField {
        let mut field = GradientField::new();

        for neighbor_id in self.neighbors.all_ids() {
            if let Some(neighbor) = self.neighbors.get(&neighbor_id) {
                let concentration = neighbor.quality.reliability;
                let distance = neighbor.quality.hop_count as f64;

                let tropism = if neighbor.is_reachable() {
                    Tropism::Positive
                } else {
                    Tropism::Negative
                };

                let gradient = Gradient::new(
                    &neighbor_id,
                    self.config.health_family,
                    concentration,
                    distance,
                )
                .with_tropism(tropism);

                field.add(gradient);
            }
        }

        field
    }

    /// Select the best route to a destination using gradient-weighted scoring.
    pub fn best_route_to(&self, destination: &str) -> Option<Route> {
        let routes = self.routing.routes_to(destination);
        if routes.is_empty() {
            return None;
        }

        let field = self.build_gradient_field();
        let selector = ChemotacticRouteSelector::new(field, self.config.clone());

        selector.select_best(&routes).cloned()
    }

    /// Get the node ID.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Get the configuration.
    pub fn config(&self) -> &MeshGradientConfig {
        &self.config
    }
}

impl ChemotacticAgent for ChemotacticRouter {
    fn sense_field(&self) -> GradientField {
        self.build_gradient_field()
    }

    fn sensitivity_threshold(&self) -> f64 {
        self.config.sensitivity_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neighbor::Neighbor;
    use crate::routing::Route;
    use crate::topology::Path;

    fn make_quality(latency: f64, reliability: f64, hops: u8) -> RouteQuality {
        RouteQuality::new(latency, reliability, hops)
    }

    fn make_route(dest: &str, next: &str, latency: f64, reliability: f64, hops: u8) -> Route {
        let mut path = Path::new("self", 16);
        let _ = path.add_hop(dest);
        Route::new(dest, next, path, make_quality(latency, reliability, hops))
    }

    fn make_neighbor(id: &str, reliability: f64) -> Neighbor {
        Neighbor::new(id, make_quality(50.0, reliability, 1))
    }

    // ---------- MeshGradientConfig tests ----------

    #[test]
    fn gradient_config_defaults() {
        let cfg = MeshGradientConfig::default();
        assert!((cfg.sensitivity_threshold - 0.01).abs() < f64::EPSILON);
        assert!((cfg.max_gradient_influence - 2.0).abs() < f64::EPSILON);
        assert_eq!(cfg.health_family, CytokineFamily::Il6);
    }

    // ---------- ChemotacticRouteSelector tests ----------

    #[test]
    fn selector_no_gradient_uses_base_quality() {
        let field = GradientField::new(); // empty
        let config = MeshGradientConfig::default();
        let selector = ChemotacticRouteSelector::new(field, config);

        let route = make_route("d", "hop-a", 50.0, 0.9, 1);
        let eff = selector.effective_quality(&route);
        let base = route.quality.score();
        assert!((eff - base).abs() < f64::EPSILON);
    }

    #[test]
    fn selector_positive_gradient_boosts_quality() {
        let mut field = GradientField::new();
        field.add(
            Gradient::new("hop-a", CytokineFamily::Il6, 0.8, 0.0).with_tropism(Tropism::Positive),
        );
        let config = MeshGradientConfig::default();
        let selector = ChemotacticRouteSelector::new(field, config);

        let route = make_route("d", "hop-a", 50.0, 0.9, 1);
        let base = route.quality.score();
        let eff = selector.effective_quality(&route);

        assert!(
            eff > base,
            "gradient-boosted quality ({eff}) should exceed base ({base})"
        );
    }

    #[test]
    fn selector_negative_gradient_penalizes_quality() {
        let mut field = GradientField::new();
        field.add(
            Gradient::new("hop-a", CytokineFamily::Il6, 0.8, 0.0).with_tropism(Tropism::Negative),
        );
        let config = MeshGradientConfig::default();
        let selector = ChemotacticRouteSelector::new(field, config);

        let route = make_route("d", "hop-a", 50.0, 0.9, 1);
        let base = route.quality.score();
        let eff = selector.effective_quality(&route);

        assert!(
            eff < base,
            "gradient-penalized quality ({eff}) should be less than base ({base})"
        );
    }

    #[test]
    fn selector_gradient_beats_higher_base_quality() {
        // Route A: lower base quality but strong positive gradient
        // Route B: higher base quality but no gradient
        let mut field = GradientField::new();
        field.add(
            Gradient::new("hop-a", CytokineFamily::Il6, 1.0, 0.0).with_tropism(Tropism::Positive),
        );
        let config = MeshGradientConfig::default();
        let selector = ChemotacticRouteSelector::new(field, config);

        let route_a = make_route("d", "hop-a", 100.0, 0.6, 2); // lower quality
        let route_b = make_route("d", "hop-b", 50.0, 0.7, 1); // higher quality

        let eff_a = selector.effective_quality(&route_a);
        let eff_b = selector.effective_quality(&route_b);

        assert!(
            eff_a > eff_b,
            "gradient-boosted route A ({eff_a}) should beat ungradient route B ({eff_b})"
        );
    }

    #[test]
    fn selector_weak_gradient_ignored() {
        let mut field = GradientField::new();
        // Very weak gradient (below default 0.01 threshold after distance decay)
        field.add(
            Gradient::new("hop-a", CytokineFamily::Il6, 0.001, 10.0)
                .with_tropism(Tropism::Positive),
        );
        let config = MeshGradientConfig::default();
        let selector = ChemotacticRouteSelector::new(field, config);

        let route = make_route("d", "hop-a", 50.0, 0.9, 1);
        let base = route.quality.score();
        let eff = selector.effective_quality(&route);

        assert!(
            (eff - base).abs() < f64::EPSILON,
            "weak gradient should not affect quality"
        );
    }

    #[test]
    fn selector_select_best_from_multiple() {
        let mut field = GradientField::new();
        field.add(
            Gradient::new("hop-b", CytokineFamily::Il6, 1.0, 0.0).with_tropism(Tropism::Positive),
        );
        let config = MeshGradientConfig::default();
        let selector = ChemotacticRouteSelector::new(field, config);

        let routes = vec![
            make_route("d", "hop-a", 50.0, 0.9, 1),
            make_route("d", "hop-b", 50.0, 0.5, 1), // lower quality but gradient-boosted
        ];

        let best = selector.select_best(&routes);
        assert!(best.is_some());
        assert_eq!(best.map(|r| r.next_hop.as_str()), Some("hop-b"));
    }

    #[test]
    fn selector_empty_routes_returns_none() {
        let field = GradientField::new();
        let selector = ChemotacticRouteSelector::new(field, MeshGradientConfig::default());
        assert!(selector.select_best(&[]).is_none());
    }

    // ---------- ChemotacticRouter tests ----------

    #[test]
    fn router_builds_gradient_from_neighbors() {
        let neighbors = NeighborRegistry::new(10);
        let _ = neighbors.add(make_neighbor("n1", 0.95));
        let _ = neighbors.add(make_neighbor("n2", 0.5));

        let routing = RoutingTable::with_defaults();
        let config = MeshGradientConfig::default();
        let router = ChemotacticRouter::new("self", config, neighbors, routing);

        let field = router.sense_field();
        assert_eq!(field.source_count(), 2);
        assert!(field.net_pull() > 0.0); // both are positive (reachable)
    }

    #[test]
    fn router_unreachable_neighbor_negative_tropism() {
        let neighbors = NeighborRegistry::new(10);
        let _ = neighbors.add(make_neighbor("down", 0.95));
        // Trip the circuit breaker
        for _ in 0..3 {
            neighbors.record_failure("down");
        }

        let routing = RoutingTable::with_defaults();
        let config = MeshGradientConfig::default();
        let router = ChemotacticRouter::new("self", config, neighbors, routing);

        let field = router.sense_field();
        assert_eq!(field.source_count(), 1);
        // Should be negative pull (repellent)
        assert!(field.net_pull() < 0.0);
    }

    #[test]
    fn router_best_route_with_gradient() {
        let neighbors = NeighborRegistry::new(10);
        let _ = neighbors.add(make_neighbor("good-hop", 0.99));
        let _ = neighbors.add(make_neighbor("bad-hop", 0.3));

        let routing = RoutingTable::with_defaults();
        // Both routes have similar base quality
        routing.upsert(make_route("dest", "good-hop", 50.0, 0.8, 1));
        routing.upsert(make_route("dest", "bad-hop", 50.0, 0.85, 1));

        let config = MeshGradientConfig::default();
        let router = ChemotacticRouter::new("self", config, neighbors, routing);

        let best = router.best_route_to("dest");
        assert!(best.is_some());
        // good-hop should win due to stronger gradient (0.99 reliability neighbor)
        assert_eq!(best.map(|r| r.next_hop), Some("good-hop".to_string()));
    }

    #[test]
    fn router_no_routes_returns_none() {
        let neighbors = NeighborRegistry::new(10);
        let routing = RoutingTable::with_defaults();
        let router =
            ChemotacticRouter::new("self", MeshGradientConfig::default(), neighbors, routing);
        assert!(router.best_route_to("nonexistent").is_none());
    }

    #[test]
    fn router_node_id_accessor() {
        let neighbors = NeighborRegistry::new(10);
        let routing = RoutingTable::with_defaults();
        let router =
            ChemotacticRouter::new("my-node", MeshGradientConfig::default(), neighbors, routing);
        assert_eq!(router.node_id(), "my-node");
    }

    #[test]
    fn gradient_influence_clamped() {
        let mut field = GradientField::new();
        // Extremely strong gradient
        field.add(
            Gradient::new("hop-x", CytokineFamily::Il6, 1.0, 0.0).with_tropism(Tropism::Positive),
        );
        // Also add more for the same source to stack pulls
        field.add(
            Gradient::new("hop-x", CytokineFamily::Il1, 1.0, 0.0).with_tropism(Tropism::Positive),
        );
        field.add(
            Gradient::new("hop-x", CytokineFamily::TnfAlpha, 1.0, 0.0)
                .with_tropism(Tropism::Positive),
        );

        let config = MeshGradientConfig {
            max_gradient_influence: 0.5,
            ..MeshGradientConfig::default()
        };
        let selector = ChemotacticRouteSelector::new(field, config);

        let route = make_route("d", "hop-x", 50.0, 0.9, 1);
        let base = route.quality.score();
        let eff = selector.effective_quality(&route);

        // Should be clamped to 1.0 + 0.5 = 1.5x at most
        let max_expected = base * 1.5;
        assert!(
            eff <= max_expected + f64::EPSILON,
            "effective ({eff}) should be <= clamped max ({max_expected})"
        );
    }
}
