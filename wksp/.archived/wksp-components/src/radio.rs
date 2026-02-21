use leptos::prelude::*;

/// Radio button group
#[component]
pub fn RadioGroup(
    #[prop(optional)] _value: Option<RwSignal<String>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="space-y-2" role="radiogroup">
            {children()}
        </div>
    }
}

/// Individual radio button option
#[component]
pub fn RadioItem(
    value: RwSignal<String>,
    option: &'static str,
    #[prop(optional)] label: &'static str,
) -> impl IntoView {
    let is_checked = move || value.get() == option;
    let display_label = if label.is_empty() { option } else { label };

    view! {
        <label class="inline-flex items-center gap-3 cursor-pointer min-h-[44px]">
            <input
                type="radio"
                class="h-5 w-5 border-slate-600 bg-slate-800 text-cyan-600 focus:ring-cyan-500 focus:ring-offset-slate-950"
                prop:checked=is_checked
                on:change=move |_| value.set(option.to_string())
            />
            <span class="text-sm text-slate-300">{display_label}</span>
        </label>
    }
}
