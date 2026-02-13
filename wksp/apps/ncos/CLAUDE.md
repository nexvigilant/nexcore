# NCOS — NexCore Operating System (Mobile PWA)

## What This Is
Mobile Progressive Web App providing access to all NexCore capabilities. Built with Leptos 0.7 SSR + hydration (100% Rust). Installable on any phone via browser "Add to Home Screen".

## Quick Start

```bash
# Prerequisites (already installed)
cargo-leptos --version     # 0.3.4
rustup target list --installed | grep wasm32  # wasm32-unknown-unknown

# Development (hot-reload)
cd ~/projects/ncos && cargo leptos watch
# Opens http://127.0.0.1:3000 — SSR + WASM hydration

# Production build
cargo leptos build --release
# Output: target/release/ncos (server binary) + target/site/ (static assets)

# Run production binary
NCOS_API_URL=https://nexcore-api-URL target/release/ncos
```

## Verification Gates (all must pass)

```bash
cargo check --features ssr
cargo check --features hydrate --target wasm32-unknown-unknown
cargo clippy --features ssr -- -D warnings
cargo clippy --features hydrate --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all -- --check
cargo doc --features ssr --no-deps
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NCOS_API_URL` | `http://localhost:3030` | nexcore-api base URL |
| `RUST_LOG` | `ncos=info` | Tracing filter |

## Architecture

```
Phone Browser → NCOS PWA (Leptos SSR :3000)
                         ↓ reqwest (server-side)
               nexcore-api (Cloud Run :3030, 120+ endpoints)
```

**Dual-target build:**
- `--features ssr` → native binary (Axum server, reqwest HTTP client)
- `--features hydrate` → WASM bundle (gloo-net client, browser hydration)
- `cargo-leptos` orchestrates both targets automatically

## File Map (23 Rust files)

```
src/
├── main.rs              # Axum entry point (SSR)
├── lib.rs               # Crate root: exports App, shell()
├── app.rs               # Root component, Router (10 routes), NavBar
├── auth.rs              # AuthContext (api_url + api_key RwSignals)
├── api_client.rs        # Typed API client (reqwest, 20+ methods)
├── components/
│   ├── mod.rs           # Re-exports
│   ├── card.rs          # Card, CardLoading
│   ├── input.rs         # TextInput, NumberInput (step, decimal)
│   ├── metric.rs        # Metric, ThresholdMetric
│   ├── modal.rs         # BottomSheet (children: Children)
│   ├── nav_bar.rs       # Bottom tab navigation (5 tabs)
│   └── status_badge.rs  # StatusBadge (Active/Warning/Error/Unknown)
└── pages/
    ├── mod.rs            # Re-exports
    ├── dashboard.rs      # System overview, quick actions
    ├── signals.rs        # 2x2 contingency table, 5 metrics
    ├── guardian.rs       # Homeostasis loop controls, risk eval
    ├── brain.rs          # Session/artifact browser
    ├── causality.rs      # Naranjo, WHO-UMC assessment
    ├── pvdsl.rs          # DSL console
    ├── skills.rs         # Skills registry browser
    ├── benefit_risk.rs   # QBRI calculator
    ├── settings.rs       # API URL/key config, theme toggle
    └── more.rs           # Links to Causality/PVDSL/Skills/B-R
```

## API Client Methods (`src/api_client.rs`)

Server-side only (`#[cfg(feature = "ssr")]`). All return `Result<T, String>`.

| Method | HTTP | Endpoint | Response Type |
|--------|------|----------|---------------|
| `health()` | GET | /health/ready | `HealthResponse` |
| `guardian_status()` | GET | /api/v1/guardian/status | `GuardianStatus` |
| `vigil_status()` | GET | /api/v1/vigil/status | `VigilStatus` |
| `llm_stats()` | GET | /api/v1/vigil/llm/stats | `LlmStats` |
| `signal_complete()` | POST | /api/v1/pv/signal/complete | `SignalResult` |
| `guardian_tick()` | POST | /api/v1/guardian/tick | `serde_json::Value` |
| `guardian_evaluate()` | POST | /api/v1/guardian/evaluate | `GuardianEvalResult` |
| `guardian_pause()` | POST | /api/v1/guardian/pause | `serde_json::Value` |
| `guardian_resume()` | POST | /api/v1/guardian/resume | `serde_json::Value` |
| `guardian_reset()` | POST | /api/v1/guardian/reset | `serde_json::Value` |
| `brain_sessions()` | GET | /api/v1/brain/sessions | `Vec<BrainSession>` |
| `brain_session_load()` | GET | /api/v1/brain/sessions/{id} | `BrainSession` |
| `brain_artifact_get()` | GET | /api/v1/brain/artifacts/{s}/{n} | `BrainArtifact` |
| `skills_list()` | GET | /api/v1/skills | `Vec<SkillInfo>` |
| `pvdsl_execute()` | POST | /api/v1/pvdsl/execute | `PvdslResult` |
| `pvdsl_compile()` | POST | /api/v1/pvdsl/compile | `serde_json::Value` |
| `pvdsl_functions()` | GET | /api/v1/pvdsl/functions | `serde_json::Value` |
| `naranjo()` | POST | /api/v1/pv/naranjo | `NaranjoCausality` |
| `who_umc()` | POST | /api/v1/pv/who-umc | `WhoUmcCausality` |
| `qbri_compute()` | POST | /api/v1/benefit-risk/qbri/compute | `QbriResult` |
| `signal_thresholds()` | GET | /api/v1/signal/thresholds | `serde_json::Value` |

## Mobile Device Linking

### Install PWA on Phone
1. Run NCOS server (locally or on Cloud Run)
2. Open the URL in Chrome/Safari on phone
3. Tap browser menu → "Add to Home Screen" / "Install App"
4. NCOS appears as native app icon on home screen

### Local Network (development)
```bash
# Find your machine's local IP
ip addr show | grep 'inet ' | grep -v 127.0.0.1
# e.g., 192.168.1.100

# Start NCOS on all interfaces
cd ~/projects/ncos && cargo leptos watch
# Default binds 127.0.0.1:3000 — for LAN access, set:
# [package.metadata.leptos] site-addr = "0.0.0.0:3000" in Cargo.toml

# On phone browser: http://192.168.1.100:3000
```

### Cloud Run Deployment (production — accessible from anywhere)
```bash
# Build Docker image
docker build -t ncos .
# Push to GCR and deploy
gcloud run deploy ncos --image=gcr.io/PROJECT/ncos \
  --set-env-vars="NCOS_API_URL=https://nexcore-api-URL" \
  --allow-unauthenticated
# Phone opens the Cloud Run URL → Install PWA
```

### PWA Assets
- `public/manifest.json` — standalone display, theme #1a1a2e, portrait
- `public/sw.js` — cache-first static, network-first /api/*
- `public/icons/` — icon-192.png, icon-512.png (required for PWA install prompt)

## Leptos 0.7 Patterns (Lessons Learned)

### Children vs ViewFn
- `children: Children` = `FnOnce` — consumed once per render. Standard.
- `children: ViewFn` = `Fn` — callable multiple times. For `<Show>`/`<Suspense>` children.
- **Gotcha**: View macro children satisfy `Children`, NOT `ViewFn`. If a component needs ViewFn, callers must explicitly construct it.
- **Pattern for modals**: Use `Children` in the component, wrap with `<Show>` at call site.

### Signal Patterns
- `Signal::derive(move || signal.get())` — explicit type for Leptos macros
- `event_target_value(&ev)` — SSR-safe, works on both server and client
- Avoid `event_target::<HtmlInputElement>` (requires web-sys on server)

### Component Props
- `#[prop(optional)]` with `&'static str` defaults to `""` (empty)
- `#[prop(optional)]` with `bool` defaults to `false`
- `Callback<()>` for event handlers, `Signal<T>` for reactive reads

## CSS Strategy
Custom CSS, no framework. `style/main.css`:
- CSS custom properties for theming (dark default, light via toggle)
- 44px minimum touch targets (Apple HIG)
- CSS Grid for dashboard cards
- Bottom nav bar 56px + safe-area-inset
- `@media (min-width: 768px)` for tablet/desktop

## Edition & Dependencies
- **Edition 2021** (standalone project, NOT in nexcore workspace)
- Leptos 0.7, Axum 0.7, reqwest 0.12, serde 1, thiserror 2
- WASM: gloo-net 0.6, gloo-storage 0.3, web-sys 0.3, wasm-bindgen 0.2
