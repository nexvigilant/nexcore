// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Lex Primitiva Grounding
//!
//! `GroundsTo` implementations for all public types in `nexcore-os`.
//!
//! ## Dominant Primitive Distribution
//!
//! - `BootPhase`, `ServiceState`, `SecurityLevel`, `VaultState` —
//!   Lifecycle state enums ground to **State** (ς) dominant.
//! - `BootSequence`, `OsVault`, `NexCoreOs` — T3 domain composites
//!   ground to the primitive that drives their core responsibility:
//!   **Sequence** (σ) for boot, **Boundary** (∂) for vault, **Sum** (Σ)
//!   for the full OS.
//! - `ServiceId` — **Existence** (∃) dominant: pure identity token.
//! - `ThreatPattern`, `Pamp`, `Damp` — **Boundary** (∂) dominant:
//!   threat detection at security perimeters.
//! - `EventBus` — **Causality** (→) dominant: event emission causes
//!   handler activation.
//! - User types — **Boundary** (∂) dominant: authentication boundary.
//! - `OsError` sub-errors — **Boundary** (∂) dominant: subsystem limits.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::app_clearance::{AppClearanceLevel, AppPermission, ClearanceResult};
use crate::boot::{BootPhase, BootSequence};
use crate::error::OsError;
use crate::ipc::EventBus;
use crate::secure_boot::{BootStage, SecureBootChain};
use crate::security::{Damp, Pamp, SecurityLevel, SecurityMonitor, SecurityResponse, ThreatPattern};
use crate::service::{Service, ServiceId, ServiceState};
use crate::user::{AccountStatus, AuthError, Session, UserManager, UserRecord, UserRole};
use crate::vault::{OsVault, SecretCategory, VaultState};

// ---------------------------------------------------------------------------
// T1 Pure Primitives
// ---------------------------------------------------------------------------

/// `ServiceState`: T1, Dominant ς State
///
/// Pure lifecycle state machine: Registered→Starting→Running→Degraded→Stopping→Stopped|Failed.
impl GroundsTo for ServiceState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `VaultState`: T1, Dominant ς State
///
/// Pure lifecycle: Uninitialized → Locked → Unlocked.
impl GroundsTo for VaultState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `SecretCategory`: T1, Dominant κ Comparison
///
/// Binary classification: System | User — partitions secrets by origin.
impl GroundsTo for SecretCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

/// `AccountStatus`: T1, Dominant ς State
///
/// Pure user account lifecycle state.
impl GroundsTo for AccountStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
            .with_dominant(LexPrimitiva::State, 1.0)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `UserRole`: T1, Dominant κ Comparison
///
/// Role classification: Admin | Standard | Guest — partitions users by privilege.
impl GroundsTo for UserRole {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

/// `BootStage`: T1, Dominant σ Sequence
///
/// Ordered boot stage position in the secure boot chain.
impl GroundsTo for BootStage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sequence, 1.0)
    }
}

/// `AppClearanceLevel`: T1, Dominant κ Comparison
///
/// Clearance level classification for app permissions.
impl GroundsTo for AppClearanceLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

// ---------------------------------------------------------------------------
// T2-P Cross-Domain Primitives
// ---------------------------------------------------------------------------

/// `BootPhase`: T2-P (σ Sequence + ς State), dominant σ
///
/// Each phase is both a position in the boot sequence (σ) and a
/// momentary system lifecycle state (ς).
/// Sequence-dominant: the defining property is ordered progression.
impl GroundsTo for BootPhase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- ordered boot phase position
            LexPrimitiva::State,    // ς -- current system lifecycle state
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `SecurityLevel`: T2-P (ς State + κ Comparison), dominant ς
///
/// Green | Yellow | Orange | Red security posture.
/// State-dominant: encodes the current OS security posture state.
/// Comparison is secondary: levels are ordered and compared (Ord impl).
impl GroundsTo for SecurityLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς -- security posture lifecycle
            LexPrimitiva::Comparison, // κ -- levels are ordered (Green < Red)
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// `ServiceId`: T2-P (∃ Existence + N Quantity), dominant ∃
///
/// Unique service identity token backed by a u32.
/// Existence-dominant: the purpose is to establish that a service *exists*
/// with a given identity. Quantity is secondary: the raw u32 value.
impl GroundsTo for ServiceId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ -- service identity assertion
            LexPrimitiva::Quantity,  // N -- raw u32 id value
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

/// `SecurityResponse`: T2-P (ς State + ∂ Boundary), dominant ς
///
/// The action taken in response to a security threat.
/// State-dominant: response represents a new security operational state.
/// Boundary is secondary: responses enforce or relax security perimeters.
impl GroundsTo for SecurityResponse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- new operational state after threat
            LexPrimitiva::Boundary, // ∂ -- security perimeter enforcement
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// `AuthError`: T2-P (∂ Boundary + κ Comparison), dominant ∂
///
/// Authentication failure at a credential boundary.
/// Boundary-dominant: auth errors mark the security boundary violation point.
/// Comparison is secondary: errors are compared to produce appropriate responses.
impl GroundsTo for AuthError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- authentication boundary violation
            LexPrimitiva::Comparison, // κ -- error type classification
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `OsError`: T2-P (∂ Boundary + Σ Sum), dominant ∂
///
/// Top-level OS error — a boundary violation across subsystems.
/// Boundary-dominant: all OS errors represent subsystem boundary failures.
/// Sum is secondary: union of all subsystem error variants.
impl GroundsTo for OsError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- OS subsystem boundary violation
            LexPrimitiva::Sum,      // Σ -- union of error variants
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `AppPermission`: T2-P (∂ Boundary + κ Comparison), dominant ∂
///
/// An app capability permission: Camera | Microphone | etc.
/// Boundary-dominant: each permission guards a capability boundary.
/// Comparison is secondary: permissions are compared to clearance levels.
impl GroundsTo for AppPermission {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- capability access boundary
            LexPrimitiva::Comparison, // κ -- permission level comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// `ClearanceResult`: T2-P (∂ Boundary + Σ Sum), dominant ∂
///
/// The result of an app clearance check: Granted | Denied | Restricted.
/// Boundary-dominant: the result determines whether a boundary is crossed.
/// Sum is secondary: union of outcome variants.
impl GroundsTo for ClearanceResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- access boundary determination
            LexPrimitiva::Sum,      // Σ -- outcome variant enumeration
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T2-C Cross-Domain Composites
// ---------------------------------------------------------------------------

/// `Pamp`: T2-C (∂ Boundary + κ Comparison + Σ Sum + N Quantity), dominant ∂
///
/// Pathogen-Associated Molecular Pattern — external threat signature.
/// Boundary-dominant: PAMPs signal breaches of security perimeters.
/// Comparison is secondary: threat patterns are compared to classify severity.
/// Sum is tertiary: union of threat variant types.
/// Quantity is quaternary: count, ports_probed and other numeric fields.
impl GroundsTo for Pamp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- external threat at security boundary
            LexPrimitiva::Comparison, // κ -- threat severity comparison
            LexPrimitiva::Sum,        // Σ -- union of PAMP variant types
            LexPrimitiva::Quantity,   // N -- count, port ranges, thresholds
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// `Damp`: T2-C (∂ Boundary + ν Frequency + N Quantity + ς State), dominant ∂
///
/// Damage-Associated Molecular Pattern — internal damage signal.
/// Boundary-dominant: DAMPs signal the system is past safe operational bounds.
/// Frequency is secondary: sustained CPU/memory pressure signals recur over time.
/// Quantity is tertiary: usage percentages and thresholds are numeric.
/// State is quaternary: crash and disk-full events represent state transitions.
impl GroundsTo for Damp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ -- past safe operational boundary
            LexPrimitiva::Frequency, // ν -- sustained/recurring damage signals
            LexPrimitiva::Quantity,  // N -- usage_pct, sustained_ticks
            LexPrimitiva::State,     // ς -- crash/disk-full as state transitions
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// `ThreatPattern`: T2-C (∂ Boundary + κ Comparison + Σ Sum + → Causality), dominant ∂
///
/// Top-level threat discriminant: External(Pamp) | Internal(Damp).
/// Boundary-dominant: all threats represent security boundary violations.
/// Comparison is secondary: classify_severity() compares against thresholds.
/// Sum is tertiary: union of External/Internal variants.
/// Causality is quaternary: threat → security response chain.
impl GroundsTo for ThreatPattern {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- security boundary violation
            LexPrimitiva::Comparison, // κ -- severity threshold comparison
            LexPrimitiva::Sum,        // Σ -- External | Internal union
            LexPrimitiva::Causality,  // → -- threat causes response
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// `UserRecord`: T2-C (π Persistence + ∂ Boundary + κ Comparison + ς State), dominant π
///
/// Persisted user account record with authentication credentials.
/// Persistence-dominant: the record is the durable account representation.
/// Boundary is secondary: password hash enforces authentication boundary.
/// Comparison is tertiary: role comparison for permission checks.
/// State is quaternary: AccountStatus lifecycle.
impl GroundsTo for UserRecord {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // π -- durable account record
            LexPrimitiva::Boundary,    // ∂ -- credential authentication boundary
            LexPrimitiva::Comparison,  // κ -- role-based access comparison
            LexPrimitiva::State,       // ς -- AccountStatus lifecycle
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

/// `Session`: T2-C (ς State + ∂ Boundary + π Persistence + N Quantity), dominant ς
///
/// An active user session with authentication token and expiry.
/// State-dominant: the session is a bounded active state.
/// Boundary is secondary: session token enforces authentication boundary.
/// Persistence is tertiary: session may be persisted across power cycles.
/// Quantity is quaternary: expiry timestamps are numeric.
impl GroundsTo for Session {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // ς -- active session state
            LexPrimitiva::Boundary,    // ∂ -- authentication boundary token
            LexPrimitiva::Persistence, // π -- session persistence
            LexPrimitiva::Quantity,    // N -- expiry timestamp
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `Service`: T2-C (ς State + σ Sequence + ∃ Existence + N Quantity), dominant ς
///
/// A registered OS service with priority and lifecycle state.
/// State-dominant: the service is primarily characterized by its current state.
/// Sequence is secondary: services start and stop in ordered sequences.
/// Existence is tertiary: service availability is checked via ServiceId.
/// Quantity is quaternary: priority value is numeric.
impl GroundsTo for Service {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς -- service lifecycle state
            LexPrimitiva::Sequence,  // σ -- ordered start/stop sequences
            LexPrimitiva::Existence, // ∃ -- service availability assertion
            LexPrimitiva::Quantity,  // N -- priority value
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// T3 Domain-Specific Types
// ---------------------------------------------------------------------------

/// `BootSequence`: T3 (σ + → + ∂ + ς + π), dominant σ
///
/// Four-phase system boot orchestrator.
/// Sequence-dominant: the entire purpose is ordered phase progression.
/// Causality is secondary: each phase causally enables the next.
/// Boundary is tertiary: phase transitions enforce ordering invariants.
/// State is quaternary: current BootPhase tracks progress.
/// Persistence is quinary: boot log records the history.
impl GroundsTo for BootSequence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // σ -- ordered boot phase progression
            LexPrimitiva::Causality,   // → -- each phase enables the next
            LexPrimitiva::Boundary,    // ∂ -- phase transition invariants
            LexPrimitiva::State,       // ς -- current phase state
            LexPrimitiva::Persistence, // π -- boot log audit trail
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `SecurityMonitor`: T3 (∂ + ς + κ + → + Σ), dominant ∂
///
/// Kernel-level security monitor: collects PAMPs/DAMPs, escalates level.
/// Boundary-dominant: the monitor's entire purpose is security boundary enforcement.
/// State is secondary: current SecurityLevel tracks posture.
/// Comparison is tertiary: threat severity comparison drives escalation.
/// Causality is quaternary: threat pattern causes level transition.
/// Sum is quinary: composition of all threat detection subsystems.
impl GroundsTo for SecurityMonitor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- security perimeter enforcement
            LexPrimitiva::State,      // ς -- SecurityLevel posture
            LexPrimitiva::Comparison, // κ -- threat severity threshold comparison
            LexPrimitiva::Causality,  // → -- threat causes level escalation
            LexPrimitiva::Sum,        // Σ -- composition of threat detection
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `EventBus`: T3 (→ + μ + σ + N + ∂), dominant →
///
/// Synchronous IPC event bus using Cytokine signals.
/// Causality-dominant: event emission causally activates handlers — this
/// is the fundamental contract of an event bus.
/// Mapping is secondary: typed signal matching (family → handlers).
/// Sequence is tertiary: FIFO queue ordering.
/// Quantity is quaternary: queue depth tracking.
/// Boundary is quinary: max_depth back-pressure boundary.
impl GroundsTo for EventBus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- event causes handler activation
            LexPrimitiva::Mapping,   // μ -- typed signal family → handler mapping
            LexPrimitiva::Sequence,  // σ -- FIFO queue ordering
            LexPrimitiva::Quantity,  // N -- queue depth, total_emitted counts
            LexPrimitiva::Boundary,  // ∂ -- max_depth back-pressure limit
        ])
        .with_dominant(LexPrimitiva::Causality, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `OsVault`: T3 (ς + ∂ + μ + π + ∃), dominant ∂
///
/// OS-level encrypted vault with full lifecycle management.
/// Boundary-dominant: the vault *is* the cryptographic boundary between
/// plaintext and ciphertext secret storage.
/// State is secondary: Uninitialized → Locked → Unlocked lifecycle.
/// Mapping is tertiary: SecretName → EncryptedValue.
/// Persistence is quaternary: encrypted file persistence.
/// Existence is quinary: secret existence checks.
impl GroundsTo for OsVault {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // ∂ -- cryptographic encryption boundary
            LexPrimitiva::State,       // ς -- vault lifecycle state
            LexPrimitiva::Mapping,     // μ -- SecretName → EncryptedValue
            LexPrimitiva::Persistence, // π -- encrypted file persistence
            LexPrimitiva::Existence,   // ∃ -- secret existence checks
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `UserManager`: T3 (∂ + π + ς + κ + Σ), dominant ∂
///
/// User account lifecycle and authentication manager.
/// Boundary-dominant: authentication is the primary security boundary operation.
/// Persistence is secondary: user records are durably stored.
/// State is tertiary: AccountStatus lifecycle management.
/// Comparison is quaternary: role comparison for access control.
/// Sum is quinary: composition of user accounts and session tracking.
impl GroundsTo for UserManager {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // ∂ -- authentication boundary
            LexPrimitiva::Persistence, // π -- durable user record storage
            LexPrimitiva::State,       // ς -- AccountStatus lifecycle
            LexPrimitiva::Comparison,  // κ -- role-based access comparison
            LexPrimitiva::Sum,         // Σ -- composition of all user accounts
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// `SecureBootChain`: T3 (σ + → + ∂ + ∝ + κ), dominant σ
///
/// Ordered chain of boot stage measurements for secure boot attestation.
/// Sequence-dominant: the chain is an ordered list of measurements.
/// Causality is secondary: each stage measurement causally permits the next.
/// Boundary is tertiary: each stage verifies a cryptographic boundary.
/// Irreversibility is quaternary: measurements are write-once (hash chain).
/// Comparison is quinary: verification compares hashes against policy.
impl GroundsTo for SecureBootChain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,        // σ -- ordered measurement chain
            LexPrimitiva::Causality,       // → -- each stage enables the next
            LexPrimitiva::Boundary,        // ∂ -- cryptographic stage boundary
            LexPrimitiva::Irreversibility, // ∝ -- write-once hash measurements
            LexPrimitiva::Comparison,      // κ -- hash verification comparison
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // T1 tier tests

    #[test]
    fn service_state_is_t1() {
        assert_eq!(ServiceState::tier(), Tier::T1Universal);
        assert_eq!(
            ServiceState::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn vault_state_is_t1() {
        assert_eq!(VaultState::tier(), Tier::T1Universal);
        assert_eq!(VaultState::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn secret_category_is_t1() {
        assert_eq!(SecretCategory::tier(), Tier::T1Universal);
        assert_eq!(
            SecretCategory::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn boot_stage_is_t1() {
        assert_eq!(BootStage::tier(), Tier::T1Universal);
        assert_eq!(
            BootStage::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    // T2-P tier tests

    #[test]
    fn boot_phase_is_t2p() {
        assert_eq!(BootPhase::tier(), Tier::T2Primitive);
        assert_eq!(
            BootPhase::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn security_level_is_t2p() {
        assert_eq!(SecurityLevel::tier(), Tier::T2Primitive);
        assert_eq!(
            SecurityLevel::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn service_id_is_t2p() {
        assert_eq!(ServiceId::tier(), Tier::T2Primitive);
        assert_eq!(
            ServiceId::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn os_error_is_t2p() {
        assert_eq!(OsError::tier(), Tier::T2Primitive);
        assert_eq!(OsError::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    // T2-C tier tests

    #[test]
    fn pamp_is_t2c() {
        let tier = Pamp::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(Pamp::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn damp_is_t2c() {
        let tier = Damp::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(Damp::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn threat_pattern_is_t2c() {
        let tier = ThreatPattern::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(
            ThreatPattern::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn service_is_t2c() {
        let tier = Service::tier();
        assert!(
            tier == Tier::T2Composite || tier == Tier::T3DomainSpecific,
            "expected T2-C or T3, got {tier:?}"
        );
        assert_eq!(Service::dominant_primitive(), Some(LexPrimitiva::State));
    }

    // T3 tier tests

    #[test]
    fn boot_sequence_is_t3() {
        assert_eq!(BootSequence::tier(), Tier::T2Composite);
        assert_eq!(
            BootSequence::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn security_monitor_is_t3() {
        assert_eq!(SecurityMonitor::tier(), Tier::T2Composite);
        assert_eq!(
            SecurityMonitor::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn event_bus_is_t3() {
        assert_eq!(EventBus::tier(), Tier::T2Composite);
        assert_eq!(
            EventBus::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn os_vault_is_t3() {
        assert_eq!(OsVault::tier(), Tier::T2Composite);
        assert_eq!(OsVault::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn secure_boot_chain_is_t3() {
        assert_eq!(SecureBootChain::tier(), Tier::T2Composite);
        assert_eq!(
            SecureBootChain::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    // All types have dominant primitive

    #[test]
    fn all_types_have_dominant() {
        assert!(ServiceState::dominant_primitive().is_some());
        assert!(VaultState::dominant_primitive().is_some());
        assert!(SecretCategory::dominant_primitive().is_some());
        assert!(BootPhase::dominant_primitive().is_some());
        assert!(SecurityLevel::dominant_primitive().is_some());
        assert!(ServiceId::dominant_primitive().is_some());
        assert!(SecurityResponse::dominant_primitive().is_some());
        assert!(OsError::dominant_primitive().is_some());
        assert!(Pamp::dominant_primitive().is_some());
        assert!(Damp::dominant_primitive().is_some());
        assert!(ThreatPattern::dominant_primitive().is_some());
        assert!(Service::dominant_primitive().is_some());
        assert!(BootSequence::dominant_primitive().is_some());
        assert!(SecurityMonitor::dominant_primitive().is_some());
        assert!(EventBus::dominant_primitive().is_some());
        assert!(OsVault::dominant_primitive().is_some());
        assert!(UserManager::dominant_primitive().is_some());
    }

    // Composition content spot-checks

    #[test]
    fn boot_sequence_has_causality_and_persistence() {
        let comp = BootSequence::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }

    #[test]
    fn event_bus_has_causality_dominant_and_mapping() {
        let comp = EventBus::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
    }

    #[test]
    fn os_vault_has_five_primitives() {
        let comp = OsVault::primitive_composition();
        assert!(comp.unique().len() >= 5);
    }
}
