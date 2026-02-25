//! MCP Relay — stdio ↔ Unix socket bridge with hot-reload support
//!
//! Architecture:
//!   Claude Code ←stdio→ mcp-relay ←unix-socket→ nexcore-mcp (daemon)
//!
//! On socket disconnect (daemon restart):
//!   1. Reconnect to socket (with exponential backoff)
//!   2. Replay cached `initialize` request
//!   3. Send `notifications/tools/list_changed` to Claude Code
//!   4. Resume normal relay
//!
//! Usage:
//!   mcp-relay [socket-path]
//!   Default: /run/user/1000/nexcore-mcp.sock

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::Mutex;

const DEFAULT_SOCKET: &str = "/run/user/1000/nexcore-mcp.sock";
const MAX_RECONNECT_DELAY_MS: u64 = 5000;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    let socket_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_SOCKET.to_string());

    if let Err(e) = run_relay(&socket_path).await {
        eprintln!("[mcp-relay] fatal: {e}");
        std::process::exit(1);
    }
}

/// Main relay loop. Reconnects on daemon restart.
async fn run_relay(socket_path: &str) -> Result<(), BoxError> {
    let cached_initialize: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    // Use channels to decouple stdin reading from socket writing.
    let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(256);
    let (sock_tx, mut sock_rx) = tokio::sync::mpsc::channel::<String>(256);

    // Task 1: Read stdin → channel
    let cached_init_reader = cached_initialize.clone();
    let stdin_task = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    // Cache initialize for replay
                    if line.contains("\"method\":\"initialize\"") && !line.contains("\"result\"") {
                        let mut cache = cached_init_reader.lock().await;
                        *cache = Some(line.clone());
                    }
                    if stdin_tx.send(line.clone()).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Initial connection
    let stream = connect_with_retry(socket_path).await?;
    let (sock_reader, sock_writer) = stream.into_split();
    let sock_writer = Arc::new(Mutex::new(sock_writer));

    // Signal channel to stop old socket readers on reconnect
    let (reconnect_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Spawn socket reader
    spawn_socket_reader(sock_reader, sock_tx.clone(), reconnect_tx.subscribe());

    // Task 3: Channel → socket (with reconnect on write failure)
    let sock_writer_fwd = sock_writer.clone();
    let cached_init_fwd = cached_initialize.clone();
    let socket_path_fwd = socket_path.to_string();
    let sock_tx_fwd = sock_tx.clone();
    let reconnect_tx_fwd = reconnect_tx.clone();

    let forward_task = tokio::spawn(async move {
        while let Some(line) = stdin_rx.recv().await {
            let write_ok = {
                let mut w = sock_writer_fwd.lock().await;
                w.write_all(line.as_bytes()).await.is_ok()
            };

            if !write_ok {
                eprintln!("[mcp-relay] socket write failed, reconnecting...");

                // Signal old reader to stop
                let _ = reconnect_tx_fwd.send(());

                // Reconnect
                let new_stream = match connect_with_retry(&socket_path_fwd).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[mcp-relay] reconnect failed: {e}");
                        break;
                    }
                };

                let (new_reader, new_writer) = new_stream.into_split();

                // Replace writer
                {
                    let mut w = sock_writer_fwd.lock().await;
                    *w = new_writer;
                }

                // Replay full MCP handshake: initialize → response → initialized
                let init_msg = {
                    let cache = cached_init_fwd.lock().await;
                    cache.clone()
                };
                let new_reader_final = if let Some(msg) = init_msg {
                    eprintln!("[mcp-relay] replaying initialize");
                    let mut w = sock_writer_fwd.lock().await;
                    let _ = w.write_all(msg.as_bytes()).await;
                    drop(w);

                    // Read the daemon's initialize response before sending initialized
                    let mut resp_reader = BufReader::new(new_reader);
                    let mut resp_line = String::new();
                    let reader_recovered = match tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        resp_reader.read_line(&mut resp_line),
                    )
                    .await
                    {
                        Ok(Ok(n)) if n > 0 => {
                            eprintln!("[mcp-relay] received initialize response ({n} bytes)");
                            // Forward the response to Claude Code
                            let _ = sock_tx_fwd.send(resp_line).await;
                            resp_reader.into_inner()
                        }
                        _ => {
                            eprintln!(
                                "[mcp-relay] warning: no initialize response, sending initialized anyway"
                            );
                            resp_reader.into_inner()
                        }
                    };

                    // Send the required initialized notification (MCP protocol step 3)
                    let initialized =
                        "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n";
                    let mut w = sock_writer_fwd.lock().await;
                    let _ = w.write_all(initialized.as_bytes()).await;
                    eprintln!("[mcp-relay] sent initialized notification");

                    reader_recovered
                } else {
                    new_reader
                };

                // Spawn new reader
                spawn_socket_reader(
                    new_reader_final,
                    sock_tx_fwd.clone(),
                    reconnect_tx_fwd.subscribe(),
                );

                // Notify Claude Code that tools may have changed
                let notification =
                    "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/tools/list_changed\"}\n";
                let _ = sock_tx_fwd.send(notification.to_string()).await;

                eprintln!("[mcp-relay] reconnected, sent tools/list_changed");

                // Retry the failed write
                let mut w = sock_writer_fwd.lock().await;
                let _ = w.write_all(line.as_bytes()).await;
            }
        }
    });

    // Task 4: Channel → stdout (daemon responses → Claude Code)
    let stdout_task = tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(line) = sock_rx.recv().await {
            if stdout.write_all(line.as_bytes()).await.is_err() {
                break;
            }
            let _ = stdout.flush().await;
        }
    });

    // Exit when stdin closes or forward loop ends
    tokio::select! {
        _ = stdin_task => {},
        _ = forward_task => {},
        _ = stdout_task => {},
    }

    Ok(())
}

fn spawn_socket_reader(
    reader: tokio::net::unix::OwnedReadHalf,
    tx: tokio::sync::mpsc::Sender<String>,
    mut stop: tokio::sync::broadcast::Receiver<()>,
) {
    tokio::spawn(async move {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            tokio::select! {
                result = buf_reader.read_line(&mut line) => {
                    match result {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if tx.send(line.clone()).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                _ = stop.recv() => break,
            }
        }
    });
}

async fn connect_with_retry(socket_path: &str) -> Result<UnixStream, BoxError> {
    let mut delay_ms = 100u64;

    for attempt in 1..=20 {
        match UnixStream::connect(socket_path).await {
            Ok(stream) => {
                eprintln!("[mcp-relay] connected to {socket_path}");
                return Ok(stream);
            }
            Err(e) => {
                if attempt == 20 {
                    return Err(format!("failed to connect after 20 attempts: {e}").into());
                }
                eprintln!("[mcp-relay] attempt {attempt}/20 failed, retrying in {delay_ms}ms...");
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms * 2).min(MAX_RECONNECT_DELAY_MS);
            }
        }
    }

    Err("exhausted retries".into())
}
