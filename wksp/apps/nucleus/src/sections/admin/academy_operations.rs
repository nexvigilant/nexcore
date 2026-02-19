//! Admin: Academy operations — enrollment, completion tracking, system health

use leptos::prelude::*;
use std::path::Path;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompoundingPipelineResult {
    mode: String,
    ok: bool,
    exit_code: i32,
    started_at: String,
    finished_at: String,
    stdout: String,
    stderr: String,
}

fn trim_output(s: &str, max_chars: usize) -> String {
    let total = s.chars().count();
    if total <= max_chars {
        return s.to_string();
    }
    let trimmed: String = s.chars().skip(total - max_chars).collect();
    format!(
        "... [truncated {}/{} chars]\n{}",
        total - max_chars,
        total,
        trimmed
    )
}

#[server(ExecuteCompoundingPipeline, "/api")]
pub async fn execute_compounding_pipeline_action(
    mode: String,
) -> Result<CompoundingPipelineResult, ServerFnError> {
    let started_at = chrono::Utc::now().to_rfc3339();
    let home =
        std::env::var("HOME").map_err(|e| ServerFnError::new(format!("HOME not set: {e}")))?;
    let script = format!("{home}/.claude/skills/compounding-pipeline/scripts/run-pipeline.sh");
    if !Path::new(&script).exists() {
        return Err(ServerFnError::new(format!(
            "Pipeline script not found at {script}"
        )));
    }

    fn run_mode(script: &str, mode: &str) -> Result<std::process::Output, ServerFnError> {
        std::process::Command::new(script)
            .arg(mode)
            .output()
            .map_err(|e| ServerFnError::new(format!("Failed to execute `{mode}`: {e}")))
    }

    let (ok, exit_code, stdout, stderr) = if mode == "full-apply" {
        let full = run_mode(&script, "full")?;
        let apply = run_mode(&script, "apply")?;
        let full_code = full.status.code().unwrap_or(-1);
        let apply_code = apply.status.code().unwrap_or(-1);
        let ok = full.status.success() && apply.status.success();
        let combined_stdout = format!(
            "$ run-pipeline.sh full\n{}\n\n$ run-pipeline.sh apply\n{}",
            String::from_utf8_lossy(&full.stdout),
            String::from_utf8_lossy(&apply.stdout)
        );
        let combined_stderr = format!(
            "$ run-pipeline.sh full\n{}\n\n$ run-pipeline.sh apply\n{}",
            String::from_utf8_lossy(&full.stderr),
            String::from_utf8_lossy(&apply.stderr)
        );
        (
            ok,
            if apply_code != 0 {
                apply_code
            } else {
                full_code
            },
            trim_output(&combined_stdout, 16_000),
            trim_output(&combined_stderr, 8_000),
        )
    } else {
        let allowed = ["full", "status", "beliefs", "proposals", "apply"];
        if !allowed.contains(&mode.as_str()) {
            return Err(ServerFnError::new(format!(
                "Unsupported mode `{mode}`. Allowed: full, status, beliefs, proposals, apply, full-apply"
            )));
        }
        let out = run_mode(&script, &mode)?;
        (
            out.status.success(),
            out.status.code().unwrap_or(-1),
            trim_output(&String::from_utf8_lossy(&out.stdout), 16_000),
            trim_output(&String::from_utf8_lossy(&out.stderr), 8_000),
        )
    };

    let finished_at = chrono::Utc::now().to_rfc3339();
    Ok(CompoundingPipelineResult {
        mode,
        ok,
        exit_code,
        started_at,
        finished_at,
        stdout,
        stderr,
    })
}

struct SystemComponent {
    name: &'static str,
    status: &'static str,
    latency: &'static str,
    uptime: &'static str,
}

const SYSTEMS: &[SystemComponent] = &[
    SystemComponent {
        name: "Content Delivery",
        status: "Operational",
        latency: "12ms",
        uptime: "99.99%",
    },
    SystemComponent {
        name: "Assessment Engine",
        status: "Operational",
        latency: "45ms",
        uptime: "99.97%",
    },
    SystemComponent {
        name: "Certificate Issuance",
        status: "Operational",
        latency: "89ms",
        uptime: "99.95%",
    },
    SystemComponent {
        name: "Progress Tracking",
        status: "Operational",
        latency: "8ms",
        uptime: "99.99%",
    },
    SystemComponent {
        name: "KSB Framework Sync",
        status: "Operational",
        latency: "156ms",
        uptime: "99.92%",
    },
    SystemComponent {
        name: "FAERS Integration",
        status: "Degraded",
        latency: "1.2s",
        uptime: "98.5%",
    },
    SystemComponent {
        name: "AI Course Generator",
        status: "Operational",
        latency: "3.4s",
        uptime: "99.8%",
    },
    SystemComponent {
        name: "Notification Service",
        status: "Operational",
        latency: "23ms",
        uptime: "99.98%",
    },
];

struct RecentEvent {
    event: &'static str,
    user: &'static str,
    time_ago: &'static str,
    event_type: &'static str,
}

const EVENTS: &[RecentEvent] = &[
    RecentEvent {
        event: "Completed: Advanced Signal Detection",
        user: "Dr. Elena Vasquez",
        time_ago: "2m ago",
        event_type: "Completion",
    },
    RecentEvent {
        event: "Enrolled: PBRER Authoring Masterclass",
        user: "James Okonkwo",
        time_ago: "15m ago",
        event_type: "Enrollment",
    },
    RecentEvent {
        event: "Certificate issued: Signal Detection Specialist",
        user: "Dr. Thomas Richter",
        time_ago: "1h ago",
        event_type: "Certificate",
    },
    RecentEvent {
        event: "Assessment scored 94%: ICSR Processing",
        user: "Aisha Patel",
        time_ago: "2h ago",
        event_type: "Assessment",
    },
    RecentEvent {
        event: "Course published: Benefit-Risk Framework",
        user: "Admin",
        time_ago: "3h ago",
        event_type: "System",
    },
    RecentEvent {
        event: "Enrolled: Risk Management Plans",
        user: "Sarah Williams",
        time_ago: "5h ago",
        event_type: "Enrollment",
    },
];

#[component]
pub fn AcademyOperationsPage() -> impl IntoView {
    let degraded = SYSTEMS.iter().filter(|s| s.status != "Operational").count();
    let pipeline_action = ServerAction::<ExecuteCompoundingPipeline>::new();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Academy Operations"</h1>
                <p class="mt-1 text-slate-400">"Monitor enrollment, completion rates, and operational health."</p>
                <div class="mt-3">
                    <a
                        href="/academy/evidence-ledger"
                        class="inline-flex rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-3 py-2 text-[10px] font-bold text-cyan-300 hover:text-cyan-200 uppercase tracking-widest font-mono"
                    >
                        "Open Academy Evidence Ledger"
                    </a>
                </div>
            </div>

            /* Compounding pipeline controls */
            <div class="mt-6 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                <div class="flex flex-wrap items-center justify-between gap-3">
                    <div>
                        <p class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">"Compounding Pipeline"</p>
                        <p class="mt-1 text-xs text-slate-300">
                            "Run and apply accepted skill proposals directly from admin operations."
                        </p>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        <button
                            on:click=move |_| {
                                pipeline_action.dispatch(ExecuteCompoundingPipeline {
                                    mode: "full-apply".to_string()
                                });
                            }
                            class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-[10px] font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono"
                        >
                            "Run + Apply"
                        </button>
                        <button
                            on:click=move |_| {
                                pipeline_action.dispatch(ExecuteCompoundingPipeline {
                                    mode: "apply".to_string()
                                });
                            }
                            class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-3 py-2 text-[10px] font-bold text-cyan-300 hover:text-cyan-200 uppercase tracking-widest font-mono"
                        >
                            "Apply Accepted"
                        </button>
                        <button
                            on:click=move |_| {
                                pipeline_action.dispatch(ExecuteCompoundingPipeline {
                                    mode: "status".to_string()
                                });
                            }
                            class="rounded-lg border border-slate-700 bg-slate-900/70 px-3 py-2 text-[10px] font-bold text-slate-300 hover:text-white uppercase tracking-widest font-mono"
                        >
                            "Refresh Status"
                        </button>
                    </div>
                </div>

                <div class="mt-4">
                    {move || if pipeline_action.pending().get() {
                        view! {
                            <p class="text-xs font-mono text-amber-300 uppercase tracking-widest">
                                "Running pipeline..."
                            </p>
                        }.into_any()
                    } else {
                        match pipeline_action.value().get() {
                            None => view! {
                                <p class="text-xs text-slate-400">
                                    "No run yet in this session."
                                </p>
                            }.into_any(),
                            Some(Err(e)) => view! {
                                <div class="rounded-lg border border-red-500/20 bg-red-500/10 p-3">
                                    <p class="text-xs font-bold text-red-300 uppercase tracking-widest font-mono">
                                        "Execution Error"
                                    </p>
                                    <p class="mt-1 text-xs text-red-200">{e.to_string()}</p>
                                </div>
                            }.into_any(),
                            Some(Ok(res)) => {
                                let status_cls = if res.ok { "text-emerald-300" } else { "text-red-300" };
                                let stderr_present = !res.stderr.trim().is_empty();
                                view! {
                                    <div class="space-y-3">
                                        <div class="flex flex-wrap items-center gap-3 text-[10px] font-mono uppercase tracking-widest">
                                            <span class=format!("font-bold {status_cls}")>
                                                {if res.ok { "Success" } else { "Failed" }}
                                            </span>
                                            <span class="text-slate-400">{format!("Mode: {}", res.mode)}</span>
                                            <span class="text-slate-500">{format!("Exit: {}", res.exit_code)}</span>
                                            <span class="text-slate-500">{format!("At: {}", res.finished_at)}</span>
                                        </div>
                                        <div class="rounded-lg border border-slate-800 bg-slate-950/60 p-3">
                                            <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Stdout"</p>
                                            <pre class="mt-2 max-h-60 overflow-auto whitespace-pre-wrap text-[11px] text-slate-300 font-mono">{res.stdout}</pre>
                                        </div>
                                        {if stderr_present {
                                            view! {
                                                <div class="rounded-lg border border-amber-500/20 bg-amber-500/5 p-3">
                                                    <p class="text-[10px] font-bold text-amber-300 uppercase tracking-widest font-mono">"Stderr"</p>
                                                    <pre class="mt-2 max-h-40 overflow-auto whitespace-pre-wrap text-[11px] text-amber-100 font-mono">{res.stderr}</pre>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </div>

            /* KPI Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"Enrollments (30d)"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">"487"</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Completions (30d)"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">"156"</p>
                </div>
                <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-5">
                    <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest font-mono">"Drop Rate"</p>
                    <p class="text-2xl font-black text-red-400 font-mono mt-2">"8.2%"</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Avg Score"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">"79.5"</p>
                </div>
            </div>

            /* System Status */
            <h2 class="mt-8 text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">
                {if degraded == 0 {
                    "System Status \u{2014} All Operational"
                } else {
                    "System Status \u{2014} Degraded"
                }}
            </h2>
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 divide-y divide-slate-800/50">
                {SYSTEMS.iter().map(|s| {
                    let (status_cls, dot_cls) = if s.status == "Operational" {
                        ("text-emerald-400", "bg-emerald-400")
                    } else {
                        ("text-amber-400", "bg-amber-400")
                    };
                    view! {
                        <div class="flex items-center justify-between px-5 py-3">
                            <div class="flex items-center gap-3">
                                <div class=format!("h-2 w-2 rounded-full {dot_cls}")></div>
                                <span class="text-sm text-white">{s.name}</span>
                            </div>
                            <div class="flex items-center gap-6 text-xs font-mono">
                                <span class=format!("font-bold {status_cls}")>{s.status}</span>
                                <span class="text-slate-500">{s.latency}</span>
                                <span class="text-slate-500">{s.uptime}</span>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            /* Recent Activity */
            <h2 class="mt-8 text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">"Recent Activity"</h2>
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 divide-y divide-slate-800/50">
                {EVENTS.iter().map(|e| {
                    let type_cls = match e.event_type {
                        "Completion" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                        "Enrollment" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                        "Certificate" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        "Assessment" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    view! {
                        <div class="flex items-center justify-between px-5 py-3">
                            <div class="flex items-center gap-3">
                                <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {type_cls}")>{e.event_type}</span>
                                <div>
                                    <p class="text-sm text-slate-300">{e.event}</p>
                                    <p class="text-[10px] text-slate-500 font-mono">{e.user}</p>
                                </div>
                            </div>
                            <span class="text-[10px] text-slate-600 font-mono shrink-0">{e.time_ago}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
