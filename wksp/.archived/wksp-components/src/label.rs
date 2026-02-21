use leptos::prelude::*;

/// Form label component
#[component]
pub fn Label(
    #[prop(optional)] for_id: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <label
            for=for_id
            class="block text-sm font-medium text-slate-300"
        >
            {children()}
        </label>
    }
}
