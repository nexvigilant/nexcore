use leptos::prelude::*;

/// Dropdown menu container
#[component]
pub fn Dropdown(
    trigger: Children,
    children: Children,
) -> impl IntoView {
    let is_open = RwSignal::new(false);

    view! {
        <div class="relative inline-block">
            <div on:click=move |_| is_open.set(!is_open.get())>
                {trigger()}
            </div>
            <div class=move || format!(
                "absolute right-0 z-50 mt-2 min-w-[12rem] rounded-lg border border-slate-700 bg-slate-900 py-1 shadow-xl transition-all {}",
                if is_open.get() { "opacity-100 visible" } else { "opacity-0 invisible" }
            )>
                {children()}
            </div>
        </div>
    }
}

/// Dropdown menu item
#[component]
pub fn DropdownItem(
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "block w-full px-4 py-2 text-left text-sm text-slate-300 hover:bg-slate-800 hover:text-white transition-colors {class}"
    );
    view! {
        <button class=classes>
            {children()}
        </button>
    }
}
