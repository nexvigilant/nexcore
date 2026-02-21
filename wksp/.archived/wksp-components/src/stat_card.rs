use leptos::prelude::*;

/// Dashboard statistic card with label, value, and optional trend
#[component]
pub fn StatCard(
    label: &'static str,
    value: String,
    #[prop(optional)] trend: &'static str,
    #[prop(optional)] trend_positive: bool,
) -> impl IntoView {
    let trend_color = if trend_positive {
        "text-emerald-400"
    } else {
        "text-red-400"
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900 p-4">
            <p class="text-xs font-medium uppercase tracking-wider text-slate-500">{label}</p>
            <p class="mt-1 text-2xl font-bold text-white">{value}</p>
            {if !trend.is_empty() {
                Some(view! {
                    <p class=format!("mt-1 text-xs {trend_color}")>{trend}</p>
                })
            } else {
                None
            }}
        </div>
    }
}
