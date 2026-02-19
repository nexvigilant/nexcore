// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! DNS resolver with caching.
//!
//! Tier: T2-C (μ Mapping + π Persistence + ν Frequency)
//!
//! DNS is a name→IP mapping (μ) with cached persistence (π) and
//! TTL-based expiry (ν frequency of refresh).

use crate::interface::IpAddr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single DNS record.
///
/// Tier: T2-P (μ Mapping — name to address)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    /// The domain name.
    pub name: String,
    /// Resolved addresses (may have multiple A/AAAA records).
    pub addresses: Vec<IpAddr>,
    /// Time-to-live in seconds.
    pub ttl_seconds: u32,
    /// When this record was cached.
    pub cached_at: DateTime<Utc>,
}

impl DnsRecord {
    /// Create a new DNS record.
    pub fn new(name: impl Into<String>, addresses: Vec<IpAddr>, ttl_seconds: u32) -> Self {
        Self {
            name: name.into(),
            addresses,
            ttl_seconds,
            cached_at: Utc::now(),
        }
    }

    /// Whether this record has expired.
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.cached_at)
            .num_seconds();
        elapsed < 0 || elapsed as u32 >= self.ttl_seconds
    }

    /// Get the first address (most common use case).
    pub fn primary_address(&self) -> Option<&IpAddr> {
        self.addresses.first()
    }

    /// Remaining TTL in seconds (0 if expired).
    pub fn remaining_ttl(&self) -> u32 {
        let elapsed = Utc::now()
            .signed_duration_since(self.cached_at)
            .num_seconds();
        if elapsed < 0 {
            return 0;
        }
        self.ttl_seconds.saturating_sub(elapsed as u32)
    }
}

/// DNS cache statistics.
///
/// Tier: T2-P (N Quantity)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DnsCacheStats {
    /// Total lookups performed.
    pub total_lookups: u64,
    /// Cache hits.
    pub cache_hits: u64,
    /// Cache misses (required resolution).
    pub cache_misses: u64,
    /// Expired entries evicted.
    pub evictions: u64,
    /// Current number of cached entries.
    pub entries: usize,
}

impl DnsCacheStats {
    /// Cache hit rate as a percentage.
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / self.total_lookups as f64 * 100.0
    }
}

/// DNS resolver with in-memory cache.
///
/// Tier: T2-C (μ + π + ν — mapping with persistent cache and TTL frequency)
#[derive(Debug)]
pub struct DnsResolver {
    /// Cached DNS records keyed by domain name.
    cache: HashMap<String, DnsRecord>,
    /// Maximum cache entries.
    max_entries: usize,
    /// Statistics.
    stats: DnsCacheStats,
    /// Upstream DNS server addresses.
    upstream_servers: Vec<IpAddr>,
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsResolver {
    /// Create a new resolver with default settings.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_entries: 1000,
            upstream_servers: vec![
                IpAddr::v4(8, 8, 8, 8), // Google DNS
                IpAddr::v4(1, 1, 1, 1), // Cloudflare DNS
                IpAddr::v4(9, 9, 9, 9), // Quad9 DNS
            ],
            stats: DnsCacheStats::default(),
        }
    }

    /// Create with custom upstream servers.
    pub fn with_upstream(servers: Vec<IpAddr>) -> Self {
        Self {
            upstream_servers: servers,
            ..Self::new()
        }
    }

    /// Set the maximum cache size.
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Look up a domain name in the cache.
    ///
    /// Returns `Some(record)` if cached and not expired, `None` otherwise.
    pub fn lookup_cached(&mut self, name: &str) -> Option<&DnsRecord> {
        self.stats.total_lookups += 1;

        // Check if cached and not expired
        if let Some(record) = self.cache.get(name) {
            if !record.is_expired() {
                self.stats.cache_hits += 1;
                return Some(record);
            }
            // Expired — will be evicted on next cleanup
        }

        self.stats.cache_misses += 1;
        None
    }

    /// Insert a resolved record into the cache.
    pub fn cache_record(&mut self, record: DnsRecord) {
        // Evict expired entries if at capacity
        if self.cache.len() >= self.max_entries {
            self.evict_expired();
        }

        // If still at capacity, evict oldest
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }

        self.cache.insert(record.name.clone(), record);
        self.stats.entries = self.cache.len();
    }

    /// Insert a simple A record (convenience).
    pub fn cache_address(&mut self, name: impl Into<String>, addr: IpAddr, ttl: u32) {
        self.cache_record(DnsRecord::new(name, vec![addr], ttl));
    }

    /// Remove all expired entries.
    pub fn evict_expired(&mut self) {
        let before = self.cache.len();
        self.cache.retain(|_, record| !record.is_expired());
        let evicted = before - self.cache.len();
        self.stats.evictions += evicted as u64;
        self.stats.entries = self.cache.len();
    }

    /// Evict the oldest entry.
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, v)| v.cached_at)
            .map(|(k, _)| k.clone())
        {
            self.cache.remove(&oldest_key);
            self.stats.evictions += 1;
        }
    }

    /// Clear the entire cache.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.stats.entries = 0;
    }

    /// Get cache statistics.
    pub fn stats(&self) -> &DnsCacheStats {
        &self.stats
    }

    /// Get upstream DNS servers.
    pub fn upstream_servers(&self) -> &[IpAddr] {
        &self.upstream_servers
    }

    /// Set upstream DNS servers.
    pub fn set_upstream_servers(&mut self, servers: Vec<IpAddr>) {
        self.upstream_servers = servers;
    }

    /// Number of cached entries.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        format!(
            "DNS: {} entries, {:.1}% hit rate, {} upstream servers",
            self.cache.len(),
            self.stats.hit_rate(),
            self.upstream_servers.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_resolver() {
        let r = DnsResolver::new();
        assert_eq!(r.cache_size(), 0);
        assert_eq!(r.upstream_servers().len(), 3);
    }

    #[test]
    fn cache_and_lookup() {
        let mut r = DnsResolver::new();
        r.cache_address("example.com", IpAddr::v4(93, 184, 216, 34), 300);
        assert_eq!(r.cache_size(), 1);

        let record = r.lookup_cached("example.com");
        assert!(record.is_some());
        if let Some(rec) = record {
            assert_eq!(rec.addresses.len(), 1);
        }
    }

    #[test]
    fn cache_miss() {
        let mut r = DnsResolver::new();
        let result = r.lookup_cached("nonexistent.com");
        assert!(result.is_none());
        assert_eq!(r.stats().cache_misses, 1);
    }

    #[test]
    fn cache_hit_rate() {
        let mut r = DnsResolver::new();
        r.cache_address("test.com", IpAddr::v4(1, 2, 3, 4), 3600);

        // 1 hit
        let _ = r.lookup_cached("test.com");
        // 1 miss
        let _ = r.lookup_cached("other.com");

        assert_eq!(r.stats().total_lookups, 2);
        assert_eq!(r.stats().cache_hits, 1);
        assert_eq!(r.stats().cache_misses, 1);
        let rate = r.stats().hit_rate();
        assert!((rate - 50.0).abs() < 0.1);
    }

    #[test]
    fn dns_record_primary_address() {
        let record = DnsRecord::new(
            "multi.com",
            vec![IpAddr::v4(1, 1, 1, 1), IpAddr::v4(2, 2, 2, 2)],
            60,
        );
        assert_eq!(record.primary_address(), Some(&IpAddr::v4(1, 1, 1, 1)));
    }

    #[test]
    fn dns_record_empty() {
        let record = DnsRecord::new("empty.com", vec![], 60);
        assert!(record.primary_address().is_none());
    }

    #[test]
    fn clear_cache() {
        let mut r = DnsResolver::new();
        r.cache_address("a.com", IpAddr::v4(1, 1, 1, 1), 60);
        r.cache_address("b.com", IpAddr::v4(2, 2, 2, 2), 60);
        assert_eq!(r.cache_size(), 2);
        r.clear();
        assert_eq!(r.cache_size(), 0);
    }

    #[test]
    fn max_entries_eviction() {
        let mut r = DnsResolver::new().with_max_entries(3);
        for i in 0..5 {
            r.cache_address(format!("host{i}.com"), IpAddr::v4(10, 0, 0, i as u8), 3600);
        }
        assert!(r.cache_size() <= 3);
    }

    #[test]
    fn custom_upstream() {
        let r = DnsResolver::with_upstream(vec![IpAddr::v4(10, 0, 0, 1)]);
        assert_eq!(r.upstream_servers().len(), 1);
    }

    #[test]
    fn set_upstream_servers() {
        let mut r = DnsResolver::new();
        r.set_upstream_servers(vec![IpAddr::v4(10, 0, 0, 53)]);
        assert_eq!(r.upstream_servers().len(), 1);
    }

    #[test]
    fn summary_format() {
        let r = DnsResolver::new();
        let s = r.summary();
        assert!(s.contains("DNS"));
        assert!(s.contains("0 entries"));
    }

    #[test]
    fn hit_rate_zero_lookups() {
        let stats = DnsCacheStats::default();
        assert!((stats.hit_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn record_remaining_ttl() {
        let record = DnsRecord::new("test.com", vec![IpAddr::v4(1, 2, 3, 4)], 3600);
        // Just created, should have nearly full TTL
        assert!(record.remaining_ttl() > 3590);
    }
}
