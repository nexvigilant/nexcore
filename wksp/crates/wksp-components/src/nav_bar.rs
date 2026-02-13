use leptos::prelude::*;
use leptos_router::components::A;

/// Bottom navigation bar (mobile pattern — 56px fixed bottom)
/// Tier: T2-C (Sequence + Mapping + Location)
#[component]
pub fn NavBar() -> impl IntoView {
    view! {
        <nav class="nav-bar">
            <NavItem href="/" icon="dashboard" label="Dashboard"/>
            <NavItem href="/store" icon="store" label="Store"/>
            <NavItem href="/signals" icon="signals" label="Signals"/>
            <NavItem href="/guardian" icon="guardian" label="Guardian"/>
            <NavItem href="/brain" icon="brain" label="Brain"/>
            <NavItem href="/more" icon="more" label="More"/>
        </nav>
    }
}

#[component]
fn NavItem(href: &'static str, icon: &'static str, label: &'static str) -> impl IntoView {
    let icon_char = match icon {
        "dashboard" => "\u{25A3}", // ▣
        "store" => "\u{1F3EA}", // 🏪
        "signals" => "\u{26A0}", // ⚠
        "guardian" => "\u{1F6E1}", // 🛡
        "brain" => "\u{1F9E0}", // 🧠
        "more" => "\u{2026}", // …
        _ => "\u{25CB}", // ○
    };

    view! {
        <A href=href attr:class="nav-item">
            <span class="nav-icon">{icon_char}</span>
            <span class="nav-label">{label}</span>
        </A>
    }
}
