use crate::events::EventBus;
use crate::models::{Event, Urgency};
use crate::sources::Source;
use async_trait::async_trait;
use notify::{EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::info;

pub struct FilesystemSource {
    bus: EventBus,
    watch_paths: Vec<PathBuf>,
}

impl FilesystemSource {
    pub fn new(bus: EventBus, watch_paths: Vec<PathBuf>) -> Self {
        Self { bus, watch_paths }
    }
}

#[async_trait]
impl Source for FilesystemSource {
    fn name(&self) -> &'static str {
        "filesystem"
    }

    async fn run(&self) -> nexcore_error::Result<()> {
        info!(paths = ?self.watch_paths, "filesystem_source_starting");

        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            })?;

        for path in &self.watch_paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }

        while let Some(fs_event) = rx.recv().await {
            if let EventKind::Modify(_) = fs_event.kind {
                for path in fs_event.paths {
                    let event = Event {
                        source: self.name().to_string(),
                        event_type: "file_changed".to_string(),
                        payload: serde_json::json!({
                            "path": path.to_string_lossy(),
                        }),
                        priority: Urgency::Normal,
                        ..Event::default()
                    };
                    self.bus.emit(event).await;
                }
            }
        }

        Ok(())
    }
}
