# NexVigilant Browser Tutorial

**Goal:** Build and run a Rust-native browser shell with Tauri + Leptos.

---

## Prerequisites

```bash
# Verify Rust 1.85+
rustc --version

# Verify Tauri CLI
cargo install tauri-cli

# Linux deps (already installed)
# sudo apt-get install -y libwebkit2gtk-4.1-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
```

---

## Step 1: Understand the Architecture

```
┌─────────────────────────────────────────────────────────┐
│               LEPTOS FRONTEND (WASM/CSR)                │
│  ┌─────────┐  ┌──────────┐  ┌────────────────────────┐  │
│  │ TabBar  │  │  NavBar  │  │  Content Placeholder   │  │
│  └─────────┘  └──────────┘  └────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    TAURI IPC LAYER                      │
│  browser_*        content_view_*        guardian_*      │
├─────────────────────────────────────────────────────────┤
│                   RUST BACKEND                          │
│  nexcore-browser (CDP)    nexcore-vigilance (Guardian)  │
└─────────────────────────────────────────────────────────┘
```

**Key insight:** The shell (Tauri + Leptos) is the UI. Actual browsing uses CDP via `nexcore-browser`.

---

## Step 2: Explore the Crate Structure

```bash
cd ~/nexcore/crates/nexcore-browser-shell
tree -L 2
```

```
nexcore-browser-shell/
├── Cargo.toml              # Dependencies: tauri 2, leptos 0.7
├── build.rs                # Tauri build script
├── tauri.conf.json         # App config (name, window size, permissions)
├── capabilities/
│   └── default.json        # Tauri permissions (webview, window ops)
├── dist/
│   └── index.html          # Frontend entry (dark theme CSS)
├── icons/
│   └── icon.png            # App icon
└── src/
    ├── main.rs             # Tauri entry point
    ├── lib.rs              # Crate exports
    ├── commands/
    │   ├── browser.rs      # CDP commands (launch, navigate, etc.)
    │   └── webview.rs      # Tauri WebView commands
    ├── components/
    │   ├── app.rs          # Root Leptos component
    │   ├── tab_bar.rs      # Tab management UI
    │   └── nav_bar.rs      # Address bar + nav buttons
    ├── models/
    │   └── mod.rs          # TabInfo, ShellError
    ├── state/
    │   └── browser_state.rs # Wraps nexcore-browser
    └── hooks/
        └── mod.rs          # Future: Guardian integration
```

---

## Step 3: Build the Project

```bash
cd ~/nexcore

# Build just the shell crate
cargo build -p nexcore-browser-shell

# Or build with release optimizations
cargo build -p nexcore-browser-shell --release
```

**Expected output:**
```
   Compiling nexcore-browser-shell v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

---

## Step 4: Run the Browser (Development Mode)

```bash
cd ~/nexcore/crates/nexcore-browser-shell

# Run with Tauri CLI (hot reload)
cargo tauri dev
```

**What happens:**
1. Tauri compiles the Rust backend
2. Serves `dist/index.html` at `localhost:1420`
3. Opens a native window with WebKit

---

## Step 5: Understand the UI Flow

### TabBar Component (`components/tab_bar.rs`)

```rust
// State: Vec<TabData> with reactive signals
let (tabs, set_tabs) = signal(vec![TabData { ... }]);

// Callbacks passed as props
on_select: Callback<String>   // Tab clicked
on_close: Callback<String>    // X button clicked
on_new_tab: Callback<()>      // + button clicked
```

### NavBar Component (`components/nav_bar.rs`)

```rust
// Address bar with Enter-to-navigate
on:keydown=move |ev| {
    if ev.key() == "Enter" {
        on_navigate.run(input_value.get());
    }
}
```

### App Component (`components/app.rs`)

Wires everything together:
```rust
<TabBar tabs=tabs on_select=... on_close=... on_new_tab=... />
<NavBar current_url=... on_navigate=... on_back=... />
```

---

## Step 6: Understand the Backend Commands

### Browser CDP Commands (`commands/browser.rs`)

| Command | Purpose |
|---------|---------|
| `browser_launch` | Start Chrome via CDP |
| `browser_new_page` | Open new tab |
| `browser_navigate` | Go to URL |
| `browser_list_pages` | Get all tabs |
| `browser_close_page` | Close tab |

### WebView Commands (`commands/webview.rs`)

| Command | Purpose |
|---------|---------|
| `content_view_create` | Spawn Tauri WebView window |
| `content_view_navigate` | Navigate WebView |
| `content_view_resize` | Resize/reposition |
| `content_view_close` | Destroy WebView |

---

## Step 7: Connect Frontend to Backend

**Current state:** UI works, but doesn't invoke Tauri commands yet.

**TODO:** Add `wasm-bindgen-futures` and Tauri invoke calls:

```rust
// In app.rs, update on_navigate callback:
use wasm_bindgen_futures::spawn_local;

let on_navigate = Callback::new(move |url: String| {
    spawn_local(async move {
        // Invoke Tauri command
        let result = tauri::invoke::<_, TabInfo>("browser_navigate", &url).await;
        // Update UI based on result
    });
});
```

---

## Step 8: Test Individual Commands

```bash
# In a separate terminal, test the CLI
cd ~/nexcore
cargo run -p nexcore-browser -- navigate "https://example.com"
```

---

## Step 9: Next Steps (Phase 3+)

| Phase | Focus | Files |
|-------|-------|-------|
| **3** | Guardian integration | `hooks/guardian.rs` |
| **4** | DevTools panel | `components/devtools.rs` |
| **5** | CDP event streaming | Real-time console/network |

---

## Quick Reference

```bash
# Build
cargo build -p nexcore-browser-shell

# Run (dev)
cd crates/nexcore-browser-shell && cargo tauri dev

# Run (release)
cargo tauri build

# Check for errors
cargo clippy -p nexcore-browser-shell

# Run tests
cargo test -p nexcore-browser-shell
```

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `webkit2gtk not found` | `sudo apt install libwebkit2gtk-4.1-dev` |
| `Permission denied` | Check `capabilities/default.json` |
| `Build script failed` | Ensure `tauri.conf.json` is valid |
| `Window doesn't open` | Check `dist/index.html` exists |

---

## Architecture Decisions

1. **Why Tauri over Electron?** Rust-native, smaller binary, WebKit security
2. **Why Leptos over Yew?** 0.7 signals API, fine-grained reactivity
3. **Why separate nexcore-browser?** Shared CDP foundation for MCP + shell
4. **Why CSR mode?** Tauri doesn't need SSR; simpler build

---

*Built with Primitive Codex compliance: T1 (State/Sequence) → T2-C (Reactive) → T3 (Browser Shell)*
