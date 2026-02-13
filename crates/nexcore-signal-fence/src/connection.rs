//! Connection scanner — reads /proc/net/tcp{,6} and /proc/net/udp{,6}.
//!
//! Tier: T2-P (σ Sequence + μ Mapping — sequential scan mapped to structured entries)
//!
//! Every active socket is an "observation" in signal theory terms.
//! The scanner converts raw /proc data into typed `SocketEntry` values.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

use crate::error::{FenceError, FenceResult};

/// TCP connection states from /proc/net/tcp `st` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TcpState {
    /// 01 — Connection established
    Established,
    /// 02 — SYN sent
    SynSent,
    /// 03 — SYN received
    SynRecv,
    /// 04 — FIN wait 1
    FinWait1,
    /// 05 — FIN wait 2
    FinWait2,
    /// 06 — Time wait
    TimeWait,
    /// 07 — Connection closed
    Close,
    /// 08 — Close wait
    CloseWait,
    /// 09 — Last ACK
    LastAck,
    /// 0A — Listen
    Listen,
    /// 0B — Closing
    Closing,
    /// Unknown state
    Unknown(u8),
}

impl TcpState {
    /// Parse from hex state value in /proc/net/tcp.
    pub fn from_hex(val: u8) -> Self {
        match val {
            0x01 => Self::Established,
            0x02 => Self::SynSent,
            0x03 => Self::SynRecv,
            0x04 => Self::FinWait1,
            0x05 => Self::FinWait2,
            0x06 => Self::TimeWait,
            0x07 => Self::Close,
            0x08 => Self::CloseWait,
            0x09 => Self::LastAck,
            0x0A => Self::Listen,
            0x0B => Self::Closing,
            other => Self::Unknown(other),
        }
    }

    /// Whether this state represents an active data-carrying connection.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Established | Self::SynSent | Self::SynRecv)
    }

    /// Whether this socket is a listening server socket.
    pub fn is_listening(&self) -> bool {
        matches!(self, Self::Listen)
    }
}

/// Network protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Tcp6,
    Udp,
    Udp6,
}

impl Protocol {
    /// Whether this is an IPv6 protocol variant.
    pub fn is_ipv6(&self) -> bool {
        matches!(self, Self::Tcp6 | Self::Udp6)
    }

    /// The /proc/net filename for this protocol.
    pub fn proc_path(&self) -> &'static str {
        match self {
            Self::Tcp => "/proc/net/tcp",
            Self::Tcp6 => "/proc/net/tcp6",
            Self::Udp => "/proc/net/udp",
            Self::Udp6 => "/proc/net/udp6",
        }
    }
}

/// A parsed socket entry from /proc/net/tcp or similar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocketEntry {
    /// Local IP address.
    pub local_addr: IpAddr,
    /// Local port number.
    pub local_port: u16,
    /// Remote IP address.
    pub remote_addr: IpAddr,
    /// Remote port number.
    pub remote_port: u16,
    /// Socket inode number (links to /proc/<pid>/fd/).
    pub inode: u64,
    /// TCP state (meaningful for TCP; UDP uses Established/Close).
    pub state: TcpState,
    /// Protocol that produced this entry.
    pub protocol: Protocol,
    /// UID of the socket owner.
    pub uid: u32,
}

/// Parse a hex-encoded IPv4 address from /proc/net/tcp format.
///
/// Format: 8 hex chars in network byte order (little-endian on x86).
/// Example: "0100007F" → 127.0.0.1
fn parse_ipv4_addr(hex: &str) -> FenceResult<Ipv4Addr> {
    if hex.len() != 8 {
        return Err(FenceError::ProcParse {
            file: "tcp".to_string(),
            line: 0,
            detail: format!("IPv4 hex must be 8 chars, got {}", hex.len()),
        });
    }
    let val = u32::from_str_radix(hex, 16).map_err(|e| FenceError::ProcParse {
        file: "tcp".to_string(),
        line: 0,
        detail: format!("invalid hex for IPv4: {e}"),
    })?;
    // /proc/net/tcp stores in host byte order (little-endian on x86)
    Ok(Ipv4Addr::from(val.to_be()))
}

/// Parse a hex-encoded IPv6 address from /proc/net/tcp6 format.
///
/// Format: 32 hex chars, 4 groups of 8 hex chars each in host byte order.
fn parse_ipv6_addr(hex: &str) -> FenceResult<Ipv6Addr> {
    if hex.len() != 32 {
        return Err(FenceError::ProcParse {
            file: "tcp6".to_string(),
            line: 0,
            detail: format!("IPv6 hex must be 32 chars, got {}", hex.len()),
        });
    }
    // IPv6 in /proc is stored as 4 groups of 32-bit words, each in host byte order
    // (same encoding as IPv4). We swap each word from host to network byte order.
    let mut octets = [0u8; 16];
    for group in 0..4 {
        let start = group * 8;
        let word =
            u32::from_str_radix(&hex[start..start + 8], 16).map_err(|e| FenceError::ProcParse {
                file: "tcp6".to_string(),
                line: 0,
                detail: format!("invalid hex for IPv6 group {group}: {e}"),
            })?;
        // Same as IPv4: swap from host byte order to network byte order
        let swapped = word.to_be();
        let bytes = swapped.to_be_bytes();
        let base = group * 4;
        octets[base] = bytes[0];
        octets[base + 1] = bytes[1];
        octets[base + 2] = bytes[2];
        octets[base + 3] = bytes[3];
    }
    Ok(Ipv6Addr::from(octets))
}

/// Parse an address:port pair like "0100007F:0050".
fn parse_addr_port(s: &str, is_ipv6: bool) -> FenceResult<(IpAddr, u16)> {
    let (addr_hex, port_hex) = s.split_once(':').ok_or_else(|| FenceError::ProcParse {
        file: "tcp".to_string(),
        line: 0,
        detail: format!("expected ADDR:PORT, got '{s}'"),
    })?;

    let addr: IpAddr = if is_ipv6 {
        parse_ipv6_addr(addr_hex)?.into()
    } else {
        parse_ipv4_addr(addr_hex)?.into()
    };

    let port = u16::from_str_radix(port_hex, 16).map_err(|e| FenceError::ProcParse {
        file: "tcp".to_string(),
        line: 0,
        detail: format!("invalid port hex '{port_hex}': {e}"),
    })?;

    Ok((addr, port))
}

/// Parse a single line from /proc/net/tcp or /proc/net/tcp6.
///
/// Format (whitespace-separated fields):
/// ```text
///   sl  local_address rem_address   st tx_queue:rx_queue tr:tm->when retrnsmt   uid  timeout inode
///    0: 0100007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345
/// ```
pub fn parse_proc_net_line(line: &str, protocol: Protocol) -> FenceResult<SocketEntry> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 10 {
        return Err(FenceError::ProcParse {
            file: protocol.proc_path().to_string(),
            line: 0,
            detail: format!("expected ≥10 fields, got {}", fields.len()),
        });
    }

    let is_ipv6 = protocol.is_ipv6();

    // Field 1: local address
    let (local_addr, local_port) = parse_addr_port(fields[1], is_ipv6)?;

    // Field 2: remote address
    let (remote_addr, remote_port) = parse_addr_port(fields[2], is_ipv6)?;

    // Field 3: state (hex)
    let state_val = u8::from_str_radix(fields[3], 16).map_err(|e| FenceError::ProcParse {
        file: protocol.proc_path().to_string(),
        line: 0,
        detail: format!("invalid state hex '{}': {e}", fields[3]),
    })?;
    let state = TcpState::from_hex(state_val);

    // Field 7: UID
    let uid = fields[7]
        .parse::<u32>()
        .map_err(|e| FenceError::ProcParse {
            file: protocol.proc_path().to_string(),
            line: 0,
            detail: format!("invalid uid '{}': {e}", fields[7]),
        })?;

    // Field 9: inode
    let inode = fields[9]
        .parse::<u64>()
        .map_err(|e| FenceError::ProcParse {
            file: protocol.proc_path().to_string(),
            line: 0,
            detail: format!("invalid inode '{}': {e}", fields[9]),
        })?;

    Ok(SocketEntry {
        local_addr,
        local_port,
        remote_addr,
        remote_port,
        inode,
        state,
        protocol,
        uid,
    })
}

/// Parse the full contents of a /proc/net/tcp-style file.
///
/// Skips the header line. Returns all successfully parsed entries.
pub fn parse_proc_net_content(content: &str, protocol: Protocol) -> Vec<SocketEntry> {
    content
        .lines()
        .skip(1) // skip header
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| parse_proc_net_line(line, protocol).ok())
        .collect()
}

/// Read and parse a /proc/net/ file for the given protocol.
///
/// Returns parsed socket entries. Errors if the file cannot be read.
pub fn scan_protocol(protocol: Protocol) -> FenceResult<Vec<SocketEntry>> {
    let content = std::fs::read_to_string(protocol.proc_path())
        .map_err(|e| FenceError::ProcRead(format!("{}: {e}", protocol.proc_path())))?;
    Ok(parse_proc_net_content(&content, protocol))
}

/// Scan all TCP and UDP sockets on the system.
///
/// Attempts all four protocols; skips any that fail to read.
pub fn scan_all() -> Vec<SocketEntry> {
    let protocols = [Protocol::Tcp, Protocol::Tcp6, Protocol::Udp, Protocol::Udp6];
    let mut entries = Vec::new();
    for proto in protocols {
        if let Ok(mut sockets) = scan_protocol(proto) {
            entries.append(&mut sockets);
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TCP: &str = r#"  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345 1 0000000000000000 100 0 0 10 0
   1: 0100007F:0CEA 0100007F:1F90 01 00000000:00000000 00:00000000 00000000  1000        0 67890 1 0000000000000000 100 0 0 10 0"#;

    #[test]
    fn test_parse_ipv4_addr() {
        // 0100007F = 127.0.0.1 in /proc little-endian format
        let addr = parse_ipv4_addr("0100007F").ok();
        assert!(addr.is_some());
        let addr = addr.unwrap_or(Ipv4Addr::UNSPECIFIED);
        assert_eq!(addr, Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn test_parse_ipv4_addr_zeros() {
        let addr = parse_ipv4_addr("00000000").ok();
        assert!(addr.is_some());
        let addr = addr.unwrap_or(Ipv4Addr::BROADCAST);
        assert_eq!(addr, Ipv4Addr::UNSPECIFIED);
    }

    #[test]
    fn test_parse_ipv4_addr_invalid_length() {
        let result = parse_ipv4_addr("0100");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ipv6_addr() {
        // All zeros
        let addr = parse_ipv6_addr("00000000000000000000000000000000").ok();
        assert!(addr.is_some());
        let addr = addr.unwrap_or(Ipv6Addr::LOCALHOST);
        assert_eq!(addr, Ipv6Addr::UNSPECIFIED);
    }

    #[test]
    fn test_parse_ipv6_loopback() {
        // ::1 in /proc format: 00000000000000000000000001000000
        let addr = parse_ipv6_addr("00000000000000000000000001000000").ok();
        assert!(addr.is_some());
        let addr = addr.unwrap_or(Ipv6Addr::UNSPECIFIED);
        assert_eq!(addr, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    }

    #[test]
    fn test_parse_ipv6_invalid_length() {
        let result = parse_ipv6_addr("0000");
        assert!(result.is_err());
    }

    #[test]
    fn test_tcp_state_from_hex() {
        assert_eq!(TcpState::from_hex(0x01), TcpState::Established);
        assert_eq!(TcpState::from_hex(0x0A), TcpState::Listen);
        assert!(matches!(TcpState::from_hex(0xFF), TcpState::Unknown(0xFF)));
    }

    #[test]
    fn test_tcp_state_is_active() {
        assert!(TcpState::Established.is_active());
        assert!(TcpState::SynSent.is_active());
        assert!(!TcpState::Listen.is_active());
        assert!(!TcpState::TimeWait.is_active());
    }

    #[test]
    fn test_tcp_state_is_listening() {
        assert!(TcpState::Listen.is_listening());
        assert!(!TcpState::Established.is_listening());
    }

    #[test]
    fn test_protocol_proc_path() {
        assert_eq!(Protocol::Tcp.proc_path(), "/proc/net/tcp");
        assert_eq!(Protocol::Tcp6.proc_path(), "/proc/net/tcp6");
        assert_eq!(Protocol::Udp.proc_path(), "/proc/net/udp");
        assert_eq!(Protocol::Udp6.proc_path(), "/proc/net/udp6");
    }

    #[test]
    fn test_protocol_is_ipv6() {
        assert!(!Protocol::Tcp.is_ipv6());
        assert!(Protocol::Tcp6.is_ipv6());
        assert!(!Protocol::Udp.is_ipv6());
        assert!(Protocol::Udp6.is_ipv6());
    }

    #[test]
    fn test_parse_proc_net_line_listening() {
        let line = "   0: 0100007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345 1 0000000000000000 100 0 0 10 0";
        let entry = parse_proc_net_line(line, Protocol::Tcp);
        assert!(entry.is_ok());
        let entry = entry.ok().unwrap_or_else(|| SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            local_port: 0,
            remote_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            remote_port: 0,
            inode: 0,
            state: TcpState::Close,
            protocol: Protocol::Tcp,
            uid: 0,
        });
        assert_eq!(entry.local_addr, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(entry.local_port, 53); // 0x0035 = 53
        assert_eq!(entry.state, TcpState::Listen);
        assert_eq!(entry.inode, 12345);
        assert_eq!(entry.uid, 0);
    }

    #[test]
    fn test_parse_proc_net_line_established() {
        let line = "   1: 0100007F:0CEA 0100007F:1F90 01 00000000:00000000 00:00000000 00000000  1000        0 67890 1 0000000000000000 100 0 0 10 0";
        let entry = parse_proc_net_line(line, Protocol::Tcp);
        assert!(entry.is_ok());
        let entry = entry.ok().unwrap_or_else(|| SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            local_port: 0,
            remote_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            remote_port: 0,
            inode: 0,
            state: TcpState::Close,
            protocol: Protocol::Tcp,
            uid: 0,
        });
        assert_eq!(entry.local_port, 3306); // 0x0CEA = 3306
        assert_eq!(entry.remote_port, 8080); // 0x1F90 = 8080
        assert_eq!(entry.state, TcpState::Established);
        assert_eq!(entry.uid, 1000);
        assert_eq!(entry.inode, 67890);
    }

    #[test]
    fn test_parse_proc_net_line_too_few_fields() {
        let line = "   0: 0100007F:0035 00000000:0000";
        let result = parse_proc_net_line(line, Protocol::Tcp);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_proc_net_content() {
        let entries = parse_proc_net_content(SAMPLE_TCP, Protocol::Tcp);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].state, TcpState::Listen);
        assert_eq!(entries[1].state, TcpState::Established);
    }

    #[test]
    fn test_parse_proc_net_content_empty() {
        let content = "  sl  local_address rem_address   st\n";
        let entries = parse_proc_net_content(content, Protocol::Tcp);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_addr_port_missing_colon() {
        let result = parse_addr_port("0100007F0035", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_socket_entry_serialization() {
        let entry = SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port: 8080,
            remote_addr: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            remote_port: 443,
            inode: 99999,
            state: TcpState::Established,
            protocol: Protocol::Tcp,
            uid: 1000,
        };
        let json = serde_json::to_string(&entry);
        assert!(json.is_ok());
        let json = json.ok().unwrap_or_default();
        assert!(json.contains("8080"));
        assert!(json.contains("99999"));
    }
}
