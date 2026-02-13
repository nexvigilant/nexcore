// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Linux network implementation via sysfs and sockets.
//!
//! Tier: T3 (μ Mapping + ∂ Boundary + σ Sequence — Linux-specific)

use nexcore_pal::Network;
use nexcore_pal::error::NetworkError;
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;

/// Linux network subsystem.
///
/// Tier: T3 (Linux-specific network implementation)
pub struct LinuxNetwork {
    /// Primary network interface name (e.g., "eth0", "wlan0").
    interface: Option<String>,
    /// Cached connectivity state.
    connected: bool,
}

impl LinuxNetwork {
    /// Create a new network subsystem.
    pub fn new() -> Self {
        Self {
            interface: None,
            connected: false,
        }
    }

    /// Probe for network interfaces.
    pub fn probe() -> Self {
        let mut net = Self::new();

        let net_dir = PathBuf::from("/sys/class/net");
        if !net_dir.exists() {
            return net;
        }

        if let Ok(entries) = fs::read_dir(&net_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy().to_string();

                // Skip loopback
                if name_str == "lo" {
                    continue;
                }

                // Check if interface is up via operstate
                let operstate_path = entry.path().join("operstate");
                if let Ok(state) = fs::read_to_string(&operstate_path) {
                    if state.trim() == "up" {
                        net.interface = Some(name_str);
                        net.connected = true;
                        return net;
                    }
                }

                // Take first non-loopback even if not up
                if net.interface.is_none() {
                    net.interface = Some(name_str);
                }
            }
        }

        net
    }

    /// Create a virtual network for testing.
    pub fn virtual_network(connected: bool) -> Self {
        Self {
            interface: Some("veth0".to_string()),
            connected,
        }
    }
}

impl Default for LinuxNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl Network for LinuxNetwork {
    fn is_connected(&self) -> bool {
        // Re-check operstate if we have an interface
        if let Some(ref iface) = self.interface {
            let operstate = format!("/sys/class/net/{iface}/operstate");
            if let Ok(state) = fs::read_to_string(&operstate) {
                return state.trim() == "up";
            }
        }
        self.connected
    }

    fn send(&mut self, data: &[u8], dest: &str) -> Result<usize, NetworkError> {
        if !self.is_connected() {
            return Err(NetworkError::NoInterface);
        }

        // Basic UDP send — real implementation would support TCP, Unix sockets, etc.
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| NetworkError::SendFailed)?;

        socket
            .send_to(data, dest)
            .map_err(|_| NetworkError::SendFailed)
    }

    fn recv(&mut self, _buf: &mut [u8]) -> Result<usize, NetworkError> {
        if !self.is_connected() {
            return Err(NetworkError::NoInterface);
        }

        // Would need an active socket — placeholder for the OS-level recv
        Err(NetworkError::ReceiveFailed)
    }

    fn local_addr(&self) -> Option<[u8; 16]> {
        // Try to determine local IP by connecting a UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        let addr = socket.local_addr().ok()?;

        let mut result = [0u8; 16];
        match addr.ip() {
            std::net::IpAddr::V4(v4) => {
                let octets = v4.octets();
                result[12..16].copy_from_slice(&octets);
            }
            std::net::IpAddr::V6(v6) => {
                result = v6.octets();
            }
        }
        Some(result)
    }

    fn signal_strength(&self) -> Option<u8> {
        // Read from /proc/net/wireless for WiFi interfaces
        let wireless_info = fs::read_to_string("/proc/net/wireless").ok()?;
        for line in wireless_info.lines().skip(2) {
            // Format: iface | status | quality | ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                // Quality is reported as link/level/noise
                let quality_str = parts[2].trim_end_matches('.');
                if let Ok(quality) = quality_str.parse::<f32>() {
                    // Normalize to 0-100 (typical range is 0-70)
                    let normalized = ((quality / 70.0) * 100.0).min(100.0) as u8;
                    return Some(normalized);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_network_connected() {
        let net = LinuxNetwork::virtual_network(true);
        // Note: is_connected() checks sysfs for "veth0" which won't exist
        // in test environment, so it falls back to cached state
        assert!(net.connected);
    }

    #[test]
    fn virtual_network_disconnected() {
        let net = LinuxNetwork::virtual_network(false);
        assert!(!net.connected);
    }

    #[test]
    fn send_without_connection() {
        let mut net = LinuxNetwork::new();
        let result = net.send(b"test", "127.0.0.1:9999");
        assert!(result.is_err());
    }

    #[test]
    fn default_state() {
        let net = LinuxNetwork::new();
        assert!(net.interface.is_none());
        assert!(!net.connected);
    }
}
