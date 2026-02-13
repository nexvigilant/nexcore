use criterion::{Criterion, criterion_group, criterion_main};
use nexcore_vigil::{EventBus, models::Event};
use tokio::runtime::Runtime;

fn bench_event_bus(c: &mut Criterion) {
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("runtime_init_failed: {err}");
            return;
        }
    };
    let bus = EventBus::new(1024);

    c.bench_function("emit_event", |b| {
        b.to_async(&rt).iter(|| async {
            let event = Event::default();
            bus.emit(event).await;
        })
    });
}

criterion_group!(benches, bench_event_bus);
criterion_main!(benches);
