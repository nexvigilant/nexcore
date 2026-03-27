// NexVigilant Terminal — DualSense PS5 Gamepad Controller
// Maps PS5 controller to terminal actions via Tauri IPC + PTY
//
// Three modes (PS button cycles):
//   TERMINAL — navigate, type, run commands, control PTY
//   CLAUDE   — submit prompts, approve/deny, slash commands, scroll
//   SYSTEM   — Station health, cloud overview, command palette, tabs

// ── Tauri IPC Bridge ──────────────────────────────────────────────

// Re-use Tauri types from chi-indicator.ts (already declared globally)
// Window.__TAURI__ type is declared there.

const tauriAvailable: boolean = typeof window.__TAURI__ !== 'undefined';
const invoke = tauriAvailable
  ? window.__TAURI__!.core.invoke
  : async function gpMockInvoke(cmd: string, args?: unknown): Promise<unknown> {
      console.log('[gamepad mock]', cmd, args);
      return null;
    };

// ── DualSense Button Map (standard Gamepad API layout) ────────────

const BTN = {
  CROSS: 0,     // X — primary action
  CIRCLE: 1,    // O — cancel / back
  SQUARE: 2,    // □ — secondary action
  TRIANGLE: 3,  // △ — tertiary action
  L1: 4,        // Left bumper
  R1: 5,        // Right bumper
  L2: 6,        // Left trigger
  R2: 7,        // Right trigger
  SHARE: 8,     // Create button
  OPTIONS: 9,   // Options
  L3: 10,       // Left stick press
  R3: 11,       // Right stick press
  UP: 12,       // D-pad up
  DOWN: 13,     // D-pad down
  LEFT: 14,     // D-pad left
  RIGHT: 15,    // D-pad right
  PS: 16,       // PS button — mode cycle
  TOUCH: 17,    // Touchpad press
} as const;

// ── Gamepad Modes ─────────────────────────────────────────────────

type GamepadMode = 'TERMINAL' | 'CLAUDE' | 'SYSTEM';
const MODES: GamepadMode[] = ['TERMINAL', 'CLAUDE', 'SYSTEM'];
const MODE_COLORS: Record<GamepadMode, string> = {
  TERMINAL: '#00ff87',
  CLAUDE: '#ff6ec7',
  SYSTEM: '#4f7df5',
};

// ── State ─────────────────────────────────────────────────────────

let currentMode: GamepadMode = 'TERMINAL';
let prevButtons: boolean[] = new Array(20).fill(false);
let connected = false;
// Session/index tracked for future PTY routing
let gpSessionId: string | null = null;
let gpIndex: number | null = null;

// Callback hooks — main.ts sets these
let onWritePty: ((data: string) => void) | null = null;
let onCycleTab: ((dir: number) => void) | null = null;
let onNewTab: (() => void) | null = null;
let onCloseTab: (() => void) | null = null;
let onToggleCommandPalette: (() => void) | null = null;
let onToast: ((msg: string, type?: string) => void) | null = null;
let onModeChange: ((mode: GamepadMode) => void) | null = null;

// ── Rumble (DualSense haptic feedback) ────────────────────────────

function rumble(weak: number, strong: number, duration: number): void {
  try {
    const gps = navigator.getGamepads();
    for (const gp of gps) {
      if (gp?.vibrationActuator) {
        (gp.vibrationActuator as any).playEffect('dual-rumble', {
          startDelay: 0, duration, weakMagnitude: weak, strongMagnitude: strong,
        });
      }
    }
  } catch { /* not all browsers support haptics */ }
}

// ── PTY Write Helper ──────────────────────────────────────────────

function writeToPty(data: string): void {
  if (onWritePty) {
    onWritePty(data);
  }
}

// ── TERMINAL Mode Actions ─────────────────────────────────────────
// The PS5 controller becomes your terminal keyboard

function handleTerminal(btn: number): void {
  switch (btn) {
    case BTN.CROSS:     writeToPty('\r'); rumble(0.1, 0, 50); break;           // Enter
    case BTN.CIRCLE:    writeToPty('\x03'); rumble(0.2, 0.1, 80); break;       // Ctrl+C
    case BTN.SQUARE:    writeToPty('\t'); rumble(0.05, 0, 30); break;          // Tab (autocomplete)
    case BTN.TRIANGLE:  writeToPty('\x0c'); rumble(0.1, 0, 40); break;        // Ctrl+L (clear)
    case BTN.L1:        writeToPty('\x1b[A'); rumble(0.03, 0, 20); break;     // Up arrow (history)
    case BTN.R1:        writeToPty('\x1b[B'); rumble(0.03, 0, 20); break;     // Down arrow (history)
    case BTN.UP:        writeToPty('\x1b[A'); rumble(0.03, 0, 20); break;     // Up arrow
    case BTN.DOWN:      writeToPty('\x1b[B'); rumble(0.03, 0, 20); break;     // Down arrow
    case BTN.LEFT:      writeToPty('\x1b[D'); rumble(0.03, 0, 20); break;     // Left arrow
    case BTN.RIGHT:     writeToPty('\x1b[C'); rumble(0.03, 0, 20); break;     // Right arrow
    case BTN.L2:        writeToPty('\x1bb'); rumble(0.05, 0, 30); break;      // Alt+B (word back)
    case BTN.R2:        writeToPty('\x1bf'); rumble(0.05, 0, 30); break;      // Alt+F (word forward)
    case BTN.SHARE:     writeToPty('\x1a'); rumble(0.1, 0, 50); break;        // Ctrl+Z (suspend)
    case BTN.OPTIONS:   writeToPty('\x04'); rumble(0.1, 0, 50); break;        // Ctrl+D (EOF)
    case BTN.L3:        writeToPty('\x01'); rumble(0.05, 0, 30); break;       // Ctrl+A (line start)
    case BTN.R3:        writeToPty('\x05'); rumble(0.05, 0, 30); break;       // Ctrl+E (line end)
    case BTN.TOUCH:     writeToPty('\x12'); rumble(0.1, 0, 40); break;        // Ctrl+R (reverse search)
  }
}

// ── CLAUDE Mode Actions ───────────────────────────────────────────
// Optimized for Claude Code interaction

function handleClaude(btn: number): void {
  switch (btn) {
    case BTN.CROSS:     writeToPty('\r'); rumble(0.15, 0.1, 80); break;       // Enter (submit prompt)
    case BTN.CIRCLE:    writeToPty('\x1b'); rumble(0.1, 0, 50); break;        // Escape
    case BTN.SQUARE:    writeToPty('\t'); rumble(0.05, 0, 30); break;         // Tab (accept)
    case BTN.TRIANGLE:  writeToPty('/'); rumble(0.1, 0.1, 60); break;         // / (slash command start)
    case BTN.L1:        writeToPty('\x1b[A'); rumble(0.03, 0, 20); break;     // Scroll up
    case BTN.R1:        writeToPty('\x1b[B'); rumble(0.03, 0, 20); break;     // Scroll down
    case BTN.UP:        writeToPty('\x1b[A'); break;                           // Up
    case BTN.DOWN:      writeToPty('\x1b[B'); break;                           // Down
    case BTN.LEFT:      writeToPty('\x1b[D'); break;                           // Left
    case BTN.RIGHT:     writeToPty('\x1b[C'); break;                           // Right
    case BTN.L2:        writeToPty('y'); rumble(0.2, 0.2, 100); break;        // Approve (y)
    case BTN.R2:        writeToPty('n'); rumble(0.3, 0.1, 100); break;        // Deny (n)
    case BTN.SHARE:     writeToPty('\x03'); rumble(0.3, 0.2, 150); break;     // Ctrl+C (interrupt)
    case BTN.OPTIONS:   writeToPty('/help\r'); rumble(0.1, 0, 50); break;     // /help + Enter
    case BTN.L3: {                                                              // /commit
      writeToPty('/commit\r');
      rumble(0.2, 0.3, 150);
      onToast?.('Commit triggered', 'info');
      break;
    }
    case BTN.R3: {                                                              // /exhale
      writeToPty('/exhale\r');
      rumble(0.2, 0.3, 150);
      onToast?.('Exhale triggered', 'info');
      break;
    }
    case BTN.TOUCH: {                                                           // Quick command palette
      onToggleCommandPalette?.();
      rumble(0.1, 0, 50);
      break;
    }
  }
}

// ── SYSTEM Mode Actions ───────────────────────────────────────────
// Terminal management: tabs, station, system health

function handleSystem(btn: number): void {
  switch (btn) {
    case BTN.CROSS: {                                                           // Station health check
      invoke('station_health').then((r) => {
        onToast?.('Station: ' + JSON.stringify(r).slice(0, 60), 'info');
      });
      rumble(0.1, 0.1, 80);
      break;
    }
    case BTN.CIRCLE:    onCloseTab?.(); rumble(0.2, 0.1, 80); break;          // Close tab
    case BTN.SQUARE:    onNewTab?.(); rumble(0.15, 0, 60); break;             // New tab
    case BTN.TRIANGLE: {                                                        // System snapshot
      invoke('remote_snapshot').then((r) => {
        const s = r as { chi: number; health_band: string; active_sessions: number };
        onToast?.(`χ=${s?.chi?.toFixed(2)} ${s?.health_band} sessions=${s?.active_sessions}`, 'info');
      });
      rumble(0.1, 0.2, 100);
      break;
    }
    case BTN.L1:        onCycleTab?.(-1); rumble(0.05, 0, 30); break;         // Prev tab
    case BTN.R1:        onCycleTab?.(1); rumble(0.05, 0, 30); break;          // Next tab
    case BTN.UP:        writeToPty('\x1b[A'); break;
    case BTN.DOWN:      writeToPty('\x1b[B'); break;
    case BTN.LEFT:      writeToPty('\x1b[D'); break;
    case BTN.RIGHT:     writeToPty('\x1b[C'); break;
    case BTN.L2: {                                                              // Cloud overview
      invoke('cloud_overview').then((r) => {
        onToast?.('Cloud: ' + JSON.stringify(r).slice(0, 80), 'info');
      });
      break;
    }
    case BTN.R2: {                                                              // Shell status
      invoke('shell_status').then((r) => {
        onToast?.('Shell: ' + JSON.stringify(r).slice(0, 80), 'info');
      });
      break;
    }
    case BTN.OPTIONS:   onToggleCommandPalette?.(); rumble(0.1, 0, 50); break; // Command palette
    case BTN.TOUCH:     onToggleCommandPalette?.(); rumble(0.1, 0.1, 60); break; // Command palette
  }
}

const HANDLERS: Record<GamepadMode, (btn: number) => void> = {
  TERMINAL: handleTerminal,
  CLAUDE: handleClaude,
  SYSTEM: handleSystem,
};

// ── Gamepad Loop ──────────────────────────────────────────────────

function gamepadLoop(): void {
  const gps = navigator.getGamepads();
  let gp: Gamepad | null = null;

  for (const g of gps) {
    if (g) { gp = g; break; }
  }

  if (!gp) {
    if (connected) {
      connected = false;
      onToast?.('Controller disconnected', 'warn');
    }
    requestAnimationFrame(gamepadLoop);
    return;
  }

  if (!connected) {
    connected = true;
    gpIndex = gp.index;
    onToast?.(`DualSense connected: ${gp.id.slice(0, 40)}`, 'info');
    rumble(0.2, 0.4, 200);
  }

  // Edge detection: pressed this frame but not last
  for (let i = 0; i < gp.buttons.length && i < 20; i++) {
    const btn = gp.buttons[i];
    if (!btn) continue;
    const pressed = btn.pressed;
    if (pressed && !prevButtons[i]) {
      // PS button cycles mode
      if (i === BTN.PS) {
        const idx = MODES.indexOf(currentMode);
        currentMode = MODES[(idx + 1) % MODES.length] ?? 'TERMINAL';
        onModeChange?.(currentMode);
        onToast?.(`Mode: ${currentMode}`, 'info');
        rumble(0.15, 0.15, 100);
      } else {
        HANDLERS[currentMode](i);
      }
    }
    prevButtons[i] = pressed;
  }

  requestAnimationFrame(gamepadLoop);
}

// ── HUD Element ───────────────────────────────────────────────────
// Small overlay showing controller state in the terminal status bar

function createGamepadPill(): HTMLSpanElement {
  const pill = document.createElement('span');
  pill.id = 'gamepad-pill';
  pill.style.cssText = `
    display: inline-flex; align-items: center; gap: 4px;
    padding: 2px 8px; border-radius: 10px; font-size: 10px;
    font-family: monospace; font-weight: 700; letter-spacing: 1px;
    border: 1px solid ${MODE_COLORS.TERMINAL};
    color: ${MODE_COLORS.TERMINAL};
    background: rgba(0, 255, 135, 0.1);
    transition: all 0.2s;
  `;
  pill.innerHTML = `<span style="width:6px;height:6px;border-radius:50%;background:#555" id="gp-dot"></span> 🎮 TERMINAL`;
  return pill;
}

function updateGamepadPill(): void {
  const pill = document.getElementById('gamepad-pill');
  if (!pill) return;

  const color = MODE_COLORS[currentMode];
  const dotColor = connected ? '#00ff87' : '#555';
  pill.style.borderColor = color;
  pill.style.color = color;
  pill.style.background = color.replace(')', ', 0.1)').replace('rgb', 'rgba');
  pill.innerHTML = `<span style="width:6px;height:6px;border-radius:50%;background:${dotColor}" id="gp-dot"></span> 🎮 ${currentMode}`;
}

// ── Public API ────────────────────────────────────────────────────

export interface GamepadConfig {
  /** Callback when data should be written to PTY */
  writePty: (data: string) => void;
  /** Callback to cycle tabs (+1 = next, -1 = prev) */
  cycleTab?: (dir: number) => void;
  /** Callback to create new tab */
  newTab?: () => void;
  /** Callback to close current tab */
  closeTab?: () => void;
  /** Callback to toggle command palette */
  toggleCommandPalette?: () => void;
  /** Callback for toast notifications */
  toast?: (msg: string, type?: string) => void;
  /** Callback when gamepad mode changes */
  onModeChange?: (mode: GamepadMode) => void;
  /** Initial PTY session ID */
  gpSessionId?: string;
}

/**
 * Initialize the gamepad controller module.
 * Call once from main.ts after terminal is ready.
 * Returns the HUD pill element to mount in the status bar.
 */
export function initGamepad(config: GamepadConfig): HTMLSpanElement {
  onWritePty = config.writePty;
  onCycleTab = config.cycleTab ?? null;
  onNewTab = config.newTab ?? null;
  onCloseTab = config.closeTab ?? null;
  onToggleCommandPalette = config.toggleCommandPalette ?? null;
  onToast = config.toast ?? null;
  onModeChange = config.onModeChange ?? null;
  gpSessionId = config.gpSessionId ?? null;

  // Listen for gamepad connect/disconnect
  window.addEventListener('gamepadconnected', () => {
    // gamepadLoop handles connect state + toast + rumble; just update HUD
    updateGamepadPill();
  });

  window.addEventListener('gamepaddisconnected', () => {
    connected = false;
    gpIndex = null;
    updateGamepadPill();
    onToast?.('Controller disconnected', 'warn');
  });

  // Start the polling loop
  requestAnimationFrame(gamepadLoop);

  // Create and return the HUD pill
  const pill = createGamepadPill();

  // Mode change updates the pill
  const originalOnMode = onModeChange;
  onModeChange = (mode) => {
    updateGamepadPill();
    originalOnMode?.(mode);
  };

  return pill;
}

/** Update the PTY session ID (call when session changes) */
export function setGamepadSession(sid: string): void {
  gpSessionId = sid;
}

/** Get current gamepad mode */
export function getGamepadMode(): GamepadMode {
  return currentMode;
}

/** Check if controller is connected */
export function isGamepadConnected(): boolean {
  return connected;
}
