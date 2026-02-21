//! SessionStats - Display session metrics

use leptos::prelude::*;

#[component]
pub fn SessionStats(duration: u32, tools: u32, tokens: u64, level: u32, xp: u64, prestige: u64, velocity: f64) -> impl IntoView {
    view! {
        <div class="session_stats bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold text-purple-400 mb-4">"📊 Session Stats"</h2>
            <div class="space-y-2">
                <StatRow icon="⏱️" label="Duration" value=format_duration(duration) />
                <StatRow icon="🔧" label="Tools Called" value=tools.to_string() />
                <StatRow icon="🪙" label="Tokens Used" value=format_tokens(tokens) />
                <hr class="border-gray-700 my-2" />
                <StatRow icon="🚀" label="Current Level" value=level.to_string() color="text-yellow-400" />
                <StatRow icon="📈" label="Basis XP" value=xp.to_string() color="text-cyan-400" />
                <StatRow icon="💎" label="Reuse Prestige" value=prestige.to_string() color="text-pink-400" />
                <StatRow icon="⚡" label="Velocity" value=format!("{:.2}", velocity) color="text-green-400" />
            </div>
        </div>
    }
}

#[component]
fn StatRow(icon: &'static str, label: &'static str, value: String, #[prop(default = "text-white")] color: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between p-2 bg-gray-700/50 rounded">
            <div class="flex items-center gap-2">
                <span class="text-lg">{icon}</span>
                <span class="text-xs text-gray-400 uppercase tracking-tighter">{label}</span>
            </div>
            <span class={format!("font-mono text-sm font-bold {}", color)}>{value}</span>
        </div>
    }
}

fn format_duration(mins: u32) -> String {
    if mins < 60 {
        format!("{}m", mins)
    } else {
        format!("{}h {}m", mins / 60, mins % 60)
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
