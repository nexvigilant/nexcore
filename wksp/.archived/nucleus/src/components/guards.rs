//! Route guards — auth protection for routes

use leptos::prelude::*;
use crate::auth::use_auth;
use wksp_types::user::AuthState;

/// Protected route wrapper — shows sign-in prompt if unauthenticated
#[component]
pub fn AuthGuard(children: Children) -> impl IntoView {
    let auth = use_auth();

    let authenticated = move || matches!(auth.state.get(), AuthState::Authenticated);

    view! {
        <Suspense fallback=|| view! { <LoadingAuth/> }>
            {if authenticated() {
                children().into_any()
            } else {
                view! { <UnauthenticatedPrompt/> }.into_any()
            }}
        </Suspense>
    }
}

#[component]
fn LoadingAuth() -> impl IntoView {
    view! {
        <div class="flex min-h-screen items-center justify-center">
            <div class="h-10 w-10 animate-spin rounded-full border-4 border-slate-700 border-t-cyan-500"></div>
        </div>
    }
}

#[component]
fn UnauthenticatedPrompt() -> impl IntoView {
    view! {
        <div class="flex min-h-screen items-center justify-center">
            <div class="text-center">
                <h2 class="text-xl font-semibold text-white">"Sign in required"</h2>
                <p class="mt-2 text-slate-400">"Please sign in to access this page"</p>
                <a href="/signin" class="mt-4 inline-block rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-medium text-white hover:bg-cyan-500 transition-colors">"Sign In"</a>
            </div>
        </div>
    }
}
