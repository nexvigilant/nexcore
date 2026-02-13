//! Routing table with quality-sorted route management.
//!
//! ## Primitive Foundation
//! - `Route`: λ (Location) + σ (Sequence) + κ (Comparison)
//! - `RoutingTable`: μ (Mapping) — DashMap of destination -> sorted routes

use crate::topology::{Path, RouteQuality};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Maximum routes stored per destination by default.
pub const DEFAULT_MAX_ROUTES_PER_DEST: usize = 5;

// ============================================================================
// Route — A known path to a destination with quality
// ============================================================================

/// A route to a destination node with quality metrics and path history.
///
/// Tier: T2-C | Dominant: λ (Location)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Destination node ID
    pub destination: String,
    /// Next hop node ID (first relay toward destination)
    pub next_hop: String,
    /// Full path to destination
    pub path: Path,
    /// Route quality metrics
    pub quality: RouteQuality,
    /// Epoch seconds when this route was learned
    pub learned_at: u64,
    /// Source of this route (e.g., "direct", "gossip", "discovery")
    pub source: String,
}

impl Route {
    /// Create a new route.
    pub fn new(
        destination: impl Into<String>,
        next_hop: impl Into<String>,
        path: Path,
        quality: RouteQuality,
    ) -> Self {
        Self {
            destination: destination.into(),
            next_hop: next_hop.into(),
            path,
            quality,
            learned_at: 0,
            source: "direct".to_string(),
        }
    }

    /// Create a route with a specified source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Create a route with a specified learned_at timestamp.
    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.learned_at = ts;
        self
    }

    /// Whether this route is still within a TTL budget.
    pub fn is_viable(&self) -> bool {
        !self.path.is_expired() && self.quality.is_usable()
    }
}

// ============================================================================
// RoutingTable — Concurrent routing table
// ============================================================================

/// Thread-safe routing table backed by DashMap.
///
/// Tier: T3 | Dominant: μ (Mapping)
///
/// Stores multiple routes per destination, sorted by quality (best first).
/// Caps routes per destination to `max_routes_per_dest`.
#[derive(Debug, Clone)]
pub struct RoutingTable {
    /// Map of destination node_id -> Vec<Route> (sorted by quality, best first)
    routes: Arc<DashMap<String, Vec<Route>>>,
    /// Maximum routes stored per destination
    max_routes_per_dest: usize,
}

impl RoutingTable {
    /// Create a new routing table.
    pub fn new(max_routes_per_dest: usize) -> Self {
        Self {
            routes: Arc::new(DashMap::new()),
            max_routes_per_dest: max_routes_per_dest.max(1),
        }
    }

    /// Create with default max routes per destination.
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_MAX_ROUTES_PER_DEST)
    }

    /// Insert or update a route. Maintains quality-sorted order per destination.
    pub fn upsert(&self, route: Route) {
        let dest = route.destination.clone();
        let mut entry = self.routes.entry(dest).or_default();
        let routes = entry.value_mut();

        // Remove existing route with same next_hop (update)
        routes.retain(|r| r.next_hop != route.next_hop);

        // Insert and re-sort by quality (descending)
        routes.push(route);
        routes.sort_by(|a, b| {
            b.quality
                .partial_cmp(&a.quality)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Trim to max
        routes.truncate(self.max_routes_per_dest);
    }

    /// Get the best route to a destination.
    pub fn best_route(&self, destination: &str) -> Option<Route> {
        self.routes
            .get(destination)
            .and_then(|routes| routes.first().cloned())
    }

    /// Get all routes to a destination, ordered by quality.
    pub fn routes_to(&self, destination: &str) -> Vec<Route> {
        self.routes
            .get(destination)
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Remove all routes to a destination.
    pub fn remove_destination(&self, destination: &str) {
        self.routes.remove(destination);
    }

    /// Remove all routes that use a specific next_hop (e.g., when neighbor goes down).
    pub fn remove_via(&self, next_hop: &str) {
        let keys_to_remove: Vec<String> = self
            .routes
            .iter()
            .filter_map(|entry| {
                let remaining: Vec<&Route> = entry
                    .value()
                    .iter()
                    .filter(|r| r.next_hop != next_hop)
                    .collect();
                if remaining.is_empty() {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove destinations with no remaining routes
        for key in &keys_to_remove {
            self.routes.remove(key);
        }

        // Filter routes in remaining destinations
        for mut entry in self.routes.iter_mut() {
            entry.value_mut().retain(|r| r.next_hop != next_hop);
        }
    }

    /// Number of known destinations.
    pub fn destination_count(&self) -> usize {
        self.routes.len()
    }

    /// Total number of routes across all destinations.
    pub fn total_routes(&self) -> usize {
        self.routes.iter().map(|r| r.value().len()).sum()
    }

    /// Get all known destination IDs.
    pub fn destinations(&self) -> Vec<String> {
        self.routes.iter().map(|r| r.key().clone()).collect()
    }

    /// Remove all routes.
    pub fn clear(&self) {
        self.routes.clear();
    }

    /// Check if a route to the destination exists.
    pub fn has_route_to(&self, destination: &str) -> bool {
        self.routes
            .get(destination)
            .map(|r| !r.is_empty())
            .unwrap_or(false)
    }

    /// Create a serializable snapshot of the routing table.
    ///
    /// Iterates the DashMap and collects all destination→routes pairs into a Vec.
    pub fn snapshot(&self) -> Vec<(String, Vec<Route>)> {
        self.routes
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Restore routing table contents from a snapshot.
    ///
    /// Clears the current table and repopulates from the snapshot data.
    pub fn restore(&self, snapshot: &[(String, Vec<Route>)]) {
        self.routes.clear();
        for (dest, routes) in snapshot {
            self.routes.insert(dest.clone(), routes.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_route(dest: &str, next: &str, reliability: f64, hops: u8) -> Route {
        let mut path = Path::new("self", 16);
        for i in 0..hops {
            let _ = path.add_hop(format!("hop-{i}"));
        }
        let _ = path.add_hop(dest);
        Route::new(dest, next, path, RouteQuality::new(50.0, reliability, hops))
    }

    #[test]
    fn routing_table_upsert_and_best() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("dest-1", "via-a", 0.8, 2));
        table.upsert(make_route("dest-1", "via-b", 0.95, 1));

        let best = table.best_route("dest-1");
        assert!(best.is_some());
        assert_eq!(best.as_ref().map(|r| r.next_hop.as_str()), Some("via-b"));
    }

    #[test]
    fn routing_table_quality_sorted() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("d", "low", 0.3, 5));
        table.upsert(make_route("d", "mid", 0.7, 2));
        table.upsert(make_route("d", "high", 0.99, 1));

        let routes = table.routes_to("d");
        assert_eq!(routes.len(), 3);
        assert_eq!(routes[0].next_hop, "high");
        assert_eq!(routes[2].next_hop, "low");
    }

    #[test]
    fn routing_table_max_routes_per_dest() {
        let table = RoutingTable::new(2);
        table.upsert(make_route("d", "a", 0.9, 1));
        table.upsert(make_route("d", "b", 0.8, 2));
        table.upsert(make_route("d", "c", 0.7, 3));

        let routes = table.routes_to("d");
        assert_eq!(routes.len(), 2); // trimmed to max
        assert_eq!(routes[0].next_hop, "a"); // best kept
    }

    #[test]
    fn routing_table_upsert_updates_existing() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("d", "via-a", 0.5, 3));
        table.upsert(make_route("d", "via-a", 0.99, 1)); // update same next_hop

        let routes = table.routes_to("d");
        assert_eq!(routes.len(), 1);
        assert!((routes[0].quality.reliability - 0.99).abs() < f64::EPSILON);
    }

    #[test]
    fn routing_table_remove_destination() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("d1", "a", 0.9, 1));
        table.upsert(make_route("d2", "b", 0.9, 1));
        table.remove_destination("d1");
        assert!(!table.has_route_to("d1"));
        assert!(table.has_route_to("d2"));
    }

    #[test]
    fn routing_table_remove_via() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("d1", "relay", 0.9, 1));
        table.upsert(make_route("d2", "relay", 0.9, 1));
        table.upsert(make_route("d2", "other", 0.8, 2));
        table.remove_via("relay");

        assert!(!table.has_route_to("d1")); // only route was via relay
        assert!(table.has_route_to("d2")); // "other" route remains
        assert_eq!(table.routes_to("d2").len(), 1);
    }

    #[test]
    fn routing_table_destination_count() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("d1", "a", 0.9, 1));
        table.upsert(make_route("d2", "b", 0.9, 1));
        assert_eq!(table.destination_count(), 2);
    }

    #[test]
    fn routing_table_total_routes() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("d1", "a", 0.9, 1));
        table.upsert(make_route("d1", "b", 0.8, 2));
        table.upsert(make_route("d2", "c", 0.7, 3));
        assert_eq!(table.total_routes(), 3);
    }

    #[test]
    fn routing_table_clear() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("d", "a", 0.9, 1));
        table.clear();
        assert_eq!(table.destination_count(), 0);
    }

    #[test]
    fn routing_table_no_route_returns_none() {
        let table = RoutingTable::with_defaults();
        assert!(table.best_route("nonexistent").is_none());
        assert!(!table.has_route_to("nonexistent"));
    }

    #[test]
    fn routing_table_destinations() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("alpha", "a", 0.9, 1));
        table.upsert(make_route("beta", "b", 0.9, 1));
        let dests = table.destinations();
        assert_eq!(dests.len(), 2);
    }

    #[test]
    fn route_is_viable() {
        let route = make_route("dest", "next", 0.9, 2);
        assert!(route.is_viable());

        let bad = make_route("dest", "next", 0.05, 2); // reliability too low
        assert!(!bad.is_viable());
    }

    #[test]
    fn route_with_source_and_timestamp() {
        let route = make_route("d", "n", 0.9, 1)
            .with_source("gossip")
            .with_timestamp(1234567890);
        assert_eq!(route.source, "gossip");
        assert_eq!(route.learned_at, 1234567890);
    }
}
