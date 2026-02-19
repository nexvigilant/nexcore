// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! TLS certificate management — trust store for secure connections.
//!
//! Tier: T2-C (∂ Boundary + π Persistence + ∝ Irreversibility)
//!
//! Certificates define trust boundaries (∂) that persist (π) and,
//! once revoked, are irreversibly (∝) invalidated. This module manages
//! the OS-level certificate store used by all secure network connections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Certificate trust level.
///
/// Tier: T2-P (κ Comparison — ordered trust)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Certificate is explicitly distrusted.
    Distrusted = 0,
    /// Trust is unknown (not yet evaluated).
    Unknown = 1,
    /// User-added certificate (trusted by user).
    UserTrusted = 2,
    /// System certificate (pre-installed in OS).
    SystemTrusted = 3,
}

impl TrustLevel {
    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Distrusted => "Distrusted",
            Self::Unknown => "Unknown",
            Self::UserTrusted => "User Trusted",
            Self::SystemTrusted => "System Trusted",
        }
    }

    /// Whether this level allows secure connections.
    pub const fn allows_connection(&self) -> bool {
        matches!(self, Self::UserTrusted | Self::SystemTrusted)
    }
}

/// Certificate status.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertStatus {
    /// Certificate is valid and active.
    Valid,
    /// Certificate has expired (past `not_after`).
    Expired,
    /// Certificate has been revoked (irreversible).
    Revoked,
    /// Certificate is not yet valid (before `not_before`).
    NotYetValid,
}

impl CertStatus {
    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Valid => "Valid",
            Self::Expired => "Expired",
            Self::Revoked => "Revoked",
            Self::NotYetValid => "Not Yet Valid",
        }
    }

    /// Whether this status permits use.
    pub const fn is_usable(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// A certificate fingerprint (SHA-256 of DER encoding).
///
/// Tier: T2-P (∃ Existence — unique identity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertFingerprint(String);

impl CertFingerprint {
    /// Create a fingerprint from a hex string.
    pub fn new(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// Get the hex string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A certificate entry in the trust store.
///
/// Tier: T2-C (∂ + π + ∝ — persistent trust boundary with revocation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate fingerprint (SHA-256).
    pub fingerprint: CertFingerprint,
    /// Subject common name (e.g., "*.example.com").
    pub subject: String,
    /// Issuer common name.
    pub issuer: String,
    /// Not valid before this date.
    pub not_before: DateTime<Utc>,
    /// Not valid after this date.
    pub not_after: DateTime<Utc>,
    /// Trust level.
    pub trust_level: TrustLevel,
    /// Current status.
    pub status: CertStatus,
    /// When this certificate was added to the store.
    pub added_at: DateTime<Utc>,
    /// Optional revocation reason.
    pub revocation_reason: Option<String>,
}

impl Certificate {
    /// Create a new valid certificate.
    pub fn new(
        fingerprint: impl Into<String>,
        subject: impl Into<String>,
        issuer: impl Into<String>,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        trust_level: TrustLevel,
    ) -> Self {
        Self {
            fingerprint: CertFingerprint::new(fingerprint),
            subject: subject.into(),
            issuer: issuer.into(),
            not_before,
            not_after,
            trust_level,
            status: CertStatus::Valid,
            added_at: Utc::now(),
            revocation_reason: None,
        }
    }

    /// Check if the certificate is currently valid (time-wise).
    pub fn is_time_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.not_before && now <= self.not_after
    }

    /// Check if the certificate is usable (valid + trusted).
    pub fn is_usable(&self) -> bool {
        self.status.is_usable() && self.is_time_valid() && self.trust_level.allows_connection()
    }

    /// Revoke this certificate (irreversible ∝).
    pub fn revoke(&mut self, reason: impl Into<String>) {
        self.status = CertStatus::Revoked;
        self.revocation_reason = Some(reason.into());
    }

    /// Update status based on current time.
    pub fn refresh_status(&mut self) {
        if self.status == CertStatus::Revoked {
            return; // Revocation is irreversible
        }
        let now = Utc::now();
        if now < self.not_before {
            self.status = CertStatus::NotYetValid;
        } else if now > self.not_after {
            self.status = CertStatus::Expired;
        } else {
            self.status = CertStatus::Valid;
        }
    }

    /// Days until expiration (negative if expired).
    pub fn days_until_expiry(&self) -> i64 {
        (self.not_after - Utc::now()).num_days()
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        format!(
            "{} [{}] {} (expires in {} days)",
            self.subject,
            self.trust_level.label(),
            self.status.label(),
            self.days_until_expiry(),
        )
    }
}

/// Certificate store — the OS trust store.
///
/// Tier: T3 (π + ∂ + ∝ — persistent trust boundaries with irreversible revocation)
#[derive(Debug, Default)]
pub struct CertStore {
    /// Certificates keyed by fingerprint.
    certs: HashMap<CertFingerprint, Certificate>,
}

impl CertStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a certificate to the store.
    pub fn add(&mut self, cert: Certificate) {
        self.certs.insert(cert.fingerprint.clone(), cert);
    }

    /// Look up a certificate by fingerprint.
    pub fn get(&self, fingerprint: &CertFingerprint) -> Option<&Certificate> {
        self.certs.get(fingerprint)
    }

    /// Look up a certificate by subject name.
    pub fn find_by_subject(&self, subject: &str) -> Vec<&Certificate> {
        self.certs
            .values()
            .filter(|c| c.subject == subject || c.subject.ends_with(subject))
            .collect()
    }

    /// Check if a subject has a valid, trusted certificate.
    pub fn is_trusted(&self, subject: &str) -> bool {
        self.find_by_subject(subject).iter().any(|c| c.is_usable())
    }

    /// Revoke a certificate by fingerprint.
    pub fn revoke(&mut self, fingerprint: &CertFingerprint, reason: impl Into<String>) -> bool {
        if let Some(cert) = self.certs.get_mut(fingerprint) {
            cert.revoke(reason);
            true
        } else {
            false
        }
    }

    /// Refresh status of all certificates.
    pub fn refresh_all(&mut self) {
        for cert in self.certs.values_mut() {
            cert.refresh_status();
        }
    }

    /// Get all expired certificates.
    pub fn expired(&self) -> Vec<&Certificate> {
        self.certs
            .values()
            .filter(|c| c.status == CertStatus::Expired)
            .collect()
    }

    /// Get all revoked certificates.
    pub fn revoked(&self) -> Vec<&Certificate> {
        self.certs
            .values()
            .filter(|c| c.status == CertStatus::Revoked)
            .collect()
    }

    /// Get certificates expiring within N days.
    pub fn expiring_within_days(&self, days: i64) -> Vec<&Certificate> {
        self.certs
            .values()
            .filter(|c| c.status == CertStatus::Valid && c.days_until_expiry() <= days)
            .collect()
    }

    /// Total certificates in the store.
    pub fn len(&self) -> usize {
        self.certs.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.certs.is_empty()
    }

    /// Count by trust level.
    pub fn count_by_trust(&self) -> (usize, usize, usize, usize) {
        let mut system = 0;
        let mut user = 0;
        let mut unknown = 0;
        let mut distrusted = 0;
        for cert in self.certs.values() {
            match cert.trust_level {
                TrustLevel::SystemTrusted => system += 1,
                TrustLevel::UserTrusted => user += 1,
                TrustLevel::Unknown => unknown += 1,
                TrustLevel::Distrusted => distrusted += 1,
            }
        }
        (system, user, unknown, distrusted)
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let (system, user, unknown, distrusted) = self.count_by_trust();
        format!(
            "Certs: {} total (system={}, user={}, unknown={}, distrusted={})",
            self.certs.len(),
            system,
            user,
            unknown,
            distrusted,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_cert(subject: &str, trust: TrustLevel) -> Certificate {
        Certificate::new(
            format!("fp_{subject}"),
            subject,
            "Test CA",
            Utc::now() - Duration::days(30),
            Utc::now() + Duration::days(335),
            trust,
        )
    }

    fn make_expired_cert(subject: &str) -> Certificate {
        Certificate::new(
            format!("fp_{subject}"),
            subject,
            "Test CA",
            Utc::now() - Duration::days(400),
            Utc::now() - Duration::days(35),
            TrustLevel::SystemTrusted,
        )
    }

    #[test]
    fn trust_level_ordering() {
        assert!(TrustLevel::SystemTrusted > TrustLevel::UserTrusted);
        assert!(TrustLevel::UserTrusted > TrustLevel::Unknown);
        assert!(TrustLevel::Unknown > TrustLevel::Distrusted);
    }

    #[test]
    fn trust_level_allows_connection() {
        assert!(TrustLevel::SystemTrusted.allows_connection());
        assert!(TrustLevel::UserTrusted.allows_connection());
        assert!(!TrustLevel::Unknown.allows_connection());
        assert!(!TrustLevel::Distrusted.allows_connection());
    }

    #[test]
    fn cert_valid_and_usable() {
        let cert = make_cert("example.com", TrustLevel::SystemTrusted);
        assert!(cert.is_time_valid());
        assert!(cert.is_usable());
        assert!(cert.days_until_expiry() > 300);
    }

    #[test]
    fn cert_expired_not_usable() {
        let mut cert = make_expired_cert("old.com");
        cert.refresh_status();
        assert_eq!(cert.status, CertStatus::Expired);
        assert!(!cert.is_usable());
    }

    #[test]
    fn cert_revocation_irreversible() {
        let mut cert = make_cert("revoked.com", TrustLevel::SystemTrusted);
        cert.revoke("compromised key");
        assert_eq!(cert.status, CertStatus::Revoked);
        assert!(!cert.is_usable());
        assert!(cert.revocation_reason.is_some());

        // Try to refresh — should stay revoked (∝ irreversibility)
        cert.refresh_status();
        assert_eq!(cert.status, CertStatus::Revoked);
    }

    #[test]
    fn cert_untrusted_not_usable() {
        let cert = make_cert("untrusted.com", TrustLevel::Unknown);
        assert!(cert.is_time_valid()); // valid time-wise
        assert!(!cert.is_usable()); // but not trusted
    }

    #[test]
    fn store_add_and_get() {
        let mut store = CertStore::new();
        let cert = make_cert("test.com", TrustLevel::SystemTrusted);
        let fp = cert.fingerprint.clone();
        store.add(cert);

        assert_eq!(store.len(), 1);
        assert!(store.get(&fp).is_some());
    }

    #[test]
    fn store_find_by_subject() {
        let mut store = CertStore::new();
        store.add(make_cert("example.com", TrustLevel::SystemTrusted));
        store.add(make_cert("other.com", TrustLevel::UserTrusted));

        let found = store.find_by_subject("example.com");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn store_is_trusted() {
        let mut store = CertStore::new();
        store.add(make_cert("trusted.com", TrustLevel::SystemTrusted));
        store.add(make_cert("untrusted.com", TrustLevel::Unknown));

        assert!(store.is_trusted("trusted.com"));
        assert!(!store.is_trusted("untrusted.com"));
        assert!(!store.is_trusted("missing.com"));
    }

    #[test]
    fn store_revoke() {
        let mut store = CertStore::new();
        let cert = make_cert("bad.com", TrustLevel::SystemTrusted);
        let fp = cert.fingerprint.clone();
        store.add(cert);

        assert!(store.revoke(&fp, "compromised"));
        assert!(!store.is_trusted("bad.com"));
        assert_eq!(store.revoked().len(), 1);
    }

    #[test]
    fn store_expired() {
        let mut store = CertStore::new();
        let mut cert = make_expired_cert("old.com");
        cert.refresh_status();
        store.add(cert);

        assert_eq!(store.expired().len(), 1);
    }

    #[test]
    fn store_expiring_soon() {
        let mut store = CertStore::new();
        // Cert expiring in 10 days
        let cert = Certificate::new(
            "fp_soon",
            "soon.com",
            "CA",
            Utc::now() - Duration::days(355),
            Utc::now() + Duration::days(10),
            TrustLevel::SystemTrusted,
        );
        store.add(cert);
        store.add(make_cert("far.com", TrustLevel::SystemTrusted)); // 335 days away

        let expiring = store.expiring_within_days(30);
        assert_eq!(expiring.len(), 1);
        assert_eq!(expiring[0].subject, "soon.com");
    }

    #[test]
    fn store_count_by_trust() {
        let mut store = CertStore::new();
        store.add(make_cert("a.com", TrustLevel::SystemTrusted));
        store.add(make_cert("b.com", TrustLevel::SystemTrusted));
        store.add(make_cert("c.com", TrustLevel::UserTrusted));
        store.add(make_cert("d.com", TrustLevel::Unknown));

        let (system, user, unknown, distrusted) = store.count_by_trust();
        assert_eq!(system, 2);
        assert_eq!(user, 1);
        assert_eq!(unknown, 1);
        assert_eq!(distrusted, 0);
    }

    #[test]
    fn store_summary() {
        let store = CertStore::new();
        let s = store.summary();
        assert!(s.contains("Certs"));
        assert!(s.contains("0 total"));
    }

    #[test]
    fn cert_summary() {
        let cert = make_cert("example.com", TrustLevel::SystemTrusted);
        let s = cert.summary();
        assert!(s.contains("example.com"));
        assert!(s.contains("System Trusted"));
        assert!(s.contains("Valid"));
    }

    #[test]
    fn cert_fingerprint() {
        let fp = CertFingerprint::new("abc123");
        assert_eq!(fp.as_str(), "abc123");
    }
}
