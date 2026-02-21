//! Admin: Content management — articles, series, media

use leptos::prelude::*;

#[component]
pub fn ContentAdminPage() -> impl IntoView {
    let active_tab = RwSignal::new("articles");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Content Admin"</h1>
                    <p class="mt-1 text-slate-400">"Articles, series, and media management"</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors">"\u{2190} Dashboard"</a>
            </div>

            <div class="mt-6 flex gap-4 border-b border-slate-800 pb-4">
                {["articles", "series", "media"].into_iter().map(|tab| {
                    view! {
                        <button
                            class=move || { if active_tab.get() == tab {
                                "text-sm font-medium text-amber-400 border-b-2 border-amber-400 pb-1"
                            } else {
                                "text-sm text-slate-400 hover:text-white transition-colors"
                            }}
                            on:click=move |_| active_tab.set(tab)
                        >{tab.to_uppercase()}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <Show when=move || active_tab.get() == "articles">
                <div class="mt-6">
                    <div class="flex items-center justify-between">
                        <h2 class="text-lg font-semibold text-white">"Articles"</h2>
                        <button class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500">"+ New Article"</button>
                    </div>
                    <div class="mt-4 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                        <p class="text-slate-400">"No articles published"</p>
                        <p class="mt-2 text-sm text-slate-500">"Create intelligence articles, regulatory updates, and educational content."</p>
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "series">
                <div class="mt-6">
                    <div class="flex items-center justify-between">
                        <h2 class="text-lg font-semibold text-white">"Article Series"</h2>
                        <button class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500">"+ New Series"</button>
                    </div>
                    <div class="mt-4 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                        <p class="text-slate-400">"No series created"</p>
                        <p class="mt-2 text-sm text-slate-500">"Group related articles into multi-part series."</p>
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "media">
                <div class="mt-6">
                    <h2 class="text-lg font-semibold text-white">"Media Library"</h2>
                    <div class="mt-4 rounded-xl border-2 border-dashed border-slate-700 bg-slate-900/30 p-12 text-center">
                        <p class="text-slate-400">"Drag and drop files here or click to upload"</p>
                        <p class="mt-2 text-sm text-slate-500">"Images, PDFs, videos \u{00B7} Max 50MB"</p>
                        <button class="mt-4 rounded-lg border border-slate-600 px-4 py-2 text-sm text-slate-300 hover:border-slate-500">"Browse Files"</button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
