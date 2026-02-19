// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! OS-level error types.
//!
//! Tier: T2-P (∂ Boundary — error boundaries between OS layers)

use nexcore_pal::PalError;

/// OS-level errors.
///
/// Tier: T2-C (∂ Boundary + Σ Sum — composite OS error)
#[derive(Debug)]
pub enum OsError {
    /// Platform abstraction layer error.
    Platform(PalError),
    /// Service lifecycle error.
    Service(ServiceError),
    /// Boot sequence error.
    Boot(BootError),
    /// State machine kernel error.
    StateKernel(String),
    /// IPC (cytokine) error.
    Ipc(String),
    /// Security (clearance) error.
    Security(String),
    /// Energy budget error.
    Energy(String),
    /// Vault (encrypted storage) error.
    Vault(crate::vault::VaultError),
    /// Authentication / user management error.
    Auth(crate::user::AuthError),
}

/// Service lifecycle errors.
///
/// Tier: T2-P (ς State — service state transition failures)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    /// Service not found by name.
    NotFound(String),
    /// Service already running.
    AlreadyRunning(String),
    /// Service failed to start.
    StartFailed(String),
    /// Service failed to stop.
    StopFailed(String),
    /// Dependency not satisfied.
    DependencyMissing { service: String, requires: String },
}

/// Boot sequence errors.
///
/// Tier: T2-P (σ Sequence — boot sequence failure points)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootError {
    /// PAL initialization failed.
    PalInitFailed(String),
    /// State kernel failed to start.
    KernelInitFailed(String),
    /// Critical service failed to start.
    CriticalServiceFailed(String),
    /// Boot sequence timed out.
    Timeout,
    /// Secure boot chain verification failed.
    SecureBootFailed(String),
}

impl core::fmt::Display for OsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Platform(e) => write!(f, "platform: {e}"),
            Self::Service(e) => write!(f, "service: {e}"),
            Self::Boot(e) => write!(f, "boot: {e}"),
            Self::StateKernel(e) => write!(f, "state kernel: {e}"),
            Self::Ipc(e) => write!(f, "ipc: {e}"),
            Self::Security(e) => write!(f, "security: {e}"),
            Self::Energy(e) => write!(f, "energy: {e}"),
            Self::Vault(e) => write!(f, "vault: {e}"),
            Self::Auth(e) => write!(f, "auth: {e}"),
        }
    }
}

impl From<crate::vault::VaultError> for OsError {
    fn from(e: crate::vault::VaultError) -> Self {
        Self::Vault(e)
    }
}

impl core::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "service not found: {name}"),
            Self::AlreadyRunning(name) => write!(f, "already running: {name}"),
            Self::StartFailed(name) => write!(f, "start failed: {name}"),
            Self::StopFailed(name) => write!(f, "stop failed: {name}"),
            Self::DependencyMissing { service, requires } => {
                write!(f, "{service} requires {requires}")
            }
        }
    }
}

impl core::fmt::Display for BootError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PalInitFailed(msg) => write!(f, "PAL init failed: {msg}"),
            Self::KernelInitFailed(msg) => write!(f, "kernel init failed: {msg}"),
            Self::CriticalServiceFailed(msg) => write!(f, "critical service failed: {msg}"),
            Self::Timeout => write!(f, "boot timeout"),
            Self::SecureBootFailed(msg) => write!(f, "secure boot failed: {msg}"),
        }
    }
}

impl From<PalError> for OsError {
    fn from(e: PalError) -> Self {
        Self::Platform(e)
    }
}

impl From<ServiceError> for OsError {
    fn from(e: ServiceError) -> Self {
        Self::Service(e)
    }
}

impl From<BootError> for OsError {
    fn from(e: BootError) -> Self {
        Self::Boot(e)
    }
}

impl From<crate::user::AuthError> for OsError {
    fn from(e: crate::user::AuthError) -> Self {
        Self::Auth(e)
    }
}
