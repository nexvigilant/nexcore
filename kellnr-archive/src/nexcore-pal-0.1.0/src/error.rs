// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! PAL error types — zero-dependency, `no_std` compatible.

use core::fmt;

/// Platform Abstraction Layer error.
///
/// Tier: T2-P (∂ Boundary — error boundaries between platform layers)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PalError {
    /// Display subsystem error.
    Display(DisplayError),
    /// Input subsystem error.
    Input(InputError),
    /// Network subsystem error.
    Network(NetworkError),
    /// Storage subsystem error.
    Storage(StorageError),
    /// Haptics subsystem error.
    Haptics(HapticsError),
    /// Power subsystem error.
    Power(PowerError),
    /// Platform-level initialization error.
    Init(InitError),
}

/// Display subsystem errors.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayError {
    /// No display device found.
    NotFound,
    /// Failed to present framebuffer.
    PresentFailed,
    /// Unsupported resolution requested.
    UnsupportedResolution { width: u32, height: u32 },
    /// Mode setting failed.
    ModeSetFailed,
}

/// Input subsystem errors.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    /// No input device found.
    NotFound,
    /// Failed to poll events.
    PollFailed,
    /// Device disconnected.
    Disconnected,
}

/// Network subsystem errors.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    /// No network interface available.
    NoInterface,
    /// Connection refused.
    ConnectionRefused,
    /// Send failed.
    SendFailed,
    /// Receive failed.
    ReceiveFailed,
    /// DNS resolution failed.
    DnsResolutionFailed,
    /// Timeout.
    Timeout,
}

/// Storage subsystem errors.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// Path not found.
    NotFound,
    /// Permission denied.
    PermissionDenied,
    /// Storage full.
    Full,
    /// I/O error.
    IoError,
    /// Corrupt data.
    Corrupt,
}

/// Haptics subsystem errors.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HapticsError {
    /// No haptic device available.
    NotAvailable,
    /// Pattern too long or complex.
    PatternInvalid,
}

/// Power subsystem errors.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerError {
    /// Power information unavailable.
    Unavailable,
    /// Sensor read failed.
    SensorFailed,
}

/// Platform initialization errors.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitError {
    /// Hardware not supported.
    UnsupportedHardware,
    /// Required subsystem missing.
    MissingSubsystem(&'static str),
    /// Configuration invalid.
    InvalidConfig,
}

impl fmt::Display for PalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Display(e) => write!(f, "display: {e}"),
            Self::Input(e) => write!(f, "input: {e}"),
            Self::Network(e) => write!(f, "network: {e}"),
            Self::Storage(e) => write!(f, "storage: {e}"),
            Self::Haptics(e) => write!(f, "haptics: {e}"),
            Self::Power(e) => write!(f, "power: {e}"),
            Self::Init(e) => write!(f, "init: {e}"),
        }
    }
}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "display not found"),
            Self::PresentFailed => write!(f, "present failed"),
            Self::UnsupportedResolution { width, height } => {
                write!(f, "unsupported resolution {width}x{height}")
            }
            Self::ModeSetFailed => write!(f, "mode set failed"),
        }
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "input device not found"),
            Self::PollFailed => write!(f, "poll failed"),
            Self::Disconnected => write!(f, "device disconnected"),
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInterface => write!(f, "no network interface"),
            Self::ConnectionRefused => write!(f, "connection refused"),
            Self::SendFailed => write!(f, "send failed"),
            Self::ReceiveFailed => write!(f, "receive failed"),
            Self::DnsResolutionFailed => write!(f, "DNS resolution failed"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::Full => write!(f, "storage full"),
            Self::IoError => write!(f, "I/O error"),
            Self::Corrupt => write!(f, "corrupt data"),
        }
    }
}

impl fmt::Display for HapticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAvailable => write!(f, "haptics not available"),
            Self::PatternInvalid => write!(f, "pattern invalid"),
        }
    }
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => write!(f, "power info unavailable"),
            Self::SensorFailed => write!(f, "sensor read failed"),
        }
    }
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedHardware => write!(f, "unsupported hardware"),
            Self::MissingSubsystem(name) => write!(f, "missing subsystem: {name}"),
            Self::InvalidConfig => write!(f, "invalid configuration"),
        }
    }
}

// Implement From conversions for subsystem errors → PalError
impl From<DisplayError> for PalError {
    fn from(e: DisplayError) -> Self {
        Self::Display(e)
    }
}

impl From<InputError> for PalError {
    fn from(e: InputError) -> Self {
        Self::Input(e)
    }
}

impl From<NetworkError> for PalError {
    fn from(e: NetworkError) -> Self {
        Self::Network(e)
    }
}

impl From<StorageError> for PalError {
    fn from(e: StorageError) -> Self {
        Self::Storage(e)
    }
}

impl From<HapticsError> for PalError {
    fn from(e: HapticsError) -> Self {
        Self::Haptics(e)
    }
}

impl From<PowerError> for PalError {
    fn from(e: PowerError) -> Self {
        Self::Power(e)
    }
}

impl From<InitError> for PalError {
    fn from(e: InitError) -> Self {
        Self::Init(e)
    }
}
