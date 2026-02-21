use leptos::prelude::*;

/// Button variant styles
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

impl ButtonVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::Primary => "bg-cyan-600 text-white hover:bg-cyan-500",
            Self::Secondary => "bg-slate-700 text-white hover:bg-slate-600",
            Self::Outline => "border border-slate-700 text-slate-300 hover:bg-slate-800",
            Self::Ghost => "text-slate-400 hover:bg-slate-800 hover:text-white",
            Self::Destructive => "bg-red-600 text-white hover:bg-red-500",
        }
    }
}

/// Button size
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ButtonSize {
    fn classes(self) -> &'static str {
        match self {
            Self::Sm => "h-9 px-3 text-sm",
            Self::Md => "h-11 px-4 text-sm",
            Self::Lg => "h-12 px-6 text-base",
        }
    }
}

/// Button component with variants, sizes, and loading state
#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] loading: bool,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "inline-flex items-center justify-center rounded-lg font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-slate-950 disabled:opacity-50 disabled:pointer-events-none {} {} {}",
        variant.classes(),
        size.classes(),
        class,
    );

    view! {
        <button
            class=classes
            disabled=disabled || loading
        >
            {move || if loading {
                Some(view! {
                    <span class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"/>
                })
            } else {
                None
            }}
            {children()}
        </button>
    }
}
