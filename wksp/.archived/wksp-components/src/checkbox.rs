use leptos::prelude::*;

/// Checkbox input component
#[component]
pub fn Checkbox(
    checked: RwSignal<bool>,
    #[prop(optional)] label: &'static str,
) -> impl IntoView {
    view! {
        <label class="inline-flex items-center gap-3 cursor-pointer min-h-[44px]">
            <input
                type="checkbox"
                class="h-5 w-5 rounded border-slate-600 bg-slate-800 text-cyan-600 focus:ring-cyan-500 focus:ring-offset-slate-950"
                prop:checked=move || checked.get()
                on:change=move |_| checked.set(!checked.get())
            />
            {if !label.is_empty() {
                Some(view! { <span class="text-sm text-slate-300">{label}</span> })
            } else {
                None
            }}
        </label>
    }
}
