// NexVigilant Terminal — Quick Command Bar
// Vim-style : prefix triggers a command palette for power users.
// Commands route to PTY injection, Tauri IPC, or internal actions.

import * as toasts from './toast';
import * as claude from './claude-client';
import * as repl from './repl-client';

interface QuickCommand {
  name: string;
  desc: string;
  /** If set, inject this into PTY. Supports {arg} placeholder. */
  pty?: string;
  /** If set, run this internal function. */
  fn?: (arg: string) => Promise<void> | void;
}

const tauriAvailable: boolean = typeof (window as any).__TAURI__ !== 'undefined';
const invoke = tauriAvailable
  ? (window as any).__TAURI__!.core.invoke
  : async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
      console.log('[mock]', cmd, args);
      return null;
    };

async function injectPty(cmd: string): Promise<void> {
  const sid = (window as any).__nvt_sessionId;
  if (!sid) {
    toasts.warn('No active session', 'Open a terminal tab first');
    return;
  }
  try {
    await invoke('pty_write', { sessionId: sid, data: cmd + '\r' });
    toasts.info('Sent', cmd);
  } catch (e: unknown) {
    toasts.warn('PTY write failed', String(e));
  }
}

const COMMANDS: QuickCommand[] = [
  // PV Workflows
  { name: 'signal', desc: 'Run signal detection pipeline', pty: '/station-signal {arg}' },
  { name: 'demo', desc: 'Station tool demo', pty: '/station-demo' },
  { name: 'faers', desc: 'Query FAERS adverse events', pty: '/station-signal {arg}' },
  { name: 'causality', desc: 'Assess causality (Naranjo)', pty: '/station-signal {arg}' },

  // System
  { name: 'health', desc: 'System health check', pty: '/system-status' },
  { name: 'pulse', desc: 'PULSE health monitor', pty: '/pulse-program' },
  { name: 'status', desc: 'Full system status', pty: '/system-status' },

  // Development
  { name: 'commit', desc: 'Git commit workflow', pty: '/commit' },
  { name: 'craft', desc: 'CRAFT code quality', pty: '/craft-program' },
  { name: 'progress', desc: 'PROGRESS measurement', pty: '/progress-program' },
  { name: 'forge', desc: 'Start FORGE autonomous dev', pty: 'START FORGE {arg}' },
  { name: 'exhale', desc: 'Session exhale', pty: '/exhale' },

  // Micrograms
  { name: 'mg', desc: 'Microgram dashboard', pty: '/mg dashboard' },
  { name: 'mg-test', desc: 'Test all micrograms', pty: '/mg test-all' },

  // Claude Code
  { name: 'claude', desc: 'Start Claude Code with Station MCP', fn: async () => {
    const state = claude.getState();
    if (state === 'running' || state === 'starting') {
      toasts.info('Claude Code', 'Already running. Use :claude-stop to stop.');
      return;
    }
    await claude.start();
  }},
  { name: 'claude-stop', desc: 'Stop Claude Code', fn: async () => {
    await claude.stop();
  }},
  { name: 'claude-restart', desc: 'Restart Claude Code', fn: async () => {
    await claude.restart();
  }},
  { name: 'claude-status', desc: 'Check Claude Code status', fn: async () => {
    const s = await claude.status();
    if (s) {
      toasts.info('Claude Code', `${s.state} | Station: ${s.station_connected ? 'connected' : 'disconnected'}`);
    }
  }},

  // Claude REPL (headless inline)
  { name: 'ask', desc: 'Ask Claude inline (headless)', fn: async (arg: string) => {
    if (!arg.trim()) {
      toasts.warn('Usage', ':ask <your question>');
      return;
    }
    toasts.info('REPL', 'Asking Claude...');
    const resp = await repl.ask(arg);
    if (resp) {
      if (resp.error) {
        toasts.warn('Claude Error', resp.text.slice(0, 100));
      } else {
        toasts.success('Claude', resp.mode === 'tui' ? 'Forwarded to TUI' : 'Response received');
      }
    }
  }},
  { name: 'repl-clear', desc: 'Clear REPL session (fresh context)', fn: async () => {
    await repl.clear();
  }},
  { name: 'repl-status', desc: 'Check REPL status', fn: async () => {
    const s = await repl.status();
    if (s) {
      toasts.info('REPL', `busy: ${s.busy} | session: ${s.session_id || 'none'}`);
    }
  }},

  // Terminal control
  { name: 'theme', desc: 'Toggle epistemic colors', fn: () => {
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'E', ctrlKey: true, shiftKey: true }));
  }},
  { name: 'split', desc: 'New session tab', fn: () => {
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'N', ctrlKey: true, shiftKey: true }));
  }},
  { name: 'dashboard', desc: 'Toggle side panel', fn: () => {
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'D', ctrlKey: true, shiftKey: true }));
  }},

  // Nucleus links
  { name: 'nucleus', desc: 'Open Nucleus portal', fn: () => {
    window.open('http://localhost:9002', '_blank');
    toasts.info('Opening Nucleus', 'localhost:9002');
  }},
  { name: 'observatory', desc: 'Open Observatory', fn: () => {
    window.open('http://localhost:9002/observatory', '_blank');
    toasts.info('Opening Observatory', 'localhost:9002/observatory');
  }},
  { name: 'academy', desc: 'Open PV Academy', fn: () => {
    window.open('http://localhost:9002/academy', '_blank');
    toasts.info('Opening Academy', 'localhost:9002/academy');
  }},
  { name: 'station', desc: 'Open NexVigilant Station', fn: () => {
    window.open('https://mcp.nexvigilant.com/tools', '_blank');
    toasts.info('Opening Station', 'mcp.nexvigilant.com');
  }},
];

let barVisible = false;
let barEl: HTMLElement | null = null;
let inputEl: HTMLInputElement | null = null;
let hintsEl: HTMLElement | null = null;

function getElements(): void {
  if (!barEl) barEl = document.getElementById('quick-cmd-bar');
  if (!inputEl) inputEl = document.getElementById('quick-cmd-input') as HTMLInputElement;
  if (!hintsEl) hintsEl = document.getElementById('quick-cmd-hints');
}

export function open(): void {
  getElements();
  if (!barEl || !inputEl) return;
  barVisible = true;
  barEl.classList.add('visible');
  inputEl.value = '';
  inputEl.focus();
  renderHints('');
}

export function close(): void {
  getElements();
  if (!barEl) return;
  barVisible = false;
  barEl.classList.remove('visible');
  if (hintsEl) hintsEl.style.display = 'none';
}

export function isOpen(): boolean {
  return barVisible;
}

function renderHints(query: string): void {
  if (!hintsEl) return;
  while (hintsEl.firstChild) hintsEl.removeChild(hintsEl.firstChild);

  const q = query.toLowerCase().split(/\s+/)[0] || '';
  const matches = q ? COMMANDS.filter((c) => c.name.startsWith(q)) : COMMANDS.slice(0, 10);

  if (matches.length === 0) {
    hintsEl.style.display = 'none';
    return;
  }

  hintsEl.style.display = 'block';
  for (const cmd of matches) {
    const row = document.createElement('div');
    row.className = 'quick-cmd-hint';

    const nameSpan = document.createElement('span');
    nameSpan.className = 'cmd';
    nameSpan.textContent = ':' + cmd.name;

    const descSpan = document.createElement('span');
    descSpan.className = 'desc';
    descSpan.textContent = cmd.desc;

    row.appendChild(nameSpan);
    row.appendChild(descSpan);

    row.addEventListener('click', () => {
      if (inputEl) inputEl.value = cmd.name + ' ';
      inputEl?.focus();
    });

    hintsEl.appendChild(row);
  }
}

async function execute(input: string): Promise<void> {
  const parts = input.trim().split(/\s+/);
  const cmdName = parts[0] || '';
  const arg = parts.slice(1).join(' ');

  const cmd = COMMANDS.find((c) => c.name === cmdName);
  if (!cmd) {
    // Not a known command — inject raw text into PTY
    await injectPty(input.trim());
    close();
    return;
  }

  if (cmd.fn) {
    await cmd.fn(arg);
  } else if (cmd.pty) {
    const expanded = cmd.pty.replace('{arg}', arg).trim();
    await injectPty(expanded);
  }

  close();
}

/** Mount keyboard listener for the quick command bar. */
export function mount(): void {
  getElements();
  if (!inputEl) return;

  inputEl.addEventListener('input', () => {
    renderHints(inputEl?.value || '');
  });

  inputEl.addEventListener('keydown', async (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      close();
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const val = inputEl?.value || '';
      if (val.trim()) {
        await execute(val);
      } else {
        close();
      }
    }
  });

  // Global : trigger — only when not in an input/textarea
  document.addEventListener('keydown', (e: KeyboardEvent) => {
    if (barVisible && e.key === 'Escape') {
      close();
      return;
    }
    // : key opens the bar (Shift+; on US keyboards)
    if (e.key === ':' && !barVisible) {
      const target = e.target as HTMLElement;
      const tag = target.tagName.toLowerCase();
      // Don't trigger inside inputs or the terminal textarea
      if (tag === 'input' || tag === 'textarea' || target.contentEditable === 'true') return;
      e.preventDefault();
      open();
    }
  });
}
