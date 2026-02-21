//! Skills registry browser — browse and search nexcore skills

use leptos::prelude::*;

#[component]
pub fn SkillsPage() -> impl IntoView {
    let skills = RwSignal::new(Vec::<(String, String, Vec<String>)>::new());
    let search = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Skills Registry"</h1>
            <p class="mt-2 text-slate-400">"Browse and manage nexcore skills ecosystem"</p>

            // Search bar
            <div class="mt-8 flex gap-3">
                <input type="search"
                    class="flex-1 rounded-lg border border-slate-700 bg-slate-800 px-4 py-2.5 text-white placeholder:text-slate-500 focus:border-amber-500 focus:outline-none"
                    placeholder="Search skills..."
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
                <button
                    class="rounded-lg bg-amber-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-amber-500 transition-colors disabled:opacity-50"
                    disabled=move || loading.get()
                    on:click=move |_| loading.set(true)
                >"Load Skills"</button>
            </div>

            // Loading
            <Show when=move || loading.get()>
                <div class="mt-6 space-y-3">
                    <div class="h-24 animate-pulse rounded-xl bg-slate-800"></div>
                    <div class="h-24 animate-pulse rounded-xl bg-slate-800"></div>
                    <div class="h-24 animate-pulse rounded-xl bg-slate-800"></div>
                </div>
            </Show>

            // Skills list
            <div class="mt-6">
                {move || {
                    let all_skills = skills.get();
                    let filter = search.get().to_lowercase();
                    let filtered: Vec<_> = if filter.is_empty() {
                        all_skills
                    } else {
                        all_skills.into_iter()
                            .filter(|(name, desc, _)| {
                                name.to_lowercase().contains(&filter)
                                || desc.to_lowercase().contains(&filter)
                            })
                            .collect()
                    };

                    if filtered.is_empty() && !loading.get() {
                        view! {
                            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                                <p class="text-slate-400">"No skills loaded"</p>
                                <p class="mt-2 text-sm text-slate-500">"Tap Load Skills to fetch from nexcore API."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-3">
                                {filtered.into_iter().map(|(name, desc, tags)| {
                                    view! {
                                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
                                            <h3 class="font-semibold text-white">{name}</h3>
                                            <p class="mt-1 text-sm text-slate-400">{desc}</p>
                                            <div class="mt-2 flex flex-wrap gap-1.5">
                                                {tags.into_iter().map(|t| {
                                                    view! {
                                                        <span class="rounded-full bg-slate-800 px-2 py-0.5 text-xs text-slate-400">{t}</span>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
