// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Network interface abstraction.
//!
//! Tier: T2-C (Σ + μ + ∃ — sum of interface types mapped to hardware existence)
//!
//! Every physical or virtual network adapter is an `Interface`. The OS manages
//! multiple interfaces simultaneously — WiFi + cellular on a phone, Ethernet +
//! WiFi on a desktop, Bluetooth on a watch.

use serde::{Deserialize, Serialize};

/// Unique identifier for a network interface.
///
/// Tier: T2-P (∃ Existence — identity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InterfaceId(String);

impl InterfaceId {
    /// Create a new interface ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Network interface type.
///
/// Tier: T2-P (Σ Sum — enumeration of transport types)
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterfaceType {
    /// WiFi (802.11).
    WiFi,
    /// Cellular (LTE/5G).
    Cellular,
    /// Wired Ethernet.
    Ethernet,
    /// Bluetooth (PAN/tethering).
    Bluetooth,
    /// Loopback (localhost).
    Loopback,
    /// VPN tunnel.
    Vpn,
    /// Unknown type.
    Unknown,
}

impl InterfaceType {
    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::WiFi => "WiFi",
            Self::Cellular => "Cellular",
            Self::Ethernet => "Ethernet",
            Self::Bluetooth => "Bluetooth",
            Self::Loopback => "Loopback",
            Self::Vpn => "VPN",
            Self::Unknown => "Unknown",
        }
    }

    /// Default priority (lower = preferred). Used for route selection.
    pub const fn default_priority(&self) -> u8 {
        match self {
            Self::Ethernet => 10,
            Self::WiFi => 20,
            Self::Vpn => 30,
            Self::Cellular => 40,
            Self::Bluetooth => 50,
            Self::Loopback => 100,
            Self::Unknown => 200,
        }
    }

    /// Whether this interface type is metered (costs money per byte).
    pub const fn is_metered(&self) -> bool {
        matches!(self, Self::Cellular)
    }
}

/// IP address representation (supports IPv4 and IPv6).
///
/// Tier: T2-P (λ Location — network address)
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IpAddr {
    /// IPv4 address (4 bytes).
    V4([u8; 4]),
    /// IPv6 address (16 bytes).
    V6([u8; 16]),
}

impl IpAddr {
    /// Create an IPv4 address.
    pub const fn v4(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self::V4([a, b, c, d])
    }

    /// Create the loopback address (127.0.0.1).
    pub const fn loopback_v4() -> Self {
        Self::V4([127, 0, 0, 1])
    }

    /// Create the unspecified address (0.0.0.0).
    pub const fn unspecified_v4() -> Self {
        Self::V4([0, 0, 0, 0])
    }

    /// Whether this is a loopback address.
    pub fn is_loopback(&self) -> bool {
        match self {
            Self::V4(b) => b[0] == 127,
            Self::V6(b) => b[..15].iter().all(|&x| x == 0) && b[15] == 1,
        }
    }

    /// Whether this is a private/local address.
    pub fn is_private(&self) -> bool {
        match self {
            Self::V4(b) => {
                b[0] == 10
                    || (b[0] == 172 && (16..=31).contains(&b[1]))
                    || (b[0] == 192 && b[1] == 168)
            }
            Self::V6(b) => b[0] == 0xfe && (b[1] & 0xc0) == 0x80,
        }
    }

    /// Format as string.
    pub fn to_string_repr(&self) -> String {
        match self {
            Self::V4(b) => format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]),
            Self::V6(b) => {
                // b is exactly [u8; 16]; indices i*2 and i*2+1 for i in 0..8 are always valid
                let parts: Vec<String> = (0_u8..8_u8)
                    .map(|i| {
                        let lo = usize::from(i.saturating_mul(2));
                        let hi = usize::from(i.saturating_mul(2).saturating_add(1));
                        let pair = [
                            b.get(lo).copied().unwrap_or(0),
                            b.get(hi).copied().unwrap_or(0),
                        ];
                        format!("{:x}", u16::from_be_bytes(pair))
                    })
                    .collect();
                parts.join(":")
            }
        }
    }
}

/// Network interface descriptor.
///
/// Tier: T2-C (Σ + μ + ∃ + ς — typed, mapped, existent, stateful)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    /// Unique interface ID.
    pub id: InterfaceId,
    /// Human-readable name (e.g., "wlan0", "eth0").
    pub name: String,
    /// Interface type.
    pub interface_type: InterfaceType,
    /// Whether the interface is currently up.
    pub is_up: bool,
    /// Assigned IP addresses.
    pub addresses: Vec<IpAddr>,
    /// Hardware (MAC) address, if available.
    pub mac: Option<[u8; 6]>,
    /// Signal strength (0-100), if applicable.
    pub signal_strength: Option<u8>,
    /// Link speed in Mbps, if known.
    pub link_speed_mbps: Option<u32>,
    /// Whether this interface is metered.
    pub is_metered: bool,
}

impl Interface {
    /// Create a new interface.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        interface_type: InterfaceType,
    ) -> Self {
        Self {
            id: InterfaceId::new(id),
            name: name.into(),
            interface_type,
            is_up: false,
            addresses: Vec::new(),
            mac: None,
            signal_strength: None,
            link_speed_mbps: None,
            is_metered: interface_type.is_metered(),
        }
    }

    /// Builder: set the interface as up.
    pub fn up(mut self) -> Self {
        self.is_up = true;
        self
    }

    /// Builder: add an IP address.
    pub fn with_address(mut self, addr: IpAddr) -> Self {
        self.addresses.push(addr);
        self
    }

    /// Builder: set MAC address.
    pub fn with_mac(mut self, mac: [u8; 6]) -> Self {
        self.mac = Some(mac);
        self
    }

    /// Builder: set signal strength.
    pub fn with_signal(mut self, strength: u8) -> Self {
        self.signal_strength = Some(strength.min(100));
        self
    }

    /// Builder: set link speed.
    pub fn with_speed(mut self, mbps: u32) -> Self {
        self.link_speed_mbps = Some(mbps);
        self
    }

    /// Whether this interface has any usable address.
    pub fn has_address(&self) -> bool {
        !self.addresses.is_empty()
    }

    /// Whether this interface is ready for traffic.
    pub fn is_ready(&self) -> bool {
        self.is_up && self.has_address()
    }

    /// Get the primary (first) address.
    pub fn primary_address(&self) -> Option<&IpAddr> {
        self.addresses.first()
    }

    /// Effective routing priority (lower = preferred).
    pub fn effective_priority(&self) -> u16 {
        let base = u16::from(self.interface_type.default_priority());
        let signal_penalty: u16 = match self.signal_strength {
            Some(s) if s < 30 => 50, // Weak signal
            Some(s) if s < 60 => 20, // Moderate signal
            _ => 0,
        };
        base.saturating_add(signal_penalty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_id() {
        let id = InterfaceId::new("wlan0");
        assert_eq!(id.as_str(), "wlan0");
    }

    #[test]
    fn interface_type_labels() {
        assert_eq!(InterfaceType::WiFi.label(), "WiFi");
        assert_eq!(InterfaceType::Cellular.label(), "Cellular");
        assert_eq!(InterfaceType::Ethernet.label(), "Ethernet");
    }

    #[test]
    fn interface_type_priority() {
        assert!(
            InterfaceType::Ethernet.default_priority() < InterfaceType::WiFi.default_priority()
        );
        assert!(
            InterfaceType::WiFi.default_priority() < InterfaceType::Cellular.default_priority()
        );
    }

    #[test]
    fn interface_type_metered() {
        assert!(InterfaceType::Cellular.is_metered());
        assert!(!InterfaceType::WiFi.is_metered());
        assert!(!InterfaceType::Ethernet.is_metered());
    }

    #[test]
    fn ip_addr_v4() {
        let addr = IpAddr::v4(192, 168, 1, 100);
        assert!(addr.is_private());
        assert!(!addr.is_loopback());
        assert_eq!(addr.to_string_repr(), "192.168.1.100");
    }

    #[test]
    fn ip_addr_loopback() {
        let addr = IpAddr::loopback_v4();
        assert!(addr.is_loopback());
        assert!(!addr.is_private());
    }

    #[test]
    fn ip_addr_private_ranges() {
        assert!(IpAddr::v4(10, 0, 0, 1).is_private());
        assert!(IpAddr::v4(172, 16, 0, 1).is_private());
        assert!(IpAddr::v4(192, 168, 0, 1).is_private());
        assert!(!IpAddr::v4(8, 8, 8, 8).is_private());
    }

    #[test]
    fn interface_builder() {
        let iface = Interface::new("wlan0", "WiFi Adapter", InterfaceType::WiFi)
            .up()
            .with_address(IpAddr::v4(192, 168, 1, 100))
            .with_signal(75)
            .with_speed(300)
            .with_mac([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);

        assert!(iface.is_up);
        assert!(iface.has_address());
        assert!(iface.is_ready());
        assert_eq!(iface.signal_strength, Some(75));
        assert_eq!(iface.link_speed_mbps, Some(300));
        assert!(iface.mac.is_some());
    }

    #[test]
    fn interface_not_ready_when_down() {
        let iface = Interface::new("eth0", "Ethernet", InterfaceType::Ethernet)
            .with_address(IpAddr::v4(10, 0, 0, 5));
        assert!(!iface.is_ready()); // has address but is down
    }

    #[test]
    fn interface_not_ready_without_address() {
        let iface = Interface::new("wlan0", "WiFi", InterfaceType::WiFi).up();
        assert!(!iface.is_ready()); // is up but no address
    }

    #[test]
    fn effective_priority_weak_signal() {
        let strong = Interface::new("w0", "WiFi", InterfaceType::WiFi).with_signal(80);
        let weak = Interface::new("w1", "WiFi", InterfaceType::WiFi).with_signal(20);
        assert!(strong.effective_priority() < weak.effective_priority());
    }

    #[test]
    fn interface_metered_from_type() {
        let cell = Interface::new("rmnet0", "Cellular", InterfaceType::Cellular);
        assert!(cell.is_metered);
        let wifi = Interface::new("wlan0", "WiFi", InterfaceType::WiFi);
        assert!(!wifi.is_metered);
    }

    #[test]
    fn signal_strength_clamped() {
        let iface = Interface::new("w0", "WiFi", InterfaceType::WiFi).with_signal(255); // over 100
        assert_eq!(iface.signal_strength, Some(100));
    }
}
