use leptos::prelude::*;

/// Range slider input
#[component]
pub fn Slider(
    value: RwSignal<f64>,
    #[prop(optional, default = 0.0)] min: f64,
    #[prop(optional, default = 100.0)] max: f64,
    #[prop(optional, default = 1.0)] step: f64,
) -> impl IntoView {
    view! {
        <div class="w-full">
            <input
                type="range"
                class="w-full h-2 rounded-lg appearance-none cursor-pointer bg-slate-700 accent-cyan-500"
                prop:value=move || value.get().to_string()
                min=min.to_string()
                max=max.to_string()
                step=step.to_string()
                on:input=move |ev| {
                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                        value.set(v);
                    }
                }
            />
        </div>
    }
}
