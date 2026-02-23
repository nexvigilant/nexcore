use nexcore_vigil::{
    AuthorityConfig, ContextAssembler, DecisionEngine, EventBus, Friday, MemoryLayer,
    llm::mock::MockLLMClient,
    models::{Event, Urgency},
    projects::ProjectRegistry,
};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_full_orchestration_flow() -> nexcore_error::Result<()> {
    let bus = EventBus::new(10);
    let authority = AuthorityConfig {
        autonomous_allowed: vec!["test".into()],
        forbidden: vec![],
        requires_confirmation: vec![],
    };
    let decision = DecisionEngine::new(authority);
    let memory = MemoryLayer::new(
        PathBuf::from("/tmp/ksb"),
        PathBuf::from("/tmp/data"),
        "http://localhost:6333",
    )
    .await?;

    let registry = Arc::new(ProjectRegistry::new(PathBuf::from("/tmp/data")));

    let context = Arc::new(ContextAssembler::new(
        PathBuf::from("/tmp/ksb"),
        Arc::new(memory.clone()),
        registry.clone(),
    ));
    let llm = Box::new(MockLLMClient {
        response: "Hello from Mock".into(),
    });

    let _friday = Friday::new(bus.clone(), decision, memory, registry, context, llm);

    // Emit an event
    bus.emit(Event {
        source: "test".into(),
        event_type: "user_spoke".into(),
        payload: serde_json::json!({"text": "hi"}),
        priority: Urgency::High,
        ..Event::default()
    })
    .await;
    Ok(())
}
