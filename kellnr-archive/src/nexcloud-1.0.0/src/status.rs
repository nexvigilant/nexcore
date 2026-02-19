//! Cloud status endpoint response types.
//!
//! Bridges nexcore-cloud primitives (HealthCheck, ResourcePool, Metering)
//! into the NexCloud runtime for the `/.nexcloud/status` HTTP endpoint.
//!
//! ## Tier Classification
//!
//! - `CloudStatus`: T3 (ς State + μ Mapping + σ Sequence + N Quantity + ∃ Existence)
//! - `ServiceStatus`: T2-C (ς State + N Quantity + ∃ Existence)
//! - `OverallHealth`: T2-P (ς State)
//! - `ResourceSnapshot`: T2-C (N Quantity + Σ Sum)

use crate::supervisor::registry::{ProcessState, ServiceRecord};
use nexcore_cloud::{HealthCheck, Metering, ResourcePool};
use serde::Serialize;

/// Overall platform health derived from service states.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OverallHealth {
    /// All services healthy.
    Healthy,
    /// Some services unhealthy but not critical.
    Degraded,
    /// Majority of services unhealthy or failed.
    Critical,
}

impl std::fmt::Display for OverallHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Status of a single managed service.
///
/// Tier: T2-C (ς State + N Quantity + ∃ Existence)
#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    /// Service name.
    pub name: String,
    /// Current lifecycle state.
    pub state: String,
    /// Process ID (if running).
    pub pid: Option<u32>,
    /// Listening port.
    pub port: u16,
    /// Number of restarts.
    pub restarts: u32,
    /// When the service started (ISO 8601).
    pub started_at: Option<String>,
    /// Last successful health check (ISO 8601).
    pub last_health: Option<String>,
    /// Health check primitive from nexcore-cloud.
    pub health_check: HealthCheck,
}

/// Resource snapshot for the platform.
///
/// Tier: T2-C (N Quantity + Sigma Sum)
#[derive(Debug, Clone, Serialize)]
pub struct ResourceSnapshot {
    /// Process resource pool (total vs allocated).
    pub process_pool: ResourcePool,
    /// Request metering.
    pub request_meter: Metering,
}

/// Full cloud status response for `/.nexcloud/status`.
///
/// Tier: T3 (ς State + mu Mapping + sigma Sequence + N Quantity + ∃ Existence)
#[derive(Debug, Clone, Serialize)]
pub struct CloudStatus {
    /// Platform name from manifest.
    pub platform_name: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Per-service status.
    pub services: Vec<ServiceStatus>,
    /// Derived overall health.
    pub overall_health: OverallHealth,
    /// Resource snapshot.
    pub resource_snapshot: ResourceSnapshot,
}

impl CloudStatus {
    /// Build a `CloudStatus` from the platform name and a registry snapshot.
    ///
    /// Derives overall health from service states:
    /// - All healthy/pending/starting -> Healthy
    /// - Any unhealthy/restarting but < half failed -> Degraded
    /// - Half or more failed -> Critical
    #[must_use]
    pub fn from_records(platform_name: &str, records: Vec<ServiceRecord>) -> Self {
        let service_count = records.len();
        let services: Vec<ServiceStatus> = records
            .iter()
            .map(|r| {
                let health_check = build_health_check(r);
                ServiceStatus {
                    name: r.name.clone(),
                    state: r.state.to_string(),
                    pid: r.pid,
                    port: r.port,
                    restarts: r.restarts,
                    started_at: r.started_at.map(|t| t.to_rfc3339()),
                    last_health: r.last_health.map(|t| t.to_rfc3339()),
                    health_check,
                }
            })
            .collect();

        let overall_health = derive_health(&records);

        // Build resource snapshot: pool = service_count capacity, allocated = running count
        let running = records.iter().filter(|r| r.pid.is_some()).count();
        let process_pool = ResourcePool {
            total: service_count as f64,
            allocated: running as f64,
        };
        let request_meter = Metering::new("requests", 60.0);

        Self {
            platform_name: platform_name.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            services,
            overall_health,
            resource_snapshot: ResourceSnapshot {
                process_pool,
                request_meter,
            },
        }
    }
}

/// Build a `HealthCheck` from a service record's state.
///
/// Uses failure_threshold=1 so a single unhealthy state flips the check.
/// This reflects the supervisor's own assessment — if it says unhealthy, we trust it.
fn build_health_check(record: &ServiceRecord) -> HealthCheck {
    let mut hc = HealthCheck::new(&record.name, 1);
    match record.state {
        ProcessState::Healthy => {
            hc.record_success();
        }
        ProcessState::Unhealthy | ProcessState::Failed => {
            hc.record_failure();
        }
        _ => {}
    }
    hc
}

/// Derive overall health from a collection of service records.
fn derive_health(records: &[ServiceRecord]) -> OverallHealth {
    if records.is_empty() {
        return OverallHealth::Healthy;
    }

    let total = records.len();
    let failed_count = records
        .iter()
        .filter(|r| matches!(r.state, ProcessState::Failed))
        .count();
    let unhealthy_count = records
        .iter()
        .filter(|r| {
            matches!(
                r.state,
                ProcessState::Unhealthy | ProcessState::Restarting | ProcessState::Failed
            )
        })
        .count();

    if unhealthy_count == 0 {
        OverallHealth::Healthy
    } else if failed_count * 2 >= total {
        OverallHealth::Critical
    } else {
        OverallHealth::Degraded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervisor::registry::ProcessState;
    use chrono::Utc;

    fn make_record(name: &str, state: ProcessState, pid: Option<u32>) -> ServiceRecord {
        ServiceRecord {
            name: name.to_string(),
            state,
            pid,
            port: 8080,
            restarts: 0,
            started_at: Some(Utc::now()),
            last_health: if state == ProcessState::Healthy {
                Some(Utc::now())
            } else {
                None
            },
        }
    }

    #[test]
    fn all_healthy() {
        let records = vec![
            make_record("a", ProcessState::Healthy, Some(100)),
            make_record("b", ProcessState::Healthy, Some(101)),
        ];
        let status = CloudStatus::from_records("test", records);
        assert_eq!(status.overall_health, OverallHealth::Healthy);
        assert_eq!(status.services.len(), 2);
        assert_eq!(status.platform_name, "test");
    }

    #[test]
    fn degraded_with_unhealthy() {
        let records = vec![
            make_record("a", ProcessState::Healthy, Some(100)),
            make_record("b", ProcessState::Unhealthy, Some(101)),
            make_record("c", ProcessState::Healthy, Some(102)),
        ];
        let status = CloudStatus::from_records("test", records);
        assert_eq!(status.overall_health, OverallHealth::Degraded);
    }

    #[test]
    fn critical_when_half_failed() {
        let records = vec![
            make_record("a", ProcessState::Failed, None),
            make_record("b", ProcessState::Failed, None),
            make_record("c", ProcessState::Healthy, Some(102)),
        ];
        let status = CloudStatus::from_records("test", records);
        assert_eq!(status.overall_health, OverallHealth::Critical);
    }

    #[test]
    fn empty_is_healthy() {
        let status = CloudStatus::from_records("test", vec![]);
        assert_eq!(status.overall_health, OverallHealth::Healthy);
        assert!(status.services.is_empty());
    }

    #[test]
    fn resource_snapshot_reflects_running() {
        let records = vec![
            make_record("a", ProcessState::Healthy, Some(100)),
            make_record("b", ProcessState::Stopped, None),
        ];
        let status = CloudStatus::from_records("test", records);
        assert!((status.resource_snapshot.process_pool.total - 2.0).abs() < f64::EPSILON);
        assert!((status.resource_snapshot.process_pool.allocated - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn health_check_set_on_healthy_service() {
        let records = vec![make_record("a", ProcessState::Healthy, Some(100))];
        let status = CloudStatus::from_records("test", records);
        assert!(status.services[0].health_check.is_healthy());
    }

    #[test]
    fn health_check_set_on_unhealthy_service() {
        let records = vec![make_record("a", ProcessState::Unhealthy, Some(100))];
        let status = CloudStatus::from_records("test", records);
        assert!(!status.services[0].health_check.is_healthy());
    }

    #[test]
    fn overall_health_display() {
        assert_eq!(format!("{}", OverallHealth::Healthy), "healthy");
        assert_eq!(format!("{}", OverallHealth::Degraded), "degraded");
        assert_eq!(format!("{}", OverallHealth::Critical), "critical");
    }
}
