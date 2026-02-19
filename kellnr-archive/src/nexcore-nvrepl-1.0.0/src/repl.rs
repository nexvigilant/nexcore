//! REPL loop for NVREPL
//!
//! Tier: T3 (grounds to σ Sequence + ς State — sequential command loop with mutable state)

use colored::Colorize;
use nexcore_guardian_engine::create_monitoring_loop;
use nexcore_guardian_engine::homeostasis::{DecisionEngine, HomeostasisLoop};
use rustyline::Editor;
use rustyline::error::ReadlineError;

use crate::commands::Command;
use crate::completer::NvCompleter;
use crate::{energy, format, monitor, safety, signal};

/// Persistent history file location
const HISTORY_FILE: &str = "~/.nvrepl_history";

struct NvreplState {
    /// Guardian homeostasis loop (original)
    loop_controller: HomeostasisLoop,
    /// Dedicated monitoring loop
    monitor_loop: HomeostasisLoop,
    /// Command history for status display
    history: Vec<String>,
    /// Cached last monitoring tick result
    last_monitor_tick: Option<nexcore_guardian_engine::homeostasis::LoopIterationResult>,
}

impl NvreplState {
    fn new() -> Self {
        Self {
            loop_controller: HomeostasisLoop::new(DecisionEngine::new()),
            monitor_loop: create_monitoring_loop(),
            history: Vec::new(),
            last_monitor_tick: None,
        }
    }

    fn reset(&mut self) {
        self.loop_controller.reset();
        self.monitor_loop = create_monitoring_loop();
        self.history.clear();
        self.last_monitor_tick = None;
    }
}

pub async fn run() -> anyhow::Result<()> {
    print_banner();

    let config = rustyline::Config::builder().auto_add_history(true).build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(NvCompleter::new()));

    // Load persistent history (ignore errors on first run)
    let history_path = expand_tilde(HISTORY_FILE);
    let _ = rl.load_history(&history_path);

    let mut state = NvreplState::new();

    loop {
        let prompt = format!("{} ", "nvrepl>".green().bold());
        match rl.readline(&prompt) {
            Ok(line) => {
                let _added = rl.add_history_entry(line.as_str());
                state.history.push(line.clone());
                execute(&line, &mut state).await;
            }
            Err(ReadlineError::Interrupted) => println!("^C"),
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        }
    }

    // Save history on exit
    let _ = rl.save_history(&history_path);

    println!("\n{}", "NVREPL terminated.".dimmed());
    Ok(())
}

fn print_banner() {
    println!(
        "{}",
        "╔════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║    NVREPL — NexVigilant Terminal Interface     ║".cyan()
    );
    println!(
        "{}",
        "║  Signal Detection | Safety | Energy | Monitor  ║".cyan()
    );
    println!(
        "{}",
        "║  No LLM • No Tokens • <10ms Latency            ║".cyan()
    );
    println!(
        "{}\n",
        "╚════════════════════════════════════════════════╝".cyan()
    );
    println!("Type {} for commands\n", "help".yellow());
}

async fn execute(line: &str, state: &mut NvreplState) {
    let start = std::time::Instant::now();
    let response = dispatch(Command::parse(line), state).await;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    if !response.is_empty() {
        println!("{response}");
    }
    println!(
        "{} {}",
        "\u{23f1}".dimmed(),
        format!("{elapsed_ms:.2}ms").dimmed()
    );
}

async fn dispatch(cmd: Command, state: &mut NvreplState) -> String {
    match cmd {
        // === Guardian (existing) ===
        Command::Risk(ctx) => format::risk_response(&ctx),
        Command::Tick => format::tick_response(&state.loop_controller.tick().await),
        Command::Status => format::status_response(
            state.loop_controller.sensor_count(),
            state.loop_controller.actuator_count(),
            state.loop_controller.iteration_count(),
            state.history.len(),
        ),
        Command::Reset => {
            state.reset();
            format!("\n{}\n", "NVREPL state reset.".green())
        }
        Command::Originator(t) => format::originator_response(&t),

        // === Signal Detection ===
        Command::Signal { a, b, c, d } => signal::handle_signal(a, b, c, d),
        Command::Prr { a, b, c, d } => signal::handle_prr(a, b, c, d),
        Command::Ror { a, b, c, d } => signal::handle_ror(a, b, c, d),
        Command::Ic { a, b, c, d } => signal::handle_ic(a, b, c, d),
        Command::Ebgm { a, b, c, d } => signal::handle_ebgm(a, b, c, d),

        // === Monitoring ===
        Command::Health => {
            let tick = state.monitor_loop.tick().await;
            let response = monitor::handle_health(&tick);
            state.last_monitor_tick = Some(tick);
            response
        }
        Command::Alerts { severity } => {
            // Use cached tick or run a fresh one
            let tick = match &state.last_monitor_tick {
                Some(t) => t.clone(),
                None => {
                    let t = state.monitor_loop.tick().await;
                    state.last_monitor_tick = Some(t.clone());
                    t
                }
            };
            monitor::handle_alerts(&tick, severity.as_deref())
        }
        Command::Sensors => monitor::handle_sensors(
            state.monitor_loop.sensor_count(),
            state.monitor_loop.actuator_count(),
        ),
        Command::MonitorTick => {
            let tick = state.monitor_loop.tick().await;
            let response = monitor::handle_montick(&tick);
            state.last_monitor_tick = Some(tick);
            response
        }

        // === Patient Safety ===
        Command::Triage { seriousness } => safety::handle_triage(&seriousness),
        Command::Priority { a, b } => safety::handle_priority(&a, &b),
        Command::Escalation { seriousness } => safety::handle_escalation(&seriousness),

        // === Energy ===
        Command::Energy { budget } => energy::handle_energy(budget),
        Command::Decide {
            budget,
            cost,
            value,
        } => energy::handle_decide(budget, cost, value),

        // === Meta ===
        Command::Help => format::help(),
        Command::Exit => std::process::exit(0),
        Command::Unknown(s) if s.is_empty() => String::new(),
        Command::Unknown(s) => format!("{}: {s}", "Unknown".red()),
    }
}

/// Expand ~ to home directory
fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{stripped}");
        }
    }
    path.to_string()
}
