//! Subject progress card component

use leptos::prelude::*;

use crate::components::progress_bar::ProgressBar;
use crate::components::phase_indicator::PhaseIndicator;

/// Subject progress card showing mastery, phase, and lesson count.
#[component]
pub fn SubjectCard(
    /// Subject ID for navigation
    #[prop(into)]
    id: String,
    /// Subject name
    #[prop(into)]
    name: String,
    /// Brief description
    #[prop(into)]
    description: String,
    /// Number of lessons
    lesson_count: usize,
    /// Mastery level [0, 1]
    mastery: f64,
    /// Current learning phase
    #[prop(into)]
    phase: String,
    /// Tags
    #[prop(into)]
    tags: Vec<String>,
) -> impl IntoView {
    let verdict_color = if mastery >= 0.85 {
        "text-green-400"
    } else if mastery >= 0.50 {
        "text-yellow-400"
    } else {
        "text-red-400"
    };

    let href = format!("/subject/{id}");

    view! {
        <a href={href} class="block bg-gray-800 rounded-lg p-6 hover:bg-gray-750 transition-colors border border-gray-700 hover:border-cyan-600">
            <div class="flex justify-between items-start mb-3">
                <h3 class="text-lg font-semibold text-white">{name}</h3>
                <span class={format!("{verdict_color} text-sm font-medium")}>
                    {format!("{:.0}%", mastery * 100.0)}
                </span>
            </div>
            <p class="text-gray-400 text-sm mb-3">{description}</p>
            <ProgressBar value=mastery />
            <div class="mt-3">
                <PhaseIndicator current_phase=phase />
            </div>
            <div class="mt-3 flex justify-between items-center">
                <span class="text-gray-500 text-xs">{format!("{lesson_count} lessons")}</span>
                <div class="flex gap-1">
                    {tags.into_iter().map(|tag| {
                        view! {
                            <span class="bg-gray-700 text-gray-300 px-2 py-0.5 rounded text-xs">{tag}</span>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </a>
    }
}
