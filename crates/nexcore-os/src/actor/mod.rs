// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Actor model for NexCore OS — message-passing concurrency.
//!
//! Phase 1: Core actor infrastructure with typed message protocol,
//! actor trait, handles, spawn function, and supervisor.
//!
//! ## Primitive Grounding
//!
//! | Concept       | Primitives  | Explanation                                    |
//! |---------------|-------------|------------------------------------------------|
//! | ActorMessage  | Σ + μ       | Sum-type variants; each maps to a handler      |
//! | Actor trait   | σ + ς + ∂   | Sequential processing, encapsulated state, isolation |
//! | ActorHandle   | ∃ + λ       | Existence proof + location via channel sender  |
//! | Supervisor    | σ + μ + κ   | Routes messages by comparing target            |
//! | Mailbox       | σ + N + ∂   | Ordered queue with bounded capacity            |

pub mod audio_actor;
pub mod network_actor;
pub mod persistence_actor;
pub mod security_actor;
pub mod user_actor;
pub mod vault_actor;

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use crate::audio::AudioState;
use crate::network::NetworkState;
use crate::persistence::OsStateSnapshot;
use crate::security::{SecurityLevel, SecurityResponse, ThreatPattern};
use crate::service::ServiceId;
use crate::user::{AuthError, Session, UserId, UserSummary};
use crate::vault::{VaultError, VaultState};

/// Default bounded mailbox capacity for actors.
pub const DEFAULT_MAILBOX_SIZE: usize = 256;

// ═══════════════════════════════════════════════════════════
// MESSAGE TYPE (Σ Sum — exclusive disjunction)
// ═══════════════════════════════════════════════════════════

/// Top-level actor message — dispatched to actors via their mailbox.
///
/// Tier: T2-C (Σ + μ — sum type with handler mapping)
///
/// This enum is intentionally flat (not nested) so that every variant
/// is visible at the top level. `SyncSender` does not implement `Debug`,
/// so this type does not derive `Debug`.
pub enum ActorMessage {
    // ── Control ──────────────────────────────────────────────
    /// Graceful shutdown — actor should clean up and exit.
    Shutdown,

    /// Liveness probe — expects a `Pong` on the reply channel.
    Ping {
        /// Channel for the Pong response.
        reply_to: SyncSender<Self>,
    },

    /// Response to a `Ping`.
    Pong,

    // ── Security ─────────────────────────────────────────────
    /// Record a threat pattern in the security monitor.
    RecordThreat(ThreatPattern),

    /// Query the current security level.
    QuerySecurityLevel {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response carrying the current security level.
    SecurityLevelResponse(SecurityLevel),

    /// Check if a service is quarantined.
    QueryQuarantine {
        /// Service to check.
        service_id: ServiceId,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response indicating quarantine status.
    QuarantineResponse(bool),

    /// Drain all pending security responses.
    DrainResponses {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response carrying drained security responses.
    ResponsesList(Vec<SecurityResponse>),

    // ── Vault ────────────────────────────────────────────────
    /// Initialize the vault for first boot.
    VaultInitialize {
        /// Vault password.
        password: String,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Lock the vault (zero the key from memory).
    VaultLock,

    /// Unlock the vault with password.
    VaultUnlock {
        /// Vault password.
        password: String,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Store a service token in the vault.
    VaultStoreToken {
        /// Service name.
        service_name: String,
        /// Token value.
        token: String,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Retrieve a service token from the vault.
    VaultGetToken {
        /// Service name.
        service_name: String,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Query the vault lifecycle state.
    QueryVaultState {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response carrying vault lifecycle state.
    VaultStateResponse(VaultState),

    /// Result of a vault mutation operation.
    VaultResult(Result<(), VaultError>),

    /// Result of a vault token retrieval.
    VaultTokenResponse(Result<String, VaultError>),

    // ── Network ──────────────────────────────────────────────
    /// Block an IP address via the firewall.
    NetworkBlockIp(nexcore_network::IpAddr),

    /// Query the network subsystem state.
    QueryNetworkState {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response carrying network state.
    NetworkStateResponse(NetworkState),

    // ── Audio ────────────────────────────────────────────────
    /// Set the master volume [0.0, 1.0].
    AudioSetVolume(f32),

    /// Toggle system mute on/off.
    AudioToggleMute,

    /// Query the audio subsystem state.
    QueryAudioState {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response carrying audio state, volume, and mute status.
    AudioStateResponse {
        /// Lifecycle state.
        state: AudioState,
        /// Master volume [0.0, 1.0].
        volume: f32,
        /// Whether system audio is muted.
        muted: bool,
    },

    // ── User/Auth ────────────────────────────────────────────
    /// Authenticate and create a session.
    UserLogin {
        /// Username.
        username: String,
        /// Password.
        password: String,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Destroy a session.
    UserLogout {
        /// Session token.
        token: String,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Create the device owner account.
    UserCreateOwner {
        /// Username.
        username: String,
        /// Display name.
        display_name: String,
        /// Password.
        password: String,
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response from a login attempt.
    UserLoginResponse(Result<Session, AuthError>),

    /// Response from a logout attempt.
    UserLogoutResponse(Result<(), AuthError>),

    /// Response from account creation.
    UserCreateResponse(Result<UserId, AuthError>),

    /// Query all users.
    QueryUsers {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response carrying the user list.
    UsersResponse(Vec<UserSummary>),

    // ── Persistence ──────────────────────────────────────────
    /// Save an OS state snapshot to storage.
    PersistSaveSnapshot(OsStateSnapshot),

    /// Load the most recent snapshot.
    PersistLoadSnapshot {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response carrying a loaded snapshot (or None).
    PersistSnapshotResponse(Option<OsStateSnapshot>),

    /// Check if crash recovery is needed.
    PersistCrashCheck {
        /// Reply channel.
        reply_to: SyncSender<Self>,
    },

    /// Response indicating whether crash recovery is needed.
    PersistCrashResponse(bool),

    // ── IPC Passthrough ─────────────────────────────────────
    /// Cytokine event routed from the IPC EventBus.
    CytokineEvent(nexcore_cytokine::Cytokine),
}

// ═══════════════════════════════════════════════════════════
// ACTOR TRAIT (σ + ς + ∂ — sequential, stateful, isolated)
// ═══════════════════════════════════════════════════════════

/// Actor trait — the fundamental unit of concurrent computation.
///
/// Each actor:
/// - Runs on its own thread (∂ isolation boundary)
/// - Processes messages sequentially (σ sequence)
/// - Maintains encapsulated state (ς state)
/// - Communicates only via messages (no shared mutable state)
///
/// The message loop automatically handles `Shutdown` and `Ping`.
/// Implementations receive all other messages via `handle`.
pub trait Actor: Send + 'static {
    /// Actor name (used as the key in the Supervisor).
    fn name(&self) -> &'static str;

    /// Process one message. Return `false` to stop the actor.
    fn handle(&mut self, msg: ActorMessage) -> bool;

    /// Called when the actor's thread starts (before first message).
    fn on_start(&mut self) {}

    /// Called when the actor's thread exits (after last message).
    fn on_stop(&mut self) {}
}

// ═══════════════════════════════════════════════════════════
// ACTOR HANDLE (∃ + λ — existence proof + channel location)
// ═══════════════════════════════════════════════════════════

/// Handle to a running actor — holds the channel sender and thread.
pub struct ActorHandle {
    /// Actor name.
    pub name: String,
    /// Bounded channel sender for the actor's mailbox.
    pub sender: SyncSender<ActorMessage>,
    /// Actor thread (taken during join).
    thread: Option<thread::JoinHandle<()>>,
}

impl ActorHandle {
    /// Send a message to this actor. Returns `true` if sent successfully.
    pub fn send(&self, msg: ActorMessage) -> bool {
        self.sender.send(msg).is_ok()
    }

    /// Get a reference to the underlying sender.
    pub fn sender(&self) -> &SyncSender<ActorMessage> {
        &self.sender
    }

    /// Check whether the actor's thread is still alive.
    pub fn is_alive(&self) -> bool {
        self.thread.as_ref().is_some_and(|t| !t.is_finished())
    }

    /// Join the actor's thread, blocking until it completes.
    pub fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

// ═══════════════════════════════════════════════════════════
// SPAWN (∃ — bring an actor into existence on its own thread)
// ═══════════════════════════════════════════════════════════

/// Spawn an actor on a dedicated thread with a bounded mailbox.
///
/// The message loop automatically handles:
/// - `Shutdown` — breaks the loop, calls `on_stop`, thread exits
/// - `Ping { reply_to }` — sends `Pong` back on the reply channel
/// - All other messages — forwarded to `Actor::handle`
///
/// When all senders are dropped, the receiver iterator ends and
/// the loop exits gracefully.
pub fn spawn_actor<A: Actor>(mut actor: A, mailbox_size: usize) -> ActorHandle {
    let (sender, receiver): (SyncSender<ActorMessage>, Receiver<ActorMessage>) =
        mpsc::sync_channel(mailbox_size);
    let name = actor.name().to_string();

    let thread_name = name.clone();
    let thread = thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            actor.on_start();

            for msg in receiver {
                match msg {
                    ActorMessage::Shutdown => break,
                    ActorMessage::Ping { reply_to } => {
                        let _ = reply_to.send(ActorMessage::Pong);
                    }
                    other => {
                        if !actor.handle(other) {
                            break;
                        }
                    }
                }
            }

            actor.on_stop();
        })
        .ok();

    ActorHandle {
        name,
        sender,
        thread,
    }
}

// ═══════════════════════════════════════════════════════════
// SUPERVISOR (σ + μ + κ — routes messages to named actors)
// ═══════════════════════════════════════════════════════════

/// Supervisor — manages a collection of named actors.
///
/// Routes messages to actors by name, handles lifecycle (spawn, shutdown).
/// All actors share the same `ActorMessage` protocol.
pub struct Supervisor {
    /// Named actor handles.
    actors: HashMap<String, ActorHandle>,
}

impl Supervisor {
    /// Create an empty supervisor.
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }

    /// Spawn an actor on its own thread and register it by name.
    pub fn spawn<A: Actor>(&mut self, actor: A, mailbox_size: usize) {
        let handle = spawn_actor(actor, mailbox_size);
        self.actors.insert(handle.name.clone(), handle);
    }

    /// Send a message to a named actor. Returns `false` if not found or channel full.
    pub fn send_to(&self, name: &str, msg: ActorMessage) -> bool {
        self.actors
            .get(name)
            .is_some_and(|h| h.sender.send(msg).is_ok())
    }

    /// Broadcast shutdown to all actors (best-effort, does not join).
    pub fn broadcast_shutdown(&self) {
        for handle in self.actors.values() {
            let _ = handle.sender.send(ActorMessage::Shutdown);
        }
    }

    /// Broadcast a cytokine event to all actors (clones per actor).
    pub fn broadcast_cytokine(&self, event: &nexcore_cytokine::Cytokine) {
        for handle in self.actors.values() {
            let _ = handle
                .sender
                .send(ActorMessage::CytokineEvent(event.clone()));
        }
    }

    /// Get the sender for a named actor (for direct channel access).
    pub fn sender_for(&self, name: &str) -> Option<&SyncSender<ActorMessage>> {
        self.actors.get(name).map(|h| &h.sender)
    }

    /// Get the names of all registered actors.
    pub fn alive_actors(&self) -> Vec<&str> {
        self.actors.keys().map(String::as_str).collect()
    }

    /// Number of managed actors.
    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    /// Gracefully shut down all actors and join their threads.
    pub fn shutdown_all(&mut self) {
        // Send Shutdown to all actors (best effort)
        for handle in self.actors.values() {
            let _ = handle.sender.send(ActorMessage::Shutdown);
        }

        // Take ownership, drop senders (guarantees channel disconnect), join threads
        let actors = std::mem::take(&mut self.actors);
        for (_, handle) in actors {
            let ActorHandle { sender, thread, .. } = handle;
            drop(sender);
            if let Some(thread) = thread {
                let _ = thread.join();
            }
        }
    }

    /// Check if a named actor is registered.
    pub fn is_alive(&self, name: &str) -> bool {
        self.actors.contains_key(name)
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Test actor that logs received messages to a shared vector.
    struct TestActor {
        actor_name: &'static str,
        log: Arc<Mutex<Vec<String>>>,
    }

    impl TestActor {
        fn new(name: &'static str, log: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                actor_name: name,
                log,
            }
        }
    }

    impl Actor for TestActor {
        fn name(&self) -> &'static str {
            self.actor_name
        }

        fn handle(&mut self, msg: ActorMessage) -> bool {
            let label = match &msg {
                ActorMessage::Ping { .. } => "ping".to_string(),
                ActorMessage::Pong => "pong".to_string(),
                ActorMessage::Shutdown => "shutdown".to_string(),
                ActorMessage::RecordThreat(_) => "record_threat".to_string(),
                ActorMessage::AudioSetVolume(v) => format!("audio_set_volume({v})"),
                ActorMessage::AudioToggleMute => "audio_toggle_mute".to_string(),
                _ => "other".to_string(),
            };
            if let Ok(mut log) = self.log.lock() {
                log.push(label);
            }
            true
        }

        fn on_start(&mut self) {
            if let Ok(mut log) = self.log.lock() {
                log.push("started".to_string());
            }
        }

        fn on_stop(&mut self) {
            if let Ok(mut log) = self.log.lock() {
                log.push("stopped".to_string());
            }
        }
    }

    #[test]
    fn spawn_and_shutdown() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let actor = TestActor::new("test", log.clone());

        let mut handle = spawn_actor(actor, 16);

        // Give actor time to start
        thread::sleep(Duration::from_millis(50));

        // Shutdown
        assert!(handle.send(ActorMessage::Shutdown));
        handle.join();

        let entries = log.lock().map(|l| l.clone()).unwrap_or_default();
        assert!(
            entries.contains(&"started".to_string()),
            "on_start should be called"
        );
        assert!(
            entries.contains(&"stopped".to_string()),
            "on_stop should be called"
        );
    }

    #[test]
    fn ping_pong() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let actor = TestActor::new("pinger", log.clone());

        let mut handle = spawn_actor(actor, 16);

        // Send Ping and wait for Pong
        let (tx, rx) = mpsc::sync_channel(1);
        assert!(handle.send(ActorMessage::Ping { reply_to: tx }));

        let response = rx.recv_timeout(Duration::from_millis(200));
        assert!(
            matches!(response, Ok(ActorMessage::Pong)),
            "expected Pong response from actor"
        );

        assert!(handle.send(ActorMessage::Shutdown));
        handle.join();
    }

    #[test]
    fn supervisor_spawn_and_route() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let actor = TestActor::new("supervised", log.clone());

        let mut supervisor = Supervisor::new();
        supervisor.spawn(actor, 16);
        assert_eq!(supervisor.actor_count(), 1);

        // Route a message via supervisor
        assert!(supervisor.send_to("supervised", ActorMessage::AudioToggleMute));

        // Unknown actor fails
        assert!(!supervisor.send_to("nonexistent", ActorMessage::Shutdown));

        // Give time to process
        thread::sleep(Duration::from_millis(50));

        supervisor.shutdown_all();

        let entries = log.lock().map(|l| l.clone()).unwrap_or_default();
        assert!(
            entries.contains(&"audio_toggle_mute".to_string()),
            "message should have been routed"
        );
    }

    #[test]
    fn supervisor_alive_actors() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let a1 = TestActor::new("alpha", log.clone());
        let a2 = TestActor::new("beta", log.clone());

        let mut supervisor = Supervisor::new();
        supervisor.spawn(a1, 16);
        supervisor.spawn(a2, 16);

        let mut alive = supervisor.alive_actors();
        alive.sort();
        assert_eq!(alive, vec!["alpha", "beta"]);

        supervisor.shutdown_all();
    }

    #[test]
    fn supervisor_sender_for() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let actor = TestActor::new("channeled", log.clone());

        let mut supervisor = Supervisor::new();
        supervisor.spawn(actor, 16);

        assert!(supervisor.sender_for("channeled").is_some());
        assert!(supervisor.sender_for("missing").is_none());

        supervisor.shutdown_all();
    }

    #[test]
    fn multiple_messages() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let actor = TestActor::new("multi", log.clone());

        let mut handle = spawn_actor(actor, 32);

        // Send 5 pings (handled by the message loop, not the actor)
        // Instead send 5 domain messages that the actor will log
        for _ in 0..5 {
            assert!(handle.send(ActorMessage::AudioToggleMute));
        }

        // Give time to process
        thread::sleep(Duration::from_millis(100));

        assert!(handle.send(ActorMessage::Shutdown));
        handle.join();

        let entries = log.lock().map(|l| l.clone()).unwrap_or_default();
        let toggle_count = entries
            .iter()
            .filter(|e| e.as_str() == "audio_toggle_mute")
            .count();
        assert_eq!(toggle_count, 5, "all 5 messages should be received");
    }

    #[test]
    fn supervisor_default() {
        let supervisor = Supervisor::default();
        assert_eq!(supervisor.actor_count(), 0);
        assert!(supervisor.alive_actors().is_empty());
    }
}
