//! Binary file watcher with debounce.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | Last-change timestamp for debounce |
//! | T1: Sequence (σ) | Event stream → debounced reload signal |

use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::debounce::Debouncer;
use crate::error::{ProxyError, Result};

/// Tier: T2-C — Watches a binary file and emits debounced reload signals.
pub struct BinaryWatcher {
    path: PathBuf,
    debounce: Duration,
}

// ── Pure helpers ──────────────────────────────────────────────────

/// Check if a notify event is a relevant file modification.
fn is_binary_change(event: &Event, path: &Path) -> bool {
    let touches = event.paths.iter().any(|p| p == path);
    let modifies = matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    );
    touches && modifies
}

/// Create a notify watcher that sends raw events to a channel.
fn create_notify_watcher(
    tx: std_mpsc::Sender<std::result::Result<Event, notify::Error>>,
) -> Result<RecommendedWatcher> {
    Watcher::new(tx, notify::Config::default())
        .map_err(|e| ProxyError::Watcher(format!("create: {e}")))
}

/// Get the parent directory of a path (for watching).
fn watch_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(p) if p.as_os_str().is_empty() => Path::new("."),
        Some(p) => p,
        None => Path::new("."),
    }
}

/// Check if a raw notify result contains a relevant binary change.
fn is_relevant_change(result: &std::result::Result<Event, notify::Error>, path: &Path) -> bool {
    match result {
        Ok(e) => is_binary_change(e, path),
        Err(e) => {
            tracing::warn!("Watch error: {e}");
            false
        }
    }
}

// ── BinaryWatcher impl ──────────────────────────────────────────

impl BinaryWatcher {
    /// Create a watcher for the given binary path.
    pub fn new(path: &Path, debounce: Duration) -> Self {
        Self {
            path: path.to_path_buf(),
            debounce,
        }
    }

    /// Start watching. Returns a receiver that emits `()` on debounced changes.
    pub fn start(&self) -> Result<mpsc::Receiver<()>> {
        let (raw_tx, raw_rx) = std_mpsc::channel();
        let mut watcher = create_notify_watcher(raw_tx)?;

        let dir = watch_dir(&self.path);
        watcher
            .watch(dir, RecursiveMode::NonRecursive)
            .map_err(|e| ProxyError::Watcher(format!("watch {}: {e}", dir.display())))?;

        let (debounced_tx, debounced_rx) = mpsc::channel(16);
        let path = self.path.clone();
        let debounce = self.debounce;

        // Use spawn_blocking because std_mpsc::Receiver is !Sync.
        tokio::task::spawn_blocking(move || {
            let _watcher = watcher; // keep alive
            let predicate =
                move |r: &std::result::Result<Event, notify::Error>| is_relevant_change(r, &path);
            let debouncer = Debouncer::new(raw_rx, debounced_tx, predicate, debounce);
            debouncer.run();
        });

        tracing::info!(
            "Watching binary: {} (debounce: {}s)",
            self.path.display(),
            self.debounce.as_secs()
        );
        Ok(debounced_rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, ModifyKind};

    fn make_event(kind: EventKind, path: &Path) -> Event {
        Event {
            kind,
            paths: vec![path.to_path_buf()],
            attrs: Default::default(),
        }
    }

    #[test]
    fn detects_modify_event() {
        let path = Path::new("/usr/bin/test");
        let event = make_event(EventKind::Modify(ModifyKind::Any), path);
        assert!(is_binary_change(&event, path));
    }

    #[test]
    fn detects_create_event() {
        let path = Path::new("/usr/bin/test");
        let event = make_event(EventKind::Create(CreateKind::Any), path);
        assert!(is_binary_change(&event, path));
    }

    #[test]
    fn ignores_wrong_path() {
        let watched = Path::new("/usr/bin/test");
        let other = Path::new("/usr/bin/other");
        let event = make_event(EventKind::Modify(ModifyKind::Any), other);
        assert!(!is_binary_change(&event, watched));
    }

    #[test]
    fn ignores_access_event() {
        let path = Path::new("/usr/bin/test");
        let event = make_event(EventKind::Access(notify::event::AccessKind::Any), path);
        assert!(!is_binary_change(&event, path));
    }

    #[test]
    fn watch_dir_returns_parent() {
        assert_eq!(watch_dir(Path::new("/usr/bin/test")), Path::new("/usr/bin"));
    }

    #[test]
    fn watch_dir_bare_filename() {
        // "test" has empty parent, watch_dir normalizes to "."
        assert_eq!(watch_dir(Path::new("test")), Path::new("."));
    }
}
