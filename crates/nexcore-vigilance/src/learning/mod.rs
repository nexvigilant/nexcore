//! # nexcore Learning
//!
//! Teaching ecosystem with observation, intervention, and A/B evaluation.
//!
//! ## Features
//! - **Observation**: Capture agent execution events
//! - **Intervention**: Generate coaching feedback
//! - **Metrics**: Track performance with A/B testing
//! - **Teaching**: Register teachers for closed-loop learning

mod agent;
mod coach;
mod metrics;
mod models;

pub use agent::{AgentConfig, TeachableAgent, Teacher};
pub use coach::{InterventionGenerator, MonitorAlert};
pub use metrics::{
    ABTestResult, ABTestSession, EvaluationEngine, KPIs, PerformanceMetrics, Variant,
};
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_chrono::DateTime;
    use nexcore_id::NexId;

    struct MockTeacher;

    #[async_trait::async_trait]
    impl Teacher for MockTeacher {
        async fn observe(&self, _observation: Observation) {}
        async fn intervene(&self, _intervention: Intervention) -> Option<String> {
            Some("acknowledged".to_string())
        }
    }

    #[tokio::test]
    async fn test_teachable_agent_session() {
        let config = AgentConfig {
            name: "test-agent".into(),
            description: "Test agent".into(),
            model: "test-model".into(),
        };

        let mut agent = TeachableAgent::new(config);
        agent.enable_teaching("mock", Box::new(MockTeacher));

        let session_id = agent.start_session().await.unwrap();
        assert!(session_id.starts_with("session_"));
        assert!(agent.is_teaching_enabled());
    }

    #[test]
    fn test_intervention_generator() {
        let generator = InterventionGenerator::new("test-model");

        let alert = MonitorAlert {
            alert_type: "error".into(),
            severity: AndonSignal::Red,
            message: "Critical failure".into(),
        };

        let intervention = generator.generate_intervention(&alert);
        assert!(matches!(
            intervention.intervention_type,
            InterventionType::Halt
        ));
        assert!(intervention.message.contains("Please stop"));
    }

    #[test]
    fn test_ab_evaluation() {
        let mut engine = EvaluationEngine::new();

        // Control session
        engine.record_session(ABTestSession {
            session_id: NexId::v4(),
            variant: Variant::Control,
            user_id: "user1".into(),
            metrics: vec![PerformanceMetrics {
                task_id: "task1".into(),
                start_time: DateTime::now(),
                end_time: DateTime::now(),
                completion_time_ms: 1000,
                success: true,
                skills_used: vec![],
                errors_encountered: 0,
                interventions_received: 0,
            }],
        });

        // Treatment session (faster)
        engine.record_session(ABTestSession {
            session_id: NexId::v4(),
            variant: Variant::Treatment,
            user_id: "user2".into(),
            metrics: vec![PerformanceMetrics {
                task_id: "task1".into(),
                start_time: DateTime::now(),
                end_time: DateTime::now(),
                completion_time_ms: 800,
                success: true,
                skills_used: vec!["skill1".into()],
                errors_encountered: 0,
                interventions_received: 1,
            }],
        });

        let results = engine.calculate_results();
        assert!(results.lift > 0.0, "Treatment should show improvement");
        assert_eq!(results.control_kpis.avg_completion_time_ms, 1000.0);
        assert_eq!(results.treatment_kpis.avg_completion_time_ms, 800.0);
    }
}
