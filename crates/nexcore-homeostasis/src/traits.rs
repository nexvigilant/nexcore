//! Sensor and Actuator traits — the dependency inversion contracts.
//!
//! The control loop depends on these traits, not concrete implementations.
//! Users provide sensors that read metrics and actuators that execute actions.
//!
//! ## Sync → Async Bridge
//!
//! [`SyncSensorAdapter`] bridges the synchronous [`SyncSensor`] interface
//! (implemented by all concrete sensors in `nexcore-homeostasis-sensing`) to
//! the async [`Sensor`] trait required by the control loop.
//!
//! Direction of data flow:
//! ```text
//! [concrete sensor] --record()--> [in-process history + assessment]
//!                                         |
//!                    SyncSensor::read_current()  (synchronous, &self)
//!                                         |
//!                    SyncSensorAdapter::read()   (async, returns Pin<Box<...>>)
//!                                         |
//!                    [Homeostasis control loop]
//! ```
//!
//! The adapter wraps `Arc<Mutex<S>>` so that mutable state (value history,
//! pattern matching) is safely shared between the recording side and the
//! async read side without extra copying.

use nexcore_error::Result;
use nexcore_homeostasis_primitives::{
    ActionData, ActionResult, ActionType, SensorReading, SensorType,
};
use nexcore_homeostasis_sensing::SyncSensor;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// A sensor that produces readings.
pub trait Sensor: Send + Sync {
    /// Human-readable name of this sensor.
    fn name(&self) -> &str;

    /// Sensor category.
    fn sensor_type(&self) -> SensorType;

    /// Take a reading. Returns `None` if the sensor has no data.
    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Option<SensorReading>>> + Send + '_>>;
}

/// An actuator that executes actions.
pub trait Actuator: Send + Sync {
    /// Human-readable name of this actuator.
    fn name(&self) -> &str;

    /// Execute an action and return the result.
    fn execute(
        &self,
        action: &ActionData,
    ) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send + '_>>;
}

// =============================================================================
// CallbackSensor
// =============================================================================

/// A sensor that wraps an async closure.
///
/// # Example
///
/// ```no_run
/// use nexcore_homeostasis::traits::CallbackSensor;
/// use nexcore_homeostasis::primitives::{SensorReading, SensorType};
///
/// let sensor = CallbackSensor::new("cpu", SensorType::SelfMeasurement, || {
///     Box::pin(async {
///         Ok(Some(SensorReading::normal(0.42, "cpu", SensorType::SelfMeasurement)))
///     })
/// });
/// assert_eq!(sensor.name(), "cpu");
/// ```
pub struct CallbackSensor<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<Option<SensorReading>>> + Send>> + Send + Sync,
{
    name: String,
    sensor_type: SensorType,
    callback: F,
}

impl<F> CallbackSensor<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<Option<SensorReading>>> + Send>> + Send + Sync,
{
    /// Create a new `CallbackSensor`.
    pub fn new(name: impl Into<String>, sensor_type: SensorType, callback: F) -> Self {
        Self {
            name: name.into(),
            sensor_type,
            callback,
        }
    }
}

impl<F> Sensor for CallbackSensor<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<Option<SensorReading>>> + Send>> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }

    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Option<SensorReading>>> + Send + '_>> {
        (self.callback)()
    }
}

// =============================================================================
// LoggingActuator
// =============================================================================

/// An actuator that logs actions to tracing instead of executing them.
///
/// Useful as a no-op placeholder during development and testing.
pub struct LoggingActuator {
    name: String,
}

impl LoggingActuator {
    /// Create a new `LoggingActuator` with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Actuator for LoggingActuator {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        action: &ActionData,
    ) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send + '_>> {
        let action_type = action.action_type;
        let target = action.target_response_level;
        let name = self.name.clone();
        Box::pin(async move {
            tracing::info!(
                actuator = %name,
                action = ?action_type,
                target = target,
                "LoggingActuator: would execute action"
            );
            Ok(ActionResult::success(action_type, Some(target), 0.0))
        })
    }
}

// =============================================================================
// CallbackActuator
// =============================================================================

/// An actuator that delegates action execution to a user-provided async closure.
///
/// Maps to Python's `CallbackActuator` in `response/actuators.py`.
/// Useful for integrating with external systems (Kubernetes, load balancers,
/// alerting APIs) where the handler is supplied at construction time.
///
/// # Example
///
/// ```no_run
/// use nexcore_homeostasis::traits::{Actuator, CallbackActuator};
/// use nexcore_homeostasis::primitives::{ActionData, ActionResult, ActionType};
///
/// let actuator = CallbackActuator::new("k8s", |action| {
///     Box::pin(async move {
///         Ok(ActionResult::success(
///             action.action_type,
///             Some(action.target_response_level),
///             0.0,
///         ))
///     })
/// });
/// assert_eq!(actuator.name(), "k8s");
/// ```
pub struct CallbackActuator<F>
where
    F: Fn(ActionData) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send>> + Send + Sync,
{
    name: String,
    handler: F,
}

impl<F> CallbackActuator<F>
where
    F: Fn(ActionData) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send>> + Send + Sync,
{
    /// Create a new `CallbackActuator` wrapping the given async handler.
    ///
    /// The handler receives an owned [`ActionData`] so the returned future can
    /// be `'static` and safely moved across await points.
    pub fn new(name: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            handler,
        }
    }
}

impl<F> Actuator for CallbackActuator<F>
where
    F: Fn(ActionData) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send>> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        action: &ActionData,
    ) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send + '_>> {
        (self.handler)(action.clone())
    }
}

// =============================================================================
// CompositeActuator
// =============================================================================

/// An actuator that fans an action out to multiple sub-actuators.
///
/// Maps to Python's `CompositeActuator` in `response/actuators.py`.
/// Every sub-actuator receives the action in sequence. The composite result
/// is successful only if every sub-actuator succeeded.
pub struct CompositeActuator {
    name: String,
    actuators: Vec<Arc<dyn Actuator>>,
}

impl CompositeActuator {
    /// Create a new composite from the given sub-actuators.
    pub fn new(name: impl Into<String>, actuators: Vec<Arc<dyn Actuator>>) -> Self {
        Self {
            name: name.into(),
            actuators,
        }
    }

    /// Add a sub-actuator to the composite.
    pub fn push(&mut self, actuator: Arc<dyn Actuator>) {
        self.actuators.push(actuator);
    }

    /// Number of registered sub-actuators.
    pub fn len(&self) -> usize {
        self.actuators.len()
    }

    /// Whether the composite has zero sub-actuators.
    pub fn is_empty(&self) -> bool {
        self.actuators.is_empty()
    }
}

impl Actuator for CompositeActuator {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        action: &ActionData,
    ) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send + '_>> {
        let action = action.clone();
        let actuators = self.actuators.clone();
        Box::pin(async move {
            let mut all_success = true;
            for actuator in &actuators {
                let result = actuator.execute(&action).await?;
                if !result.success {
                    all_success = false;
                }
            }
            if all_success {
                Ok(ActionResult::success(action.action_type, None, 0.0))
            } else {
                Ok(ActionResult::failure(
                    action.action_type,
                    "one or more sub-actuators failed",
                    0.0,
                ))
            }
        })
    }
}

// =============================================================================
// ThrottledActuator
// =============================================================================

/// An actuator wrapper that rate-limits execution.
///
/// Maps to Python's `ThrottledActuator` in `response/actuators.py`.
/// Prevents excessive action execution that could itself cause system instability.
/// Enforces two rate limits simultaneously:
/// - Minimum interval between consecutive executions
/// - Maximum executions per rolling 60-second window
pub struct ThrottledActuator {
    name: String,
    wrapped: Arc<dyn Actuator>,
    min_interval: std::time::Duration,
    max_per_minute: u32,
    state: Arc<RwLock<ThrottleState>>,
}

struct ThrottleState {
    last_execution: Option<tokio::time::Instant>,
    minute_start: tokio::time::Instant,
    minute_count: u32,
}

impl ThrottledActuator {
    /// Wrap an actuator with rate limiting.
    pub fn new(
        wrapped: Arc<dyn Actuator>,
        min_interval: std::time::Duration,
        max_per_minute: u32,
    ) -> Self {
        let name = format!("throttled_{}", wrapped.name());
        Self {
            name,
            wrapped,
            min_interval,
            max_per_minute,
            state: Arc::new(RwLock::new(ThrottleState {
                last_execution: None,
                minute_start: tokio::time::Instant::now(),
                minute_count: 0,
            })),
        }
    }
}

impl Actuator for ThrottledActuator {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        action: &ActionData,
    ) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send + '_>> {
        let action = action.clone();
        let wrapped = Arc::clone(&self.wrapped);
        let state = Arc::clone(&self.state);
        let min_interval = self.min_interval;
        let max_per_minute = self.max_per_minute;
        Box::pin(async move {
            let now = tokio::time::Instant::now();
            let mut s = state.write().await;

            if let Some(last) = s.last_execution {
                let elapsed = now.saturating_duration_since(last);
                if elapsed < min_interval {
                    return Ok(ActionResult::failure(
                        action.action_type,
                        format!(
                            "throttled: {:.1}s < {:.1}s minimum interval",
                            elapsed.as_secs_f64(),
                            min_interval.as_secs_f64()
                        ),
                        0.0,
                    ));
                }
            }

            if now.saturating_duration_since(s.minute_start).as_secs() > 60 {
                s.minute_count = 0;
                s.minute_start = now;
            }

            if s.minute_count >= max_per_minute {
                return Ok(ActionResult::failure(
                    action.action_type,
                    format!(
                        "rate limit: {}/{} per minute",
                        s.minute_count, max_per_minute
                    ),
                    0.0,
                ));
            }

            s.last_execution = Some(now);
            s.minute_count += 1;
            drop(s);

            wrapped.execute(&action).await
        })
    }
}

// =============================================================================
// SimulatedThreatSource
// =============================================================================

/// Internal state for [`SimulatedThreatSource`].
struct SimulatedThreatInner {
    attacking: bool,
    intensity: f64,
}

/// A test-only sensor that simulates an external threat.
///
/// Can be controlled programmatically to start and stop attacks,
/// making it useful for integration tests and demos.
pub struct SimulatedThreatSource {
    name: String,
    inner: Arc<RwLock<SimulatedThreatInner>>,
}

impl SimulatedThreatSource {
    /// Create a new idle `SimulatedThreatSource`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            inner: Arc::new(RwLock::new(SimulatedThreatInner {
                attacking: false,
                intensity: 0.0,
            })),
        }
    }

    /// Begin an attack at the given intensity (0–10 scale).
    pub async fn start_attack(&self, intensity: f64) {
        let mut inner = self.inner.write().await;
        inner.attacking = true;
        inner.intensity = intensity;
    }

    /// End the attack and return intensity to zero.
    pub async fn stop_attack(&self) {
        let mut inner = self.inner.write().await;
        inner.attacking = false;
        inner.intensity = 0.0;
    }

    /// Return `true` if an attack is currently active.
    pub async fn is_attacking(&self) -> bool {
        self.inner.read().await.attacking
    }
}

impl Sensor for SimulatedThreatSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::ExternalThreat
    }

    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Option<SensorReading>>> + Send + '_>> {
        let inner = Arc::clone(&self.inner);
        let name = self.name.clone();
        Box::pin(async move {
            let state = inner.read().await;
            if state.attacking {
                Ok(Some(SensorReading::anomalous(
                    state.intensity,
                    name,
                    SensorType::ExternalThreat,
                    (state.intensity / 10.0).min(1.0),
                    0.95,
                )))
            } else {
                Ok(Some(SensorReading::normal(
                    0.0,
                    name,
                    SensorType::ExternalThreat,
                )))
            }
        })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn callback_sensor_produces_reading() {
        let sensor = CallbackSensor::new("test", SensorType::Environmental, || {
            Box::pin(async {
                Ok(Some(SensorReading::normal(
                    42.0,
                    "test",
                    SensorType::Environmental,
                )))
            })
        });
        assert_eq!(sensor.name(), "test");
        assert_eq!(sensor.sensor_type(), SensorType::Environmental);
        let reading = sensor.read().await.unwrap().unwrap();
        assert_eq!(reading.value, 42.0);
    }

    #[tokio::test]
    async fn logging_actuator_succeeds() {
        let actuator = LoggingActuator::new("test-log");
        let action = ActionData::new(ActionType::Dampen, 50.0, "test reason");
        let result = actuator.execute(&action).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn simulated_threat_starts_and_stops() {
        let source = SimulatedThreatSource::new("threat");
        assert!(!source.is_attacking().await);

        let reading = source.read().await.unwrap().unwrap();
        assert!(!reading.is_anomalous);

        source.start_attack(7.5).await;
        assert!(source.is_attacking().await);
        let reading = source.read().await.unwrap().unwrap();
        assert!(reading.is_anomalous);
        assert_eq!(reading.value, 7.5);

        source.stop_attack().await;
        assert!(!source.is_attacking().await);
        let reading = source.read().await.unwrap().unwrap();
        assert!(!reading.is_anomalous);
    }

    #[tokio::test]
    async fn simulated_threat_severity_clamped_to_one() {
        let source = SimulatedThreatSource::new("extreme");
        // intensity = 15.0 → severity = 15.0/10.0 = 1.5 → clamped to 1.0
        source.start_attack(15.0).await;
        let reading = source.read().await.unwrap().unwrap();
        assert!(reading.is_anomalous);
        assert_eq!(reading.anomaly_severity, 1.0);
    }

    #[tokio::test]
    async fn callback_sensor_can_return_none() {
        let sensor = CallbackSensor::new("absent", SensorType::Custom, || {
            Box::pin(async { Ok(None) })
        });
        let result = sensor.read().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn callback_actuator_routes_to_handler() {
        let actuator = CallbackActuator::new("test-cb", |action: ActionData| {
            Box::pin(async move {
                Ok(ActionResult::success(
                    action.action_type,
                    Some(action.target_response_level * 2.0),
                    1.0,
                ))
            })
        });
        assert_eq!(actuator.name(), "test-cb");
        let action = ActionData::new(ActionType::Dampen, 10.0, "test");
        let result = actuator.execute(&action).await.unwrap();
        assert!(result.success);
        assert_eq!(result.actual_value, Some(20.0));
    }

    #[tokio::test]
    async fn composite_actuator_all_succeed() {
        let a1 = Arc::new(LoggingActuator::new("a1")) as Arc<dyn Actuator>;
        let a2 = Arc::new(LoggingActuator::new("a2")) as Arc<dyn Actuator>;
        let composite = CompositeActuator::new("comp", vec![a1, a2]);
        assert_eq!(composite.len(), 2);
        assert!(!composite.is_empty());
        let action = ActionData::new(ActionType::Dampen, 5.0, "fan-out");
        let result = composite.execute(&action).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn composite_actuator_one_failure_fails_all() {
        struct FailActuator;
        impl Actuator for FailActuator {
            fn name(&self) -> &str {
                "fail"
            }
            fn execute(
                &self,
                action: &ActionData,
            ) -> Pin<Box<dyn Future<Output = Result<ActionResult>> + Send + '_>> {
                let at = action.action_type;
                Box::pin(async move { Ok(ActionResult::failure(at, "nope", 0.0)) })
            }
        }
        let ok = Arc::new(LoggingActuator::new("ok")) as Arc<dyn Actuator>;
        let bad = Arc::new(FailActuator) as Arc<dyn Actuator>;
        let composite = CompositeActuator::new("mixed", vec![ok, bad]);
        let action = ActionData::new(ActionType::Dampen, 1.0, "test");
        let result = composite.execute(&action).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn composite_actuator_empty_succeeds() {
        let composite = CompositeActuator::new("empty", vec![]);
        assert!(composite.is_empty());
        let action = ActionData::new(ActionType::Idle, 0.0, "test");
        let result = composite.execute(&action).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test(start_paused = true)]
    async fn throttled_actuator_enforces_min_interval() {
        let inner = Arc::new(LoggingActuator::new("inner")) as Arc<dyn Actuator>;
        let throttled = ThrottledActuator::new(inner, std::time::Duration::from_secs(5), 100);
        let action = ActionData::new(ActionType::Dampen, 1.0, "test");

        let r1 = throttled.execute(&action).await.unwrap();
        assert!(r1.success, "first execution should pass");

        // Second execution within interval should be throttled.
        let r2 = throttled.execute(&action).await.unwrap();
        assert!(!r2.success);
        assert!(r2.error_message.unwrap().contains("throttled"));

        // After interval elapses, execution should succeed again.
        tokio::time::advance(std::time::Duration::from_secs(6)).await;
        let r3 = throttled.execute(&action).await.unwrap();
        assert!(r3.success);
    }

    #[tokio::test(start_paused = true)]
    async fn throttled_actuator_enforces_rate_limit() {
        let inner = Arc::new(LoggingActuator::new("inner")) as Arc<dyn Actuator>;
        let throttled = ThrottledActuator::new(inner, std::time::Duration::from_millis(1), 3);
        let action = ActionData::new(ActionType::Dampen, 1.0, "test");

        for _ in 0..3 {
            tokio::time::advance(std::time::Duration::from_millis(10)).await;
            let r = throttled.execute(&action).await.unwrap();
            assert!(r.success);
        }

        tokio::time::advance(std::time::Duration::from_millis(10)).await;
        let r = throttled.execute(&action).await.unwrap();
        assert!(!r.success);
        assert!(r.error_message.unwrap().contains("rate limit"));
    }
}
