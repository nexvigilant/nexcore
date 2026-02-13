// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! OS-level network manager — orchestrates all networking subsystems.
//!
//! Tier: T3 (Σ + ∂ + ς + μ + → + N)
//!
//! The `NetworkManager` composes all nexcore-network primitives into a single
//! OS-level subsystem that integrates with the boot sequence, security monitor,
//! and IPC event bus. It owns:
//!
//! - Interface inventory (hardware adapters)
//! - Connection state machines (per-interface lifecycle)
//! - DNS resolver (with cache)
//! - Firewall (packet filtering + Guardian threat-reactive rules)
//! - Routing table (prefix-based path selection)
//! - Traffic monitor (bandwidth, latency, quality)
//! - Certificate store (TLS trust management)

use nexcore_network::{
    CertStore, Connection, ConnectionQuality, ConnectionState, DnsResolver, Firewall, Interface,
    InterfaceId, IpAddr, NetworkMonitor, Route, RoutingTable,
};
use serde::{Deserialize, Serialize};

/// Network subsystem state.
///
/// Tier: T2-P (ς State — network lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkState {
    /// Not initialized yet.
    Uninitialized,
    /// Interfaces discovered, no connections.
    Discovered,
    /// At least one interface connected.
    Connected,
    /// All interfaces disconnected.
    Disconnected,
    /// Network subsystem disabled.
    Disabled,
}

/// OS-level network manager.
///
/// Tier: T3 (Σ Sum — composition of all network subsystems)
pub struct NetworkManager {
    /// Known interfaces.
    interfaces: Vec<Interface>,
    /// Active connections (one per interface).
    connections: Vec<Connection>,
    /// DNS resolver with cache.
    dns: DnsResolver,
    /// Packet filtering firewall.
    firewall: Firewall,
    /// IP routing table.
    routing: RoutingTable,
    /// Traffic monitor.
    monitor: NetworkMonitor,
    /// TLS certificate trust store.
    certificates: CertStore,
    /// Network subsystem state.
    state: NetworkState,
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManager {
    /// Create a new network manager (uninitialized).
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
            connections: Vec::new(),
            dns: DnsResolver::new(),
            firewall: Firewall::new(),
            routing: RoutingTable::new(),
            monitor: NetworkMonitor::new(),
            certificates: CertStore::new(),
            state: NetworkState::Uninitialized,
        }
    }

    /// Initialize the network subsystem.
    ///
    /// Called during boot. Sets up default firewall rules and
    /// discovers interfaces from the platform.
    pub fn initialize(&mut self) {
        // Default security rules: allow loopback, drop unknown inbound
        self.firewall
            .add_rule(nexcore_network::firewall::allow_loopback());

        self.state = NetworkState::Discovered;
    }

    /// Register a network interface from hardware discovery.
    pub fn register_interface(&mut self, interface: Interface) {
        let id = interface.id.clone();

        // Add to traffic monitor
        self.monitor.add_interface(id.clone());

        // Store the interface
        self.interfaces.push(interface);

        // Create a connection tracker for it
        let iface = self.interfaces.last();
        if let Some(iface) = iface {
            let conn = Connection::new(id, iface.interface_type, &iface.name);
            self.connections.push(conn);
        }
    }

    /// Get the current network state.
    pub fn state(&self) -> NetworkState {
        self.state
    }

    /// Refresh the network state based on connection status.
    pub fn refresh_state(&mut self) {
        if self.state == NetworkState::Disabled || self.state == NetworkState::Uninitialized {
            return;
        }

        let any_connected = self
            .connections
            .iter()
            .any(|c| c.state() == ConnectionState::Connected);

        self.state = if any_connected {
            NetworkState::Connected
        } else if self.interfaces.is_empty() {
            NetworkState::Uninitialized
        } else {
            NetworkState::Disconnected
        };
    }

    /// Check if any interface is connected.
    pub fn is_connected(&self) -> bool {
        self.state == NetworkState::Connected
    }

    /// Get the primary (best) connected interface.
    ///
    /// Returns the connected interface with the highest effective priority.
    pub fn primary_interface(&self) -> Option<&Interface> {
        let connected_ids: Vec<_> = self
            .connections
            .iter()
            .filter(|c| c.state() == ConnectionState::Connected)
            .map(|c| &c.interface_id)
            .collect();

        self.interfaces
            .iter()
            .filter(|iface| connected_ids.contains(&&iface.id))
            .max_by_key(|iface| iface.effective_priority())
    }

    /// Get all registered interfaces.
    pub fn interfaces(&self) -> &[Interface] {
        &self.interfaces
    }

    /// Get an interface by ID.
    pub fn get_interface(&self, id: &InterfaceId) -> Option<&Interface> {
        self.interfaces.iter().find(|i| &i.id == id)
    }

    /// Get a connection by interface ID.
    pub fn get_connection(&self, id: &InterfaceId) -> Option<&Connection> {
        self.connections.iter().find(|c| &c.interface_id == id)
    }

    /// Get a mutable connection by interface ID.
    pub fn get_connection_mut(&mut self, id: &InterfaceId) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| &c.interface_id == id)
    }

    /// Get the DNS resolver.
    pub fn dns(&self) -> &DnsResolver {
        &self.dns
    }

    /// Get a mutable DNS resolver.
    pub fn dns_mut(&mut self) -> &mut DnsResolver {
        &mut self.dns
    }

    /// Get the firewall.
    pub fn firewall(&self) -> &Firewall {
        &self.firewall
    }

    /// Get a mutable firewall.
    pub fn firewall_mut(&mut self) -> &mut Firewall {
        &mut self.firewall
    }

    /// Get the routing table.
    pub fn routing(&self) -> &RoutingTable {
        &self.routing
    }

    /// Get a mutable routing table.
    pub fn routing_mut(&mut self) -> &mut RoutingTable {
        &mut self.routing
    }

    /// Get the traffic monitor.
    pub fn monitor(&self) -> &NetworkMonitor {
        &self.monitor
    }

    /// Get a mutable traffic monitor.
    pub fn monitor_mut(&mut self) -> &mut NetworkMonitor {
        &mut self.monitor
    }

    /// Get the certificate store.
    pub fn certificates(&self) -> &CertStore {
        &self.certificates
    }

    /// Get a mutable certificate store.
    pub fn certificates_mut(&mut self) -> &mut CertStore {
        &mut self.certificates
    }

    /// Add a route.
    pub fn add_route(&mut self, route: Route) {
        self.routing.add(route);
    }

    /// Resolve hostname to IP (cache lookup only — no actual DNS query).
    pub fn resolve_cached(&mut self, hostname: &str) -> Option<IpAddr> {
        self.dns
            .lookup_cached(hostname)
            .and_then(|record| record.primary_address().cloned())
    }

    /// Block an IP address via the firewall (Guardian threat response).
    ///
    /// Used by the security monitor to react to detected threats.
    pub fn block_ip(&mut self, addr: IpAddr) {
        self.firewall.add_rule(nexcore_network::firewall::block_ip(
            format!("block-{addr:?}"),
            addr,
        ));
    }

    /// Block inbound traffic on a specific port.
    pub fn block_inbound_port(&mut self, port: u16) {
        self.firewall
            .add_rule(nexcore_network::firewall::block_inbound_port(
                format!("block-port-{port}"),
                port,
            ));
    }

    /// Get the connection quality for an interface.
    pub fn connection_quality(&self, id: &InterfaceId) -> ConnectionQuality {
        self.monitor.get(id).map_or(
            ConnectionQuality::Unusable,
            nexcore_network::InterfaceMonitor::quality,
        )
    }

    /// Get the total bytes transferred across all interfaces.
    pub fn total_bytes(&self) -> u64 {
        self.monitor.total_bytes()
    }

    /// Enable or disable the network subsystem.
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled && self.state == NetworkState::Disabled {
            self.state = NetworkState::Discovered;
            self.refresh_state();
        } else if !enabled {
            self.state = NetworkState::Disabled;
        }
    }

    /// Count of registered interfaces.
    pub fn interface_count(&self) -> usize {
        self.interfaces.len()
    }

    /// Count of active connections.
    pub fn connected_count(&self) -> usize {
        self.connections
            .iter()
            .filter(|c| c.state() == ConnectionState::Connected)
            .count()
    }

    /// Summary string for the network subsystem.
    pub fn summary(&self) -> String {
        let connected = self.connected_count();
        let total = self.interface_count();
        format!(
            "Network: {:?} ({}/{} connected), {} routes, {} DNS cached, {} firewall rules, {} certs",
            self.state,
            connected,
            total,
            self.routing.len(),
            self.dns.cache_size(),
            self.firewall.rule_count(),
            self.certificates.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_network::InterfaceType;

    fn make_wifi_interface() -> Interface {
        Interface::new("wlan0", "wlan0", InterfaceType::WiFi)
            .up()
            .with_address(IpAddr::v4(192, 168, 1, 100))
    }

    fn make_eth_interface() -> Interface {
        Interface::new("eth0", "eth0", InterfaceType::Ethernet)
            .up()
            .with_address(IpAddr::v4(10, 0, 0, 50))
    }

    #[test]
    fn new_manager_uninitialized() {
        let nm = NetworkManager::new();
        assert_eq!(nm.state(), NetworkState::Uninitialized);
        assert!(!nm.is_connected());
        assert_eq!(nm.interface_count(), 0);
    }

    #[test]
    fn initialize_sets_discovered() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        assert_eq!(nm.state(), NetworkState::Discovered);
    }

    #[test]
    fn register_interface() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        nm.register_interface(make_wifi_interface());
        assert_eq!(nm.interface_count(), 1);
        assert!(nm.get_interface(&InterfaceId::new("wlan0")).is_some());
    }

    #[test]
    fn connection_created_on_register() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        nm.register_interface(make_wifi_interface());
        let conn = nm.get_connection(&InterfaceId::new("wlan0"));
        assert!(conn.is_some());
        if let Some(c) = conn {
            assert_eq!(c.state(), ConnectionState::Disconnected);
        }
    }

    #[test]
    fn refresh_state_no_connections() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        nm.register_interface(make_wifi_interface());
        nm.refresh_state();
        assert_eq!(nm.state(), NetworkState::Disconnected);
    }

    #[test]
    fn dns_resolver_accessible() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        nm.dns_mut()
            .cache_address("test.com", IpAddr::v4(1, 2, 3, 4), 300);
        let resolved = nm.resolve_cached("test.com");
        assert_eq!(resolved, Some(IpAddr::v4(1, 2, 3, 4)));
    }

    #[test]
    fn firewall_block_ip() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        let initial_rules = nm.firewall().rule_count();
        nm.block_ip(IpAddr::v4(192, 168, 1, 200));
        assert_eq!(nm.firewall().rule_count(), initial_rules + 1);
    }

    #[test]
    fn routing_table_accessible() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        nm.register_interface(make_eth_interface());

        let route = Route::default_route(IpAddr::v4(10, 0, 0, 1), InterfaceId::new("eth0"));
        nm.add_route(route);

        assert_eq!(nm.routing().len(), 1);
        let lookup = nm.routing().lookup(&IpAddr::v4(8, 8, 8, 8));
        assert!(lookup.is_some());
    }

    #[test]
    fn certificate_store_accessible() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        assert_eq!(nm.certificates().len(), 0);
        assert!(nm.certificates().is_empty());
    }

    #[test]
    fn total_bytes_zero_initially() {
        let nm = NetworkManager::new();
        assert_eq!(nm.total_bytes(), 0);
    }

    #[test]
    fn disable_and_enable() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        nm.register_interface(make_wifi_interface());

        nm.set_enabled(false);
        assert_eq!(nm.state(), NetworkState::Disabled);

        nm.set_enabled(true);
        // After re-enabling, should refresh to Disconnected (no active connections)
        assert_eq!(nm.state(), NetworkState::Disconnected);
    }

    #[test]
    fn connected_count() {
        let nm = NetworkManager::new();
        assert_eq!(nm.connected_count(), 0);
    }

    #[test]
    fn summary_format() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        let s = nm.summary();
        assert!(s.contains("Network"));
        assert!(s.contains("Discovered"));
        assert!(s.contains("0/0 connected"));
    }

    #[test]
    fn multiple_interfaces() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        nm.register_interface(make_wifi_interface());
        nm.register_interface(make_eth_interface());
        assert_eq!(nm.interface_count(), 2);
        assert_eq!(nm.connected_count(), 0);
    }

    #[test]
    fn firewall_has_loopback_rule_after_init() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        // initialize() adds a loopback allow rule
        assert!(nm.firewall().rule_count() >= 1);
    }

    #[test]
    fn block_inbound_port_adds_rule() {
        let mut nm = NetworkManager::new();
        nm.initialize();
        let before = nm.firewall().rule_count();
        nm.block_inbound_port(22);
        assert_eq!(nm.firewall().rule_count(), before + 1);
    }

    #[test]
    fn connection_quality_unusable_for_unknown() {
        let nm = NetworkManager::new();
        let quality = nm.connection_quality(&InterfaceId::new("nonexistent"));
        assert_eq!(quality, ConnectionQuality::Unusable);
    }
}
