// NexVigilant Terminal — Claude Code Client
// First-class Claude Code integration via Tauri IPC.
// Manages lifecycle, routes output to xterm.js, shows status.

import * as toasts from './toast';

const tauriAvailable: boolean = typeof (window as any).__TAURI__ !== 'undefined';
const invoke = tauriAvailable
  ? (window as any).__TAURI__!.core.invoke
  : async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
      console.log('[claude-mock]', cmd, args);
      return null;
    };

const listen = tauriAvailable
  ? (window as any).__TAURI__!.event.listen
  : async function mockListen<T>(_event: string, _handler: (e: { payload: T }) => void): Promise<() => void> {
      return function unlisten(): void {};
    };

/** Claude Code process states. */
export type ClaudeState = 'stopped' | 'starting' | 'running' | 'exited';

/** Status from claude_status Tauri command. */
export interface ClaudeStatus {
  state: ClaudeState;
  session_id: string | null;
  station_connected: boolean;
  working_dir: string | null;
  args: string[];
  exit_code: number | null;
}

/** Exit event payload. */
interface ClaudeExitPayload {
  session_id: string;
  code: number | null;
}

// ── State ──────────────────────────────────────────────────────

let currentState: ClaudeState = 'stopped';
let currentSessionId: string | null = null;
let onStateChange: ((state: ClaudeState) => void) | null = null;
let onExit: ((code: number | null) => void) | null = null;

// ── API ────────────────────────────────────────────────────────

/** Start Claude Code with Station MCP pre-configured. */
export async function start(
  workingDir?: string,
  args?: string[],
): Promise<ClaudeStatus | null> {
  try {
    const result = await invoke('claude_start', {
      workingDir: workingDir || null,
      args: args || null,
    }) as ClaudeStatus;

    if (result) {
      currentState = result.state;
      currentSessionId = result.session_id;
      toasts.success('Claude Code', 'Starting...');
      onStateChange?.(currentState);
    }
    return result;
  } catch (e) {
    toasts.warn('Claude Code Failed', String(e));
    return null;
  }
}

/** Stop the running Claude Code process. */
export async function stop(): Promise<void> {
  try {
    await invoke('claude_stop');
    currentState = 'stopped';
    currentSessionId = null;
    toasts.success('Claude Code', 'Stopped');
    onStateChange?.(currentState);
  } catch (e) {
    toasts.warn('Claude Stop Failed', String(e));
  }
}

/** Get current Claude Code status. */
export async function status(): Promise<ClaudeStatus | null> {
  try {
    const result = await invoke('claude_status') as ClaudeStatus;
    if (result) {
      currentState = result.state;
      currentSessionId = result.session_id;
    }
    return result;
  } catch (e) {
    return null;
  }
}

/** Restart Claude Code. */
export async function restart(): Promise<ClaudeStatus | null> {
  try {
    const result = await invoke('claude_restart') as ClaudeStatus;
    if (result) {
      currentState = result.state;
      currentSessionId = result.session_id;
      toasts.success('Claude Code', 'Restarting...');
      onStateChange?.(currentState);
    }
    return result;
  } catch (e) {
    toasts.warn('Claude Restart Failed', String(e));
    return null;
  }
}

/** Register state change callback. */
export function onState(cb: (state: ClaudeState) => void): void {
  onStateChange = cb;
}

/** Register exit callback. */
export function onExitEvent(cb: (code: number | null) => void): void {
  onExit = cb;
}

/** Get current session ID (for PTY event routing). */
export function getSessionId(): string | null {
  return currentSessionId;
}

/** Get current state synchronously. */
export function getState(): ClaudeState {
  return currentState;
}

// ── Exit Listener ──────────────────────────────────────────────

(listen as any)('claude-exit', (event: { payload: ClaudeExitPayload }) => {
  currentState = 'exited';
  const code = event.payload?.code ?? null;
  onStateChange?.('exited');
  onExit?.(code);
});

// ── Status Panel ───────────────────────────────────────────────

let panelEl: HTMLElement | null = null;

/** Mount the Claude Code status indicator into a container. */
export function mountStatusPanel(container: HTMLElement): void {
  panelEl = document.createElement('div');
  panelEl.className = 'claude-status';
  Object.assign(panelEl.style, {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '4px 8px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
  });

  updatePanel();
  container.appendChild(panelEl);

  // Auto-refresh
  onState(() => updatePanel());
}

function updatePanel(): void {
  if (!panelEl) return;

  const stateColors: Record<ClaudeState, string> = {
    stopped: '#556677',
    starting: '#fbbf24',
    running: '#4ade80',
    exited: '#f87171',
  };

  const stateIcons: Record<ClaudeState, string> = {
    stopped: '\u25CB',  // ○
    starting: '\u25D4', // ◔
    running: '\u25CF',  // ●
    exited: '\u25C6',   // ◆
  };

  panelEl.innerHTML = '';

  const dot = document.createElement('span');
  dot.textContent = stateIcons[currentState];
  dot.style.color = stateColors[currentState];
  panelEl.appendChild(dot);

  const label = document.createElement('span');
  label.textContent = 'Claude Code';
  label.style.color = '#e0e8f0';
  panelEl.appendChild(label);

  const stateLabel = document.createElement('span');
  stateLabel.textContent = currentState;
  stateLabel.style.color = stateColors[currentState];
  panelEl.appendChild(stateLabel);

  if (currentState === 'running') {
    const stationBadge = document.createElement('span');
    stationBadge.textContent = 'Station \u2713';
    Object.assign(stationBadge.style, {
      color: '#34d399',
      fontSize: '9px',
      border: '1px solid #1a3a2a',
      borderRadius: '3px',
      padding: '1px 4px',
    });
    panelEl.appendChild(stationBadge);
  }

  // Action button
  const btn = document.createElement('button');
  Object.assign(btn.style, {
    background: 'none',
    border: '1px solid #2a3a4e',
    borderRadius: '3px',
    color: '#93c5fd',
    fontSize: '9px',
    padding: '1px 6px',
    cursor: 'pointer',
    marginLeft: '4px',
  });

  if (currentState === 'running' || currentState === 'starting') {
    btn.textContent = 'Stop';
    btn.addEventListener('click', () => stop());
  } else {
    btn.textContent = 'Start';
    btn.addEventListener('click', () => start());
  }
  panelEl.appendChild(btn);
}
