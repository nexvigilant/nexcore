use leptos::prelude::*;

/// Breadcrumb navigation trail
#[component]
pub fn Breadcrumb(
    children: Children,
) -> impl IntoView {
    view! {
        <nav aria-label="Breadcrumb">
            <ol class="flex items-center gap-2 text-sm text-slate-400">
                {children()}
            </ol>
        </nav>
    }
}

/// Individual breadcrumb item
#[component]
pub fn BreadcrumbItem(
    #[prop(optional)] href: &'static str,
    #[prop(optional)] active: bool,
    children: Children,
) -> impl IntoView {
    let content = children();

    if active {
        view! {
            <li class="flex items-center gap-2">
                <span class="text-slate-600">"/"</span>
                <span class="font-medium text-white">{content}</span>
            </li>
        }
        .into_any()
    } else if !href.is_empty() {
        view! {
            <li class="flex items-center gap-2">
                <span class="text-slate-600">"/"</span>
                <a href=href class="hover:text-white transition-colors">{content}</a>
            </li>
        }
        .into_any()
    } else {
        view! {
            <li class="flex items-center gap-2">
                <span class="text-slate-600">"/"</span>
                <span>{content}</span>
            </li>
        }
        .into_any()
    }
}
