// NexVigilant Terminal — xterm.js Frontend
// Full Unicode terminal emulation with epistemic color processing

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import { Unicode11Addon } from '@xterm/addon-unicode11';
import { mountChiIndicator } from './chi-indicator';
import { mountRemotePanel } from './remote-panel';

// ── Tauri IPC ──────────────────────────────────────────────────
// Window.__TAURI__ type is declared in chi-indicator.ts.
// Path API accessed via type assertion since the shared type
// only declares core + event.

interface PtySpawnResult {
  session_id: string;
  pid: number;
}

interface PtyExitPayload {
  session_id: string;
  code: number | null;
}

const tauriAvailable: boolean = typeof window.__TAURI__ !== 'undefined';

const invoke = tauriAvailable
  ? window.__TAURI__!.core.invoke
  : async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
      console.log('[mock]', cmd, args);
      return null;
    };

const listen = tauriAvailable
  ? window.__TAURI__!.event.listen
  : async function mockListen<T>(_event: string, _handler: (e: { payload: T }) => void): Promise<() => void> {
      console.log('[mock listen]', _event);
      return function unlisten(): void {};
    };

// ── Epistemic Color Theme ──────────────────────────────────────
// 8 cognitive functions from learning theory, mapped to terminal colors

const _EPISTEMIC_PALETTE: Record<string, string> = {
  question:    '#ff9900', // Orange — interrogative acts
  evidence:    '#44cc44', // Green — grounded data
  hypothesis:  '#cc44ff', // Purple — provisional claims
  convergence: '#00ccff', // Cyan — agreement/synthesis
  divergence:  '#ff4444', // Red — disagreement/branching
  structure:   '#cccc00', // Yellow — organizational markers
  primitive:   '#ff66b2', // Pink — T1 primitives
  relay:       '#00ffcc', // Teal — cross-domain transfers
};

const NV_THEME = {
  background: '#0a0e14',
  foreground: '#e0e8f0',
  cursor: '#34d399',
  cursorAccent: '#0a0e14',
  selectionBackground: '#2a4a6e80',
  selectionForeground: '#ffffff',
  selectionInactiveBackground: '#2a3a4e60',
  black: '#1a2233',
  red: '#f87171',
  green: '#4ade80',
  yellow: '#fbbf24',
  blue: '#60a5fa',
  magenta: '#c084fc',
  cyan: '#34d399',
  white: '#e0e8f0',
  brightBlack: '#556677',
  brightRed: '#fca5a5',
  brightGreen: '#86efac',
  brightYellow: '#fde68a',
  brightBlue: '#93c5fd',
  brightMagenta: '#d8b4fe',
  brightCyan: '#6ee7b7',
  brightWhite: '#f8fafc',
};

// ── Terminal Setup ─────────────────────────────────────────────

const term = new Terminal({
  theme: NV_THEME,
  fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Noto Sans Mono', monospace",
  fontSize: 14,
  lineHeight: 1.3,
  cursorBlink: true,
  cursorStyle: 'bar',
  cursorWidth: 2,
  scrollback: 10000,
  allowProposedApi: true,
  allowTransparency: true,
  drawBoldTextInBrightColors: true,
  minimumContrastRatio: 4.5,
  // @ts-expect-error — unicodeVersion is valid at runtime but missing from TS types
  unicodeVersion: '11',
});

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

const unicode11Addon = new Unicode11Addon();
term.loadAddon(unicode11Addon);
term.unicode.activeVersion = '11';

// ── Mount Terminal ─────────────────────────────────────────────

const container = document.getElementById('terminal-container');
if (container) {
  term.open(container);
}

// Try WebGL for GPU rendering, fall back to canvas
try {
  const webglAddon = new WebglAddon();
  webglAddon.onContextLoss(() => {
    webglAddon.dispose();
    console.warn('WebGL context lost, falling back to canvas renderer');
  });
  term.loadAddon(webglAddon);
  console.log('WebGL renderer active');
} catch (e: unknown) {
  const msg = e instanceof Error ? e.message : String(e);
  console.warn('WebGL not available, using canvas renderer:', msg);
}

fitAddon.fit();

// ── IPC Types ───────────────────────────────────────────────────

interface PaletteEntryInfo {
  label: string;
  action: string;
  source: string;
}

interface AppInfoResult {
  id: string;
  name: string;
  state: string;
  icon: string | null;
}

interface ServiceInfoResult {
  name: string;
  port: number;
  state: string;
  health: string;
  uptime_secs: number;
  restarts: number;
}

interface CloudEventResult {
  timestamp: string;
  event_type: string;
  service: string;
  message: string;
}

// ── Session State ──────────────────────────────────────────────

let sessionId: string | null = null;
let ptyConnected = false;

// ── Mode State ──────────────────────────────────────────────────

const MODES = ['Normal', 'Focus', 'Agent', 'Pairing'] as const;
type TerminalMode = typeof MODES[number];
let currentMode: TerminalMode = 'Normal';

// ── Overlay State ───────────────────────────────────────────────

let paletteOpen = false;
let appLauncherOpen = false;

// ── PTY Data Flow ──────────────────────────────────────────────

// Listen for PTY output events from Tauri backend
listen<string>('pty-output', (event) => {
  if (event.payload) {
    term.write(event.payload);
  }
});

// Listen for PTY exit events
listen<PtyExitPayload>('pty-exit', (event) => {
  ptyConnected = false;
  term.write('\r\n\x1b[33m[Process exited');
  if (event.payload && event.payload.code !== undefined && event.payload.code !== null) {
    term.write(' with code ' + event.payload.code);
  }
  term.write(']\x1b[0m\r\n');
  updateStatusBar();
});

// Send terminal input to PTY
term.onData(async (data: string) => {
  if (!sessionId || !ptyConnected) return;
  try {
    await invoke('pty_write', { sessionId, data });
  } catch (e: unknown) {
    console.error('PTY write failed:', e);
  }
});

// Handle terminal resize
term.onResize(async ({ cols, rows }: { cols: number; rows: number }) => {
  if (!sessionId || !ptyConnected) return;
  try {
    await invoke('pty_resize', { sessionId, cols, rows });
  } catch (e: unknown) {
    console.error('PTY resize failed:', e);
  }
});

// ── Window Resize ──────────────────────────────────────────────

window.addEventListener('resize', () => {
  fitAddon.fit();
});

// ResizeObserver for more reliable container tracking
if (container) {
  const resizeObserver = new ResizeObserver(() => {
    fitAddon.fit();
  });
  resizeObserver.observe(container);
}

// ── Mode Pill ───────────────────────────────────────────────────

function createModePill(): HTMLSpanElement {
  const pill = document.createElement('span');
  pill.className = 'mode-pill';
  pill.dataset.mode = currentMode;
  pill.textContent = currentMode;
  pill.title = 'Ctrl+Shift+M to cycle modes';
  pill.addEventListener('click', () => cycleMode());
  return pill;
}

let modePillEl: HTMLSpanElement | null = null;

function updateModePill(): void {
  if (modePillEl) {
    modePillEl.dataset.mode = currentMode;
    modePillEl.textContent = currentMode;
  }
}

async function cycleMode(): Promise<void> {
  const idx = MODES.indexOf(currentMode);
  const nextIdx = (idx + 1) % MODES.length;
  currentMode = MODES[nextIdx] as TerminalMode;
  updateModePill();
  if (sessionId) {
    // Backend uses lowercase mode names: shell/regulatory/ai/hybrid
    const modeMap: Record<TerminalMode, string> = {
      Normal: 'hybrid',
      Focus: 'shell',
      Agent: 'ai',
      Pairing: 'regulatory',
    };
    try {
      await invoke('terminal_switch_mode', {
        sessionId,
        mode: modeMap[currentMode],
      });
    } catch (e: unknown) {
      console.warn('Mode switch failed:', e);
    }
  }
}

// ── Status Bar ─────────────────────────────────────────────────

function updateStatusBar(): void {
  const statusEl = document.getElementById('connection-status');
  const sessionEl = document.getElementById('session-info');
  const sizeEl = document.getElementById('terminal-size');

  if (statusEl) {
    statusEl.className = 'status-dot ' + (ptyConnected ? 'connected' : 'disconnected');
    statusEl.title = ptyConnected ? 'PTY Connected' : 'Disconnected';
  }
  if (sessionEl) {
    sessionEl.textContent = sessionId
      ? 'Session: ' + sessionId.substring(0, 8) + '...'
      : 'No session';
  }
  if (sizeEl) {
    sizeEl.textContent = term.cols + 'x' + term.rows;
  }
  updateModePill();
}

// ── Command Palette ─────────────────────────────────────────────

let paletteBackdrop: HTMLDivElement | null = null;
let paletteInput: HTMLInputElement | null = null;
let paletteResults: HTMLDivElement | null = null;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function buildPaletteOverlay(): void {
  if (paletteBackdrop) return; // already built

  paletteBackdrop = document.createElement('div');
  paletteBackdrop.className = 'overlay-backdrop hidden';
  paletteBackdrop.addEventListener('click', (e) => {
    if (e.target === paletteBackdrop) closePalette();
  });

  const box = document.createElement('div');
  box.className = 'palette-box';

  paletteInput = document.createElement('input');
  paletteInput.type = 'text';
  paletteInput.className = 'palette-input';
  paletteInput.placeholder = 'Type a command...';
  paletteInput.addEventListener('input', () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => queryPalette(), 150);
  });
  paletteInput.addEventListener('keydown', (e: KeyboardEvent) => {
    if (e.key === 'Escape') closePalette();
  });

  paletteResults = document.createElement('div');
  paletteResults.className = 'palette-results';

  box.appendChild(paletteInput);
  box.appendChild(paletteResults);
  paletteBackdrop.appendChild(box);
  document.body.appendChild(paletteBackdrop);
}

function openPalette(): void {
  buildPaletteOverlay();
  paletteOpen = true;
  if (paletteBackdrop) paletteBackdrop.classList.remove('hidden');
  if (paletteInput) {
    paletteInput.value = '';
    paletteInput.focus();
  }
  if (paletteResults) paletteResults.innerHTML = '';
}

function closePalette(): void {
  paletteOpen = false;
  if (paletteBackdrop) paletteBackdrop.classList.add('hidden');
  term.focus();
}

async function queryPalette(): Promise<void> {
  if (!paletteInput || !paletteResults) return;
  const query = paletteInput.value;
  try {
    const results = await invoke('shell_command_palette', { query }) as PaletteEntryInfo[] | null;
    if (!paletteResults) return;
    paletteResults.innerHTML = '';
    if (results && results.length > 0) {
      for (const entry of results) {
        const item = document.createElement('div');
        item.className = 'palette-item';
        item.innerHTML = '<span class="label">' + escapeHtml(entry.label) + '</span>'
          + '<span class="source">' + escapeHtml(entry.source) + '</span>';
        item.addEventListener('click', () => {
          closePalette();
          term.write('\r\n\x1b[36m[Palette] ' + entry.action + '\x1b[0m\r\n');
        });
        paletteResults.appendChild(item);
      }
    } else if (query.length > 0) {
      const empty = document.createElement('div');
      empty.className = 'palette-item';
      empty.innerHTML = '<span class="label" style="color: var(--text-muted)">No results</span>';
      paletteResults.appendChild(empty);
    }
  } catch (e: unknown) {
    console.warn('Palette query failed:', e);
  }
}

// ── App Launcher ────────────────────────────────────────────────

let appBackdrop: HTMLDivElement | null = null;
let appListEl: HTMLDivElement | null = null;

function buildAppLauncherOverlay(): void {
  if (appBackdrop) return;

  appBackdrop = document.createElement('div');
  appBackdrop.className = 'overlay-backdrop hidden';
  appBackdrop.addEventListener('click', (e) => {
    if (e.target === appBackdrop) closeAppLauncher();
  });

  const box = document.createElement('div');
  box.className = 'app-list';

  const header = document.createElement('div');
  header.className = 'app-list-header';
  header.textContent = 'Applications';

  appListEl = document.createElement('div');

  box.appendChild(header);
  box.appendChild(appListEl);
  appBackdrop.appendChild(box);
  document.body.appendChild(appBackdrop);
}

async function openAppLauncher(): Promise<void> {
  buildAppLauncherOverlay();
  appLauncherOpen = true;
  if (appBackdrop) appBackdrop.classList.remove('hidden');
  if (!appListEl) return;
  appListEl.innerHTML = '';

  try {
    const apps = await invoke('shell_list_apps') as AppInfoResult[] | null;
    if (apps && apps.length > 0) {
      for (const app of apps) {
        const item = document.createElement('div');
        item.className = 'app-item';
        item.innerHTML = '<span class="name">' + escapeHtml(app.name) + '</span>'
          + '<span class="state">' + escapeHtml(app.state) + '</span>';
        item.addEventListener('click', () => launchApp(app.id));
        appListEl!.appendChild(item);
      }
    } else {
      const empty = document.createElement('div');
      empty.className = 'app-item';
      empty.innerHTML = '<span class="name" style="color: var(--text-muted)">No apps registered</span>';
      appListEl.appendChild(empty);
    }
  } catch (e: unknown) {
    console.warn('App list failed:', e);
  }
}

function closeAppLauncher(): void {
  appLauncherOpen = false;
  if (appBackdrop) appBackdrop.classList.add('hidden');
  term.focus();
}

async function launchApp(appId: string): Promise<void> {
  closeAppLauncher();
  try {
    const result = await invoke('shell_launch_app', { appId }) as AppInfoResult | null;
    if (result) {
      term.write('\r\n\x1b[32m[Launched] ' + result.name + '\x1b[0m\r\n');
    }
  } catch (e: unknown) {
    term.write('\r\n\x1b[31m[Launch failed] ' + e + '\x1b[0m\r\n');
  }
}

// ── HTML Escape Utility ─────────────────────────────────────────

function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.appendChild(document.createTextNode(text));
  return div.innerHTML;
}

// ── Tab Management ──────────────────────────────────────────────

interface TabSession {
  id: string;
  label: string;
  tabEl: HTMLDivElement;
}

const tabs: TabSession[] = [];
let activeTabId: string | null = null;

function addTab(sid: string): void {
  const tabBar = document.getElementById('tab-bar');
  const addBtn = document.getElementById('tab-add');
  if (!tabBar || !addBtn) return;

  const label = 'Session ' + (tabs.length + 1);
  const tabEl = document.createElement('div');
  tabEl.className = 'tab';
  tabEl.dataset.sid = sid;

  const labelSpan = document.createElement('span');
  labelSpan.textContent = label;
  tabEl.appendChild(labelSpan);

  const closeBtn = document.createElement('span');
  closeBtn.className = 'close-btn';
  closeBtn.textContent = '\u00d7';
  closeBtn.addEventListener('click', (e: MouseEvent) => {
    e.stopPropagation();
    killTab(sid);
  });
  tabEl.appendChild(closeBtn);

  tabEl.addEventListener('click', () => switchTab(sid));

  tabBar.insertBefore(tabEl, addBtn);
  tabs.push({ id: sid, label, tabEl });
  setActiveTab(sid);
}

function setActiveTab(sid: string): void {
  activeTabId = sid;
  for (const t of tabs) {
    t.tabEl.classList.toggle('active', t.id === sid);
  }
}

function switchTab(sid: string): void {
  if (sid === sessionId) return;
  // For now, we share one terminal — tab switching updates sessionId
  // Future: each tab gets its own Terminal instance
  sessionId = sid;
  ptyConnected = true; // Assume connected if tab exists
  setActiveTab(sid);
  updateStatusBar();
  term.focus();
}

async function killTab(sid: string): Promise<void> {
  try {
    await invoke('pty_kill', { sessionId: sid });
  } catch (e: unknown) {
    console.warn('Kill tab failed:', e);
  }
  const idx = tabs.findIndex((t) => t.id === sid);
  if (idx >= 0) {
    const removed = tabs[idx];
    if (removed) removed.tabEl.remove();
    tabs.splice(idx, 1);
  }
  // Switch to another tab if the killed one was active
  if (sid === sessionId) {
    if (tabs.length > 0) {
      const next = tabs[Math.max(0, idx - 1)];
      if (next) switchTab(next.id);
    } else {
      sessionId = null;
      ptyConnected = false;
      updateStatusBar();
    }
  }
}

// ── Side Panel: Cloud Dashboard + Agent Observation ─────────────

let sidePanelVisible = false;

function toggleSidePanel(): void {
  const panel = document.getElementById('side-panel');
  if (!panel) return;
  sidePanelVisible = !sidePanelVisible;
  panel.classList.toggle('hidden', !sidePanelVisible);
  // Re-fit terminal after panel toggle changes available width
  setTimeout(() => fitAddon.fit(), 50);
}

async function refreshCloudServices(): Promise<void> {
  const listEl = document.getElementById('cloud-services-list');
  if (!listEl) return;
  listEl.innerHTML = '';

  try {
    const services = await invoke('cloud_list_services') as ServiceInfoResult[] | null;
    if (services && services.length > 0) {
      for (const svc of services) {
        const row = document.createElement('div');
        row.className = 'svc-row';

        const name = document.createElement('span');
        name.className = 'svc-name';
        name.textContent = svc.name;

        const badge = document.createElement('span');
        badge.className = 'svc-badge ' + svc.health;
        badge.textContent = svc.health;

        row.appendChild(name);
        row.appendChild(badge);
        listEl.appendChild(row);
      }
    } else {
      const empty = document.createElement('div');
      empty.className = 'agent-empty';
      empty.textContent = 'No services registered';
      listEl.appendChild(empty);
    }
  } catch (e: unknown) {
    console.warn('Cloud services refresh failed:', e);
  }
}

async function refreshAgentEvents(): Promise<void> {
  const listEl = document.getElementById('agent-events-list');
  if (!listEl) return;
  listEl.innerHTML = '';

  try {
    const events = await invoke('cloud_events', { limit: 20 }) as CloudEventResult[] | null;
    if (events && events.length > 0) {
      for (const ev of events) {
        const row = document.createElement('div');
        row.className = 'agent-row';
        row.innerHTML = '<span class="agent-time">' + escapeHtml(ev.timestamp) + '</span> '
          + '<span class="agent-svc">' + escapeHtml(ev.service) + '</span> '
          + '<span class="agent-msg">' + escapeHtml(ev.message) + '</span>';
        listEl.appendChild(row);
      }
    } else {
      const empty = document.createElement('div');
      empty.className = 'agent-empty';
      empty.textContent = 'No recent events';
      listEl.appendChild(empty);
    }
  } catch (e: unknown) {
    console.warn('Agent events refresh failed:', e);
  }
}

// ── Keyboard Shortcuts ─────────────────────────────────────────

document.addEventListener('keydown', async (e: KeyboardEvent) => {
  // Escape: Close any open overlay
  if (e.key === 'Escape') {
    if (paletteOpen) { closePalette(); return; }
    if (appLauncherOpen) { closeAppLauncher(); return; }
  }
  // Ctrl+Shift+N: New session
  if (e.ctrlKey && e.shiftKey && e.key === 'N') {
    e.preventDefault();
    await createSession();
  }
  // Ctrl+Shift+R: Reconnect
  if (e.ctrlKey && e.shiftKey && e.key === 'R') {
    e.preventDefault();
    if (sessionId) {
      await connectPty();
    }
  }
  // Ctrl+Shift+M: Cycle terminal mode
  if (e.ctrlKey && e.shiftKey && e.key === 'M') {
    e.preventDefault();
    await cycleMode();
  }
  // Ctrl+Shift+P: Toggle command palette
  if (e.ctrlKey && e.shiftKey && e.key === 'P') {
    e.preventDefault();
    if (paletteOpen) {
      closePalette();
    } else {
      openPalette();
    }
  }
  // Ctrl+Shift+A: Toggle app launcher
  if (e.ctrlKey && e.shiftKey && e.key === 'A') {
    e.preventDefault();
    if (appLauncherOpen) {
      closeAppLauncher();
    } else {
      await openAppLauncher();
    }
  }
  // Ctrl+Shift+D: Toggle side panel (cloud + agents)
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    e.preventDefault();
    toggleSidePanel();
    if (sidePanelVisible) {
      await refreshCloudServices();
      await refreshAgentEvents();
    }
  }
});

// ── Session Management ─────────────────────────────────────────

/** Resolve the user's home directory portably via Tauri path API. */
async function getHomeDir(): Promise<string> {
  if (tauriAvailable && window.__TAURI__) {
    try {
      // Tauri path plugin — access via invoke since types don't declare path module
      const home = await window.__TAURI__.core.invoke('plugin:path|home_dir') as string;
      if (home) return home;
    } catch {
      // Path plugin not available — fall through to invoke-based fallback
    }
    try {
      // Fallback: ask the Rust backend to resolve $HOME
      const home = await invoke('get_home_dir') as string | null;
      if (home) return home;
    } catch {
      // No backend command either — use default
    }
  }
  return '/root';
}

async function createSession(): Promise<void> {
  try {
    const shell = '/bin/bash';
    const workingDir = await getHomeDir();
    const result = await invoke('pty_spawn', {
      shell,
      workingDir,
      cols: term.cols,
      rows: term.rows,
    }) as PtySpawnResult | null;
    if (result && result.session_id) {
      sessionId = result.session_id;
      ptyConnected = true;
      term.clear();
      addTab(result.session_id);
      updateStatusBar();
    }
  } catch (e: unknown) {
    term.write('\x1b[31mFailed to create session: ' + e + '\x1b[0m\r\n');
  }
}

async function connectPty(): Promise<void> {
  if (!sessionId) return;
  try {
    await invoke('pty_reconnect', { sessionId });
    ptyConnected = true;
    updateStatusBar();
  } catch (e: unknown) {
    term.write('\x1b[31mReconnect failed: ' + e + '\x1b[0m\r\n');
  }
}

// ── Welcome Banner ─────────────────────────────────────────────

function showWelcome(): void {
  const emerald = '\x1b[38;2;52;211;153m';
  const dim = '\x1b[2m';
  const reset = '\x1b[0m';
  const blue = '\x1b[38;2;96;165;250m';

  term.write(emerald + '  NexVigilant Terminal' + reset + dim + ' v0.1.0' + reset + '\r\n');
  term.write(dim + '  Full Unicode ' + reset + '\u2713' + dim + '  GPU Rendering ' + reset + '\u2713' + dim + '  Epistemic Colors ' + reset + '\u2713' + reset + '\r\n');
  term.write(dim + '  \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500' + reset + '\r\n');
  term.write(blue + '  Ctrl+Shift+N' + reset + dim + '  New Session    ' + reset);
  term.write(blue + 'Ctrl+Shift+R' + reset + dim + '  Reconnect' + reset + '\r\n');
  term.write(blue + '  Ctrl+Shift+M' + reset + dim + '  Cycle Mode    ' + reset);
  term.write(blue + 'Ctrl+Shift+P' + reset + dim + '  Command Palette' + reset + '\r\n');
  term.write(blue + '  Ctrl+Shift+A' + reset + dim + '  App Launcher  ' + reset);
  term.write(blue + 'Ctrl+Shift+D' + reset + dim + '  Dashboard' + reset + '\r\n');
  term.write('\r\n');
}

// ── Unicode Test ───────────────────────────────────────────────

function showUnicodeTest(): void {
  const dim = '\x1b[2m';
  const reset = '\x1b[0m';
  const green = '\x1b[38;2;68;204;68m';
  const pink = '\x1b[38;2;255;102;178m';

  term.write(dim + '  Unicode check: ' + reset);
  term.write(green + '\u2713 \u2717 \u25cf \u25cb \u2192 \u2500 \u2502 \u250c \u2510 \u2514 \u2518' + reset);
  term.write(' ' + pink + '\u2203 \u2205 \u2202 \u03c2 \u2192 \u03ba \u03bc \u03c3 \u03bb \u03bd \u03c1 \u03c0 \u221d \u03a3 \u00d7' + reset);
  term.write('\r\n');
  term.write(dim + '  Box drawing:   ' + reset);
  term.write('\u250c\u2500\u2500\u2500\u2500\u2510\r\n');
  term.write(dim + '                 ' + reset);
  term.write('\u2502 OK \u2502\r\n');
  term.write(dim + '                 ' + reset);
  term.write('\u2514\u2500\u2500\u2500\u2500\u2518\r\n');
  term.write('\r\n');
}

// ── Init ───────────────────────────────────────────────────────

async function init(): Promise<void> {
  showWelcome();
  showUnicodeTest();

  if (tauriAvailable) {
    await createSession();
  } else {
    term.write('\x1b[33m  [Dev mode] Tauri not available. Terminal UI preview only.\x1b[0m\r\n');
    term.write('\x1b[2m  Build with: cargo tauri dev\x1b[0m\r\n\r\n');
    // In dev mode, allow typing to echo (for testing)
    term.onData((data: string) => {
      if (!tauriAvailable) {
        if (data === '\r') {
          term.write('\r\n');
        } else if (data === '\x7f') {
          term.write('\b \b');
        } else {
          term.write(data);
        }
      }
    });
  }

  // Wire tab-add button
  const tabAddBtn = document.getElementById('tab-add');
  if (tabAddBtn) {
    tabAddBtn.addEventListener('click', () => createSession());
  }

  // Wire side panel refresh buttons
  const cloudRefresh = document.getElementById('cloud-refresh');
  if (cloudRefresh) {
    cloudRefresh.addEventListener('click', () => refreshCloudServices());
  }
  const agentRefresh = document.getElementById('agent-refresh');
  if (agentRefresh) {
    agentRefresh.addEventListener('click', () => refreshAgentEvents());
  }

  // Mount mode pill into status bar left section
  const statusBarLeft = document.querySelector('.status-bar .left') as HTMLElement | null;
  if (statusBarLeft) {
    modePillEl = createModePill();
    statusBarLeft.appendChild(modePillEl);
  }

  updateStatusBar();

  // Mount chi health indicator into the title-bar right section
  const titleBarRight = document.querySelector('.title-bar .right') as HTMLElement | null;
  if (titleBarRight) {
    mountChiIndicator(titleBarRight).catch((e: unknown) => {
      console.warn('[chi-indicator] mount failed:', e);
    });
  }

  // Mount remote controller panel (Claude accessibility layer)
  const remoteContainer = document.getElementById('remote-panel-container');
  if (remoteContainer) {
    mountRemotePanel(remoteContainer).catch((e: unknown) => {
      console.warn('[remote-panel] mount failed:', e);
    });
  }

  term.focus();
}

init();
