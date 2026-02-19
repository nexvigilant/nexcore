// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Routing table — determines which interface handles which traffic.
//!
//! Tier: T2-C (→ Causality + κ Comparison + λ Location)
//!
//! Routing is causal (→): a destination address CAUSES a packet to flow
//! through a specific interface. Routes are compared (κ) by metric to
//! select the best path. Addresses are locations (λ) in the network.

use crate::interface::{InterfaceId, IpAddr};
use serde::{Deserialize, Serialize};

/// A routing table entry.
///
/// Tier: T2-C (→ + κ + λ — causal path selection by location)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Destination network address.
    pub destination: IpAddr,
    /// Subnet prefix length (e.g., 24 for /24 = 255.255.255.0).
    pub prefix_len: u8,
    /// Gateway address (None = directly connected).
    pub gateway: Option<IpAddr>,
    /// Interface to send packets through.
    pub interface_id: InterfaceId,
    /// Route metric (lower = preferred).
    pub metric: u32,
    /// Whether this is the default route.
    pub is_default: bool,
}

impl Route {
    /// Create a new route.
    pub fn new(destination: IpAddr, prefix_len: u8, interface_id: InterfaceId) -> Self {
        Self {
            destination,
            prefix_len,
            interface_id,
            gateway: None,
            metric: 100,
            is_default: false,
        }
    }

    /// Create a default route (0.0.0.0/0).
    pub fn default_route(gateway: IpAddr, interface_id: InterfaceId) -> Self {
        Self {
            destination: IpAddr::unspecified_v4(),
            prefix_len: 0,
            interface_id,
            gateway: Some(gateway),
            metric: 100,
            is_default: true,
        }
    }

    /// Builder: set gateway.
    pub fn via(mut self, gateway: IpAddr) -> Self {
        self.gateway = Some(gateway);
        self
    }

    /// Builder: set metric.
    pub fn with_metric(mut self, metric: u32) -> Self {
        self.metric = metric;
        self
    }

    /// Builder: mark as default route.
    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    /// Check if a destination address matches this route.
    ///
    /// Uses prefix matching: the destination must match
    /// the route's network/prefix_len bits.
    pub fn matches(&self, dest: &IpAddr) -> bool {
        if self.is_default {
            return true; // Default route matches everything
        }

        match (&self.destination, dest) {
            (IpAddr::V4(route_addr), IpAddr::V4(pkt_addr)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                if self.prefix_len >= 32 {
                    return route_addr == pkt_addr;
                }
                let mask = u32::MAX << (32 - self.prefix_len);
                let route_net = u32::from_be_bytes(*route_addr) & mask;
                let pkt_net = u32::from_be_bytes(*pkt_addr) & mask;
                route_net == pkt_net
            }
            // IPv6 matching
            (IpAddr::V6(route_addr), IpAddr::V6(pkt_addr)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                // Compare byte-by-byte up to prefix_len bits
                let full_bytes = (self.prefix_len / 8) as usize;
                let remaining_bits = self.prefix_len % 8;

                // Full bytes must match
                if route_addr[..full_bytes] != pkt_addr[..full_bytes] {
                    return false;
                }

                // Check remaining bits in the next byte
                if remaining_bits > 0 && full_bytes < 16 {
                    let mask = 0xFF << (8 - remaining_bits);
                    if (route_addr[full_bytes] & mask) != (pkt_addr[full_bytes] & mask) {
                        return false;
                    }
                }

                true
            }
            _ => false, // V4/V6 mismatch
        }
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let dest = if self.is_default {
            "default".to_string()
        } else {
            format!("{}/{}", self.destination.to_string_repr(), self.prefix_len)
        };
        let via = self.gateway.as_ref().map_or("direct".to_string(), |g| {
            format!("via {}", g.to_string_repr())
        });
        format!(
            "{} {} dev {} metric {}",
            dest,
            via,
            self.interface_id.as_str(),
            self.metric,
        )
    }
}

/// Routing table — the set of all routes.
///
/// Tier: T2-C (→ + κ — ordered causal path selection)
#[derive(Debug, Default)]
pub struct RoutingTable {
    /// All routes, ordered by specificity (longest prefix first).
    routes: Vec<Route>,
}

impl RoutingTable {
    /// Create an empty routing table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a route.
    pub fn add(&mut self, route: Route) {
        self.routes.push(route);
        // Sort by specificity (longest prefix first), then metric
        self.routes.sort_by(|a, b| {
            b.prefix_len
                .cmp(&a.prefix_len)
                .then(a.metric.cmp(&b.metric))
        });
    }

    /// Remove all routes for an interface.
    pub fn remove_interface(&mut self, interface_id: &InterfaceId) {
        self.routes.retain(|r| &r.interface_id != interface_id);
    }

    /// Look up the best route for a destination.
    ///
    /// Returns the most specific matching route with the lowest metric.
    pub fn lookup(&self, dest: &IpAddr) -> Option<&Route> {
        // Routes are sorted by specificity, first match wins
        self.routes.iter().find(|r| r.matches(dest))
    }

    /// Get the default route.
    pub fn default_route(&self) -> Option<&Route> {
        self.routes.iter().find(|r| r.is_default)
    }

    /// Get all routes.
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Number of routes.
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let default = self
            .default_route()
            .map_or("none".to_string(), |r| r.interface_id.as_str().to_string());
        format!(
            "Routes: {} entries, default via {}",
            self.routes.len(),
            default,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eth0() -> InterfaceId {
        InterfaceId::new("eth0")
    }

    fn wlan0() -> InterfaceId {
        InterfaceId::new("wlan0")
    }

    #[test]
    fn route_new() {
        let r = Route::new(IpAddr::v4(192, 168, 1, 0), 24, eth0());
        assert!(!r.is_default);
        assert!(r.gateway.is_none());
        assert_eq!(r.metric, 100);
    }

    #[test]
    fn default_route() {
        let r = Route::default_route(IpAddr::v4(192, 168, 1, 1), eth0());
        assert!(r.is_default);
        assert!(r.gateway.is_some());
    }

    #[test]
    fn route_matches_exact() {
        let r = Route::new(IpAddr::v4(192, 168, 1, 0), 32, eth0());
        assert!(r.matches(&IpAddr::v4(192, 168, 1, 0)));
        assert!(!r.matches(&IpAddr::v4(192, 168, 1, 1)));
    }

    #[test]
    fn route_matches_subnet() {
        let r = Route::new(IpAddr::v4(192, 168, 1, 0), 24, eth0());
        assert!(r.matches(&IpAddr::v4(192, 168, 1, 100)));
        assert!(r.matches(&IpAddr::v4(192, 168, 1, 255)));
        assert!(!r.matches(&IpAddr::v4(192, 168, 2, 1)));
    }

    #[test]
    fn route_matches_wide_subnet() {
        let r = Route::new(IpAddr::v4(10, 0, 0, 0), 8, eth0());
        assert!(r.matches(&IpAddr::v4(10, 0, 0, 1)));
        assert!(r.matches(&IpAddr::v4(10, 255, 255, 255)));
        assert!(!r.matches(&IpAddr::v4(11, 0, 0, 1)));
    }

    #[test]
    fn default_route_matches_everything() {
        let r = Route::default_route(IpAddr::v4(192, 168, 1, 1), eth0());
        assert!(r.matches(&IpAddr::v4(8, 8, 8, 8)));
        assert!(r.matches(&IpAddr::v4(1, 1, 1, 1)));
    }

    #[test]
    fn routing_table_longest_prefix_wins() {
        let mut table = RoutingTable::new();

        // More specific route via wlan0
        table.add(Route::new(IpAddr::v4(192, 168, 1, 0), 24, wlan0()));
        // Less specific default via eth0
        table.add(Route::default_route(IpAddr::v4(10, 0, 0, 1), eth0()));

        // 192.168.1.x should go via wlan0 (more specific)
        let route = table.lookup(&IpAddr::v4(192, 168, 1, 50));
        assert!(route.is_some());
        if let Some(r) = route {
            assert_eq!(r.interface_id, wlan0());
        }

        // 8.8.8.8 should go via eth0 (default route)
        let route = table.lookup(&IpAddr::v4(8, 8, 8, 8));
        assert!(route.is_some());
        if let Some(r) = route {
            assert_eq!(r.interface_id, eth0());
        }
    }

    #[test]
    fn routing_table_metric_tiebreak() {
        let mut table = RoutingTable::new();
        table.add(Route::default_route(IpAddr::v4(192, 168, 1, 1), wlan0()).with_metric(200));
        table.add(Route::default_route(IpAddr::v4(10, 0, 0, 1), eth0()).with_metric(100));

        // Lower metric wins
        let route = table.lookup(&IpAddr::v4(8, 8, 8, 8));
        assert!(route.is_some());
        if let Some(r) = route {
            assert_eq!(r.metric, 100);
        }
    }

    #[test]
    fn routing_table_remove_interface() {
        let mut table = RoutingTable::new();
        table.add(Route::new(IpAddr::v4(192, 168, 1, 0), 24, eth0()));
        table.add(Route::default_route(IpAddr::v4(10, 0, 0, 1), wlan0()));
        assert_eq!(table.len(), 2);

        table.remove_interface(&eth0());
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn routing_table_empty() {
        let table = RoutingTable::new();
        assert!(table.is_empty());
        assert!(table.lookup(&IpAddr::v4(8, 8, 8, 8)).is_none());
        assert!(table.default_route().is_none());
    }

    #[test]
    fn route_summary() {
        let r = Route::new(IpAddr::v4(192, 168, 1, 0), 24, eth0()).via(IpAddr::v4(192, 168, 1, 1));
        let s = r.summary();
        assert!(s.contains("192.168.1.0/24"));
        assert!(s.contains("via 192.168.1.1"));
        assert!(s.contains("eth0"));
    }

    #[test]
    fn routing_table_summary() {
        let table = RoutingTable::new();
        let s = table.summary();
        assert!(s.contains("Routes"));
        assert!(s.contains("0 entries"));
    }

    #[test]
    fn route_builder_chain() {
        let r = Route::new(IpAddr::v4(10, 0, 0, 0), 8, eth0())
            .via(IpAddr::v4(10, 0, 0, 1))
            .with_metric(50)
            .as_default();
        assert_eq!(r.metric, 50);
        assert!(r.is_default);
        assert!(r.gateway.is_some());
    }
}
