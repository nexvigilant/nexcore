//! Shared components for the Academy section

use leptos::prelude::*;

/* ------------------------------------------------------------------ */
/*  Academy Dashboard Stat Card                                        */
/* ------------------------------------------------------------------ */

#[component]
pub fn AcademyStatCard(
    title: &'static str,
    value: String,
    subtext: String,
    variant: &'static str, // "cyan" or "gold"
) -> impl IntoView {
    let (text_color, bg_color, border_hover) = match variant {
        "gold" => ("text-amber-400", "bg-amber-500/10", "hover:border-amber-500/50"),
        _ => ("text-cyan-400", "bg-cyan-500/10", "hover:border-cyan-500/50"),
    };

    view! {
        <div class=format!("rounded-xl border border-slate-800 bg-slate-900/50 p-5 transition-all duration-300 {border_hover}")>
            <div class="flex items-center justify-between mb-2">
                <span class="text-xs font-semibold text-slate-400">{title}</span>
                <div class=format!("p-1.5 rounded-lg {bg_color}")>
                    <span class=format!("text-xs {text_color}")>
                        {match variant {
                            "gold" => "★",
                            _ => "◈",
                        }}
                    </span>
                </div>
            </div>
            <div class=format!("text-2xl font-bold {text_color} font-mono")>
                {value}
            </div>
            <p class="text-[10px] text-slate-500 font-medium mt-1 uppercase tracking-wider">
                {subtext}
            </p>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Streak Widget                                                      */
/* ------------------------------------------------------------------ */

#[component]
pub fn StreakWidget(streak: u32) -> impl IntoView {
    let has_streak = streak > 0;
    
    view! {
        <div class="flex items-center gap-4 bg-slate-900/80 border border-slate-800/50 rounded-2xl px-5 py-4">
            /* Fire Icon with Glow */
            <div class="relative flex items-center justify-center h-12 w-12">
                <div class=move || {
                    let base = "absolute inset-0 rounded-full blur-md opacity-30";
                    if has_streak { format!("{base} bg-orange-500") } else { format!("{base} bg-slate-600") }
                }></div>
                
                /* Circular progress ring (simplified for CSS) */
                <svg class="absolute inset-0 h-full w-full -rotate-90" viewBox="0 0 48 48">
                    <circle cx="24" cy="24" r="20" fill="none" class="stroke-slate-800" stroke-width="3" />
                    <circle cx="24" cy="24" r="20" fill="none" 
                        class=move || {
                            if has_streak { "stroke-cyan-500" } else { "stroke-slate-700" }
                        }
                        stroke-width="3"
                        stroke-dasharray="126"
                        stroke-dashoffset=move || {
                            if has_streak { format!("{:.1}", 126.0 - (streak as f64 * 12.6).min(126.0)) } else { "126".to_string() }
                        }
                        stroke-linecap="round"
                    />
                </svg>
                
                <span class="text-2xl relative z-10">"🔥"</span>
            </div>

            /* Text Content */
            <div class="flex flex-col">
                <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    "Daily Streak"
                </span>
                <div class="flex items-center gap-2">
                    <span class=move || {
                        let base = "text-lg font-bold font-mono";
                        if has_streak { format!("{base} text-cyan-400") } else { format!("{base} text-slate-400") }
                    }>
                        {streak} " Days"
                    </span>
                    <span class="text-slate-600">" — "</span>
                    <span class="text-xs text-slate-500 italic">
                        {if has_streak { "Keep it going!" } else { "Start your streak!" }}
                    </span>
                </div>
            </div>

            /* Decorative Status Dot */
            <div class=move || {
                let base = "h-2 w-2 rounded-full ml-auto";
                if has_streak { format!("{base} bg-cyan-500 shadow-[0_0_8px_rgba(34,211,238,0.6)]") }
                else { format!("{base} bg-slate-700") }
            }></div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Resume Card                                                        */
/* ------------------------------------------------------------------ */

#[component]
pub fn ResumeCard(
    course_id: String,
    title: String,
    progress: f64,
) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6 hover:border-cyan-500/30 transition-all group overflow-hidden relative">
            <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
                <span class="text-4xl">"▶"</span>
            </div>
            
            <div class="relative z-10">
                <span class="text-[10px] font-bold text-cyan-500 font-mono uppercase tracking-widest">"Continue Building"</span>
                <h3 class="text-xl font-bold text-white mt-1 mb-4">{title}</h3>
                
                <div class="flex items-center justify-between text-[10px] font-mono mb-2">
                    <span class="text-slate-500">"OVERALL PROGRESS"</span>
                    <span class="text-cyan-400">{(progress * 100.0) as u32}"%"</span>
                </div>
                
                <div class="h-1.5 w-full bg-slate-800 rounded-full overflow-hidden mb-6">
                    <div class="h-full bg-cyan-500" style=format!("width: {}%", progress * 100.0)></div>
                </div>
                
                <a href=format!("/academy/learn/{}", course_id) class="inline-flex items-center gap-2 rounded-lg bg-cyan-600 px-6 py-2.5 text-xs font-bold text-white hover:bg-cyan-500 transition-all font-mono uppercase tracking-widest shadow-lg shadow-cyan-900/20">
                    "RESUME PATHWAY"
                </a>
            </div>
        </div>
    }
}
