use leptos::prelude::*;

/// Select option definition
#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// Select dropdown component
#[component]
pub fn Select(
    #[prop(optional)] label: &'static str,
    value: RwSignal<String>,
    options: Vec<SelectOption>,
    #[prop(optional)] placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div>
            {if !label.is_empty() {
                Some(view! { <label class="block text-sm font-medium text-slate-300 mb-1">{label}</label> })
            } else {
                None
            }}
            <select
                class="block w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white focus:border-cyan-500 focus:outline-none focus:ring-1 focus:ring-cyan-500"
                on:change=move |ev| value.set(event_target_value(&ev))
            >
                {if !placeholder.is_empty() {
                    Some(view! { <option value="" disabled=true selected=true>{placeholder}</option> })
                } else {
                    None
                }}
                {options.into_iter().map(|opt| {
                    view! {
                        <option value=opt.value>{opt.label}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
        </div>
    }
}
