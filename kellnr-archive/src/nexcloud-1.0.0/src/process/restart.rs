use std::time::Duration;

/// Exponential backoff restart policy.
///
/// Tier: T2-P (ρ Recursion) — each retry doubles the wait, recursing toward max.
#[derive(Debug, Clone)]
pub struct RestartPolicy {
    pub max_retries: u32,
    initial_backoff: Duration,
    multiplier: f64,
    max_backoff: Duration,
    current_attempt: u32,
}

impl RestartPolicy {
    /// Create a new restart policy.
    pub fn new(max_retries: u32, initial_backoff_ms: u64) -> Self {
        Self {
            max_retries,
            initial_backoff: Duration::from_millis(initial_backoff_ms),
            multiplier: 2.0,
            max_backoff: Duration::from_secs(60),
            current_attempt: 0,
        }
    }

    /// Whether we should attempt another restart.
    pub fn should_retry(&self) -> bool {
        self.current_attempt < self.max_retries
    }

    /// Get the next backoff duration and increment attempt counter.
    pub fn next_backoff(&mut self) -> Duration {
        let backoff_ms = self.initial_backoff.as_millis() as f64
            * self.multiplier.powi(self.current_attempt as i32);
        let backoff = Duration::from_millis(backoff_ms as u64).min(self.max_backoff);
        self.current_attempt += 1;
        backoff
    }

    /// Reset the policy after sustained health (called when service stays healthy).
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// Current attempt number (0-indexed).
    pub fn attempts(&self) -> u32 {
        self.current_attempt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_backoff() {
        let mut policy = RestartPolicy::new(5, 1000);
        assert!(policy.should_retry());

        let b1 = policy.next_backoff();
        assert_eq!(b1, Duration::from_millis(1000));

        let b2 = policy.next_backoff();
        assert_eq!(b2, Duration::from_millis(2000));

        let b3 = policy.next_backoff();
        assert_eq!(b3, Duration::from_millis(4000));
    }

    #[test]
    fn backoff_caps_at_max() {
        let mut policy = RestartPolicy::new(10, 1000);
        // 2^6 * 1000 = 64000 > 60000 max
        for _ in 0..6 {
            policy.next_backoff();
        }
        let b = policy.next_backoff();
        assert!(b <= Duration::from_secs(60));
    }

    #[test]
    fn stops_retrying_at_max() {
        let mut policy = RestartPolicy::new(3, 100);
        assert!(policy.should_retry());
        policy.next_backoff();
        policy.next_backoff();
        policy.next_backoff();
        assert!(!policy.should_retry());
    }

    #[test]
    fn reset_clears_attempts() {
        let mut policy = RestartPolicy::new(3, 100);
        policy.next_backoff();
        policy.next_backoff();
        assert_eq!(policy.attempts(), 2);
        policy.reset();
        assert_eq!(policy.attempts(), 0);
        assert!(policy.should_retry());
    }

    #[test]
    fn zero_max_retries_never_retries() {
        let policy = RestartPolicy::new(0, 1000);
        assert!(!policy.should_retry());
        assert_eq!(policy.attempts(), 0);
    }

    #[test]
    fn zero_backoff_stays_zero() {
        let mut policy = RestartPolicy::new(5, 0);
        let b1 = policy.next_backoff();
        let b2 = policy.next_backoff();
        assert_eq!(b1, Duration::from_millis(0));
        assert_eq!(b2, Duration::from_millis(0));
    }

    #[test]
    fn single_retry_allowed() {
        let mut policy = RestartPolicy::new(1, 500);
        assert!(policy.should_retry());
        policy.next_backoff();
        assert!(!policy.should_retry());
    }

    #[test]
    fn clone_preserves_state() {
        let mut policy = RestartPolicy::new(5, 100);
        policy.next_backoff();
        policy.next_backoff();

        let cloned = policy.clone();
        assert_eq!(cloned.attempts(), 2);
        assert!(cloned.should_retry());
    }

    #[test]
    fn reset_after_exhaustion_allows_retries() {
        let mut policy = RestartPolicy::new(2, 100);
        policy.next_backoff();
        policy.next_backoff();
        assert!(!policy.should_retry());
        policy.reset();
        assert!(policy.should_retry());
        assert_eq!(policy.attempts(), 0);
    }

    #[test]
    fn backoff_sequence_is_deterministic() {
        let mut p1 = RestartPolicy::new(5, 1000);
        let mut p2 = RestartPolicy::new(5, 1000);

        for _ in 0..5 {
            assert_eq!(p1.next_backoff(), p2.next_backoff());
        }
    }

    #[test]
    fn max_retries_field_accessible() {
        let policy = RestartPolicy::new(42, 100);
        assert_eq!(policy.max_retries, 42);
    }
}
