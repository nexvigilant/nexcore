//! Generic stdio proxy loop state machine.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | ProxyState enum (Starting→Proxying→Reloading→Shutdown) |
//! | T1: Sequence (σ) | Message forwarding pipeline |
//! | T1: Mapping (μ) | Client ↔ Server message routing |

use std::path::PathBuf;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use crate::child::{ChildLine, ManagedChild, verify_binary};
use crate::error::{ProxyError, Result};
use crate::protocol::ProtocolCapture;
use crate::queue::MessageQueue;
use crate::watcher::BinaryWatcher;

/// Tier: T2-C — Proxy configuration (protocol-agnostic).
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Path to the binary to proxy.
    pub binary: PathBuf,
    /// Arguments to pass to the child binary.
    pub child_args: Vec<String>,
    /// Debounce duration for file change events.
    pub debounce: Duration,
    /// Grace period before SIGKILL on reload.
    pub grace_period: Duration,
    /// Maximum number of messages to queue during reload.
    pub queue_capacity: usize,
}

/// Tier: T2-C — Proxy loop state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyState {
    Starting,
    Proxying,
    Reloading,
    Shutdown,
}

/// Tier: T2-C — Generic stdio proxy with protocol-specific capture.
pub struct StdioProxy<P: ProtocolCapture> {
    config: ProxyConfig,
    state: ProxyState,
    capture: P,
    queue: MessageQueue,
}

/// What happened in one select iteration.
enum SelectOutcome {
    ClientLine(String),
    ClientClosed,
    ChildStdout(String),
    ChildStderr(String),
    ChildClosed,
    ReloadSignal,
}

// ── Pure helpers ──────────────────────────────────────────────────

/// Write a line to tokio stdout (client-facing).
async fn write_to_client(line: &str) -> Result<()> {
    let mut stdout = tokio::io::stdout();
    stdout.write_all(line.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

/// Write a line to tokio stderr (logging).
async fn log_to_stderr(line: &str) {
    let mut stderr = tokio::io::stderr();
    let _ = stderr.write_all(line.as_bytes()).await;
    let _ = stderr.write_all(b"\n").await;
    let _ = stderr.flush().await;
}

/// Wait for one event from client, child, or watcher.
async fn select_once(
    client_rx: &mut mpsc::Receiver<String>,
    child_rx: &mut mpsc::Receiver<ChildLine>,
    reload_rx: &mut mpsc::Receiver<()>,
) -> SelectOutcome {
    tokio::select! {
        line = client_rx.recv() => match line {
            Some(l) => SelectOutcome::ClientLine(l),
            None => SelectOutcome::ClientClosed,
        },
        line = child_rx.recv() => match line {
            Some(ChildLine::Stdout(s)) => SelectOutcome::ChildStdout(s),
            Some(ChildLine::Stderr(s)) => SelectOutcome::ChildStderr(s),
            None => SelectOutcome::ChildClosed,
        },
        _ = reload_rx.recv() => SelectOutcome::ReloadSignal,
    }
}

/// Read lines from real stdin and forward to channel.
async fn read_client_stdin(tx: mpsc::Sender<String>) {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if tx.send(line).await.is_err() {
            break;
        }
    }
}

// ── StdioProxy<P> impl ──────────────────────────────────────────

impl<P: ProtocolCapture> StdioProxy<P> {
    /// Create a new proxy with the given config.
    pub fn new(config: ProxyConfig) -> Self {
        let queue = MessageQueue::new(config.queue_capacity);
        Self {
            config,
            state: ProxyState::Starting,
            capture: P::default(),
            queue,
        }
    }

    /// Run the proxy loop. This blocks until shutdown.
    pub async fn run(&mut self) -> Result<()> {
        verify_binary(&self.config.binary)?;

        let mut child = ManagedChild::new(&self.config.binary, self.config.child_args.clone());
        let mut child_rx = child.spawn()?;

        let watcher = BinaryWatcher::new(&self.config.binary, self.config.debounce);
        let mut reload_rx = watcher.start()?;

        let (client_tx, mut client_rx) = mpsc::channel::<String>(256);
        tokio::spawn(read_client_stdin(client_tx));

        self.state = ProxyState::Proxying;
        tracing::info!("Proxy active: {} → client", self.config.binary.display());

        self.event_loop(&mut child, &mut child_rx, &mut client_rx, &mut reload_rx)
            .await
    }

    /// Main event loop — dispatches select outcomes.
    async fn event_loop(
        &mut self,
        child: &mut ManagedChild,
        child_rx: &mut mpsc::Receiver<ChildLine>,
        client_rx: &mut mpsc::Receiver<String>,
        reload_rx: &mut mpsc::Receiver<()>,
    ) -> Result<()> {
        loop {
            if self.state == ProxyState::Shutdown {
                child.shutdown(self.config.grace_period).await?;
                return Ok(());
            }

            let outcome = select_once(client_rx, child_rx, reload_rx).await;
            self.handle_outcome(outcome, child, child_rx).await?;
        }
    }

    /// Handle one select outcome.
    async fn handle_outcome(
        &mut self,
        outcome: SelectOutcome,
        child: &mut ManagedChild,
        child_rx: &mut mpsc::Receiver<ChildLine>,
    ) -> Result<()> {
        match outcome {
            SelectOutcome::ClientLine(line) => self.on_client_line(&line, child).await,
            SelectOutcome::ClientClosed => {
                tracing::info!("Client stdin closed, shutting down");
                self.state = ProxyState::Shutdown;
                Ok(())
            }
            SelectOutcome::ChildStdout(s) => self.on_child_stdout(&s).await,
            SelectOutcome::ChildStderr(s) => {
                log_to_stderr(&s).await;
                Ok(())
            }
            SelectOutcome::ChildClosed => {
                tracing::warn!("Child closed unexpectedly");
                Ok(())
            }
            SelectOutcome::ReloadSignal => self.reload(child, child_rx).await,
        }
    }

    /// Handle a line from the client.
    async fn on_client_line(&mut self, line: &str, child: &ManagedChild) -> Result<()> {
        self.capture.try_capture_client(line);
        if self.state == ProxyState::Reloading {
            return self.queue.push(line.to_string());
        }
        child.send_line(line).await
    }

    /// Handle a stdout line from the child.
    async fn on_child_stdout(&mut self, line: &str) -> Result<()> {
        self.capture.try_capture_server(line);
        write_to_client(line).await
    }

    /// Execute the reload sequence.
    async fn reload(
        &mut self,
        child: &mut ManagedChild,
        child_rx: &mut mpsc::Receiver<ChildLine>,
    ) -> Result<()> {
        self.state = ProxyState::Reloading;
        tracing::info!("=== RELOAD STARTING ===");

        child.shutdown(self.config.grace_period).await?;
        verify_binary(&self.config.binary)?;
        *child_rx = child.spawn()?;

        // Delegate handshake replay to the protocol capture
        self.capture.replay_handshake(child, child_rx).await?;
        self.drain_queue(child).await?;

        self.state = ProxyState::Proxying;
        tracing::info!("=== RELOAD COMPLETE ===");
        Ok(())
    }

    /// Drain queued messages into the child.
    async fn drain_queue(&mut self, child: &ManagedChild) -> Result<()> {
        let messages = self.queue.drain();
        let count = messages.len();
        for msg in messages {
            child.send_line(&msg).await?;
        }
        if count > 0 {
            tracing::info!("Drained {count} queued messages");
        }
        Ok(())
    }

    /// Current state (for testing).
    pub fn state(&self) -> ProxyState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::NoCapture;

    fn test_config() -> ProxyConfig {
        ProxyConfig {
            binary: PathBuf::from("/bin/cat"),
            child_args: vec![],
            debounce: Duration::from_secs(1),
            grace_period: Duration::from_secs(1),
            queue_capacity: 100,
        }
    }

    #[test]
    fn initial_state_is_starting() {
        let proxy = StdioProxy::<NoCapture>::new(test_config());
        assert_eq!(proxy.state(), ProxyState::Starting);
    }

    #[test]
    fn queue_messages_during_reload() {
        let mut proxy = StdioProxy::<NoCapture>::new(test_config());
        proxy.state = ProxyState::Reloading;
        assert!(proxy.queue.push("msg1".into()).is_ok());
        assert!(proxy.queue.push("msg2".into()).is_ok());
        assert_eq!(proxy.queue.len(), 2);
    }

    #[test]
    fn proxy_state_equality() {
        assert_eq!(ProxyState::Starting, ProxyState::Starting);
        assert_ne!(ProxyState::Starting, ProxyState::Proxying);
        assert_ne!(ProxyState::Reloading, ProxyState::Shutdown);
    }

    #[tokio::test]
    async fn write_to_client_ok() {
        let result = write_to_client(r#"{"jsonrpc":"2.0","id":1}"#).await;
        assert!(result.is_ok());
    }
}
