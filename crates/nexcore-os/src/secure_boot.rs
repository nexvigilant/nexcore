// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Secure boot chain — measured boot, image verification, and chain of trust.
//!
//! ## Architecture
//!
//! Implements a TPM-style measured boot chain where each boot stage extends
//! a Platform Configuration Register (PCR) with its measurement. The chain
//! of trust is verified at each transition:
//!
//! ```text
//! UEFI firmware  →  Bootloader  →  Linux kernel  →  NexCore OS
//!     PCR[0]           PCR[1]          PCR[2]          PCR[3]
//! ```
//!
//! Each stage measures the next before handing off, creating an immutable
//! audit trail. If any measurement fails verification, boot policy determines
//! the response (warn, degrade, or halt).
//!
//! ## Primitive Grounding
//!
//! - σ Sequence: Boot chain stages execute in strict order
//! - → Causality: Each stage causes the next (hash extends)
//! - ∂ Boundary: Trust boundary between verified/unverified
//! - ∃ Existence: Image existence verification
//! - ∝ Irreversibility: PCR extends are one-way (append-only hash chain)
//! - κ Comparison: Expected vs actual measurement comparison

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Number of Platform Configuration Registers.
const PCR_COUNT: usize = 8;

/// Size of a SHA-256 digest in bytes.
const DIGEST_SIZE: usize = 32;

/// Boot chain stage — each stage in the measured boot sequence.
///
/// Tier: T2-P (σ Sequence — boot chain ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BootStage {
    /// UEFI firmware measurement (PCR 0).
    Firmware = 0,
    /// Bootloader measurement (PCR 1).
    Bootloader = 1,
    /// Linux kernel measurement (PCR 2).
    Kernel = 2,
    /// NexCore OS binary measurement (PCR 3).
    NexCoreOs = 3,
    /// Init system measurement (PCR 4).
    Init = 4,
    /// System services measurement (PCR 5).
    Services = 5,
    /// Shell/UI measurement (PCR 6).
    Shell = 6,
    /// User-space apps measurement (PCR 7).
    Apps = 7,
}

impl BootStage {
    /// Get the PCR index for this stage.
    pub fn pcr_index(self) -> usize {
        self as usize
    }

    /// Get the stage name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Firmware => "firmware",
            Self::Bootloader => "bootloader",
            Self::Kernel => "kernel",
            Self::NexCoreOs => "nexcore-os",
            Self::Init => "init",
            Self::Services => "services",
            Self::Shell => "shell",
            Self::Apps => "apps",
        }
    }

    /// All stages in boot order.
    pub fn all() -> &'static [Self] {
        &[
            Self::Firmware,
            Self::Bootloader,
            Self::Kernel,
            Self::NexCoreOs,
            Self::Init,
            Self::Services,
            Self::Shell,
            Self::Apps,
        ]
    }
}

impl core::fmt::Display for BootStage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Boot policy — what to do when a measurement fails.
///
/// Tier: T2-P (∂ Boundary — security policy boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootPolicy {
    /// Strict: halt boot on any verification failure.
    Strict,
    /// Degraded: continue boot in reduced-privilege mode on failure.
    Degraded,
    /// Permissive: log warnings but continue (development mode).
    Permissive,
}

impl BootPolicy {
    /// Whether this policy allows boot to continue after a failure.
    pub fn allows_continue(self) -> bool {
        !matches!(self, Self::Strict)
    }
}

impl core::fmt::Display for BootPolicy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Degraded => write!(f, "degraded"),
            Self::Permissive => write!(f, "permissive"),
        }
    }
}

/// A measurement — SHA-256 hash of a boot component.
///
/// Tier: T2-P (∝ Irreversibility — one-way hash)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Measurement {
    /// The 32-byte SHA-256 digest.
    digest: [u8; DIGEST_SIZE],
}

impl Measurement {
    /// Create a measurement from raw data (hashes the input).
    pub fn from_data(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut digest = [0u8; DIGEST_SIZE];
        digest.copy_from_slice(&result);
        Self { digest }
    }

    /// Create a measurement from a pre-computed digest.
    pub fn from_digest(digest: [u8; DIGEST_SIZE]) -> Self {
        Self { digest }
    }

    /// Create a zero measurement (initial PCR state).
    pub fn zero() -> Self {
        Self {
            digest: [0u8; DIGEST_SIZE],
        }
    }

    /// Get the raw digest bytes.
    pub fn as_bytes(&self) -> &[u8; DIGEST_SIZE] {
        &self.digest
    }

    /// Extend this measurement with new data (TPM PCR extend operation).
    ///
    /// `PCR_new = SHA-256(PCR_old || new_measurement)`
    ///
    /// This is the core of the measured boot chain — irreversible accumulation.
    #[must_use]
    pub fn extend(&self, new: &Self) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(self.digest);
        hasher.update(new.digest);
        let result = hasher.finalize();
        let mut digest = [0u8; DIGEST_SIZE];
        digest.copy_from_slice(&result);
        Self { digest }
    }

    /// Format as hex string (first 8 bytes for display).
    pub fn short_hex(&self) -> String {
        use core::fmt::Write;
        self.digest[..8]
            .iter()
            .fold(String::with_capacity(16), |mut s, b| {
                let _ = write!(s, "{b:02x}");
                s
            })
    }

    /// Format as full hex string.
    pub fn hex(&self) -> String {
        use core::fmt::Write;
        self.digest
            .iter()
            .fold(String::with_capacity(64), |mut s, b| {
                let _ = write!(s, "{b:02x}");
                s
            })
    }
}

impl core::fmt::Debug for Measurement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Measurement({}...)", self.short_hex())
    }
}

impl core::fmt::Display for Measurement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.short_hex())
    }
}

/// Boot attestation record — a single measurement event.
///
/// Tier: T2-C (σ + → + ∝ — ordered, causal, irreversible record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationRecord {
    /// Which boot stage produced this measurement.
    pub stage: BootStage,
    /// The measurement value.
    pub measurement: Measurement,
    /// Description of what was measured.
    pub description: String,
    /// Whether this measurement matched the expected value.
    pub verified: bool,
    /// Timestamp (seconds since boot).
    pub boot_time_secs: u64,
}

/// Verification result for a single stage.
///
/// Tier: T2-P (κ Comparison — expected vs actual)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyResult {
    /// Measurement matches expected value.
    Ok,
    /// Measurement mismatch — stage has been tampered with.
    Mismatch {
        expected: Measurement,
        actual: Measurement,
    },
    /// No expected measurement registered for this stage.
    NoExpectation,
    /// Stage has not been measured yet.
    NotMeasured,
}

impl VerifyResult {
    /// Whether the result is a successful verification.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok | Self::NoExpectation)
    }

    /// Whether the result indicates a security violation.
    pub fn is_violation(&self) -> bool {
        matches!(self, Self::Mismatch { .. })
    }
}

/// Secure boot chain — measured boot with PCR extend and verification.
///
/// Tier: T3 (σ + → + ∂ + ∃ + ∝ + κ — full secure boot system)
pub struct SecureBootChain {
    /// Platform Configuration Registers (PCR).
    pcrs: [Measurement; PCR_COUNT],
    /// Expected measurements (golden values) for each stage.
    expected: [Option<Measurement>; PCR_COUNT],
    /// Boot attestation log (append-only).
    log: Vec<AttestationRecord>,
    /// Current boot policy.
    policy: BootPolicy,
    /// The highest stage that has been measured.
    last_measured: Option<BootStage>,
    /// Whether the boot chain is in a degraded state.
    degraded: bool,
    /// Count of verification failures.
    failure_count: u32,
    /// Boot start time reference (monotonic counter).
    boot_counter: u64,
}

impl SecureBootChain {
    /// Create a new secure boot chain with the given policy.
    pub fn new(policy: BootPolicy) -> Self {
        Self {
            pcrs: core::array::from_fn(|_| Measurement::zero()),
            expected: core::array::from_fn(|_| None),
            log: Vec::new(),
            policy,
            last_measured: None,
            degraded: false,
            failure_count: 0,
            boot_counter: 0,
        }
    }

    /// Create a permissive boot chain (for development/testing).
    pub fn permissive() -> Self {
        Self::new(BootPolicy::Permissive)
    }

    /// Register an expected measurement for a boot stage.
    ///
    /// In production, these come from a signed manifest.
    pub fn register_expected(&mut self, stage: BootStage, measurement: Measurement) {
        self.expected[stage.pcr_index()] = Some(measurement);
    }

    /// Measure a boot stage — extend its PCR and log the attestation.
    ///
    /// This is the core boot chain operation:
    /// 1. Hash the stage data
    /// 2. Extend the PCR: `PCR[i] = SHA-256(PCR[i] || hash(data))`
    /// 3. Log the attestation record
    /// 4. Verify against expected measurement if registered
    ///
    /// Returns the verification result.
    pub fn measure(
        &mut self,
        stage: BootStage,
        data: &[u8],
        description: impl Into<String>,
    ) -> VerifyResult {
        let measurement = Measurement::from_data(data);
        self.measure_with(stage, measurement, description)
    }

    /// Measure a boot stage with a pre-computed measurement.
    pub fn measure_with(
        &mut self,
        stage: BootStage,
        measurement: Measurement,
        description: impl Into<String>,
    ) -> VerifyResult {
        let pcr_idx = stage.pcr_index();

        // Extend the PCR
        self.pcrs[pcr_idx] = self.pcrs[pcr_idx].extend(&measurement);

        // Verify against expected
        let result = self.verify_stage_measurement(stage, &measurement);

        // Log the attestation
        self.boot_counter += 1;
        self.log.push(AttestationRecord {
            stage,
            measurement,
            description: description.into(),
            verified: result.is_ok(),
            boot_time_secs: self.boot_counter,
        });

        // Track state
        self.last_measured = Some(stage);

        if result.is_violation() {
            self.failure_count += 1;
            if self.policy == BootPolicy::Degraded {
                self.degraded = true;
            }
        }

        result
    }

    /// Verify a measurement against expected values.
    fn verify_stage_measurement(
        &self,
        stage: BootStage,
        measurement: &Measurement,
    ) -> VerifyResult {
        self.expected[stage.pcr_index()]
            .as_ref()
            .map_or(VerifyResult::NoExpectation, |expected| {
                if measurement == expected {
                    VerifyResult::Ok
                } else {
                    VerifyResult::Mismatch {
                        expected: expected.clone(),
                        actual: measurement.clone(),
                    }
                }
            })
    }

    /// Check if a specific stage can proceed based on policy and verification.
    pub fn can_proceed(&self, stage: BootStage) -> bool {
        match self.policy {
            BootPolicy::Permissive | BootPolicy::Degraded => true,
            BootPolicy::Strict => {
                // In strict mode, only proceed if all previous stages verified
                let idx = stage.pcr_index();
                for record in &self.log {
                    if record.stage.pcr_index() < idx && !record.verified {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Get the current PCR value for a stage.
    pub fn pcr(&self, stage: BootStage) -> &Measurement {
        &self.pcrs[stage.pcr_index()]
    }

    /// Get the full attestation log.
    pub fn attestation_log(&self) -> &[AttestationRecord] {
        &self.log
    }

    /// Get the boot policy.
    pub fn policy(&self) -> BootPolicy {
        self.policy
    }

    /// Whether the boot chain is in degraded mode.
    pub fn is_degraded(&self) -> bool {
        self.degraded
    }

    /// Get the count of verification failures.
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Get the last measured stage.
    pub fn last_measured(&self) -> Option<BootStage> {
        self.last_measured
    }

    /// Get the total number of attestation records.
    pub fn record_count(&self) -> usize {
        self.log.len()
    }

    /// Verify the entire chain — check all measured stages against expectations.
    pub fn verify_chain(&self) -> ChainVerification {
        let mut results = Vec::new();
        let mut all_ok = true;

        for record in &self.log {
            let stage = record.stage;
            let result = self.verify_stage_measurement(stage, &record.measurement);
            if !result.is_ok() {
                all_ok = false;
            }
            results.push((stage, result));
        }

        ChainVerification {
            results,
            all_verified: all_ok,
            policy: self.policy,
        }
    }

    /// Generate a boot quote — a summary of all PCR values.
    ///
    /// In a real TPM, this would be signed by the TPM's attestation key.
    pub fn quote(&self) -> BootQuote {
        let mut pcr_values = Vec::with_capacity(PCR_COUNT);
        for (i, pcr) in self.pcrs.iter().enumerate() {
            if *pcr != Measurement::zero() {
                pcr_values.push((i, pcr.clone()));
            }
        }

        // Compute composite hash: SHA-256(PCR[0] || PCR[1] || ... || PCR[7])
        let mut hasher = Sha256::new();
        for pcr in &self.pcrs {
            hasher.update(pcr.as_bytes());
        }
        let result = hasher.finalize();
        let mut composite = [0u8; DIGEST_SIZE];
        composite.copy_from_slice(&result);

        BootQuote {
            pcr_values,
            composite: Measurement::from_digest(composite),
            degraded: self.degraded,
            failure_count: self.failure_count,
        }
    }
}

/// Chain verification result — outcome of verifying the entire boot chain.
///
/// Tier: T2-C (κ + Σ — aggregated comparison results)
#[derive(Debug)]
pub struct ChainVerification {
    /// Per-stage verification results.
    pub results: Vec<(BootStage, VerifyResult)>,
    /// Whether all stages verified successfully.
    pub all_verified: bool,
    /// The policy under which verification was performed.
    pub policy: BootPolicy,
}

impl ChainVerification {
    /// Get all stages that failed verification.
    pub fn failures(&self) -> Vec<&(BootStage, VerifyResult)> {
        self.results
            .iter()
            .filter(|(_, r)| r.is_violation())
            .collect()
    }

    /// Whether boot should proceed given the policy and results.
    pub fn should_proceed(&self) -> bool {
        if self.all_verified {
            return true;
        }
        self.policy.allows_continue()
    }
}

/// Boot quote — signed attestation of PCR state.
///
/// Tier: T2-C (Σ + ∝ — aggregated irreversible measurements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootQuote {
    /// Non-zero PCR values (index, value).
    pub pcr_values: Vec<(usize, Measurement)>,
    /// Composite hash of all PCRs.
    pub composite: Measurement,
    /// Whether boot was degraded.
    pub degraded: bool,
    /// Number of verification failures.
    pub failure_count: u32,
}

impl BootQuote {
    /// Get a specific PCR value from the quote.
    pub fn pcr(&self, index: usize) -> Option<&Measurement> {
        self.pcr_values
            .iter()
            .find(|(i, _)| *i == index)
            .map(|(_, m)| m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_from_data() {
        let m1 = Measurement::from_data(b"hello");
        let m2 = Measurement::from_data(b"hello");
        let m3 = Measurement::from_data(b"world");

        // Deterministic
        assert_eq!(m1, m2);
        // Different input → different hash
        assert_ne!(m1, m3);
    }

    #[test]
    fn measurement_zero() {
        let zero = Measurement::zero();
        assert_eq!(zero.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn measurement_extend_irreversible() {
        let pcr = Measurement::zero();
        let m1 = Measurement::from_data(b"stage1");

        let extended = pcr.extend(&m1);

        // Extended is not the original
        assert_ne!(extended, pcr);
        // Extended is not the measurement
        assert_ne!(extended, m1);
        // Extending again with same value gives different result (chain property)
        let double = extended.extend(&m1);
        assert_ne!(double, extended);
    }

    #[test]
    fn measurement_extend_order_matters() {
        let m1 = Measurement::from_data(b"first");
        let m2 = Measurement::from_data(b"second");

        let ab = Measurement::zero().extend(&m1).extend(&m2);
        let ba = Measurement::zero().extend(&m2).extend(&m1);

        // Order matters — different chain = different result
        assert_ne!(ab, ba);
    }

    #[test]
    fn measurement_hex_display() {
        let m = Measurement::from_data(b"test");
        let short = m.short_hex();
        let full = m.hex();

        assert_eq!(short.len(), 16); // 8 bytes * 2 hex chars
        assert_eq!(full.len(), 64); // 32 bytes * 2 hex chars
        assert!(full.starts_with(&short));
    }

    #[test]
    fn boot_stage_ordering() {
        assert!(BootStage::Firmware < BootStage::Bootloader);
        assert!(BootStage::Bootloader < BootStage::Kernel);
        assert!(BootStage::Kernel < BootStage::NexCoreOs);
        assert!(BootStage::NexCoreOs < BootStage::Init);
    }

    #[test]
    fn boot_stage_pcr_index() {
        assert_eq!(BootStage::Firmware.pcr_index(), 0);
        assert_eq!(BootStage::NexCoreOs.pcr_index(), 3);
        assert_eq!(BootStage::Apps.pcr_index(), 7);
    }

    #[test]
    fn boot_stage_names() {
        assert_eq!(BootStage::Firmware.name(), "firmware");
        assert_eq!(BootStage::NexCoreOs.name(), "nexcore-os");
        assert_eq!(BootStage::all().len(), 8);
    }

    #[test]
    fn policy_continue() {
        assert!(!BootPolicy::Strict.allows_continue());
        assert!(BootPolicy::Degraded.allows_continue());
        assert!(BootPolicy::Permissive.allows_continue());
    }

    #[test]
    fn policy_display() {
        assert_eq!(BootPolicy::Strict.to_string(), "strict");
        assert_eq!(BootPolicy::Degraded.to_string(), "degraded");
        assert_eq!(BootPolicy::Permissive.to_string(), "permissive");
    }

    #[test]
    fn chain_permissive_no_expectations() {
        let mut chain = SecureBootChain::permissive();

        let r1 = chain.measure(BootStage::Firmware, b"uefi-binary", "UEFI firmware");
        assert_eq!(r1, VerifyResult::NoExpectation);

        let r2 = chain.measure(BootStage::Kernel, b"vmlinuz", "Linux kernel");
        assert_eq!(r2, VerifyResult::NoExpectation);

        assert_eq!(chain.record_count(), 2);
        assert_eq!(chain.failure_count(), 0);
    }

    #[test]
    fn chain_with_expected_measurements() {
        let mut chain = SecureBootChain::new(BootPolicy::Strict);

        // Register expected measurements
        let fw_expected = Measurement::from_data(b"firmware-v1.0");
        chain.register_expected(BootStage::Firmware, fw_expected);

        let kern_expected = Measurement::from_data(b"vmlinuz-6.17");
        chain.register_expected(BootStage::Kernel, kern_expected);

        // Measure with correct data
        let r1 = chain.measure(BootStage::Firmware, b"firmware-v1.0", "UEFI firmware");
        assert_eq!(r1, VerifyResult::Ok);

        let r2 = chain.measure(BootStage::Kernel, b"vmlinuz-6.17", "Linux kernel");
        assert_eq!(r2, VerifyResult::Ok);

        assert_eq!(chain.failure_count(), 0);
    }

    #[test]
    fn chain_detects_tampered_stage() {
        let mut chain = SecureBootChain::new(BootPolicy::Degraded);

        // Register expected firmware measurement
        let expected = Measurement::from_data(b"genuine-firmware");
        chain.register_expected(BootStage::Firmware, expected.clone());

        // Measure with tampered data
        let result = chain.measure(
            BootStage::Firmware,
            b"tampered-firmware",
            "Tampered firmware",
        );

        assert!(result.is_violation());
        assert_eq!(chain.failure_count(), 1);
        assert!(chain.is_degraded());

        if let VerifyResult::Mismatch {
            expected: exp,
            actual,
        } = &result
        {
            assert_eq!(exp, &expected);
            assert_ne!(exp, actual);
        }
    }

    #[test]
    fn chain_strict_blocks_after_failure() {
        let mut chain = SecureBootChain::new(BootPolicy::Strict);

        let expected = Measurement::from_data(b"good-firmware");
        chain.register_expected(BootStage::Firmware, expected);

        // Tampered firmware
        chain.measure(BootStage::Firmware, b"bad-firmware", "Bad firmware");

        // Next stage should not proceed in strict mode
        assert!(!chain.can_proceed(BootStage::Kernel));
    }

    #[test]
    fn chain_degraded_continues_after_failure() {
        let mut chain = SecureBootChain::new(BootPolicy::Degraded);

        let expected = Measurement::from_data(b"good-firmware");
        chain.register_expected(BootStage::Firmware, expected);

        chain.measure(BootStage::Firmware, b"bad-firmware", "Bad firmware");

        // Degraded mode still allows proceeding
        assert!(chain.can_proceed(BootStage::Kernel));
        assert!(chain.is_degraded());
    }

    #[test]
    fn pcr_values_accumulate() {
        let mut chain = SecureBootChain::permissive();

        let pcr_before = chain.pcr(BootStage::Firmware).clone();
        assert_eq!(pcr_before, Measurement::zero());

        chain.measure(BootStage::Firmware, b"firmware", "Firmware");
        let pcr_after = chain.pcr(BootStage::Firmware);

        assert_ne!(pcr_after, &Measurement::zero());
        assert_ne!(pcr_after, &pcr_before);
    }

    #[test]
    fn pcr_extend_multiple() {
        let mut chain = SecureBootChain::permissive();

        // Extending same PCR twice gives different value each time
        chain.measure(BootStage::Firmware, b"module-a", "Module A");
        let after_a = chain.pcr(BootStage::Firmware).clone();

        chain.measure(BootStage::Firmware, b"module-b", "Module B");
        let after_ab = chain.pcr(BootStage::Firmware).clone();

        assert_ne!(after_a, after_ab);
    }

    #[test]
    fn attestation_log_records_all() {
        let mut chain = SecureBootChain::permissive();

        chain.measure(BootStage::Firmware, b"fw", "Firmware");
        chain.measure(BootStage::Bootloader, b"grub", "GRUB");
        chain.measure(BootStage::Kernel, b"vmlinuz", "Linux");
        chain.measure(BootStage::NexCoreOs, b"nexcore", "NexCore OS");

        let log = chain.attestation_log();
        assert_eq!(log.len(), 4);
        assert_eq!(log[0].stage, BootStage::Firmware);
        assert_eq!(log[3].stage, BootStage::NexCoreOs);

        // Boot times are sequential
        assert!(log[0].boot_time_secs < log[3].boot_time_secs);
    }

    #[test]
    fn verify_chain_all_ok() {
        let mut chain = SecureBootChain::new(BootPolicy::Strict);

        let fw = Measurement::from_data(b"firmware");
        let kern = Measurement::from_data(b"kernel");
        chain.register_expected(BootStage::Firmware, fw);
        chain.register_expected(BootStage::Kernel, kern);

        chain.measure(BootStage::Firmware, b"firmware", "Firmware");
        chain.measure(BootStage::Kernel, b"kernel", "Kernel");

        let verification = chain.verify_chain();
        assert!(verification.all_verified);
        assert!(verification.should_proceed());
        assert!(verification.failures().is_empty());
    }

    #[test]
    fn verify_chain_with_failure() {
        let mut chain = SecureBootChain::new(BootPolicy::Degraded);

        let expected = Measurement::from_data(b"good");
        chain.register_expected(BootStage::Firmware, expected);

        chain.measure(BootStage::Firmware, b"bad", "Tampered");

        let verification = chain.verify_chain();
        assert!(!verification.all_verified);
        assert!(verification.should_proceed()); // Degraded allows continue
        assert_eq!(verification.failures().len(), 1);
    }

    #[test]
    fn boot_quote_composite() {
        let mut chain = SecureBootChain::permissive();

        chain.measure(BootStage::Firmware, b"fw", "Firmware");
        chain.measure(BootStage::Kernel, b"kern", "Kernel");

        let quote = chain.quote();
        assert!(!quote.degraded);
        assert_eq!(quote.failure_count, 0);

        // Should have 2 non-zero PCR values
        assert_eq!(quote.pcr_values.len(), 2);

        // Composite hash is not zero
        assert_ne!(quote.composite, Measurement::zero());
    }

    #[test]
    fn boot_quote_pcr_lookup() {
        let mut chain = SecureBootChain::permissive();
        chain.measure(BootStage::Kernel, b"vmlinuz", "Linux kernel");

        let quote = chain.quote();
        assert!(quote.pcr(BootStage::Kernel.pcr_index()).is_some());
        assert!(quote.pcr(BootStage::Firmware.pcr_index()).is_none());
    }

    #[test]
    fn last_measured_tracks() {
        let mut chain = SecureBootChain::permissive();
        assert!(chain.last_measured().is_none());

        chain.measure(BootStage::Firmware, b"fw", "FW");
        assert_eq!(chain.last_measured(), Some(BootStage::Firmware));

        chain.measure(BootStage::Kernel, b"kern", "Kern");
        assert_eq!(chain.last_measured(), Some(BootStage::Kernel));
    }

    #[test]
    fn full_boot_chain_simulation() {
        let mut chain = SecureBootChain::new(BootPolicy::Strict);

        // Register golden measurements (from signed manifest)
        let stages = [
            (BootStage::Firmware, b"uefi-v2.0-signed" as &[u8]),
            (BootStage::Bootloader, b"grub-2.12-signed"),
            (BootStage::Kernel, b"vmlinuz-6.17.9-nexcore"),
            (BootStage::NexCoreOs, b"nexcore-os-0.1.0"),
        ];

        for &(stage, data) in &stages {
            chain.register_expected(stage, Measurement::from_data(data));
        }

        // Simulate boot — all stages match
        for &(stage, data) in &stages {
            let result = chain.measure(stage, data, format!("{} binary", stage.name()));
            assert_eq!(result, VerifyResult::Ok, "Stage {} should verify", stage);
        }

        // Full chain verified
        let verification = chain.verify_chain();
        assert!(verification.all_verified);
        assert_eq!(chain.record_count(), 4);
        assert_eq!(chain.failure_count(), 0);

        // Quote shows all 4 PCRs
        let quote = chain.quote();
        assert_eq!(quote.pcr_values.len(), 4);
        assert!(!quote.degraded);
    }

    #[test]
    fn verify_result_properties() {
        assert!(VerifyResult::Ok.is_ok());
        assert!(!VerifyResult::Ok.is_violation());

        assert!(VerifyResult::NoExpectation.is_ok());
        assert!(!VerifyResult::NoExpectation.is_violation());

        assert!(!VerifyResult::NotMeasured.is_ok());
        assert!(!VerifyResult::NotMeasured.is_violation());

        let mismatch = VerifyResult::Mismatch {
            expected: Measurement::from_data(b"a"),
            actual: Measurement::from_data(b"b"),
        };
        assert!(!mismatch.is_ok());
        assert!(mismatch.is_violation());
    }
}
