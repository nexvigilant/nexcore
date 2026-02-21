//! Mastery progress bar component (red → yellow → green)

use leptos::prelude::*;

/// Mastery progress bar with color gradient.
///
/// - Red: < 0.50
/// - Yellow: 0.50 - 0.84
/// - Green: >= 0.85
#[component]
pub fn ProgressBar(
    /// Mastery value [0, 1]
    #[prop(into)]
    value: f64,
    /// Optional label
    #[prop(optional, into)]
    label: String,
) -> impl IntoView {
    let pct = (value * 100.0).clamp(0.0, 100.0);
    let color = if value >= 0.85 {
        "bg-green-500"
    } else if value >= 0.50 {
        "bg-yellow-500"
    } else {
        "bg-red-500"
    };
    let width_style = format!("width: {pct:.0}%");

    view! {
        <div class="w-full">
            {if !label.is_empty() {
                Some(view! {
                    <div class="flex justify-between text-sm mb-1">
                        <span class="text-gray-300">{label.clone()}</span>
                        <span class="text-gray-400">{format!("{pct:.0}%")}</span>
                    </div>
                })
            } else {
                None
            }}
            <div class="w-full bg-gray-700 rounded-full h-2.5">
                <div class={format!("{color} h-2.5 rounded-full transition-all duration-500")}
                     style={width_style}>
                </div>
            </div>
        </div>
    }
}
