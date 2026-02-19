use leptos::prelude::*;

/// Badge variant
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Success,
    Warning,
    Destructive,
    Outline,
}

impl BadgeVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::Default => "bg-cyan-600/20 text-cyan-400 border-cyan-600/30",
            Self::Secondary => "bg-slate-700/50 text-slate-300 border-slate-600",
            Self::Success => "bg-emerald-600/20 text-emerald-400 border-emerald-600/30",
            Self::Warning => "bg-amber-600/20 text-amber-400 border-amber-600/30",
            Self::Destructive => "bg-red-600/20 text-red-400 border-red-600/30",
            Self::Outline => "bg-transparent text-slate-400 border-slate-600",
        }
    }
}

/// Small badge/tag component
#[component]
pub fn Badge(
    #[prop(optional)] variant: BadgeVariant,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold {} {}",
        variant.classes(),
        class,
    );

    view! {
        <span class=classes>
            {children()}
        </span>
    }
}
