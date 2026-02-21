//! Admin: Quiz session leads — users who started assessments from public pages

use leptos::prelude::*;

struct QuizSession {
    date: &'static str,
    quiz: &'static str,
    score: Option<u8>,
    email: Option<&'static str>,
    converted: bool,
    time_spent: &'static str,
}

const SESSIONS: &[QuizSession] = &[
    QuizSession {
        date: "2026-02-15",
        quiz: "PV Knowledge Check",
        score: Some(82),
        email: Some("k.weber@pharma-eu.de"),
        converted: true,
        time_spent: "8m 34s",
    },
    QuizSession {
        date: "2026-02-15",
        quiz: "Signal Detection Basics",
        score: Some(91),
        email: Some("j.liu@meditech.com"),
        converted: true,
        time_spent: "6m 12s",
    },
    QuizSession {
        date: "2026-02-15",
        quiz: "PV Knowledge Check",
        score: Some(67),
        email: None,
        converted: false,
        time_spent: "12m 45s",
    },
    QuizSession {
        date: "2026-02-14",
        quiz: "Regulatory IQ Test",
        score: Some(88),
        email: Some("r.gupta@safepharma.in"),
        converted: true,
        time_spent: "9m 08s",
    },
    QuizSession {
        date: "2026-02-14",
        quiz: "Signal Detection Basics",
        score: None,
        email: None,
        converted: false,
        time_spent: "2m 15s",
    },
    QuizSession {
        date: "2026-02-14",
        quiz: "ICSR Processing Quiz",
        score: Some(74),
        email: Some("a.kowalski@pvteam.pl"),
        converted: false,
        time_spent: "7m 33s",
    },
    QuizSession {
        date: "2026-02-13",
        quiz: "PV Knowledge Check",
        score: Some(95),
        email: Some("m.thompson@biotrial.co.uk"),
        converted: true,
        time_spent: "5m 48s",
    },
    QuizSession {
        date: "2026-02-13",
        quiz: "Causality Assessment",
        score: Some(79),
        email: None,
        converted: false,
        time_spent: "11m 22s",
    },
    QuizSession {
        date: "2026-02-13",
        quiz: "Regulatory IQ Test",
        score: None,
        email: None,
        converted: false,
        time_spent: "1m 03s",
    },
    QuizSession {
        date: "2026-02-12",
        quiz: "Signal Detection Basics",
        score: Some(86),
        email: Some("s.martin@agence-med.fr"),
        converted: true,
        time_spent: "7m 19s",
    },
];

#[component]
pub fn LeadsQuizSessionsPage() -> impl IntoView {
    let completed = SESSIONS.iter().filter(|s| s.score.is_some()).count();
    let emails_captured = SESSIONS.iter().filter(|s| s.email.is_some()).count();
    let converted = SESSIONS.iter().filter(|s| s.converted).count();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Quiz Session Leads"</h1>
                    <p class="mt-1 text-slate-400">"Users who started public assessments \u{2014} potential conversion targets."</p>
                </div>
                <a href="/admin/leads" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} All Leads"</a>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 md:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Sessions"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{SESSIONS.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Completed"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{completed.to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Emails Captured"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{emails_captured.to_string()}</p>
                </div>
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"Converted"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{converted.to_string()}</p>
                </div>
            </div>

            /* Sessions table */
            <div class="mt-8 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Date"</th>
                            <th class="px-4 py-3">"Quiz"</th>
                            <th class="px-4 py-3 text-right">"Score"</th>
                            <th class="px-4 py-3">"Time"</th>
                            <th class="px-4 py-3">"Email"</th>
                            <th class="px-4 py-3 text-center">"Converted"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {SESSIONS.iter().map(|s| {
                            let score_text = s.score.map(|v| format!("{}%", v)).unwrap_or_else(|| "DNF".to_string());
                            let score_cls = match s.score {
                                Some(v) if v >= 80 => "text-emerald-400",
                                Some(_) => "text-amber-400",
                                None => "text-red-400",
                            };
                            view! {
                                <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono">{s.date}</td>
                                    <td class="px-4 py-3 text-sm text-white">{s.quiz}</td>
                                    <td class="px-4 py-3 text-right">
                                        <span class=format!("text-xs font-bold font-mono {score_cls}")>{score_text}</span>
                                    </td>
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono">{s.time_spent}</td>
                                    <td class="px-4 py-3 text-[10px] text-slate-500 font-mono">{s.email.unwrap_or("\u{2014}")}</td>
                                    <td class="px-4 py-3 text-center">
                                        {if s.converted {
                                            view! { <span class="text-emerald-400 text-xs font-bold font-mono">"\u{2713}"</span> }.into_any()
                                        } else {
                                            view! { <span class="text-slate-600 text-xs font-mono">"\u{2014}"</span> }.into_any()
                                        }}
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            <div class="mt-4 flex items-center justify-between text-[10px] text-slate-600 font-mono">
                <span>{format!("{} sessions \u{00B7} {}% completion rate", SESSIONS.len(), completed * 100 / SESSIONS.len())}</span>
                <span>{format!("Conversion rate: {}%", converted * 100 / SESSIONS.len())}</span>
            </div>
        </div>
    }
}
