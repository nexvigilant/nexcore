use leptos::prelude::*;

/// Alert variant
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl AlertVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::Info => "border-cyan-500/30 bg-cyan-500/10 text-cyan-400",
            Self::Success => "border-emerald-500/30 bg-emerald-500/10 text-emerald-400",
            Self::Warning => "border-amber-500/30 bg-amber-500/10 text-amber-400",
            Self::Error => "border-red-500/30 bg-red-500/10 text-red-400",
        }
    }
}

/// Alert notification component
#[component]
pub fn Alert(
    #[prop(optional)] variant: AlertVariant,
    #[prop(optional)] title: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "rounded-lg border p-4 {}",
        variant.classes(),
    );

    view! {
        <div class=classes role="alert">
            {if !title.is_empty() {
                Some(view! { <h4 class="mb-1 font-semibold">{title}</h4> })
            } else {
                None
            }}
            <div class="text-sm">{children()}</div>
        </div>
    }
}
