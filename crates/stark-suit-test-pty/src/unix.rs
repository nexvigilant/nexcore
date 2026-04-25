//! Unix PTY fixture impl. `nix::pty::openpty` returns master + slave fds; we
//! own the master (writable end), drop the slave fd (kernel keeps the device
//! alive until master closes), and hand the slave path to the consumer.

use std::io;
use std::os::fd::OwnedFd;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// PTY pair fixture. Drop closes the master fd, the kernel reaps the pair.
#[derive(Debug)]
pub struct TestPty {
    /// Writable master end. Bytes written here are read out of `slave_path`.
    pub master: File,
    /// Path the consumer (e.g. `SerialBmsSource::open`) should pass to `tokio-serial`.
    pub slave_path: PathBuf,
}

impl TestPty {
    /// Open a kernel PTY pair. Returns `Err` if the host does not support pty
    /// allocation (rare; container ulimits or `/dev/ptmx` permission denials).
    pub fn spawn() -> io::Result<Self> {
        let pty = nix::pty::openpty(None, None)
            .map_err(|e| io::Error::other(format!("openpty failed: {e}")))?;
        // Discover slave path BEFORE dropping the slave fd — ttyname requires
        // an open fd to resolve.
        let slave_path = nix::unistd::ttyname(&pty.slave)
            .map_err(|e| io::Error::other(format!("ttyname failed: {e}")))?;
        // Convert master to a tokio File (non-blocking owner).
        let master_owned: OwnedFd = pty.master;
        let master_std = std::fs::File::from(master_owned);
        let master = File::from_std(master_std);
        // pty.slave drops here; the device stays alive while `master` lives.
        drop(pty.slave);
        Ok(TestPty { master, slave_path })
    }

    /// Write one NDJSON line (`payload + "\n"`) to the master and flush.
    /// Tests should call this once per BmsFrame.
    pub async fn write_line(&mut self, payload: &str) -> io::Result<()> {
        self.master.write_all(payload.as_bytes()).await?;
        self.master.write_all(b"\n").await?;
        self.master.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, BufReader};

    #[tokio::test]
    async fn pty_round_trips_one_line() {
        let mut pty = TestPty::spawn().expect("openpty");
        // Open the slave end as a plain async read stream (a reader doesn't
        // need tokio-serial — pty slaves are character devices accessible via
        // tokio::fs::File).
        let slave = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&pty.slave_path)
            .await
            .expect("open slave");
        let mut reader = BufReader::new(slave);

        // Write one NDJSON line through the master.
        let payload = r#"{"version":1,"ts_ms":1,"pack_voltage_v":400.0,"pack_current_a":10.0,"cell_temp_c":25.0,"soc_pct":100.0,"soh_pct":100.0,"tier":"comms"}"#;
        pty.write_line(payload).await.expect("write_line");

        // Read it back through the slave.
        let mut buf = String::new();
        reader.read_line(&mut buf).await.expect("read_line");
        assert_eq!(buf.trim(), payload);
    }
}
