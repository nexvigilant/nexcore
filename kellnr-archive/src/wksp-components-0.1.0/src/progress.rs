use leptos::prelude::*;

/// Progress bar component
#[component]
pub fn Progress(
    /// Value from 0 to 100
    #[prop(into)]
    value: Signal<f32>,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let outer_classes = format!("h-2 w-full overflow-hidden rounded-full bg-slate-800 {class}");

    view! {
        <div class=outer_classes role="progressbar">
            <div
                class="h-full rounded-full bg-cyan-500 transition-all duration-300"
                style=move || format!("width: {}%", value.get().clamp(0.0, 100.0))
            />
        </div>
    }
}
