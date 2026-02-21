//! Academy bookmarks — saved courses, lessons, and resources

use leptos::prelude::*;

#[component]
pub fn BookmarksPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Bookmarks"</h1>
            <p class="mt-2 text-slate-400">"Your saved courses, lessons, and learning resources."</p>

            <div class="mt-8 flex flex-col items-center justify-center py-16 text-center">
                <div class="text-4xl text-slate-600">"🔖"</div>
                <p class="mt-4 text-slate-500">"No bookmarks yet. Browse courses and save items for quick access."</p>
                <a href="/academy/courses" class="mt-4 text-sm text-cyan-400 hover:text-cyan-300">"Browse Courses"</a>
            </div>
        </div>
    }
}
