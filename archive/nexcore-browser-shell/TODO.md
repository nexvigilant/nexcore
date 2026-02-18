# NexVigilant Browser - TODO

## Immediate (Run the Browser)

- [ ] **Install Tauri CLI** (if not installed)
  ```bash
  cargo install tauri-cli
  ```

- [ ] **First run test**
  ```bash
  cd ~/nexcore/crates/nexcore-browser-shell
  cargo tauri dev
  ```

- [ ] **Verify window opens** with dark theme UI

---

## Phase 3: Tauri-Leptos IPC Connection

- [ ] Add `tauri-sys` or `wasm-bindgen` for frontend invoke
- [ ] Wire `on_navigate` to `browser_navigate` command
- [ ] Wire `on_new_tab` to `browser_new_page` command
- [ ] Wire `on_tab_close` to `browser_close_page` command
- [ ] Add error handling UI (toast/modal)

**Key file:** `src/components/app.rs`

---

## Phase 4: Guardian Integration

- [ ] Create `src/hooks/guardian.rs`
- [ ] Subscribe to `nexcore-browser` event broadcast
- [ ] Detect console errors (PAMP signal)
- [ ] Detect network failures (PAMP signal)
- [ ] Update status bar with threat level
- [ ] Add Guardian alert component

**Key file:** `src/hooks/guardian.rs`

---

## Phase 5: DevTools Panel

- [ ] Create `src/components/devtools/mod.rs`
- [ ] Console tab (logs from CDP)
- [ ] Network tab (requests from CDP)
- [ ] Performance tab (metrics)
- [ ] Toggle devtools visibility
- [ ] Keyboard shortcut (F12)

---

## Phase 6: Real WebView Embedding

- [ ] Replace placeholder with actual Tauri WebView
- [ ] Use `window.add_child()` for embedded view
- [ ] Sync address bar with WebView URL
- [ ] Handle WebView navigation events
- [ ] Implement back/forward history

---

## Phase 7: Polish

- [ ] Custom window chrome (frameless)
- [ ] Favicon loading in tabs
- [ ] Tab drag-and-drop reordering
- [ ] Context menu (right-click)
- [ ] Keyboard shortcuts (Ctrl+T, Ctrl+W, etc.)
- [ ] Settings panel

---

## Tech Debt

- [ ] Fix missing doc warnings (Leptos macro props)
- [ ] Add unit tests for state management
- [ ] Add integration tests for Tauri commands
- [ ] CTVP Phase 0-2 validation

---

## Quick Commands

```bash
# Build
cargo build -p nexcore-browser-shell

# Run dev
cd crates/nexcore-browser-shell && cargo tauri dev

# Check
cargo clippy -p nexcore-browser-shell -- -D warnings

# Test
cargo test -p nexcore-browser-shell
```

---

*Priority: Run first, then wire IPC, then Guardian, then polish.*
