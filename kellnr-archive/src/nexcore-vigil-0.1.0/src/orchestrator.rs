use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::context::ContextAssembler;
use crate::decision::DecisionEngine;
use crate::events::EventBus;
use crate::executors::Executor;
use crate::llm::LLMClient;
use crate::memory::MemoryLayer;
use crate::models::{DecisionAction, ExecutorType};
use crate::projects::ProjectRegistry;
use crate::sources::Source;

pub enum ExecutorImpl {
    Shell(crate::executors::shell::ShellExecutor),
    Notify(crate::executors::notify::NotifyExecutor),
    Speech(crate::executors::speech::SpeechExecutor),
    Maestro(crate::executors::maestro::MaestroExecutor),
}

#[async_trait::async_trait]
impl Executor for ExecutorImpl {
    fn executor_type(&self) -> ExecutorType {
        match self {
            Self::Shell(e) => e.executor_type(),
            Self::Notify(e) => e.executor_type(),
            Self::Speech(e) => e.executor_type(),
            Self::Maestro(e) => e.executor_type(),
        }
    }
    async fn execute(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<crate::models::ExecutorResult> {
        match self {
            Self::Shell(e) => e.execute(action, params).await,
            Self::Notify(e) => e.execute(action, params).await,
            Self::Speech(e) => e.execute(action, params).await,
            Self::Maestro(e) => e.execute(action, params).await,
        }
    }
    async fn health_check(&self) -> bool {
        match self {
            Self::Shell(e) => e.health_check().await,
            Self::Notify(e) => e.health_check().await,
            Self::Speech(e) => e.health_check().await,
            Self::Maestro(e) => e.health_check().await,
        }
    }
}

pub enum SourceImpl {
    Filesystem(crate::sources::filesystem::FilesystemSource),
    Webhook(crate::sources::webhook::WebhookSource),
    Voice(crate::sources::voice::VoiceSource),
    GitMonitor(crate::sources::git_monitor::GitMonitor),
}

#[async_trait::async_trait]
impl Source for SourceImpl {
    fn name(&self) -> &'static str {
        match self {
            Self::Filesystem(s) => s.name(),
            Self::Webhook(s) => s.name(),
            Self::Voice(s) => s.name(),
            Self::GitMonitor(s) => s.name(),
        }
    }
    async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Filesystem(s) => s.run().await,
            Self::Webhook(s) => s.run().await,
            Self::Voice(s) => s.run().await,
            Self::GitMonitor(s) => s.run().await,
        }
    }
}

pub struct Friday {
    bus: Arc<EventBus>,
    decision: Arc<DecisionEngine>,
    memory: Arc<MemoryLayer>,
    registry: Arc<ProjectRegistry>,
    context: Arc<ContextAssembler>,
    llm: Arc<dyn LLMClient>,
    executors: Arc<Mutex<Vec<ExecutorImpl>>>,
    sources: Vec<SourceImpl>,
}

impl Friday {
    pub fn new(
        bus: EventBus,
        decision: DecisionEngine,
        memory: MemoryLayer,
        registry: Arc<ProjectRegistry>,
        context: Arc<ContextAssembler>,
        llm: Box<dyn LLMClient>,
    ) -> Self {
        Self {
            bus: Arc::new(bus),
            decision: Arc::new(decision),
            memory: Arc::new(memory),
            registry,
            context,
            llm: Arc::from(llm),
            executors: Arc::new(Mutex::new(vec![])),
            sources: vec![],
        }
    }

    pub async fn add_executor(&self, executor: ExecutorImpl) {
        let mut execs = self.executors.lock().await;
        execs.push(executor);
    }

    pub fn add_source(&mut self, source: SourceImpl) {
        self.sources.push(source);
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!("FRIDAY-RS core loop starting");

        for source in self.sources {
            tokio::spawn(async move {
                loop {
                    let name = source.name();
                    info!(source = %name, "source_starting");
                    if let Err(e) = source.run().await {
                        error!(source = %name, error = %e, "source_crashed_restarting_in_5s");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    } else {
                        info!(source = %name, "source_stopped_cleanly");
                        break;
                    }
                }
            });
        }

        loop {
            let event = self.bus.consume().await;
            let decision_engine = self.decision.clone();
            let context_assembler = self.context.clone();
            let memory = self.memory.clone();
            let llm = self.llm.clone();
            let executors = self.executors.clone();
            let registry = self.registry.clone();

            tokio::spawn(async move {
                let context_task = {
                    let ctx_asm = context_assembler.clone();
                    let ev = event.clone();
                    tokio::spawn(async move { ctx_asm.build_context(&ev).await })
                };

                let action = decision_engine.decide(&event).await;

                match action {
                    DecisionAction::InvokeClaude => {
                        let context_result = match context_task.await {
                            Ok(res) => res,
                            Err(e) => {
                                error!(error = %e, "context_task_panic");
                                return;
                            }
                        };

                        match context_result {
                            Ok(context_str) => match llm.invoke(&context_str, &event).await {
                                Ok(interaction) => {
                                    info!(
                                        resp_len = interaction.response.len(),
                                        "claude_responded"
                                    );

                                    let user_text = event
                                        .payload
                                        .get("text")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("")
                                        .to_string();

                                    context_assembler
                                        .add_to_buffer("user".to_string(), user_text)
                                        .await;
                                    context_assembler
                                        .add_to_buffer(
                                            "assistant".to_string(),
                                            interaction.response.clone(),
                                        )
                                        .await;

                                    let _ = memory.remember_interaction(interaction.clone()).await;

                                    let execs = executors.lock().await;
                                    for action_str in &interaction.actions_taken {
                                        for exec in execs.iter() {
                                            match exec
                                                .execute(action_str, serde_json::json!({}))
                                                .await
                                            {
                                                Ok(res) => {
                                                    if !res.success {
                                                        warn!(action = %action_str, error = ?res.error, "action_failed");
                                                    }
                                                }
                                                Err(e) => {
                                                    error!(action = %action_str, error = %e, "executor_error")
                                                }
                                            }
                                        }
                                    }

                                    if event.source == "voice" {
                                        for exec in execs.iter() {
                                            if exec.executor_type() == ExecutorType::Speech {
                                                let _ = exec
                                                    .execute(
                                                        &interaction.response,
                                                        serde_json::json!({}),
                                                    )
                                                    .await;
                                            }
                                        }
                                    }
                                }
                                Err(e) => error!(error = %e, "llm_invocation_failed"),
                            },
                            Err(e) => warn!(error = %e, "failed_to_build_context"),
                        }
                    }
                    DecisionAction::Escalate => {
                        info!("escalating_event");
                    }
                    DecisionAction::AutonomousAct => {
                        if event.source == "git_monitor" && event.event_type == "git_stale" {
                            if let Some(path_str) =
                                event.payload.get("path").and_then(|p| p.as_str())
                            {
                                info!(path = %path_str, "handling_stale_repo_autonomously");
                                let _ = registry
                                    .update_status(path_str, crate::projects::ProjectStatus::Stale)
                                    .await;
                            }
                        }
                    }
                    _ => {
                        info!(action = ?action, "action_not_fully_implemented");
                    }
                }
            });
        }
    }
}
