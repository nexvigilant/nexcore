// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 6: Sequence Controller (STOS-SQ)
//!
//! **Dominant Primitive**: σ (Sequence)
//!
//! Controls transition ordering, sequencing, and execution flow.
//!
//! ## Responsibilities
//!
//! - Transition queue management
//! - Execution ordering
//! - Batched transitions
//! - Sequence validation
//!
//! ## Tier Classification
//!
//! `SequenceController` is T2-P (σ + →) — sequence, causality.

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

use super::transition_engine::TransitionId;
use crate::MachineId;

/// Execution order strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionOrder {
    /// First-in-first-out.
    #[default]
    Fifo,
    /// Last-in-first-out.
    Lifo,
    /// Priority-based (highest first).
    Priority,
}

/// A queued transition.
#[derive(Debug, Clone)]
pub struct QueuedTransition {
    /// Transition ID.
    pub transition_id: TransitionId,
    /// Priority (higher = execute first when using Priority order).
    pub priority: u32,
    /// Queue timestamp.
    pub queued_at: u64,
    /// Optional label.
    pub label: Option<String>,
}

impl QueuedTransition {
    /// Create a new queued transition.
    #[must_use]
    pub fn new(transition_id: TransitionId, queued_at: u64) -> Self {
        Self {
            transition_id,
            priority: 0,
            queued_at,
            label: None,
        }
    }

    /// With priority.
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// With label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A transition sequence (batch).
#[derive(Debug, Clone)]
pub struct TransitionSequence {
    /// Sequence ID.
    pub id: u64,
    /// Transitions in order.
    pub transitions: Vec<TransitionId>,
    /// Whether sequence is atomic (all or nothing).
    pub atomic: bool,
}

impl TransitionSequence {
    /// Create a new sequence.
    #[must_use]
    pub fn new(id: u64, transitions: Vec<TransitionId>) -> Self {
        Self {
            id,
            transitions,
            atomic: false,
        }
    }

    /// Make the sequence atomic.
    #[must_use]
    pub fn atomic(mut self) -> Self {
        self.atomic = true;
        self
    }
}

/// The sequence controller.
///
/// ## Tier: T2-P (σ + →)
///
/// Dominant primitive: σ (Sequence)
#[derive(Debug, Clone)]
pub struct SequenceController {
    /// Machine ID.
    machine_id: MachineId,
    /// Pending transitions queue.
    queue: VecDeque<QueuedTransition>,
    /// Execution order strategy.
    order: ExecutionOrder,
    /// Monotonic counter.
    counter: u64,
    /// Registered sequences.
    sequences: Vec<TransitionSequence>,
    /// Maximum queue size.
    max_queue_size: usize,
}

impl SequenceController {
    /// Create a new sequence controller.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            queue: VecDeque::new(),
            order: ExecutionOrder::Fifo,
            counter: 0,
            sequences: Vec::new(),
            max_queue_size: 1000,
        }
    }

    /// Set execution order.
    pub fn set_order(&mut self, order: ExecutionOrder) {
        self.order = order;
    }

    /// Enqueue a transition.
    pub fn enqueue(&mut self, transition_id: TransitionId) -> bool {
        if self.queue.len() >= self.max_queue_size {
            return false;
        }

        self.counter = self.counter.saturating_add(1);
        let queued = QueuedTransition::new(transition_id, self.counter);
        self.queue.push_back(queued);
        true
    }

    /// Enqueue with priority.
    pub fn enqueue_with_priority(&mut self, transition_id: TransitionId, priority: u32) -> bool {
        if self.queue.len() >= self.max_queue_size {
            return false;
        }

        self.counter = self.counter.saturating_add(1);
        let queued = QueuedTransition::new(transition_id, self.counter).with_priority(priority);
        self.queue.push_back(queued);
        true
    }

    /// Dequeue the next transition based on order.
    pub fn dequeue(&mut self) -> Option<QueuedTransition> {
        match self.order {
            ExecutionOrder::Fifo => self.queue.pop_front(),
            ExecutionOrder::Lifo => self.queue.pop_back(),
            ExecutionOrder::Priority => {
                // Find highest priority
                let idx = self
                    .queue
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, q)| q.priority)
                    .map(|(i, _)| i)?;
                self.queue.remove(idx)
            }
        }
    }

    /// Peek at the next transition without removing.
    #[must_use]
    pub fn peek(&self) -> Option<&QueuedTransition> {
        match self.order {
            ExecutionOrder::Fifo => self.queue.front(),
            ExecutionOrder::Lifo => self.queue.back(),
            ExecutionOrder::Priority => self.queue.iter().max_by_key(|q| q.priority),
        }
    }

    /// Register a sequence.
    pub fn register_sequence(&mut self, transitions: Vec<TransitionId>) -> u64 {
        self.counter = self.counter.saturating_add(1);
        let seq = TransitionSequence::new(self.counter, transitions);
        let id = seq.id;
        self.sequences.push(seq);
        id
    }

    /// Register an atomic sequence.
    pub fn register_atomic_sequence(&mut self, transitions: Vec<TransitionId>) -> u64 {
        self.counter = self.counter.saturating_add(1);
        let seq = TransitionSequence::new(self.counter, transitions).atomic();
        let id = seq.id;
        self.sequences.push(seq);
        id
    }

    /// Get a sequence by ID.
    #[must_use]
    pub fn get_sequence(&self, id: u64) -> Option<&TransitionSequence> {
        self.sequences.iter().find(|s| s.id == id)
    }

    /// Enqueue all transitions from a sequence.
    pub fn enqueue_sequence(&mut self, sequence_id: u64) -> bool {
        let transitions = self
            .sequences
            .iter()
            .find(|s| s.id == sequence_id)
            .map(|s| s.transitions.clone());

        if let Some(transitions) = transitions {
            for t in transitions {
                if !self.enqueue(t) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Current queue length.
    #[must_use]
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Whether queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clear the queue.
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Current order strategy.
    #[must_use]
    pub fn order(&self) -> ExecutionOrder {
        self.order
    }

    /// Number of registered sequences.
    #[must_use]
    pub fn sequence_count(&self) -> usize {
        self.sequences.len()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_order() {
        let mut controller = SequenceController::new(1);
        controller.set_order(ExecutionOrder::Fifo);

        controller.enqueue(0);
        controller.enqueue(1);
        controller.enqueue(2);

        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(0));
        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(1));
        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(2));
    }

    #[test]
    fn test_lifo_order() {
        let mut controller = SequenceController::new(1);
        controller.set_order(ExecutionOrder::Lifo);

        controller.enqueue(0);
        controller.enqueue(1);
        controller.enqueue(2);

        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(2));
        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(1));
        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(0));
    }

    #[test]
    fn test_priority_order() {
        let mut controller = SequenceController::new(1);
        controller.set_order(ExecutionOrder::Priority);

        controller.enqueue_with_priority(0, 1);
        controller.enqueue_with_priority(1, 10);
        controller.enqueue_with_priority(2, 5);

        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(1)); // Priority 10
        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(2)); // Priority 5
        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(0)); // Priority 1
    }

    #[test]
    fn test_sequence_registration() {
        let mut controller = SequenceController::new(1);

        let seq_id = controller.register_sequence(vec![0, 1, 2]);
        let seq = controller.get_sequence(seq_id);

        assert!(seq.is_some());
        assert_eq!(seq.map(|s| s.transitions.len()), Some(3));
    }

    #[test]
    fn test_enqueue_sequence() {
        let mut controller = SequenceController::new(1);
        controller.set_order(ExecutionOrder::Fifo);

        let seq_id = controller.register_sequence(vec![5, 6, 7]);
        assert!(controller.enqueue_sequence(seq_id));

        assert_eq!(controller.queue_len(), 3);
        assert_eq!(controller.dequeue().map(|q| q.transition_id), Some(5));
    }
}
