// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! User actor -- wraps `UserManager` for message-passing concurrency.
//!
//! ## Primitive Grounding
//!
//! | Concept    | Primitives | Explanation                                    |
//! |------------|------------|------------------------------------------------|
//! | UserActor  | ς + ∂      | Encapsulated user state behind actor boundary  |
//! | handle()   | Σ + μ      | Sum-type dispatch, each variant maps to user op |

use crate::actor::{Actor, ActorMessage};
use crate::user::UserManager;

/// Actor wrapping the OS user/authentication manager.
pub struct UserActor {
    manager: UserManager,
}

impl UserActor {
    /// Create a new user actor with an empty user manager.
    pub fn new() -> Self {
        Self {
            manager: UserManager::new(),
        }
    }
}

impl Default for UserActor {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for UserActor {
    fn name(&self) -> &'static str {
        "user"
    }

    fn handle(&mut self, msg: ActorMessage) -> bool {
        match msg {
            ActorMessage::UserLogin {
                username,
                password,
                reply_to,
            } => {
                let result = self.manager.login(&username, &password);
                let _ = reply_to.send(ActorMessage::UserLoginResponse(result));
                true
            }
            ActorMessage::UserLogout { token, reply_to } => {
                let result = self.manager.logout(&token);
                let _ = reply_to.send(ActorMessage::UserLogoutResponse(result));
                true
            }
            ActorMessage::UserCreateOwner {
                username,
                display_name,
                password,
                reply_to,
            } => {
                let result = self
                    .manager
                    .create_owner(&username, &display_name, &password);
                let _ = reply_to.send(ActorMessage::UserCreateResponse(result));
                true
            }
            ActorMessage::QueryUsers { reply_to } => {
                let _ = reply_to.send(ActorMessage::UsersResponse(self.manager.list_users()));
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
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn create_owner_and_login() {
        let mut handle = spawn_actor(UserActor::new(), 16);

        // Create owner.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::UserCreateOwner {
            username: "matthew".into(),
            display_name: "Matthew Campion".into(),
            password: "Nexcore2026!".into(),
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::UserCreateResponse(Ok(id))) => {
                assert_eq!(id.0, 1);
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected UserCreateResponse(Ok(...))"),
        }

        // Login.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::UserLogin {
            username: "matthew".into(),
            password: "Nexcore2026!".into(),
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::UserLoginResponse(Ok(session))) => {
                assert_eq!(session.username, "matthew");
                assert!(session.is_valid());
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected UserLoginResponse(Ok(...))"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn login_and_logout() {
        let mut handle = spawn_actor(UserActor::new(), 16);

        // Create owner first.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::UserCreateOwner {
            username: "matthew".into(),
            display_name: "Matthew".into(),
            password: "Nexcore2026!".into(),
            reply_to: tx,
        });
        let _ = rx.recv_timeout(Duration::from_secs(1));

        // Login.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::UserLogin {
            username: "matthew".into(),
            password: "Nexcore2026!".into(),
            reply_to: tx,
        });

        let token = match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::UserLoginResponse(Ok(session))) => session.token,
            _ => panic!("expected successful login"),
        };

        // Logout.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::UserLogout {
            token,
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::UserLogoutResponse(Ok(()))) => {}
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected UserLogoutResponse(Ok(()))"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn query_users() {
        let mut handle = spawn_actor(UserActor::new(), 16);

        // Create owner first.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::UserCreateOwner {
            username: "matthew".into(),
            display_name: "Matthew Campion".into(),
            password: "Nexcore2026!".into(),
            reply_to: tx,
        });
        let _ = rx.recv_timeout(Duration::from_secs(1));

        // Query all users.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryUsers { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::UsersResponse(users)) => {
                assert_eq!(users.len(), 1);
                assert_eq!(users[0].username, "matthew");
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected UsersResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }
}
