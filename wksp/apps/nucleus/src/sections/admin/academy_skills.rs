//! Admin: Academy skills — manage skill taxonomy and assessment criteria

use leptos::prelude::*;

struct SkillCategory {
    name: &'static str,
    skills: u32,
    assessments: u32,
    avg_proficiency: u8,
    color: &'static str,
}

const CATEGORIES: &[SkillCategory] = &[
    SkillCategory {
        name: "Signal Detection",
        skills: 28,
        assessments: 6,
        avg_proficiency: 72,
        color: "text-red-400",
    },
    SkillCategory {
        name: "Case Processing",
        skills: 24,
        assessments: 5,
        avg_proficiency: 78,
        color: "text-cyan-400",
    },
    SkillCategory {
        name: "Aggregate Reporting",
        skills: 22,
        assessments: 4,
        avg_proficiency: 68,
        color: "text-amber-400",
    },
    SkillCategory {
        name: "Risk Management",
        skills: 20,
        assessments: 3,
        avg_proficiency: 65,
        color: "text-violet-400",
    },
    SkillCategory {
        name: "Benefit-Risk",
        skills: 18,
        assessments: 3,
        avg_proficiency: 71,
        color: "text-emerald-400",
    },
    SkillCategory {
        name: "Regulatory Affairs",
        skills: 26,
        assessments: 5,
        avg_proficiency: 74,
        color: "text-blue-400",
    },
    SkillCategory {
        name: "PV Systems",
        skills: 18,
        assessments: 3,
        avg_proficiency: 69,
        color: "text-orange-400",
    },
];

#[component]
pub fn AcademySkillsAdminPage() -> impl IntoView {
    let total_skills: u32 = CATEGORIES.iter().map(|c| c.skills).sum();
    let total_assessments: u32 = CATEGORIES.iter().map(|c| c.assessments).sum();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Skills Administration"</h1>
                <p class="mt-1 text-slate-400">"Manage the PV skill taxonomy, assessment criteria, and proficiency levels."</p>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Skills"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{total_skills.to_string()}</p>
                </div>
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"Categories"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{CATEGORIES.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Assessments"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{total_assessments.to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Proficiency Levels"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">"5"</p>
                </div>
            </div>

            /* Proficiency Levels reference */
            <div class="mt-6 flex gap-2 flex-wrap">
                {[("Novice", "text-slate-400 bg-slate-500/10 border-slate-500/20"),
                  ("Beginner", "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"),
                  ("Competent", "text-cyan-400 bg-cyan-500/10 border-cyan-500/20"),
                  ("Proficient", "text-amber-400 bg-amber-500/10 border-amber-500/20"),
                  ("Expert", "text-violet-400 bg-violet-500/10 border-violet-500/20")]
                    .into_iter().map(|(level, cls)| view! {
                        <span class=format!("rounded-full border px-3 py-1 text-[10px] font-bold font-mono uppercase {cls}")>{level}</span>
                    }).collect_view()}
            </div>

            /* Skill Categories */
            <div class="mt-8 space-y-2">
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">"Skill Categories"</h2>
                {CATEGORIES.iter().map(|c| {
                    let proficiency_cls = if c.avg_proficiency >= 75 { "bg-emerald-500" } else if c.avg_proficiency >= 65 { "bg-amber-500" } else { "bg-red-500" };
                    view! {
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-600 transition-colors cursor-pointer">
                            <div class="flex items-center justify-between">
                                <div>
                                    <h3 class=format!("text-sm font-bold {}", c.color)>{c.name}</h3>
                                    <p class="text-[10px] text-slate-500 font-mono mt-1">{format!("{} skills \u{00B7} {} assessments", c.skills, c.assessments)}</p>
                                </div>
                                <div class="flex items-center gap-3">
                                    <div class="flex items-center gap-2">
                                        <span class="text-[10px] text-slate-500 font-mono">"Avg Proficiency"</span>
                                        <div class="h-1.5 w-16 rounded-full bg-slate-800 overflow-hidden">
                                            <div class=format!("h-full rounded-full {proficiency_cls}") style=format!("width:{}%", c.avg_proficiency)></div>
                                        </div>
                                        <span class="text-xs font-bold font-mono text-slate-400">{format!("{}%", c.avg_proficiency)}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
