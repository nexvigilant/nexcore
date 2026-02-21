/// PVOS Browser — 15-layer grid with KSB domains
/// Tier: T3 (κ Comparison + Σ Sum + μ Mapping)
use leptos::prelude::*;

use crate::api::pvos;
use crate::components::layer_card::LayerCard;

#[component]
pub fn PvosPage() -> impl IntoView {
    let domains = LocalResource::new(|| pvos::fetch_ksb_domains());
    let (expanded, set_expanded) = signal(None::<String>);

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"PVOS"</h1>
                <p class="page-subtitle">"15-Layer Operating System"</p>
            </header>

            <Suspense fallback=move || view! { <div class="loading">"Loading layers..."</div> }>
                {move || {
                    domains.read().as_ref().map(|result| {
                        match result {
                            Ok(items) => {
                                let total_ksbs: u32 = items.iter().map(|d| d.ksb_count).sum();
                                view! {
                                    <div class="pvos-summary">
                                        <div class="stat-card">
                                            <div class="stat-label">"Layers"</div>
                                            <div class="stat-value">{items.len()}</div>
                                        </div>
                                        <div class="stat-card">
                                            <div class="stat-label">"Total KSBs"</div>
                                            <div class="stat-value">{total_ksbs}</div>
                                        </div>
                                    </div>

                                    <div class="layer-grid">
                                        {items.iter().map(|d| {
                                            let code = d.code.clone();
                                            let domain = d.clone();
                                            let exp = expanded.clone();
                                            let set_exp = set_expanded.clone();
                                            let is_expanded = move || exp.get().as_deref() == Some(&code);
                                            let code_toggle = d.code.clone();
                                            let domain_detail = d.clone();

                                            view! {
                                                <LayerCard
                                                    domain=domain
                                                    on_tap=move || {
                                                        let current = expanded.get();
                                                        if current.as_deref() == Some(&code_toggle) {
                                                            set_exp.set(None);
                                                        } else {
                                                            set_exp.set(Some(code_toggle.clone()));
                                                        }
                                                    }
                                                />
                                                {move || {
                                                    if is_expanded() {
                                                        Some(view! {
                                                            <div class="layer-detail">
                                                                <div class="layer-detail-row">
                                                                    <span class="layer-detail-label">"PVOS Layer"</span>
                                                                    <span class="layer-detail-value">{domain_detail.pvos_layer.clone()}</span>
                                                                </div>
                                                                <div class="layer-detail-row">
                                                                    <span class="layer-detail-label">"Transfer Confidence"</span>
                                                                    <span class="layer-detail-value">{format!("{:.0}%", domain_detail.transfer_confidence * 100.0)}</span>
                                                                </div>
                                                                <div class="layer-detail-row">
                                                                    <span class="layer-detail-label">"KSB Count"</span>
                                                                    <span class="layer-detail-value">{domain_detail.ksb_count}</span>
                                                                </div>
                                                            </div>
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                }}
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <div class="error-card">
                                    <div class="error-icon">"!"</div>
                                    <div class="error-msg">{e.message.clone()}</div>
                                    <p class="error-hint">"Check Settings for API URL"</p>
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
