// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Network actor -- wraps `NetworkManager` for message-passing concurrency.
//!
//! ## Primitive Grounding
//!
//! | Concept       | Primitives | Explanation                                    |
//! |---------------|------------|------------------------------------------------|
//! | NetworkActor  | ς + ∂      | Encapsulated network state behind actor boundary |
//! | handle()      | Σ + μ      | Sum-type dispatch, each variant maps to network op |

use crate::actor::{Actor, ActorMessage};
use crate::network::NetworkManager;

/// Actor wrapping the OS network manager.
pub struct NetworkActor {
    manager: NetworkManager,
}

impl NetworkActor {
    /// Create a new network actor with an initialized manager.
    pub fn new() -> Self {
        let mut manager = NetworkManager::new();
        manager.initialize();
        Self { manager }
    }
}

impl Default for NetworkActor {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for NetworkActor {
    fn name(&self) -> &'static str {
        "network"
    }

    fn handle(&mut self, msg: ActorMessage) -> bool {
        match msg {
            ActorMessage::NetworkBlockIp(addr) => {
                self.manager.block_ip(addr);
                true
            }
            ActorMessage::QueryNetworkState { reply_to } => {
                let _ = reply_to.send(ActorMessage::NetworkStateResponse(self.manager.state()));
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
    use crate::network::NetworkState;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn query_state_after_init() {
        let mut handle = spawn_actor(NetworkActor::new(), 16);

        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryNetworkState { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::NetworkStateResponse(state)) => {
                assert_eq!(state, NetworkState::Discovered);
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected NetworkStateResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn block_ip_and_query() {
        let mut handle = spawn_actor(NetworkActor::new(), 16);

        // Block an IP (fire-and-forget).
        let addr = nexcore_network::IpAddr::v4(10, 0, 0, 1);
        handle.send(ActorMessage::NetworkBlockIp(addr));
        std::thread::sleep(Duration::from_millis(50));

        // Verify actor is still alive via state query.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryNetworkState { reply_to: tx });

        assert!(
            matches!(
                rx.recv_timeout(Duration::from_secs(1)),
                Ok(ActorMessage::NetworkStateResponse(_))
            ),
            "expected NetworkStateResponse"
        );

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }
}
