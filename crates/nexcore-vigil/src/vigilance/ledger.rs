//! # Vigilance Ledger — π (Persistence) Layer
//!
//! Append-only, SHA-256 hash-chained ledger for tamper-evident audit trails.
//! Every event, boundary check, and consequence is recorded BEFORE execution.
//!
//! ## Tier: T3 (π + ∝ + σ + ∂)
//! The ledger is the persistence backbone of the entire vigilance system.

use crate::vigilance::error::{VigilError, VigilResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Types of entries that can appear in the ledger.
///
/// Tier: T2-P (Σ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerEntryType {
    EventObserved,
    BoundaryViolation,
    ConsequenceScheduled,
    ConsequenceExecuted,
    ConsequenceFailed,
    DaemonStarted,
    DaemonStopped,
}

impl std::fmt::Display for LedgerEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventObserved => write!(f, "event_observed"),
            Self::BoundaryViolation => write!(f, "boundary_violation"),
            Self::ConsequenceScheduled => write!(f, "consequence_scheduled"),
            Self::ConsequenceExecuted => write!(f, "consequence_executed"),
            Self::ConsequenceFailed => write!(f, "consequence_failed"),
            Self::DaemonStarted => write!(f, "daemon_started"),
            Self::DaemonStopped => write!(f, "daemon_stopped"),
        }
    }
}

/// A single entry in the vigilance ledger.
///
/// Each entry's hash includes the previous entry's hash, creating a
/// tamper-evident chain (blockchain-lite without the bloat).
///
/// Tier: T2-P (π)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Monotonically increasing sequence number
    pub sequence: u64,
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Type of this entry
    pub entry_type: LedgerEntryType,
    /// Structured payload
    pub payload: serde_json::Value,
    /// Hash of the previous entry (zeroes for genesis)
    pub prev_hash: [u8; 32],
    /// SHA-256(sequence || timestamp || entry_type || payload || prev_hash)
    pub hash: [u8; 32],
}

impl LedgerEntry {
    /// Compute the hash for this entry.
    fn compute_hash(
        sequence: u64,
        timestamp: u64,
        entry_type: &LedgerEntryType,
        payload: &serde_json::Value,
        prev_hash: &[u8; 32],
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(sequence.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.update(entry_type.to_string().as_bytes());
        // Use compact JSON for deterministic hashing
        let payload_bytes = serde_json::to_vec(payload).unwrap_or_default();
        hasher.update(&payload_bytes);
        hasher.update(prev_hash);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Verify this entry's hash is correct given its fields.
    pub fn verify(&self) -> bool {
        let expected = Self::compute_hash(
            self.sequence,
            self.timestamp,
            &self.entry_type,
            &self.payload,
            &self.prev_hash,
        );
        self.hash == expected
    }
}

/// Filter for querying ledger entries.
#[derive(Debug, Default)]
pub struct LedgerQuery {
    /// Filter by entry type
    pub entry_type: Option<LedgerEntryType>,
    /// Only entries after this timestamp
    pub since: Option<u64>,
    /// Maximum number of entries to return
    pub limit: Option<usize>,
}

/// Append-only, content-addressed vigilance ledger.
///
/// Tier: T3 (π + ∝ + σ + ∂), dominant π
pub struct VigilanceLedger {
    entries: Vec<LedgerEntry>,
    wal_path: PathBuf,
    head_hash: [u8; 32],
    sequence: u64,
}

impl VigilanceLedger {
    /// Create a new empty ledger with the given WAL path.
    pub fn new(wal_path: PathBuf) -> Self {
        Self {
            entries: Vec::new(),
            wal_path,
            head_hash: [0u8; 32],
            sequence: 0,
        }
    }

    /// Append a new entry to the ledger.
    ///
    /// Returns a reference to the newly appended entry.
    pub fn append(
        &mut self,
        entry_type: LedgerEntryType,
        payload: serde_json::Value,
    ) -> VigilResult<&LedgerEntry> {
        let timestamp = crate::vigilance::event::now_millis();
        let hash = LedgerEntry::compute_hash(
            self.sequence,
            timestamp,
            &entry_type,
            &payload,
            &self.head_hash,
        );

        let entry = LedgerEntry {
            sequence: self.sequence,
            timestamp,
            entry_type,
            payload,
            prev_hash: self.head_hash,
            hash,
        };

        self.head_hash = hash;
        self.sequence += 1;
        self.entries.push(entry);

        // WAL append
        self.write_wal_entry(self.entries.last().ok_or_else(|| {
            VigilError::LedgerIntegrity("append succeeded but entries is empty".to_string())
        })?)?;

        // Return reference to last entry
        self.entries.last().ok_or_else(|| {
            VigilError::LedgerIntegrity("append succeeded but entries is empty".to_string())
        })
    }

    /// Verify the entire hash chain from genesis to head.
    ///
    /// Returns true if every entry's hash matches its content and
    /// links correctly to the previous entry.
    pub fn verify_chain(&self) -> VigilResult<bool> {
        let mut expected_prev = [0u8; 32];

        for entry in &self.entries {
            // Check prev_hash links
            if entry.prev_hash != expected_prev {
                return Ok(false);
            }
            // Check hash integrity
            if !entry.verify() {
                return Ok(false);
            }
            expected_prev = entry.hash;
        }

        Ok(true)
    }

    /// Query ledger entries with optional filters.
    pub fn query(&self, filter: &LedgerQuery) -> Vec<&LedgerEntry> {
        let mut results: Vec<&LedgerEntry> = self
            .entries
            .iter()
            .filter(|e| {
                if let Some(ref et) = filter.entry_type {
                    if e.entry_type != *et {
                        return false;
                    }
                }
                if let Some(since) = filter.since {
                    if e.timestamp < since {
                        return false;
                    }
                }
                true
            })
            .collect();

        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    /// Write a single entry to the WAL file.
    fn write_wal_entry(&self, entry: &LedgerEntry) -> VigilResult<()> {
        // Create parent directory if needed
        if let Some(parent) = self.wal_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)?;

        let line = serde_json::to_string(entry)?;
        writeln!(file, "{line}")?;
        file.flush()?;

        Ok(())
    }

    /// Flush the entire ledger to the WAL (rewrite).
    pub fn flush_wal(&self) -> VigilResult<()> {
        if let Some(parent) = self.wal_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut file = std::fs::File::create(&self.wal_path)?;
        for entry in &self.entries {
            let line = serde_json::to_string(entry)?;
            writeln!(file, "{line}")?;
        }
        file.flush()?;
        Ok(())
    }

    /// Recover a ledger from an existing WAL file.
    pub fn recover_from_wal(path: &Path) -> VigilResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let entry: LedgerEntry = serde_json::from_str(line)
                .map_err(|e| VigilError::WalRecovery(format!("Failed to parse WAL line: {e}")))?;
            entries.push(entry);
        }

        let (head_hash, sequence) = if let Some(last) = entries.last() {
            (last.hash, last.sequence + 1)
        } else {
            ([0u8; 32], 0)
        };

        let ledger = Self {
            entries,
            wal_path: path.to_path_buf(),
            head_hash,
            sequence,
        };

        // Verify the recovered chain
        if !ledger.verify_chain()? {
            return Err(VigilError::WalRecovery(
                "Recovered ledger failed chain verification".to_string(),
            ));
        }

        Ok(ledger)
    }

    /// Number of entries in the ledger.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the ledger is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Current head hash.
    pub fn head_hash(&self) -> &[u8; 32] {
        &self.head_hash
    }

    /// Current sequence number (next to be assigned).
    pub fn next_sequence(&self) -> u64 {
        self.sequence
    }

    /// Get all entries (for serialization).
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_wal() -> PathBuf {
        let dir = tempfile::tempdir().ok();
        dir.map(|d| d.keep().join("test-ledger.wal"))
            .unwrap_or_else(|| PathBuf::from("/tmp/vigil-test-ledger.wal"))
    }

    #[test]
    fn empty_ledger_verifies() {
        let ledger = VigilanceLedger::new(temp_wal());
        assert!(ledger.verify_chain().unwrap_or(false));
        assert!(ledger.is_empty());
    }

    #[test]
    fn append_and_verify() {
        let mut ledger = VigilanceLedger::new(temp_wal());

        let result = ledger.append(
            LedgerEntryType::DaemonStarted,
            serde_json::json!({"version": "1.0"}),
        );
        assert!(result.is_ok());
        assert_eq!(ledger.len(), 1);
        assert!(ledger.verify_chain().unwrap_or(false));
    }

    #[test]
    fn multi_entry_chain_integrity() {
        let mut ledger = VigilanceLedger::new(temp_wal());

        for i in 0..10 {
            let result = ledger.append(
                LedgerEntryType::EventObserved,
                serde_json::json!({"index": i}),
            );
            assert!(result.is_ok());
        }

        assert_eq!(ledger.len(), 10);
        assert!(ledger.verify_chain().unwrap_or(false));
    }

    #[test]
    fn tampered_entry_detected() {
        let mut ledger = VigilanceLedger::new(temp_wal());

        let _ = ledger.append(
            LedgerEntryType::DaemonStarted,
            serde_json::json!({"version": "1.0"}),
        );
        let _ = ledger.append(
            LedgerEntryType::EventObserved,
            serde_json::json!({"event": "test"}),
        );

        // Tamper with the first entry's payload
        if let Some(entry) = ledger.entries.get_mut(0) {
            entry.payload = serde_json::json!({"version": "TAMPERED"});
        }

        assert!(!ledger.verify_chain().unwrap_or(true));
    }

    #[test]
    fn query_by_type() {
        let mut ledger = VigilanceLedger::new(temp_wal());

        let _ = ledger.append(LedgerEntryType::DaemonStarted, serde_json::json!({}));
        let _ = ledger.append(LedgerEntryType::EventObserved, serde_json::json!({}));
        let _ = ledger.append(LedgerEntryType::EventObserved, serde_json::json!({}));
        let _ = ledger.append(LedgerEntryType::BoundaryViolation, serde_json::json!({}));

        let query = LedgerQuery {
            entry_type: Some(LedgerEntryType::EventObserved),
            ..Default::default()
        };
        let results = ledger.query(&query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_with_limit() {
        let mut ledger = VigilanceLedger::new(temp_wal());

        for _ in 0..5 {
            let _ = ledger.append(LedgerEntryType::EventObserved, serde_json::json!({}));
        }

        let query = LedgerQuery {
            limit: Some(3),
            ..Default::default()
        };
        let results = ledger.query(&query);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn wal_recovery() {
        let wal = temp_wal();
        let mut ledger = VigilanceLedger::new(wal.clone());

        let _ = ledger.append(LedgerEntryType::DaemonStarted, serde_json::json!({"v": 1}));
        let _ = ledger.append(LedgerEntryType::EventObserved, serde_json::json!({"e": 1}));
        let _ = ledger.append(LedgerEntryType::EventObserved, serde_json::json!({"e": 2}));

        // Recover from WAL
        let recovered = VigilanceLedger::recover_from_wal(&wal);
        assert!(recovered.is_ok());
        let recovered = recovered.unwrap_or_else(|_| VigilanceLedger::new(wal));
        assert_eq!(recovered.len(), 3);
        assert!(recovered.verify_chain().unwrap_or(false));
    }

    #[test]
    fn entry_individual_verify() {
        let mut ledger = VigilanceLedger::new(temp_wal());
        let _ = ledger.append(LedgerEntryType::DaemonStarted, serde_json::json!({}));

        let entry = &ledger.entries()[0];
        assert!(entry.verify());
    }

    #[test]
    fn head_hash_updates() {
        let mut ledger = VigilanceLedger::new(temp_wal());
        let initial = *ledger.head_hash();
        assert_eq!(initial, [0u8; 32]);

        let _ = ledger.append(LedgerEntryType::DaemonStarted, serde_json::json!({}));
        assert_ne!(*ledger.head_hash(), [0u8; 32]);
    }
}
