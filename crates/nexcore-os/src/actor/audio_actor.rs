// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio actor -- wraps `AudioManager` for message-passing concurrency.
//!
//! ## Primitive Grounding
//!
//! | Concept     | Primitives | Explanation                                    |
//! |-------------|------------|------------------------------------------------|
//! | AudioActor  | ς + ∂      | Encapsulated audio state behind actor boundary |
//! | handle()    | Σ + μ      | Sum-type dispatch, each variant maps to audio op |

use crate::actor::{Actor, ActorMessage};
use crate::audio::AudioManager;

/// Actor wrapping the OS audio manager.
pub struct AudioActor {
    manager: AudioManager,
}

impl AudioActor {
    /// Create a new audio actor with an initialized manager.
    pub fn new() -> Self {
        let mut manager = AudioManager::new();
        manager.initialize();
        Self { manager }
    }
}

impl Default for AudioActor {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for AudioActor {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn handle(&mut self, msg: ActorMessage) -> bool {
        match msg {
            ActorMessage::AudioSetVolume(volume) => {
                self.manager.set_master_volume(volume);
                true
            }
            ActorMessage::AudioToggleMute => {
                self.manager.toggle_mute();
                true
            }
            ActorMessage::QueryAudioState { reply_to } => {
                let _ = reply_to.send(ActorMessage::AudioStateResponse {
                    state: self.manager.state(),
                    volume: self.manager.master_volume(),
                    muted: self.manager.is_muted(),
                });
                true
            }
            // Ignore messages intended for other actors.
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::spawn_actor;
    use crate::audio::AudioState;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn query_state_after_init() {
        let mut handle = spawn_actor(AudioActor::new(), 16);

        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryAudioState { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::AudioStateResponse { state, .. }) => {
                assert_eq!(state, AudioState::Ready);
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected AudioStateResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn set_volume_and_query() {
        let mut handle = spawn_actor(AudioActor::new(), 16);

        handle.send(ActorMessage::AudioSetVolume(0.5));
        std::thread::sleep(Duration::from_millis(50));

        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryAudioState { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::AudioStateResponse { volume, muted, .. }) => {
                assert!((volume - 0.5).abs() < f32::EPSILON);
                assert!(!muted);
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected AudioStateResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn toggle_mute_and_query() {
        let mut handle = spawn_actor(AudioActor::new(), 16);

        handle.send(ActorMessage::AudioToggleMute);
        std::thread::sleep(Duration::from_millis(50));

        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryAudioState { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::AudioStateResponse { muted, .. }) => {
                assert!(muted, "should be muted after toggle");
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected AudioStateResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }
}
