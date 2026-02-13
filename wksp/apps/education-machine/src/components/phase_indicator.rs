//! 5-phase FSM visualization component

use leptos::prelude::*;

/// Phases of the learning process
const PHASES: [&str; 5] = ["Discover", "Extract", "Practice", "Assess", "Master"];

/// Visual indicator showing progress through the 5-phase learning FSM.
#[component]
pub fn PhaseIndicator(
    /// Current phase name
    #[prop(into)]
    current_phase: String,
) -> impl IntoView {
    let current_idx = PHASES
        .iter()
        .position(|p| *p == current_phase)
        .unwrap_or(0);

    view! {
        <div class="flex items-center space-x-1">
            {PHASES.iter().enumerate().map(|(i, phase)| {
                let (bg, text_color) = if i < current_idx {
                    ("bg-green-600", "text-green-200")
                } else if i == current_idx {
                    ("bg-cyan-600", "text-white")
                } else {
                    ("bg-gray-700", "text-gray-500")
                };

                let connector = if i < PHASES.len() - 1 {
                    Some(view! {
                        <div class={if i < current_idx { "w-4 h-0.5 bg-green-600" } else { "w-4 h-0.5 bg-gray-700" }}></div>
                    })
                } else {
                    None
                };

                view! {
                    <div class="flex items-center">
                        <div class={format!("{bg} {text_color} px-2 py-1 rounded text-xs font-medium")}>
                            {*phase}
                        </div>
                        {connector}
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
