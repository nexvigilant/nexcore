//! Password reset page component

use leptos::prelude::*;
use crate::auth::server_reset_password;

/// Password reset page
#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let sent = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let email_val = email.get();
        loading.set(true);
        error_msg.set(None);

        leptos::task::spawn_local(async move {
            match server_reset_password(email_val).await {
                Ok(()) => {
                    sent.set(true);
                }
                Err(e) => {
                    error_msg.set(Some(format!("{e}")));
                }
            }
            loading.set(false);
        });
    };

    view! {
        <div class="flex min-h-screen items-center justify-center px-4">
            <div class="w-full max-w-md space-y-8">
                <div class="text-center">
                    <h1 class="text-3xl font-bold text-white">"Reset Password"</h1>
                    <p class="mt-2 text-slate-400">"We'll send you a reset link"</p>
                </div>

                {move || error_msg.get().map(|msg| view! {
                    <div class="rounded-lg bg-red-500/10 border border-red-500/20 p-4 text-red-400 text-sm">
                        {msg}
                    </div>
                })}

                {move || if sent.get() {
                    view! {
                        <div class="rounded-lg bg-emerald-500/10 border border-emerald-500/20 p-6 text-center">
                            <p class="text-emerald-400 font-medium">"Check your email"</p>
                            <p class="mt-2 text-sm text-slate-400">"We sent a password reset link to your email address."</p>
                            <a href="/signin" class="mt-4 inline-block text-cyan-400 hover:text-cyan-300 text-sm">"Back to sign in"</a>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <form class="space-y-6" on:submit=on_submit>
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
                            <button
                                type="submit"
                                class="w-full rounded-lg bg-cyan-600 px-4 py-3 font-semibold text-white hover:bg-cyan-500 disabled:opacity-50 transition-colors"
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() { "Sending..." } else { "Send Reset Link" }}
                            </button>
                        </form>
                    }.into_any()
                }}

                <p class="text-center text-sm text-slate-400">
                    <a href="/signin" class="text-cyan-400 hover:text-cyan-300">"Back to sign in"</a>
                </p>
            </div>
        </div>
    }
}
