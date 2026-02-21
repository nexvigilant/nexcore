//! Clicker Game - Home page. Tier: T2-C

use crate::components::{AutoTicker, ClickTarget, GameMetrics, Measures, ScoreDisplay, UpgradeShop, UPGRADES};
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let (score, set_score) = signal(0u64);
    let (cps, set_cps) = signal(0u64);
    let (owned, set_owned) = signal(vec![0u64; UPGRADES.len()]);
    let (metrics, set_metrics) = signal(GameMetrics::default());

    let on_click = move || {
        set_score.update(|s| *s += 1);
        set_metrics.update(|m| m.manual_clicks += 1);
    };

    let on_buy = move |idx: usize| {
        let cost = UPGRADES[idx].cost * (owned.get()[idx] + 1);
        if score.get() >= cost {
            set_score.update(|s| *s -= cost);
            set_cps.update(|c| *c += UPGRADES[idx].cps);
            set_owned.update(|o| o[idx] += 1);
        }
    };

    view! {
        <div class="min-h-screen bg-gray-900 text-white p-8">
            <div class="max-w-xl mx-auto">
                <h1 class="text-5xl font-bold text-center mb-2 text-yellow-400">"Ferro Clicker"</h1>
                <p class="text-center text-gray-400 mb-8">"Click the rust crab!"</p>
                <ScoreDisplay score cps />
                <div class="flex justify-center mb-8"><ClickTarget on_click /></div>
                <UpgradeShop score owned on_buy />
                <Measures metrics />
                <AutoTicker cps set_score set_metrics />
            </div>
        </div>
    }
}
