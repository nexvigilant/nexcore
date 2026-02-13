use leptos::prelude::*;

/// Table wrapper with dark theme styling
#[component]
pub fn Table(
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!("w-full text-sm {class}");
    view! {
        <div class="overflow-x-auto rounded-lg border border-slate-800">
            <table class=classes>
                {children()}
            </table>
        </div>
    }
}

/// Table header row
#[component]
pub fn TableHeader(children: Children) -> impl IntoView {
    view! {
        <thead class="bg-slate-900/50">
            <tr class="border-b border-slate-800">
                {children()}
            </tr>
        </thead>
    }
}

/// Table header cell
#[component]
pub fn TableHead(
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!("px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-slate-400 {class}");
    view! {
        <th class=classes>{children()}</th>
    }
}

/// Table body
#[component]
pub fn TableBody(children: Children) -> impl IntoView {
    view! {
        <tbody class="divide-y divide-slate-800">
            {children()}
        </tbody>
    }
}

/// Table row
#[component]
pub fn TableRow(
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!("hover:bg-slate-800/50 transition-colors {class}");
    view! {
        <tr class=classes>{children()}</tr>
    }
}

/// Table cell
#[component]
pub fn TableCell(
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!("px-4 py-3 text-slate-300 {class}");
    view! {
        <td class=classes>{children()}</td>
    }
}
