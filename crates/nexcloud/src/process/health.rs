use std::time::Duration;

/// Result of a health check probe.
///
/// Tier: T2-P (κ Comparison) — binary comparison against health threshold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
    Unreachable(String),
}

/// HTTP health checker that polls a service endpoint.
///
/// Tier: T2-C (κ Comparison + ν Frequency + ∂ Boundary)
/// Periodically compares service response against health boundary.
pub struct HealthChecker {
    client: reqwest::Client,
    check_timeout: Duration,
}

impl HealthChecker {
    /// Create a new health checker with the given timeout per check.
    pub fn new(check_timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(check_timeout)
            .no_proxy()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            check_timeout,
        }
    }

    /// Probe a service at the given address and health path.
    pub async fn check(&self, port: u16, health_path: &str) -> HealthStatus {
        let url = format!("http://127.0.0.1:{port}{health_path}");
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Unhealthy(format!("status {}", resp.status()))
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    HealthStatus::Unreachable(format!(
                        "timeout after {}ms",
                        self.check_timeout.as_millis()
                    ))
                } else if e.is_connect() {
                    HealthStatus::Unreachable("connection refused".to_string())
                } else {
                    HealthStatus::Unreachable(e.to_string())
                }
            }
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_debug() {
        let h = HealthStatus::Healthy;
        assert_eq!(format!("{h:?}"), "Healthy");

        let u = HealthStatus::Unhealthy("bad".to_string());
        assert!(format!("{u:?}").contains("bad"));

        let r = HealthStatus::Unreachable("refused".to_string());
        assert!(format!("{r:?}").contains("refused"));
    }

    #[test]
    fn health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(
            HealthStatus::Healthy,
            HealthStatus::Unhealthy("x".to_string())
        );
        assert_ne!(
            HealthStatus::Unhealthy("a".to_string()),
            HealthStatus::Unhealthy("b".to_string())
        );
        assert_eq!(
            HealthStatus::Unreachable("same".to_string()),
            HealthStatus::Unreachable("same".to_string())
        );
    }

    #[test]
    fn health_status_clone() {
        let original = HealthStatus::Unhealthy("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn health_checker_custom_timeout() {
        let checker = HealthChecker::new(Duration::from_millis(100));
        assert_eq!(checker.check_timeout, Duration::from_millis(100));
    }

    #[test]
    fn health_checker_default() {
        let checker = HealthChecker::default();
        assert_eq!(checker.check_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn health_check_unreachable_port() {
        // Port 1 is almost certainly not listening
        let checker = HealthChecker::new(Duration::from_millis(500));
        let status = checker.check(1, "/health").await;
        match status {
            HealthStatus::Unreachable(_) => {} // expected
            other => panic!("expected Unreachable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn health_check_nonexistent_host() {
        let checker = HealthChecker::new(Duration::from_millis(500));
        let status = checker.check(19999, "/health").await;
        match status {
            HealthStatus::Unreachable(reason) => {
                assert!(!reason.is_empty());
            }
            other => panic!("expected Unreachable, got {other:?}"),
        }
    }
}
