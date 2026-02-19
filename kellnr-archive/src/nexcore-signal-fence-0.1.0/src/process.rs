//! Process resolver — maps socket inodes to process information via /proc/<pid>/fd/.
//!
//! Tier: T2-P (μ Mapping + ∃ Existence — maps inodes to processes, validates existence)
//!
//! In signal theory terms, this module provides the "source attribution" for each
//! observed connection. Without process identity, a connection is just noise.

use std::net::IpAddr;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::connection::{Protocol, SocketEntry, TcpState};

/// Direction of network traffic relative to the local machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Incoming connection (remote initiated).
    Ingress,
    /// Outgoing connection (local initiated).
    Egress,
    /// Direction unknown or both (e.g., listening sockets).
    Both,
}

impl Direction {
    /// Infer direction from a socket entry.
    ///
    /// Listening sockets → Both, SynSent → Egress, Established with
    /// remote port < 1024 likely Egress (connecting to a service).
    pub fn infer(entry: &SocketEntry) -> Self {
        if entry.state.is_listening() {
            return Self::Both;
        }
        match entry.state {
            crate::connection::TcpState::SynSent => Self::Egress,
            crate::connection::TcpState::SynRecv => Self::Ingress,
            _ => {
                // Heuristic: if remote port is well-known (<1024), likely egress
                if entry.remote_port > 0 && entry.remote_port < 1024 {
                    Self::Egress
                } else if entry.local_port < 1024 {
                    Self::Ingress
                } else {
                    Self::Both
                }
            }
        }
    }
}

/// Information about a process owning a socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// Process name from /proc/<pid>/comm.
    pub name: String,
    /// Executable path from /proc/<pid>/exe.
    pub exe: PathBuf,
    /// User ID of the process.
    pub uid: u32,
    /// Command line from /proc/<pid>/cmdline (NUL-separated → space-joined).
    pub cmdline: String,
}

/// A connection event combining socket data with process attribution.
///
/// This is the primary "observation" in signal theory — a network connection
/// attributed to a specific process, ready for policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEvent {
    /// The socket entry from /proc/net/tcp.
    pub socket: SocketEntry,
    /// Process information (None if resolution failed).
    pub process: Option<ProcessInfo>,
    /// Inferred traffic direction.
    pub direction: Direction,
    /// When this event was observed.
    pub timestamp: DateTime<Utc>,
}

impl ConnectionEvent {
    /// Create a new connection event from a socket entry and optional process info.
    pub fn new(socket: SocketEntry, process: Option<ProcessInfo>) -> Self {
        let direction = Direction::infer(&socket);
        Self {
            socket,
            process,
            direction,
            timestamp: Utc::now(),
        }
    }

    /// Create a connection event with an explicit timestamp (for testing).
    pub fn with_timestamp(
        socket: SocketEntry,
        process: Option<ProcessInfo>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let direction = Direction::infer(&socket);
        Self {
            socket,
            process,
            direction,
            timestamp,
        }
    }

    /// The process name, or "<unknown>" if no process was resolved.
    pub fn process_name(&self) -> &str {
        self.process.as_ref().map_or("<unknown>", |p| &p.name)
    }

    /// Whether this event has a known process attribution.
    pub fn is_attributed(&self) -> bool {
        self.process.is_some()
    }
}

/// Resolve a socket inode to a process by scanning /proc/<pid>/fd/.
///
/// This reads /proc/<pid>/fd/ symlinks looking for `socket:[<inode>]`.
/// Returns None if no process is found (permission denied, process exited, etc.).
pub fn resolve_inode(inode: u64) -> Option<ProcessInfo> {
    if inode == 0 {
        return None;
    }
    let target = format!("socket:[{inode}]");

    let proc_dir = std::fs::read_dir("/proc").ok()?;
    for entry in proc_dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Only look at numeric directories (PIDs)
        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let fd_dir = format!("/proc/{pid}/fd");
        let fd_entries = match std::fs::read_dir(&fd_dir) {
            Ok(entries) => entries,
            Err(_) => continue, // permission denied or process gone
        };

        for fd_entry in fd_entries.flatten() {
            let link = match std::fs::read_link(fd_entry.path()) {
                Ok(l) => l,
                Err(_) => continue,
            };
            if link.to_string_lossy() == target {
                return read_process_info(pid);
            }
        }
    }
    None
}

/// Read process information from /proc/<pid>/.
fn read_process_info(pid: u32) -> Option<ProcessInfo> {
    let comm = std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| format!("pid:{pid}"));

    let exe = std::fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .unwrap_or_else(|| PathBuf::from(format!("/proc/{pid}/exe")));

    let cmdline = std::fs::read_to_string(format!("/proc/{pid}/cmdline"))
        .ok()
        .map(|s| s.replace('\0', " ").trim().to_string())
        .unwrap_or_default();

    // Read UID from /proc/<pid>/status
    let uid = read_uid(pid).unwrap_or(0);

    Some(ProcessInfo {
        pid,
        name: comm,
        exe,
        uid,
        cmdline,
    })
}

/// Read the real UID from /proc/<pid>/status.
fn read_uid(pid: u32) -> Option<u32> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            let uid_str = rest.split_whitespace().next()?;
            return uid_str.parse().ok();
        }
    }
    None
}

/// Resolve all socket entries to connection events.
///
/// Builds a fast inode→pid lookup and resolves each entry.
pub fn resolve_all(entries: &[SocketEntry]) -> Vec<ConnectionEvent> {
    entries
        .iter()
        .map(|socket| {
            let process = resolve_inode(socket.inode);
            ConnectionEvent::new(socket.clone(), process)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    fn make_socket(local_port: u16, remote_port: u16, state: TcpState, inode: u64) -> SocketEntry {
        SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            remote_addr: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            remote_port,
            inode,
            state,
            protocol: Protocol::Tcp,
            uid: 1000,
        }
    }

    #[test]
    fn test_direction_infer_listening() {
        let socket = make_socket(8080, 0, TcpState::Listen, 100);
        assert_eq!(Direction::infer(&socket), Direction::Both);
    }

    #[test]
    fn test_direction_infer_syn_sent() {
        let socket = make_socket(54321, 443, TcpState::SynSent, 100);
        assert_eq!(Direction::infer(&socket), Direction::Egress);
    }

    #[test]
    fn test_direction_infer_syn_recv() {
        let socket = make_socket(80, 54321, TcpState::SynRecv, 100);
        assert_eq!(Direction::infer(&socket), Direction::Ingress);
    }

    #[test]
    fn test_direction_infer_established_to_well_known() {
        // Connecting to port 443 (well-known) = egress
        let socket = make_socket(54321, 443, TcpState::Established, 100);
        assert_eq!(Direction::infer(&socket), Direction::Egress);
    }

    #[test]
    fn test_direction_infer_established_from_well_known() {
        // Local port 80 (well-known), remote high port = ingress
        let socket = make_socket(80, 54321, TcpState::Established, 100);
        assert_eq!(Direction::infer(&socket), Direction::Ingress);
    }

    #[test]
    fn test_direction_infer_both_high_ports() {
        // Both ports high — ambiguous
        let socket = make_socket(54321, 54322, TcpState::Established, 100);
        assert_eq!(Direction::infer(&socket), Direction::Both);
    }

    #[test]
    fn test_connection_event_new() {
        let socket = make_socket(80, 54321, TcpState::Established, 100);
        let event = ConnectionEvent::new(socket, None);
        assert!(!event.is_attributed());
        assert_eq!(event.process_name(), "<unknown>");
    }

    #[test]
    fn test_connection_event_with_process() {
        let socket = make_socket(443, 54321, TcpState::Established, 100);
        let proc_info = ProcessInfo {
            pid: 1234,
            name: "nginx".to_string(),
            exe: PathBuf::from("/usr/sbin/nginx"),
            uid: 33,
            cmdline: "nginx -g daemon off;".to_string(),
        };
        let event = ConnectionEvent::new(socket, Some(proc_info));
        assert!(event.is_attributed());
        assert_eq!(event.process_name(), "nginx");
    }

    #[test]
    fn test_connection_event_with_timestamp() {
        let socket = make_socket(8080, 0, TcpState::Listen, 200);
        let ts = Utc::now();
        let event = ConnectionEvent::with_timestamp(socket, None, ts);
        assert_eq!(event.timestamp, ts);
    }

    #[test]
    fn test_resolve_inode_zero() {
        // Inode 0 means kernel socket, skip immediately
        assert!(resolve_inode(0).is_none());
    }

    #[test]
    fn test_process_info_serialization() {
        let info = ProcessInfo {
            pid: 1,
            name: "systemd".to_string(),
            exe: PathBuf::from("/usr/lib/systemd/systemd"),
            uid: 0,
            cmdline: "/usr/lib/systemd/systemd --system".to_string(),
        };
        let json = serde_json::to_string(&info);
        assert!(json.is_ok());
        let json_str = json.ok().unwrap_or_default();
        assert!(json_str.contains("systemd"));
        assert!(json_str.contains("\"pid\":1"));
    }

    #[test]
    fn test_resolve_all_empty() {
        let entries: Vec<SocketEntry> = vec![];
        let events = resolve_all(&entries);
        assert!(events.is_empty());
    }
}
