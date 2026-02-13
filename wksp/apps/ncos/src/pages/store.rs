use leptos::prelude::*;
use wksp_components::card::Card;

/// App Store Page
/// Displays available applications in the NexVigilant ecosystem
#[component]
pub fn StorePage() -> impl IntoView {
    // Mock app list from ferrostack.toml
    let apps = vec![
        ("Adventure HUD", "Game HUD with metric tracking", "3001", "experimental"),
        ("Borrow Miner", "Ore mining game with FDA signal checks", "3002", "experimental"),
        ("Education Machine", "Educational content", "3003", "experimental"),
        ("Ferro Clicker", "Clicker game", "3004", "experimental"),
        ("Ferro Explore", "Ferrostack exploration", "3005", "experimental"),
        ("NexCore Watch", "Galaxy Watch 7 Application", "3006", "experimental")
    ];

    view! {
        <div class="page store-page">
            <div class="header">
                <h1>"App Store"</h1>
                <p>"NexVigilant Ecosystem Applications"</p>
            </div>
            
            <div class="app-grid">
                {apps.into_iter().map(|(name, desc, port, tier)| {
                    view! {
                        <AppCard name=name description=desc port=port tier=tier/>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn AppCard(
    #[prop(into)] name: String,
    #[prop(into)] description: String,
    #[prop(into)] port: String,
    #[prop(into)] tier: String
) -> impl IntoView {
    let url = format!("http://localhost:{}", port);
    let tier_class = format!("badge {}", tier);

    view! {
        <Card class="app-card">
            <div class="app-header">
                <h3>{name}</h3>
                <span class={tier_class}>{tier}</span>
            </div>
            <p>{description}</p>
            <a href={url} target="_blank" class="btn-launch">"Launch"</a>
        </Card>
    }
}
