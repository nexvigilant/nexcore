use leptos::prelude::*;

/// Form wrapper with consistent spacing
#[component]
pub fn FormGroup(
    children: Children,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            {children()}
        </div>
    }
}

/// Form field with label and optional error
#[component]
pub fn FormField(
    label: &'static str,
    #[prop(optional)] error: &'static str,
    #[prop(optional)] required: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="space-y-1.5">
            <label class="block text-sm font-medium text-slate-300">
                {label}
                {if required {
                    Some(view! { <span class="ml-1 text-red-400">"*"</span> })
                } else {
                    None
                }}
            </label>
            {children()}
            {if !error.is_empty() {
                Some(view! { <p class="text-xs text-red-400">{error}</p> })
            } else {
                None
            }}
        </div>
    }
}

/// Form description/help text
#[component]
pub fn FormDescription(
    children: Children,
) -> impl IntoView {
    view! {
        <p class="text-xs text-slate-500">{children()}</p>
    }
}
