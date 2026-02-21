//! Verifications page — EPA/CPA verification requests and sign-offs

use leptos::prelude::*;

/* ------------------------------------------------------------------ */
/*  Data model                                                         */
/* ------------------------------------------------------------------ */

#[derive(Clone, Copy)]
struct Verification {
    #[allow(dead_code)]
    id: &'static str,
    epa_id: &'static str,
    epa_name: &'static str,
    status: &'static str,
    submitted_date: &'static str,
    assessor: &'static str,
    evidence_count: u32,
    feedback: &'static str,
}

const VERIFICATIONS: &[Verification] = &[
    Verification {
        id: "VRF-001",
        epa_id: "EPA-1.1",
        epa_name: "Individual Case Safety Report Processing",
        status: "Verified",
        submitted_date: "2026-02-10",
        assessor: "Dr. K. Yamamoto",
        evidence_count: 4,
        feedback: "Excellent narrative quality. MedDRA coding accurate at PT and LLT level. Causality assessment well-reasoned with appropriate use of WHO-UMC criteria.",
    },
    Verification {
        id: "VRF-002",
        epa_id: "EPA-1.3",
        epa_name: "Signal Detection and Evaluation",
        status: "Submitted",
        submitted_date: "2026-02-12",
        assessor: "Prof. M. Chen",
        evidence_count: 3,
        feedback: "",
    },
    Verification {
        id: "VRF-003",
        epa_id: "EPA-2.1",
        epa_name: "Periodic Safety Update Report Authoring",
        status: "Pending",
        submitted_date: "2026-02-14",
        assessor: "Dr. S. Okonkwo",
        evidence_count: 5,
        feedback: "",
    },
    Verification {
        id: "VRF-004",
        epa_id: "EPA-2.4",
        epa_name: "Risk Management Plan Development",
        status: "Returned",
        submitted_date: "2026-02-08",
        assessor: "Dr. K. Yamamoto",
        evidence_count: 2,
        feedback: "Risk minimisation measures need stronger justification. Please add comparative effectiveness data and revise the pharmacovigilance plan timeline.",
    },
    Verification {
        id: "VRF-005",
        epa_id: "EPA-3.2",
        epa_name: "Benefit-Risk Assessment Communication",
        status: "Verified",
        submitted_date: "2026-02-05",
        assessor: "Prof. M. Chen",
        evidence_count: 6,
        feedback: "Strong integration of quantitative B-R framework. Visual communication of uncertainty intervals was particularly effective.",
    },
    Verification {
        id: "VRF-006",
        epa_id: "EPA-1.2",
        epa_name: "Aggregate Report Data Analysis",
        status: "Submitted",
        submitted_date: "2026-02-13",
        assessor: "Dr. A. Petrova",
        evidence_count: 4,
        feedback: "",
    },
    Verification {
        id: "VRF-007",
        epa_id: "EPA-3.1",
        epa_name: "Regulatory Submission Preparation",
        status: "Pending",
        submitted_date: "2026-02-15",
        assessor: "Dr. S. Okonkwo",
        evidence_count: 3,
        feedback: "",
    },
    Verification {
        id: "VRF-008",
        epa_id: "EPA-4.1",
        epa_name: "Cross-functional Safety Governance",
        status: "Verified",
        submitted_date: "2026-01-28",
        assessor: "Prof. M. Chen",
        evidence_count: 5,
        feedback: "Demonstrated mature understanding of safety governance structures. Committee meeting facilitation evidence was convincing.",
    },
];

/* ------------------------------------------------------------------ */
/*  Helper functions                                                   */
/* ------------------------------------------------------------------ */

fn count_by_status(status: &str) -> usize {
    VERIFICATIONS.iter().filter(|v| v.status == status).count()
}

fn status_color(status: &str) -> &'static str {
    match status {
        "Pending" => "text-amber-400 bg-amber-400/10 border-amber-400/20",
        "Submitted" => "text-cyan-400 bg-cyan-400/10 border-cyan-400/20",
        "Verified" => "text-emerald-400 bg-emerald-400/10 border-emerald-400/20",
        "Returned" => "text-red-400 bg-red-400/10 border-red-400/20",
        _ => "text-slate-400 bg-slate-400/10 border-slate-400/20",
    }
}

fn status_dot(status: &str) -> &'static str {
    match status {
        "Pending" => "bg-amber-400",
        "Submitted" => "bg-cyan-400",
        "Verified" => "bg-emerald-400",
        "Returned" => "bg-red-400",
        _ => "bg-slate-400",
    }
}

fn stat_accent(status: &str) -> &'static str {
    match status {
        "Pending" => "text-amber-400",
        "Submitted" => "text-cyan-400",
        "Verified" => "text-emerald-400",
        "Returned" => "text-red-400",
        _ => "text-slate-400",
    }
}

/* ------------------------------------------------------------------ */
/*  Page component                                                     */
/* ------------------------------------------------------------------ */

#[component]
pub fn VerificationsPage() -> impl IntoView {
    let active_filter = RwSignal::new("All");

    let filtered = move || {
        let f = active_filter.get();
        VERIFICATIONS
            .iter()
            .copied()
            .filter(|v| f == "All" || v.status == f)
            .collect::<Vec<_>>()
    };

    let pending = count_by_status("Pending");
    let submitted = count_by_status("Submitted");
    let verified = count_by_status("Verified");
    let returned = count_by_status("Returned");

    view! {
        <div class="space-y-8">
            /* --- Header --- */
            <div>
                <h1 class="text-2xl font-bold text-white font-mono uppercase tracking-tight">"Verifications"</h1>
                <p class="text-slate-400 mt-1 text-sm leading-relaxed">
                    "Track EPA and CPA verification requests, evidence submissions, and assessor sign-offs"
                </p>
            </div>

            /* --- Stats row --- */
            <div class="grid gap-4 sm:grid-cols-4">
                <StatCard label="PENDING" value=pending color=stat_accent("Pending") icon="\u{23F3}"/>
                <StatCard label="SUBMITTED" value=submitted color=stat_accent("Submitted") icon="\u{1F4E4}"/>
                <StatCard label="VERIFIED" value=verified color=stat_accent("Verified") icon="\u{2705}"/>
                <StatCard label="RETURNED" value=returned color=stat_accent("Returned") icon="\u{21A9}"/>
            </div>

            /* --- Filter tabs --- */
            <div class="flex items-center gap-1 bg-slate-900/50 border border-slate-800 rounded-lg p-1 w-fit">
                {["All", "Pending", "Submitted", "Verified", "Returned"]
                    .into_iter()
                    .map(|tab| {
                        let is_active = move || active_filter.get() == tab;
                        view! {
                            <button
                                class=move || {
                                    if is_active() {
                                        "px-4 py-1.5 rounded-md text-xs font-mono font-bold uppercase tracking-widest bg-slate-700/80 text-white transition-all"
                                    } else {
                                        "px-4 py-1.5 rounded-md text-xs font-mono font-bold uppercase tracking-widest text-slate-500 hover:text-slate-300 transition-all"
                                    }
                                }
                                on:click=move |_| active_filter.set(tab)
                            >
                                {tab}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            /* --- Data table --- */
            <div class="bg-slate-900/50 border border-slate-800 rounded-xl overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full text-sm">
                        <thead>
                            <tr class="border-b border-slate-800">
                                <th class="text-left px-4 py-3 text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"EPA ID"</th>
                                <th class="text-left px-4 py-3 text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"EPA Name"</th>
                                <th class="text-left px-4 py-3 text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"Status"</th>
                                <th class="text-left px-4 py-3 text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"Submitted"</th>
                                <th class="text-left px-4 py-3 text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"Assessor"</th>
                                <th class="text-center px-4 py-3 text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"Evidence"</th>
                                <th class="text-right px-4 py-3 text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let rows = filtered();
                                if rows.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="7" class="px-4 py-12 text-center text-slate-500 font-mono text-xs">
                                                "No verification requests match the current filter."
                                            </td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    rows.into_iter().map(|v| {
                                        let badge_class = format!(
                                            "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-mono font-bold uppercase tracking-widest border {}",
                                            status_color(v.status)
                                        );
                                        let dot_class = format!("w-1.5 h-1.5 rounded-full {}", status_dot(v.status));
                                        let action_label = match v.status {
                                            "Pending" => "Submit",
                                            "Submitted" => "View",
                                            "Verified" => "Certificate",
                                            "Returned" => "Revise",
                                            _ => "View",
                                        };
                                        view! {
                                            <tr class="border-b border-slate-800/50 hover:bg-slate-800/30 transition-colors">
                                                <td class="px-4 py-3 font-mono text-cyan-400 text-xs font-bold">{v.epa_id}</td>
                                                <td class="px-4 py-3 text-slate-300 text-sm">{v.epa_name}</td>
                                                <td class="px-4 py-3">
                                                    <span class=badge_class>
                                                        <span class=dot_class></span>
                                                        {v.status}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-slate-400 text-xs font-mono">{v.submitted_date}</td>
                                                <td class="px-4 py-3 text-slate-300 text-xs">{v.assessor}</td>
                                                <td class="px-4 py-3 text-center">
                                                    <span class="text-violet-400 font-mono font-bold text-xs">{v.evidence_count.to_string()}</span>
                                                </td>
                                                <td class="px-4 py-3 text-right">
                                                    <button class="text-[10px] font-mono font-bold uppercase tracking-widest text-cyan-400 hover:text-cyan-300 transition-colors">
                                                        {action_label}
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                </div>

                /* --- Table footer --- */
                <div class="border-t border-slate-800 px-4 py-3 flex items-center justify-between">
                    <p class="text-[9px] font-mono text-slate-500 uppercase tracking-widest">
                        {move || format!("{} of {} verifications shown", filtered().len(), VERIFICATIONS.len())}
                    </p>
                    <p class="text-[9px] font-mono text-slate-600 uppercase tracking-widest">
                        "Last synced: 2026-02-15 09:42 UTC"
                    </p>
                </div>
            </div>

            /* --- Verification process --- */
            <div>
                <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-4">"// VERIFICATION PROCESS"</h2>
                <div class="grid gap-4 lg:grid-cols-4">
                    <StepCard step=1 title="Complete Activity" desc="Perform the EPA activity in your workplace or simulation environment"/>
                    <StepCard step=2 title="Gather Evidence" desc="Collect documentation, reflections, and supervisor observations"/>
                    <StepCard step=3 title="Submit for Review" desc="Upload evidence and request assessor verification"/>
                    <StepCard step=4 title="Receive Feedback" desc="Assessor reviews, provides feedback, and signs off on competency"/>
                </div>
            </div>

            /* --- Recent feedback --- */
            <div>
                <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-4">"// RECENT FEEDBACK"</h2>
                <div class="grid gap-4 lg:grid-cols-3">
                    {VERIFICATIONS
                        .iter()
                        .filter(|v| !v.feedback.is_empty())
                        .take(3)
                        .map(|v| {
                            view! {
                                <FeedbackCard
                                    epa_id=v.epa_id
                                    epa_name=v.epa_name
                                    assessor=v.assessor
                                    status=v.status
                                    feedback=v.feedback
                                />
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Sub-components                                                     */
/* ------------------------------------------------------------------ */

#[component]
fn StatCard(
    #[prop(into)] label: &'static str,
    value: usize,
    #[prop(into)] color: &'static str,
    #[prop(into)] icon: &'static str,
) -> impl IntoView {
    view! {
        <div class="bg-slate-900/50 border border-slate-800 rounded-xl p-5 group hover:border-slate-700 transition-colors">
            <div class="flex items-center justify-between mb-2">
                <span class="text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest">{label}</span>
                <span class="text-lg opacity-50 group-hover:opacity-80 transition-opacity">{icon}</span>
            </div>
            <div class={format!("text-3xl font-bold font-mono {}", color)}>{value.to_string()}</div>
        </div>
    }
}

#[component]
fn StepCard(step: u32, #[prop(into)] title: String, #[prop(into)] desc: String) -> impl IntoView {
    view! {
        <div class="bg-slate-900/50 border border-slate-800 rounded-xl p-5 relative overflow-hidden group">
            /* --- Step number background --- */
            <div class="absolute -right-2 -top-2 text-6xl font-black font-mono text-slate-800/30 select-none group-hover:text-slate-700/30 transition-colors">
                {step.to_string()}
            </div>
            <div class="relative">
                <div class="w-8 h-8 rounded-full bg-cyan-600/20 text-cyan-400 flex items-center justify-center text-sm font-bold font-mono mb-3">
                    {step.to_string()}
                </div>
                <h3 class="text-white font-semibold text-sm mb-1">{title}</h3>
                <p class="text-slate-400 text-xs leading-relaxed">{desc}</p>
            </div>
        </div>
    }
}

#[component]
fn FeedbackCard(
    #[prop(into)] epa_id: &'static str,
    #[prop(into)] epa_name: &'static str,
    #[prop(into)] assessor: &'static str,
    #[prop(into)] status: &'static str,
    #[prop(into)] feedback: &'static str,
) -> impl IntoView {
    let border_accent = match status {
        "Verified" => "border-l-emerald-500/60",
        "Returned" => "border-l-red-500/60",
        _ => "border-l-cyan-500/60",
    };
    let card_class = format!(
        "bg-slate-900/50 border border-slate-800 border-l-2 {} rounded-xl p-5 space-y-3",
        border_accent
    );
    let badge_class = format!(
        "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[9px] font-mono font-bold uppercase tracking-widest border {}",
        status_color(status)
    );
    let dot_class = format!("w-1.5 h-1.5 rounded-full {}", status_dot(status));

    view! {
        <div class=card_class>
            <div class="flex items-start justify-between gap-2">
                <div>
                    <span class="text-cyan-400 font-mono font-bold text-xs">{epa_id}</span>
                    <span class="text-slate-600 mx-1.5">{"\u{2022}"}</span>
                    <span class="text-slate-300 text-xs">{epa_name}</span>
                </div>
                <span class=badge_class>
                    <span class=dot_class></span>
                    {status}
                </span>
            </div>
            <blockquote class="text-slate-400 text-xs leading-relaxed italic border-l-2 border-slate-700 pl-3">
                {feedback}
            </blockquote>
            <div class="flex items-center gap-2 pt-1">
                <div class="w-5 h-5 rounded-full bg-violet-600/20 text-violet-400 flex items-center justify-center text-[9px] font-bold font-mono">
                    {assessor.chars().next().map(|c| c.to_string()).unwrap_or_default()}
                </div>
                <span class="text-[10px] font-mono text-slate-500">{assessor}</span>
            </div>
        </div>
    }
}
