//! Sign-up page component

use leptos::prelude::*;
use wksp_types::user::AuthState;
use crate::auth::{use_auth, server_sign_up};

/// Sign-up page
#[component]
pub fn SignUpPage() -> impl IntoView {
    let auth = use_auth();
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name_val = name.get();
        let email_val = email.get();
        let password_val = password.get();
        loading.set(true);
        error_msg.set(None);

        leptos::task::spawn_local(async move {
            match server_sign_up(email_val, password_val, name_val).await {
                Ok(data) => {
                    auth.token.set(Some(data.id_token));
                    auth.refresh_token.set(Some(data.refresh_token));
                    auth.user.set(Some(wksp_types::user::UserProfile {
                        uid: data.local_id,
                        email: data.email.clone(),
                        display_name: Some(data.email.clone()),
                        ..Default::default()
                    }));
                    auth.state.set(AuthState::Authenticated);
                    let nav = leptos_router::hooks::use_navigate();
                    nav("/academy", Default::default());
                }
                Err(e) => {
                    error_msg.set(Some(format!("{e}")));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex min-h-screen items-center justify-center px-4">
            <div class="w-full max-w-md space-y-8">
                <div class="text-center">
                    <h1 class="text-3xl font-bold text-white">"Create Account"</h1>
                    <p class="mt-2 text-slate-400">"Join the NexVigilant community"</p>
                </div>

                {move || error_msg.get().map(|msg| view! {
                    <div class="rounded-lg bg-red-500/10 border border-red-500/20 p-4 text-red-400 text-sm">
                        {msg}
                    </div>
                })}

                <form class="space-y-6" on:submit=on_submit>
                    <div>
                        <label class="block text-sm font-medium text-slate-300" for="name">"Full Name"</label>
                        <input
                            id="name"
                            type="text"
                            required=true
                            class="mt-1 block w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none focus:ring-1 focus:ring-cyan-500"
                            placeholder="Your name"
                            on:input=move |ev| name.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-300" for="email">"Email"</label>
                        <input
                            id="email"
                            type="email"
                            required=true
                            class="mt-1 block w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none focus:ring-1 focus:ring-cyan-500"
                            placeholder="you@example.com"
                            on:input=move |ev| email.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-300" for="password">"Password"</label>
                        <input
                            id="password"
                            type="password"
                            required=true
                            minlength="8"
                            class="mt-1 block w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none focus:ring-1 focus:ring-cyan-500"
                            placeholder="At least 8 characters"
                            on:input=move |ev| password.set(event_target_value(&ev))
                        />
                    </div>
                    <button
                        type="submit"
                        class="w-full rounded-lg bg-cyan-600 px-4 py-3 font-semibold text-white hover:bg-cyan-500 disabled:opacity-50 transition-colors"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Creating account..." } else { "Create Account" }}
                    </button>
                </form>

                <p class="text-center text-sm text-slate-400">
                    "Already have an account? "
                    <a href="/signin" class="text-cyan-400 hover:text-cyan-300 font-medium">"Sign in"</a>
                </p>
            </div>
        </div>
    }
}
