// NexVigilant Terminal — Simplified xterm.js Frontend
// PTY terminal with tabs, Wayland keyboard bypass, fallback input bar.

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import { Unicode11Addon } from '@xterm/addon-unicode11';
import * as repl from './repl-client';

// ── Diagnostic Log ────────────────────────────────────────────
// Writes console.log to /tmp/nvterm-diag.log via Tauri FS.
// Read from Claude Code with: cat /tmp/nvterm-diag.log
const _origLog = console.log;
const _diagLines: string[] = [];
console.log = (...args: unknown[]) => {
  _origLog(...args);
  const line = `${new Date().toISOString()} ${args.map(String).join(' ')}`;
  _diagLines.push(line);
  // Flush to file every 5 lines
  if (_diagLines.length >= 5) {
    try {
      const text = _diagLines.join('\n') + '\n';
      _diagLines.length = 0;
      // Use synchronous XHR to localhost as a file-write workaround
      // Actually — use Tauri invoke to write
      if (typeof (window as any).__TAURI__ !== 'undefined') {
        // Tauri doesn't have a direct file write command by default
        // Use pty_write to echo to a file as a hack
        // Simpler: just accumulate in a global and expose via window
      }
      (window as any).__nvterm_diag = ((window as any).__nvterm_diag || '') + text;
    } catch {}
  }
};

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

// Diagnostic: log PTY output to a file via Tauri FS (or console)
let ptyOutputCount = 0;

(listen as any)('pty-output', (event: { payload: string }) => {
  if (!event.payload) return;
  ptyOutputCount++;
  // Log first 20 events to console for diagnosis
  if (ptyOutputCount <= 20) {
    console.log(`[pty-output #${ptyOutputCount}] len=${event.payload.length} sid=${sessionId} tabs=${tabs.length} preview=${JSON.stringify(event.payload.slice(0, 60))}`);
  }
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
  // Must be visible BEFORE open() — xterm.js needs to measure the container
  termDiv.style.display = 'block';
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

// ── Wayland Keyboard Bypass (v2) ──────────────────────────────
// WebKitGTK on Wayland can't route keyboard events to xterm.js textarea.
// v2 fix: always activate on Tauri (don't wait for detection), handle
// both global and per-tab terminals, and suppress only when xterm.js
// actually processes the keystroke (proven by onData firing).

let waylandBypassActive = false;

// On Tauri, activate bypass immediately — the 2-second detection was unreliable
// because it only checked the global terminal, not per-tab instances.
if (tauriAvailable) {
  // Give xterm.js one chance to prove it works (500ms is enough)
  let xtermProven = false;
  term.onData(() => { xtermProven = true; });
  setTimeout(() => {
    if (!xtermProven) {
      waylandBypassActive = true;
      console.log('[wayland-bypass-v2] Activated — routing keystrokes directly to PTY');
    } else {
      console.log('[wayland-bypass-v2] xterm.js keyboard working — bypass not needed');
    }
  }, 500);
}

function keyToData(e: KeyboardEvent): string | null {
  // Let Ctrl+Shift combos through for app shortcuts (new tab, reconnect, etc.)
  if (e.ctrlKey && e.shiftKey) return null;
  if (e.altKey && e.key.length > 1) return null;
  if (e.metaKey) return null;

  // Ctrl+key → control character
  if (e.ctrlKey) {
    const code = e.key.toLowerCase().charCodeAt(0);
    if (code >= 97 && code <= 122) return String.fromCharCode(code - 96);
    return null;
  }

  // Alt+key → ESC prefix (for readline alt-b, alt-f, etc.)
  if (e.altKey && e.key.length === 1) {
    return '\x1b' + e.key;
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
    case 'Insert': return '\x1b[2~';
    case 'F1': return '\x1bOP';
    case 'F2': return '\x1bOQ';
    case 'F3': return '\x1bOR';
    case 'F4': return '\x1bOS';
    case 'F5': return '\x1b[15~';
    case 'F6': return '\x1b[17~';
    case 'F7': return '\x1b[18~';
    case 'F8': return '\x1b[19~';
    case 'F9': return '\x1b[20~';
    case 'F10': return '\x1b[21~';
    case 'F11': return '\x1b[23~';
    case 'F12': return '\x1b[24~';
    default: break;
  }

  if (e.key.length > 1) return null;
  return e.key;
}

document.addEventListener('keydown', async (e: KeyboardEvent) => {
  if (!waylandBypassActive) {
    console.log(`[wayland-bypass] INACTIVE — key '${e.key}' not forwarded`);
    return;
  }

  // Don't intercept when user is in the fallback input bar or quick-cmd
  const active = document.activeElement;
  if (active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA')) return;

  const data = keyToData(e);
  if (!data) {
    console.log(`[wayland-bypass] key '${e.key}' not mapped`);
    return;
  }

  console.log(`[wayland-bypass] sending key '${e.key}' → pty_write (sid=${sessionId})`);
  e.preventDefault();
  if (!sessionId) return;
  try { await invoke('pty_write', { sessionId, data }); } catch {}
});

// ── REPL Integration ──────────────────────────────────────────
// Intercept @claude prompts from the fallback input bar and route
// them through the REPL backend instead of writing raw to PTY.

repl.onReply((resp) => {
  // Write the formatted response to the active terminal
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const target = activeTab ? activeTab.term : term;
  target.write(repl.formatResponse(resp));
});

/**
 * Handle a line of input — check for @claude prefix and route accordingly.
 * Returns true if the input was handled by the REPL (caller should not send to PTY).
 */
async function handleReplInput(input: string, targetTerm: Terminal): Promise<boolean> {
  const prompt = repl.extractPrompt(input);
  if (!prompt) return false;

  // Show thinking indicator
  const dim = '\x1b[2m';
  const reset = '\x1b[0m';
  const cyan = '\x1b[38;2;34;211;238m';
  targetTerm.write('\r\n' + dim + '\u23F3 ' + reset + cyan + 'Asking Claude...' + reset + '\r\n');

  const resp = await repl.ask(prompt);
  if (resp) {
    targetTerm.write(repl.formatResponse(resp));
  }

  return true;
}

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
  term.write(blue + '  @claude <q>' + reset + dim + '   Ask Claude inline' + reset + '\r\n');
  term.write(blue + '  :claude' + reset + dim + '       Full Claude TUI' + reset + '\r\n');
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
      // Check for @claude prefix — route to REPL instead of PTY
      const activeTab = tabs.find((t) => t.id === activeTabId);
      const target = activeTab ? activeTab.term : term;
      const handled = await handleReplInput(text, target);
      if (handled) return;
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
