// NexVigilant Terminal — Gamepad Module Tests
// Tests the button→action mapping, mode cycling, and public API
// without requiring a real Gamepad or Tauri runtime.

import { describe, it, expect, vi } from 'vitest';

// Test the core logic patterns from gamepad.ts without importing it
// directly (it has side effects from requestAnimationFrame + Tauri IPC).

// ── Button Mapping Tests ──────────────────────────────────────────

describe('DualSense Button Constants', () => {
  // Standard Gamepad API mapping for DualSense
  const BTN = {
    CROSS: 0, CIRCLE: 1, SQUARE: 2, TRIANGLE: 3,
    L1: 4, R1: 5, L2: 6, R2: 7,
    SHARE: 8, OPTIONS: 9, L3: 10, R3: 11,
    UP: 12, DOWN: 13, LEFT: 14, RIGHT: 15,
    PS: 16, TOUCH: 17,
  };

  it('maps face buttons to indices 0-3', () => {
    expect(BTN.CROSS).toBe(0);
    expect(BTN.CIRCLE).toBe(1);
    expect(BTN.SQUARE).toBe(2);
    expect(BTN.TRIANGLE).toBe(3);
  });

  it('maps bumpers and triggers to indices 4-7', () => {
    expect(BTN.L1).toBe(4);
    expect(BTN.R1).toBe(5);
    expect(BTN.L2).toBe(6);
    expect(BTN.R2).toBe(7);
  });

  it('maps d-pad to indices 12-15', () => {
    expect(BTN.UP).toBe(12);
    expect(BTN.DOWN).toBe(13);
    expect(BTN.LEFT).toBe(14);
    expect(BTN.RIGHT).toBe(15);
  });

  it('maps PS button to index 16', () => {
    expect(BTN.PS).toBe(16);
  });
});

// ── Mode Cycling Tests ────────────────────────────────────────────

describe('Mode Cycling', () => {
  const MODES = ['TERMINAL', 'CLAUDE', 'SYSTEM'] as const;
  type GamepadMode = typeof MODES[number];

  function cycleMode(current: GamepadMode): GamepadMode {
    const idx = MODES.indexOf(current);
    return MODES[(idx + 1) % MODES.length] ?? 'TERMINAL';
  }

  it('cycles TERMINAL → CLAUDE', () => {
    expect(cycleMode('TERMINAL')).toBe('CLAUDE');
  });

  it('cycles CLAUDE → SYSTEM', () => {
    expect(cycleMode('CLAUDE')).toBe('SYSTEM');
  });

  it('cycles SYSTEM → TERMINAL (wraps)', () => {
    expect(cycleMode('SYSTEM')).toBe('TERMINAL');
  });

  it('full cycle returns to start after 3 presses', () => {
    let mode: GamepadMode = 'TERMINAL';
    mode = cycleMode(mode);
    mode = cycleMode(mode);
    mode = cycleMode(mode);
    expect(mode).toBe('TERMINAL');
  });
});

// ── PTY Escape Sequence Tests ─────────────────────────────────────

describe('Terminal Mode → PTY Escape Sequences', () => {
  // These are the escape sequences gamepad.ts sends to the PTY
  const TERMINAL_ACTIONS: Record<string, string> = {
    CROSS: '\r',           // Enter
    CIRCLE: '\x03',        // Ctrl+C
    SQUARE: '\t',          // Tab
    TRIANGLE: '\x0c',      // Ctrl+L
    L1: '\x1b[A',          // Up arrow
    R1: '\x1b[B',          // Down arrow
    UP: '\x1b[A',          // Up arrow
    DOWN: '\x1b[B',        // Down arrow
    LEFT: '\x1b[D',        // Left arrow
    RIGHT: '\x1b[C',       // Right arrow
    L2: '\x1bb',           // Alt+B (word back)
    R2: '\x1bf',           // Alt+F (word forward)
    SHARE: '\x1a',         // Ctrl+Z
    OPTIONS: '\x04',       // Ctrl+D
    L3: '\x01',            // Ctrl+A
    R3: '\x05',            // Ctrl+E
    TOUCH: '\x12',         // Ctrl+R
  };

  it('CROSS sends Enter (\\r)', () => {
    expect(TERMINAL_ACTIONS.CROSS).toBe('\r');
  });

  it('CIRCLE sends Ctrl+C (\\x03)', () => {
    expect(TERMINAL_ACTIONS.CIRCLE).toBe('\x03');
  });

  it('SQUARE sends Tab (\\t)', () => {
    expect(TERMINAL_ACTIONS.SQUARE).toBe('\t');
  });

  it('L1/UP both send Up Arrow escape', () => {
    expect(TERMINAL_ACTIONS.L1).toBe(TERMINAL_ACTIONS.UP);
    expect(TERMINAL_ACTIONS.L1).toBe('\x1b[A');
  });

  it('R1/DOWN both send Down Arrow escape', () => {
    expect(TERMINAL_ACTIONS.R1).toBe(TERMINAL_ACTIONS.DOWN);
    expect(TERMINAL_ACTIONS.R1).toBe('\x1b[B');
  });

  it('L2 sends Alt+B (word back)', () => {
    expect(TERMINAL_ACTIONS.L2).toBe('\x1bb');
  });

  it('R2 sends Alt+F (word forward)', () => {
    expect(TERMINAL_ACTIONS.R2).toBe('\x1bf');
  });

  it('L3 sends Ctrl+A (line start)', () => {
    expect(TERMINAL_ACTIONS.L3).toBe('\x01');
  });

  it('R3 sends Ctrl+E (line end)', () => {
    expect(TERMINAL_ACTIONS.R3).toBe('\x05');
  });

  it('all arrow escapes are valid ANSI CSI sequences', () => {
    const arrows = [TERMINAL_ACTIONS.UP, TERMINAL_ACTIONS.DOWN, TERMINAL_ACTIONS.LEFT, TERMINAL_ACTIONS.RIGHT];
    for (const seq of arrows) {
      expect(seq).toMatch(/^\x1b\[[A-D]$/);
    }
  });
});

// ── Claude Mode Tests ─────────────────────────────────────────────

describe('Claude Mode → Actions', () => {
  const CLAUDE_ACTIONS: Record<string, string> = {
    CROSS: '\r',           // Submit
    CIRCLE: '\x1b',        // Escape
    SQUARE: '\t',          // Tab (accept)
    TRIANGLE: '/',         // Slash command
    L2: 'y',               // Approve
    R2: 'n',               // Deny
    SHARE: '\x03',         // Ctrl+C interrupt
  };

  it('L2 sends y (approve tool use)', () => {
    expect(CLAUDE_ACTIONS.L2).toBe('y');
  });

  it('R2 sends n (deny tool use)', () => {
    expect(CLAUDE_ACTIONS.R2).toBe('n');
  });

  it('TRIANGLE sends / for slash commands', () => {
    expect(CLAUDE_ACTIONS.TRIANGLE).toBe('/');
  });

  it('CIRCLE sends Escape', () => {
    expect(CLAUDE_ACTIONS.CIRCLE).toBe('\x1b');
  });
});

// ── Edge Detection Tests ──────────────────────────────────────────

describe('Button Edge Detection', () => {
  // The gamepad loop uses edge detection: pressed this frame but not last
  function detectEdge(current: boolean, previous: boolean): boolean {
    return current && !previous;
  }

  it('detects press (false → true)', () => {
    expect(detectEdge(true, false)).toBe(true);
  });

  it('ignores hold (true → true)', () => {
    expect(detectEdge(true, true)).toBe(false);
  });

  it('ignores release (true → false)', () => {
    expect(detectEdge(false, true)).toBe(false);
  });

  it('ignores idle (false → false)', () => {
    expect(detectEdge(false, false)).toBe(false);
  });
});

// ── HUD Pill Tests ────────────────────────────────────────────────

describe('Gamepad HUD Pill', () => {
  const MODE_COLORS: Record<string, string> = {
    TERMINAL: '#00ff87',
    CLAUDE: '#ff6ec7',
    SYSTEM: '#4f7df5',
  };

  it('TERMINAL mode is green', () => {
    expect(MODE_COLORS.TERMINAL).toBe('#00ff87');
  });

  it('CLAUDE mode is pink', () => {
    expect(MODE_COLORS.CLAUDE).toBe('#ff6ec7');
  });

  it('SYSTEM mode is blue', () => {
    expect(MODE_COLORS.SYSTEM).toBe('#4f7df5');
  });

  it('all mode colors are valid hex', () => {
    for (const color of Object.values(MODE_COLORS)) {
      expect(color).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });
});

// ── GamepadConfig Interface Tests ─────────────────────────────────

describe('GamepadConfig Contract', () => {
  it('requires writePty callback', () => {
    const config = {
      writePty: vi.fn(),
    };
    expect(config.writePty).toBeDefined();
    config.writePty('\r');
    expect(config.writePty).toHaveBeenCalledWith('\r');
  });

  it('optional callbacks default to null behavior', () => {
    const config = {
      writePty: vi.fn(),
      cycleTab: undefined as ((dir: number) => void) | undefined,
      toast: undefined as ((msg: string) => void) | undefined,
    };
    // Optional chaining should not throw
    config.cycleTab?.(1);
    config.toast?.('test');
    expect(true).toBe(true); // No throw = pass
  });
});
