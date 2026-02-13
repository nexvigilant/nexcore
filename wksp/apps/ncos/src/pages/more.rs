use leptos::prelude::*;
use leptos_router::components::A;

/// "More" page — navigation to secondary pages
/// Tier: T2-P (Mapping + Sequence)
#[component]
pub fn MorePage() -> impl IntoView {
    view! {
        <div class="page more">
            <h1 class="page-title">"More"</h1>

            <div class="more-list">
                <A href="/causality" attr:class="more-item">
                    <span class="more-icon">"\u{2696}"</span>
                    <span class="more-label">"Causality Assessment"</span>
                    <span class="more-arrow">"\u{203A}"</span>
                </A>
                <A href="/pvdsl" attr:class="more-item">
                    <span class="more-icon">"\u{2328}"</span>
                    <span class="more-label">"PVDSL Console"</span>
                    <span class="more-arrow">"\u{203A}"</span>
                </A>
                <A href="/skills" attr:class="more-item">
                    <span class="more-icon">"\u{2699}"</span>
                    <span class="more-label">"Skills Registry"</span>
                    <span class="more-arrow">"\u{203A}"</span>
                </A>
                <A href="/benefit-risk" attr:class="more-item">
                    <span class="more-icon">"\u{2696}"</span>
                    <span class="more-label">"Benefit-Risk (QBRI)"</span>
                    <span class="more-arrow">"\u{203A}"</span>
                </A>
                <A href="/settings" attr:class="more-item">
                    <span class="more-icon">"\u{2699}"</span>
                    <span class="more-label">"Settings"</span>
                    <span class="more-arrow">"\u{203A}"</span>
                </A>
            </div>
        </div>
    }
}
