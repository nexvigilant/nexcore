//! Shared academy UI components — stat cards, streak widget, resume card

use leptos::prelude::*;

#[component]
pub fn AcademyStatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] subtext: String,
    #[prop(into)] variant: String,
) -> impl IntoView {
    let color = if variant == "gold" {
        "text-amber-400"
    } else {
        "text-cyan-400"
    };
    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-5 text-center">
            <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">{title}</p>
            <p class=format!("mt-2 text-3xl font-black font-mono {color}")>{value}</p>
            <p class="mt-1 text-[10px] text-slate-600 font-mono">{subtext}</p>
        </div>
    }
}

#[component]
pub fn ResumeCard(
    #[prop(into)] course_id: String,
    #[prop(into)] title: String,
    progress: f64,
) -> impl IntoView {
    let pct = (progress * 100.0) as u32;
    view! {
        <a href=format!("/academy/learn/{course_id}") class="block rounded-2xl border border-cyan-500/20 bg-cyan-500/5 p-6 hover:border-cyan-500/40 transition-all group">
            <div class="flex items-center justify-between mb-3">
                <span class="text-[10px] font-bold text-cyan-500 uppercase tracking-widest font-mono">"Continue Learning"</span>
                <span class="text-xs font-bold text-cyan-400 font-mono group-hover:translate-x-1 transition-transform">"RESUME →"</span>
            </div>
            <p class="text-sm font-bold text-white mb-3">{title}</p>
            <div class="flex items-center gap-3">
                <div class="flex-1 h-1 bg-slate-800 rounded-full overflow-hidden">
                    <div class="h-full bg-cyan-500 transition-all" style=format!("width: {}%", pct)></div>
                </div>
                <span class="text-[10px] font-bold text-slate-400 font-mono">{pct}"%"</span>
            </div>
        </a>
    }
}

#[component]
pub fn StreakWidget(streak: u32) -> impl IntoView {
    view! {
        <div class="flex-1 rounded-2xl border border-slate-800 bg-slate-900/50 p-6 flex items-center justify-between hover:border-amber-500/30 transition-all cursor-pointer group">
            <div class="flex items-center gap-4">
                <div class="h-10 w-10 rounded-full bg-amber-500/10 flex items-center justify-center text-amber-400">
                    "🔥"
                </div>
                <div>
                    <p class="text-xs font-bold text-slate-500 uppercase tracking-widest">"Learning Streak"</p>
                    <p class="text-sm font-bold text-white">{streak} " day streak"</p>
                </div>
            </div>
            <span class="text-xs font-bold text-amber-400 group-hover:translate-x-1 transition-transform">"KEEP GOING →"</span>
        </div>
    }
}

#[component]
pub fn GuardianApplyCard(
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] summary: String,
    #[prop(optional, into)] cta: String,
    #[prop(optional, into)] href: String,
) -> impl IntoView {
    let title = if title.is_empty() {
        "Apply in Guardian".to_string()
    } else {
        title
    };
    let summary = if summary.is_empty() {
        "Move from theory to live pharmacovigilance execution in Guardian.".to_string()
    } else {
        summary
    };
    let cta = if cta.is_empty() {
        "Open Guardian".to_string()
    } else {
        cta
    };
    let href = if href.is_empty() {
        "/vigilance/guardian".to_string()
    } else {
        href
    };

    view! {
        <a
            href=href
            class="block rounded-2xl border border-emerald-500/20 bg-emerald-500/5 p-6 hover:border-emerald-500/40 transition-all group"
        >
            <div class="flex items-center justify-between gap-4">
                <div>
                    <p class="text-[10px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Practice Bridge"</p>
                    <h3 class="mt-2 text-lg font-bold text-white">{title}</h3>
                    <p class="mt-1 text-sm text-slate-400">{summary}</p>
                </div>
                <span class="text-emerald-300 font-mono text-xs group-hover:translate-x-1 transition-transform uppercase tracking-widest">
                    {cta}
                    " →"
                </span>
            </div>
        </a>
    }
}
