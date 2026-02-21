//! Admin: Media manager — images, documents, uploads

use leptos::prelude::*;

#[component]
pub fn MediaPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Media Manager"</h1>
                    <p class="mt-1 text-slate-400">"Upload and manage images, documents, and course assets."</p>
                </div>
                <button class="rounded-lg bg-cyan-500 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-400 transition-colors">
                    "Upload"
                </button>
            </div>

            <div class="mt-8 flex flex-col items-center justify-center rounded-xl border-2 border-dashed border-slate-700 py-16">
                <p class="text-lg text-slate-500">"Drop files here or click Upload"</p>
                <p class="mt-2 text-xs text-slate-600">"Supports PNG, JPG, PDF, MP4 up to 50MB"</p>
            </div>
        </div>
    }
}
