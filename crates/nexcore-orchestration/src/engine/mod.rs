//! Parallel Task Execution Engine — T1 primitive: concurrency.
//!
//! The JARVIS engine. Runs named subsystems simultaneously with
//! priority queuing and graceful degradation under load.
//!
//! Design:
//! - Each `Subsystem` is a named concurrent work stream (vitals, threats, intel, conversation)
//! - `TaskEngine` coordinates all subsystems through a shared `LoadMonitor`
//! - Under load, low-priority tasks are shed before high-priority ones
//! - No noticeable latency degradation for Critical tasks even at max load
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                  TaskEngine                      │
//! │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
//! │  │ vitals   │ │ threats  │ │  intel   │  ...    │
//! │  │ Critical │ │  High    │ │ Normal   │         │
//! │  └────┬─────┘ └────┬─────┘ └────┬─────┘        │
//! │       │             │            │               │
//! │       ▼             ▼            ▼               │
//! │  ┌─────────────────────────────────────┐        │
//! │  │   BoundedPriorityQueue (shared)      │        │
//! │  └──────────────┬──────────────────────┘        │
//! │                 │                                │
//! │                 ▼                                │
//! │  ┌─────────────────────────────────────┐        │
//! │  │   AgentSupervisor (semaphore pool)   │        │
//! │  └──────────────┬──────────────────────┘        │
//! │                 │                                │
//! │  LoadMonitor ◄──┘ (evaluates every submit)      │
//! └─────────────────────────────────────────────────┘
//! ```

pub mod load;
pub mod metrics;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, info, warn};

use crate::agent::AgentTask;
use crate::agent::registry::AgentRegistry;
use crate::agent::supervisor::{AgentSupervisor, SupervisorConfig};
use crate::error::{OrcError, OrcResult};
use crate::queue::priority::BoundedPriorityQueue;
use crate::types::{AgentId, Priority, TaskGroupId};

use load::{DegradationLevel, LoadMonitor, LoadThresholds};
use metrics::{EngineSnapshot, LatencyTracker, SubsystemSnapshot, TaskTimer};

/// Configuration for the parallel task engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Supervisor configuration (concurrency, timeouts).
    pub supervisor: SupervisorConfig,
    /// Load thresholds for degradation.
    pub load_thresholds: LoadThresholds,
    /// Global queue capacity.
    pub queue_capacity: usize,
    /// Latency tracker window size.
    pub latency_window: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            supervisor: SupervisorConfig::default(),
            load_thresholds: LoadThresholds::default(),
            queue_capacity: 256,
            latency_window: 1000,
        }
    }
}

/// A named subsystem — a concurrent work stream with its own counters.
///
/// Subsystems are logical groupings (not separate executors).
/// All subsystems share the same priority queue and supervisor pool,
/// but each tracks its own metrics independently.
#[derive(Debug)]
pub struct Subsystem {
    /// Subsystem name (e.g., "vitals", "threats", "intel").
    name: String,
    /// Default priority for tasks in this subsystem.
    default_priority: Priority,
    /// Active task count.
    active: AtomicU64,
    /// Completed task count.
    completed: AtomicU64,
    /// Failed task count.
    failed: AtomicU64,
    /// Whether this subsystem is currently degraded (tasks being shed).
    degraded: std::sync::atomic::AtomicBool,
}

impl Subsystem {
    /// Create a new subsystem.
    #[must_use]
    pub fn new(name: impl Into<String>, default_priority: Priority) -> Self {
        Self {
            name: name.into(),
            default_priority,
            active: AtomicU64::new(0),
            completed: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            degraded: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Subsystem name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Default priority for this subsystem.
    #[must_use]
    pub fn default_priority(&self) -> Priority {
        self.default_priority
    }

    /// Current active task count.
    #[must_use]
    pub fn active_count(&self) -> u64 {
        self.active.load(Ordering::Relaxed)
    }

    /// Snapshot for reporting.
    #[must_use]
    pub fn snapshot(&self, queued: usize) -> SubsystemSnapshot {
        SubsystemSnapshot {
            name: self.name.clone(),
            active: self.active.load(Ordering::Relaxed) as usize,
            queued,
            completed: self.completed.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            degraded: self.degraded.load(Ordering::Relaxed),
        }
    }
}

/// Wrapper to carry subsystem info through the queue.
struct EngineWorkItem {
    subsystem_name: String,
    agent_id: AgentId,
    timer: TaskTimer,
}

/// The JARVIS parallel task execution engine.
///
/// Coordinates named subsystems, shared priority queue, load monitoring,
/// and graceful degradation. All subsystems run simultaneously through
/// a shared thread pool with priority-based scheduling.
pub struct TaskEngine {
    supervisor: AgentSupervisor,
    registry: Arc<AgentRegistry>,
    load_monitor: Arc<LoadMonitor>,
    latency: Arc<LatencyTracker>,
    subsystems: TokioMutex<HashMap<String, Arc<Subsystem>>>,
    queue_capacity: usize,
    // Global counters
    total_submitted: AtomicU64,
    total_completed: AtomicU64,
    total_failed: AtomicU64,
}

impl TaskEngine {
    /// Create a new engine with the given configuration.
    #[must_use]
    pub fn new(config: EngineConfig) -> Self {
        let registry = Arc::new(AgentRegistry::new());
        let supervisor = AgentSupervisor::new(registry.clone(), config.supervisor);
        let load_monitor = Arc::new(LoadMonitor::new(config.load_thresholds));
        let latency = Arc::new(LatencyTracker::new(config.latency_window));

        Self {
            supervisor,
            registry,
            load_monitor,
            latency,
            subsystems: TokioMutex::new(HashMap::new()),
            queue_capacity: config.queue_capacity,
            total_submitted: AtomicU64::new(0),
            total_completed: AtomicU64::new(0),
            total_failed: AtomicU64::new(0),
        }
    }

    /// Create an engine with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(EngineConfig::default())
    }

    /// Register a named subsystem.
    pub async fn register_subsystem(&self, subsystem: Subsystem) {
        let name = subsystem.name.clone();
        self.subsystems
            .lock()
            .await
            .insert(name.clone(), Arc::new(subsystem));
        info!(subsystem = %name, "subsystem registered");
    }

    /// Submit a task to a named subsystem.
    ///
    /// Returns `Ok(AgentId)` if accepted, or sheds the task if the
    /// current degradation level doesn't accept the task's priority.
    pub async fn submit<T: AgentTask>(
        &self,
        subsystem_name: &str,
        task: T,
        priority_override: Option<Priority>,
    ) -> OrcResult<AgentId>
    where
        T::Output: Serialize,
    {
        let subsystems = self.subsystems.lock().await;
        let subsystem = subsystems
            .get(subsystem_name)
            .ok_or_else(|| OrcError::ExecutionFailed {
                id: AgentId::new(),
                reason: format!("subsystem not found: {subsystem_name}"),
            })?
            .clone();
        drop(subsystems);

        let priority = priority_override.unwrap_or(subsystem.default_priority);

        // Evaluate load and check for shedding
        let active = self.active_task_count().await;
        let level =
            self.load_monitor
                .evaluate(active, self.queue_capacity, self.latency.percentile(0.95));

        if level.should_shed(priority) {
            self.load_monitor.record_shed();
            subsystem.degraded.store(true, Ordering::Relaxed);
            warn!(
                subsystem = %subsystem_name,
                priority = %priority,
                level = %level,
                "task shed under load"
            );
            return Err(OrcError::ExecutionFailed {
                id: AgentId::new(),
                reason: format!(
                    "task shed: priority {priority} below threshold at degradation level {level}"
                ),
            });
        }

        // Clear degraded flag if we're accepting tasks again
        subsystem.degraded.store(false, Ordering::Relaxed);

        // Submit to supervisor
        self.total_submitted.fetch_add(1, Ordering::Relaxed);
        subsystem.active.fetch_add(1, Ordering::Relaxed);

        let timer_start = std::time::Instant::now();
        let sub_name = subsystem_name.to_string();
        let latency_ref = self.latency.clone();
        let sub_ref = subsystem.clone();
        let completed_ref = &self.total_completed;
        let failed_ref = &self.total_failed;

        let id = self.supervisor.spawn(task, None).await?;

        // Spawn a background monitor for this task's lifecycle
        let registry = self.registry.clone();
        let agent_id = id.clone();
        let total_completed = Arc::new(AtomicU64::new(0));
        let total_failed_arc = Arc::new(AtomicU64::new(0));

        // Record completion asynchronously
        // We increment our local counters when the task finishes
        let tc = total_completed.clone();
        let tf = total_failed_arc.clone();
        tokio::spawn(async move {
            loop {
                if let Some(record) = registry.get(&agent_id) {
                    if record.state.is_terminal() {
                        let elapsed = timer_start.elapsed();
                        latency_ref.record(elapsed);
                        sub_ref.active.fetch_sub(1, Ordering::Relaxed);
                        if record.state == crate::agent::AgentState::Done {
                            sub_ref.completed.fetch_add(1, Ordering::Relaxed);
                            tc.fetch_add(1, Ordering::Relaxed);
                        } else {
                            sub_ref.failed.fetch_add(1, Ordering::Relaxed);
                            tf.fetch_add(1, Ordering::Relaxed);
                        }
                        debug!(
                            subsystem = %sub_name,
                            agent = %agent_id,
                            elapsed_ms = elapsed.as_millis() as u64,
                            "task completed"
                        );
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
        });

        Ok(id)
    }

    /// Submit a group of tasks to a subsystem. Returns group ID + agent IDs.
    pub async fn submit_group<T: AgentTask>(
        &self,
        subsystem_name: &str,
        tasks: Vec<T>,
        priority_override: Option<Priority>,
    ) -> OrcResult<(TaskGroupId, Vec<AgentId>)>
    where
        T::Output: Serialize,
    {
        let group_id = TaskGroupId::new();
        let mut ids = Vec::with_capacity(tasks.len());

        for task in tasks {
            let id = self.submit(subsystem_name, task, priority_override).await?;
            ids.push(id);
        }

        Ok((group_id, ids))
    }

    /// Current degradation level.
    #[must_use]
    pub fn degradation_level(&self) -> DegradationLevel {
        self.load_monitor.level()
    }

    /// Total active tasks across all subsystems.
    async fn active_task_count(&self) -> usize {
        let subsystems = self.subsystems.lock().await;
        subsystems
            .values()
            .map(|s| s.active.load(Ordering::Relaxed) as usize)
            .sum()
    }

    /// Snapshot of engine-wide metrics.
    pub async fn snapshot(&self) -> EngineSnapshot {
        let subsystems = self.subsystems.lock().await;
        let subsystem_snapshots: Vec<SubsystemSnapshot> = subsystems
            .values()
            .map(|s| s.snapshot(0)) // queue depth per-subsystem not tracked separately
            .collect();

        let active: usize = subsystem_snapshots.iter().map(|s| s.active).sum();
        let total_completed: u64 = subsystem_snapshots.iter().map(|s| s.completed).sum();
        let total_failed: u64 = subsystem_snapshots.iter().map(|s| s.failed).sum();

        EngineSnapshot {
            total_submitted: self.total_submitted.load(Ordering::Relaxed),
            total_completed,
            total_failed,
            total_shed: self.load_monitor.total_shed(),
            active_tasks: active,
            queue_depth: active, // approximation: active ≈ queued + executing
            mean_latency_us: self.latency.mean().map(|d| d.as_micros() as u64),
            p95_latency_us: self.latency.percentile(0.95).map(|d| d.as_micros() as u64),
            p99_latency_us: self.latency.percentile(0.99).map(|d| d.as_micros() as u64),
            degradation_level: self.load_monitor.level().to_string(),
            subsystems: subsystem_snapshots,
        }
    }

    /// Access the underlying supervisor for direct operations.
    #[must_use]
    pub fn supervisor(&self) -> &AgentSupervisor {
        &self.supervisor
    }

    /// Access the load monitor.
    #[must_use]
    pub fn load_monitor(&self) -> &LoadMonitor {
        &self.load_monitor
    }

    /// Access the latency tracker.
    #[must_use]
    pub fn latency_tracker(&self) -> &LatencyTracker {
        &self.latency
    }

    /// List registered subsystem names.
    pub async fn subsystem_names(&self) -> Vec<String> {
        self.subsystems.lock().await.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::context::IsolatedContext;

    /// Fast echo task for engine tests.
    struct QuickTask {
        label: String,
    }

    #[async_trait::async_trait]
    impl AgentTask for QuickTask {
        type Output = serde_json::Value;

        async fn execute(&self, _ctx: &IsolatedContext) -> OrcResult<Self::Output> {
            Ok(serde_json::json!({ "label": self.label }))
        }

        fn name(&self) -> &str {
            "quick-task"
        }
    }

    fn test_config() -> EngineConfig {
        EngineConfig {
            supervisor: SupervisorConfig {
                max_concurrency: 8,
                agent_timeout: std::time::Duration::from_secs(5),
                context_base_path: std::env::temp_dir().join("nexcore-engine-test"),
                cleanup_contexts: true,
            },
            load_thresholds: LoadThresholds {
                hold_duration: std::time::Duration::ZERO,
                ..LoadThresholds::default()
            },
            queue_capacity: 100,
            latency_window: 100,
        }
    }

    #[tokio::test]
    async fn engine_register_and_submit() {
        let engine = TaskEngine::new(test_config());
        engine
            .register_subsystem(Subsystem::new("vitals", Priority::Critical))
            .await;

        let id = engine
            .submit(
                "vitals",
                QuickTask {
                    label: "heartbeat".into(),
                },
                None,
            )
            .await;
        assert!(id.is_ok());

        // Wait for completion
        if let Ok(id) = id {
            let record = engine.supervisor().wait_for(&id).await;
            assert!(record.is_ok());
        }
    }

    #[tokio::test]
    async fn engine_parallel_subsystems() {
        let engine = TaskEngine::new(test_config());
        engine
            .register_subsystem(Subsystem::new("vitals", Priority::Critical))
            .await;
        engine
            .register_subsystem(Subsystem::new("threats", Priority::High))
            .await;
        engine
            .register_subsystem(Subsystem::new("intel", Priority::Normal))
            .await;

        // Submit to all three simultaneously
        let v = engine
            .submit(
                "vitals",
                QuickTask {
                    label: "pulse".into(),
                },
                None,
            )
            .await;
        let t = engine
            .submit(
                "threats",
                QuickTask {
                    label: "scan".into(),
                },
                None,
            )
            .await;
        let i = engine
            .submit(
                "intel",
                QuickTask {
                    label: "brief".into(),
                },
                None,
            )
            .await;

        assert!(v.is_ok());
        assert!(t.is_ok());
        assert!(i.is_ok());

        // Wait for all
        for id in [v, t, i].into_iter().flatten() {
            let record = engine.supervisor().wait_for(&id).await;
            assert!(record.is_ok());
            if let Ok(r) = record {
                assert_eq!(r.state, crate::agent::AgentState::Done);
            }
        }

        // Check subsystem names
        let names = engine.subsystem_names().await;
        assert_eq!(names.len(), 3);
    }

    #[tokio::test]
    async fn engine_unknown_subsystem_errors() {
        let engine = TaskEngine::new(test_config());
        let result = engine
            .submit(
                "nonexistent",
                QuickTask {
                    label: "orphan".into(),
                },
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn engine_snapshot_reports_state() {
        let engine = TaskEngine::new(test_config());
        engine
            .register_subsystem(Subsystem::new("vitals", Priority::Critical))
            .await;

        let snap = engine.snapshot().await;
        assert_eq!(snap.total_submitted, 0);
        assert_eq!(snap.subsystems.len(), 1);
        assert_eq!(snap.subsystems[0].name, "vitals");
        assert_eq!(snap.degradation_level, "normal");
    }

    #[tokio::test]
    async fn engine_sheds_low_priority_under_load() {
        let config = EngineConfig {
            supervisor: SupervisorConfig {
                max_concurrency: 8,
                agent_timeout: std::time::Duration::from_secs(5),
                context_base_path: std::env::temp_dir().join("nexcore-engine-shed-test"),
                cleanup_contexts: true,
            },
            load_thresholds: LoadThresholds {
                // Very aggressive thresholds for testing
                elevated_ratio: 0.01,
                high_ratio: 0.02,
                critical_ratio: 0.03,
                hold_duration: std::time::Duration::ZERO,
                ..LoadThresholds::default()
            },
            queue_capacity: 10,
            latency_window: 100,
        };

        let engine = TaskEngine::new(config);
        engine
            .register_subsystem(Subsystem::new("bg", Priority::Low))
            .await;
        engine
            .register_subsystem(Subsystem::new("vitals", Priority::Critical))
            .await;

        // Submit a Critical task first to create load
        let _v = engine
            .submit(
                "vitals",
                QuickTask {
                    label: "pulse".into(),
                },
                None,
            )
            .await;

        // Now try Low priority — should be shed because ratio > elevated_ratio
        let bg = engine
            .submit(
                "bg",
                QuickTask {
                    label: "cleanup".into(),
                },
                None,
            )
            .await;
        assert!(bg.is_err(), "Low priority task should be shed under load");

        // Critical should still work
        let v2 = engine
            .submit(
                "vitals",
                QuickTask {
                    label: "pulse2".into(),
                },
                None,
            )
            .await;
        assert!(v2.is_ok(), "Critical task should always be accepted");
    }

    #[tokio::test]
    async fn engine_group_submit() {
        let engine = TaskEngine::new(test_config());
        engine
            .register_subsystem(Subsystem::new("intel", Priority::Normal))
            .await;

        let tasks = vec![
            QuickTask { label: "a".into() },
            QuickTask { label: "b".into() },
            QuickTask { label: "c".into() },
        ];

        let result = engine.submit_group("intel", tasks, None).await;
        assert!(result.is_ok());
        if let Ok((_gid, ids)) = result {
            assert_eq!(ids.len(), 3);
        }
    }
}
