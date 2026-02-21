//! Biological Telemetry Dashboard — real-time view of the 8 NexCore biological crates
//!
//! Surfaces cytokine signaling, hormone levels, immune status, energy metabolism,
//! synapse learning, transcriptase inference, ribosome codegen, and phenotype mutation.
//!
//! Each crate maps to a biological analog for system-wide organismic health monitoring.

use leptos::prelude::*;

/* ── Static telemetry data ── */

#[derive(Clone)]
struct BioSystem {
    name: &'static str,
    crate_name: &'static str,
    biology: &'static str,
    icon: &'static str,
    status: &'static str,
    color: &'static str,
    mcp_tools: u8,
    description: &'static str,
    metrics: &'static [Metric],
}

#[derive(Clone)]
struct Metric {
    label: &'static str,
    value: &'static str,
    unit: &'static str,
    trend: &'static str,
}

const SYSTEMS: &[BioSystem] = &[
    BioSystem {
        name: "Cytokine",
        crate_name: "nexcore-cytokine",
        biology: "Inter-cell signaling",
        icon: "\u{1F9EC}",
        status: "Active",
        color: "cyan",
        mcp_tools: 5,
        description: "Event-driven inter-crate communication via cytokine families: IL (interleukin), TNF (tumor necrosis factor), IFN (interferon), TGF (transforming growth factor), CSF (colony-stimulating factor).",
        metrics: &[
            Metric { label: "Events Emitted", value: "1,247", unit: "total", trend: "up" },
            Metric { label: "Active Families", value: "5", unit: "of 5", trend: "stable" },
            Metric { label: "IL Signals", value: "423", unit: "events", trend: "up" },
            Metric { label: "TNF Signals", value: "189", unit: "events", trend: "stable" },
            Metric { label: "CSF Signals", value: "312", unit: "events", trend: "up" },
            Metric { label: "Queue Depth", value: "0", unit: "pending", trend: "stable" },
        ],
    },
    BioSystem {
        name: "Hormones",
        crate_name: "nexcore-hormones",
        biology: "System-wide configuration",
        icon: "\u{1F9EA}",
        status: "Active",
        color: "purple",
        mcp_tools: 4,
        description: "Slow-acting global configuration propagation with 6 hormone types. Each decays toward baseline over time, providing system-wide state without tight coupling.",
        metrics: &[
            Metric { label: "Hormone Types", value: "6", unit: "active", trend: "stable" },
            Metric { label: "Cortisol Level", value: "0.35", unit: "normalized", trend: "down" },
            Metric { label: "Thyroid Output", value: "0.72", unit: "normalized", trend: "stable" },
            Metric { label: "Adrenaline", value: "0.12", unit: "normalized", trend: "down" },
            Metric { label: "Decay Rate", value: "0.05", unit: "per tick", trend: "stable" },
            Metric { label: "Config Pushes", value: "89", unit: "total", trend: "up" },
        ],
    },
    BioSystem {
        name: "Immunity",
        crate_name: "nexcore-immunity",
        biology: "Antipattern detection",
        icon: "\u{1F6E1}",
        status: "Active",
        color: "red",
        mcp_tools: 5,
        description: "PAMP (pathogen-associated molecular pattern) and DAMP (damage-associated) detection. Maintains an antibody registry for known antipatterns with adaptive learning.",
        metrics: &[
            Metric { label: "Antibodies", value: "8", unit: "registered", trend: "up" },
            Metric { label: "PAMPs Detected", value: "23", unit: "total", trend: "up" },
            Metric { label: "DAMPs Detected", value: "7", unit: "total", trend: "stable" },
            Metric { label: "False Positives", value: "2", unit: "total", trend: "down" },
            Metric { label: "Scan Coverage", value: "94%", unit: "", trend: "up" },
            Metric { label: "Last Scan", value: "2m ago", unit: "", trend: "stable" },
        ],
    },
    BioSystem {
        name: "Energy",
        crate_name: "nexcore-energy",
        biology: "ATP/ADP token budget",
        icon: "\u{26A1}",
        status: "Active",
        color: "amber",
        mcp_tools: 4,
        description: "Token budget management via ATP/ADP energy model. EC = (ATP + 0.5*ADP) / total. Four metabolic regimes: anabolic, catabolic, balanced, starvation.",
        metrics: &[
            Metric { label: "Energy Charge", value: "0.78", unit: "EC", trend: "stable" },
            Metric { label: "ATP Available", value: "45,200", unit: "tokens", trend: "down" },
            Metric { label: "ADP Pool", value: "12,400", unit: "tokens", trend: "up" },
            Metric { label: "Metabolic Regime", value: "Balanced", unit: "", trend: "stable" },
            Metric { label: "Burn Rate", value: "340", unit: "tokens/min", trend: "stable" },
            Metric { label: "Efficiency", value: "92%", unit: "", trend: "up" },
        ],
    },
    BioSystem {
        name: "Synapse",
        crate_name: "nexcore-synapse",
        biology: "Learning connections",
        icon: "\u{1F9E0}",
        status: "Active",
        color: "emerald",
        mcp_tools: 8,
        description: "Learning system with amplitude growth following Michaelis-Menten saturation kinetics. Synapses strengthen with repeated stimulation, decay without it.",
        metrics: &[
            Metric { label: "Total Synapses", value: "156", unit: "connections", trend: "up" },
            Metric { label: "Strong (>0.8)", value: "34", unit: "synapses", trend: "up" },
            Metric { label: "Weak (<0.2)", value: "18", unit: "synapses", trend: "down" },
            Metric { label: "Avg Amplitude", value: "0.54", unit: "normalized", trend: "up" },
            Metric { label: "Learning Rate", value: "0.12", unit: "V_max", trend: "stable" },
            Metric { label: "Pruned Today", value: "3", unit: "synapses", trend: "stable" },
        ],
    },
    BioSystem {
        name: "Transcriptase",
        crate_name: "nexcore-transcriptase",
        biology: "Schema inference",
        icon: "\u{1F52C}",
        status: "Active",
        color: "blue",
        mcp_tools: 4,
        description: "Reverse transcriptase: infers structured schemas from unstructured JSON data. Like biological RT converts RNA back to DNA, this converts data into type definitions.",
        metrics: &[
            Metric { label: "Schemas Inferred", value: "47", unit: "total", trend: "up" },
            Metric { label: "Fields Mapped", value: "812", unit: "total", trend: "up" },
            Metric { label: "Type Accuracy", value: "96%", unit: "", trend: "stable" },
            Metric { label: "Nullable Detected", value: "134", unit: "fields", trend: "up" },
            Metric { label: "Enum Candidates", value: "23", unit: "detected", trend: "stable" },
            Metric { label: "Last Inference", value: "14m ago", unit: "", trend: "stable" },
        ],
    },
    BioSystem {
        name: "Ribosome",
        crate_name: "nexcore-ribosome",
        biology: "Schema-to-code generation",
        icon: "\u{1F3ED}",
        status: "Active",
        color: "orange",
        mcp_tools: 6,
        description: "Translates inferred schemas into executable Rust code. Detects drift between schema and generated code. Like biological ribosomes translate mRNA into proteins.",
        metrics: &[
            Metric { label: "Code Generated", value: "31", unit: "modules", trend: "up" },
            Metric { label: "Drift Detected", value: "2", unit: "modules", trend: "down" },
            Metric { label: "Lines Output", value: "4,280", unit: "LoC", trend: "up" },
            Metric { label: "Type Coverage", value: "98%", unit: "", trend: "stable" },
            Metric { label: "Regenerations", value: "7", unit: "total", trend: "stable" },
            Metric { label: "Compile Pass", value: "100%", unit: "", trend: "stable" },
        ],
    },
    BioSystem {
        name: "Phenotype",
        crate_name: "nexcore-phenotype",
        biology: "Adversarial mutation",
        icon: "\u{1F9EC}",
        status: "Active",
        color: "rose",
        mcp_tools: 1,
        description: "Adversarial test generation via 7 mutation types: boundary, null injection, type coercion, overflow, encoding, timing, and state corruption. Phenotypic expression of code under stress.",
        metrics: &[
            Metric { label: "Mutation Types", value: "7", unit: "of 7", trend: "stable" },
            Metric { label: "Tests Generated", value: "89", unit: "total", trend: "up" },
            Metric { label: "Bugs Found", value: "12", unit: "total", trend: "up" },
            Metric { label: "Survival Rate", value: "86%", unit: "", trend: "up" },
            Metric { label: "Last Run", value: "8m ago", unit: "", trend: "stable" },
            Metric { label: "Coverage", value: "73%", unit: "of APIs", trend: "up" },
        ],
    },
];

/* ── Event log data ── */

#[derive(Clone)]
struct BioEvent {
    time: &'static str,
    system: &'static str,
    event_type: &'static str,
    message: &'static str,
    severity: &'static str,
}

const EVENTS: &[BioEvent] = &[
    BioEvent { time: "14:23:01", system: "Cytokine", event_type: "CSF", message: "Task completion signal emitted (build succeeded)", severity: "info" },
    BioEvent { time: "14:22:47", system: "Guardian", event_type: "SENSE", message: "Homeostasis tick #847 completed (8 signals, 6ms)", severity: "info" },
    BioEvent { time: "14:22:15", system: "Immunity", event_type: "PAMP", message: "Detected unwrap() in nexcore-api test module", severity: "warn" },
    BioEvent { time: "14:21:58", system: "Energy", event_type: "ATP", message: "Energy charge stable at 0.78 (balanced regime)", severity: "info" },
    BioEvent { time: "14:21:30", system: "Synapse", event_type: "GROW", message: "Synapse 'leptos-view-pattern' strengthened to 0.82", severity: "info" },
    BioEvent { time: "14:20:45", system: "Hormones", event_type: "DECAY", message: "Cortisol decaying toward baseline (0.35 \u{2192} 0.33)", severity: "info" },
    BioEvent { time: "14:20:12", system: "Transcriptase", event_type: "INFER", message: "Schema inferred for FAERS drug_event response (14 fields)", severity: "info" },
    BioEvent { time: "14:19:33", system: "Phenotype", event_type: "MUTATE", message: "Boundary mutation found overflow in pagination limit", severity: "warn" },
    BioEvent { time: "14:19:01", system: "Ribosome", event_type: "DRIFT", message: "Drift detected: FaersResponse struct missing new field", severity: "warn" },
    BioEvent { time: "14:18:44", system: "Cytokine", event_type: "IL", message: "Inter-crate event: vigilance \u{2192} guardian (signal threshold)", severity: "info" },
    BioEvent { time: "14:18:20", system: "Immunity", event_type: "ANTIBODY", message: "Antibody #8 registered: python-file-creation pattern", severity: "info" },
    BioEvent { time: "14:17:55", system: "Energy", event_type: "ADP", message: "Token spend: 1,240 tokens on FAERS analysis", severity: "info" },
];

/* ── Organismic summary ── */

#[derive(Clone)]
struct VitalSign {
    label: &'static str,
    value: &'static str,
    status: &'static str,
    description: &'static str,
}

const VITALS: &[VitalSign] = &[
    VitalSign { label: "Overall Health", value: "HEALTHY", status: "green", description: "All 8 biological systems operational" },
    VitalSign { label: "Energy Charge", value: "0.78", status: "green", description: "Balanced metabolic regime (>0.6 = healthy)" },
    VitalSign { label: "Immune Load", value: "LOW", status: "green", description: "8 antibodies, 2 active threats" },
    VitalSign { label: "Learning Rate", value: "GROWING", status: "green", description: "156 synapses, avg amplitude rising" },
    VitalSign { label: "Signal Integrity", value: "99.8%", status: "green", description: "1,247 events, 0 dropped" },
    VitalSign { label: "Code Drift", value: "2 modules", status: "amber", description: "Ribosome detected schema-code divergence" },
];

/* ── UI ── */

#[component]
pub fn TelemetryPage() -> impl IntoView {
    let selected_system = RwSignal::new(Option::<usize>::None);

    let total_tools: u8 = SYSTEMS.iter().map(|s| s.mcp_tools).sum();

    view! {
        <div class="mx-auto max-w-7xl px-4 py-8">
            <header class="mb-6">
                <h1 class="text-3xl font-black text-white font-mono uppercase tracking-tight">"Biological Telemetry"</h1>
                <p class="mt-2 text-slate-400 max-w-3xl">
                    "Real-time health monitoring across 8 biological crates. Each crate models a biological system for organismic software architecture."
                </p>
            </header>

            /* Vital signs bar */
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3 mb-6">
                {VITALS.iter().map(|v| {
                    let label = v.label;
                    let value = v.value;
                    let desc = v.description;
                    let border_color = match v.status {
                        "green" => "border-emerald-500/30",
                        "amber" => "border-amber-500/30",
                        "red" => "border-red-500/30",
                        _ => "border-slate-700",
                    };
                    let value_color = match v.status {
                        "green" => "text-emerald-400",
                        "amber" => "text-amber-400",
                        "red" => "text-red-400",
                        _ => "text-slate-400",
                    };
                    view! {
                        <div class=format!("rounded-lg border bg-slate-900/50 p-3 {border_color}") title=desc>
                            <p class="text-[8px] font-bold text-slate-600 uppercase tracking-widest">{label}</p>
                            <p class=format!("text-sm font-black font-mono mt-0.5 {value_color}")>{value}</p>
                        </div>
                    }
                }).collect_view()}
            </div>

            /* System overview stat bar */
            <div class="flex items-center gap-6 mb-6 px-1">
                <span class="text-[9px] font-bold text-slate-500 font-mono uppercase">"8 BIO CRATES"</span>
                <span class="text-[9px] text-slate-600 font-mono">{format!("{total_tools} MCP TOOLS")}</span>
                <span class="text-[9px] text-emerald-500 font-mono">"ALL SYSTEMS OPERATIONAL"</span>
            </div>

            /* System grid */
            <div class="grid md:grid-cols-2 xl:grid-cols-4 gap-4 mb-8">
                {SYSTEMS.iter().enumerate().map(|(idx, sys)| {
                    let name = sys.name;
                    let icon = sys.icon;
                    let biology = sys.biology;
                    let crate_name = sys.crate_name;
                    let tools = sys.mcp_tools;
                    let color = sys.color;

                    let border_class = match color {
                        "cyan" => "border-cyan-500/20 hover:border-cyan-500/40",
                        "purple" => "border-purple-500/20 hover:border-purple-500/40",
                        "red" => "border-red-500/20 hover:border-red-500/40",
                        "amber" => "border-amber-500/20 hover:border-amber-500/40",
                        "emerald" => "border-emerald-500/20 hover:border-emerald-500/40",
                        "blue" => "border-blue-500/20 hover:border-blue-500/40",
                        "orange" => "border-orange-500/20 hover:border-orange-500/40",
                        "rose" => "border-rose-500/20 hover:border-rose-500/40",
                        _ => "border-slate-700",
                    };
                    let name_color = match color {
                        "cyan" => "text-cyan-400",
                        "purple" => "text-purple-400",
                        "red" => "text-red-400",
                        "amber" => "text-amber-400",
                        "emerald" => "text-emerald-400",
                        "blue" => "text-blue-400",
                        "orange" => "text-orange-400",
                        "rose" => "text-rose-400",
                        _ => "text-slate-400",
                    };

                    let card_class = format!("rounded-xl border bg-slate-900/50 p-4 cursor-pointer transition-all {border_class}");

                    view! {
                        <div
                            class=card_class
                            on:click=move |_| {
                                let current = selected_system.get();
                                if current == Some(idx) {
                                    selected_system.set(None);
                                } else {
                                    selected_system.set(Some(idx));
                                }
                            }
                        >
                            <div class="flex items-center justify-between mb-2">
                                <div class="flex items-center gap-2">
                                    <span class="text-lg">{icon}</span>
                                    <span class=format!("text-sm font-black font-mono {name_color}")>{name}</span>
                                </div>
                                <span class="text-[8px] font-bold text-emerald-500 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">"ACTIVE"</span>
                            </div>
                            <p class="text-[10px] text-slate-500 mb-2">{biology}</p>
                            <div class="flex items-center justify-between">
                                <span class="text-[8px] font-mono text-slate-600">{crate_name}</span>
                                <span class="text-[8px] font-mono text-slate-500">{format!("{tools} tools")}</span>
                            </div>

                            /* Mini metrics (first 3) */
                            <div class="mt-3 grid grid-cols-3 gap-2">
                                {sys.metrics.iter().take(3).map(|m| {
                                    let label = m.label;
                                    let value = m.value;
                                    let trend_icon = match m.trend {
                                        "up" => "\u{2191}",
                                        "down" => "\u{2193}",
                                        _ => "\u{2022}",
                                    };
                                    let trend_color = match m.trend {
                                        "up" => "text-emerald-500",
                                        "down" => "text-red-400",
                                        _ => "text-slate-600",
                                    };
                                    view! {
                                        <div>
                                            <p class="text-[7px] text-slate-600 uppercase truncate">{label}</p>
                                            <p class="text-[10px] font-bold text-slate-300 font-mono">
                                                {value}
                                                <span class=format!("ml-0.5 text-[8px] {trend_color}")>{trend_icon}</span>
                                            </p>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            /* Detail panel (expanded system) */
            {move || selected_system.get().map(|idx| {
                let sys = &SYSTEMS[idx];
                let name = sys.name.to_string();
                let crate_name = sys.crate_name.to_string();
                let description = sys.description.to_string();
                let tools = sys.mcp_tools;
                let color = sys.color;

                let border_class = match color {
                    "cyan" => "border-cyan-500/30",
                    "purple" => "border-purple-500/30",
                    "red" => "border-red-500/30",
                    "amber" => "border-amber-500/30",
                    "emerald" => "border-emerald-500/30",
                    "blue" => "border-blue-500/30",
                    "orange" => "border-orange-500/30",
                    "rose" => "border-rose-500/30",
                    _ => "border-slate-700",
                };
                let header_color = match color {
                    "cyan" => "text-cyan-400",
                    "purple" => "text-purple-400",
                    "red" => "text-red-400",
                    "amber" => "text-amber-400",
                    "emerald" => "text-emerald-400",
                    "blue" => "text-blue-400",
                    "orange" => "text-orange-400",
                    "rose" => "text-rose-400",
                    _ => "text-slate-400",
                };

                view! {
                    <div class=format!("rounded-xl border bg-slate-900/50 p-6 mb-8 {border_class}")>
                        <div class="flex items-center justify-between mb-4">
                            <div>
                                <h2 class=format!("text-lg font-black font-mono {header_color}")>{name}</h2>
                                <p class="text-[10px] text-slate-500 font-mono">{crate_name} " \u{2022} " {format!("{tools} MCP tools")}</p>
                            </div>
                            <button
                                on:click=move |_| selected_system.set(None)
                                class="text-slate-500 hover:text-white text-xs font-mono"
                            >"\u{2715} Close"</button>
                        </div>

                        <p class="text-xs text-slate-400 leading-relaxed mb-5">{description}</p>

                        /* Full metrics grid */
                        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
                            {sys.metrics.iter().map(|m| {
                                let label = m.label;
                                let value = m.value;
                                let unit = m.unit;
                                let trend_icon = match m.trend {
                                    "up" => "\u{2191}",
                                    "down" => "\u{2193}",
                                    _ => "\u{2022}",
                                };
                                let trend_color = match m.trend {
                                    "up" => "text-emerald-500",
                                    "down" => "text-red-400",
                                    _ => "text-slate-600",
                                };
                                view! {
                                    <div class="rounded-lg bg-slate-950 border border-slate-800 p-3">
                                        <p class="text-[7px] font-bold text-slate-600 uppercase tracking-widest">{label}</p>
                                        <p class="text-sm font-black text-white font-mono mt-0.5">
                                            {value}
                                            <span class=format!("ml-1 text-[9px] {trend_color}")>{trend_icon}</span>
                                        </p>
                                        <p class="text-[8px] text-slate-600 font-mono">{unit}</p>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }
            })}

            /* Event log */
            <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                <h3 class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-4">"Biological Event Log (Last 15 min)"</h3>
                <div class="space-y-1">
                    {EVENTS.iter().map(|e| {
                        let time = e.time;
                        let system = e.system;
                        let event_type = e.event_type;
                        let message = e.message;
                        let severity_color = match e.severity {
                            "warn" => "text-amber-400",
                            "error" => "text-red-400",
                            _ => "text-slate-500",
                        };
                        let system_color = match e.system {
                            "Cytokine" => "text-cyan-400",
                            "Hormones" => "text-purple-400",
                            "Immunity" => "text-red-400",
                            "Energy" => "text-amber-400",
                            "Synapse" => "text-emerald-400",
                            "Transcriptase" => "text-blue-400",
                            "Ribosome" => "text-orange-400",
                            "Phenotype" => "text-rose-400",
                            "Guardian" => "text-yellow-400",
                            _ => "text-slate-400",
                        };

                        view! {
                            <div class="flex items-center gap-3 py-1.5 border-b border-slate-800/50 last:border-0">
                                <span class="text-[9px] font-mono text-slate-600 w-16 flex-shrink-0">{time}</span>
                                <span class=format!("text-[9px] font-bold font-mono w-24 flex-shrink-0 {system_color}")>{system}</span>
                                <span class=format!("text-[8px] font-bold font-mono px-1.5 py-0.5 rounded bg-slate-950 border border-slate-800 w-14 text-center flex-shrink-0 {severity_color}")>{event_type}</span>
                                <span class="text-[10px] text-slate-400 font-mono truncate">{message}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </section>
        </div>
    }
}
