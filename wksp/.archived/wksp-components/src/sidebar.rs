use leptos::prelude::*;

/// Sidebar navigation component
#[component]
pub fn Sidebar(
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "hidden w-64 flex-shrink-0 border-r border-slate-800 bg-slate-950 p-4 lg:block {class}"
    );
    view! {
        <aside class=classes>
            <nav class="space-y-1">
                {children()}
            </nav>
        </aside>
    }
}

/// Sidebar navigation link
#[component]
pub fn SidebarLink(
    href: &'static str,
    #[prop(optional)] active: bool,
    children: Children,
) -> impl IntoView {
    let classes = if active {
        "flex items-center gap-3 rounded-lg bg-slate-800 px-3 py-2 text-sm font-medium text-white"
    } else {
        "flex items-center gap-3 rounded-lg px-3 py-2 text-sm text-slate-400 hover:bg-slate-800 hover:text-white transition-colors"
    };

    view! {
        <a href=href class=classes>
            {children()}
        </a>
    }
}

/// Sidebar section header
#[component]
pub fn SidebarSection(
    title: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="pt-4">
            <h3 class="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-slate-500">{title}</h3>
            <div class="space-y-1">
                {children()}
            </div>
        </div>
    }
}
