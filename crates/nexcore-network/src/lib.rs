// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # nexcore-network — NexCore OS Networking Subsystem
//!
//! Tier: T3 (Σ + μ + ∂ + ς + σ + π + ∝ + N + ν + κ + → + λ + ∃)
//!
//! Complete networking stack for NexCore OS sitting above `nexcore-pal::Network`.
//! Manages interfaces, connections, DNS, routing, firewall rules, traffic
//! monitoring, and TLS certificate trust stores across all three form factors
//! (watch, phone, desktop).
//!
//! ## Module Architecture
//!
//! | Module | Tier | Dominant | Purpose |
//! |--------|------|----------|---------|
//! | `interface` | T2-C | Σ + μ + ∃ | Network interface abstraction |
//! | `connection` | T2-C | ς + σ + ∂ | Connection state machine |
//! | `dns` | T2-C | μ + π + ν | DNS resolver with caching |
//! | `firewall` | T2-C | ∂ + κ + → | Packet filtering rules |
//! | `route` | T2-C | → + κ + λ | Routing table with prefix matching |
//! | `monitor` | T2-C | N + ν + σ | Bandwidth, latency, and quality tracking |
//! | `certificate` | T2-C | ∂ + π + ∝ | TLS certificate trust store |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_network::{Interface, InterfaceType, IpAddr, Connection, ConnectionState};
//!
//! // Create a WiFi interface
//! let iface = Interface::new("wlan0", "wlan0", InterfaceType::WiFi)
//!     .up()
//!     .with_address(IpAddr::v4(192, 168, 1, 100));
//!
//! assert!(iface.is_up);
//! assert_eq!(iface.interface_type, InterfaceType::WiFi);
//!
//! // Create a connection tracking its lifecycle
//! let conn = Connection::new(iface.id.clone(), iface.interface_type, "HomeWiFi");
//! assert_eq!(conn.state(), ConnectionState::Disconnected);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

// ── Modules ──────────────────────────────────────────────────────────

pub mod certificate;
pub mod composites;
pub mod connection;
pub mod dns;
pub mod firewall;
pub mod interface;
pub mod monitor;
pub mod prelude;
pub mod route;

// ── Re-exports: interface ────────────────────────────────────────────

pub use interface::{Interface, InterfaceId, InterfaceType, IpAddr};

// ── Re-exports: connection ───────────────────────────────────────────

pub use connection::{Connection, ConnectionError, ConnectionState, FailureReason};

// ── Re-exports: dns ──────────────────────────────────────────────────

pub use dns::{DnsCacheStats, DnsRecord, DnsResolver};

// ── Re-exports: firewall ─────────────────────────────────────────────

pub use firewall::{
    Firewall, FirewallRule, PacketDisposition, PortRange, Protocol, TrafficDirection,
};

/// Backward-compatible re-export.
#[deprecated(note = "use PacketDisposition — F2 equivocation fix")]
#[allow(
    deprecated,
    reason = "re-exporting our own deprecated type alias for backward compatibility"
)]
pub use firewall::Action;

// ── Re-exports: route ────────────────────────────────────────────────

pub use route::{Route, RoutingTable};

// ── Re-exports: monitor ──────────────────────────────────────────────

pub use monitor::{
    ConnectionQuality, InterfaceMonitor, LatencySample, NetworkMonitor, TrafficCounters,
};

// ── Re-exports: certificate ──────────────────────────────────────────

pub use certificate::{CertFingerprint, CertStatus, CertStore, Certificate, TrustLevel};
