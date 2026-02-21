//! Notifications — activity updates

use leptos::prelude::*;

#[component]
pub fn NotificationsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <div class="flex items-center justify-between">
                <h1 class="text-3xl font-bold text-white">"Notifications"</h1>
                <button class="text-sm text-slate-400 hover:text-white transition-colors">"Mark all read"</button>
            </div>

            <div class="mt-6 flex gap-4 border-b border-slate-800 pb-4">
                <button class="text-sm font-medium text-cyan-400">"All"</button>
                <button class="text-sm text-slate-400 hover:text-white transition-colors">"Mentions"</button>
                <button class="text-sm text-slate-400 hover:text-white transition-colors">"Replies"</button>
            </div>

            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-12 text-center">
                <p class="text-slate-400">"No notifications yet"</p>
                <p class="mt-2 text-sm text-slate-500">"You'll see updates when people interact with your posts or mention you."</p>
            </div>
        </div>
    }
}
