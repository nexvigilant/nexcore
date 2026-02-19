use crate::models::Event;
use flume::{Receiver, Sender, bounded};
use tracing::{info, warn};

/// v2.0 Titan Event Bus: Multi-priority lock-free event routing using flume.
#[derive(Clone)]
pub struct EventBus {
    critical_tx: Sender<Event>,
    normal_tx: Sender<Event>,
    critical_rx: Receiver<Event>,
    normal_rx: Receiver<Event>,
    #[allow(dead_code)] // Retained for future capacity monitoring
    max_size: usize,
}

impl EventBus {
    pub fn new(max_size: usize) -> Self {
        let (ctx_tx, ctx_rx) = bounded(max_size / 4);
        let (norm_tx, norm_rx) = bounded(max_size);

        Self {
            critical_tx: ctx_tx,
            normal_tx: norm_tx,
            critical_rx: ctx_rx,
            normal_rx: norm_rx,
            max_size,
        }
    }

    pub async fn emit(&self, event: Event) {
        use crate::models::Urgency;

        let tx = if event.priority >= Urgency::High {
            &self.critical_tx
        } else {
            &self.normal_tx
        };

        if tx.try_send(event.clone()).is_err() {
            warn!(event_id = ?event.id, "event_bus_backpressure_dropping_event");
            return;
        }

        info!(
            event_id = ?event.id,
            priority = ?event.priority,
            "event_emitted_v2"
        );
    }

    pub async fn consume(&self) -> Event {
        // Prefer critical events, then normal
        if let Ok(event) = self.critical_rx.try_recv() {
            return event;
        }
        if let Ok(event) = self.normal_rx.try_recv() {
            return event;
        }
        // If both empty, wait for either (biased towards critical)
        tokio::select! {
            Ok(event) = self.critical_rx.recv_async() => event,
            Ok(event) = self.normal_rx.recv_async() => event,
        }
    }

    pub fn pending_count(&self) -> usize {
        self.critical_rx.len() + self.normal_rx.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000)
    }
}
