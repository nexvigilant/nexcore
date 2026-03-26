// NexVigilant Terminal — Remote Controller Panel
// Visual control surface for Claude-driven terminal operations.
// Renders action history, system state, and diff indicators.
// Screenshot-capturable for visual feedback steering.

import * as station from './station-client';

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

  let executeAction = async function(action: RemoteAction): Promise<void> {
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

  // ── PTY Injection Bridge ────────────────────────────────────────
  // Maps remote controller actions to shell commands that get injected
  // into the active PTY session. If Claude Code is running in the terminal,
  // button presses become Claude input automatically.

  const PTY_COMMAND_MAP: Record<string, string> = {
    SystemSnapshot: '/system-status',
    GetHealth: '/pulse-program',
    CloudOverview: '/system-status',
    ListSessions: '/system-status',
    CreateSession: '',  // handled by remote_execute, not PTY injection
    StartHealthPolling: '',
    StopHealthPolling: '',
  };

  /**
   * Inject a command string into the active PTY session.
   * If the PTY has Claude Code running, this becomes Claude input.
   * The command is written as if the user typed it + Enter.
   */
  async function injectIntoPty(command: string): Promise<void> {
    if (!command) return;

    // Find active session ID from the global scope
    // Access via window — main.ts exposes sessionId on the module scope
    const sid = (window as any).__nvt_sessionId;
    if (!sid) return;

    try {
      await invoke('pty_write', { sessionId: sid, data: command + '\r' });
    } catch (e) {
      console.warn('[pty-inject] write failed:', e);
    }
  }

  // Add PTY inject buttons row
  const injectLabel = el('div', { color: '#556677', fontSize: '9px', marginTop: '4px' }, 'PTY INJECT (sends to active terminal)');
  panel.appendChild(injectLabel);

  const injectRow = el('div', { display: 'flex', gap: '4px', flexWrap: 'wrap' });
  injectRow.className = 'remote-inject';

  const injectButtons = [
    { label: 'System Status', cmd: '/system-status', color: '#60a5fa' },
    { label: 'Signal Check', cmd: '/station-signal ', color: '#34d399' },
    { label: 'Station Demo', cmd: '/station-demo', color: '#c084fc' },
    { label: 'FAERS Query', cmd: '/station-signal metformin lactic acidosis', color: '#fbbf24' },
    { label: 'Commit', cmd: '/commit', color: '#34d399' },
    { label: 'Progress', cmd: '/progress-program', color: '#60a5fa' },
    { label: 'CRAFT', cmd: '/craft-program', color: '#ff66b2' },
    { label: 'Exhale', cmd: '/exhale', color: '#f87171' },
  ];

  for (const ib of injectButtons) {
    const btn = actionButton(ib.label, ib.cmd, ib.color);
    btn.title = 'Sends: ' + ib.cmd;
    btn.addEventListener('click', async () => {
      await injectIntoPty(ib.cmd);
      // Visual feedback
      btn.style.opacity = '0.5';
      setTimeout(() => { btn.style.opacity = '1'; }, 300);
    });
    injectRow.appendChild(btn);
  }
  panel.appendChild(injectRow);

  // Also inject when remote_execute runs an action with a PTY mapping
  const originalExecute = executeAction;
  executeAction = async function(action: RemoteAction): Promise<void> {
    await originalExecute(action);
    const cmd = PTY_COMMAND_MAP[action.action];
    if (cmd) {
      await injectIntoPty(cmd);
    }
  };

  // ── Custom Command Input ──────────────────────────────────────
  const cmdSection = el('div', { marginTop: '4px' });
  const cmdLabel = el('div', { color: '#556677', fontSize: '9px', marginBottom: '2px' }, 'COMMAND INPUT');
  cmdSection.appendChild(cmdLabel);

  const cmdRow = el('div', { display: 'flex', gap: '4px' });
  const cmdInput = document.createElement('input');
  cmdInput.type = 'text';
  cmdInput.placeholder = 'Type command to inject...';
  Object.assign(cmdInput.style, {
    flex: '1',
    background: '#111827',
    border: '1px solid #1a2233',
    color: '#e0e8f0',
    padding: '4px 8px',
    borderRadius: '3px',
    fontFamily: 'inherit',
    fontSize: '11px',
    outline: 'none',
  });
  cmdInput.addEventListener('focus', () => { cmdInput.style.borderColor = '#34d399'; });
  cmdInput.addEventListener('blur', () => { cmdInput.style.borderColor = '#1a2233'; });
  cmdInput.addEventListener('keydown', async (e: KeyboardEvent) => {
    if (e.key === 'Enter' && cmdInput.value.trim()) {
      e.preventDefault();
      const cmd = cmdInput.value.trim();
      await injectIntoPty(cmd);
      // Add to recent commands
      addRecentCommand(cmd);
      cmdInput.value = '';
    }
  });

  const sendBtn = actionButton('Send', '', '#34d399');
  sendBtn.addEventListener('click', async () => {
    if (cmdInput.value.trim()) {
      await injectIntoPty(cmdInput.value.trim());
      addRecentCommand(cmdInput.value.trim());
      cmdInput.value = '';
    }
  });

  cmdRow.appendChild(cmdInput);
  cmdRow.appendChild(sendBtn);
  cmdSection.appendChild(cmdRow);

  // Recent commands (last 5, clickable to re-send)
  const recentDiv = el('div', { display: 'flex', gap: '3px', flexWrap: 'wrap', marginTop: '3px' });
  recentDiv.className = 'recent-commands';
  cmdSection.appendChild(recentDiv);
  panel.appendChild(cmdSection);

  const recentCommands: string[] = [];

  function addRecentCommand(cmd: string): void {
    // Deduplicate
    const idx = recentCommands.indexOf(cmd);
    if (idx >= 0) recentCommands.splice(idx, 1);
    recentCommands.unshift(cmd);
    if (recentCommands.length > 5) recentCommands.pop();
    renderRecent();
  }

  function renderRecent(): void {
    while (recentDiv.firstChild) recentDiv.removeChild(recentDiv.firstChild);
    for (const cmd of recentCommands) {
      const chip = el('span', {
        background: '#1a2233',
        color: '#93c5fd',
        padding: '1px 6px',
        borderRadius: '3px',
        fontSize: '9px',
        cursor: 'pointer',
      }, cmd.length > 30 ? cmd.substring(0, 27) + '...' : cmd);
      chip.title = cmd;
      chip.addEventListener('click', async () => {
        await injectIntoPty(cmd);
        chip.style.opacity = '0.5';
        setTimeout(() => { chip.style.opacity = '1'; }, 300);
      });
      recentDiv.appendChild(chip);
    }
  }

  // ── Station PV Workflow Panel ─────────────────────────────────
  // One-click research courses from NexVigilant Station.
  // Each button injects the full pipeline command into the terminal.

  const pvSection = el('div', { marginTop: '4px' });

  const pvHeader = el('div', {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    cursor: 'pointer',
  });
  const pvTitle = el('span', { color: '#c084fc', fontSize: '9px', fontWeight: 'bold' }, 'PV WORKFLOWS');
  const pvToggle = el('span', { color: '#556677', fontSize: '9px' }, '\u25BC');
  pvHeader.appendChild(pvTitle);
  pvHeader.appendChild(pvToggle);

  const pvBody = el('div', { display: 'flex', flexDirection: 'column', gap: '3px', marginTop: '3px' });

  let pvExpanded = true;
  pvHeader.addEventListener('click', () => {
    pvExpanded = !pvExpanded;
    pvBody.style.display = pvExpanded ? 'flex' : 'none';
    pvToggle.textContent = pvExpanded ? '\u25BC' : '\u25B6';
  });

  const pvWorkflows = [
    {
      label: 'Drug Safety Profile',
      desc: 'Full 6-step: name \u2192 FAERS \u2192 ADRs \u2192 lit \u2192 EU \u2192 WHO',
      cmd: '/station-signal ',
      color: '#34d399',
      needsInput: true,
      placeholder: 'drug name',
    },
    {
      label: 'Signal Investigation',
      desc: 'FAERS \u2192 disproportionality \u2192 EU \u2192 case reports \u2192 SAEs',
      cmd: '/station-signal ',
      color: '#60a5fa',
      needsInput: true,
      placeholder: 'drug event',
    },
    {
      label: 'Competitive Landscape',
      desc: 'Drug targets \u2192 head-to-head \u2192 clinical pipeline',
      cmd: 'Compare the safety profiles of ',
      color: '#fbbf24',
      needsInput: true,
      placeholder: 'drug1 vs drug2',
    },
    {
      label: 'Station Demo',
      desc: 'Run sample tools across all 16 configs',
      cmd: '/station-demo',
      color: '#c084fc',
      needsInput: false,
      placeholder: '',
    },
    {
      label: 'Microgram Dashboard',
      desc: 'Ecosystem health: 448 programs, chains, coverage',
      cmd: '/mg dashboard',
      color: '#ff66b2',
      needsInput: false,
      placeholder: '',
    },
    {
      label: 'System Health',
      desc: '13-surface + 6-wire health check',
      cmd: '/system-status',
      color: '#00ccff',
      needsInput: false,
      placeholder: '',
    },
  ];

  for (const wf of pvWorkflows) {
    const row = el('div', { display: 'flex', gap: '4px', alignItems: 'center' });

    const btn = actionButton(wf.label, wf.cmd, wf.color);
    btn.title = wf.desc;
    Object.assign(btn.style, { minWidth: '120px', textAlign: 'left' });

    if (wf.needsInput) {
      const wfInput = document.createElement('input');
      wfInput.type = 'text';
      wfInput.placeholder = wf.placeholder;
      Object.assign(wfInput.style, {
        flex: '1',
        background: '#111827',
        border: '1px solid #1a2233',
        color: '#e0e8f0',
        padding: '2px 6px',
        borderRadius: '3px',
        fontFamily: 'inherit',
        fontSize: '10px',
        outline: 'none',
      });

      const go = async (): Promise<void> => {
        const fullCmd = wf.cmd + wfInput.value.trim();
        if (wfInput.value.trim()) {
          await injectIntoPty(fullCmd);
          addRecentCommand(fullCmd);
          wfInput.value = '';
        }
      };

      wfInput.addEventListener('keydown', async (e: KeyboardEvent) => {
        if (e.key === 'Enter') { e.preventDefault(); await go(); }
      });
      btn.addEventListener('click', go);

      row.appendChild(btn);
      row.appendChild(wfInput);
    } else {
      btn.addEventListener('click', async () => {
        await injectIntoPty(wf.cmd);
        addRecentCommand(wf.cmd);
        btn.style.opacity = '0.5';
        setTimeout(() => { btn.style.opacity = '1'; }, 300);
      });
      row.appendChild(btn);
    }

    const desc = el('span', { color: '#556677', fontSize: '8px', marginLeft: '4px' }, wf.desc);
    row.appendChild(desc);
    pvBody.appendChild(row);
  }

  pvSection.appendChild(pvHeader);
  pvSection.appendChild(pvBody);
  panel.appendChild(pvSection);

  // ── Station Direct Panel ──────────────────────────────────────
  // Call Station tools WITHOUT Claude. Results appear in the results panel.

  const directSection = el('div', { marginTop: '4px' });

  const directHeader = el('div', {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    cursor: 'pointer',
  });
  const directTitle = el('span', { color: '#34d399', fontSize: '9px', fontWeight: 'bold' }, 'STATION DIRECT (no Claude needed)');
  const directToggle = el('span', { color: '#556677', fontSize: '9px' }, '\u25B6');
  directHeader.appendChild(directTitle);
  directHeader.appendChild(directToggle);

  const directBody = el('div', { display: 'none', flexDirection: 'column', gap: '3px', marginTop: '3px' });

  let directExpanded = false;
  directHeader.addEventListener('click', () => {
    directExpanded = !directExpanded;
    directBody.style.display = directExpanded ? 'flex' : 'none';
    directToggle.textContent = directExpanded ? '\u25BC' : '\u25B6';
  });

  // Station Direct workflow rows — each calls station_call via Rust→HTTPS
  const directWorkflows = [
    { label: 'FAERS Search', tool: 'api_fda_gov_search_adverse_events', argKey: 'drug', placeholder: 'drug name', color: '#34d399' },
    { label: 'DailyMed Label', tool: 'dailymed_nlm_nih_gov_search_drugs', argKey: 'name', placeholder: 'drug name', color: '#60a5fa' },
    { label: 'Compute PRR', tool: 'calculate_nexvigilant_com_compute_prr', argKey: 'drug', argKey2: 'event', placeholder: 'drug event', color: '#fbbf24' },
    { label: 'Compute ROR', tool: 'calculate_nexvigilant_com_compute_ror', argKey: 'drug', argKey2: 'event', placeholder: 'drug event', color: '#c084fc' },
    { label: 'Station Health', tool: 'nexvigilant_station_health', argKey: '', placeholder: '', color: '#00ccff' },
    { label: 'PubMed Search', tool: 'pubmed_ncbi_nlm_nih_gov_search_articles', argKey: 'query', placeholder: 'search query', color: '#ff66b2' },
  ];

  for (const dw of directWorkflows) {
    const row = el('div', { display: 'flex', gap: '4px', alignItems: 'center' });

    const btn = actionButton(dw.label, dw.tool, dw.color);
    Object.assign(btn.style, { minWidth: '110px', textAlign: 'left' });

    if (dw.argKey) {
      const dwInput = document.createElement('input');
      dwInput.type = 'text';
      dwInput.placeholder = dw.placeholder;
      Object.assign(dwInput.style, {
        flex: '1',
        background: '#111827',
        border: '1px solid #1a2233',
        color: '#e0e8f0',
        padding: '2px 6px',
        borderRadius: '3px',
        fontFamily: 'inherit',
        fontSize: '10px',
        outline: 'none',
      });

      const callDirect = async (): Promise<void> => {
        const val = dwInput.value.trim();
        if (!val) return;
        btn.style.opacity = '0.5';

        // Build args — handle single or dual arg (drug + event)
        const args: Record<string, unknown> = {};
        if ((dw as any).argKey2) {
          const parts = val.split(/\s+/);
          args[dw.argKey] = parts[0] || '';
          args[(dw as any).argKey2] = parts.slice(1).join(' ') || '';
        } else {
          args[dw.argKey] = val;
        }

        const result = await station.call(dw.tool, args);
        if (result) station.pushResult(result);
        btn.style.opacity = '1';
        dwInput.value = '';
      };

      dwInput.addEventListener('keydown', async (e: KeyboardEvent) => {
        if (e.key === 'Enter') { e.preventDefault(); await callDirect(); }
      });
      btn.addEventListener('click', callDirect);
      row.appendChild(btn);
      row.appendChild(dwInput);
    } else {
      btn.addEventListener('click', async () => {
        btn.style.opacity = '0.5';
        const result = await station.call(dw.tool, {});
        if (result) station.pushResult(result);
        btn.style.opacity = '1';
      });
      row.appendChild(btn);
    }

    directBody.appendChild(row);
  }

  directSection.appendChild(directHeader);
  directSection.appendChild(directBody);
  panel.appendChild(directSection);

  // ── Results Panel ─────────────────────────────────────────────
  station.mountResultsPanel(panel);
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
