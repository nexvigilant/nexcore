//! # Cloud Computing Primitives
//!
//! 20 irreducible types: 6 T1 Universal + 14 T2-P Cross-Domain.
//!
//! Each type grounds to 1-3 unique Lex Primitiva symbols with documented
//! cross-domain transfers (PV, biology, economics, physics).

use serde::{Deserialize, Serialize};

// ============================================================================
// T1 Universal Primitives (6)
// ============================================================================

/// Unique identifier establishing existence of an entity.
///
/// Transfers: biology (genome), economics (account), PV (case ID).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    /// Unique identifier value
    pub id: String,
    /// Optional human-readable label
    pub label: Option<String>,
}

impl Identity {
    /// Create a new identity.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: None,
        }
    }

    /// Create with a label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A boundary value that triggers state transitions.
///
/// Transfers: biology (action potential), economics (price floor/ceiling), PV (signal threshold).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Threshold {
    /// The boundary value
    pub value: f64,
    /// Direction: true = trigger when above, false = trigger when below
    pub upper: bool,
}

impl Threshold {
    /// Create a new threshold.
    pub fn new(value: f64, upper: bool) -> Self {
        Self { value, upper }
    }

    /// Check if a measurement exceeds this threshold.
    pub fn exceeded(&self, measurement: f64) -> bool {
        if self.upper {
            measurement >= self.value
        } else {
            measurement <= self.value
        }
    }

    /// Distance from measurement to threshold (positive = safe side).
    pub fn margin(&self, measurement: f64) -> f64 {
        if self.upper {
            self.value - measurement
        } else {
            measurement - self.value
        }
    }
}

/// Self-correcting cycle adjusting output based on measured error.
///
/// Transfers: biology (homeostasis), control theory (PID), PV (signal-action loop).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackLoop {
    /// Target setpoint
    pub setpoint: f64,
    /// Current measured value
    pub current: f64,
    /// Gain factor for correction
    pub gain: f64,
}

impl FeedbackLoop {
    /// Create a new feedback loop.
    pub fn new(setpoint: f64, gain: f64) -> Self {
        Self {
            setpoint,
            current: 0.0,
            gain: gain.max(0.0),
        }
    }

    /// Error: difference between setpoint and current.
    pub fn error(&self) -> f64 {
        self.setpoint - self.current
    }

    /// Correction signal: error * gain.
    pub fn correction(&self) -> f64 {
        self.error() * self.gain
    }

    /// Apply one tick of the feedback loop.
    pub fn tick(&mut self) {
        self.current += self.correction();
    }

    /// Whether the system has converged (error < tolerance).
    pub fn converged(&self, tolerance: f64) -> bool {
        self.error().abs() < tolerance
    }
}

/// Guarantee that applying an operation N times equals applying it once.
///
/// Transfers: mathematics (projection), HTTP (PUT), PV (duplicate report dedup).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Idempotency {
    /// Unique key identifying this operation
    pub key: String,
    /// How many times apply has been called
    pub application_count: u64,
    /// Whether the operation has been applied at least once
    pub applied: bool,
}

impl Idempotency {
    /// Create a new idempotency guard.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            application_count: 0,
            applied: false,
        }
    }

    /// Apply the operation. Returns true only on first application.
    pub fn apply(&mut self) -> bool {
        self.application_count += 1;
        if !self.applied {
            self.applied = true;
            return true;
        }
        false
    }

    /// Whether this operation has been applied.
    pub fn is_applied(&self) -> bool {
        self.applied
    }

    /// Total attempts (including duplicates).
    pub fn attempts(&self) -> u64 {
        self.application_count
    }
}

/// State that cannot be reversed once committed.
///
/// Transfers: thermodynamics (entropy), economics (sunk cost), PV (regulatory submission).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Immutability {
    /// Label for the immutable record
    pub label: String,
    /// Whether the record has been sealed
    pub sealed: bool,
    /// Content hash (empty until sealed)
    pub content_hash: String,
}

impl Immutability {
    /// Create an unsealed record.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            sealed: false,
            content_hash: String::new(),
        }
    }

    /// Seal with a content hash, making it immutable.
    pub fn seal(&mut self, hash: impl Into<String>) -> Result<(), &'static str> {
        if self.sealed {
            return Err("already sealed");
        }
        self.content_hash = hash.into();
        self.sealed = true;
        Ok(())
    }

    /// Whether the record is sealed.
    pub fn is_sealed(&self) -> bool {
        self.sealed
    }
}

/// Process by which distributed state reaches agreement.
///
/// Transfers: physics (equilibrium), biology (tissue homeostasis), PV (consensus causality).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Convergence {
    /// Number of participants
    pub participants: usize,
    /// Number that have agreed
    pub agreed: usize,
    /// Required fraction for convergence (0.0-1.0)
    pub quorum: f64,
}

impl Convergence {
    /// Create a new convergence tracker.
    pub fn new(participants: usize, quorum: f64) -> Self {
        Self {
            participants: participants.max(1),
            agreed: 0,
            quorum: quorum.clamp(0.0, 1.0),
        }
    }

    /// Record an agreement.
    pub fn agree(&mut self) {
        if self.agreed < self.participants {
            self.agreed += 1;
        }
    }

    /// Current agreement fraction.
    pub fn agreement_ratio(&self) -> f64 {
        self.agreed as f64 / self.participants as f64
    }

    /// Whether quorum has been reached.
    pub fn has_converged(&self) -> bool {
        self.agreement_ratio() >= self.quorum
    }

    /// How many more agreements are needed.
    pub fn remaining(&self) -> usize {
        let needed = (self.quorum * self.participants as f64).ceil() as usize;
        needed.saturating_sub(self.agreed)
    }
}

// ============================================================================
// T2-P Cross-Domain Primitives (14)
// ============================================================================

/// Capacity to execute instructions on a processing unit.
///
/// Transfers: physics (work), biology (metabolism), economics (labor).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Compute {
    /// Available processing capacity (abstract units)
    pub capacity: f64,
    /// Clock frequency / execution rate
    pub frequency: f64,
}

impl Compute {
    /// Create a new compute resource.
    pub fn new(capacity: f64, frequency: f64) -> Self {
        Self {
            capacity: capacity.max(0.0),
            frequency: frequency.max(0.0),
        }
    }

    /// Throughput: capacity * frequency.
    pub fn throughput(&self) -> f64 {
        self.capacity * self.frequency
    }
}

/// Durable medium for data persistence.
///
/// Transfers: biology (DNA), economics (inventory), PV (case database).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Storage {
    /// Total capacity in abstract units
    pub capacity: f64,
    /// Currently used
    pub used: f64,
}

impl Storage {
    /// Create a new storage resource.
    pub fn new(capacity: f64) -> Self {
        Self {
            capacity: capacity.max(0.0),
            used: 0.0,
        }
    }

    /// Available free space.
    pub fn available(&self) -> f64 {
        (self.capacity - self.used).max(0.0)
    }

    /// Utilization fraction (0.0-1.0).
    pub fn utilization(&self) -> f64 {
        if self.capacity <= 0.0 {
            return 0.0;
        }
        (self.used / self.capacity).clamp(0.0, 1.0)
    }

    /// Allocate space. Returns Err if insufficient.
    pub fn allocate(&mut self, amount: f64) -> Result<f64, &'static str> {
        let amount = amount.max(0.0);
        if self.used + amount > self.capacity {
            return Err("insufficient storage");
        }
        self.used += amount;
        Ok(self.available())
    }
}

/// Directed communication path between endpoints.
///
/// Transfers: biology (axon), economics (trade route), PV (reporting pathway).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkLink {
    /// Source endpoint identifier
    pub source: String,
    /// Destination endpoint identifier
    pub destination: String,
    /// Bandwidth capacity
    pub bandwidth: f64,
    /// Current latency
    pub latency: f64,
}

impl NetworkLink {
    /// Create a new network link.
    pub fn new(source: impl Into<String>, destination: impl Into<String>, bandwidth: f64) -> Self {
        Self {
            source: source.into(),
            destination: destination.into(),
            bandwidth: bandwidth.max(0.0),
            latency: 0.0,
        }
    }

    /// Set latency.
    pub fn with_latency(mut self, latency: f64) -> Self {
        self.latency = latency.max(0.0);
        self
    }

    /// Bandwidth-delay product.
    pub fn bdp(&self) -> f64 {
        self.bandwidth * self.latency
    }
}

/// Boundary separating contexts with different trust/resource domains.
///
/// Transfers: biology (cell membrane), security (DMZ), PV (regulatory jurisdiction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsolationBoundary {
    /// Boundary name
    pub name: String,
    /// Isolation strength (0.0 = porous, 1.0 = hermetic)
    pub strength: f64,
}

impl IsolationBoundary {
    /// Create a new isolation boundary.
    pub fn new(name: impl Into<String>, strength: f64) -> Self {
        Self {
            name: name.into(),
            strength: strength.clamp(0.0, 1.0),
        }
    }

    /// Whether this boundary is strong enough to meet a requirement.
    pub fn meets_requirement(&self, minimum: f64) -> bool {
        self.strength >= minimum
    }
}

/// Mapping from identity to allowed actions within a boundary.
///
/// Transfers: biology (enzyme specificity), economics (license), PV (role-based access).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    /// Subject (who)
    pub subject: String,
    /// Resource (what)
    pub resource: String,
    /// Action allowed
    pub action: String,
    /// Whether currently granted
    pub granted: bool,
}

impl Permission {
    /// Create a new permission.
    pub fn new(
        subject: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            resource: resource.into(),
            action: action.into(),
            granted: true,
        }
    }

    /// Revoke this permission.
    pub fn revoke(&mut self) {
        self.granted = false;
    }

    /// Check if this permission matches a request.
    pub fn matches(&self, subject: &str, resource: &str, action: &str) -> bool {
        self.granted
            && self.subject == subject
            && self.resource == resource
            && self.action == action
    }
}

/// Finite collection of fungible resources available for allocation.
///
/// Transfers: biology (blood pool), economics (capital pool), PV (reviewer pool).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Total resources in the pool
    pub total: f64,
    /// Currently allocated
    pub allocated: f64,
}

impl ResourcePool {
    /// Create a new resource pool.
    pub fn new(total: f64) -> Self {
        Self {
            total: total.max(0.0),
            allocated: 0.0,
        }
    }

    /// Available resources.
    pub fn available(&self) -> f64 {
        (self.total - self.allocated).max(0.0)
    }

    /// Allocate from the pool. Returns Err if insufficient.
    pub fn allocate(&mut self, amount: f64) -> Result<f64, &'static str> {
        let amount = amount.max(0.0);
        if self.allocated + amount > self.total {
            return Err("pool exhausted");
        }
        self.allocated += amount;
        Ok(self.available())
    }

    /// Release back to the pool.
    pub fn release(&mut self, amount: f64) {
        let amount = amount.max(0.0);
        self.allocated = (self.allocated - amount).max(0.0);
    }

    /// Utilization fraction.
    pub fn utilization(&self) -> f64 {
        if self.total <= 0.0 {
            return 0.0;
        }
        (self.allocated / self.total).clamp(0.0, 1.0)
    }
}

/// Measurement of resource consumption over time.
///
/// Transfers: physics (energy meter), economics (billing), PV (case processing rate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metering {
    /// What is being metered
    pub resource: String,
    /// Accumulated consumption
    pub consumed: f64,
    /// Time window in abstract units
    pub window: f64,
}

impl Metering {
    /// Create a new meter.
    pub fn new(resource: impl Into<String>, window: f64) -> Self {
        Self {
            resource: resource.into(),
            consumed: 0.0,
            window: window.max(f64::EPSILON),
        }
    }

    /// Record consumption.
    pub fn record(&mut self, amount: f64) {
        self.consumed += amount.max(0.0);
    }

    /// Consumption rate (units per time window).
    pub fn rate(&self) -> f64 {
        self.consumed / self.window
    }

    /// Reset for a new window.
    pub fn reset(&mut self) {
        self.consumed = 0.0;
    }
}

/// Duplication of data/state across multiple locations for durability.
///
/// Transfers: biology (DNA replication), economics (diversification), PV (data backup).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replication {
    /// Number of desired replicas
    pub desired: usize,
    /// Number of confirmed replicas
    pub confirmed: usize,
}

impl Replication {
    /// Create a new replication spec.
    pub fn new(desired: usize) -> Self {
        Self {
            desired: desired.max(1),
            confirmed: 0,
        }
    }

    /// Confirm one replica.
    pub fn confirm(&mut self) {
        if self.confirmed < self.desired {
            self.confirmed += 1;
        }
    }

    /// Whether all replicas are confirmed.
    pub fn is_fully_replicated(&self) -> bool {
        self.confirmed >= self.desired
    }

    /// Replication factor (confirmed / desired).
    pub fn factor(&self) -> f64 {
        self.confirmed as f64 / self.desired as f64
    }
}

/// Decision logic mapping requests to destinations.
///
/// Transfers: biology (chemotaxis), economics (market routing), PV (case triage).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Routing {
    /// Number of available routes
    pub route_count: usize,
    /// Total requests routed
    pub requests_routed: u64,
}

impl Routing {
    /// Create a new router.
    pub fn new(route_count: usize) -> Self {
        Self {
            route_count: route_count.max(1),
            requests_routed: 0,
        }
    }

    /// Route a request (returns route index via round-robin).
    pub fn route(&mut self) -> usize {
        let idx = (self.requests_routed as usize) % self.route_count;
        self.requests_routed += 1;
        idx
    }

    /// Requests per route (average).
    pub fn load_per_route(&self) -> f64 {
        self.requests_routed as f64 / self.route_count as f64
    }
}

/// Time-bounded exclusive access to a resource.
///
/// Transfers: biology (receptor binding), economics (lease/rent), PV (case lock).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lease {
    /// Resource being leased
    pub resource: String,
    /// Lease holder identifier
    pub holder: String,
    /// Time-to-live in abstract units
    pub ttl: f64,
    /// Elapsed time
    pub elapsed: f64,
}

impl Lease {
    /// Create a new lease.
    pub fn new(resource: impl Into<String>, holder: impl Into<String>, ttl: f64) -> Self {
        Self {
            resource: resource.into(),
            holder: holder.into(),
            ttl: ttl.max(0.0),
            elapsed: 0.0,
        }
    }

    /// Advance time.
    pub fn tick(&mut self, dt: f64) {
        self.elapsed += dt.max(0.0);
    }

    /// Whether the lease has expired.
    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.ttl
    }

    /// Remaining time.
    pub fn remaining(&self) -> f64 {
        (self.ttl - self.elapsed).max(0.0)
    }

    /// Renew the lease (reset elapsed).
    pub fn renew(&mut self) {
        self.elapsed = 0.0;
    }
}

/// Transformation of data to prevent unauthorized access.
///
/// Transfers: biology (protein folding), economics (obfuscation), PV (data anonymization).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Encryption {
    /// Algorithm identifier
    pub algorithm: String,
    /// Key length in bits
    pub key_bits: u32,
    /// Whether currently encrypted
    pub encrypted: bool,
}

impl Encryption {
    /// Create a new encryption spec.
    pub fn new(algorithm: impl Into<String>, key_bits: u32) -> Self {
        Self {
            algorithm: algorithm.into(),
            key_bits,
            encrypted: false,
        }
    }

    /// Mark as encrypted.
    pub fn encrypt(&mut self) {
        self.encrypted = true;
    }

    /// Mark as decrypted.
    pub fn decrypt(&mut self) {
        self.encrypted = false;
    }

    /// Security level estimate (bits of security).
    ///
    /// Symmetric ciphers (AES, ChaCha): security_bits == key_bits.
    /// Asymmetric ciphers (RSA, EC): security_bits ~= key_bits / 2.
    pub fn security_bits(&self) -> u32 {
        let algo_upper = self.algorithm.to_uppercase();
        let is_symmetric = algo_upper.starts_with("AES")
            || algo_upper.starts_with("CHACHA")
            || algo_upper.starts_with("3DES")
            || algo_upper.starts_with("BLOWFISH")
            || algo_upper.starts_with("CAMELLIA");
        if is_symmetric {
            self.key_bits
        } else {
            self.key_bits / 2
        }
    }
}

/// Ordered buffer for asynchronous message passing.
///
/// Transfers: biology (synaptic vesicles), economics (order book), PV (case backlog).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Queue {
    /// Queue name
    pub name: String,
    /// Current depth (pending items)
    pub depth: u64,
    /// Maximum capacity (0 = unbounded)
    pub max_capacity: u64,
    /// Total items processed
    pub processed: u64,
}

impl Queue {
    /// Create a new queue.
    pub fn new(name: impl Into<String>, max_capacity: u64) -> Self {
        Self {
            name: name.into(),
            depth: 0,
            max_capacity,
            processed: 0,
        }
    }

    /// Enqueue an item. Returns Err if at capacity.
    pub fn enqueue(&mut self) -> Result<u64, &'static str> {
        if self.max_capacity > 0 && self.depth >= self.max_capacity {
            return Err("queue full");
        }
        self.depth += 1;
        Ok(self.depth)
    }

    /// Dequeue an item. Returns Err if empty.
    pub fn dequeue(&mut self) -> Result<u64, &'static str> {
        if self.depth == 0 {
            return Err("queue empty");
        }
        self.depth -= 1;
        self.processed += 1;
        Ok(self.depth)
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.depth == 0
    }
}

/// Periodic probe verifying service liveness.
///
/// Transfers: biology (pulse), economics (audit), PV (system uptime check).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Target being checked
    pub target: String,
    /// Consecutive successes
    pub successes: u64,
    /// Consecutive failures
    pub failures: u64,
    /// Failure threshold to mark unhealthy
    pub failure_threshold: u64,
}

impl HealthCheck {
    /// Create a new health check.
    pub fn new(target: impl Into<String>, failure_threshold: u64) -> Self {
        Self {
            target: target.into(),
            successes: 0,
            failures: 0,
            failure_threshold: failure_threshold.max(1),
        }
    }

    /// Record a successful check.
    pub fn record_success(&mut self) {
        self.successes += 1;
        self.failures = 0;
    }

    /// Record a failed check.
    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.successes = 0;
    }

    /// Whether the target is considered healthy.
    pub fn is_healthy(&self) -> bool {
        self.failures < self.failure_threshold
    }
}

/// Ability to dynamically adjust resource allocation based on demand.
///
/// Transfers: biology (tissue growth), economics (workforce scaling), PV (surge staffing).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Elasticity {
    /// Current scale (number of units)
    pub current: u64,
    /// Minimum scale
    pub min: u64,
    /// Maximum scale
    pub max: u64,
}

impl Elasticity {
    /// Create a new elasticity config.
    pub fn new(min: u64, max: u64) -> Self {
        Self {
            current: min,
            min: min.max(1),
            max: max.max(min.max(1)),
        }
    }

    /// Scale up by n units.
    pub fn scale_up(&mut self, n: u64) {
        self.current = (self.current + n).min(self.max);
    }

    /// Scale down by n units.
    pub fn scale_down(&mut self, n: u64) {
        self.current = self.current.saturating_sub(n).max(self.min);
    }

    /// Current utilization as fraction of max.
    pub fn utilization(&self) -> f64 {
        if self.max == 0 {
            return 0.0;
        }
        self.current as f64 / self.max as f64
    }

    /// Whether at maximum scale.
    pub fn is_at_max(&self) -> bool {
        self.current >= self.max
    }

    /// Whether at minimum scale.
    pub fn is_at_min(&self) -> bool {
        self.current <= self.min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T1 tests

    #[test]
    fn test_identity_basic() {
        let id = Identity::new("user-123").with_label("Admin");
        assert_eq!(id.id, "user-123");
        assert_eq!(id.label.as_deref(), Some("Admin"));
    }

    #[test]
    fn test_threshold_upper() {
        let t = Threshold::new(100.0, true);
        assert!(t.exceeded(100.0));
        assert!(t.exceeded(150.0));
        assert!(!t.exceeded(99.0));
        assert!((t.margin(90.0) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_threshold_lower() {
        let t = Threshold::new(10.0, false);
        assert!(t.exceeded(10.0));
        assert!(t.exceeded(5.0));
        assert!(!t.exceeded(15.0));
    }

    #[test]
    fn test_feedback_loop_convergence() {
        let mut fb = FeedbackLoop::new(100.0, 0.5);
        for _ in 0..20 {
            fb.tick();
        }
        assert!(fb.converged(1.0));
    }

    #[test]
    fn test_feedback_loop_error() {
        let fb = FeedbackLoop::new(10.0, 1.0);
        assert!((fb.error() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_idempotency_first_apply() {
        let mut idem = Idempotency::new("op-1");
        assert!(idem.apply());
        assert!(!idem.apply());
        assert!(!idem.apply());
        assert_eq!(idem.attempts(), 3);
        assert!(idem.is_applied());
    }

    #[test]
    fn test_immutability_seal() {
        let mut rec = Immutability::new("audit-log");
        assert!(!rec.is_sealed());
        assert!(rec.seal("sha256:abc").is_ok());
        assert!(rec.is_sealed());
        assert!(rec.seal("sha256:def").is_err());
    }

    #[test]
    fn test_convergence_quorum() {
        let mut c = Convergence::new(5, 0.6);
        assert!(!c.has_converged());
        c.agree();
        c.agree();
        c.agree();
        assert!(c.has_converged());
        assert!((c.agreement_ratio() - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_convergence_remaining() {
        let c = Convergence::new(10, 0.5);
        assert_eq!(c.remaining(), 5);
    }

    // T2-P tests

    #[test]
    fn test_compute_throughput() {
        let c = Compute::new(4.0, 3.0);
        assert!((c.throughput() - 12.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_clamps_negative() {
        let c = Compute::new(-1.0, -2.0);
        assert!((c.capacity - 0.0).abs() < f64::EPSILON);
        assert!((c.frequency - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_storage_lifecycle() {
        let mut s = Storage::new(100.0);
        assert!((s.available() - 100.0).abs() < f64::EPSILON);
        assert!(s.allocate(40.0).is_ok());
        assert!((s.utilization() - 0.4).abs() < f64::EPSILON);
        assert!(s.allocate(70.0).is_err());
    }

    #[test]
    fn test_network_link_bdp() {
        let link = NetworkLink::new("a", "b", 1000.0).with_latency(0.05);
        assert!((link.bdp() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_isolation_boundary_strength() {
        let b = IsolationBoundary::new("dmz", 0.8);
        assert!(b.meets_requirement(0.7));
        assert!(!b.meets_requirement(0.9));
    }

    #[test]
    fn test_permission_matching() {
        let p = Permission::new("alice", "db", "read");
        assert!(p.matches("alice", "db", "read"));
        assert!(!p.matches("bob", "db", "read"));
    }

    #[test]
    fn test_permission_revoke() {
        let mut p = Permission::new("alice", "db", "write");
        p.revoke();
        assert!(!p.matches("alice", "db", "write"));
    }

    #[test]
    fn test_resource_pool_lifecycle() {
        let mut pool = ResourcePool::new(10.0);
        assert!(pool.allocate(3.0).is_ok());
        assert!(pool.allocate(3.0).is_ok());
        pool.release(2.0);
        assert!((pool.available() - 6.0).abs() < f64::EPSILON);
        assert!(pool.allocate(7.0).is_err());
    }

    #[test]
    fn test_metering_rate() {
        let mut m = Metering::new("cpu", 60.0);
        m.record(120.0);
        assert!((m.rate() - 2.0).abs() < f64::EPSILON);
        m.reset();
        assert!((m.consumed - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_replication_factor() {
        let mut r = Replication::new(3);
        r.confirm();
        r.confirm();
        assert!(!r.is_fully_replicated());
        assert!((r.factor() - 2.0 / 3.0).abs() < 0.01);
        r.confirm();
        assert!(r.is_fully_replicated());
    }

    #[test]
    fn test_routing_round_robin() {
        let mut r = Routing::new(3);
        assert_eq!(r.route(), 0);
        assert_eq!(r.route(), 1);
        assert_eq!(r.route(), 2);
        assert_eq!(r.route(), 0);
    }

    #[test]
    fn test_lease_expiry() {
        let mut l = Lease::new("vm-1", "user-a", 10.0);
        assert!(!l.is_expired());
        l.tick(5.0);
        assert!((l.remaining() - 5.0).abs() < f64::EPSILON);
        l.tick(6.0);
        assert!(l.is_expired());
        l.renew();
        assert!(!l.is_expired());
    }

    #[test]
    fn test_encryption_lifecycle() {
        let mut e = Encryption::new("AES", 256);
        assert!(!e.encrypted);
        e.encrypt();
        assert!(e.encrypted);
        e.decrypt();
        assert!(!e.encrypted);
        assert_eq!(e.security_bits(), 256); // AES is symmetric: bits == key_bits
    }

    #[test]
    fn test_queue_lifecycle() {
        let mut q = Queue::new("tasks", 3);
        assert!(q.is_empty());
        assert!(q.enqueue().is_ok());
        assert!(q.enqueue().is_ok());
        assert!(q.enqueue().is_ok());
        assert!(q.enqueue().is_err()); // full
        assert!(q.dequeue().is_ok());
        assert_eq!(q.processed, 1);
        assert_eq!(q.depth, 2);
    }

    #[test]
    fn test_health_check_lifecycle() {
        let mut hc = HealthCheck::new("api", 3);
        assert!(hc.is_healthy());
        hc.record_failure();
        hc.record_failure();
        assert!(hc.is_healthy()); // 2 < 3
        hc.record_failure();
        assert!(!hc.is_healthy()); // 3 >= 3
        hc.record_success();
        assert!(hc.is_healthy()); // reset
    }

    #[test]
    fn test_elasticity_scaling() {
        let mut e = Elasticity::new(1, 10);
        assert_eq!(e.current, 1);
        e.scale_up(3);
        assert_eq!(e.current, 4);
        e.scale_up(100);
        assert_eq!(e.current, 10); // capped
        assert!(e.is_at_max());
        e.scale_down(100);
        assert_eq!(e.current, 1); // floored
        assert!(e.is_at_min());
    }

    // Serde round-trip tests

    #[test]
    fn test_serde_identity() {
        let orig = Identity::new("test-123").with_label("Test");
        let json = serde_json::to_string(&orig).ok();
        assert!(json.is_some());
        let back: Result<Identity, _> = serde_json::from_str(json.as_deref().unwrap_or(""));
        assert!(back.is_ok());
        assert_eq!(back.ok(), Some(orig));
    }

    #[test]
    fn test_serde_all_t1() {
        let threshold = Threshold::new(5.0, true);
        let json = serde_json::to_string(&threshold).ok();
        assert!(json.is_some());

        let fb = FeedbackLoop::new(10.0, 0.5);
        let json = serde_json::to_string(&fb).ok();
        assert!(json.is_some());

        let idem = Idempotency::new("k");
        let json = serde_json::to_string(&idem).ok();
        assert!(json.is_some());

        let imm = Immutability::new("rec");
        let json = serde_json::to_string(&imm).ok();
        assert!(json.is_some());

        let conv = Convergence::new(3, 0.5);
        let json = serde_json::to_string(&conv).ok();
        assert!(json.is_some());
    }

    #[test]
    fn test_serde_all_t2p() {
        // Verify all T2-P types survive serde round-trip
        let types_json: Vec<String> = vec![
            serde_json::to_string(&Compute::new(1.0, 1.0)).unwrap_or_default(),
            serde_json::to_string(&Storage::new(100.0)).unwrap_or_default(),
            serde_json::to_string(&NetworkLink::new("a", "b", 10.0)).unwrap_or_default(),
            serde_json::to_string(&IsolationBoundary::new("test", 0.5)).unwrap_or_default(),
            serde_json::to_string(&Permission::new("a", "b", "c")).unwrap_or_default(),
            serde_json::to_string(&ResourcePool::new(10.0)).unwrap_or_default(),
            serde_json::to_string(&Metering::new("cpu", 60.0)).unwrap_or_default(),
            serde_json::to_string(&Replication::new(3)).unwrap_or_default(),
            serde_json::to_string(&Routing::new(2)).unwrap_or_default(),
            serde_json::to_string(&Lease::new("r", "h", 10.0)).unwrap_or_default(),
            serde_json::to_string(&Encryption::new("AES", 256)).unwrap_or_default(),
            serde_json::to_string(&Queue::new("q", 10)).unwrap_or_default(),
            serde_json::to_string(&HealthCheck::new("t", 3)).unwrap_or_default(),
            serde_json::to_string(&Elasticity::new(1, 10)).unwrap_or_default(),
        ];
        for json in &types_json {
            assert!(
                !json.is_empty(),
                "Serialization should not produce empty string"
            );
        }
    }
}
