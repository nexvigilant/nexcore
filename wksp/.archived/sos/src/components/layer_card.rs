/// PVOS layer card — Σ Sum (KSB count) + κ Comparison (transfer confidence)
use leptos::prelude::*;

use crate::api::pvos::KsbDomain;
use crate::components::progress_bar::ProgressBar;

#[component]
pub fn LayerCard(domain: KsbDomain, on_tap: impl Fn() + 'static) -> impl IntoView {
    let confidence_pct = (domain.transfer_confidence * 100.0) as u32;

    view! {
        <button class="layer-card" on:click=move |_| on_tap()>
            <div class="layer-abbrev">{domain.code.clone()}</div>
            <div class="layer-name">{domain.name.clone()}</div>
            <div class="layer-primitive">
                <span class="primitive">{domain.dominant_primitive.clone()}</span>
            </div>
            <div class="layer-stats">
                <span class="layer-ksb-count">{domain.ksb_count}" KSBs"</span>
            </div>
            <div class="layer-confidence">
                <span class="layer-confidence-label">"Transfer"</span>
                <ProgressBar value=confidence_pct max=100 />
            </div>
        </button>
    }
}
