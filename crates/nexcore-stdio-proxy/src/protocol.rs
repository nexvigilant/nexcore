//! Protocol capture trait — defines how a proxy captures and replays handshakes.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | Line → capture decision |
//! | T1: State (ς) | Captured handshake state |
//! | T1: Void (∅) | NoCapture — no protocol awareness |

use tokio::sync::mpsc;

use crate::child::{ChildLine, ManagedChild};
use crate::error::Result;

/// Tier: T2-C — Trait for protocol-specific handshake capture and replay.
///
/// Implementations capture protocol initialization messages and can replay
/// them when the child process is restarted. The proxy calls these methods
/// during normal operation and reload sequences.
pub trait ProtocolCapture: Send + Default {
    /// Try to capture a client→server message. Returns true if captured.
    fn try_capture_client(&mut self, line: &str) -> bool;

    /// Try to capture a server→client message. Returns true if captured.
    fn try_capture_server(&mut self, line: &str) -> bool;

    /// Whether we have a complete handshake to replay.
    fn is_complete(&self) -> bool;

    /// Replay the captured handshake to a newly spawned child.
    ///
    /// Default implementation is a no-op (for protocols with no handshake).
    async fn replay_handshake(
        &self,
        _child: &ManagedChild,
        _child_rx: &mut mpsc::Receiver<ChildLine>,
    ) -> Result<()> {
        Ok(())
    }
}

/// Tier: T2-P — No-op capture for protocols without initialization handshakes.
#[derive(Debug, Default)]
pub struct NoCapture;

impl ProtocolCapture for NoCapture {
    fn try_capture_client(&mut self, _line: &str) -> bool {
        false
    }

    fn try_capture_server(&mut self, _line: &str) -> bool {
        false
    }

    fn is_complete(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_capture_never_captures_client() {
        let mut cap = NoCapture;
        assert!(!cap.try_capture_client("anything"));
        assert!(!cap.try_capture_client(r#"{"jsonrpc":"2.0"}"#));
    }

    #[test]
    fn no_capture_never_captures_server() {
        let mut cap = NoCapture;
        assert!(!cap.try_capture_server("anything"));
    }

    #[test]
    fn no_capture_never_complete() {
        let cap = NoCapture;
        assert!(!cap.is_complete());
    }
}
