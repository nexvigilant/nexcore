//! # Cloud Service Models (T3 Domain-Specific)
//!
//! 5 domain-specific types composing 6+ unique Lex Primitiva each.
//! These are cloud-native concepts that don't transfer cleanly to other domains.

use crate::primitives::*;
use serde::{Deserialize, Serialize};

/// Lightweight isolated compute with shared kernel.
///
/// Composes: Compute + IsolationBoundary + ResourcePool (6+ unique primitives via composition)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Container {
    /// Compute allocation
    pub compute: Compute,
    /// Container isolation
    pub boundary: IsolationBoundary,
    /// Resource limits
    pub pool: ResourcePool,
    /// Container image identifier
    pub image: String,
    /// Whether currently running
    pub running: bool,
}

impl Container {
    /// Create a new container.
    pub fn new(image: impl Into<String>, vcpus: f64, memory: f64) -> Self {
        Self {
            compute: Compute::new(vcpus, 1.0),
            boundary: IsolationBoundary::new("container-ns", 0.8),
            pool: ResourcePool::new(memory),
            image: image.into(),
            running: false,
        }
    }

    /// Start the container.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the container.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Whether the container is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Effective throughput when running.
    pub fn throughput(&self) -> f64 {
        if self.running {
            self.compute.throughput()
        } else {
            0.0
        }
    }
}

/// Infrastructure as a Service: raw compute, storage, networking.
///
/// Composes: Compute + Storage + NetworkLink + ResourcePool + Lease + IsolationBoundary + Metering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Iaas {
    /// Compute resource
    pub compute: Compute,
    /// Storage resource
    pub storage: Storage,
    /// Network connectivity
    pub network: NetworkLink,
    /// Resource pool
    pub pool: ResourcePool,
    /// Access lease
    pub lease: Lease,
    /// Isolation
    pub boundary: IsolationBoundary,
    /// Usage metering
    pub metering: Metering,
}

impl Iaas {
    /// Create a new IaaS allocation.
    pub fn new(
        vcpus: f64,
        storage_gb: f64,
        bandwidth: f64,
        holder: impl Into<String>,
        ttl: f64,
    ) -> Self {
        let holder_str: String = holder.into();
        Self {
            compute: Compute::new(vcpus, 1.0),
            storage: Storage::new(storage_gb),
            network: NetworkLink::new("iaas-internal", "internet", bandwidth),
            pool: ResourcePool::new(vcpus + storage_gb),
            lease: Lease::new("iaas", holder_str, ttl),
            boundary: IsolationBoundary::new("iaas-boundary", 0.95),
            metering: Metering::new("iaas-usage", 3600.0),
        }
    }

    /// Whether the IaaS allocation is active.
    pub fn is_active(&self) -> bool {
        !self.lease.is_expired()
    }

    /// Record resource usage.
    pub fn record_usage(&mut self, amount: f64) {
        self.metering.record(amount);
    }
}

/// Platform as a Service: IaaS + managed runtime abstraction.
///
/// Composes: Iaas components + additional IsolationBoundary (runtime abstraction)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paas {
    /// Underlying IaaS
    pub infrastructure: Iaas,
    /// Runtime abstraction boundary
    pub runtime_boundary: IsolationBoundary,
    /// Runtime identifier
    pub runtime: String,
}

impl Paas {
    /// Create a new PaaS allocation.
    pub fn new(
        runtime: impl Into<String>,
        vcpus: f64,
        storage_gb: f64,
        holder: impl Into<String>,
        ttl: f64,
    ) -> Self {
        Self {
            infrastructure: Iaas::new(vcpus, storage_gb, 100.0, holder, ttl),
            runtime_boundary: IsolationBoundary::new("runtime-sandbox", 0.9),
            runtime: runtime.into(),
        }
    }

    /// Whether the PaaS allocation is active.
    pub fn is_active(&self) -> bool {
        self.infrastructure.is_active()
    }
}

/// Software as a Service: PaaS + application-level isolation and metering.
///
/// Composes: Paas components + Metering + IsolationBoundary (tenant)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Saas {
    /// Underlying PaaS
    pub platform: Paas,
    /// Tenant isolation
    pub tenant_boundary: IsolationBoundary,
    /// Application-level metering
    pub app_metering: Metering,
    /// Application name
    pub application: String,
}

impl Saas {
    /// Create a new SaaS offering.
    pub fn new(
        application: impl Into<String>,
        runtime: impl Into<String>,
        holder: impl Into<String>,
        ttl: f64,
    ) -> Self {
        Self {
            platform: Paas::new(runtime, 2.0, 50.0, holder, ttl),
            tenant_boundary: IsolationBoundary::new("tenant-isolation", 0.95),
            app_metering: Metering::new("api-calls", 3600.0),
            application: application.into(),
        }
    }

    /// Record an API call.
    pub fn record_api_call(&mut self) {
        self.app_metering.record(1.0);
    }

    /// API call rate.
    pub fn api_rate(&self) -> f64 {
        self.app_metering.rate()
    }

    /// Whether the SaaS instance is active.
    pub fn is_active(&self) -> bool {
        self.platform.is_active()
    }
}

/// Event-driven compute with per-invocation billing and zero idle cost.
///
/// Composes: Compute + Lease + ResourcePool + Metering + Threshold
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Serverless {
    /// Compute allocation (per invocation)
    pub compute: Compute,
    /// Invocation lease (short-lived)
    pub lease: Lease,
    /// Concurrency pool
    pub pool: ResourcePool,
    /// Invocation metering
    pub metering: Metering,
    /// Concurrency threshold
    pub threshold: Threshold,
    /// Total invocations
    pub invocations: u64,
}

impl Serverless {
    /// Create a new serverless function config.
    pub fn new(max_concurrency: f64, timeout: f64) -> Self {
        Self {
            compute: Compute::new(1.0, 1.0),
            lease: Lease::new("invocation", "runtime", timeout),
            pool: ResourcePool::new(max_concurrency),
            metering: Metering::new("invocations", 1.0),
            threshold: Threshold::new(max_concurrency * 0.9, true),
            invocations: 0,
        }
    }

    /// Invoke the function.
    pub fn invoke(&mut self) -> Result<u64, &'static str> {
        self.pool.allocate(1.0)?;
        self.metering.record(1.0);
        self.invocations += 1;
        Ok(self.invocations)
    }

    /// Complete an invocation (release concurrency slot).
    pub fn complete(&mut self) {
        self.pool.release(1.0);
    }

    /// Whether nearing concurrency limit.
    pub fn is_throttled(&self) -> bool {
        self.threshold.exceeded(self.pool.allocated)
    }

    /// Total invocations.
    pub fn total_invocations(&self) -> u64 {
        self.invocations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_lifecycle() {
        let mut c = Container::new("nginx:latest", 2.0, 512.0);
        assert!(!c.is_running());
        assert!((c.throughput() - 0.0).abs() < f64::EPSILON);
        c.start();
        assert!(c.is_running());
        assert!((c.throughput() - 2.0).abs() < f64::EPSILON);
        c.stop();
        assert!(!c.is_running());
    }

    #[test]
    fn test_iaas_lifecycle() {
        let mut iaas = Iaas::new(4.0, 100.0, 1000.0, "enterprise", 365.0);
        assert!(iaas.is_active());
        iaas.record_usage(50.0);
        assert!((iaas.metering.consumed - 50.0).abs() < f64::EPSILON);
        iaas.lease.tick(366.0);
        assert!(!iaas.is_active());
    }

    #[test]
    fn test_paas() {
        let paas = Paas::new("python-3.12", 2.0, 50.0, "startup", 365.0);
        assert!(paas.is_active());
        assert_eq!(paas.runtime, "python-3.12");
    }

    #[test]
    fn test_saas_api_metering() {
        let mut saas = Saas::new("crm", "node-20", "customer", 365.0);
        assert!(saas.is_active());
        saas.record_api_call();
        saas.record_api_call();
        saas.record_api_call();
        assert!((saas.api_rate() - 3.0 / 3600.0).abs() < 0.001);
    }

    #[test]
    fn test_serverless_invocations() {
        let mut sf = Serverless::new(10.0, 30.0);
        assert!(sf.invoke().is_ok());
        assert!(sf.invoke().is_ok());
        assert_eq!(sf.total_invocations(), 2);
        sf.complete();
        assert!((sf.pool.available() - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_serverless_concurrency_limit() {
        let mut sf = Serverless::new(2.0, 30.0);
        assert!(sf.invoke().is_ok());
        assert!(sf.invoke().is_ok());
        assert!(sf.invoke().is_err()); // pool exhausted
    }

    #[test]
    fn test_serverless_throttle_detection() {
        let mut sf = Serverless::new(10.0, 30.0);
        for _ in 0..9 {
            let _ = sf.invoke();
        }
        assert!(sf.is_throttled()); // 9 >= 10 * 0.9
    }

    // Serde round-trips

    #[test]
    fn test_serde_service_models() {
        let c = Container::new("img", 1.0, 256.0);
        let json = serde_json::to_string(&c).unwrap_or_default();
        let c2: Container =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("Container deser failed: {e}"));
        assert_eq!(c, c2);

        let iaas = Iaas::new(1.0, 10.0, 100.0, "u", 30.0);
        let json = serde_json::to_string(&iaas).unwrap_or_default();
        let iaas2: Iaas =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("Iaas deser failed: {e}"));
        assert_eq!(iaas, iaas2);

        let paas = Paas::new("py", 1.0, 10.0, "u", 30.0);
        let json = serde_json::to_string(&paas).unwrap_or_default();
        let paas2: Paas =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("Paas deser failed: {e}"));
        assert_eq!(paas, paas2);

        let saas = Saas::new("app", "rt", "u", 30.0);
        let json = serde_json::to_string(&saas).unwrap_or_default();
        let saas2: Saas =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("Saas deser failed: {e}"));
        assert_eq!(saas, saas2);

        let sf = Serverless::new(5.0, 30.0);
        let json = serde_json::to_string(&sf).unwrap_or_default();
        let sf2: Serverless =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("Serverless deser failed: {e}"));
        assert_eq!(sf, sf2);
    }
}
