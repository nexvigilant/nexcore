//! Assessment question form and result display

use leptos::prelude::*;

use crate::server::submit_assessment::{AssessmentResultData, QuestionData};

/// Single question display with answer input
#[component]
pub fn QuestionView(
    /// Question data
    question: QuestionData,
    /// Index number
    number: usize,
) -> impl IntoView {
    let difficulty_color = match question.difficulty.as_str() {
        "Hard" => "text-red-400 bg-red-900/30",
        "Medium" => "text-yellow-400 bg-yellow-900/30",
        _ => "text-green-400 bg-green-900/30",
    };

    view! {
        <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <div class="flex justify-between items-start mb-3">
                <span class="text-gray-500 text-sm">{format!("Question {number}")}</span>
                <span class={format!("text-xs px-2 py-0.5 rounded {difficulty_color}")}>{question.difficulty}</span>
            </div>
            <p class="text-white font-medium mb-3">{question.prompt}</p>
            <p class="text-gray-500 text-xs">{format!("Concept: {}", question.concept)}</p>
            <textarea
                class="w-full mt-3 bg-gray-900 border border-gray-600 rounded p-3 text-white placeholder-gray-500 focus:border-cyan-500 focus:outline-none"
                placeholder="Type your answer..."
                rows="3"
                name={format!("answer_{}", question.id)}
            ></textarea>
        </div>
    }
}

/// Assessment result display
#[component]
pub fn AssessmentResultView(
    /// Result data
    result: AssessmentResultData,
) -> impl IntoView {
    let pct = (result.mastery * 100.0).min(100.0);
    let bar_width = format!("width: {pct:.0}%");
    let bar_color = match result.verdict.as_str() {
        "Mastered" => "bg-green-500",
        "Developing" => "bg-yellow-500",
        _ => "bg-red-500",
    };

    view! {
        <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <h3 class="text-xl font-semibold text-white mb-4">"Assessment Complete"</h3>
            <div class="grid grid-cols-3 gap-4 mb-4">
                <div class="text-center">
                    <p class="text-3xl font-bold text-cyan-400">{format!("{}/{}", result.correct_count, result.total_count)}</p>
                    <p class="text-gray-500 text-sm">"Correct"</p>
                </div>
                <div class="text-center">
                    <p class={format!("text-3xl font-bold {}", result.verdict_color)}>{format!("{pct:.0}%")}</p>
                    <p class="text-gray-500 text-sm">"Mastery"</p>
                </div>
                <div class="text-center">
                    <p class={format!("text-3xl font-bold {}", result.verdict_color)}>{result.verdict.clone()}</p>
                    <p class="text-gray-500 text-sm">"Verdict"</p>
                </div>
            </div>
            <div class="w-full bg-gray-700 rounded-full h-3">
                <div class={format!("{bar_color} h-3 rounded-full transition-all duration-1000")}
                     style={bar_width}>
                </div>
            </div>
        </div>
    }
}
