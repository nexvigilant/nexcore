//! # Bathroom Lock — Visible-State File Advisory Lock
//!
//! Advisory file lock with observable Vacant/Occupied state.
//! Uses atomic `.lock` sidecar files to prevent write race conditions.
//!
//! Named for the bathroom door lock — visible state indicator
//! before attempting entry. Peek at the door, don't rattle the handle.
//!
//! ## T1 Grounding
//!
//! - `ς` (State): Binary state machine (Vacant ↔ Occupied)
//! - `∂` (Boundary): Mutually-exclusive write access
//! - `∃` (Existence): Lock file existence = occupancy signal
//!
//! ## Tier: T2-P (ς + ∂ + ∃)
//!
//! ## Domain Mappings
//!
//! | Domain | Lock Target | Holder |
//! |--------|-------------|--------|
//! | Brain | artifact files | session ID |
//! | Telemetry | signals.jsonl | agent ID |
//! | Guardian | actuator state | iteration number |
//! | Hooks | Cargo.lock | hook binary name |

use nexcore_fs::dirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Cached home directory — avoids repeated env var lookups in hot paths.
/// `dirs::home_dir()` queries `$HOME` each call; 175+ hooks × 10 factory
/// methods = 1750 redundant env reads per session without this cache.
fn cached_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

// ============================================================================
// Occupancy — Observable Lock State (T2-P: ς + ∂ + ∃)
// ============================================================================

/// Observable lock state: Vacant or Occupied.
///
/// The bathroom door indicator — visible without touching the handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Occupancy {
    /// Lock is free — resource available for writing.
    Vacant,
    /// Lock is held — resource is being written to.
    Occupied {
        /// Identity of the lock holder.
        holder: String,
    },
}

impl Occupancy {
    /// Returns true if the lock is available.
    #[must_use]
    pub const fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant)
    }

    /// Returns true if the lock is held.
    #[must_use]
    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied { .. })
    }

    /// Returns the holder identity if occupied.
    #[must_use]
    pub fn holder(&self) -> Option<&str> {
        match self {
            Self::Occupied { holder } => Some(holder),
            Self::Vacant => None,
        }
    }
}

impl fmt::Display for Occupancy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vacant => write!(f, "VACANT"),
            Self::Occupied { holder } => write!(f, "OCCUPIED({holder})"),
        }
    }
}

// ============================================================================
// LockError — Diagnostic Error Types
// ============================================================================

/// Error during lock operations.
#[derive(Debug)]
pub enum LockError {
    /// Lock is already held by another holder.
    AlreadyOccupied {
        /// Identity of the current holder.
        holder: String,
    },
    /// Lock is not held — cannot release what is vacant.
    AlreadyVacant,
    /// Holder identity mismatch on release attempt.
    NotYourLock {
        /// Who tried to release.
        expected: String,
        /// Who actually holds the lock.
        actual: String,
    },
    /// Filesystem I/O error.
    Io(io::Error),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyOccupied { holder } => write!(f, "lock occupied by: {holder}"),
            Self::AlreadyVacant => write!(f, "lock already vacant"),
            Self::NotYourLock { expected, actual } => {
                write!(f, "lock held by {actual}, not {expected}")
            }
            Self::Io(e) => write!(f, "lock I/O error: {e}"),
        }
    }
}

impl std::error::Error for LockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for LockError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

// ============================================================================
// BathroomLock — File Advisory Lock
// ============================================================================

/// Advisory file lock with visible Vacant/Occupied state.
///
/// Creates a `.lock` sidecar file next to the target path.
/// Acquisition is atomic via `create_new` (O_CREAT|O_EXCL on Unix,
/// CREATE_NEW on Windows).
///
/// # Usage
///
/// ```no_run
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use nexcore_constants::bathroom_lock::BathroomLock;
///
/// let lock = BathroomLock::new("/tmp/data.json");
///
/// // Peek at the door
/// if lock.is_vacant() {
///     // RAII guard: auto-releases on drop
///     let _guard = lock.try_acquire("writer-1")?;
///     // ... safe to write to data.json ...
/// } // guard dropped → lock released
/// # Ok(())
/// # }
/// ```
pub struct BathroomLock {
    /// Path to the protected resource.
    target: PathBuf,
    /// Path to the `.lock` sidecar file.
    lock_path: PathBuf,
}

impl BathroomLock {
    /// Creates a new bathroom lock for the given file path.
    ///
    /// The lock file will be `<path>.lock` (e.g., `data.json.lock`).
    #[must_use]
    pub fn new(target: impl AsRef<Path>) -> Self {
        let target = target.as_ref().to_path_buf();
        let mut lock_name = target.as_os_str().to_os_string();
        lock_name.push(".lock");
        let lock_path = PathBuf::from(lock_name);
        Self { target, lock_path }
    }

    /// Peek at the lock state without side effects.
    #[must_use]
    pub fn peek(&self) -> Occupancy {
        match fs::read_to_string(&self.lock_path) {
            Ok(holder) => Occupancy::Occupied {
                holder: holder.trim().to_string(),
            },
            Err(_) => Occupancy::Vacant,
        }
    }

    /// Returns true if the lock is available (no lock file exists).
    #[inline]
    #[must_use]
    pub fn is_vacant(&self) -> bool {
        !self.lock_path.exists()
    }

    /// Returns true if the lock is held (lock file exists).
    #[inline]
    #[must_use]
    pub fn is_occupied(&self) -> bool {
        self.lock_path.exists()
    }

    /// Acquire the lock with an RAII guard that auto-releases on drop.
    ///
    /// Uses atomic `create_new` — only one caller can succeed.
    /// Returns `LockError::AlreadyOccupied` if another holder has the lock.
    pub fn try_acquire(&self, holder: &str) -> Result<OccupiedGuard<'_>, LockError> {
        self.acquire_impl(holder)?;
        Ok(OccupiedGuard {
            lock: self,
            holder: holder.to_string(),
            released: false,
        })
    }

    /// Manually acquire the lock (no RAII guard).
    ///
    /// Caller MUST call [`release`](Self::release) when done.
    /// Prefer [`try_acquire`](Self::try_acquire) for automatic cleanup.
    pub fn acquire(&self, holder: &str) -> Result<(), LockError> {
        self.acquire_impl(holder)
    }

    /// Release the lock, verifying holder identity.
    ///
    /// Returns `LockError::NotYourLock` if the holder doesn't match.
    /// Returns `LockError::AlreadyVacant` if no lock is held.
    pub fn release(&self, holder: &str) -> Result<(), LockError> {
        match fs::read_to_string(&self.lock_path) {
            Ok(existing) => {
                let existing = existing.trim();
                if existing != holder {
                    return Err(LockError::NotYourLock {
                        expected: holder.to_string(),
                        actual: existing.to_string(),
                    });
                }
                fs::remove_file(&self.lock_path).map_err(LockError::Io)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(LockError::AlreadyVacant),
            Err(e) => Err(LockError::Io(e)),
        }
    }

    /// Force-release the lock regardless of holder identity.
    ///
    /// Use for stale lock recovery when the original holder crashed.
    pub fn force_release(&self) -> Result<(), LockError> {
        match fs::remove_file(&self.lock_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(LockError::AlreadyVacant),
            Err(e) => Err(LockError::Io(e)),
        }
    }

    /// The protected resource path.
    #[must_use]
    pub fn target(&self) -> &Path {
        &self.target
    }

    /// The lock sidecar file path.
    #[must_use]
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    // ========================================================================
    // Well-Known Lock Targets — NexCore Race Condition Hotspots
    // ========================================================================

    /// Lock for any JSONL append-only log file.
    #[must_use]
    pub fn for_jsonl(path: impl AsRef<Path>) -> Self {
        Self::new(path)
    }

    /// Lock for Brain artifact write operations.
    #[must_use]
    pub fn for_artifact(artifact_path: impl AsRef<Path>) -> Self {
        Self::new(artifact_path)
    }

    /// Lock for hook telemetry log.
    ///
    /// Target: `~/.claude/logs/hook_telemetry.jsonl`
    #[must_use]
    pub fn for_hook_telemetry() -> Self {
        Self::new(cached_home().join(".claude/logs/hook_telemetry.jsonl"))
    }

    /// Lock for brain metric snapshots.
    ///
    /// Target: `~/.claude/brain/telemetry/brain_snapshots.jsonl`
    #[must_use]
    pub fn for_brain_snapshots() -> Self {
        Self::new(cached_home().join(".claude/brain/telemetry/brain_snapshots.jsonl"))
    }

    /// Lock for decision audit trail.
    ///
    /// Target: `~/.claude/decision-audit/decisions.jsonl`
    #[must_use]
    pub fn for_decision_audit() -> Self {
        Self::new(cached_home().join(".claude/decision-audit/decisions.jsonl"))
    }

    /// Lock for verification chain.
    ///
    /// Target: `~/.claude/verified/verification_chain.jsonl`
    #[must_use]
    pub fn for_verification_chain() -> Self {
        Self::new(cached_home().join(".claude/verified/verification_chain.jsonl"))
    }

    /// Lock for MCP telemetry.
    ///
    /// Target: `~/.claude/brain/telemetry/mcp_calls.jsonl`
    #[must_use]
    pub fn for_mcp_telemetry() -> Self {
        Self::new(cached_home().join(".claude/brain/telemetry/mcp_calls.jsonl"))
    }

    /// Lock for coordination registry itself.
    ///
    /// Target: `~/.claude/file-locks/registry.json`
    #[must_use]
    pub fn for_coordination_registry() -> Self {
        Self::new(cached_home().join(".claude/file-locks/registry.json"))
    }

    /// Lock for CTVP validation events.
    ///
    /// Target: `~/.claude/brain/ctvp/events.jsonl`
    #[must_use]
    pub fn for_ctvp_events() -> Self {
        Self::new(cached_home().join(".claude/brain/ctvp/events.jsonl"))
    }

    /// Lock for vocabulary proposals.
    ///
    /// Target: `~/.claude/brain/implicit/vocabulary_proposals.jsonl`
    #[must_use]
    pub fn for_vocabulary_proposals() -> Self {
        Self::new(cached_home().join(".claude/brain/implicit/vocabulary_proposals.jsonl"))
    }

    // ========================================================================
    // Batch Operations — Fused lock+write+unlock for hot paths
    // ========================================================================

    /// Atomically append a line to a JSONL file under lock.
    ///
    /// Fuses acquire → append → release into a single call.
    pub fn locked_append(&self, holder: &str, line: &str) -> Result<(), LockError> {
        use std::io::Write;

        let _guard = self.try_acquire(holder)?;

        if let Some(parent) = self.target.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(LockError::Io)?;
            }
        }

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.target)
            .map_err(LockError::Io)?;

        writeln!(file, "{line}").map_err(LockError::Io)?;
        Ok(())
    }

    /// Atomically write full contents to a file under lock.
    ///
    /// Fuses acquire → write → release.
    pub fn locked_write(&self, holder: &str, content: &str) -> Result<(), LockError> {
        let _guard = self.try_acquire(holder)?;
        fs::write(&self.target, content).map_err(LockError::Io)?;
        Ok(())
    }

    // -- Internal --

    fn acquire_impl(&self, holder: &str) -> Result<(), LockError> {
        use std::io::Write;

        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.lock_path)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(holder.as_bytes()) {
                    let _ = fs::remove_file(&self.lock_path);
                    return Err(LockError::Io(e));
                }
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                let existing =
                    fs::read_to_string(&self.lock_path).unwrap_or_else(|_| "unknown".into());
                Err(LockError::AlreadyOccupied {
                    holder: existing.trim().to_string(),
                })
            }
            Err(e) => Err(LockError::Io(e)),
        }
    }
}

impl fmt::Debug for BathroomLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BathroomLock")
            .field("target", &self.target)
            .field("occupancy", &self.peek())
            .finish()
    }
}

impl fmt::Display for BathroomLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ς:{} [{}]", self.peek(), self.target.display())
    }
}

// ============================================================================
// OccupiedGuard — RAII Auto-Release
// ============================================================================

/// RAII guard that auto-releases the lock on drop.
pub struct OccupiedGuard<'a> {
    lock: &'a BathroomLock,
    holder: String,
    released: bool,
}

impl OccupiedGuard<'_> {
    /// The protected resource path.
    #[must_use]
    pub fn target(&self) -> &Path {
        self.lock.target()
    }

    /// The lock holder identity.
    #[must_use]
    pub fn holder(&self) -> &str {
        &self.holder
    }
}

impl Drop for OccupiedGuard<'_> {
    fn drop(&mut self) {
        if !self.released {
            let _ = self.lock.release(&self.holder);
        }
    }
}

impl fmt::Debug for OccupiedGuard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedGuard")
            .field("holder", &self.holder)
            .field("target", &self.lock.target)
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir() -> PathBuf {
        let dir = PathBuf::from("/tmp/claude/bathroom_lock_tests");
        fs::create_dir_all(&dir).ok();
        dir
    }

    fn unique_path(name: &str) -> PathBuf {
        test_dir().join(name)
    }

    fn cleanup(lock: &BathroomLock) {
        let _ = fs::remove_file(lock.lock_path());
        let _ = fs::remove_file(lock.target());
    }

    // -- Occupancy enum --

    #[test]
    fn test_occupancy_vacant() {
        let o = Occupancy::Vacant;
        assert!(o.is_vacant());
        assert!(!o.is_occupied());
        assert!(o.holder().is_none());
    }

    #[test]
    fn test_occupancy_occupied() {
        let o = Occupancy::Occupied {
            holder: "agent-1".into(),
        };
        assert!(!o.is_vacant());
        assert!(o.is_occupied());
        assert_eq!(o.holder(), Some("agent-1"));
    }

    #[test]
    fn test_occupancy_display() {
        assert_eq!(format!("{}", Occupancy::Vacant), "VACANT");
        let occ = Occupancy::Occupied {
            holder: "me".into(),
        };
        assert_eq!(format!("{occ}"), "OCCUPIED(me)");
    }

    #[test]
    fn test_occupancy_serialize_roundtrip() {
        let cases = vec![
            Occupancy::Vacant,
            Occupancy::Occupied {
                holder: "test".into(),
            },
        ];
        for case in cases {
            let json = serde_json::to_string(&case);
            assert!(json.is_ok());
            if let Ok(json) = json {
                let parsed: Result<Occupancy, _> = serde_json::from_str(&json);
                assert!(parsed.is_ok());
                if let Ok(parsed) = parsed {
                    assert_eq!(parsed, case);
                }
            }
        }
    }

    // -- BathroomLock --

    #[test]
    fn test_lock_path_construction() {
        let lock = BathroomLock::new("/tmp/data.json");
        assert_eq!(lock.target(), Path::new("/tmp/data.json"));
        assert_eq!(lock.lock_path(), Path::new("/tmp/data.json.lock"));
    }

    #[test]
    fn test_lock_path_no_extension() {
        let lock = BathroomLock::new("/tmp/myfile");
        assert_eq!(lock.lock_path(), Path::new("/tmp/myfile.lock"));
    }

    #[test]
    fn test_lock_starts_vacant() {
        let path = unique_path("const_starts_vacant.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);
        assert!(lock.is_vacant());
        assert!(!lock.is_occupied());
        assert!(lock.peek().is_vacant());
    }

    #[test]
    fn test_lock_acquire_release() {
        let path = unique_path("const_acq_rel.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let result = lock.acquire("writer-1");
        assert!(result.is_ok());
        assert!(lock.is_occupied());

        let peek = lock.peek();
        assert_eq!(
            peek,
            Occupancy::Occupied {
                holder: "writer-1".into()
            }
        );

        let result = lock.release("writer-1");
        assert!(result.is_ok());
        assert!(lock.is_vacant());

        cleanup(&lock);
    }

    #[test]
    fn test_lock_double_acquire_fails() {
        let path = unique_path("const_double_acq.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let first = lock.acquire("holder-a");
        assert!(first.is_ok());

        let second = lock.acquire("holder-b");
        assert!(second.is_err());
        if let Err(LockError::AlreadyOccupied { holder }) = second {
            assert_eq!(holder, "holder-a");
        }

        let _ = lock.force_release();
        cleanup(&lock);
    }

    #[test]
    fn test_lock_release_wrong_holder() {
        let path = unique_path("const_wrong_holder.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let _ = lock.acquire("real-owner");
        let result = lock.release("imposter");
        assert!(result.is_err());
        if let Err(LockError::NotYourLock { expected, actual }) = result {
            assert_eq!(expected, "imposter");
            assert_eq!(actual, "real-owner");
        }

        let _ = lock.force_release();
        cleanup(&lock);
    }

    #[test]
    fn test_lock_release_when_vacant() {
        let path = unique_path("const_rel_vacant.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let result = lock.release("nobody");
        assert!(result.is_err());
        assert!(matches!(result, Err(LockError::AlreadyVacant)));
    }

    #[test]
    fn test_lock_force_release() {
        let path = unique_path("const_force_rel.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let _ = lock.acquire("stale-holder");
        assert!(lock.is_occupied());

        let result = lock.force_release();
        assert!(result.is_ok());
        assert!(lock.is_vacant());

        cleanup(&lock);
    }

    #[test]
    fn test_lock_force_release_when_vacant() {
        let path = unique_path("const_force_vacant.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let result = lock.force_release();
        assert!(matches!(result, Err(LockError::AlreadyVacant)));
    }

    // -- OccupiedGuard (RAII) --

    #[test]
    fn test_guard_auto_release() {
        let path = unique_path("const_guard_auto.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        {
            let guard = lock.try_acquire("raii-holder");
            assert!(guard.is_ok());
            if let Ok(guard) = &guard {
                assert_eq!(guard.holder(), "raii-holder");
                assert_eq!(guard.target(), path.as_path());
            }
            assert!(lock.is_occupied());
        }

        assert!(lock.is_vacant());
        cleanup(&lock);
    }

    #[test]
    fn test_guard_prevents_double_acquire() {
        let path = unique_path("const_guard_double.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let guard = lock.try_acquire("first");
        assert!(guard.is_ok());

        let second = lock.try_acquire("second");
        assert!(second.is_err());

        drop(guard);
        let third = lock.try_acquire("third");
        assert!(third.is_ok());

        cleanup(&lock);
    }

    #[test]
    fn test_lock_display() {
        let path = unique_path("const_display.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let display = format!("{lock}");
        assert!(display.contains("VACANT"));

        let _ = lock.acquire("tester");
        let display = format!("{lock}");
        assert!(display.contains("OCCUPIED(tester)"));

        let _ = lock.force_release();
        cleanup(&lock);
    }

    // -- Well-Known Constructors --

    #[test]
    fn test_well_known_hook_telemetry() {
        let lock = BathroomLock::for_hook_telemetry();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with(".claude/logs/hook_telemetry.jsonl"));
        assert!(lock.lock_path().to_string_lossy().ends_with(".jsonl.lock"));
    }

    #[test]
    fn test_well_known_brain_snapshots() {
        let lock = BathroomLock::for_brain_snapshots();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with("brain/telemetry/brain_snapshots.jsonl"));
    }

    #[test]
    fn test_well_known_decision_audit() {
        let lock = BathroomLock::for_decision_audit();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with("decision-audit/decisions.jsonl"));
    }

    #[test]
    fn test_well_known_verification_chain() {
        let lock = BathroomLock::for_verification_chain();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with("verified/verification_chain.jsonl"));
    }

    #[test]
    fn test_well_known_coordination_registry() {
        let lock = BathroomLock::for_coordination_registry();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with("file-locks/registry.json"));
    }

    #[test]
    fn test_well_known_mcp_telemetry() {
        let lock = BathroomLock::for_mcp_telemetry();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with("brain/telemetry/mcp_calls.jsonl"));
    }

    #[test]
    fn test_well_known_ctvp_events() {
        let lock = BathroomLock::for_ctvp_events();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with("brain/ctvp/events.jsonl"));
    }

    #[test]
    fn test_well_known_vocabulary_proposals() {
        let lock = BathroomLock::for_vocabulary_proposals();
        let target = lock.target().to_string_lossy();
        assert!(target.ends_with("implicit/vocabulary_proposals.jsonl"));
    }

    #[test]
    fn test_for_artifact_factory() {
        let lock = BathroomLock::for_artifact("/tmp/brain/sessions/task.md");
        assert_eq!(lock.target(), Path::new("/tmp/brain/sessions/task.md"));
        assert_eq!(
            lock.lock_path(),
            Path::new("/tmp/brain/sessions/task.md.lock")
        );
    }

    #[test]
    fn test_for_jsonl_factory() {
        let lock = BathroomLock::for_jsonl("/var/log/signals.jsonl");
        assert_eq!(lock.target(), Path::new("/var/log/signals.jsonl"));
    }

    // -- Batch Operations --

    #[test]
    fn test_locked_append() {
        let path = unique_path("const_locked_append.jsonl");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let r1 = lock.locked_append("hook-1", r#"{"event":"start"}"#);
        assert!(r1.is_ok());
        let r2 = lock.locked_append("hook-2", r#"{"event":"stop"}"#);
        assert!(r2.is_ok());

        let content = fs::read_to_string(&path);
        assert!(content.is_ok());
        if let Ok(content) = content {
            let lines: Vec<&str> = content.lines().collect();
            assert_eq!(lines.len(), 2);
            assert!(lines[0].contains("start"));
            assert!(lines[1].contains("stop"));
        }

        assert!(lock.is_vacant());
        cleanup(&lock);
    }

    #[test]
    fn test_locked_write() {
        let path = unique_path("const_locked_write.json");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let r = lock.locked_write("writer-1", r#"{"version":1}"#);
        assert!(r.is_ok());

        let content = fs::read_to_string(&path);
        assert!(content.is_ok());
        if let Ok(content) = content {
            assert!(content.contains("version"));
        }

        assert!(lock.is_vacant());
        cleanup(&lock);
    }

    #[test]
    fn test_locked_append_blocked_when_occupied() {
        let path = unique_path("const_append_blocked.jsonl");
        let lock = BathroomLock::new(&path);
        cleanup(&lock);

        let _ = lock.acquire("blocker");
        let r = lock.locked_append("hook-1", "data");
        assert!(r.is_err());

        let _ = lock.force_release();
        cleanup(&lock);
    }

    // -- Error Display --

    #[test]
    fn test_lock_error_display() {
        let e = LockError::AlreadyOccupied {
            holder: "bob".into(),
        };
        assert_eq!(format!("{e}"), "lock occupied by: bob");

        let e = LockError::AlreadyVacant;
        assert_eq!(format!("{e}"), "lock already vacant");

        let e = LockError::NotYourLock {
            expected: "alice".into(),
            actual: "bob".into(),
        };
        assert_eq!(format!("{e}"), "lock held by bob, not alice");
    }
}
