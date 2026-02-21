//! Lesson content viewer component

use leptos::prelude::*;

use crate::server::get_lessons::LessonStepData;

/// Renders a single lesson step based on content type.
#[component]
pub fn LessonStepView(
    /// Step data
    step: LessonStepData,
    /// Step number (1-indexed)
    number: usize,
) -> impl IntoView {
    let icon = match step.content_type.as_str() {
        "decomposition" => "🔬",
        "exercise" => "✏️",
        _ => "📖",
    };

    let type_label = match step.content_type.as_str() {
        "decomposition" => "Primitive Decomposition",
        "exercise" => "Practice Exercise",
        _ => "Reading",
    };

    let type_color = match step.content_type.as_str() {
        "decomposition" => "text-purple-400 border-purple-600",
        "exercise" => "text-cyan-400 border-cyan-600",
        _ => "text-gray-300 border-gray-600",
    };

    view! {
        <div class={format!("bg-gray-800 rounded-lg p-6 border-l-4 {type_color}")}>
            <div class="flex items-center gap-2 mb-3">
                <span class="text-lg">{icon}</span>
                <span class="text-gray-500 text-sm">{format!("Step {number}")}</span>
                <span class={format!("text-xs px-2 py-0.5 rounded bg-gray-700 {type_color}")}>{type_label}</span>
            </div>
            <h3 class="text-white font-semibold mb-2">{step.title}</h3>
            <div class="text-gray-300 leading-relaxed">
                {step.body}
            </div>
            {if !step.hints.is_empty() {
                Some(view! {
                    <div class="mt-4 bg-gray-750 rounded p-3">
                        <p class="text-yellow-400 text-sm font-medium mb-1">"Hints"</p>
                        <ul class="list-disc list-inside text-gray-400 text-sm space-y-1">
                            {step.hints.into_iter().map(|hint| {
                                view! { <li>{hint}</li> }
                            }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}
