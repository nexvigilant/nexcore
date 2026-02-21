//! Contact form thank you page

use leptos::prelude::*;

#[component]
pub fn ContactThankYouPage() -> impl IntoView {
    view! {
        <div class="flex min-h-[60vh] flex-col items-center justify-center px-4 text-center">
            <h1 class="text-3xl font-bold text-white">"Thank You!"</h1>
            <p class="mt-3 max-w-md text-slate-400">
                "We've received your message and will respond within 24 hours. In the meantime, explore what NexVigilant offers."
            </p>
            <a href="/" class="mt-8 text-cyan-400 hover:text-cyan-300 underline">"Back to Home"</a>
        </div>
    }
}
