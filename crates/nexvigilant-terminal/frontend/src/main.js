// NexVigilant Terminal — xterm.js Frontend
// Full Unicode terminal emulation with epistemic color processing

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import { Unicode11Addon } from '@xterm/addon-unicode11';
import { mountChiIndicator } from './chi-indicator.ts';
import { mountRemotePanel } from './remote-panel.ts';

// ── Tauri IPC ──────────────────────────────────────────────────

const tauriAvailable = typeof window.__TAURI__ !== 'undefined';

const invoke = tauriAvailable
  ? window.__TAURI__.core.invoke
  : async function mockInvoke(cmd, args) {
      console.log('[mock]', cmd, args);
      return null;
    };

const listen = tauriAvailable
  ? window.__TAURI__.event.listen
  : async function mockListen(event, handler) {
      console.log('[mock listen]', event);
      return function unlisten() {};
    };

// ── Epistemic Color Theme ──────────────────────────────────────
// 8 cognitive functions from learning theory, mapped to terminal colors

const EPISTEMIC_PALETTE = {
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
  // Unicode handling
  unicodeVersion: '11',
});

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

const unicode11Addon = new Unicode11Addon();
term.loadAddon(unicode11Addon);
term.unicode.activeVersion = '11';

// ── Mount Terminal ─────────────────────────────────────────────

const container = document.getElementById('terminal-container');
term.open(container);

// Try WebGL for GPU rendering, fall back to canvas
try {
  const webglAddon = new WebglAddon();
  webglAddon.onContextLoss(() => {
    webglAddon.dispose();
    console.warn('WebGL context lost, falling back to canvas renderer');
  });
  term.loadAddon(webglAddon);
  console.log('WebGL renderer active');
} catch (e) {
  console.warn('WebGL not available, using canvas renderer:', e.message);
}

fitAddon.fit();

// ── Session State ──────────────────────────────────────────────

let sessionId = null;
let ptyConnected = false;

// ── PTY Data Flow ──────────────────────────────────────────────

// Listen for PTY output events from Tauri backend
listen('pty-output', (event) => {
  if (event.payload) {
    term.write(event.payload);
  }
});

// Listen for PTY exit events
listen('pty-exit', (event) => {
  ptyConnected = false;
  term.write('\r\n\x1b[33m[Process exited');
  if (event.payload && event.payload.code !== undefined) {
    term.write(' with code ' + event.payload.code);
  }
  term.write(']\x1b[0m\r\n');
  updateStatusBar();
});

// Send terminal input to PTY
term.onData(async (data) => {
  if (!sessionId || !ptyConnected) return;
  try {
    await invoke('pty_write', { sessionId, data });
  } catch (e) {
    console.error('PTY write failed:', e);
  }
});

// Handle terminal resize
term.onResize(async ({ cols, rows }) => {
  if (!sessionId || !ptyConnected) return;
  try {
    await invoke('pty_resize', { sessionId, cols, rows });
  } catch (e) {
    console.error('PTY resize failed:', e);
  }
});

// ── Window Resize ──────────────────────────────────────────────

window.addEventListener('resize', () => {
  fitAddon.fit();
});

// ResizeObserver for more reliable container tracking
const resizeObserver = new ResizeObserver(() => {
  fitAddon.fit();
});
resizeObserver.observe(container);

// ── Status Bar ─────────────────────────────────────────────────

function updateStatusBar() {
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
}

// ── Keyboard Shortcuts ─────────────────────────────────────────

document.addEventListener('keydown', async (e) => {
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
});

// ── Session Management ─────────────────────────────────────────

async function createSession() {
  try {
    const shell = '/bin/bash';
    const workingDir = '/home/matthew';
    const result = await invoke('pty_spawn', {
      shell,
      workingDir,
      cols: term.cols,
      rows: term.rows,
    });
    if (result && result.session_id) {
      sessionId = result.session_id;
      ptyConnected = true;
      term.clear();
      updateStatusBar();
    }
  } catch (e) {
    term.write('\x1b[31mFailed to create session: ' + e + '\x1b[0m\r\n');
  }
}

async function connectPty() {
  // Reconnect to existing session
  if (!sessionId) return;
  try {
    await invoke('pty_reconnect', { sessionId });
    ptyConnected = true;
    updateStatusBar();
  } catch (e) {
    term.write('\x1b[31mReconnect failed: ' + e + '\x1b[0m\r\n');
  }
}

// ── Welcome Banner ─────────────────────────────────────────────

function showWelcome() {
  const emerald = '\x1b[38;2;52;211;153m';
  const dim = '\x1b[2m';
  const reset = '\x1b[0m';
  const blue = '\x1b[38;2;96;165;250m';

  term.write(emerald + '  NexVigilant Terminal' + reset + dim + ' v0.1.0' + reset + '\r\n');
  term.write(dim + '  Full Unicode ' + reset + '\u2713' + dim + '  GPU Rendering ' + reset + '\u2713' + dim + '  Epistemic Colors ' + reset + '\u2713' + reset + '\r\n');
  term.write(dim + '  \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500' + reset + '\r\n');
  term.write(blue + '  Ctrl+Shift+N' + reset + dim + '  New Session    ' + reset);
  term.write(blue + 'Ctrl+Shift+R' + reset + dim + '  Reconnect' + reset + '\r\n');
  term.write('\r\n');
}

// ── Unicode Test ───────────────────────────────────────────────

function showUnicodeTest() {
  const dim = '\x1b[2m';
  const reset = '\x1b[0m';
  const green = '\x1b[38;2;68;204;68m';
  const pink = '\x1b[38;2;255;102;178m';

  term.write(dim + '  Unicode check: ' + reset);
  term.write(green + '\u2713 \u2717 \u25cf \u25cb \u2192 \u2500 \u2502 \u250c \u2510 \u2514 \u2518' + reset);
  term.write(' ' + pink + '\u2203 \u2205 \u2202 \u03c2 \u2192 \u03ba \u03bc \u03c3 \u03bb \u03bd \u03c1 \u03c0 \u221d \u03a3 \u00d7' + reset);
  term.write('\r\n');
  // Box drawing test
  term.write(dim + '  Box drawing:   ' + reset);
  term.write('\u250c\u2500\u2500\u2500\u2500\u2510\r\n');
  term.write(dim + '                 ' + reset);
  term.write('\u2502 OK \u2502\r\n');
  term.write(dim + '                 ' + reset);
  term.write('\u2514\u2500\u2500\u2500\u2500\u2518\r\n');
  term.write('\r\n');
}

// ── Init ───────────────────────────────────────────────────────

async function init() {
  showWelcome();
  showUnicodeTest();

  if (tauriAvailable) {
    await createSession();
  } else {
    term.write('\x1b[33m  [Dev mode] Tauri not available. Terminal UI preview only.\x1b[0m\r\n');
    term.write('\x1b[2m  Build with: cargo tauri dev\x1b[0m\r\n\r\n');
    // In dev mode, allow typing to echo (for testing)
    term.onData((data) => {
      if (!tauriAvailable) {
        // Local echo for testing
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

  updateStatusBar();

  // Mount χ health indicator into the title-bar right section
  const titleBarRight = document.querySelector('.title-bar .right');
  if (titleBarRight) {
    mountChiIndicator(titleBarRight).catch((e) => {
      console.warn('[chi-indicator] mount failed:', e);
    });
  }

  // Mount remote controller panel (Claude accessibility layer)
  const remoteContainer = document.getElementById('remote-panel-container');
  if (remoteContainer) {
    mountRemotePanel(remoteContainer).catch((e) => {
      console.warn('[remote-panel] mount failed:', e);
    });
  }

  term.focus();
}

init();
