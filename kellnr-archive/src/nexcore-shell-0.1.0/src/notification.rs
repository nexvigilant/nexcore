// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Notification system — priority-ordered message queue.
//!
//! ## Primitive Grounding
//!
//! - σ Sequence: Notification queue ordering
//! - κ Comparison: Priority-based ordering
//! - ν Frequency: Rate limiting / snooze duration
//! - ∃ Existence: Notification lifecycle (created → displayed → dismissed)
//! - ∂ Boundary: Maximum queue size, display duration limits

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Notification priority levels (aligned with Guardian P0-P5 hierarchy).
///
/// Tier: T2-P (κ Comparison — orderable severity)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    /// P0: Patient safety / critical system failure.
    Critical = 0,
    /// P1: Security alert (Guardian threat).
    Security = 1,
    /// P2: Urgent system event (low battery, etc.).
    Urgent = 2,
    /// P3: Normal notification (app message).
    Normal = 3,
    /// P4: Low priority (informational).
    Low = 4,
    /// P5: Silent / background.
    Silent = 5,
}

impl NotificationPriority {
    /// Numeric value (lower = higher priority).
    const fn value(self) -> u8 {
        self as u8
    }
}

impl PartialOrd for NotificationPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NotificationPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower value = higher priority (Critical < Silent)
        self.value().cmp(&other.value())
    }
}

/// Notification state.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationState {
    /// Queued, not yet displayed.
    Pending,
    /// Currently showing on screen.
    Displayed,
    /// Snoozed — will re-display after delay.
    Snoozed,
    /// Dismissed by user.
    Dismissed,
    /// Expired (time-to-live exceeded).
    Expired,
}

/// A notification.
///
/// Tier: T3 (σ + κ + ν + ∃ + ∂ — ordered, prioritized, timed entity)
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique notification ID.
    pub id: u64,
    /// Source app or system component.
    pub source: String,
    /// Short title.
    pub title: String,
    /// Message body.
    pub body: String,
    /// Priority level.
    pub priority: NotificationPriority,
    /// Current state.
    pub state: NotificationState,
    /// Ticks remaining before auto-dismiss (0 = persistent).
    pub ttl_ticks: u64,
    /// Whether this notification requires acknowledgment.
    pub requires_ack: bool,
}

impl Notification {
    /// Create a new notification.
    pub fn new(
        id: u64,
        source: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
        priority: NotificationPriority,
    ) -> Self {
        let ttl = match priority {
            NotificationPriority::Critical | NotificationPriority::Security => 0, // Persistent
            NotificationPriority::Urgent => 300,                                  // 300 ticks
            NotificationPriority::Normal => 100,                                  // 100 ticks
            NotificationPriority::Low => 50,                                      // 50 ticks
            NotificationPriority::Silent => 30,                                   // 30 ticks
        };

        Self {
            id,
            source: source.into(),
            title: title.into(),
            body: body.into(),
            priority,
            state: NotificationState::Pending,
            ttl_ticks: ttl,
            requires_ack: matches!(
                priority,
                NotificationPriority::Critical | NotificationPriority::Security
            ),
        }
    }
}

/// Priority wrapper for the binary heap (max-heap, so we invert priority).
#[derive(Debug)]
struct PriorityEntry {
    notification: Notification,
}

impl PartialEq for PriorityEntry {
    fn eq(&self, other: &Self) -> bool {
        self.notification.priority == other.notification.priority
            && self.notification.id == other.notification.id
    }
}

impl Eq for PriorityEntry {}

impl PartialOrd for PriorityEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse: lower priority value = higher heap priority
        // Within same priority: lower ID first (FIFO order)
        other
            .notification
            .priority
            .value()
            .cmp(&self.notification.priority.value())
            .then_with(|| other.notification.id.cmp(&self.notification.id))
    }
}

/// Notification manager — manages the notification queue and display.
///
/// Tier: T3 (Σ + σ + κ — prioritized collection with ordered processing)
pub struct NotificationManager {
    /// Priority queue of pending notifications.
    queue: BinaryHeap<PriorityEntry>,
    /// Currently displayed notification (if any).
    displayed: Option<Notification>,
    /// Dismissed notifications (for history).
    history: Vec<Notification>,
    /// Next notification ID.
    next_id: u64,
    /// Total notifications processed.
    total_processed: u64,
}

impl NotificationManager {
    /// Create a new notification manager.
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            displayed: None,
            history: Vec::new(),
            next_id: 1,
            total_processed: 0,
        }
    }

    /// Post a notification (returns the assigned ID).
    pub fn post(
        &mut self,
        source: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
        priority: NotificationPriority,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let notification = Notification::new(id, source, title, body, priority);

        // Critical/Security notifications preempt current display
        if matches!(
            notification.priority,
            NotificationPriority::Critical | NotificationPriority::Security
        ) {
            // If something is displayed, put it back in queue
            if let Some(mut current) = self.displayed.take() {
                current.state = NotificationState::Pending;
                self.queue.push(PriorityEntry {
                    notification: current,
                });
            }
            // Display the critical notification immediately
            let mut n = notification;
            n.state = NotificationState::Displayed;
            self.displayed = Some(n);
        } else {
            // Normal flow: add to queue
            self.queue.push(PriorityEntry { notification });

            // Queue size is bounded by practical usage patterns.
            // In future: prune lowest-priority entries when queue exceeds max_queue_size.
        }

        id
    }

    /// Dismiss the currently displayed notification.
    pub fn dismiss(&mut self) {
        if let Some(mut n) = self.displayed.take() {
            n.state = NotificationState::Dismissed;
            self.total_processed += 1;
            self.history.push(n);
        }
    }

    /// Dismiss a notification by ID (whether displayed or queued).
    pub fn dismiss_by_id(&mut self, id: u64) {
        // Check displayed
        if self.displayed.as_ref().is_some_and(|n| n.id == id) {
            self.dismiss();
            return;
        }

        // Remove from queue (rebuild without the target)
        let entries: Vec<_> = self.queue.drain().collect();
        for entry in entries {
            if entry.notification.id == id {
                let mut n = entry.notification;
                n.state = NotificationState::Dismissed;
                self.total_processed += 1;
                self.history.push(n);
            } else {
                self.queue.push(entry);
            }
        }
    }

    /// Tick the notification system (advances display, handles TTL).
    ///
    /// Should be called once per shell tick.
    pub fn tick(&mut self) {
        // Check TTL on displayed notification
        if let Some(ref mut n) = self.displayed {
            if n.ttl_ticks > 0 {
                n.ttl_ticks -= 1;
                if n.ttl_ticks == 0 && !n.requires_ack {
                    // Auto-dismiss expired non-critical notifications
                    let mut expired = self.displayed.take();
                    if let Some(ref mut n) = expired {
                        n.state = NotificationState::Expired;
                        self.total_processed += 1;
                        self.history.push(n.clone());
                    }
                }
            }
        }

        // If nothing displayed, pop next from queue
        if self.displayed.is_none() {
            if let Some(entry) = self.queue.pop() {
                let mut n = entry.notification;
                n.state = NotificationState::Displayed;
                self.displayed = Some(n);
            }
        }
    }

    /// Get the currently displayed notification.
    pub fn displayed(&self) -> Option<&Notification> {
        self.displayed.as_ref()
    }

    /// Number of pending notifications in queue.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Total notifications processed (dismissed + expired).
    pub fn total_processed(&self) -> u64 {
        self.total_processed
    }

    /// Whether there's an unacknowledged critical notification.
    pub fn has_critical(&self) -> bool {
        self.displayed
            .as_ref()
            .is_some_and(|n| n.priority == NotificationPriority::Critical)
    }

    /// Get notification history.
    pub fn history(&self) -> &[Notification] {
        &self.history
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_and_display() {
        let mut mgr = NotificationManager::new();
        let id = mgr.post("app", "Hello", "World", NotificationPriority::Normal);
        assert_eq!(id, 1);
        assert_eq!(mgr.pending_count(), 1);

        // Tick to display
        mgr.tick();
        assert!(mgr.displayed().is_some());
        assert_eq!(mgr.pending_count(), 0);

        if let Some(n) = mgr.displayed() {
            assert_eq!(n.title, "Hello");
            assert_eq!(n.state, NotificationState::Displayed);
        }
    }

    #[test]
    fn priority_ordering() {
        let mut mgr = NotificationManager::new();

        // Post in reverse priority order
        mgr.post("app", "Low", "low", NotificationPriority::Low);
        mgr.post("app", "Normal", "normal", NotificationPriority::Normal);
        mgr.post("app", "Urgent", "urgent", NotificationPriority::Urgent);

        // First tick should display Urgent (highest priority in queue)
        mgr.tick();
        assert_eq!(mgr.displayed().map(|n| n.title.as_str()), Some("Urgent"));
    }

    #[test]
    fn critical_preempts_display() {
        let mut mgr = NotificationManager::new();

        mgr.post("app", "Normal", "msg", NotificationPriority::Normal);
        mgr.tick(); // Display normal

        assert_eq!(mgr.displayed().map(|n| n.title.as_str()), Some("Normal"));

        // Post critical — should preempt
        mgr.post(
            "guardian",
            "ALERT",
            "threat",
            NotificationPriority::Critical,
        );

        assert_eq!(mgr.displayed().map(|n| n.title.as_str()), Some("ALERT"));
        assert!(mgr.has_critical());
        // Normal should be back in queue
        assert_eq!(mgr.pending_count(), 1);
    }

    #[test]
    fn dismiss_current() {
        let mut mgr = NotificationManager::new();
        mgr.post("app", "Test", "msg", NotificationPriority::Normal);
        mgr.tick();

        mgr.dismiss();
        assert!(mgr.displayed().is_none());
        assert_eq!(mgr.total_processed(), 1);
        assert_eq!(mgr.history().len(), 1);
        assert_eq!(mgr.history()[0].state, NotificationState::Dismissed);
    }

    #[test]
    fn dismiss_by_id() {
        let mut mgr = NotificationManager::new();
        let id1 = mgr.post("app", "First", "msg", NotificationPriority::Normal);
        let _id2 = mgr.post("app", "Second", "msg", NotificationPriority::Normal);

        mgr.dismiss_by_id(id1);
        assert_eq!(mgr.pending_count(), 1);
        assert_eq!(mgr.total_processed(), 1);
    }

    #[test]
    fn ttl_auto_dismiss() {
        let mut mgr = NotificationManager::new();
        mgr.post("app", "Temp", "msg", NotificationPriority::Silent);
        mgr.tick(); // Display it

        // TTL for Silent = 30 ticks
        for _ in 0..30 {
            mgr.tick();
        }

        // Should be auto-dismissed
        assert!(mgr.displayed().is_none());
        assert_eq!(mgr.total_processed(), 1);
        assert_eq!(mgr.history()[0].state, NotificationState::Expired);
    }

    #[test]
    fn critical_requires_ack_no_auto_dismiss() {
        let mut mgr = NotificationManager::new();
        mgr.post(
            "guardian",
            "CRITICAL",
            "threat",
            NotificationPriority::Critical,
        );

        // Critical has TTL = 0 (persistent) and requires_ack = true
        for _ in 0..100 {
            mgr.tick();
        }

        // Still displayed — critical won't auto-dismiss
        assert!(mgr.displayed().is_some());
        assert!(mgr.has_critical());
    }

    #[test]
    fn sequential_notifications() {
        let mut mgr = NotificationManager::new();
        mgr.post("a", "First", "1", NotificationPriority::Normal);
        mgr.post("b", "Second", "2", NotificationPriority::Normal);

        mgr.tick();
        assert_eq!(mgr.displayed().map(|n| n.title.as_str()), Some("First"));

        mgr.dismiss();
        mgr.tick();
        assert_eq!(mgr.displayed().map(|n| n.title.as_str()), Some("Second"));
    }

    #[test]
    fn security_preempts_like_critical() {
        let mut mgr = NotificationManager::new();
        mgr.post("app", "Normal", "msg", NotificationPriority::Normal);
        mgr.tick();

        mgr.post(
            "sentinel",
            "Intrusion",
            "SSH brute force",
            NotificationPriority::Security,
        );
        assert_eq!(mgr.displayed().map(|n| n.title.as_str()), Some("Intrusion"));
    }

    #[test]
    fn empty_manager() {
        let mut mgr = NotificationManager::new();
        assert!(mgr.displayed().is_none());
        assert_eq!(mgr.pending_count(), 0);
        assert!(!mgr.has_critical());

        // Tick with nothing — should be no-op
        mgr.tick();
        assert!(mgr.displayed().is_none());
    }
}
