// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Prelude — nexcore-network
//!
//! Convenience re-exports for the most commonly used networking types.
//!
//! Import everything from this module with:
//!
//! ```rust
//! use nexcore_network::prelude::*;
//! ```
//!
//! ## Included Types
//!
//! | Category | Types |
//! |----------|-------|
//! | Interface | [`Interface`], [`InterfaceId`], [`InterfaceType`], [`IpAddr`] |
//! | Connection | [`Connection`], [`ConnectionError`], [`ConnectionState`], [`FailureReason`] |
//! | DNS | [`DnsCacheStats`], [`DnsRecord`], [`DnsResolver`] |
//! | Firewall | [`Firewall`], [`FirewallRule`], [`PacketDisposition`], [`PortRange`], [`Protocol`], [`TrafficDirection`] |
//! | Routing | [`Route`], [`RoutingTable`] |
//! | Monitor | [`ConnectionQuality`], [`InterfaceMonitor`], [`LatencySample`], [`NetworkMonitor`], [`TrafficCounters`] |
//! | Certificate | [`CertFingerprint`], [`CertStatus`], [`CertStore`], [`Certificate`], [`TrustLevel`] |

// ── Interface ────────────────────────────────────────────────────────────────

pub use crate::interface::{Interface, InterfaceId, InterfaceType, IpAddr};

// ── Connection ───────────────────────────────────────────────────────────────

pub use crate::connection::{Connection, ConnectionError, ConnectionState, FailureReason};

// ── DNS ──────────────────────────────────────────────────────────────────────

pub use crate::dns::{DnsCacheStats, DnsRecord, DnsResolver};

// ── Firewall ─────────────────────────────────────────────────────────────────

pub use crate::firewall::{
    Firewall, FirewallRule, PacketDisposition, PortRange, Protocol, TrafficDirection,
};

// ── Routing ──────────────────────────────────────────────────────────────────

pub use crate::route::{Route, RoutingTable};

// ── Monitor ──────────────────────────────────────────────────────────────────

pub use crate::monitor::{
    ConnectionQuality, InterfaceMonitor, LatencySample, NetworkMonitor, TrafficCounters,
};

// ── Certificate ──────────────────────────────────────────────────────────────

pub use crate::certificate::{CertFingerprint, CertStatus, CertStore, Certificate, TrustLevel};
