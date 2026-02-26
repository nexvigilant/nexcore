//! # Cloud Computing Composites (T2-C)
//!
//! 10 composite types built from T1/T2-P primitives.
//! Each composes 4-5 unique Lex Primitiva, classifying as T2-C.

use crate::primitives::*;
use serde::{Deserialize, Serialize};

/// Isolated compute environment with dedicated resources and time-bounded access.
///
/// Composes: Compute + IsolationBoundary + ResourcePool + Lease
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualMachine {
    /// Compute allocation
    pub compute: Compute,
    /// Isolation boundary
    pub boundary: IsolationBoundary,
    /// Resource pool backing
    pub pool: ResourcePool,
    /// Access lease
    pub lease: Lease,
}

impl VirtualMachine {
    /// Create a new VM.
    pub fn new(
        vcpus: f64,
        frequency: f64,
        memory: f64,
        holder: impl Into<String>,
        ttl: f64,
    ) -> Self {
        Self {
            compute: Compute::new(vcpus, frequency),
            boundary: IsolationBoundary::new("vm-boundary", 0.95),
            pool: ResourcePool::new(memory),
            lease: Lease::new("vm", holder, ttl),
        }
    }

    /// Whether the VM is still active (lease not expired).
    pub fn is_active(&self) -> bool {
        !self.lease.is_expired()
    }

    /// Effective throughput.
    pub fn throughput(&self) -> f64 {
        self.compute.throughput()
    }
}

/// Request distributor across healthy backends with metering.
///
/// Composes: Routing + HealthCheck + Metering + Threshold
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadBalancer {
    /// Routing logic
    pub routing: Routing,
    /// Health checking
    pub health: HealthCheck,
    /// Traffic metering
    pub metering: Metering,
    /// Overload threshold
    pub threshold: Threshold,
}

impl LoadBalancer {
    /// Create a new load balancer.
    pub fn new(backends: usize, failure_threshold: u64, max_rps: f64) -> Self {
        Self {
            routing: Routing::new(backends),
            health: HealthCheck::new("backends", failure_threshold),
            metering: Metering::new("requests", 1.0),
            threshold: Threshold::new(max_rps, true),
        }
    }

    /// Route a request and record it.
    pub fn route_request(&mut self) -> usize {
        self.metering.record(1.0);
        self.routing.route()
    }

    /// Whether the load balancer is overloaded.
    pub fn is_overloaded(&self) -> bool {
        self.threshold.exceeded(self.metering.rate())
    }

    /// Whether backends are healthy.
    pub fn is_healthy(&self) -> bool {
        self.health.is_healthy()
    }
}

/// Feedback-driven scaling of resources based on metered demand.
///
/// Composes: Metering + Threshold + ResourcePool + FeedbackLoop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoScaling {
    /// Demand metering
    pub metering: Metering,
    /// Scale-up threshold
    pub threshold: Threshold,
    /// Resource pool
    pub pool: ResourcePool,
    /// Feedback loop for corrections
    pub feedback: FeedbackLoop,
}

impl AutoScaling {
    /// Create a new auto-scaling policy.
    pub fn new(pool_size: f64, target_utilization: f64, threshold: f64) -> Self {
        Self {
            metering: Metering::new("demand", 1.0),
            threshold: Threshold::new(threshold, true),
            pool: ResourcePool::new(pool_size),
            feedback: FeedbackLoop::new(target_utilization, 0.3),
        }
    }

    /// Record demand and check if scaling is needed.
    pub fn record_demand(&mut self, demand: f64) {
        self.metering.record(demand);
        self.feedback.current = self.pool.utilization();
    }

    /// Whether scale-up is needed.
    pub fn needs_scale_up(&self) -> bool {
        self.threshold.exceeded(self.metering.rate())
    }

    /// Get correction signal.
    pub fn correction(&self) -> f64 {
        self.feedback.correction()
    }
}

/// Identity + permission management within isolation boundaries.
///
/// Composes: Identity + Permission + IsolationBoundary + Lease
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Iam {
    /// The identity
    pub identity: Identity,
    /// Permissions granted
    pub permissions: Vec<Permission>,
    /// Security boundary
    pub boundary: IsolationBoundary,
    /// Session lease
    pub session: Lease,
}

impl Iam {
    /// Create a new IAM entry.
    pub fn new(id: impl Into<String>, session_ttl: f64) -> Self {
        let id_str: String = id.into();
        Self {
            identity: Identity::new(id_str.clone()),
            permissions: Vec::new(),
            boundary: IsolationBoundary::new("iam-boundary", 1.0),
            session: Lease::new("session", id_str, session_ttl),
        }
    }

    /// Grant a permission.
    pub fn grant(&mut self, resource: impl Into<String>, action: impl Into<String>) {
        self.permissions
            .push(Permission::new(self.identity.id.clone(), resource, action));
    }

    /// Check if an action is authorized.
    pub fn is_authorized(&self, resource: &str, action: &str) -> bool {
        if self.session.is_expired() {
            return false;
        }
        self.permissions
            .iter()
            .any(|p| p.matches(&self.identity.id, resource, action))
    }
}

/// Guarantees that replicated state will eventually agree.
///
/// Composes: Replication + Convergence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventualConsistency {
    /// Replication state
    pub replication: Replication,
    /// Convergence tracker
    pub convergence: Convergence,
}

impl EventualConsistency {
    /// Create a new eventual consistency tracker.
    pub fn new(replicas: usize, quorum: f64) -> Self {
        Self {
            replication: Replication::new(replicas),
            convergence: Convergence::new(replicas, quorum),
        }
    }

    /// Record a replica acknowledging consistency.
    pub fn acknowledge(&mut self) {
        self.replication.confirm();
        self.convergence.agree();
    }

    /// Whether the system has reached eventual consistency.
    pub fn is_consistent(&self) -> bool {
        self.convergence.has_converged()
    }
}

/// Isolated resource allocation per tenant identity.
///
/// Composes: Identity + ResourcePool + IsolationBoundary + Lease
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tenancy {
    /// Tenant identity
    pub tenant: Identity,
    /// Resource allocation
    pub pool: ResourcePool,
    /// Isolation
    pub boundary: IsolationBoundary,
    /// Tenancy period
    pub lease: Lease,
}

impl Tenancy {
    /// Create a new tenancy.
    pub fn new(tenant_id: impl Into<String>, resources: f64, ttl: f64) -> Self {
        let id: String = tenant_id.into();
        Self {
            tenant: Identity::new(id.clone()),
            pool: ResourcePool::new(resources),
            boundary: IsolationBoundary::new("tenant-boundary", 0.9),
            lease: Lease::new("tenancy", id, ttl),
        }
    }

    /// Whether tenancy is active.
    pub fn is_active(&self) -> bool {
        !self.lease.is_expired()
    }

    /// Resource utilization.
    pub fn utilization(&self) -> f64 {
        self.pool.utilization()
    }
}

/// Billing model where cost = metered consumption * unit price.
///
/// Composes: Metering (dominant)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayPerUse {
    /// Consumption metering
    pub metering: Metering,
    /// Unit price
    pub unit_price: f64,
}

impl PayPerUse {
    /// Create a new pay-per-use model.
    pub fn new(resource: impl Into<String>, unit_price: f64) -> Self {
        Self {
            metering: Metering::new(resource, 1.0),
            unit_price: unit_price.max(0.0),
        }
    }

    /// Record usage.
    pub fn record(&mut self, amount: f64) {
        self.metering.record(amount);
    }

    /// Total cost so far.
    pub fn total_cost(&self) -> f64 {
        self.metering.consumed * self.unit_price
    }
}

/// Pre-allocated capacity with guaranteed availability.
///
/// Composes: Lease + ResourcePool + Threshold
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReservedCapacity {
    /// Reservation lease
    pub lease: Lease,
    /// Reserved resources
    pub pool: ResourcePool,
    /// Minimum utilization threshold (use-it-or-lose-it)
    pub threshold: Threshold,
}

impl ReservedCapacity {
    /// Create a new reservation.
    pub fn new(holder: impl Into<String>, capacity: f64, ttl: f64, min_utilization: f64) -> Self {
        Self {
            lease: Lease::new("reservation", holder, ttl),
            pool: ResourcePool::new(capacity),
            threshold: Threshold::new(min_utilization, false),
        }
    }

    /// Whether the reservation is active.
    pub fn is_active(&self) -> bool {
        !self.lease.is_expired()
    }

    /// Whether utilization meets minimum threshold.
    pub fn is_utilized(&self) -> bool {
        !self.threshold.exceeded(self.pool.utilization())
    }
}

/// Market-driven pricing with preemptible resources.
///
/// Composes: Metering + ResourcePool + Threshold + Lease + FeedbackLoop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotPricing {
    /// Demand metering
    pub metering: Metering,
    /// Available pool
    pub pool: ResourcePool,
    /// Eviction threshold
    pub threshold: Threshold,
    /// Allocation lease
    pub lease: Lease,
    /// Price feedback loop
    pub feedback: FeedbackLoop,
}

impl SpotPricing {
    /// Create a new spot pricing model.
    pub fn new(pool_size: f64, max_price: f64, holder: impl Into<String>, ttl: f64) -> Self {
        Self {
            metering: Metering::new("demand", 1.0),
            pool: ResourcePool::new(pool_size),
            threshold: Threshold::new(max_price, true),
            lease: Lease::new("spot", holder, ttl),
            feedback: FeedbackLoop::new(max_price * 0.5, 0.1),
        }
    }

    /// Current spot price (driven by demand).
    pub fn current_price(&self) -> f64 {
        self.feedback.current.max(0.0)
    }

    /// Whether the spot instance should be evicted (price > threshold).
    pub fn should_evict(&self) -> bool {
        self.threshold.exceeded(self.current_price()) || self.lease.is_expired()
    }
}

/// Encrypted storage with permissioned access and audit metering.
///
/// Composes: Encryption + Storage + Permission + Lease + Metering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretsManagement {
    /// Encryption layer
    pub encryption: Encryption,
    /// Secret storage
    pub storage: Storage,
    /// Access permission
    pub access: Permission,
    /// Access lease
    pub lease: Lease,
    /// Access metering
    pub metering: Metering,
}

impl SecretsManagement {
    /// Create a new secrets manager.
    pub fn new(owner: impl Into<String>, capacity: f64, ttl: f64) -> Self {
        let owner_str: String = owner.into();
        Self {
            encryption: Encryption::new("AES-256-GCM", 256),
            storage: Storage::new(capacity),
            access: Permission::new(owner_str.clone(), "secrets", "read"),
            lease: Lease::new("secrets-access", owner_str, ttl),
            metering: Metering::new("secret-reads", 3600.0),
        }
    }

    /// Attempt to read a secret (checks permission and lease).
    pub fn access(&mut self, requester: &str) -> Result<(), &'static str> {
        if self.lease.is_expired() {
            return Err("lease expired");
        }
        if !self.access.matches(requester, "secrets", "read") {
            return Err("unauthorized");
        }
        self.metering.record(1.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_machine() {
        let mut vm = VirtualMachine::new(4.0, 3.0, 16.0, "user-1", 100.0);
        assert!(vm.is_active());
        assert!((vm.throughput() - 12.0).abs() < f64::EPSILON);
        vm.lease.tick(101.0);
        assert!(!vm.is_active());
    }

    #[test]
    fn test_load_balancer() {
        let mut lb = LoadBalancer::new(3, 3, 1000.0);
        assert!(lb.is_healthy());
        assert_eq!(lb.route_request(), 0);
        assert_eq!(lb.route_request(), 1);
        assert_eq!(lb.route_request(), 2);
        assert_eq!(lb.route_request(), 0);
    }

    #[test]
    fn test_auto_scaling() {
        let mut as_ = AutoScaling::new(100.0, 0.7, 50.0);
        as_.record_demand(60.0);
        assert!(as_.needs_scale_up());
    }

    #[test]
    fn test_iam_authorization() {
        let mut iam = Iam::new("alice", 3600.0);
        iam.grant("database", "read");
        assert!(iam.is_authorized("database", "read"));
        assert!(!iam.is_authorized("database", "write"));
    }

    #[test]
    fn test_iam_expired_session() {
        let mut iam = Iam::new("bob", 10.0);
        iam.grant("api", "call");
        iam.session.tick(11.0);
        assert!(!iam.is_authorized("api", "call"));
    }

    #[test]
    fn test_eventual_consistency() {
        let mut ec = EventualConsistency::new(3, 0.66);
        assert!(!ec.is_consistent());
        ec.acknowledge();
        ec.acknowledge();
        assert!(ec.is_consistent()); // 2/3 = 0.666... >= 0.66
    }

    #[test]
    fn test_tenancy() {
        let t = Tenancy::new("tenant-1", 100.0, 365.0);
        assert!(t.is_active());
        assert!((t.utilization() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pay_per_use() {
        let mut ppu = PayPerUse::new("compute-hours", 0.10);
        ppu.record(100.0);
        assert!((ppu.total_cost() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reserved_capacity() {
        let rc = ReservedCapacity::new("enterprise", 50.0, 365.0, 0.2);
        assert!(rc.is_active());
    }

    #[test]
    fn test_spot_pricing() {
        let sp = SpotPricing::new(100.0, 1.0, "user-1", 3600.0);
        assert!(!sp.should_evict());
    }

    #[test]
    fn test_secrets_management() {
        let mut sm = SecretsManagement::new("admin", 100.0, 3600.0);
        assert!(sm.access("admin").is_ok());
        assert!(sm.access("hacker").is_err());
    }

    #[test]
    fn test_secrets_management_expired() {
        let mut sm = SecretsManagement::new("admin", 100.0, 10.0);
        sm.lease.tick(11.0);
        assert!(sm.access("admin").is_err());
    }

    // Serde round-trips for composites

    #[test]
    fn test_serde_composites() {
        let vm = VirtualMachine::new(2.0, 2.0, 8.0, "u", 60.0);
        let json = serde_json::to_string(&vm).unwrap_or_default();
        let vm2: VirtualMachine = serde_json::from_str(&json).unwrap_or_else(|e| {
            panic!("VirtualMachine deserialization failed: {e}");
        });
        assert_eq!(vm, vm2);

        let lb = LoadBalancer::new(2, 2, 100.0);
        let json = serde_json::to_string(&lb).unwrap_or_default();
        let lb2: LoadBalancer = serde_json::from_str(&json).unwrap_or_else(|e| {
            panic!("LoadBalancer deserialization failed: {e}");
        });
        assert_eq!(lb, lb2);

        let ec = EventualConsistency::new(3, 0.5);
        let json = serde_json::to_string(&ec).unwrap_or_default();
        let ec2: EventualConsistency = serde_json::from_str(&json).unwrap_or_else(|e| {
            panic!("EventualConsistency deserialization failed: {e}");
        });
        assert_eq!(ec, ec2);

        let iam = Iam::new("test", 60.0);
        let json = serde_json::to_string(&iam).unwrap_or_default();
        let iam2: Iam = serde_json::from_str(&json).unwrap_or_else(|e| {
            panic!("Iam deserialization failed: {e}");
        });
        assert_eq!(iam, iam2);

        let sm = SecretsManagement::new("o", 10.0, 60.0);
        let json = serde_json::to_string(&sm).unwrap_or_default();
        let sm2: SecretsManagement = serde_json::from_str(&json).unwrap_or_else(|e| {
            panic!("SecretsManagement deserialization failed: {e}");
        });
        assert_eq!(sm, sm2);
    }
}
