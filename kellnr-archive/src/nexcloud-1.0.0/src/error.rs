#![allow(missing_docs)]

use std::path::PathBuf;

/// All errors that can occur in NexCloud operations.
///
/// Tier: T2-C (∂ Boundary + ∅ Void + ς State)
/// Boundary violations (bad config, missing files), void results (spawn failures),
/// and state transitions (shutdown) all converge here.
#[derive(Debug, thiserror::Error)]
pub enum NexCloudError {
    #[error("manifest parse error: {0}")]
    ManifestParse(String),

    #[error("manifest validation: {0}")]
    ManifestValidation(String),

    #[error("binary not found: {path}")]
    BinaryNotFound { path: PathBuf },

    #[error("process spawn failed for '{name}': {reason}")]
    ProcessSpawn { name: String, reason: String },

    #[error("process '{name}' exceeded max restarts ({max})")]
    MaxRestartsExceeded { name: String, max: u32 },

    #[error("health check failed for '{name}': {reason}")]
    HealthCheck { name: String, reason: String },

    #[error("route error: {0}")]
    ProxyRoute(String),

    #[error("TLS config error: {0}")]
    TlsConfig(String),

    #[error("shutdown error: {0}")]
    Shutdown(String),

    #[error("service '{name}' not found")]
    ServiceNotFound { name: String },

    #[error("dependency cycle detected: {cycle}")]
    DependencyCycle { cycle: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience Result type for NexCloud.
pub type Result<T> = std::result::Result<T, NexCloudError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_manifest_parse() {
        let e = NexCloudError::ManifestParse("bad toml".to_string());
        assert_eq!(format!("{e}"), "manifest parse error: bad toml");
    }

    #[test]
    fn error_display_manifest_validation() {
        let e = NexCloudError::ManifestValidation("dup port".to_string());
        assert_eq!(format!("{e}"), "manifest validation: dup port");
    }

    #[test]
    fn error_display_binary_not_found() {
        let e = NexCloudError::BinaryNotFound {
            path: PathBuf::from("/bin/nope"),
        };
        assert!(format!("{e}").contains("/bin/nope"));
    }

    #[test]
    fn error_display_process_spawn() {
        let e = NexCloudError::ProcessSpawn {
            name: "api".to_string(),
            reason: "permission denied".to_string(),
        };
        let msg = format!("{e}");
        assert!(msg.contains("api"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn error_display_max_restarts() {
        let e = NexCloudError::MaxRestartsExceeded {
            name: "web".to_string(),
            max: 5,
        };
        let msg = format!("{e}");
        assert!(msg.contains("web"));
        assert!(msg.contains("5"));
    }

    #[test]
    fn error_display_health_check() {
        let e = NexCloudError::HealthCheck {
            name: "svc".to_string(),
            reason: "timeout".to_string(),
        };
        assert!(format!("{e}").contains("timeout"));
    }

    #[test]
    fn error_display_proxy_route() {
        let e = NexCloudError::ProxyRoute("bind failed".to_string());
        assert!(format!("{e}").contains("bind failed"));
    }

    #[test]
    fn error_display_tls_config() {
        let e = NexCloudError::TlsConfig("no cert".to_string());
        assert!(format!("{e}").contains("no cert"));
    }

    #[test]
    fn error_display_shutdown() {
        let e = NexCloudError::Shutdown("timeout".to_string());
        assert!(format!("{e}").contains("timeout"));
    }

    #[test]
    fn error_display_service_not_found() {
        let e = NexCloudError::ServiceNotFound {
            name: "ghost".to_string(),
        };
        assert!(format!("{e}").contains("ghost"));
    }

    #[test]
    fn error_display_cycle() {
        let e = NexCloudError::DependencyCycle {
            cycle: "a -> b -> a".to_string(),
        };
        assert!(format!("{e}").contains("a -> b -> a"));
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let e: NexCloudError = io_err.into();
        assert!(format!("{e}").contains("gone"));
    }

    #[test]
    fn error_debug_impl() {
        let e = NexCloudError::Shutdown("test".to_string());
        // Debug should not panic
        let _ = format!("{e:?}");
    }
}
