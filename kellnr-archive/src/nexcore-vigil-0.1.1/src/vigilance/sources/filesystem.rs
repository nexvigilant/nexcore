//! # Filesystem Source — ν (Frequency) + ∂ (Boundary)
//!
//! A WatchSource that uses the `notify` crate to detect file system changes.
//! Wraps notify events into WatchEvents for the vigilance pipeline.

use crate::vigilance::error::{VigilError, VigilResult};
use crate::vigilance::event::{EventKind, EventSeverity, WatchEvent};
use crate::vigilance::watcher::WatchSource;
use notify::{Event as NotifyEvent, EventKind as NotifyEventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc as std_mpsc};
use std::time::Duration;

/// A WatchSource backed by the notify file watcher.
///
/// Uses `Arc<Mutex<Receiver>>` to satisfy `Sync` bound on WatchSource.
///
/// Tier: T2-P (ν + ∂)
pub struct FileSystemSource {
    name: String,
    rx: Arc<Mutex<std_mpsc::Receiver<NotifyEvent>>>,
    _watcher: notify::RecommendedWatcher,
    event_counter: u64,
    frequency: Duration,
}

impl FileSystemSource {
    /// Create a new filesystem source watching the given paths.
    pub fn new(
        name: impl Into<String>,
        paths: &[PathBuf],
        recursive: bool,
        frequency: Duration,
    ) -> VigilResult<Self> {
        let (tx, rx) = std_mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res: Result<NotifyEvent, _>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .map_err(|e| VigilError::Watcher {
            source_name: "filesystem".to_string(),
            message: format!("Failed to create watcher: {e}"),
        })?;

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        for path in paths {
            watcher.watch(path, mode).map_err(|e| VigilError::Watcher {
                source_name: "filesystem".to_string(),
                message: format!("Failed to watch {}: {e}", path.display()),
            })?;
        }

        Ok(Self {
            name: name.into(),
            rx: Arc::new(Mutex::new(rx)),
            _watcher: watcher,
            event_counter: 0,
            frequency,
        })
    }

    /// Map a notify event kind to our severity level.
    fn severity_for(kind: &NotifyEventKind) -> EventSeverity {
        match kind {
            NotifyEventKind::Create(_) => EventSeverity::Low,
            NotifyEventKind::Modify(_) => EventSeverity::Low,
            NotifyEventKind::Remove(_) => EventSeverity::Medium,
            NotifyEventKind::Access(_) => EventSeverity::Info,
            _ => EventSeverity::Info,
        }
    }
}

impl WatchSource for FileSystemSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn frequency(&self) -> Duration {
        self.frequency
    }

    fn poll(&mut self) -> VigilResult<Vec<WatchEvent>> {
        let mut events = Vec::new();

        let rx = self.rx.lock().map_err(|e| VigilError::Watcher {
            source_name: self.name.clone(),
            message: format!("Receiver lock failed: {e}"),
        })?;

        // Drain all buffered notify events
        while let Ok(notify_event) = rx.try_recv() {
            self.event_counter += 1;
            let paths: Vec<String> = notify_event
                .paths
                .iter()
                .map(|p| p.display().to_string())
                .collect();

            events.push(WatchEvent::new(
                self.event_counter,
                &self.name,
                EventKind::FileChange,
                Self::severity_for(&notify_event.kind),
                serde_json::json!({
                    "kind": format!("{:?}", notify_event.kind),
                    "paths": paths,
                }),
            ));
        }

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filesystem_source_watches_temp_dir() {
        let dir = tempfile::tempdir();
        if dir.is_err() {
            return; // Skip in environments without tmpdir
        }
        let dir = dir.unwrap_or_else(|_| {
            // This branch won't execute due to the early return
            tempfile::tempdir().unwrap_or_else(|_| panic!("test needs tmpdir"))
        });

        let source = FileSystemSource::new(
            "test-fs",
            &[dir.path().to_path_buf()],
            false,
            Duration::from_millis(100),
        );
        assert!(source.is_ok());
        let source = source.unwrap_or_else(|_| panic!("test needs filesystem source"));
        assert_eq!(source.name(), "test-fs");
    }

    #[test]
    fn severity_mapping() {
        assert_eq!(
            FileSystemSource::severity_for(&NotifyEventKind::Create(
                notify::event::CreateKind::File
            )),
            EventSeverity::Low
        );
        assert_eq!(
            FileSystemSource::severity_for(&NotifyEventKind::Remove(
                notify::event::RemoveKind::File
            )),
            EventSeverity::Medium
        );
    }
}
