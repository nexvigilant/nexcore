//! Measures - Key metrics and KPIs for adventure tracking

use leptos::prelude::*;

/// Adventure metrics data
#[derive(Debug, Clone, Default)]
pub struct MetricsData {
    pub reality_score: f64,
    pub token_efficiency: f64,
    pub task_velocity: f64,
    pub skill_coverage: f64,
    pub compliance_level: String,
}

#[component]
pub fn Measures(metrics: MetricsData) -> impl IntoView {
    view! {
        <div class="measures bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold text-emerald-400 mb-4">"📏 Measures"</h2>
            <MetricsGrid metrics=metrics.clone() />
            <ComplianceBadge level=metrics.compliance_level />
        </div>
    }
}

#[component]
fn MetricsGrid(metrics: MetricsData) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-4">
            <GaugeMetric label="Reality Score" value=metrics.reality_score color="cyan" />
            <GaugeMetric label="Token Efficiency" value=metrics.token_efficiency color="green" />
            <GaugeMetric label="Task Velocity" value=metrics.task_velocity color="purple" />
            <GaugeMetric label="Skill Coverage" value=metrics.skill_coverage color="orange" />
        </div>
    }
}

#[component]
fn GaugeMetric(label: &'static str, value: f64, color: &'static str) -> impl IntoView {
    let pct = (value * 100.0).min(100.0) as u32;
    let width = format!("{}%", pct);

    view! {
        <div class="p-3 bg-gray-700 rounded">
            <div class="flex justify-between text-sm mb-1">
                <span class="text-gray-300">{label}</span>
                <span class="text-white font-mono">{format!("{:.2}", value)}</span>
            </div>
            <div class="w-full bg-gray-600 rounded-full h-2">
                <div class="bg-cyan-500 h-2 rounded-full" style:width=width></div>
            </div>
        </div>
    }
}

#[component]
fn ComplianceBadge(level: String) -> impl IntoView {
    let icon = compliance_icon(&level);
    view! {
        <div class="mt-4 flex items-center gap-2 p-3 bg-gray-700 rounded">
            <span class="text-xl">{icon}</span>
            <span class="text-gray-300">"Compliance:"</span>
            <span class="font-bold text-yellow-400">{level}</span>
        </div>
    }
}

fn compliance_icon(level: &str) -> &'static str {
    match level.to_lowercase().as_str() {
        "diamond" => "💎",
        "platinum" => "🏆",
        "gold" => "🥇",
        "silver" => "🥈",
        _ => "🥉",
    }
}
