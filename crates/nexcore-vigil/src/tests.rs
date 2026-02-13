#[cfg(test)]
mod tests {
    use crate::events::EventBus;
    use crate::models::{Event, Urgency};

    #[tokio::test]
    async fn test_event_bus_priority() {
        let bus = EventBus::new(10);

        bus.emit(Event {
            priority: Urgency::Low,
            source: "test".into(),
            ..Event::default()
        })
        .await;
        bus.emit(Event {
            priority: Urgency::Critical,
            source: "test".into(),
            ..Event::default()
        })
        .await;

        let first = bus.consume().await;
        assert_eq!(first.priority, Urgency::Critical);
    }

    #[tokio::test]
    async fn test_event_bus_capacity() {
        let bus = EventBus::new(1);
        bus.emit(Event::default()).await;
        bus.emit(Event::default()).await; // Should be dropped

        assert_eq!(bus.pending_count(), 1);
    }
}
