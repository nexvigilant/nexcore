//! ScoreDisplay component
//! Tier: T2-C - Displays score and CPS from signals

use leptos::prelude::*;

#[component]
pub fn ScoreDisplay(
    score: ReadSignal<u64>,
    cps: ReadSignal<u64>,
) -> impl IntoView {
    view! {
        <div class="text-center mb-8">
            <div class="text-6xl font-bold text-orange-500" id="score">
                {move || score.get()}
            </div>
            <div class="text-gray-400">"clicks"</div>
            <div class="text-green-400 mt-2" id="cps">
                {move || format!("+{} per second", cps.get())}
            </div>
        </div>
    }
}
