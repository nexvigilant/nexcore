// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Traffic monitoring — bandwidth, latency, and connection quality tracking.
//!
//! Tier: T2-C (N Quantity + ν Frequency + σ Sequence)
//!
//! Monitors are quantitative (N) measurements taken at regular frequency (ν)
//! forming time-series sequences (σ). Used for:
//! - Bandwidth metering (especially on cellular)
//! - Quality-of-service decisions
//! - Guardian security anomaly detection

use crate::interface::InterfaceId;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Traffic counters for a single interface.
///
/// Tier: T2-P (N Quantity — byte counts)
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrafficCounters {
    /// Bytes sent.
    pub bytes_sent: u64,
    /// Bytes received.
    pub bytes_received: u64,
    /// Packets sent.
    pub packets_sent: u64,
    /// Packets received.
    pub packets_received: u64,
    /// Packets dropped (send failures).
    pub packets_dropped: u64,
    /// Errors encountered.
    pub errors: u64,
}

impl TrafficCounters {
    /// Total bytes (sent + received).
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent.saturating_add(self.bytes_received)
    }

    /// Total packets (sent + received).
    pub fn total_packets(&self) -> u64 {
        self.packets_sent.saturating_add(self.packets_received)
    }

    /// Packet loss rate (0.0 - 1.0).
    pub fn packet_loss_rate(&self) -> f64 {
        let total = self.total_packets().saturating_add(self.packets_dropped);
        if total == 0 {
            return 0.0;
        }
        // u64→f64: packet counters fit exactly in f64 mantissa for any realistic traffic volume
        #[allow(
            clippy::as_conversions,
            reason = "u64 packet counts fit exactly in f64 mantissa for any realistic traffic volume"
        )]
        let dropped = self.packets_dropped as f64;
        #[allow(
            clippy::as_conversions,
            reason = "u64 packet counts fit exactly in f64 mantissa for any realistic traffic volume"
        )]
        let total_f = total as f64;
        dropped / total_f
    }

    /// Record bytes sent.
    pub fn record_sent(&mut self, bytes: u64) {
        self.bytes_sent = self.bytes_sent.saturating_add(bytes);
        self.packets_sent = self.packets_sent.saturating_add(1);
    }

    /// Record bytes received.
    pub fn record_received(&mut self, bytes: u64) {
        self.bytes_received = self.bytes_received.saturating_add(bytes);
        self.packets_received = self.packets_received.saturating_add(1);
    }

    /// Record a dropped packet.
    pub fn record_dropped(&mut self) {
        self.packets_dropped = self.packets_dropped.saturating_add(1);
    }

    /// Record an error.
    pub fn record_error(&mut self) {
        self.errors = self.errors.saturating_add(1);
    }

    /// Reset all counters.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Human-readable byte count.
    pub fn bytes_sent_human(&self) -> String {
        format_bytes(self.bytes_sent)
    }

    /// Human-readable byte count.
    pub fn bytes_received_human(&self) -> String {
        format_bytes(self.bytes_received)
    }
}

/// Format bytes into human-readable form.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    // u64→f64: these are display-only divisions; precision loss beyond 2^53 is acceptable
    #[allow(
        clippy::as_conversions,
        reason = "display-only conversion; precision loss beyond 2^53 bytes (~8 exabytes) is acceptable for human-readable formatting"
    )]
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// A latency measurement sample.
///
/// Tier: T2-P (N Quantity + ν Frequency)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySample {
    /// Round-trip time in microseconds.
    pub rtt_us: u64,
    /// When this measurement was taken.
    pub timestamp: DateTime,
}

/// Connection quality assessment.
///
/// Tier: T2-C (N + κ — quantified comparison)
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConnectionQuality {
    /// Connection is unusable.
    Unusable = 0,
    /// Very poor quality.
    Poor = 1,
    /// Acceptable but degraded.
    Fair = 2,
    /// Normal quality.
    Good = 3,
    /// Excellent quality.
    Excellent = 4,
}

impl ConnectionQuality {
    /// Compute quality from latency and packet loss.
    pub fn from_metrics(latency_ms: u64, packet_loss_pct: f64) -> Self {
        if packet_loss_pct > 10.0 || latency_ms > 1000 {
            Self::Unusable
        } else if packet_loss_pct > 5.0 || latency_ms > 500 {
            Self::Poor
        } else if packet_loss_pct > 2.0 || latency_ms > 200 {
            Self::Fair
        } else if packet_loss_pct > 0.5 || latency_ms > 50 {
            Self::Good
        } else {
            Self::Excellent
        }
    }

    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Unusable => "Unusable",
            Self::Poor => "Poor",
            Self::Fair => "Fair",
            Self::Good => "Good",
            Self::Excellent => "Excellent",
        }
    }
}

/// Per-interface traffic monitor.
///
/// Tier: T2-C (N + ν + σ — quantified periodic measurement sequence)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMonitor {
    /// Interface being monitored.
    pub interface_id: InterfaceId,
    /// Traffic counters.
    pub counters: TrafficCounters,
    /// Recent latency samples (ring buffer, newest last).
    pub latency_samples: Vec<LatencySample>,
    /// Maximum latency samples to keep.
    pub max_samples: usize,
    /// When monitoring started.
    pub started_at: DateTime,
}

impl InterfaceMonitor {
    /// Create a new monitor for an interface.
    pub fn new(interface_id: InterfaceId) -> Self {
        Self {
            interface_id,
            counters: TrafficCounters::default(),
            latency_samples: Vec::new(),
            max_samples: 100,
            started_at: DateTime::now(),
        }
    }

    /// Record a latency measurement.
    pub fn record_latency(&mut self, rtt_us: u64) {
        if self.latency_samples.len() >= self.max_samples {
            self.latency_samples.remove(0);
        }
        self.latency_samples.push(LatencySample {
            rtt_us,
            timestamp: DateTime::now(),
        });
    }

    /// Average latency in microseconds.
    pub fn avg_latency_us(&self) -> Option<u64> {
        if self.latency_samples.is_empty() {
            return None;
        }
        let sum: u64 = self.latency_samples.iter().map(|s| s.rtt_us).sum();
        // usize→u64: sample count is bounded by max_samples (100 by default), well within u64
        let count = u64::try_from(self.latency_samples.len())
            .unwrap_or(1)
            .max(1);
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "count is derived from try_from with a minimum of 1, so division is always safe — no division by zero, no overflow"
        )]
        Some(sum / count)
    }

    /// Average latency in milliseconds.
    pub fn avg_latency_ms(&self) -> Option<u64> {
        self.avg_latency_us().map(|us| us / 1000)
    }

    /// Min latency in microseconds.
    pub fn min_latency_us(&self) -> Option<u64> {
        self.latency_samples.iter().map(|s| s.rtt_us).min()
    }

    /// Max latency in microseconds.
    pub fn max_latency_us(&self) -> Option<u64> {
        self.latency_samples.iter().map(|s| s.rtt_us).max()
    }

    /// Current connection quality assessment.
    pub fn quality(&self) -> ConnectionQuality {
        let latency_ms = self.avg_latency_ms().unwrap_or(0);
        let loss = self.counters.packet_loss_rate() * 100.0;
        ConnectionQuality::from_metrics(latency_ms, loss)
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let latency = self
            .avg_latency_ms()
            .map_or("N/A".to_string(), |ms| format!("{ms}ms"));
        format!(
            "{}: ↑{} ↓{} latency={} quality={}",
            self.interface_id.as_str(),
            self.counters.bytes_sent_human(),
            self.counters.bytes_received_human(),
            latency,
            self.quality().label(),
        )
    }
}

/// System-wide network monitor aggregating all interfaces.
///
/// Tier: T3 (Σ Sum — aggregation of all interface monitors)
#[derive(Debug, Default)]
pub struct NetworkMonitor {
    /// Per-interface monitors.
    monitors: Vec<InterfaceMonitor>,
}

impl NetworkMonitor {
    /// Create a new system-wide monitor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an interface to monitor.
    pub fn add_interface(&mut self, interface_id: InterfaceId) {
        if !self.monitors.iter().any(|m| m.interface_id == interface_id) {
            self.monitors.push(InterfaceMonitor::new(interface_id));
        }
    }

    /// Remove an interface.
    pub fn remove_interface(&mut self, interface_id: &InterfaceId) {
        self.monitors.retain(|m| &m.interface_id != interface_id);
    }

    /// Get a specific interface monitor.
    pub fn get(&self, interface_id: &InterfaceId) -> Option<&InterfaceMonitor> {
        self.monitors
            .iter()
            .find(|m| &m.interface_id == interface_id)
    }

    /// Get a mutable interface monitor.
    pub fn get_mut(&mut self, interface_id: &InterfaceId) -> Option<&mut InterfaceMonitor> {
        self.monitors
            .iter_mut()
            .find(|m| &m.interface_id == interface_id)
    }

    /// Total bytes across all interfaces.
    pub fn total_bytes(&self) -> u64 {
        self.monitors.iter().map(|m| m.counters.total_bytes()).sum()
    }

    /// Total bytes sent across all interfaces.
    pub fn total_bytes_sent(&self) -> u64 {
        self.monitors.iter().map(|m| m.counters.bytes_sent).sum()
    }

    /// Total bytes received across all interfaces.
    pub fn total_bytes_received(&self) -> u64 {
        self.monitors
            .iter()
            .map(|m| m.counters.bytes_received)
            .sum()
    }

    /// Number of monitored interfaces.
    pub fn interface_count(&self) -> usize {
        self.monitors.len()
    }

    /// Summary of all interfaces.
    pub fn summary(&self) -> String {
        format!(
            "Network: {} interfaces, ↑{} ↓{}",
            self.monitors.len(),
            format_bytes(self.total_bytes_sent()),
            format_bytes(self.total_bytes_received()),
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
    fn traffic_counters_default() {
        let c = TrafficCounters::default();
        assert_eq!(c.total_bytes(), 0);
        assert_eq!(c.total_packets(), 0);
        assert!((c.packet_loss_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn traffic_counters_record() {
        let mut c = TrafficCounters::default();
        c.record_sent(1000);
        c.record_received(2000);
        assert_eq!(c.bytes_sent, 1000);
        assert_eq!(c.bytes_received, 2000);
        assert_eq!(c.packets_sent, 1);
        assert_eq!(c.packets_received, 1);
        assert_eq!(c.total_bytes(), 3000);
    }

    #[test]
    fn traffic_counters_packet_loss() {
        let mut c = TrafficCounters::default();
        for _ in 0..9 {
            c.record_sent(100);
        }
        c.record_dropped();
        // 9 sent + 1 dropped = 10 total attempts, 1 lost = 10%
        let rate = c.packet_loss_rate();
        assert!((rate - 0.1).abs() < 0.01);
    }

    #[test]
    fn traffic_counters_reset() {
        let mut c = TrafficCounters::default();
        c.record_sent(1000);
        c.record_received(2000);
        c.reset();
        assert_eq!(c.total_bytes(), 0);
    }

    #[test]
    fn format_bytes_display() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.5 KB");
        assert_eq!(format_bytes(1_500_000), "1.4 MB");
        assert_eq!(format_bytes(1_500_000_000), "1.4 GB");
    }

    #[test]
    fn connection_quality_from_metrics() {
        assert_eq!(
            ConnectionQuality::from_metrics(10, 0.0),
            ConnectionQuality::Excellent
        );
        assert_eq!(
            ConnectionQuality::from_metrics(100, 1.0),
            ConnectionQuality::Good
        );
        assert_eq!(
            ConnectionQuality::from_metrics(300, 3.0),
            ConnectionQuality::Fair
        );
        assert_eq!(
            ConnectionQuality::from_metrics(700, 6.0),
            ConnectionQuality::Poor
        );
        assert_eq!(
            ConnectionQuality::from_metrics(2000, 15.0),
            ConnectionQuality::Unusable
        );
    }

    #[test]
    fn connection_quality_ordering() {
        assert!(ConnectionQuality::Excellent > ConnectionQuality::Good);
        assert!(ConnectionQuality::Good > ConnectionQuality::Fair);
        assert!(ConnectionQuality::Fair > ConnectionQuality::Poor);
        assert!(ConnectionQuality::Poor > ConnectionQuality::Unusable);
    }

    #[test]
    fn interface_monitor_latency() {
        let mut m = InterfaceMonitor::new(eth0());
        m.record_latency(10_000); // 10ms
        m.record_latency(20_000); // 20ms
        m.record_latency(30_000); // 30ms

        assert_eq!(m.avg_latency_us(), Some(20_000));
        assert_eq!(m.avg_latency_ms(), Some(20));
        assert_eq!(m.min_latency_us(), Some(10_000));
        assert_eq!(m.max_latency_us(), Some(30_000));
    }

    #[test]
    fn interface_monitor_no_latency() {
        let m = InterfaceMonitor::new(eth0());
        assert!(m.avg_latency_us().is_none());
        assert!(m.avg_latency_ms().is_none());
    }

    #[test]
    fn interface_monitor_quality() {
        let mut m = InterfaceMonitor::new(eth0());
        m.record_latency(5_000); // 5ms = excellent
        assert_eq!(m.quality(), ConnectionQuality::Excellent);
    }

    #[test]
    fn interface_monitor_ring_buffer() {
        let mut m = InterfaceMonitor::new(eth0());
        m.max_samples = 3;
        for i in 0..5 {
            m.record_latency(i * 1000);
        }
        assert_eq!(m.latency_samples.len(), 3);
        // Should have the last 3: 2000, 3000, 4000
        assert_eq!(m.latency_samples[0].rtt_us, 2000);
    }

    #[test]
    fn network_monitor_add_remove() {
        let mut nm = NetworkMonitor::new();
        nm.add_interface(eth0());
        nm.add_interface(wlan0());
        assert_eq!(nm.interface_count(), 2);

        nm.remove_interface(&eth0());
        assert_eq!(nm.interface_count(), 1);
    }

    #[test]
    fn network_monitor_no_duplicates() {
        let mut nm = NetworkMonitor::new();
        nm.add_interface(eth0());
        nm.add_interface(eth0()); // duplicate
        assert_eq!(nm.interface_count(), 1);
    }

    #[test]
    fn network_monitor_total_bytes() {
        let mut nm = NetworkMonitor::new();
        nm.add_interface(eth0());
        nm.add_interface(wlan0());

        if let Some(m) = nm.get_mut(&eth0()) {
            m.counters.record_sent(1000);
            m.counters.record_received(2000);
        }
        if let Some(m) = nm.get_mut(&wlan0()) {
            m.counters.record_sent(500);
            m.counters.record_received(1500);
        }

        assert_eq!(nm.total_bytes_sent(), 1500);
        assert_eq!(nm.total_bytes_received(), 3500);
        assert_eq!(nm.total_bytes(), 5000);
    }

    #[test]
    fn network_monitor_summary() {
        let nm = NetworkMonitor::new();
        let s = nm.summary();
        assert!(s.contains("Network"));
        assert!(s.contains("0 interfaces"));
    }

    #[test]
    fn interface_monitor_summary() {
        let m = InterfaceMonitor::new(eth0());
        let s = m.summary();
        assert!(s.contains("eth0"));
    }
}
