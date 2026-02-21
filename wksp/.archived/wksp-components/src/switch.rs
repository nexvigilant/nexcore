use leptos::prelude::*;

/// Toggle switch component
#[component]
pub fn Switch(
    checked: RwSignal<bool>,
    #[prop(optional)] label: &'static str,
) -> impl IntoView {
    view! {
        <label class="inline-flex items-center gap-3 cursor-pointer">
            <button
                type="button"
                role="switch"
                class=move || format!(
                    "relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-slate-950 {}",
                    if checked.get() { "bg-cyan-600" } else { "bg-slate-700" }
                )
                on:click=move |_| checked.set(!checked.get())
            >
                <span
                    class=move || format!(
                        "inline-block h-4 w-4 transform rounded-full bg-white transition-transform {}",
                        if checked.get() { "translate-x-6" } else { "translate-x-1" }
                    )
                />
            </button>
            {if !label.is_empty() {
                Some(view! { <span class="text-sm text-slate-300">{label}</span> })
            } else {
                None
            }}
        </label>
    }
}
