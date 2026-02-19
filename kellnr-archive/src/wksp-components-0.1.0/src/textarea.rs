use leptos::prelude::*;

/// Textarea input component
#[component]
pub fn Textarea(
    #[prop(optional)] label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] rows: u32,
) -> impl IntoView {
    let row_count = if rows == 0 { 4 } else { rows };

    view! {
        <div>
            {if !label.is_empty() {
                Some(view! { <label class="block text-sm font-medium text-slate-300 mb-1">{label}</label> })
            } else {
                None
            }}
            <textarea
                class="block w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none focus:ring-1 focus:ring-cyan-500 resize-y"
                placeholder=placeholder
                rows=row_count
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            />
        </div>
    }
}
