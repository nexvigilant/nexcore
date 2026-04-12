//! Communication security gate — validates security posture before transmission.
//!
//! Maps to the `comms-security-gate` microgram.
//! Gates on mTLS, firmware signature, replay protection, and cert pinning.

use super::link::LinkType;
use serde::{Deserialize, Serialize};

/// Security posture check result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityVerdict {
    /// All required controls active — transmit freely.
    Pass,
    /// Some controls missing — transmit with elevated logging.
    Warn,
    /// Critical controls missing — block transmission.
    Block,
}

/// Security level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Full mTLS with certificate pinning.
    FullMtlsPinned,
    /// mTLS active but certificate not pinned.
    EncryptedUnpinned,
    /// Encrypted payload, no IP-layer auth (LoRa).
    EncryptedNoMtls,
    /// Replay protection missing.
    EncryptedReplayRisk,
    /// No encryption — required by regulation (VHF/ADS-B).
    UnencryptedRegulated,
    /// Broadcast, unencrypted by design (ADS-B).
    BroadcastUnencrypted,
    /// Firmware signature check failed.
    UntrustedFirmware,
    /// No encryption at all — blocked.
    Unencrypted,
}

/// Security posture inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPosture {
    /// mTLS handshake established.
    pub mtls_active: bool,
    /// Modem firmware signature verified at boot.
    pub firmware_signed: bool,
    /// Replay protection nonce counter current.
    pub replay_nonce_valid: bool,
    /// TLS certificate pinned to known CA.
    pub certificate_pinned: bool,
}

/// Security gate check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    /// Pass/warn/block verdict.
    pub verdict: SecurityVerdict,
    /// Security level achieved.
    pub level: SecurityLevel,
    /// What's missing.
    pub missing: &'static str,
}

/// Validate security posture for a given link type.
pub fn check_security(link: LinkType, posture: &SecurityPosture) -> SecurityCheck {
    // Aviation bands — unencrypted by regulation, always pass
    if !link.encryption_allowed() {
        let level = match link {
            LinkType::AdsbOut => SecurityLevel::BroadcastUnencrypted,
            _ => SecurityLevel::UnencryptedRegulated,
        };
        return SecurityCheck {
            verdict: SecurityVerdict::Pass,
            level,
            missing: "none — unencrypted by regulation",
        };
    }

    // LoRa — no IP stack, AES-128 payload only
    if link == LinkType::LoraSx1276 {
        return if posture.replay_nonce_valid {
            SecurityCheck {
                verdict: SecurityVerdict::Pass,
                level: SecurityLevel::EncryptedNoMtls,
                missing: "no mTLS (acceptable — no IP stack)",
            }
        } else {
            SecurityCheck {
                verdict: SecurityVerdict::Warn,
                level: SecurityLevel::EncryptedReplayRisk,
                missing: "replay nonce counter stale",
            }
        };
    }

    // All IP-based links require mTLS
    if !posture.mtls_active {
        return SecurityCheck {
            verdict: SecurityVerdict::Block,
            level: SecurityLevel::Unencrypted,
            missing: "mTLS not established",
        };
    }

    if !posture.firmware_signed {
        return SecurityCheck {
            verdict: SecurityVerdict::Block,
            level: SecurityLevel::UntrustedFirmware,
            missing: "firmware signature verification failed",
        };
    }

    if !posture.replay_nonce_valid {
        return SecurityCheck {
            verdict: SecurityVerdict::Warn,
            level: SecurityLevel::EncryptedReplayRisk,
            missing: "replay nonce stale",
        };
    }

    if !posture.certificate_pinned {
        return SecurityCheck {
            verdict: SecurityVerdict::Warn,
            level: SecurityLevel::EncryptedUnpinned,
            missing: "certificate not pinned — MITM risk",
        };
    }

    SecurityCheck {
        verdict: SecurityVerdict::Pass,
        level: SecurityLevel::FullMtlsPinned,
        missing: "none",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_posture() -> SecurityPosture {
        SecurityPosture {
            mtls_active: true,
            firmware_signed: true,
            replay_nonce_valid: true,
            certificate_pinned: true,
        }
    }

    #[test]
    fn test_vhf_always_passes() {
        let empty = SecurityPosture {
            mtls_active: false,
            firmware_signed: false,
            replay_nonce_valid: false,
            certificate_pinned: false,
        };
        let r = check_security(LinkType::VhfRadio, &empty);
        assert_eq!(r.verdict, SecurityVerdict::Pass);
        assert_eq!(r.level, SecurityLevel::UnencryptedRegulated);
    }

    #[test]
    fn test_adsb_broadcast() {
        let r = check_security(LinkType::AdsbOut, &full_posture());
        assert_eq!(r.level, SecurityLevel::BroadcastUnencrypted);
    }

    #[test]
    fn test_5g_full_security() {
        let r = check_security(LinkType::FiveGModem, &full_posture());
        assert_eq!(r.verdict, SecurityVerdict::Pass);
        assert_eq!(r.level, SecurityLevel::FullMtlsPinned);
    }

    #[test]
    fn test_5g_no_mtls_blocks() {
        let mut p = full_posture();
        p.mtls_active = false;
        let r = check_security(LinkType::FiveGModem, &p);
        assert_eq!(r.verdict, SecurityVerdict::Block);
    }

    #[test]
    fn test_wifi_unsigned_firmware_blocks() {
        let mut p = full_posture();
        p.firmware_signed = false;
        let r = check_security(LinkType::Wifi6e, &p);
        assert_eq!(r.verdict, SecurityVerdict::Block);
        assert_eq!(r.level, SecurityLevel::UntrustedFirmware);
    }

    #[test]
    fn test_unpinned_cert_warns() {
        let mut p = full_posture();
        p.certificate_pinned = false;
        let r = check_security(LinkType::Ble5, &p);
        assert_eq!(r.verdict, SecurityVerdict::Warn);
    }

    #[test]
    fn test_lora_with_nonce_passes() {
        let r = check_security(LinkType::LoraSx1276, &full_posture());
        assert_eq!(r.verdict, SecurityVerdict::Pass);
        assert_eq!(r.level, SecurityLevel::EncryptedNoMtls);
    }

    #[test]
    fn test_lora_stale_nonce_warns() {
        let mut p = full_posture();
        p.replay_nonce_valid = false;
        let r = check_security(LinkType::LoraSx1276, &p);
        assert_eq!(r.verdict, SecurityVerdict::Warn);
    }
}
