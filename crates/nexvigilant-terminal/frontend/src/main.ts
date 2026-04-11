// NexVigilant Terminal — Simplified xterm.js Frontend
// PTY terminal with tabs, Wayland keyboard bypass, fallback input bar.

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import { Unicode11Addon } from '@xterm/addon-unicode11';

// ── Tauri IPC ──────────────────────────────────────────────────

declare global {
  interface Window {
    __TAURI__?: {
      core: { invoke: (cmd: string, args?: unknown) => Promise<unknown> };
      event: {
        listen: <T>(
          event: string,
          handler: (e: { payload: T }) => void,
        ) => Promise<() => void>;
      };
    };
  }
}

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
      return function unlisten(): void {};
    };

// ── Theme ──────────────────────────────────────────────────────

const NV_THEME = {
  background: '#0a0e14',
  foreground: '#e0e8f0',
  cursor: '#34d399',
  cursorAccent: '#0a0e14',
  selectionBackground: '#2a4a6e80',
  selectionForeground: '#ffffff',
  black: '#1a2233',
  red: '#f87171',
  green: '#4ade80',
  yellow: '#fbbf24',
  blue: '#60a5fa',
  magenta: '#c084fc',
  cyan: '#22d3ee',
  white: '#e0e8f0',
  brightBlack: '#556677',
  brightRed: '#fca5a5',
  brightGreen: '#86efac',
  brightYellow: '#fde68a',
  brightBlue: '#93c5fd',
  brightMagenta: '#d8b4fe',
  brightCyan: '#67e8f9',
  brightWhite: '#f8fafc',
};

// ── State ──────────────────────────────────────────────────────

let sessionId: string | null = null;
let ptyConnected = false;
let activeTabId: string | null = null;

interface TabSession {
  id: string;
  label: string;
  tabEl: HTMLElement;
  term: Terminal;
  fitAddon: FitAddon;
  termContainer: HTMLDivElement;
  connected: boolean;
}

const tabs: TabSession[] = [];

// ── Global Terminal (welcome banner only) ──────────────────────

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
});

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

const unicode11Addon = new Unicode11Addon();
term.loadAddon(unicode11Addon);
term.unicode.activeVersion = '11';

const container = document.getElementById('terminal-container');
if (container) {
  term.open(container);

  container.addEventListener('click', () => term.focus());
  container.addEventListener('mousedown', () => term.focus());
}

try {
  const webglAddon = new WebglAddon();
  webglAddon.onContextLoss(() => webglAddon.dispose());
  term.loadAddon(webglAddon);
} catch {
  // canvas fallback
}

fitAddon.fit();

// ── PTY Output ─────────────────────────────────────────────────

listen<string>('pty-output', (event) => {
  if (!event.payload) return;
  const activeTab = tabs.find((t) => t.id === sessionId);
  if (activeTab) {
    activeTab.term.write(event.payload);
  } else {
    term.write(event.payload);
  }
});

listen<PtyExitPayload>('pty-exit', (event) => {
  const exitSid = event.payload ? event.payload.session_id : null;
  if (exitSid && exitSid !== sessionId) return;

  ptyConnected = false;
  const activeTab = tabs.find((t) => t.id === exitSid);
  const target = activeTab ? activeTab.term : term;

  target.write('\r\n\x1b[33m[Process exited');
  if (event.payload && event.payload.code !== undefined && event.payload.code !== null) {
    target.write(' with code ' + event.payload.code);
  }
  target.write(']\x1b[0m\r\n');

  if (activeTab) activeTab.connected = false;
  updateStatusBar();

  // Auto-respawn
  target.write('\x1b[2m  Restarting...\x1b[0m\r\n');
  setTimeout(async () => { await createSession(); }, 500);
});

// ── PTY Input (global term) ────────────────────────────────────

term.onData(async (data: string) => {
  if (!sessionId) return;
  try {
    await invoke('pty_write', { sessionId, data });
    if (!ptyConnected) { ptyConnected = true; updateStatusBar(); }
  } catch (e: unknown) {
    console.error('PTY write failed:', e);
    ptyConnected = false;
    updateStatusBar();
  }
});

term.onResize(async ({ cols, rows }: { cols: number; rows: number }) => {
  if (!sessionId || !ptyConnected) return;
  try { await invoke('pty_resize', { sessionId, cols, rows }); } catch {}
});

// ── Window Resize ──────────────────────────────────────────────

window.addEventListener('resize', () => {
  fitAddon.fit();
  const activeTab = tabs.find((t) => t.id === activeTabId);
  if (activeTab) activeTab.fitAddon.fit();
});

if (container) {
  new ResizeObserver(() => {
    fitAddon.fit();
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (activeTab) activeTab.fitAddon.fit();
  }).observe(container);
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
    sessionEl.textContent = sessionId ? 'Session: ' + sessionId : 'No session';
  }
  if (sizeEl) {
    const t = tabs.find((t) => t.id === activeTabId);
    const active = t ? t.term : term;
    sizeEl.textContent = active.cols + 'x' + active.rows;
  }
}

// ── Tab Management ─────────────────────────────────────────────

function createTerminalInstance(): { term: Terminal; fitAddon: FitAddon; container: HTMLDivElement } {
  const newTerm = new Terminal({
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
  });

  const newFit = new FitAddon();
  newTerm.loadAddon(newFit);

  const newUnicode = new Unicode11Addon();
  newTerm.loadAddon(newUnicode);
  newTerm.unicode.activeVersion = '11';

  const termDiv = document.createElement('div');
  termDiv.className = 'tab-terminal-container';
  termDiv.style.display = 'none';
  termDiv.style.width = '100%';
  termDiv.style.height = '100%';

  return { term: newTerm, fitAddon: newFit, container: termDiv };
}

function addTab(sid: string): void {
  const tabBar = document.getElementById('tab-bar');
  const addBtn = document.getElementById('tab-add');
  const mainContainer = document.getElementById('terminal-container');
  if (!tabBar || !addBtn || !mainContainer) return;

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

  const { term: tabTerm, fitAddon: tabFit, container: termDiv } = createTerminalInstance();
  mainContainer.appendChild(termDiv);
  tabTerm.open(termDiv);

  // Wire per-tab input
  tabTerm.onData(async (data: string) => {
    if (!sid) return;
    try { await invoke('pty_write', { sessionId: sid, data }); } catch {}
  });

  tabTerm.onResize(async ({ cols, rows }: { cols: number; rows: number }) => {
    const tab = tabs.find((t) => t.id === sid);
    if (!tab || !tab.connected) return;
    try { await invoke('pty_resize', { sessionId: sid, cols, rows }); } catch {}
  });

  try {
    const webgl = new WebglAddon();
    webgl.onContextLoss(() => webgl.dispose());
    tabTerm.loadAddon(webgl);
  } catch { /* canvas fallback */ }

  tabs.push({ id: sid, label, tabEl, term: tabTerm, fitAddon: tabFit, termContainer: termDiv, connected: true });
  switchTab(sid);
}

function setActiveTab(sid: string): void {
  activeTabId = sid;
  for (const t of tabs) {
    t.tabEl.classList.toggle('active', t.id === sid);
    t.termContainer.style.display = t.id === sid ? 'block' : 'none';
  }
  // Hide global terminal when tabs are active (prevents focus stealing)
  const globalXterm = container ? container.querySelector(':scope > .xterm') as HTMLElement : null;
  if (globalXterm) {
    globalXterm.style.display = tabs.length > 0 ? 'none' : '';
  }
}

function switchTab(sid: string): void {
  if (sid === sessionId) return;
  sessionId = sid;
  const tab = tabs.find((t) => t.id === sid);
  if (tab) ptyConnected = tab.connected;
  setActiveTab(sid);
  updateStatusBar();
  if (tab) {
    setTimeout(() => {
      tab.fitAddon.fit();
      tab.term.focus();
    }, 10);
  }
}

async function killTab(sid: string): Promise<void> {
  try { await invoke('pty_kill', { sessionId: sid }); } catch {}
  const idx = tabs.findIndex((t) => t.id === sid);
  if (idx >= 0) {
    const removed = tabs[idx];
    if (removed) {
      removed.tabEl.remove();
      removed.term.dispose();
      removed.termContainer.remove();
    }
    tabs.splice(idx, 1);
  }
  if (sid === sessionId) {
    if (tabs.length > 0) {
      const next = tabs[Math.max(0, idx - 1)];
      if (next) switchTab(next.id);
    } else {
      sessionId = null;
      ptyConnected = false;
      updateStatusBar();
      // Re-show global terminal
      const globalXterm = container ? container.querySelector(':scope > .xterm') as HTMLElement : null;
      if (globalXterm) globalXterm.style.display = '';
    }
  }
}

// ── Session Management ─────────────────────────────────────────

async function getHomeDir(): Promise<string> {
  if (tauriAvailable && window.__TAURI__) {
    try {
      const home = await window.__TAURI__.core.invoke('plugin:path|home_dir') as string;
      if (home) return home;
    } catch {}
    try {
      const home = await invoke('get_home_dir') as string | null;
      if (home) return home;
    } catch {}
  }
  return '/home';
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

// ── Keyboard Shortcuts ─────────────────────────────────────────

document.addEventListener('keydown', async (e: KeyboardEvent) => {
  if (e.ctrlKey && e.shiftKey && e.key === 'N') {
    e.preventDefault();
    await createSession();
  }
  if (e.ctrlKey && e.shiftKey && e.key === 'R') {
    e.preventDefault();
    if (sessionId) await connectPty();
  }
});

// ── Wayland Keyboard Bypass ────────────────────────────────────
// WebKitGTK on Wayland can't route keyboard events to xterm.js textarea.
// This interceptor detects the failure and forwards keystrokes directly to PTY.

let waylandBypassSuppressed = false;
let waylandBypassActive = false;

{
  const xtermTextarea = container ? container.querySelector('textarea') : null;

  if (xtermTextarea && tauriAvailable) {
    let xtermReceivedInput = false;
    xtermTextarea.addEventListener('input', () => { xtermReceivedInput = true; }, { once: true });
    setTimeout(() => {
      if (!xtermReceivedInput) {
        waylandBypassActive = true;
        console.log('[wayland-bypass] Activated');
      }
    }, 2000);
  }

  term.onData(() => { waylandBypassSuppressed = true; });
}

function keyToData(e: KeyboardEvent): string | null {
  if (e.ctrlKey && e.shiftKey) return null;
  if (e.altKey && e.key.length > 1) return null;
  if (e.metaKey) return null;

  if (e.ctrlKey) {
    const code = e.key.toLowerCase().charCodeAt(0);
    if (code >= 97 && code <= 122) return String.fromCharCode(code - 96);
    return null;
  }

  switch (e.key) {
    case 'Enter': return '\r';
    case 'Backspace': return '\x7f';
    case 'Tab': return '\t';
    case 'Escape': return '\x1b';
    case 'ArrowUp': return '\x1b[A';
    case 'ArrowDown': return '\x1b[B';
    case 'ArrowRight': return '\x1b[C';
    case 'ArrowLeft': return '\x1b[D';
    case 'Home': return '\x1b[H';
    case 'End': return '\x1b[F';
    case 'Delete': return '\x1b[3~';
    case 'PageUp': return '\x1b[5~';
    case 'PageDown': return '\x1b[6~';
    default: break;
  }

  if (e.key.length > 1) return null;
  return e.key;
}

document.addEventListener('keydown', async (e: KeyboardEvent) => {
  if (!waylandBypassActive) return;
  const active = document.activeElement;
  if (active && active.tagName === 'INPUT') return;

  waylandBypassSuppressed = false;
  await Promise.resolve();
  if (waylandBypassSuppressed) return;

  const data = keyToData(e);
  if (!data) return;

  e.preventDefault();
  if (!sessionId) return;
  try { await invoke('pty_write', { sessionId, data }); } catch {}
});

// ── Welcome Banner ─────────────────────────────────────────────

function showWelcome(): void {
  const emerald = '\x1b[38;2;52;211;153m';
  const dim = '\x1b[2m';
  const reset = '\x1b[0m';
  const blue = '\x1b[38;2;96;165;250m';

  term.write(emerald + '  NexVigilant Terminal' + reset + dim + ' v0.3.0' + reset + '\r\n');
  term.write(dim + '  \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500' + reset + '\r\n');
  term.write(blue + '  Ctrl+Shift+N' + reset + dim + '  New Tab' + reset + '\r\n');
  term.write(blue + '  Ctrl+Shift+R' + reset + dim + '  Reconnect' + reset + '\r\n');
  term.write('\r\n');
}

// ── Fallback Input Bar ─────────────────────────────────────────

function wireInputBar(): void {
  const inputField = document.getElementById('input-field') as HTMLInputElement | null;
  if (!inputField) return;

  inputField.addEventListener('keydown', async (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const text = inputField.value;
      inputField.value = '';
      if (!sessionId) {
        term.write('\x1b[31m[No session]\x1b[0m\r\n');
        await createSession();
        return;
      }
      try {
        await invoke('pty_write', { sessionId, data: text + '\r' });
      } catch (err: unknown) {
        term.write('\x1b[31m[pty_write failed: ' + err + ']\x1b[0m\r\n');
      }
    } else if (e.key === 'c' && e.ctrlKey) {
      e.preventDefault();
      if (sessionId) {
        try { await invoke('pty_write', { sessionId, data: '\x03' }); } catch {}
      }
    }
  });
}

// ── Init ───────────────────────────────────────────────────────

async function init(): Promise<void> {
  showWelcome();

  if (tauriAvailable) {
    await createSession();
  } else {
    term.write('\x1b[33m  [Dev mode] Tauri not available.\x1b[0m\r\n');
    term.onData((data: string) => {
      if (data === '\r') term.write('\r\n');
      else if (data === '\x7f') term.write('\b \b');
      else term.write(data);
    });
  }

  const tabAddBtn = document.getElementById('tab-add');
  if (tabAddBtn) tabAddBtn.addEventListener('click', () => createSession());

  wireInputBar();
  updateStatusBar();
}

init();
