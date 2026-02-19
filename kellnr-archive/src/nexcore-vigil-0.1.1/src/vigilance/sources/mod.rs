//! # Watch Sources — ν (Frequency) Implementations
//!
//! Concrete WatchSource implementations:
//! - `TimerSource` — interval-based periodic watching
//! - `ChannelSource` — tokio mpsc receiver bridge
//! - `FileSystemSource` — notify-based file change detection

pub mod channel;
pub mod filesystem;
pub mod friday_bridge;
pub mod timer;

pub use channel::ChannelSource;
pub use filesystem::FileSystemSource;
pub use friday_bridge::{friday_to_watch_event, map_event_type, map_priority};
pub use timer::TimerSource;
