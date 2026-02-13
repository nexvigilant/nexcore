//! TLS/Security layer for mesh peer-to-peer links.
//!
//! ## Primitive Foundation
//! - `TlsTier`: T1, ς (State) — each tier is a distinct security state
//! - `PeerIdentity`: T2-P, λ (Location) + ∂ (Boundary) — who and where
//! - `SecurityPolicy`: T2-C, ∂ (Boundary) + κ (Comparison) + ς (State) + → (Causality)
//! - `SecurityGate`: T3, evaluates peer against policy to produce verdict
//!
//! ## Design
//!
//! Models TLS tiers, peer certificates, and policy validation without requiring
//! actual TCP transport. Maps to `nexcore-clearance::ClassificationLevel` for
//! consistent security classification across the platform.

use nexcore_clearance::ClassificationLevel;
use serde::{Deserialize, Serialize};

// ============================================================================
// TlsTier — Security classification for mesh links
// ============================================================================

/// Security tier for mesh peer-to-peer links.
///
/// Maps 1:1 to `ClassificationLevel` from nexcore-clearance,
/// providing mesh-specific semantics for each tier.
///
/// Tier: T1 | Dominant: ς (State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TlsTier {
    /// No encryption — plaintext mesh links (development only).
    Plaintext,
    /// Basic TLS — encrypted but no mutual authentication.
    Basic,
    /// Standard TLS — encrypted with server certificate validation.
    Standard,
    /// Mutual TLS — both peers present certificates.
    Mutual,
    /// Maximum security — mTLS with certificate pinning.
    Pinned,
}

impl TlsTier {
    /// Convert to the corresponding classification level.
    pub fn to_classification(self) -> ClassificationLevel {
        match self {
            Self::Plaintext => ClassificationLevel::Public,
            Self::Basic => ClassificationLevel::Internal,
            Self::Standard => ClassificationLevel::Confidential,
            Self::Mutual => ClassificationLevel::Secret,
            Self::Pinned => ClassificationLevel::TopSecret,
        }
    }

    /// Create from a classification level.
    pub fn from_classification(level: ClassificationLevel) -> Self {
        match level {
            ClassificationLevel::Public => Self::Plaintext,
            ClassificationLevel::Internal => Self::Basic,
            ClassificationLevel::Confidential => Self::Standard,
            ClassificationLevel::Secret => Self::Mutual,
            ClassificationLevel::TopSecret => Self::Pinned,
        }
    }

    /// Ordinal for comparison (0 = least secure, 4 = most secure).
    pub fn ordinal(self) -> u8 {
        self.to_classification().ordinal()
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Plaintext => "Plaintext",
            Self::Basic => "Basic TLS",
            Self::Standard => "Standard TLS",
            Self::Mutual => "Mutual TLS",
            Self::Pinned => "Pinned mTLS",
        }
    }

    /// Whether this tier provides encryption.
    pub fn is_encrypted(self) -> bool {
        !matches!(self, Self::Plaintext)
    }

    /// Whether this tier requires mutual authentication.
    pub fn requires_mutual_auth(self) -> bool {
        matches!(self, Self::Mutual | Self::Pinned)
    }
}

impl PartialOrd for TlsTier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TlsTier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

impl std::fmt::Display for TlsTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ============================================================================
// CertValidationMode — How to validate peer certificates
// ============================================================================

/// Certificate validation strategy.
///
/// Tier: T1 | Dominant: κ (Comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CertValidationMode {
    /// No certificate validation (development only).
    None,
    /// Accept self-signed certificates.
    SelfSigned,
    /// Require CA-signed certificates.
    CaSigned,
}

// ============================================================================
// PeerIdentity — Identity of a mesh peer
// ============================================================================

/// Identity information for a mesh peer.
///
/// Tier: T2-P | Dominant: λ (Location)
///
/// Contains the peer's node ID, public key fingerprint, and TLS tier.
/// Used by `SecurityGate` to evaluate access decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerIdentity {
    /// Node ID of the peer.
    pub node_id: String,
    /// SHA-256 fingerprint of the peer's public key (hex-encoded).
    /// Empty string means no certificate presented.
    pub fingerprint: String,
    /// The TLS tier the peer is operating at.
    pub tier: TlsTier,
    /// Whether the peer presented a valid certificate.
    pub has_certificate: bool,
}

impl PeerIdentity {
    /// Create a new peer identity.
    pub fn new(node_id: impl Into<String>, tier: TlsTier) -> Self {
        Self {
            node_id: node_id.into(),
            fingerprint: String::new(),
            tier,
            has_certificate: false,
        }
    }

    /// Set the certificate fingerprint.
    pub fn with_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        let fp = fingerprint.into();
        self.has_certificate = !fp.is_empty();
        self.fingerprint = fp;
        self
    }
}

// ============================================================================
// SecurityPolicy — Policy for a mesh node
// ============================================================================

/// Security policy governing which peers can connect.
///
/// Tier: T2-C | Dominant: ∂ (Boundary)
///
/// Defines the minimum TLS tier, certificate validation mode,
/// and whether mutual authentication is required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Minimum TLS tier required for peer connections.
    pub min_tier: TlsTier,
    /// Certificate validation mode.
    pub cert_validation: CertValidationMode,
    /// Whether mutual authentication is required.
    pub mutual_auth_required: bool,
}

impl SecurityPolicy {
    /// Create a new security policy.
    pub fn new(min_tier: TlsTier, cert_validation: CertValidationMode) -> Self {
        Self {
            min_tier,
            cert_validation,
            mutual_auth_required: false,
        }
    }

    /// Require mutual authentication.
    pub fn with_mutual_auth(mut self) -> Self {
        self.mutual_auth_required = true;
        self
    }

    /// Default permissive policy (any tier, no validation).
    pub fn permissive() -> Self {
        Self::new(TlsTier::Plaintext, CertValidationMode::None)
    }

    /// Strict policy (mutual TLS, CA-signed certs).
    pub fn strict() -> Self {
        Self::new(TlsTier::Mutual, CertValidationMode::CaSigned).with_mutual_auth()
    }

    /// Standard policy (standard TLS, self-signed OK).
    pub fn standard() -> Self {
        Self::new(TlsTier::Standard, CertValidationMode::SelfSigned)
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

// ============================================================================
// SecurityVerdict — Outcome of security evaluation
// ============================================================================

/// Result of evaluating a peer identity against a security policy.
///
/// Tier: T2-P | Dominant: → (Causality)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityVerdict {
    /// Peer is allowed to connect.
    Allow,
    /// Peer is denied. Includes reason.
    Deny(String),
    /// Peer should upgrade their security tier to connect.
    Upgrade {
        /// Current peer tier.
        current: TlsTier,
        /// Required minimum tier.
        required: TlsTier,
    },
}

impl SecurityVerdict {
    /// Whether access is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Whether access is denied.
    pub fn is_denied(&self) -> bool {
        !self.is_allowed()
    }
}

// ============================================================================
// SecurityGate — Evaluates peers against policy
// ============================================================================

/// Evaluates peer identities against a security policy.
///
/// Tier: T3 | Dominant: ∂ (Boundary)
///
/// Stateless evaluator: takes a peer and a policy, returns a verdict.
/// Can be used at connection time to gate peer access.
#[derive(Debug, Clone)]
pub struct SecurityGate;

impl SecurityGate {
    /// Evaluate a peer identity against a security policy.
    pub fn evaluate(peer: &PeerIdentity, policy: &SecurityPolicy) -> SecurityVerdict {
        // Check tier requirement
        if peer.tier < policy.min_tier {
            return SecurityVerdict::Upgrade {
                current: peer.tier,
                required: policy.min_tier,
            };
        }

        // Check mutual auth requirement
        if policy.mutual_auth_required && !peer.has_certificate {
            return SecurityVerdict::Deny(
                "mutual authentication required but peer has no certificate".to_string(),
            );
        }

        // Check cert validation mode
        match policy.cert_validation {
            CertValidationMode::None => {} // no check needed
            CertValidationMode::SelfSigned | CertValidationMode::CaSigned => {
                // If cert validation is required and tier demands it, check fingerprint
                if peer.tier >= TlsTier::Standard && peer.fingerprint.is_empty() {
                    return SecurityVerdict::Deny(
                        "certificate required at this tier but none presented".to_string(),
                    );
                }
            }
        }

        SecurityVerdict::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- TlsTier tests ----------

    #[test]
    fn tls_tier_ordering() {
        assert!(TlsTier::Pinned > TlsTier::Mutual);
        assert!(TlsTier::Mutual > TlsTier::Standard);
        assert!(TlsTier::Standard > TlsTier::Basic);
        assert!(TlsTier::Basic > TlsTier::Plaintext);
    }

    #[test]
    fn tls_tier_classification_roundtrip() {
        for tier in [
            TlsTier::Plaintext,
            TlsTier::Basic,
            TlsTier::Standard,
            TlsTier::Mutual,
            TlsTier::Pinned,
        ] {
            let cl = tier.to_classification();
            let back = TlsTier::from_classification(cl);
            assert_eq!(tier, back);
        }
    }

    #[test]
    fn tls_tier_is_encrypted() {
        assert!(!TlsTier::Plaintext.is_encrypted());
        assert!(TlsTier::Basic.is_encrypted());
        assert!(TlsTier::Standard.is_encrypted());
        assert!(TlsTier::Mutual.is_encrypted());
        assert!(TlsTier::Pinned.is_encrypted());
    }

    #[test]
    fn tls_tier_requires_mutual_auth() {
        assert!(!TlsTier::Plaintext.requires_mutual_auth());
        assert!(!TlsTier::Basic.requires_mutual_auth());
        assert!(!TlsTier::Standard.requires_mutual_auth());
        assert!(TlsTier::Mutual.requires_mutual_auth());
        assert!(TlsTier::Pinned.requires_mutual_auth());
    }

    #[test]
    fn tls_tier_display() {
        assert_eq!(TlsTier::Plaintext.to_string(), "Plaintext");
        assert_eq!(TlsTier::Pinned.to_string(), "Pinned mTLS");
    }

    // ---------- PeerIdentity tests ----------

    #[test]
    fn peer_identity_new_no_cert() {
        let peer = PeerIdentity::new("node-1", TlsTier::Basic);
        assert_eq!(peer.node_id, "node-1");
        assert_eq!(peer.tier, TlsTier::Basic);
        assert!(!peer.has_certificate);
        assert!(peer.fingerprint.is_empty());
    }

    #[test]
    fn peer_identity_with_fingerprint() {
        let peer =
            PeerIdentity::new("node-2", TlsTier::Mutual).with_fingerprint("sha256:abc123def456");
        assert!(peer.has_certificate);
        assert_eq!(peer.fingerprint, "sha256:abc123def456");
    }

    // ---------- SecurityPolicy tests ----------

    #[test]
    fn security_policy_permissive() {
        let p = SecurityPolicy::permissive();
        assert_eq!(p.min_tier, TlsTier::Plaintext);
        assert_eq!(p.cert_validation, CertValidationMode::None);
        assert!(!p.mutual_auth_required);
    }

    #[test]
    fn security_policy_strict() {
        let p = SecurityPolicy::strict();
        assert_eq!(p.min_tier, TlsTier::Mutual);
        assert_eq!(p.cert_validation, CertValidationMode::CaSigned);
        assert!(p.mutual_auth_required);
    }

    #[test]
    fn security_policy_default_is_standard() {
        let p = SecurityPolicy::default();
        assert_eq!(p.min_tier, TlsTier::Standard);
    }

    // ---------- SecurityGate tests ----------

    #[test]
    fn gate_allow_public_peer_permissive_policy() {
        let peer = PeerIdentity::new("public-node", TlsTier::Plaintext);
        let policy = SecurityPolicy::permissive();
        let verdict = SecurityGate::evaluate(&peer, &policy);
        assert!(verdict.is_allowed());
    }

    #[test]
    fn gate_deny_low_tier_peer() {
        let peer = PeerIdentity::new("basic-node", TlsTier::Basic);
        let policy = SecurityPolicy::new(TlsTier::Mutual, CertValidationMode::None);
        let verdict = SecurityGate::evaluate(&peer, &policy);
        assert!(verdict.is_denied());
        assert!(matches!(verdict, SecurityVerdict::Upgrade { .. }));
    }

    #[test]
    fn gate_upgrade_contains_tiers() {
        let peer = PeerIdentity::new("n", TlsTier::Standard);
        let policy = SecurityPolicy::strict();
        let verdict = SecurityGate::evaluate(&peer, &policy);
        if let SecurityVerdict::Upgrade { current, required } = verdict {
            assert_eq!(current, TlsTier::Standard);
            assert_eq!(required, TlsTier::Mutual);
        } else {
            panic!("expected Upgrade verdict");
        }
    }

    #[test]
    fn gate_deny_mutual_auth_no_cert() {
        let peer = PeerIdentity::new("no-cert-node", TlsTier::Mutual);
        let policy = SecurityPolicy::strict();
        let verdict = SecurityGate::evaluate(&peer, &policy);
        assert!(verdict.is_denied());
        assert!(matches!(verdict, SecurityVerdict::Deny(_)));
    }

    #[test]
    fn gate_allow_mutual_auth_with_cert() {
        let peer = PeerIdentity::new("cert-node", TlsTier::Mutual)
            .with_fingerprint("sha256:valid-fingerprint");
        let policy = SecurityPolicy::strict();
        let verdict = SecurityGate::evaluate(&peer, &policy);
        assert!(verdict.is_allowed());
    }

    #[test]
    fn gate_deny_standard_tier_no_cert_when_validation_required() {
        let peer = PeerIdentity::new("std-no-cert", TlsTier::Standard);
        let policy = SecurityPolicy::standard();
        let verdict = SecurityGate::evaluate(&peer, &policy);
        assert!(verdict.is_denied());
    }

    #[test]
    fn gate_allow_standard_with_cert() {
        let peer = PeerIdentity::new("std-cert", TlsTier::Standard)
            .with_fingerprint("sha256:some-fingerprint");
        let policy = SecurityPolicy::standard();
        let verdict = SecurityGate::evaluate(&peer, &policy);
        assert!(verdict.is_allowed());
    }

    #[test]
    fn verdict_serialization_roundtrip() {
        let v = SecurityVerdict::Upgrade {
            current: TlsTier::Basic,
            required: TlsTier::Standard,
        };
        let json = serde_json::to_string(&v);
        assert!(json.is_ok());
        if let Ok(json) = json {
            let parsed: Result<SecurityVerdict, _> = serde_json::from_str(&json);
            assert!(parsed.is_ok());
            if let Ok(parsed) = parsed {
                assert_eq!(parsed, v);
            }
        }
    }

    #[test]
    fn tier_transitivity() {
        // If a >= b and b >= c then a >= c
        let tiers = [
            TlsTier::Plaintext,
            TlsTier::Basic,
            TlsTier::Standard,
            TlsTier::Mutual,
            TlsTier::Pinned,
        ];
        for (i, a) in tiers.iter().enumerate() {
            for b in tiers.iter().skip(i) {
                assert!(*b >= *a, "{:?} should be >= {:?}", b, a);
            }
        }
    }
}
