// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 12: Temporal Scheduler (STOS-TM)
//!
//! **Dominant Primitive**: ν (Frequency)
//!
//! Schedules state transitions based on time constraints,
//! timeouts, and periodic execution.
//!
//! ## Tier Classification
//!
//! `TemporalScheduler` is T2-P (ν + σ) — frequency, sequence.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::transition_engine::TransitionId;
use crate::MachineId;

/// A scheduled transition.
#[derive(Debug, Clone)]
pub struct ScheduledTransition {
    /// Schedule ID.
    pub id: u64,
    /// Machine ID.
    pub machine_id: MachineId,
    /// Transition to execute.
    pub transition_id: TransitionId,
    /// Scheduled time (monotonic).
    pub scheduled_time: u64,
    /// Whether this is a repeating schedule.
    pub repeating: bool,
    /// Repeat interval (if repeating).
    pub interval: u64,
    /// Times executed.
    pub execution_count: u64,
    /// Maximum executions (0 = unlimited).
    pub max_executions: u64,
}

impl ScheduledTransition {
    /// Create a one-shot schedule.
    #[must_use]
    pub fn one_shot(
        id: u64,
        machine_id: MachineId,
        transition_id: TransitionId,
        time: u64,
    ) -> Self {
        Self {
            id,
            machine_id,
            transition_id,
            scheduled_time: time,
            repeating: false,
            interval: 0,
            execution_count: 0,
            max_executions: 1,
        }
    }

    /// Create a repeating schedule.
    #[must_use]
    pub fn repeating(
        id: u64,
        machine_id: MachineId,
        transition_id: TransitionId,
        start_time: u64,
        interval: u64,
    ) -> Self {
        Self {
            id,
            machine_id,
            transition_id,
            scheduled_time: start_time,
            repeating: true,
            interval,
            execution_count: 0,
            max_executions: 0,
        }
    }

    /// With maximum executions.
    #[must_use]
    pub fn with_max_executions(mut self, max: u64) -> Self {
        self.max_executions = max;
        self
    }

    /// Whether schedule is exhausted.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.max_executions > 0 && self.execution_count >= self.max_executions
    }
}

/// A timeout configuration.
#[derive(Debug, Clone)]
pub struct Timeout {
    /// Timeout ID.
    pub id: u64,
    /// Machine ID.
    pub machine_id: MachineId,
    /// Start time.
    pub start_time: u64,
    /// Duration.
    pub duration: u64,
    /// Transition to execute on timeout.
    pub timeout_transition: TransitionId,
    /// Whether triggered.
    pub triggered: bool,
}

impl Timeout {
    /// Create a new timeout.
    #[must_use]
    pub fn new(
        id: u64,
        machine_id: MachineId,
        start_time: u64,
        duration: u64,
        timeout_transition: TransitionId,
    ) -> Self {
        Self {
            id,
            machine_id,
            start_time,
            duration,
            timeout_transition,
            triggered: false,
        }
    }

    /// Deadline time.
    #[must_use]
    pub fn deadline(&self) -> u64 {
        self.start_time.saturating_add(self.duration)
    }

    /// Whether timeout is expired at given time.
    #[must_use]
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.deadline()
    }
}

/// The temporal scheduler.
///
/// ## Tier: T2-P (ν + σ)
///
/// Dominant primitive: ν (Frequency)
#[derive(Debug, Clone)]
pub struct TemporalScheduler {
    /// Scheduled transitions.
    schedules: BTreeMap<u64, ScheduledTransition>,
    /// Active timeouts.
    timeouts: BTreeMap<u64, Timeout>,
    /// Counter for IDs.
    counter: u64,
    /// Current time (monotonic).
    current_time: u64,
}

impl TemporalScheduler {
    /// Create a new temporal scheduler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schedules: BTreeMap::new(),
            timeouts: BTreeMap::new(),
            counter: 0,
            current_time: 0,
        }
    }

    /// Set current time.
    pub fn set_time(&mut self, time: u64) {
        self.current_time = time;
    }

    /// Advance time.
    pub fn advance(&mut self, delta: u64) {
        self.current_time = self.current_time.saturating_add(delta);
    }

    /// Current time.
    #[must_use]
    pub fn time(&self) -> u64 {
        self.current_time
    }

    /// Schedule a one-shot transition.
    pub fn schedule_once(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
        delay: u64,
    ) -> u64 {
        self.counter = self.counter.saturating_add(1);
        let scheduled = ScheduledTransition::one_shot(
            self.counter,
            machine_id,
            transition_id,
            self.current_time.saturating_add(delay),
        );
        self.schedules.insert(self.counter, scheduled);
        self.counter
    }

    /// Schedule a repeating transition.
    pub fn schedule_repeating(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
        interval: u64,
    ) -> u64 {
        self.counter = self.counter.saturating_add(1);
        let scheduled = ScheduledTransition::repeating(
            self.counter,
            machine_id,
            transition_id,
            self.current_time.saturating_add(interval),
            interval,
        );
        self.schedules.insert(self.counter, scheduled);
        self.counter
    }

    /// Cancel a schedule.
    pub fn cancel_schedule(&mut self, schedule_id: u64) {
        self.schedules.remove(&schedule_id);
    }

    /// Set a timeout.
    pub fn set_timeout(
        &mut self,
        machine_id: MachineId,
        duration: u64,
        timeout_transition: TransitionId,
    ) -> u64 {
        self.counter = self.counter.saturating_add(1);
        let timeout = Timeout::new(
            self.counter,
            machine_id,
            self.current_time,
            duration,
            timeout_transition,
        );
        self.timeouts.insert(self.counter, timeout);
        self.counter
    }

    /// Cancel a timeout.
    pub fn cancel_timeout(&mut self, timeout_id: u64) {
        self.timeouts.remove(&timeout_id);
    }

    /// Get due transitions (ready to execute).
    #[must_use]
    pub fn due_transitions(&self) -> Vec<(u64, MachineId, TransitionId)> {
        self.schedules
            .iter()
            .filter(|(_, s)| s.scheduled_time <= self.current_time && !s.is_exhausted())
            .map(|(&id, s)| (id, s.machine_id, s.transition_id))
            .collect()
    }

    /// Mark a scheduled transition as executed.
    pub fn mark_executed(&mut self, schedule_id: u64) {
        if let Some(schedule) = self.schedules.get_mut(&schedule_id) {
            schedule.execution_count = schedule.execution_count.saturating_add(1);

            if schedule.repeating && !schedule.is_exhausted() {
                // Reschedule
                schedule.scheduled_time = self.current_time.saturating_add(schedule.interval);
            } else if schedule.is_exhausted() {
                // Will be cleaned up
            }
        }
    }

    /// Get expired timeouts.
    #[must_use]
    pub fn expired_timeouts(&self) -> Vec<(u64, MachineId, TransitionId)> {
        self.timeouts
            .iter()
            .filter(|(_, t)| t.is_expired(self.current_time) && !t.triggered)
            .map(|(&id, t)| (id, t.machine_id, t.timeout_transition))
            .collect()
    }

    /// Mark a timeout as triggered.
    pub fn mark_timeout_triggered(&mut self, timeout_id: u64) {
        if let Some(timeout) = self.timeouts.get_mut(&timeout_id) {
            timeout.triggered = true;
        }
    }

    /// Clean up exhausted schedules and triggered timeouts.
    pub fn cleanup(&mut self) {
        self.schedules.retain(|_, s| !s.is_exhausted());
        self.timeouts.retain(|_, t| !t.triggered);
    }

    /// Number of active schedules.
    #[must_use]
    pub fn schedule_count(&self) -> usize {
        self.schedules.len()
    }

    /// Number of active timeouts.
    #[must_use]
    pub fn timeout_count(&self) -> usize {
        self.timeouts.len()
    }
}

impl Default for TemporalScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_shot_schedule() {
        let mut scheduler = TemporalScheduler::new();

        let id = scheduler.schedule_once(1, 5, 100);

        // Not due yet
        assert!(scheduler.due_transitions().is_empty());

        // Advance to due time
        scheduler.advance(100);
        let due = scheduler.due_transitions();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0], (id, 1, 5));
    }

    #[test]
    fn test_repeating_schedule() {
        let mut scheduler = TemporalScheduler::new();

        let id = scheduler.schedule_repeating(1, 5, 50);

        scheduler.advance(50);
        assert_eq!(scheduler.due_transitions().len(), 1);

        scheduler.mark_executed(id);
        assert!(scheduler.due_transitions().is_empty()); // Rescheduled

        scheduler.advance(50);
        assert_eq!(scheduler.due_transitions().len(), 1);
    }

    #[test]
    fn test_timeout() {
        let mut scheduler = TemporalScheduler::new();

        let id = scheduler.set_timeout(1, 100, 10);

        // Not expired yet
        assert!(scheduler.expired_timeouts().is_empty());

        scheduler.advance(100);
        let expired = scheduler.expired_timeouts();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0], (id, 1, 10));
    }

    #[test]
    fn test_cancel_schedule() {
        let mut scheduler = TemporalScheduler::new();

        let id = scheduler.schedule_once(1, 5, 100);
        scheduler.cancel_schedule(id);

        scheduler.advance(100);
        assert!(scheduler.due_transitions().is_empty());
    }

    #[test]
    fn test_max_executions() {
        let mut scheduler = TemporalScheduler::new();

        scheduler.counter = 0;
        let scheduled = ScheduledTransition::repeating(1, 1, 5, 0, 10).with_max_executions(2);
        scheduler.schedules.insert(1, scheduled);

        // First execution
        scheduler.mark_executed(1);
        assert!(
            !scheduler
                .schedules
                .get(&1)
                .map_or(true, |s| s.is_exhausted())
        );

        // Second execution
        scheduler.mark_executed(1);
        assert!(
            scheduler
                .schedules
                .get(&1)
                .map_or(false, |s| s.is_exhausted())
        );
    }
}
