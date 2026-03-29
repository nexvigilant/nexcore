//! Endocytosis - selective signal absorption across trust boundaries.
//!
//! ## T1 Grounding
//!
//! - `Vesicle` → ∂ (boundary) - signal container that crosses membrane
//! - `VesiclePool` → ρ (recursion) - internalized signals processed recursively
//! - `EndocyticReceptor` → μ (mapping) - selective binding determines what enters
//! - `InternalizationPolicy` → κ (comparison) - criteria for acceptance
//!
//! ## Biological Analog
//!
//! Endocytosis is the process by which cells absorb external material:
//! 1. **Receptor-mediated**: Specific receptors bind specific ligands (selective)
//! 2. **Clathrin-coated pit**: Signal is wrapped in a vesicle (boundary crossing)
//! 3. **Endosome processing**: Internalized signals are recursively processed
//! 4. **Recycling**: Receptors return to surface; waste is degraded
//!
//! ## Claude Code Analog
//!
//! Selective signal absorption with capacity limits and recursive processing:
//! - **Receptor binding**: Only signals matching policy are internalized
//! - **Vesicle buffer**: Capacity-limited buffer prevents overload (backpressure)
//! - **Processing cascade**: Internalized signals trigger internal state changes
//! - **Receptor recycling**: After processing, capacity is restored

use crate::{Cytokine, CytokineFamily, ThreatLevel};
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Policy for what signals are accepted for internalization.
///
/// # Tier: T2-P
/// Grounds to: κ (comparison — accept/reject criteria)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalizationPolicy {
    /// Accepted cytokine families (empty = accept all)
    pub accepted_families: Vec<CytokineFamily>,

    /// Minimum severity to internalize
    pub min_severity: ThreatLevel,

    /// Whether to accept cascadable signals only
    pub cascadable_only: bool,

    /// Maximum signal age in seconds (reject stale signals)
    pub max_age_secs: u32,
}

impl Default for InternalizationPolicy {
    fn default() -> Self {
        Self {
            accepted_families: Vec::new(),
            min_severity: ThreatLevel::Low,
            cascadable_only: false,
            max_age_secs: 600, // 10 minutes
        }
    }
}

impl InternalizationPolicy {
    /// Create a policy that accepts a specific family.
    pub fn for_family(family: CytokineFamily) -> Self {
        Self {
            accepted_families: vec![family],
            ..Default::default()
        }
    }

    /// Create a policy that only accepts high-severity signals.
    pub fn high_priority() -> Self {
        Self {
            min_severity: ThreatLevel::High,
            ..Default::default()
        }
    }

    /// Set minimum severity.
    #[must_use]
    pub fn with_min_severity(mut self, severity: ThreatLevel) -> Self {
        self.min_severity = severity;
        self
    }

    /// Check if a signal satisfies this policy.
    pub fn accepts(&self, signal: &Cytokine) -> bool {
        // Family check
        if !self.accepted_families.is_empty() && !self.accepted_families.contains(&signal.family) {
            return false;
        }

        // Severity check
        if signal.severity < self.min_severity {
            return false;
        }

        // Cascadable check
        if self.cascadable_only && !signal.cascadable {
            return false;
        }

        // Age check
        if self.max_age_secs > 0 {
            let age = DateTime::now()
                .signed_duration_since(signal.emitted_at)
                .num_seconds();
            if age > i64::from(self.max_age_secs) {
                return false;
            }
        }

        true
    }
}

/// A vesicle containing an internalized signal.
///
/// The signal has crossed the membrane boundary and is now
/// inside the cell's processing environment.
///
/// # Tier: T2-P
/// Grounds to: ∂ (boundary — signal wrapped in container)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vesicle {
    /// The internalized cytokine signal
    pub signal: Cytokine,

    /// When the signal was internalized
    pub internalized_at: DateTime,

    /// Processing state
    pub state: VesicleState,

    /// Number of times this vesicle has been processed
    pub processing_depth: u8,
}

/// State of a vesicle in the endocytic pathway.
///
/// # Tier: T2-P
/// Grounds to: ς (state)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VesicleState {
    /// Just internalized, awaiting processing
    EarlyEndosome,
    /// Being actively processed
    LateEndosome,
    /// Processing complete, awaiting response emission
    Lysosome,
    /// Vesicle recycled, receptor returned to surface
    Recycled,
}

impl Vesicle {
    /// Create a new vesicle wrapping a signal.
    pub fn new(signal: Cytokine) -> Self {
        Self {
            signal,
            internalized_at: DateTime::now(),
            state: VesicleState::EarlyEndosome,
            processing_depth: 0,
        }
    }

    /// Advance the vesicle to the next processing state.
    pub fn advance(&mut self) {
        self.state = match self.state {
            VesicleState::EarlyEndosome => VesicleState::LateEndosome,
            VesicleState::LateEndosome => VesicleState::Lysosome,
            VesicleState::Lysosome | VesicleState::Recycled => VesicleState::Recycled,
        };
        self.processing_depth += 1;
    }

    /// Check if processing is complete.
    pub fn is_processed(&self) -> bool {
        matches!(self.state, VesicleState::Lysosome | VesicleState::Recycled)
    }

    /// Check if the vesicle has been recycled.
    pub fn is_recycled(&self) -> bool {
        self.state == VesicleState::Recycled
    }

    /// Time since internalization in seconds.
    pub fn age_secs(&self) -> i64 {
        DateTime::now()
            .signed_duration_since(self.internalized_at)
            .num_seconds()
    }
}

/// Result of an internalization attempt.
///
/// # Tier: T2-P
/// Grounds to: ∃ (existence — did the signal enter or not)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalizationResult {
    /// Signal was accepted and internalized
    Accepted,
    /// Signal rejected by policy
    Rejected,
    /// Pool is at capacity — backpressure
    AtCapacity,
    /// Signal is expired/stale
    Expired,
}

/// A pool of vesicles with capacity limits.
///
/// Models the endosomal compartment with bounded capacity.
/// When full, new signals are rejected (backpressure signal).
///
/// # Tier: T2-C (Composite)
/// Grounds to: ∂ (boundary) + ρ (recursion via cascade processing)
#[derive(Debug)]
pub struct VesiclePool {
    /// Internalized vesicles awaiting or undergoing processing
    vesicles: VecDeque<Vesicle>,

    /// Maximum number of vesicles the pool can hold
    capacity: usize,

    /// Policy for accepting new signals
    policy: InternalizationPolicy,

    /// Total signals internalized (lifetime counter)
    total_internalized: u64,

    /// Total signals rejected (lifetime counter)
    total_rejected: u64,

    /// Total vesicles recycled (lifetime counter)
    total_recycled: u64,
}

impl VesiclePool {
    /// Create a new vesicle pool with given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            vesicles: VecDeque::with_capacity(capacity),
            capacity,
            policy: InternalizationPolicy::default(),
            total_internalized: 0,
            total_rejected: 0,
            total_recycled: 0,
        }
    }

    /// Set the internalization policy.
    #[must_use]
    pub fn with_policy(mut self, policy: InternalizationPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Attempt to internalize a signal.
    ///
    /// The signal must pass the policy check and the pool must have capacity.
    pub fn internalize(&mut self, signal: Cytokine) -> InternalizationResult {
        // Check if signal is expired
        if signal.is_expired() {
            self.total_rejected += 1;
            return InternalizationResult::Expired;
        }

        // Check policy
        if !self.policy.accepts(&signal) {
            self.total_rejected += 1;
            return InternalizationResult::Rejected;
        }

        // Check capacity
        if self.vesicles.len() >= self.capacity {
            self.total_rejected += 1;
            return InternalizationResult::AtCapacity;
        }

        // Internalize: wrap signal in vesicle
        self.vesicles.push_back(Vesicle::new(signal));
        self.total_internalized += 1;
        InternalizationResult::Accepted
    }

    /// Process one step of all vesicles in the pool.
    ///
    /// Advances each vesicle through its endocytic pathway.
    /// Returns response signals generated by fully processed vesicles.
    pub fn process_step(&mut self) -> Vec<Cytokine> {
        let mut responses = Vec::new();

        for vesicle in &mut self.vesicles {
            if vesicle.is_recycled() {
                continue;
            }

            vesicle.advance();

            // When vesicle reaches Lysosome state, generate response signals
            if vesicle.state == VesicleState::Lysosome {
                // Generate response based on internalized signal
                if let Some(response) = generate_endocytic_response(&vesicle.signal) {
                    responses.push(response);
                }
            }
        }

        responses
    }

    /// Recycle all fully processed vesicles, freeing capacity.
    ///
    /// Returns the number of vesicles recycled.
    pub fn recycle(&mut self) -> usize {
        let before = self.vesicles.len();

        // Advance any Lysosome vesicles to Recycled before removing
        for vesicle in &mut self.vesicles {
            if vesicle.state == VesicleState::Lysosome {
                vesicle.advance();
            }
        }

        // Remove recycled vesicles
        self.vesicles.retain(|v| !v.is_recycled());

        let recycled = before - self.vesicles.len();
        self.total_recycled += recycled as u64;
        recycled
    }

    /// Get current pool occupancy.
    pub fn occupancy(&self) -> usize {
        self.vesicles.len()
    }

    /// Get remaining capacity.
    pub fn remaining_capacity(&self) -> usize {
        self.capacity.saturating_sub(self.vesicles.len())
    }

    /// Get pool utilization as a fraction \[0.0, 1.0\].
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            return 1.0;
        }
        // Precision loss acceptable: vesicle counts are small
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        {
            self.vesicles.len() as f64 / self.capacity as f64
        }
    }

    /// Check if the pool is at capacity.
    pub fn is_full(&self) -> bool {
        self.vesicles.len() >= self.capacity
    }

    /// Get pool statistics.
    pub fn stats(&self) -> PoolStats {
        let mut by_state = std::collections::HashMap::new();
        for v in &self.vesicles {
            *by_state.entry(format!("{:?}", v.state)).or_insert(0u64) += 1;
        }

        PoolStats {
            capacity: self.capacity,
            occupancy: self.vesicles.len(),
            utilization: self.utilization(),
            total_internalized: self.total_internalized,
            total_rejected: self.total_rejected,
            total_recycled: self.total_recycled,
            by_state,
        }
    }

    /// Get the policy.
    pub fn policy(&self) -> &InternalizationPolicy {
        &self.policy
    }

    /// Peek at the next vesicle to be processed (FIFO).
    pub fn peek(&self) -> Option<&Vesicle> {
        self.vesicles.front()
    }

    /// Get all vesicles in a specific state.
    pub fn vesicles_in_state(&self, state: VesicleState) -> Vec<&Vesicle> {
        self.vesicles.iter().filter(|v| v.state == state).collect()
    }

    /// Drain all processed vesicles and return their signals.
    ///
    /// This is the "antigen presentation" step — expose internalized
    /// signal contents to the broader immune system.
    pub fn present_antigens(&mut self) -> Vec<Cytokine> {
        let processed: Vec<_> = self
            .vesicles
            .iter()
            .filter(|v| v.is_processed())
            .map(|v| v.signal.clone())
            .collect();

        // Mark all processed vesicles as recycled
        for vesicle in &mut self.vesicles {
            if vesicle.is_processed() {
                vesicle.state = VesicleState::Recycled;
            }
        }

        // Clean up
        self.recycle();

        processed
    }
}

/// Statistics for a vesicle pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Maximum pool capacity
    pub capacity: usize,
    /// Current occupancy
    pub occupancy: usize,
    /// Utilization fraction
    pub utilization: f64,
    /// Lifetime internalized count
    pub total_internalized: u64,
    /// Lifetime rejected count
    pub total_rejected: u64,
    /// Lifetime recycled count
    pub total_recycled: u64,
    /// Vesicle counts by state
    pub by_state: std::collections::HashMap<String, u64>,
}

/// Generate a response signal from a fully processed vesicle.
///
/// Maps the internalized signal to an appropriate immune response:
/// - IL-1 (alarm) → IFN-γ (amplify defenses)
/// - TNF-α (terminate) → IL-10 (suppress further destruction)
/// - IFN-γ (activation) → IL-2 (proliferate responders)
/// - Other → IL-6 (acute phase coordination)
fn generate_endocytic_response(signal: &Cytokine) -> Option<Cytokine> {
    let response = match signal.family {
        CytokineFamily::Il1 => {
            // Alarm internalized → amplify defenses
            Cytokine::new(CytokineFamily::IfnGamma, "defense_amplification")
                .with_severity(signal.severity)
                .with_source("endosome")
        }
        CytokineFamily::TnfAlpha => {
            // Termination signal → suppress after processing
            Cytokine::suppress("post_termination_regulation").with_source("endosome")
        }
        CytokineFamily::IfnGamma => {
            // Activation → proliferate
            Cytokine::new(CytokineFamily::Il2, "proliferation_trigger")
                .with_severity(ThreatLevel::Medium)
                .with_source("endosome")
        }
        CytokineFamily::Custom(_) => return None,
        _ => {
            // Default: coordinate acute response
            Cytokine::new(CytokineFamily::Il6, "acute_coordination")
                .with_severity(ThreatLevel::Medium)
                .with_source("endosome")
        }
    };

    Some(response.no_cascade()) // Endocytic responses don't cascade (prevents loops)
}

/// Trait for components that perform endocytosis.
///
/// # Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: ∂ (boundary crossing) + ρ (recursive processing)
pub trait EndocyticReceptor {
    /// Get the vesicle pool for internalization.
    fn pool(&self) -> &VesiclePool;

    /// Get mutable access to the vesicle pool.
    fn pool_mut(&mut self) -> &mut VesiclePool;

    /// Attempt to internalize a signal.
    fn internalize(&mut self, signal: Cytokine) -> InternalizationResult {
        self.pool_mut().internalize(signal)
    }

    /// Process one step of the endocytic pathway.
    fn process(&mut self) -> Vec<Cytokine> {
        self.pool_mut().process_step()
    }

    /// Recycle processed vesicles to free capacity.
    fn recycle(&mut self) -> usize {
        self.pool_mut().recycle()
    }

    /// Get pool utilization.
    fn utilization(&self) -> f64 {
        self.pool().utilization()
    }
}

/// A simple endocytic receptor implementation.
///
/// # Tier: T2-C (Composite)
pub struct SimpleEndocyticReceptor {
    id: String,
    pool: VesiclePool,
}

impl SimpleEndocyticReceptor {
    /// Create a new endocytic receptor with given capacity.
    pub fn new(id: impl Into<String>, capacity: usize) -> Self {
        Self {
            id: id.into(),
            pool: VesiclePool::new(capacity),
        }
    }

    /// Set the internalization policy.
    #[must_use]
    pub fn with_policy(mut self, policy: InternalizationPolicy) -> Self {
        self.pool = self.pool.with_policy(policy);
        self
    }

    /// Get the receptor ID.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl EndocyticReceptor for SimpleEndocyticReceptor {
    fn pool(&self) -> &VesiclePool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut VesiclePool {
        &mut self.pool
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::Scope;

    #[test]
    fn test_internalization_policy_default() {
        let policy = InternalizationPolicy::default();
        let signal = Cytokine::new(CytokineFamily::Il1, "test").with_severity(ThreatLevel::Medium);

        assert!(policy.accepts(&signal));
    }

    #[test]
    fn test_internalization_policy_family_filter() {
        let policy = InternalizationPolicy::for_family(CytokineFamily::Il1);

        let il1 = Cytokine::new(CytokineFamily::Il1, "test");
        let tnf = Cytokine::new(CytokineFamily::TnfAlpha, "test");

        assert!(policy.accepts(&il1));
        assert!(!policy.accepts(&tnf));
    }

    #[test]
    fn test_internalization_policy_severity_filter() {
        let policy = InternalizationPolicy::high_priority();

        let low = Cytokine::new(CytokineFamily::Il1, "test").with_severity(ThreatLevel::Low);
        let critical =
            Cytokine::new(CytokineFamily::Il1, "test").with_severity(ThreatLevel::Critical);

        assert!(!policy.accepts(&low));
        assert!(policy.accepts(&critical));
    }

    #[test]
    fn test_vesicle_lifecycle() {
        let signal = Cytokine::new(CytokineFamily::Il1, "test");
        let mut vesicle = Vesicle::new(signal);

        assert_eq!(vesicle.state, VesicleState::EarlyEndosome);
        assert!(!vesicle.is_processed());

        vesicle.advance();
        assert_eq!(vesicle.state, VesicleState::LateEndosome);
        assert!(!vesicle.is_processed());

        vesicle.advance();
        assert_eq!(vesicle.state, VesicleState::Lysosome);
        assert!(vesicle.is_processed());

        vesicle.advance();
        assert_eq!(vesicle.state, VesicleState::Recycled);
        assert!(vesicle.is_recycled());
    }

    #[test]
    fn test_vesicle_pool_basic() {
        let mut pool = VesiclePool::new(3);
        assert_eq!(pool.occupancy(), 0);
        assert_eq!(pool.remaining_capacity(), 3);
        assert!(!pool.is_full());
    }

    #[test]
    fn test_vesicle_pool_internalize() {
        let mut pool = VesiclePool::new(2);

        let result = pool.internalize(Cytokine::new(CytokineFamily::Il1, "signal_1"));
        assert_eq!(result, InternalizationResult::Accepted);
        assert_eq!(pool.occupancy(), 1);

        let result = pool.internalize(Cytokine::new(CytokineFamily::Il6, "signal_2"));
        assert_eq!(result, InternalizationResult::Accepted);
        assert_eq!(pool.occupancy(), 2);

        // Pool is now full
        let result = pool.internalize(Cytokine::new(CytokineFamily::TnfAlpha, "signal_3"));
        assert_eq!(result, InternalizationResult::AtCapacity);
        assert_eq!(pool.occupancy(), 2);
    }

    #[test]
    fn test_vesicle_pool_policy_rejection() {
        let policy = InternalizationPolicy::for_family(CytokineFamily::Il1);
        let mut pool = VesiclePool::new(10).with_policy(policy);

        let result = pool.internalize(Cytokine::new(CytokineFamily::TnfAlpha, "wrong_family"));
        assert_eq!(result, InternalizationResult::Rejected);
        assert_eq!(pool.occupancy(), 0);
    }

    #[test]
    fn test_vesicle_pool_process_step() {
        let mut pool = VesiclePool::new(5);

        pool.internalize(Cytokine::new(CytokineFamily::Il1, "alarm"));

        // Step 1: EarlyEndosome → LateEndosome
        let responses = pool.process_step();
        assert!(responses.is_empty());
        let early_vesicles = pool.vesicles_in_state(VesicleState::LateEndosome);
        assert_eq!(early_vesicles.len(), 1);

        // Step 2: LateEndosome → Lysosome (generates response)
        let responses = pool.process_step();
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].family, CytokineFamily::IfnGamma);
    }

    #[test]
    fn test_vesicle_pool_recycle() {
        let mut pool = VesiclePool::new(5);

        pool.internalize(Cytokine::new(CytokineFamily::Il1, "test"));

        // Process through full lifecycle
        pool.process_step(); // → LateEndosome
        pool.process_step(); // → Lysosome

        let recycled = pool.recycle();
        assert_eq!(recycled, 1);
        assert_eq!(pool.occupancy(), 0);
    }

    #[test]
    fn test_vesicle_pool_utilization() {
        let mut pool = VesiclePool::new(4);
        assert!((pool.utilization() - 0.0).abs() < f64::EPSILON);

        pool.internalize(Cytokine::new(CytokineFamily::Il1, "a"));
        pool.internalize(Cytokine::new(CytokineFamily::Il1, "b"));
        assert!((pool.utilization() - 0.5).abs() < f64::EPSILON);

        pool.internalize(Cytokine::new(CytokineFamily::Il1, "c"));
        pool.internalize(Cytokine::new(CytokineFamily::Il1, "d"));
        assert!((pool.utilization() - 1.0).abs() < f64::EPSILON);
        assert!(pool.is_full());
    }

    #[test]
    fn test_vesicle_pool_stats() {
        let mut pool = VesiclePool::new(5);

        pool.internalize(Cytokine::new(CytokineFamily::Il1, "accepted"));
        pool.internalize(Cytokine::new(CytokineFamily::Il1, "accepted2"));

        let stats = pool.stats();
        assert_eq!(stats.capacity, 5);
        assert_eq!(stats.occupancy, 2);
        assert_eq!(stats.total_internalized, 2);
        assert_eq!(stats.total_rejected, 0);
    }

    #[test]
    fn test_endocytic_response_il1_to_ifn_gamma() {
        let signal = Cytokine::new(CytokineFamily::Il1, "alarm").with_severity(ThreatLevel::High);
        let response = generate_endocytic_response(&signal);

        assert!(response.is_some());
        let r = response.unwrap_or_else(|| Cytokine::new(CytokineFamily::Il1, "fallback"));
        assert_eq!(r.family, CytokineFamily::IfnGamma);
        assert!(!r.cascadable); // Endocytic responses don't cascade
    }

    #[test]
    fn test_endocytic_response_tnf_to_suppression() {
        let signal = Cytokine::terminate("threat");
        let response = generate_endocytic_response(&signal);

        assert!(response.is_some());
        let r = response.unwrap_or_else(|| Cytokine::new(CytokineFamily::Il1, "fallback"));
        assert_eq!(r.family, CytokineFamily::Il10);
    }

    #[test]
    fn test_simple_endocytic_receptor() {
        let mut receptor = SimpleEndocyticReceptor::new("test_receptor", 3);

        // Internalize
        let result = receptor.internalize(Cytokine::new(CytokineFamily::Il1, "test"));
        assert_eq!(result, InternalizationResult::Accepted);
        assert!((receptor.utilization() - (1.0 / 3.0)).abs() < 0.01);

        // Process
        let responses = receptor.process(); // → LateEndosome
        assert!(responses.is_empty());

        let responses = receptor.process(); // → Lysosome (generates response)
        assert_eq!(responses.len(), 1);

        // Recycle
        let recycled = receptor.recycle();
        assert_eq!(recycled, 1);
        assert!((receptor.utilization() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_present_antigens() {
        let mut pool = VesiclePool::new(5);

        pool.internalize(Cytokine::new(CytokineFamily::Il1, "alarm1"));
        pool.internalize(Cytokine::new(CytokineFamily::Il6, "acute1"));

        // Process to Lysosome state
        pool.process_step(); // → LateEndosome
        pool.process_step(); // → Lysosome

        // Present antigens (returns original internalized signals)
        let antigens = pool.present_antigens();
        assert_eq!(antigens.len(), 2);

        // Pool should be empty after presentation + recycling
        assert_eq!(pool.occupancy(), 0);
    }

    #[test]
    fn test_endocytic_receptor_with_policy() {
        let policy = InternalizationPolicy::for_family(CytokineFamily::Il1)
            .with_min_severity(ThreatLevel::High);

        let mut receptor = SimpleEndocyticReceptor::new("selective", 5).with_policy(policy);

        // Rejected: wrong family
        let result = receptor.internalize(
            Cytokine::new(CytokineFamily::TnfAlpha, "wrong").with_severity(ThreatLevel::Critical),
        );
        assert_eq!(result, InternalizationResult::Rejected);

        // Rejected: too low severity
        let result = receptor
            .internalize(Cytokine::new(CytokineFamily::Il1, "low").with_severity(ThreatLevel::Low));
        assert_eq!(result, InternalizationResult::Rejected);

        // Accepted: correct family + high severity
        let result = receptor.internalize(
            Cytokine::new(CytokineFamily::Il1, "important").with_severity(ThreatLevel::High),
        );
        assert_eq!(result, InternalizationResult::Accepted);
    }

    #[test]
    fn test_zero_capacity_pool() {
        let mut pool = VesiclePool::new(0);
        assert!(pool.is_full());
        assert!((pool.utilization() - 1.0).abs() < f64::EPSILON);

        let result = pool.internalize(Cytokine::new(CytokineFamily::Il1, "test"));
        assert_eq!(result, InternalizationResult::AtCapacity);
    }
}
