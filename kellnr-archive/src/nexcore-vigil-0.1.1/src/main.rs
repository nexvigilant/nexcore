use clap::{Parser, Subcommand};
use nexcore_vigil::{
    AuthorityConfig, ContextAssembler, DecisionEngine, EventBus, Friday, MemoryLayer,
    config::Config,
    executors::{shell::ShellExecutor, speech::SpeechExecutor},
    llm::{LLMClient, claude::ClaudeClient, gemini::GeminiClient},
    orchestrator::{ExecutorImpl, SourceImpl},
    projects::ProjectRegistry,
    sources::{git_monitor::GitMonitor, voice::VoiceSource, webhook::WebhookSource},
};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "vigil")]
#[command(about = "FRIDAY AI Assistant - High Performance Rust Core", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FRIDAY daemon
    Run,
    /// Show the current status of FRIDAY
    Status,
    /// Index the KSB directory
    Index {
        /// Force re-indexing of all files
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;

    match cli.command {
        Commands::Run => run_daemon(config).await,
        Commands::Status => {
            println!("FRIDAY-RS Status: Online");
            Ok(())
        }
        Commands::Index { force: _ } => {
            let memory = MemoryLayer::new(
                config.ksb_root.clone(),
                config.data_dir.clone(),
                &config.qdrant_url,
            )
            .await?;
            println!("Starting indexing of {:?}...", config.ksb_root);
            let count = memory.index_ksb().await?;
            println!("Indexed {} new/changed files.", count);
            Ok(())
        }
    }
}

async fn run_daemon(config: Config) -> anyhow::Result<()> {
    tracing::info!("FRIDAY-RS starting daemon...");

    let bus = EventBus::new(config.event_bus_size);

    let authority = AuthorityConfig {
        autonomous_allowed: vec!["scheduler".to_string(), "git_monitor".to_string()],
        forbidden: vec![],
        requires_confirmation: vec!["shell".to_string()],
    };
    let decision = DecisionEngine::new(authority);

    tokio::fs::create_dir_all(&config.data_dir).await?;

    let memory = Arc::new(
        MemoryLayer::new(
            config.ksb_root.clone(),
            config.data_dir.clone(),
            &config.qdrant_url,
        )
        .await?,
    );

    memory.initialize(false).await?;

    let registry = Arc::new(ProjectRegistry::new(config.data_dir.clone()));
    registry.initialize().await?;

    let context = Arc::new(ContextAssembler::new(
        config.ksb_root.clone(),
        memory.clone(),
        registry.clone(),
    ));

    let llm: Box<dyn LLMClient> = match config.llm_provider.as_str() {
        "gemini" | "gemini-3" => {
            let model = std::env::var("GEMINI_MODEL")
                .unwrap_or_else(|_| "gemini-3-flash-preview".to_string());
            // Prefer Vertex AI if project is set, otherwise use API key
            if let Ok(project) = std::env::var("GCLOUD_PROJECT") {
                tracing::info!(model = %model, project = %project, "Initializing Gemini via Vertex AI");
                Box::new(GeminiClient::vertex(project, model))
            } else {
                let key = config
                    .gemini_api_key
                    .ok_or_else(|| anyhow::anyhow!("GEMINI_API_KEY or GCLOUD_PROJECT required"))?;
                tracing::info!(model = %model, "Initializing Gemini via API key");
                Box::new(GeminiClient::new(key, model))
            }
        }
        "claude" => {
            let key = config
                .anthropic_api_key
                .ok_or_else(|| anyhow::anyhow!("ANTHROPIC_API_KEY not set"))?;
            let model = std::env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
            tracing::info!(model = %model, "Initializing Claude LLM");
            Box::new(ClaudeClient::new(key, model)?)
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown LLM provider: {}",
                config.llm_provider
            ));
        }
    };

    let mut vigil = Friday::new(
        bus.clone(),
        decision,
        memory.as_ref().clone(),
        registry.clone(),
        context,
        llm,
    );

    // Add Executors
    vigil
        .add_executor(ExecutorImpl::Shell(ShellExecutor::new()))
        .await;
    vigil
        .add_executor(ExecutorImpl::Speech(SpeechExecutor))
        .await;

    // Add Sources
    let api_key = config.friday_api_key.clone();
    vigil.add_source(SourceImpl::Webhook(WebhookSource::new(
        bus.clone(),
        config.webhook_port,
        api_key,
    )));
    vigil.add_source(SourceImpl::Voice(VoiceSource::new(bus.clone())));

    // Add Git Monitor for current directory
    vigil.add_source(SourceImpl::GitMonitor(GitMonitor::new(
        bus.clone(),
        vec![std::env::current_dir()?],
        3600,
    )));

    tracing::info!("FRIDAY-RS daemon online!");
    vigil.run().await?;

    Ok(())
}
