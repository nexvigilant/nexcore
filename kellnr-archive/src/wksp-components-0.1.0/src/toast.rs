use leptos::prelude::*;

/// Toast notification variant
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl ToastVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::Info => "border-cyan-500/30 bg-slate-900",
            Self::Success => "border-emerald-500/30 bg-slate-900",
            Self::Warning => "border-amber-500/30 bg-slate-900",
            Self::Error => "border-red-500/30 bg-slate-900",
        }
    }

    fn icon_color(self) -> &'static str {
        match self {
            Self::Info => "text-cyan-400",
            Self::Success => "text-emerald-400",
            Self::Warning => "text-amber-400",
            Self::Error => "text-red-400",
        }
    }
}

/// Individual toast notification
#[component]
pub fn Toast(
    message: String,
    #[prop(optional)] variant: ToastVariant,
    #[prop(optional)] on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let classes = format!(
        "flex items-start gap-3 rounded-lg border p-4 shadow-lg {}",
        variant.classes(),
    );

    view! {
        <div class=classes role="alert">
            <span class=format!("mt-0.5 text-lg {}", variant.icon_color())>"*"</span>
            <p class="flex-1 text-sm text-slate-200">{message}</p>
            {on_dismiss.map(|dismiss| view! {
                <button
                    class="text-slate-500 hover:text-slate-300 transition-colors"
                    on:click=move |_| dismiss.run(())
                >
                    "x"
                </button>
            })}
        </div>
    }
}
