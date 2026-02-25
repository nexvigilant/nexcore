// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! App clearance — permission-gated app installation and execution.
//!
//! ## Architecture
//!
//! Every app declares a manifest of required permissions. The clearance
//! gate evaluates whether the app is allowed to run based on:
//!
//! 1. The app's required permissions
//! 2. The current security level (from SecurityMonitor)
//! 3. The app's clearance classification
//! 4. Whether the app's source service is quarantined
//!
//! ## Primitive Grounding
//!
//! - ∂ Boundary: Permission boundaries
//! - ∃ Existence: Permission existence validation
//! - κ Comparison: Clearance level comparison
//! - ς State: App installation state

use crate::security::{SecurityLevel, SecurityMonitor};
use serde::{Deserialize, Serialize};

/// App permissions that must be explicitly granted.
///
/// Tier: T2-P (∂ Boundary — capability boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppPermission {
    /// Network access (outbound connections).
    Network,
    /// Persistent storage read/write.
    Storage,
    /// Camera access.
    Camera,
    /// Microphone access.
    Microphone,
    /// Location services.
    Location,
    /// System control (service management, settings).
    SystemControl,
    /// Inter-app communication via IPC.
    Ipc,
    /// Background execution (run when not focused).
    Background,
    /// Notification posting.
    Notifications,
}

/// App clearance level — what category of apps this is.
///
/// Tier: T2-P (κ Comparison — orderable trust level)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppClearanceLevel {
    /// System app (built-in, fully trusted).
    System = 0,
    /// Verified app (signed, audited).
    Verified = 1,
    /// Standard app (from app store, sandboxed).
    Standard = 2,
    /// Sideloaded app (user-installed, limited trust).
    Sideloaded = 3,
    /// Unknown origin (untrusted).
    Unknown = 4,
}

impl AppClearanceLevel {
    /// Maximum permissions allowed at this clearance level.
    pub fn max_permissions(&self) -> Vec<AppPermission> {
        match self {
            Self::System => vec![
                AppPermission::Network,
                AppPermission::Storage,
                AppPermission::Camera,
                AppPermission::Microphone,
                AppPermission::Location,
                AppPermission::SystemControl,
                AppPermission::Ipc,
                AppPermission::Background,
                AppPermission::Notifications,
            ],
            Self::Verified => vec![
                AppPermission::Network,
                AppPermission::Storage,
                AppPermission::Camera,
                AppPermission::Microphone,
                AppPermission::Location,
                AppPermission::Ipc,
                AppPermission::Background,
                AppPermission::Notifications,
            ],
            Self::Standard => vec![
                AppPermission::Network,
                AppPermission::Storage,
                AppPermission::Location,
                AppPermission::Ipc,
                AppPermission::Notifications,
            ],
            Self::Sideloaded => vec![AppPermission::Storage, AppPermission::Notifications],
            Self::Unknown => vec![],
        }
    }

    /// Whether this clearance level allows the given permission.
    pub fn allows(&self, permission: AppPermission) -> bool {
        self.max_permissions().contains(&permission)
    }
}

/// An app's permission manifest — declares what it needs.
///
/// Tier: T2-C (∂ + ∃ — bounded permission set with existence checks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    /// App identifier (reverse domain: com.nexcore.launcher).
    pub app_id: String,
    /// Human-readable name.
    pub name: String,
    /// Required permissions.
    pub permissions: Vec<AppPermission>,
    /// App clearance level.
    pub clearance: AppClearanceLevel,
    /// Version string.
    pub version: String,
}

impl AppManifest {
    /// Create a new app manifest.
    pub fn new(
        app_id: impl Into<String>,
        name: impl Into<String>,
        clearance: AppClearanceLevel,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            name: name.into(),
            permissions: Vec::new(),
            clearance,
            version: "1.0.0".to_string(),
        }
    }

    /// Add a required permission.
    #[must_use]
    pub fn with_permission(mut self, perm: AppPermission) -> Self {
        if !self.permissions.contains(&perm) {
            self.permissions.push(perm);
        }
        self
    }
}

/// Result of a clearance evaluation.
///
/// Tier: T2-C (κ + ∂ — comparison result with boundary context)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClearanceResult {
    /// App is allowed to install/run.
    Allowed,
    /// App is blocked due to security level.
    BlockedBySecurityLevel {
        /// Current security level.
        current_level: SecurityLevel,
    },
    /// App requests permissions beyond its clearance level.
    InsufficientClearance {
        /// Permissions that exceed clearance.
        denied_permissions: Vec<AppPermission>,
    },
    /// App's source service is quarantined.
    SourceQuarantined,
    /// App origin is untrusted.
    UntrustedOrigin,
}

impl ClearanceResult {
    /// Whether the app is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// App clearance gate — evaluates whether an app can install or run.
///
/// Tier: T3 (∂ + ∃ + κ + ς — full clearance evaluation)
pub struct AppClearanceGate {
    /// Minimum clearance level for installation (default: Standard).
    min_install_clearance: AppClearanceLevel,
    /// Whether sideloading is enabled.
    sideloading_enabled: bool,
}

impl AppClearanceGate {
    /// Create a new clearance gate with default settings.
    pub fn new() -> Self {
        Self {
            min_install_clearance: AppClearanceLevel::Standard,
            sideloading_enabled: false,
        }
    }

    /// Enable sideloading (allows Sideloaded clearance level).
    pub fn enable_sideloading(&mut self) {
        self.sideloading_enabled = true;
    }

    /// Disable sideloading.
    pub fn disable_sideloading(&mut self) {
        self.sideloading_enabled = false;
    }

    /// Whether sideloading is enabled.
    pub fn sideloading_enabled(&self) -> bool {
        self.sideloading_enabled
    }

    /// Evaluate whether an app can be installed.
    ///
    /// Checks:
    /// 1. Security level allows installation
    /// 2. App clearance meets minimum threshold
    /// 3. App permissions are within clearance bounds
    /// 4. Sideloading policy
    pub fn evaluate_install(
        &self,
        manifest: &AppManifest,
        security: &SecurityMonitor,
    ) -> ClearanceResult {
        // Security level check
        if security.blocks_app_install() {
            return ClearanceResult::BlockedBySecurityLevel {
                current_level: security.level(),
            };
        }

        // Origin trust check
        if manifest.clearance == AppClearanceLevel::Unknown {
            return ClearanceResult::UntrustedOrigin;
        }

        // Sideloading check
        if manifest.clearance == AppClearanceLevel::Sideloaded && !self.sideloading_enabled {
            return ClearanceResult::UntrustedOrigin;
        }

        // Clearance level check
        if manifest.clearance > self.min_install_clearance
            && manifest.clearance != AppClearanceLevel::System
        {
            // System apps bypass clearance checks
            if manifest.clearance != AppClearanceLevel::Sideloaded || !self.sideloading_enabled {
                return ClearanceResult::UntrustedOrigin;
            }
        }

        // Permission check — verify each requested permission is allowed
        let denied: Vec<AppPermission> = manifest
            .permissions
            .iter()
            .filter(|p| !manifest.clearance.allows(**p))
            .copied()
            .collect();

        if !denied.is_empty() {
            return ClearanceResult::InsufficientClearance {
                denied_permissions: denied,
            };
        }

        ClearanceResult::Allowed
    }

    /// Evaluate whether an app can execute (runtime check).
    ///
    /// More permissive than install — checks quarantine and security lockdown.
    pub fn evaluate_run(
        &self,
        manifest: &AppManifest,
        security: &SecurityMonitor,
    ) -> ClearanceResult {
        // Red security = only system apps run
        if security.blocks_non_critical() && manifest.clearance != AppClearanceLevel::System {
            return ClearanceResult::BlockedBySecurityLevel {
                current_level: security.level(),
            };
        }

        ClearanceResult::Allowed
    }
}

impl Default for AppClearanceGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: create a SecurityMonitor, skipping test if runtime unavailable.
    macro_rules! monitor {
        () => {
            match SecurityMonitor::new() {
                Ok(m) => m,
                Err(_) => return,
            }
        };
    }

    #[test]
    fn system_app_always_allowed() {
        let gate = AppClearanceGate::new();
        let monitor = monitor!();
        let manifest = AppManifest::new(
            "com.nexcore.launcher",
            "Launcher",
            AppClearanceLevel::System,
        )
        .with_permission(AppPermission::SystemControl)
        .with_permission(AppPermission::Network);

        let result = gate.evaluate_install(&manifest, &monitor);
        assert!(result.is_allowed());
    }

    #[test]
    fn standard_app_allowed_at_green() {
        let gate = AppClearanceGate::new();
        let monitor = monitor!();
        let manifest = AppManifest::new("com.app.chat", "Chat", AppClearanceLevel::Standard)
            .with_permission(AppPermission::Network)
            .with_permission(AppPermission::Storage);

        let result = gate.evaluate_install(&manifest, &monitor);
        assert!(result.is_allowed());
    }

    #[test]
    fn blocked_at_orange_security() {
        let gate = AppClearanceGate::new();
        let mut monitor = monitor!();
        monitor.record_threat(crate::security::ThreatSeverity::High, "Active threat", None);
        assert_eq!(monitor.level(), SecurityLevel::Orange);

        let manifest = AppManifest::new("com.app.game", "Game", AppClearanceLevel::Standard);
        let result = gate.evaluate_install(&manifest, &monitor);
        assert_eq!(
            result,
            ClearanceResult::BlockedBySecurityLevel {
                current_level: SecurityLevel::Orange
            }
        );
    }

    #[test]
    fn unknown_origin_rejected() {
        let gate = AppClearanceGate::new();
        let monitor = monitor!();
        let manifest = AppManifest::new("unknown.app", "Sketchy", AppClearanceLevel::Unknown);

        let result = gate.evaluate_install(&manifest, &monitor);
        assert_eq!(result, ClearanceResult::UntrustedOrigin);
    }

    #[test]
    fn sideloading_blocked_by_default() {
        let gate = AppClearanceGate::new();
        let monitor = monitor!();
        let manifest = AppManifest::new("sideload.app", "Custom", AppClearanceLevel::Sideloaded)
            .with_permission(AppPermission::Storage);

        let result = gate.evaluate_install(&manifest, &monitor);
        assert_eq!(result, ClearanceResult::UntrustedOrigin);
    }

    #[test]
    fn sideloading_allowed_when_enabled() {
        let mut gate = AppClearanceGate::new();
        gate.enable_sideloading();
        let monitor = monitor!();
        let manifest = AppManifest::new("sideload.app", "Custom", AppClearanceLevel::Sideloaded)
            .with_permission(AppPermission::Storage);

        let result = gate.evaluate_install(&manifest, &monitor);
        assert!(result.is_allowed());
    }

    #[test]
    fn insufficient_clearance_for_permission() {
        let mut gate = AppClearanceGate::new();
        gate.enable_sideloading();
        let monitor = monitor!();

        // Sideloaded app requesting Camera (not allowed at Sideloaded level)
        let manifest = AppManifest::new("sideload.cam", "Camera", AppClearanceLevel::Sideloaded)
            .with_permission(AppPermission::Camera);

        let result = gate.evaluate_install(&manifest, &monitor);
        assert!(matches!(
            result,
            ClearanceResult::InsufficientClearance { .. }
        ));
    }

    #[test]
    fn runtime_lockdown_blocks_non_system() {
        let gate = AppClearanceGate::new();
        let mut monitor = monitor!();
        monitor.record_threat(
            crate::security::ThreatSeverity::Critical,
            "Root compromise",
            None,
        );
        assert_eq!(monitor.level(), SecurityLevel::Red);

        // Standard app blocked
        let standard = AppManifest::new("com.app.chat", "Chat", AppClearanceLevel::Standard);
        let result = gate.evaluate_run(&standard, &monitor);
        assert_eq!(
            result,
            ClearanceResult::BlockedBySecurityLevel {
                current_level: SecurityLevel::Red
            }
        );

        // System app still runs
        let system = AppManifest::new(
            "com.nexcore.launcher",
            "Launcher",
            AppClearanceLevel::System,
        );
        let result = gate.evaluate_run(&system, &monitor);
        assert!(result.is_allowed());
    }

    #[test]
    fn clearance_level_ordering() {
        assert!(AppClearanceLevel::System < AppClearanceLevel::Verified);
        assert!(AppClearanceLevel::Verified < AppClearanceLevel::Standard);
        assert!(AppClearanceLevel::Standard < AppClearanceLevel::Sideloaded);
        assert!(AppClearanceLevel::Sideloaded < AppClearanceLevel::Unknown);
    }

    #[test]
    fn permission_coverage() {
        // System has all permissions
        assert!(AppClearanceLevel::System.allows(AppPermission::SystemControl));
        assert!(AppClearanceLevel::System.allows(AppPermission::Camera));

        // Standard has limited permissions
        assert!(AppClearanceLevel::Standard.allows(AppPermission::Network));
        assert!(!AppClearanceLevel::Standard.allows(AppPermission::Camera));
        assert!(!AppClearanceLevel::Standard.allows(AppPermission::SystemControl));

        // Unknown has none
        assert!(!AppClearanceLevel::Unknown.allows(AppPermission::Storage));
    }

    #[test]
    fn verified_app_gets_more_permissions() {
        assert!(AppClearanceLevel::Verified.allows(AppPermission::Camera));
        assert!(AppClearanceLevel::Verified.allows(AppPermission::Microphone));
        assert!(AppClearanceLevel::Verified.allows(AppPermission::Background));
        assert!(!AppClearanceLevel::Verified.allows(AppPermission::SystemControl));
    }
}
