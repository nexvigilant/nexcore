//! For You — personalized community feed

use leptos::prelude::*;

#[component]
pub fn ForYouPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"For You"</h1>
            <p class="mt-2 text-slate-400">"Personalized content based on your interests and activity."</p>

            <div class="mt-8 space-y-4">
                <SuggestedItem
                    item_type="Circle"
                    title="Signal Detection Practitioners"
                    reason="Based on your interest in signal detection"
                />
                <SuggestedItem
                    item_type="Post"
                    title="PRR vs ROR: When to Use Which?"
                    reason="Trending in Signal Detection circle"
                />
                <SuggestedItem
                    item_type="Member"
                    title="Connect with PV professionals in your area"
                    reason="3 members near you"
                />
                <SuggestedItem
                    item_type="Course"
                    title="Introduction to Benefit-Risk Assessment"
                    reason="Recommended based on your competency assessment"
                />
            </div>
        </div>
    }
}

#[component]
fn SuggestedItem(item_type: &'static str, title: &'static str, reason: &'static str) -> impl IntoView {
    let type_color = match item_type {
        "Circle" => "text-violet-400 bg-violet-500/10",
        "Post" => "text-cyan-400 bg-cyan-500/10",
        "Member" => "text-emerald-400 bg-emerald-500/10",
        "Course" => "text-amber-400 bg-amber-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors cursor-pointer">
            <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {type_color}")>{item_type}</span>
            <h3 class="mt-2 font-semibold text-white">{title}</h3>
            <p class="mt-1 text-xs text-slate-500">{reason}</p>
        </div>
    }
}
