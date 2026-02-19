//! Bounded message queue for buffering during reload.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Sequence (σ) | FIFO message ordering |
//! | T1: State (ς) | Queue contents + capacity |
//! | T1: Void (∅) | Empty queue state |

use std::collections::VecDeque;

use crate::error::{ProxyError, Result};

/// Tier: T2-P — Bounded FIFO message queue.
#[derive(Debug)]
pub struct MessageQueue {
    messages: VecDeque<String>,
    capacity: usize,
}

impl MessageQueue {
    /// Create a queue with the given max capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(capacity.min(1024)),
            capacity,
        }
    }

    /// Enqueue a message. Returns error if full.
    pub fn push(&mut self, message: String) -> Result<()> {
        if self.messages.len() >= self.capacity {
            return Err(ProxyError::QueueFull {
                capacity: self.capacity,
                pending: self.messages.len(),
            });
        }
        self.messages.push_back(message);
        Ok(())
    }

    /// Drain all messages in FIFO order.
    pub fn drain(&mut self) -> Vec<String> {
        self.messages.drain(..).collect()
    }

    /// Number of queued messages.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Maximum capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all messages.
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_drain() {
        let mut q = MessageQueue::new(10);
        assert!(q.is_empty());

        assert!(q.push("msg1".into()).is_ok());
        assert!(q.push("msg2".into()).is_ok());
        assert_eq!(q.len(), 2);

        let msgs = q.drain();
        assert_eq!(msgs, vec!["msg1", "msg2"]);
        assert!(q.is_empty());
    }

    #[test]
    fn queue_full_error() {
        let mut q = MessageQueue::new(2);
        assert!(q.push("a".into()).is_ok());
        assert!(q.push("b".into()).is_ok());

        match q.push("c".into()) {
            Err(ProxyError::QueueFull { capacity, pending }) => {
                assert_eq!(capacity, 2);
                assert_eq!(pending, 2);
            }
            other => panic!("expected QueueFull, got: {other:?}"),
        }
    }

    #[test]
    fn clear_empties_queue() {
        let mut q = MessageQueue::new(10);
        assert!(q.push("x".into()).is_ok());
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn capacity_reports_max() {
        let q = MessageQueue::new(42);
        assert_eq!(q.capacity(), 42);
    }
}
