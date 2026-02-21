//! Signal Cascade Visualizer Component

use leptos::prelude::*;
use crate::server::get_guardian_signals::{get_guardian_signals, GuardianSignalInfo};

#[component]
pub fn SignalCascadeVisualizer() -> impl IntoView {
    let signals = Resource::new(|| (), |_| get_guardian_signals());

    view! {
        <div class="mt-8 bg-gray-800 rounded-lg p-6 border border-red-900/30">
            <h2 class="text-xl font-semibold text-red-400 mb-4 flex items-center gap-2">
                <span class="animate-pulse">"🛡️"</span>
                "Immune Response: Signal Cascade"
            </h2>
            <div class="space-y-4">
                <Suspense fallback=|| view! { <p class="text-gray-500">"Sensing environment..."</p> }>
                    {move || {
                        let data = signals.get();
                        match data {
                            Some(Ok(s)) => view! { <SignalList signals=s /> }.into_any(),
                            Some(Err(e)) => view! { <p class="text-red-500">"Error sensing: " {e.to_string()}</p> }.into_any(),
                            None => view! { <p class="text-gray-500">"Loading signals..."</p> }.into_any(),
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn SignalList(signals: Vec<GuardianSignalInfo>) -> impl IntoView {
    if signals.is_empty() {
        return view! { <p class="text-gray-500 italic">"No pathogenic patterns detected. System healthy."</p> }.into_any();
    }

    view! {
        <div class="grid gap-3">
            {signals.into_iter().map(|s| view! { <SignalItem signal=s /> }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}

#[component]
fn SignalItem(signal: GuardianSignalInfo) -> impl IntoView {
    let severity_class = match signal.severity.as_str() {
        "Critical" => "border-red-600 bg-red-900/20 text-red-200",
        "High" => "border-orange-600 bg-orange-900/20 text-orange-200",
        "Medium" => "border-yellow-600 bg-yellow-900/20 text-yellow-200",
        _ => "border-blue-600 bg-blue-900/20 text-blue-200",
    };

    view! {
        <div class={format!("flex items-start justify-between p-3 border rounded-md shadow-sm {}", severity_class)}>
            <div class="flex-1">
                <div class="flex items-center gap-2 mb-1">
                    <span class="text-xs font-bold uppercase tracking-wider px-1.5 py-0.5 rounded bg-black/30">
                        {signal.severity}
                    </span>
                    <span class="font-mono text-sm">{signal.pattern}</span>
                </div>
                <p class="text-xs text-gray-400 font-mono">"ID: " {signal.id}</p>
                <div class="text-xs mt-1">
                    {if let Some(v) = signal.verdict {
                        let v_clone = v.clone();
                        view! {
                            <span>
                                <span class="text-gray-500">"Verdict: "</span>
                                <span class={if v_clone == "generated" { "text-red-400 font-bold" } else { "text-green-400" }}>{v}</span>
                                {if let Some(p) = signal.probability {
                                    view! { <span class="text-gray-500 ml-2">"(" {(p * 100.0) as u32} "%)"</span> }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </span>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
            <div class="text-[10px] text-gray-500 font-mono italic">
                {signal.timestamp}
            </div>
        </div>
    }
}
