// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Connection state machine — the lifecycle of a network connection.
//!
//! Tier: T2-C (ς State + σ Sequence + ∂ Boundary)
//!
//! Every connection — WiFi association, cellular attach, VPN tunnel —
//! follows the same state machine. The transitions are ordered (σ),
//! the states are discrete (ς), and the boundaries (∂) determine
//! when transitions are valid.

use crate::interface::{InterfaceId, InterfaceType, IpAddr};
use serde::{Deserialize, Serialize};

/// Connection state — the lifecycle of a network connection.
///
/// Tier: T2-P (ς State — connection lifecycle)
///
/// ```text
/// Disconnected → Connecting → Authenticating → Configuring → Connected
///       ↑              │              │              │           │
///       └──────────────┴──────────────┴──────────────┴───────────┘
///                              (any failure → Disconnected)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Not connected to any network.
    Disconnected,
    /// Attempting to establish link (scanning, associating).
    Connecting,
    /// Link established, authenticating (WPA handshake, VPN auth).
    Authenticating,
    /// Authenticated, configuring (DHCP, address assignment).
    Configuring,
    /// Fully connected with IP address and routes.
    Connected,
    /// Connection is being torn down gracefully.
    Disconnecting,
    /// Connection failed (transient — will retry or go to Disconnected).
    Failed,
}

impl ConnectionState {
    /// Whether this state has network connectivity.
    pub const fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Whether a transition is in progress.
    pub const fn is_transitioning(&self) -> bool {
        matches!(
            self,
            Self::Connecting | Self::Authenticating | Self::Configuring | Self::Disconnecting
        )
    }

    /// Whether this is a terminal failure state.
    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting",
            Self::Authenticating => "Authenticating",
            Self::Configuring => "Configuring",
            Self::Connected => "Connected",
            Self::Disconnecting => "Disconnecting",
            Self::Failed => "Failed",
        }
    }
}

/// Reason for a connection failure.
///
/// Tier: T2-P (Σ Sum — enumeration of failure modes)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureReason {
    /// Network not found (SSID doesn't exist).
    NetworkNotFound,
    /// Authentication failed (wrong password, certificate rejected).
    AuthenticationFailed,
    /// DHCP configuration failed (no address assigned).
    ConfigurationFailed,
    /// Connection timed out.
    Timeout,
    /// Signal lost during connection.
    SignalLost,
    /// Manually disconnected by user or system.
    ManualDisconnect,
    /// Hardware error.
    HardwareError,
    /// Other failure with description.
    Other(String),
}

impl FailureReason {
    /// Whether this failure is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout | Self::SignalLost | Self::ConfigurationFailed
        )
    }

    /// Human-readable label.
    pub fn label(&self) -> &str {
        match self {
            Self::NetworkNotFound => "Network not found",
            Self::AuthenticationFailed => "Authentication failed",
            Self::ConfigurationFailed => "Configuration failed",
            Self::Timeout => "Connection timed out",
            Self::SignalLost => "Signal lost",
            Self::ManualDisconnect => "Disconnected",
            Self::HardwareError => "Hardware error",
            Self::Other(msg) => msg.as_str(),
        }
    }
}

/// A network connection — one active link on one interface.
///
/// Tier: T2-C (ς + σ + ∂ + μ — stateful, ordered, bounded, typed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Which interface this connection belongs to.
    pub interface_id: InterfaceId,
    /// Interface type (cached for quick access).
    pub interface_type: InterfaceType,
    /// Network name (SSID for WiFi, APN for cellular, interface name for ethernet).
    pub network_name: String,
    /// Current state.
    state: ConnectionState,
    /// Assigned IP address (set during Configuring → Connected).
    pub address: Option<IpAddr>,
    /// Gateway address.
    pub gateway: Option<IpAddr>,
    /// DNS servers.
    pub dns_servers: Vec<IpAddr>,
    /// Last failure reason (set when transitioning to Failed).
    pub last_failure: Option<FailureReason>,
    /// Number of consecutive connection attempts.
    pub retry_count: u32,
    /// Maximum retries before giving up.
    pub max_retries: u32,
}

impl Connection {
    /// Create a new disconnected connection.
    pub fn new(
        interface_id: InterfaceId,
        interface_type: InterfaceType,
        network_name: impl Into<String>,
    ) -> Self {
        Self {
            interface_id,
            interface_type,
            network_name: network_name.into(),
            state: ConnectionState::Disconnected,
            address: None,
            gateway: None,
            dns_servers: Vec::new(),
            last_failure: None,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Get current state.
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Whether this connection is fully connected.
    pub fn is_connected(&self) -> bool {
        self.state.is_connected()
    }

    /// Whether this connection is in transition.
    pub fn is_transitioning(&self) -> bool {
        self.state.is_transitioning()
    }

    // ── State transitions ──

    /// Begin connecting (Disconnected/Failed → Connecting).
    pub fn connect(&mut self) -> Result<(), ConnectionError> {
        match self.state {
            ConnectionState::Disconnected | ConnectionState::Failed => {
                self.state = ConnectionState::Connecting;
                self.last_failure = None;
                Ok(())
            }
            _ => Err(ConnectionError::InvalidTransition {
                from: self.state,
                to: ConnectionState::Connecting,
            }),
        }
    }

    /// Authentication phase reached (Connecting → Authenticating).
    pub fn begin_auth(&mut self) -> Result<(), ConnectionError> {
        if self.state == ConnectionState::Connecting {
            self.state = ConnectionState::Authenticating;
            Ok(())
        } else {
            Err(ConnectionError::InvalidTransition {
                from: self.state,
                to: ConnectionState::Authenticating,
            })
        }
    }

    /// Configuration phase reached (Authenticating → Configuring).
    pub fn begin_config(&mut self) -> Result<(), ConnectionError> {
        if self.state == ConnectionState::Authenticating {
            self.state = ConnectionState::Configuring;
            Ok(())
        } else {
            Err(ConnectionError::InvalidTransition {
                from: self.state,
                to: ConnectionState::Configuring,
            })
        }
    }

    /// Connection established (Configuring → Connected).
    pub fn establish(
        &mut self,
        address: IpAddr,
        gateway: Option<IpAddr>,
        dns: Vec<IpAddr>,
    ) -> Result<(), ConnectionError> {
        if self.state == ConnectionState::Configuring {
            self.address = Some(address);
            self.gateway = gateway;
            self.dns_servers = dns;
            self.state = ConnectionState::Connected;
            self.retry_count = 0;
            Ok(())
        } else {
            Err(ConnectionError::InvalidTransition {
                from: self.state,
                to: ConnectionState::Connected,
            })
        }
    }

    /// Begin disconnecting (Connected → Disconnecting).
    pub fn begin_disconnect(&mut self) -> Result<(), ConnectionError> {
        if self.state == ConnectionState::Connected {
            self.state = ConnectionState::Disconnecting;
            Ok(())
        } else {
            Err(ConnectionError::InvalidTransition {
                from: self.state,
                to: ConnectionState::Disconnecting,
            })
        }
    }

    /// Complete disconnection (Disconnecting → Disconnected).
    pub fn complete_disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
        self.address = None;
        self.gateway = None;
        self.dns_servers.clear();
    }

    /// Mark connection as failed (any state → Failed).
    pub fn fail(&mut self, reason: FailureReason) {
        self.last_failure = Some(reason);
        self.state = ConnectionState::Failed;
        self.retry_count += 1;
        self.address = None;
        self.gateway = None;
        self.dns_servers.clear();
    }

    /// Whether the connection should retry.
    pub fn should_retry(&self) -> bool {
        self.state == ConnectionState::Failed
            && self.retry_count < self.max_retries
            && self
                .last_failure
                .as_ref()
                .is_some_and(FailureReason::is_retryable)
    }

    /// Force disconnect from any state.
    pub fn force_disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
        self.address = None;
        self.gateway = None;
        self.dns_servers.clear();
        self.retry_count = 0;
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let addr_str = self
            .address
            .as_ref()
            .map_or("no address".to_string(), IpAddr::to_string_repr);
        format!(
            "{} ({}) [{}] {}",
            self.network_name,
            self.interface_type.label(),
            self.state.label(),
            addr_str,
        )
    }
}

/// Connection state machine errors.
///
/// Tier: T2-P (∂ Boundary — constraint violations)
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConnectionError {
    /// Invalid state transition attempted.
    #[error("invalid transition: {from:?} → {to:?}")]
    InvalidTransition {
        from: ConnectionState,
        to: ConnectionState,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conn() -> Connection {
        Connection::new(
            InterfaceId::new("wlan0"),
            InterfaceType::WiFi,
            "HomeNetwork",
        )
    }

    #[test]
    fn new_connection_disconnected() {
        let c = make_conn();
        assert_eq!(c.state(), ConnectionState::Disconnected);
        assert!(!c.is_connected());
        assert!(!c.is_transitioning());
    }

    #[test]
    fn full_connection_lifecycle() {
        let mut c = make_conn();

        // Connect
        assert!(c.connect().is_ok());
        assert_eq!(c.state(), ConnectionState::Connecting);
        assert!(c.is_transitioning());

        // Authenticate
        assert!(c.begin_auth().is_ok());
        assert_eq!(c.state(), ConnectionState::Authenticating);

        // Configure
        assert!(c.begin_config().is_ok());
        assert_eq!(c.state(), ConnectionState::Configuring);

        // Establish
        assert!(
            c.establish(
                IpAddr::v4(192, 168, 1, 100),
                Some(IpAddr::v4(192, 168, 1, 1)),
                vec![IpAddr::v4(8, 8, 8, 8)],
            )
            .is_ok()
        );
        assert_eq!(c.state(), ConnectionState::Connected);
        assert!(c.is_connected());
        assert!(c.address.is_some());

        // Disconnect
        assert!(c.begin_disconnect().is_ok());
        assert_eq!(c.state(), ConnectionState::Disconnecting);
        c.complete_disconnect();
        assert_eq!(c.state(), ConnectionState::Disconnected);
        assert!(c.address.is_none());
    }

    #[test]
    fn invalid_transition_blocked() {
        let mut c = make_conn();
        // Can't authenticate from Disconnected
        assert!(c.begin_auth().is_err());
        // Can't establish from Disconnected
        assert!(c.establish(IpAddr::v4(0, 0, 0, 0), None, vec![]).is_err());
        // Can't disconnect from Disconnected
        assert!(c.begin_disconnect().is_err());
    }

    #[test]
    fn connection_failure() {
        let mut c = make_conn();
        assert!(c.connect().is_ok());
        c.fail(FailureReason::Timeout);
        assert_eq!(c.state(), ConnectionState::Failed);
        assert!(c.state().is_failed());
        assert_eq!(c.retry_count, 1);
    }

    #[test]
    fn retry_on_retryable_failure() {
        let mut c = make_conn();
        assert!(c.connect().is_ok());
        c.fail(FailureReason::Timeout);
        assert!(c.should_retry());
    }

    #[test]
    fn no_retry_on_auth_failure() {
        let mut c = make_conn();
        assert!(c.connect().is_ok());
        c.fail(FailureReason::AuthenticationFailed);
        assert!(!c.should_retry());
    }

    #[test]
    fn max_retries_exceeded() {
        let mut c = make_conn();
        c.max_retries = 2;
        for _ in 0..3 {
            assert!(c.connect().is_ok());
            c.fail(FailureReason::Timeout);
        }
        assert!(!c.should_retry()); // 3 retries, max is 2
    }

    #[test]
    fn force_disconnect() {
        let mut c = make_conn();
        assert!(c.connect().is_ok());
        assert!(c.begin_auth().is_ok());
        c.force_disconnect();
        assert_eq!(c.state(), ConnectionState::Disconnected);
        assert_eq!(c.retry_count, 0);
    }

    #[test]
    fn reconnect_after_failure() {
        let mut c = make_conn();
        assert!(c.connect().is_ok());
        c.fail(FailureReason::SignalLost);
        assert!(c.connect().is_ok()); // Can reconnect from Failed
    }

    #[test]
    fn connection_summary() {
        let mut c = make_conn();
        assert!(c.connect().is_ok());
        assert!(c.begin_auth().is_ok());
        assert!(c.begin_config().is_ok());
        assert!(c.establish(IpAddr::v4(10, 0, 0, 5), None, vec![]).is_ok());
        let s = c.summary();
        assert!(s.contains("HomeNetwork"));
        assert!(s.contains("WiFi"));
        assert!(s.contains("Connected"));
        assert!(s.contains("10.0.0.5"));
    }

    #[test]
    fn failure_reason_labels() {
        assert_eq!(FailureReason::Timeout.label(), "Connection timed out");
        assert_eq!(
            FailureReason::AuthenticationFailed.label(),
            "Authentication failed"
        );
    }

    #[test]
    fn connection_state_labels() {
        assert_eq!(ConnectionState::Connected.label(), "Connected");
        assert_eq!(ConnectionState::Authenticating.label(), "Authenticating");
    }
}
