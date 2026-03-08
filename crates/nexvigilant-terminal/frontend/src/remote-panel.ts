// NexVigilant Terminal — Remote Controller Panel
// Visual control surface for Claude-driven terminal operations.
// Renders action history, system state, and diff indicators.
// Screenshot-capturable for visual feedback steering.

interface RemoteAction {
  action: string;
  params?: Record<string, unknown>;
}

interface StateDiff {
  field: string;
  before: unknown;
  after: unknown;
}

interface SystemState {
  active_sessions: number;
  active_ptys: number;
  health_band: string;
  chi: number;
  health_polling: boolean;
  cloud_services: number;
  cloud_healthy: number;
  shell_state: string;
  timestamp_ms: number;
}

interface RemoteResult {
  success: boolean;
  action: string;
  before: SystemState;
  after: SystemState;
  diff: StateDiff[];
  duration_ms: number;
  payload: unknown;
  error: string | null;
  seq: number;
}

interface ControllerEvent {
  action: string;
  success: boolean;
  diff_count: number;
  chi: number;
  health_band: string;
}

// Band color mapping (matches chi-indicator.ts)
const BAND_COLORS: Record<string, string> = {
  SpinningUp: '#60a5fa', // blue
  Healthy: '#34d399',    // emerald
  Depleting: '#fbbf24',  // amber
  Critical: '#f87171',   // red
};

const MAX_HISTORY = 50;

/** Create a styled element with given tag, styles, and optional text. */
function el(
  tag: string,
  styles: Record<string, string>,
  text?: string,
): HTMLElement {
  const elem = document.createElement(tag);
  Object.assign(elem.style, styles);
  if (text !== undefined) {
    elem.textContent = text;
  }
  return elem;
}

/** Create a button with color accent. */
function actionButton(label: string, actionName: string, color: string): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.textContent = label;
  btn.dataset.action = actionName;
  btn.className = 'action-btn';
  Object.assign(btn.style, {
    background: 'transparent',
    border: `1px solid ${color}40`,
    color,
    padding: '2px 8px',
    borderRadius: '3px',
    cursor: 'pointer',
    fontFamily: 'inherit',
    fontSize: '10px',
  });
  return btn;
}

/**
 * Mount the remote controller panel into a container element.
 * Creates a visual dashboard showing system state, action history,
 * and state diffs — all capturable via screenshot for feedback.
 */
export async function mountRemotePanel(container: HTMLElement): Promise<void> {
  const tauriAvailable = typeof (window as any).__TAURI__ !== 'undefined';
  const invoke = tauriAvailable
    ? (window as any).__TAURI__.core.invoke
    : async (cmd: string, args?: unknown) => {
        console.log('[remote-mock]', cmd, args);
        return null;
      };

  const listen = tauriAvailable
    ? (window as any).__TAURI__.event.listen
    : async (_event: string, _handler: (e: unknown) => void) => () => {};

  // State
  let history: RemoteResult[] = [];
  let currentState: SystemState | null = null;
  let actionCount = 0;

  // Build DOM safely (no innerHTML)
  const panel = el('div', {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    background: '#0d1117',
    color: '#e0e8f0',
    border: '1px solid #1a2233',
    borderRadius: '6px',
    padding: '8px',
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
    maxHeight: '300px',
    overflow: 'hidden',
  });
  panel.className = 'remote-panel';

  // -- Header --
  const header = el('div', {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingBottom: '4px',
    borderBottom: '1px solid #1a2233',
  });
  const headerTitle = el('span', { color: '#34d399', fontWeight: 'bold' }, 'REMOTE CONTROLLER');
  const statusSpan = el('span', { color: '#556677' }, 'idle');
  statusSpan.className = 'remote-status';
  header.appendChild(headerTitle);
  header.appendChild(statusSpan);
  panel.appendChild(header);

  // -- State grid --
  const stateGrid = el('div', {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '4px',
  });
  stateGrid.className = 'remote-state';

  const stateFields = [
    { field: 'sessions', label: 'SESSIONS', initial: '0' },
    { field: 'chi', label: 'CHI', initial: '0.00' },
    { field: 'band', label: 'BAND', initial: 'SpinningUp' },
    { field: 'cloud', label: 'CLOUD', initial: '0/0' },
  ];

  for (const sf of stateFields) {
    const cell = el('div', { textAlign: 'center' });
    cell.className = 'state-cell';
    cell.dataset.field = sf.field;

    const labelEl = el('div', { color: '#556677', fontSize: '9px' }, sf.label);
    const valueEl = el('div', {
      fontSize: sf.field === 'band' ? '12px' : '16px',
      fontWeight: 'bold',
    }, sf.initial);
    valueEl.className = 'state-value';

    cell.appendChild(labelEl);
    cell.appendChild(valueEl);
    stateGrid.appendChild(cell);
  }
  panel.appendChild(stateGrid);

  // -- Action buttons --
  const actionsRow = el('div', { display: 'flex', gap: '4px', flexWrap: 'wrap' });
  actionsRow.className = 'remote-actions';

  const buttonDefs = [
    { label: 'Snapshot', action: 'SystemSnapshot', color: '#60a5fa' },
    { label: '+ Session', action: 'CreateSession', color: '#34d399' },
    { label: 'Health', action: 'GetHealth', color: '#fbbf24' },
    { label: 'Cloud', action: 'CloudOverview', color: '#c084fc' },
    { label: 'Sessions', action: 'ListSessions', color: '#60a5fa' },
    { label: 'Poll On', action: 'StartHealthPolling', color: '#34d399' },
    { label: 'Poll Off', action: 'StopHealthPolling', color: '#f87171' },
  ];

  for (const bd of buttonDefs) {
    const btn = actionButton(bd.label, bd.action, bd.color);
    btn.addEventListener('click', async () => {
      const action = buildAction(bd.action);
      await executeAction(action);
    });
    actionsRow.appendChild(btn);
  }
  panel.appendChild(actionsRow);

  // -- History section --
  const historySection = el('div', {
    flex: '1',
    overflowY: 'auto',
    fontSize: '10px',
    maxHeight: '120px',
  });
  historySection.className = 'remote-history';

  const historyLabel = el('div', { color: '#556677', fontSize: '9px', marginBottom: '2px' }, 'ACTION HISTORY');
  const historyList = el('div', {});
  historyList.className = 'history-list';

  historySection.appendChild(historyLabel);
  historySection.appendChild(historyList);
  panel.appendChild(historySection);

  container.appendChild(panel);

  // Controller event listener
  await listen('controller-state-changed', (event: { payload: ControllerEvent }) => {
    updateStatusIndicator(event.payload);
  });

  // Initial snapshot
  try {
    currentState = await invoke('remote_snapshot') as SystemState;
    if (currentState) {
      updateStateDisplay(currentState);
    }
    actionCount = (await invoke('remote_action_count') as number) || 0;
  } catch {
    // Running outside Tauri — use mock state
    currentState = mockState();
    updateStateDisplay(currentState);
  }

  // --- Internal functions ---

  async function executeAction(action: RemoteAction): Promise<void> {
    statusSpan.textContent = 'executing...';
    statusSpan.style.color = '#fbbf24';

    try {
      const result = await invoke('remote_execute', { action }) as RemoteResult;
      if (result) {
        history.unshift(result);
        if (history.length > MAX_HISTORY) history.pop();
        currentState = result.after;
        actionCount = result.seq;
        updateStateDisplay(result.after);
        updateHistory();
      }
    } catch (e) {
      console.error('[remote]', e);
    }

    statusSpan.textContent = `#${actionCount} idle`;
    statusSpan.style.color = '#556677';
  }

  function updateStateDisplay(state: SystemState): void {
    const cells = panel.querySelectorAll('.state-cell');
    cells.forEach((cell) => {
      const field = (cell as HTMLElement).dataset.field;
      const valueEl = cell.querySelector('.state-value') as HTMLElement;
      if (!field || !valueEl) return;

      switch (field) {
        case 'sessions':
          valueEl.textContent = String(state.active_sessions);
          break;
        case 'chi':
          valueEl.textContent = state.chi.toFixed(2);
          break;
        case 'band':
          valueEl.textContent = state.health_band;
          valueEl.style.color = BAND_COLORS[state.health_band] || '#e0e8f0';
          break;
        case 'cloud':
          valueEl.textContent = `${state.cloud_healthy}/${state.cloud_services}`;
          break;
      }
    });
  }

  function updateHistory(): void {
    // Clear and rebuild using safe DOM methods
    while (historyList.firstChild) {
      historyList.removeChild(historyList.firstChild);
    }

    for (const r of history.slice(0, 10)) {
      const row = el('div', { padding: '1px 0', borderBottom: '1px solid #111827' });

      const icon = el('span', { color: r.success ? '#34d399' : '#f87171' }, r.success ? '\u2713' : '\u2717');
      const seq = el('span', { color: '#93c5fd' }, `#${r.seq}`);
      const name = el('span', {}, ` ${r.action} `);
      const duration = el('span', { color: '#556677' }, `${r.duration_ms}ms`);

      row.appendChild(icon);
      row.appendChild(seq);
      row.appendChild(name);
      row.appendChild(duration);

      if (r.diff.length > 0) {
        const diffBadge = el('span', { color: '#fbbf24' }, ` \u0394${r.diff.length}`);
        row.appendChild(diffBadge);
      }

      if (r.error) {
        const errSpan = el('span', { color: '#f87171' }, ` ${r.error}`);
        row.appendChild(errSpan);
      }

      historyList.appendChild(row);
    }
  }

  function updateStatusIndicator(event: ControllerEvent): void {
    const color = event.success ? '#34d399' : '#f87171';
    statusSpan.textContent = `${event.action} \u0394${event.diff_count}`;
    statusSpan.style.color = color;
    setTimeout(() => {
      statusSpan.textContent = 'idle';
      statusSpan.style.color = '#556677';
    }, 2000);
  }

  function buildAction(name: string): RemoteAction {
    switch (name) {
      case 'CreateSession':
        return { action: 'CreateSession', params: { mode: 'hybrid' } };
      default:
        return { action: name };
    }
  }
}

function mockState(): SystemState {
  return {
    active_sessions: 0,
    active_ptys: 0,
    health_band: 'SpinningUp',
    chi: 0.0,
    health_polling: false,
    cloud_services: 4,
    cloud_healthy: 0,
    shell_state: 'running',
    timestamp_ms: 0,
  };
}
