//! Brain CLI - Antigravity-style working memory for Claude Code
//!
//! Usage:
//!   brain session new                    # Create new session
//!   brain session list                   # List all sessions
//!   brain session current                # Get current (latest) session ID
//!   brain session load <id>              # Load session by ID
//!   brain artifact save <file>           # Save artifact to current session
//!   brain artifact resolve <name>        # Create resolved snapshot
//!   brain artifact get <name>            # Get current artifact
//!   brain artifact get <name> --version N  # Get specific version
//!   brain artifact list                  # List artifacts in session
//!   brain artifact diff <name> <v1> <v2> # Diff two versions
//!   brain track <file>                   # Track a file for change detection
//!   brain changed <file>                 # Check if file changed
//!   brain original <file>                # Get original content
//!   brain init                           # Initialize brain directories

use clap::{Parser, Subcommand};
use nexcore_brain::{
    Artifact, ArtifactType, BrainSession, CodeTracker, ImplicitKnowledge, attempt_recovery,
    check_brain_availability, check_index_health, detect_partial_writes, initialize_directories,
    rebuild_index_from_sessions, repair_partial_writes,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "brain")]
#[command(about = "Antigravity-style working memory for Claude Code")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize brain directories
    Init,

    /// Session management
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// Artifact management
    Artifact {
        #[command(subcommand)]
        action: ArtifactAction,
    },

    /// Track a file for change detection
    Track {
        /// File to track
        file: PathBuf,

        /// Project name (defaults to current directory name)
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Check if a tracked file has changed
    Changed {
        /// File to check
        file: PathBuf,

        /// Project name
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Get original content of a tracked file
    Original {
        /// File to get original content for
        file: PathBuf,

        /// Project name
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Implicit knowledge operations
    Implicit {
        #[command(subcommand)]
        action: ImplicitAction,
    },

    /// Recovery and health check operations
    Recovery {
        #[command(subcommand)]
        action: RecoveryAction,
    },
}

#[derive(Subcommand)]
enum SessionAction {
    /// Create a new session
    New {
        /// Project name
        #[arg(short, long)]
        project: Option<String>,

        /// Git commit hash
        #[arg(short, long)]
        commit: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,

        /// Inject session ID into context (for hooks)
        #[arg(long)]
        inject_context: bool,
    },

    /// List all sessions
    List,

    /// Get the current (latest) session ID
    Current,

    /// Load a session by ID
    Load {
        /// Session ID
        id: String,
    },

    /// Import sessions from Antigravity brain
    ImportAntigravity,
}

#[derive(Subcommand)]
enum ArtifactAction {
    /// Save an artifact from a file
    Save {
        /// File to save as artifact
        file: PathBuf,

        /// Session ID (defaults to latest)
        #[arg(short, long)]
        session: Option<String>,

        /// Artifact type (task, plan, walkthrough, review, research, decision, custom)
        #[arg(short = 't', long)]
        artifact_type: Option<String>,
    },

    /// Resolve an artifact (create immutable snapshot)
    Resolve {
        /// Artifact name
        name: String,

        /// Session ID (defaults to latest)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Get an artifact's content
    Get {
        /// Artifact name
        name: String,

        /// Session ID (defaults to latest)
        #[arg(short, long)]
        session: Option<String>,

        /// Specific version number
        #[arg(short, long)]
        version: Option<u32>,
    },

    /// List artifacts in a session
    List {
        /// Session ID (defaults to latest)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// List versions of an artifact
    Versions {
        /// Artifact name
        name: String,

        /// Session ID (defaults to latest)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Diff two versions of an artifact
    Diff {
        /// Artifact name
        name: String,

        /// First version
        v1: u32,

        /// Second version
        v2: u32,

        /// Session ID (defaults to latest)
        #[arg(short, long)]
        session: Option<String>,
    },
}

#[derive(Subcommand)]
enum ImplicitAction {
    /// Get a preference
    GetPref {
        /// Preference key
        key: String,
    },

    /// Set a preference
    SetPref {
        /// Preference key
        key: String,

        /// Preference value (JSON)
        value: String,
    },

    /// List all preferences
    ListPrefs,

    /// Get statistics
    Stats,
}

#[derive(Subcommand)]
enum RecoveryAction {
    /// Check brain health status
    Check,

    /// Repair partial writes (create missing metadata)
    Repair {
        /// Session ID to repair (defaults to latest)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Rebuild index from session directories
    RebuildIndex,

    /// Attempt automatic recovery
    Auto,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> nexcore_brain::error::Result<()> {
    match cli.command {
        Commands::Init => {
            initialize_directories()?;
            println!("Brain directories initialized");
            Ok(())
        }
        Commands::Session { action } => handle_session_action(action),
        Commands::Artifact { action } => handle_artifact_action(action),
        Commands::Track { file, project } => handle_track(file, project),
        Commands::Changed { file, project } => handle_changed(file, project),
        Commands::Original { file, project } => handle_original(file, project),
        Commands::Implicit { action } => handle_implicit_action(action),
        Commands::Recovery { action } => handle_recovery_action(action),
    }
}

fn handle_session_action(action: SessionAction) -> nexcore_brain::error::Result<()> {
    match action {
        SessionAction::New {
            project,
            commit,
            description,
            inject_context,
        } => create_new_session(project, commit, description, inject_context),
        SessionAction::List => list_sessions(),
        SessionAction::Current => get_current_session(),
        SessionAction::Load { id } => load_session_by_id(id),
        SessionAction::ImportAntigravity => import_antigravity_sessions(),
    }
}

fn import_antigravity_sessions() -> nexcore_brain::error::Result<()> {
    println!("Scanning Antigravity brain for sessions...");
    let count = BrainSession::import_from_antigravity()?;
    if count > 0 {
        println!("Successfully imported {count} session(s)");
    } else {
        println!("No new sessions found in Antigravity brain");
    }
    Ok(())
}

fn create_new_session(
    project: Option<String>,
    commit: Option<String>,
    description: Option<String>,
    inject_context: bool,
) -> nexcore_brain::error::Result<()> {
    let session = BrainSession::create_with_options(project, commit, description)?;
    let id = session.id;

    if inject_context {
        println!("{{\"brain_session_id\": \"{id}\"}}");
    } else {
        println!("{id}");
    }
    Ok(())
}

fn list_sessions() -> nexcore_brain::error::Result<()> {
    let sessions = BrainSession::list_all()?;
    if sessions.is_empty() {
        println!("No sessions found");
        return Ok(());
    }

    for session in sessions {
        let project = session.project.as_deref().unwrap_or("-");
        let desc = session.description.as_deref().unwrap_or("");
        println!(
            "{}\t{}\t{}\t{}",
            session.id,
            session
                .created_at
                .format("%Y-%m-%d %H:%M")
                .unwrap_or_default(),
            project,
            desc
        );
    }
    Ok(())
}

fn get_current_session() -> nexcore_brain::error::Result<()> {
    let session = BrainSession::load_latest()?;
    println!("{}", session.id);
    Ok(())
}

fn load_session_by_id(id: String) -> nexcore_brain::error::Result<()> {
    let session = BrainSession::load_str(&id)?;
    println!("Loaded session: {}", session.id);
    println!("Created: {}", session.created_at);
    if let Some(p) = &session.project {
        println!("Project: {p}");
    }
    if let Some(c) = &session.git_commit {
        println!("Git commit: {c}");
    }
    Ok(())
}

fn handle_artifact_action(action: ArtifactAction) -> nexcore_brain::error::Result<()> {
    match action {
        ArtifactAction::Save {
            file,
            session,
            artifact_type,
        } => save_artifact_cmd(file, session, artifact_type),
        ArtifactAction::Resolve { name, session } => resolve_artifact_cmd(name, session),
        ArtifactAction::Get {
            name,
            session,
            version,
        } => get_artifact_cmd(name, session, version),
        ArtifactAction::List { session } => list_artifacts_cmd(session),
        ArtifactAction::Versions { name, session } => list_artifact_versions_cmd(name, session),
        ArtifactAction::Diff {
            name,
            v1,
            v2,
            session,
        } => diff_artifact_versions_cmd(name, v1, v2, session),
    }
}

fn save_artifact_cmd(
    file: PathBuf,
    session: Option<String>,
    artifact_type: Option<String>,
) -> nexcore_brain::error::Result<()> {
    let session = get_session(session)?;
    let content = std::fs::read_to_string(&file)?;
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("artifact");

    let art_type = artifact_type
        .map(|t| t.parse::<ArtifactType>())
        .transpose()?
        .unwrap_or_else(|| ArtifactType::from_filename(name));

    let artifact = Artifact::new(name, art_type, content);
    session.save_artifact(&artifact)?;
    println!("Saved artifact: {name}");
    Ok(())
}

fn resolve_artifact_cmd(name: String, session: Option<String>) -> nexcore_brain::error::Result<()> {
    let session = get_session(session)?;
    let version = session.resolve_artifact(&name)?;
    println!("Resolved {name} to version {version}");
    Ok(())
}

fn get_artifact_cmd(
    name: String,
    session: Option<String>,
    version: Option<u32>,
) -> nexcore_brain::error::Result<()> {
    let session = get_session(session)?;
    let artifact = session.get_artifact(&name, version)?;
    print!("{}", artifact.content);
    Ok(())
}

fn list_artifacts_cmd(session: Option<String>) -> nexcore_brain::error::Result<()> {
    let session = get_session(session)?;
    let artifacts = session.list_artifacts()?;
    if artifacts.is_empty() {
        println!("No artifacts in session");
    } else {
        for name in artifacts {
            println!("{name}");
        }
    }
    Ok(())
}

fn list_artifact_versions_cmd(
    name: String,
    session: Option<String>,
) -> nexcore_brain::error::Result<()> {
    let session = get_session(session)?;
    let versions = session.list_versions(&name)?;
    if versions.is_empty() {
        println!("No resolved versions for {name}");
    } else {
        for v in versions {
            println!("{v}");
        }
    }
    Ok(())
}

fn diff_artifact_versions_cmd(
    name: String,
    v1: u32,
    v2: u32,
    session: Option<String>,
) -> nexcore_brain::error::Result<()> {
    let session = get_session(session)?;
    let diff = session.diff_versions(&name, v1, v2)?;
    print!("{diff}");
    Ok(())
}

fn handle_track(file: PathBuf, project: Option<String>) -> nexcore_brain::error::Result<()> {
    let project = project.unwrap_or_else(get_default_project);
    let mut tracker = CodeTracker::new(project, None)?;
    let tracked = tracker.track_file(&file)?;
    println!(
        "Tracked {} (hash: {})",
        file.display(),
        &tracked.content_hash[..16]
    );
    Ok(())
}

fn handle_changed(file: PathBuf, project: Option<String>) -> nexcore_brain::error::Result<()> {
    let project = project.unwrap_or_else(get_default_project);
    let tracker = CodeTracker::load(&project)?;
    let changed = tracker.has_changed(&file)?;
    if changed {
        println!("CHANGED");
        std::process::exit(1);
    } else {
        println!("UNCHANGED");
    }
    Ok(())
}

fn handle_original(file: PathBuf, project: Option<String>) -> nexcore_brain::error::Result<()> {
    let project = project.unwrap_or_else(get_default_project);
    let tracker = CodeTracker::load(&project)?;
    let content = tracker.get_original(&file)?;
    print!("{content}");
    Ok(())
}

fn handle_implicit_action(action: ImplicitAction) -> nexcore_brain::error::Result<()> {
    match action {
        ImplicitAction::GetPref { key } => get_preference_cmd(key),
        ImplicitAction::SetPref { key, value } => set_preference_cmd(key, value),
        ImplicitAction::ListPrefs => list_preferences_cmd(),
        ImplicitAction::Stats => get_implicit_stats_cmd(),
    }
}

fn get_preference_cmd(key: String) -> nexcore_brain::error::Result<()> {
    let knowledge = ImplicitKnowledge::load()?;
    if let Some(pref) = knowledge.get_preference(&key) {
        println!("{}", serde_json::to_string_pretty(&pref.value)?);
    } else {
        eprintln!("Preference not found: {key}");
        std::process::exit(1);
    }
    Ok(())
}

fn set_preference_cmd(key: String, value: String) -> nexcore_brain::error::Result<()> {
    let mut knowledge = ImplicitKnowledge::load()?;
    let value: serde_json::Value = serde_json::from_str(&value)?;
    knowledge.set_preference_value(&key, value);
    knowledge.save()?;
    println!("Set preference: {key}");
    Ok(())
}

fn list_preferences_cmd() -> nexcore_brain::error::Result<()> {
    let knowledge = ImplicitKnowledge::load()?;
    let prefs = knowledge.list_preferences();
    if prefs.is_empty() {
        println!("No preferences set");
        return Ok(());
    }

    for pref in prefs {
        println!(
            "{}\t{:.2}\t{}",
            pref.key,
            pref.confidence,
            serde_json::to_string(&pref.value)?
        );
    }
    Ok(())
}

fn get_implicit_stats_cmd() -> nexcore_brain::error::Result<()> {
    let knowledge = ImplicitKnowledge::load()?;
    let stats = knowledge.stats();
    println!("{}", serde_json::to_string_pretty(&stats)?);
    Ok(())
}

fn handle_recovery_action(action: RecoveryAction) -> nexcore_brain::error::Result<()> {
    match action {
        RecoveryAction::Check => check_recovery_cmd(),
        RecoveryAction::Repair { session } => repair_recovery_cmd(session),
        RecoveryAction::RebuildIndex => rebuild_index_cmd(),
        RecoveryAction::Auto => auto_recovery_cmd(),
    }
}

fn check_recovery_cmd() -> nexcore_brain::error::Result<()> {
    println!("Brain Health Check");
    println!("==================");

    if let Some(reason) = check_brain_availability() {
        println!("Status: DEGRADED");
        println!("Reason: {reason}");
    } else {
        println!("Status: HEALTHY");
    }

    print!("Index: ");
    match check_index_health() {
        Some(reason) => println!("CORRUPTED - {reason}"),
        None => println!("OK"),
    }

    if let Ok(session) = BrainSession::load_latest() {
        print_partial_writes(&session);
    }
    Ok(())
}

fn print_partial_writes(session: &BrainSession) {
    let partials = detect_partial_writes(&session.dir());
    if partials.is_empty() {
        println!("Partial writes: None detected");
    } else {
        println!(
            "Partial writes: {} artifact(s) missing metadata",
            partials.len()
        );
        for name in &partials {
            println!("  - {name}");
        }
    }
}

fn repair_recovery_cmd(session: Option<String>) -> nexcore_brain::error::Result<()> {
    let session = get_session(session)?;
    println!("Repairing partial writes in session: {}", session.id);

    let result = repair_partial_writes(&session.dir())?;
    if result.success {
        println!("Repaired {} artifact(s)", result.recovered_count);
        for detail in &result.details {
            println!("  {detail}");
        }
    } else {
        println!("No repairs needed");
    }

    if !result.warnings.is_empty() {
        println!("Warnings:");
        for warning in &result.warnings {
            println!("  {warning}");
        }
    }
    Ok(())
}

fn rebuild_index_cmd() -> nexcore_brain::error::Result<()> {
    println!("Rebuilding index from session directories...");

    let result = rebuild_index_from_sessions()?;
    if result.success {
        println!("Rebuilt index with {} session(s)", result.recovered_count);
        for detail in &result.details {
            println!("  {detail}");
        }
    } else {
        println!("Index rebuild failed");
    }

    if !result.warnings.is_empty() {
        println!("Warnings:");
        for warning in &result.warnings {
            println!("  {warning}");
        }
    }
    Ok(())
}

fn auto_recovery_cmd() -> nexcore_brain::error::Result<()> {
    println!("Attempting automatic recovery...");

    let result = attempt_recovery()?;
    if result.success {
        if result.recovered_count > 0 {
            println!(
                "Recovery successful: {} item(s) recovered",
                result.recovered_count
            );
        } else {
            println!("No recovery needed - brain is healthy");
        }
        for detail in &result.details {
            println!("  {detail}");
        }
    } else {
        println!("Recovery not performed");
        for warning in &result.warnings {
            println!("  {warning}");
        }
    }
    Ok(())
}

fn get_session(id: Option<String>) -> nexcore_brain::error::Result<BrainSession> {
    match id {
        Some(id) => Ok(BrainSession::load_str(&id)?),
        None => Ok(BrainSession::load_latest()?),
    }
}

fn get_default_project() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "default".to_string())
}
