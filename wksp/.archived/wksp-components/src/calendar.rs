use leptos::prelude::*;

/// Simple calendar date display (month header + grid placeholder)
#[component]
pub fn Calendar(
    #[prop(optional, default = "Select a date")] placeholder: &'static str,
) -> impl IntoView {
    let selected = RwSignal::new(String::new());

    view! {
        <div class="rounded-lg border border-slate-700 bg-slate-900 p-4">
            <div class="mb-3 flex items-center justify-between">
                <button class="rounded p-1 text-slate-400 hover:bg-slate-800 hover:text-white transition-colors">
                    "<"
                </button>
                <span class="text-sm font-medium text-slate-200">
                    {move || {
                        let val = selected.get();
                        if val.is_empty() { placeholder.to_string() } else { val }
                    }}
                </span>
                <button class="rounded p-1 text-slate-400 hover:bg-slate-800 hover:text-white transition-colors">
                    ">"
                </button>
            </div>
            <div class="grid grid-cols-7 gap-1 text-center text-xs text-slate-500">
                <span>"Su"</span><span>"Mo"</span><span>"Tu"</span><span>"We"</span>
                <span>"Th"</span><span>"Fr"</span><span>"Sa"</span>
            </div>
            <p class="mt-3 text-center text-xs text-slate-600">"Calendar grid renders here"</p>
        </div>
    }
}
