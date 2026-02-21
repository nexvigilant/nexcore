use leptos::prelude::*;

/// Carousel/slider container for cycling through content
#[component]
pub fn Carousel(
    children: Children,
) -> impl IntoView {
    let current = RwSignal::new(0_usize);

    view! {
        <div class="relative overflow-hidden rounded-lg">
            <div class="flex transition-transform">
                {children()}
            </div>
            <div class="absolute bottom-0 left-0 right-0 flex justify-center gap-2 p-4">
                <button
                    class="rounded-full bg-slate-800/80 p-2 text-white hover:bg-slate-700 transition-colors"
                    on:click=move |_| {
                        let c = current.get();
                        if c > 0 { current.set(c - 1); }
                    }
                >
                    "<"
                </button>
                <button
                    class="rounded-full bg-slate-800/80 p-2 text-white hover:bg-slate-700 transition-colors"
                    on:click=move |_| current.set(current.get() + 1)
                >
                    ">"
                </button>
            </div>
        </div>
    }
}

/// Individual carousel slide
#[component]
pub fn CarouselSlide(
    children: Children,
) -> impl IntoView {
    view! {
        <div class="min-w-full flex-shrink-0">
            {children()}
        </div>
    }
}
