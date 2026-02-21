use leptos::prelude::*;

/// Avatar size
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl AvatarSize {
    fn classes(self) -> &'static str {
        match self {
            Self::Sm => "h-8 w-8 text-xs",
            Self::Md => "h-10 w-10 text-sm",
            Self::Lg => "h-12 w-12 text-base",
            Self::Xl => "h-16 w-16 text-lg",
        }
    }
}

/// Avatar with image or initials fallback
#[component]
pub fn Avatar(
    #[prop(optional)] src: &'static str,
    #[prop(optional)] alt: &'static str,
    #[prop(optional)] initials: &'static str,
    #[prop(optional)] size: AvatarSize,
) -> impl IntoView {
    let classes = format!(
        "inline-flex items-center justify-center rounded-full bg-slate-700 font-medium text-slate-300 overflow-hidden {}",
        size.classes(),
    );

    view! {
        <span class=classes>
            {if !src.is_empty() {
                Some(view! { <img src=src alt=alt class="h-full w-full object-cover"/> }.into_any())
            } else {
                Some(view! { <span>{initials}</span> }.into_any())
            }}
        </span>
    }
}
