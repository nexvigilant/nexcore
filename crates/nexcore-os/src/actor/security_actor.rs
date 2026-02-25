// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Security actor -- wraps `SecurityMonitor` for message-passing concurrency.
//!
//! ## Primitive Grounding
//!
//! | Concept         | Primitives | Explanation                                   |
//! |-----------------|------------|-----------------------------------------------|
//! | SecurityActor   | ς + ∂      | Encapsulated security state behind actor boundary |
//! | handle dispatch | Σ + μ      | Sum-type matching maps messages to monitor ops |

use crate::actor::{Actor, ActorMessage};
use crate::security::SecurityMonitor;

/// Actor wrapping the OS security monitor.
///
/// Runs on a dedicated thread, processes security-related `ActorMessage`
/// variants through the underlying `SecurityMonitor`, and replies via
/// one-shot channels.
pub struct SecurityActor {
    monitor: SecurityMonitor,
}

impl SecurityActor {
    /// Create a new security actor with a fresh monitor.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `SecurityMonitor` runtime cannot be created.
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            monitor: SecurityMonitor::new()?,
        })
    }
}

impl Actor for SecurityActor {
    fn name(&self) -> &'static str {
        "security"
    }

    fn handle(&mut self, msg: ActorMessage) -> bool {
        match msg {
            ActorMessage::RecordThreat(pattern) => {
                self.monitor.record_pattern(&pattern);
                true
            }
            ActorMessage::QuerySecurityLevel { reply_to } => {
                let _ = reply_to.send(ActorMessage::SecurityLevelResponse(self.monitor.level()));
                true
            }
            ActorMessage::QueryQuarantine {
                service_id,
                reply_to,
            } => {
                let _ = reply_to.send(ActorMessage::QuarantineResponse(
                    self.monitor.is_quarantined(service_id),
                ));
                true
            }
            ActorMessage::DrainResponses { reply_to } => {
                let _ = reply_to.send(ActorMessage::ResponsesList(self.monitor.drain_responses()));
                true
            }
            // Ignore messages intended for other actors.
            _ => true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::actor::spawn_actor;
    use crate::security::{Pamp, SecurityLevel, SecurityResponse, ThreatPattern};
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn record_and_query_level() {
        let actor = match SecurityActor::new() {
            Ok(a) => a,
            Err(_) => return, // skip if runtime unavailable
        };
        let mut handle = spawn_actor(actor, 16);

        // Record a high-severity threat via pattern.
        handle.send(ActorMessage::RecordThreat(ThreatPattern::External(
            Pamp::UnauthorizedAccess {
                resource: "/etc/shadow".into(),
                actor: "rogue-proc".into(),
            },
        )));

        // Give time to process.
        std::thread::sleep(Duration::from_millis(50));

        // Query the security level.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QuerySecurityLevel { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::SecurityLevelResponse(level)) => {
                assert!(level >= SecurityLevel::Orange, "expected at least Orange");
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("unexpected message variant"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn drain_responses_after_critical_threat() {
        let actor = match SecurityActor::new() {
            Ok(a) => a,
            Err(e) => return, // skip if runtime unavailable
        };
        let mut handle = spawn_actor(actor, 16);

        // Record critical threat -> should produce Lockdown response.
        handle.send(ActorMessage::RecordThreat(ThreatPattern::External(
            Pamp::MaliciousPayload {
                payload_type: "RCE".into(),
                location: "api-endpoint".into(),
            },
        )));

        // Give time to process.
        std::thread::sleep(Duration::from_millis(50));

        // Drain responses.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::DrainResponses { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::ResponsesList(responses)) => {
                assert!(!responses.is_empty(), "expected pending responses");
                assert!(
                    responses.contains(&SecurityResponse::Lockdown),
                    "expected Lockdown for critical threat"
                );
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("unexpected message variant"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn quarantine_query() {
        let actor = match SecurityActor::new() {
            Ok(a) => a,
            Err(_) => return, // skip if runtime unavailable
        };
        let mut handle = spawn_actor(actor, 16);
        let svc_id = crate::service::ServiceId::new(42);

        // Fresh monitor -- nothing quarantined.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::QueryQuarantine {
            service_id: svc_id,
            reply_to: tx,
        });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::QuarantineResponse(quarantined)) => {
                assert!(!quarantined, "nothing should be quarantined initially");
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("unexpected message variant"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }
}
