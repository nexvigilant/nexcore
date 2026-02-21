//! Measures component - Tier: T2-C
//! Tracks game metrics: clicks, time, efficiency

use leptos::prelude::*;

/// Game metrics. Tier: T2-P
#[derive(Clone, Copy, Default)]
pub struct GameMetrics {
    pub manual_clicks: u64,
    pub auto_clicks: u64,
    pub seconds_played: u64,
}

impl GameMetrics {
    pub fn total_clicks(&self) -> u64 {
        self.manual_clicks + self.auto_clicks
    }
    pub fn clicks_per_minute(&self) -> f64 {
        if self.seconds_played == 0 { return 0.0; }
        (self.total_clicks() as f64 / self.seconds_played as f64) * 60.0
    }
}

#[component]
pub fn Measures(metrics: ReadSignal<GameMetrics>) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg p-4 mt-4" id="measures">
            <h3 class="text-lg font-bold text-yellow-400 mb-2">"Measures"</h3>
            <div class="grid grid-cols-2 gap-2 text-sm">
                <div>"Manual clicks:"</div>
                <div class="text-right">{move || metrics.get().manual_clicks}</div>
                <div>"Auto clicks:"</div>
                <div class="text-right">{move || metrics.get().auto_clicks}</div>
                <div>"Time played:"</div>
                <div class="text-right">{move || format!("{}s", metrics.get().seconds_played)}</div>
                <div>"Clicks/min:"</div>
                <div class="text-right">{move || format!("{:.1}", metrics.get().clicks_per_minute())}</div>
            </div>
        </div>
    }
}
