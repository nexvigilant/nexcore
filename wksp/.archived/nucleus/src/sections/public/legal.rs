use leptos::prelude::*;

#[component]
pub fn PrivacyPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Privacy Policy"</h1>
            <p class="mt-6 text-slate-400">"Your privacy matters to NexVigilant."</p>
        </div>
    }
}

#[component]
pub fn TermsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Terms of Service"</h1>
            <p class="mt-6 text-slate-400">"Terms and conditions for using NexVigilant services."</p>
        </div>
    }
}
