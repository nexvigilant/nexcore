//! Spaced repetition review queue component

use leptos::prelude::*;

use crate::server::schedule_review::{ReviewItem, get_review_queue};

/// Review queue showing items due for spaced repetition review.
#[component]
pub fn ReviewQueue() -> impl IntoView {
    let reviews = Resource::new(|| (), |_| get_review_queue());

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold text-white mb-4">
                "Review Queue"
            </h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading reviews..."</p> }>
                {move || {
                    reviews.get().map(|result| {
                        match result {
                            Ok(items) => {
                                if items.is_empty() {
                                    view! {
                                        <p class="text-gray-500 text-center py-4">
                                            "No reviews due. Great work!"
                                        </p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            {items.into_iter().map(|item| {
                                                view! { <ReviewItemRow item=item /> }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => view! {
                                <p class="text-red-400">{format!("Error: {e}")}</p>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Single review item row
#[component]
fn ReviewItemRow(item: ReviewItem) -> impl IntoView {
    let urgency_color = if item.hours_until_due < 0.0 {
        "text-red-400"
    } else if item.hours_until_due < 4.0 {
        "text-yellow-400"
    } else {
        "text-green-400"
    };

    let due_label = if item.hours_until_due < 0.0 {
        format!("{:.0}h overdue", -item.hours_until_due)
    } else {
        format!("due in {:.0}h", item.hours_until_due)
    };

    let r_pct = (item.retrievability * 100.0).min(100.0);
    let r_color = if item.retrievability >= 0.85 {
        "text-green-400"
    } else if item.retrievability >= 0.50 {
        "text-yellow-400"
    } else {
        "text-red-400"
    };

    view! {
        <div class="flex items-center justify-between bg-gray-750 rounded p-3 border border-gray-700">
            <div>
                <p class="text-white font-medium text-sm">{item.title}</p>
                <p class="text-gray-500 text-xs">{item.subject_name}</p>
            </div>
            <div class="flex items-center gap-4 text-xs">
                <span class={r_color.to_string()}>{format!("R={r_pct:.0}%")}</span>
                <span class={urgency_color.to_string()}>{due_label}</span>
                <span class="text-gray-500">{format!("#{}", item.review_count)}</span>
            </div>
        </div>
    }
}
