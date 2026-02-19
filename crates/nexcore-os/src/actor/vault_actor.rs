// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Vault actor -- wraps `OsVault` for message-passing concurrency.
//!
//! ## Primitive Grounding
//!
//! | Concept     | Primitives | Explanation                                |
//! |-------------|------------|--------------------------------------------|
//! | VaultActor  | ς + ∂      | Encapsulated vault state behind actor boundary |
//! | handle()    | Σ + μ      | Sum-type dispatch, each variant maps to vault op |

use crate::actor::{Actor, ActorMessage};
use crate::vault::OsVault;

/// Actor wrapping the OS encrypted vault.
pub struct VaultActor {
    vault: OsVault,
}

impl VaultActor {
    /// Create with a virtual (in-memory) vault for testing.
    pub fn virtual_vault() -> Self {
        Self {
            vault: OsVault::virtual_vault(),
        }
    }

    /// Create with a real vault at the given data directory.
    pub fn new(data_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            vault: OsVault::new(data_dir),
        }
    }
}

impl Actor for VaultActor {
    fn name(&self) -> &'static str {
        "vault"
    }

    fn handle(&mut self, msg: ActorMessage) -> bool {
        match msg {
            ActorMessage::VaultInitialize { password, reply_to } => {
                let result = self.vault.initialize(&password);
                let _ = reply_to.send(ActorMessage::VaultResult(result));
                true
            }
            ActorMessage::VaultLock => {
                self.vault.lock();
                true
            }
            ActorMessage::VaultUnlock { password, reply_to } => {
                let result = self.vault.unlock(&password);
                let _ = reply_to.send(ActorMessage::VaultResult(result));
                true
            }
            ActorMessage::VaultStoreToken {
                service_name,
                token,
                reply_to,
            } => {
                let result = self.vault.store_service_token(&service_name, &token);
                let _ = reply_to.send(ActorMessage::VaultResult(result));
                true
            }
            ActorMessage::VaultGetToken {
                service_name,
                reply_to,
            } => {
                let result = self.vault.get_service_token(&service_name);
                let _ = reply_to.send(ActorMessage::VaultTokenResponse(result));
                true
            }
            ActorMessage::QueryVaultState { reply_to } => {
                let _ = reply_to.send(ActorMessage::VaultStateResponse(self.vault.state()));
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
    use crate::vault::VaultState;
    use std::sync::mpsc;
    use std::time::Duration;

    fn temp_vault_actor() -> Option<(VaultActor, tempfile::TempDir)> {
        let dir = tempfile::TempDir::new().ok()?;
        let actor = VaultActor::new(dir.path());
        Some((actor, dir))
    }

    #[test]
    fn initialize_and_query_state() {
        let (actor, _dir) = match temp_vault_actor() {
            Some(v) => v,
            None => return,
        };
        let mut handle = spawn_actor(actor, 16);

        // Initialize.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::VaultInitialize {
            password: "test-password".into(),
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(ActorMessage::VaultResult(Ok(()))) => {}
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected VaultResult(Ok(()))"),
        }

        // Query state -- should be Unlocked after initialize.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryVaultState { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(ActorMessage::VaultStateResponse(state)) => {
                assert_eq!(state, VaultState::Unlocked);
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected VaultStateResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn lock_and_query() {
        let (actor, _dir) = match temp_vault_actor() {
            Some(v) => v,
            None => return,
        };
        let mut handle = spawn_actor(actor, 16);

        // Initialize.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::VaultInitialize {
            password: "lock-test".into(),
            reply_to: tx,
        });
        let _ = rx.recv_timeout(Duration::from_secs(2));

        // Lock.
        handle.send(ActorMessage::VaultLock);
        std::thread::sleep(Duration::from_millis(50));

        // Query state.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryVaultState { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(ActorMessage::VaultStateResponse(state)) => {
                assert_eq!(state, VaultState::Locked);
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected VaultStateResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn store_and_get_token() {
        let (actor, _dir) = match temp_vault_actor() {
            Some(v) => v,
            None => return,
        };
        let mut handle = spawn_actor(actor, 16);

        // Initialize.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::VaultInitialize {
            password: "token-test".into(),
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(ActorMessage::VaultResult(Ok(()))) => {}
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected VaultResult(Ok(())): init failed"),
        }

        // Store a service token.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::VaultStoreToken {
            service_name: "guardian".into(),
            token: "grd-secret-42".into(),
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(ActorMessage::VaultResult(Ok(()))) => {}
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected VaultResult(Ok(()))"),
        }

        // Retrieve the token.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::VaultGetToken {
            service_name: "guardian".into(),
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(ActorMessage::VaultTokenResponse(Ok(token))) => {
                assert_eq!(token, "grd-secret-42");
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected VaultTokenResponse(Ok(...))"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }
}
