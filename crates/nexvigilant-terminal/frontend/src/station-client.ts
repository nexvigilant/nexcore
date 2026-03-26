// NexVigilant Terminal — Station Client
// Calls NexVigilant Station tools directly via Tauri IPC → Rust → HTTPS.
// No Claude needed. The terminal IS the client.

import * as toasts from './toast';

const tauriAvailable: boolean = typeof (window as any).__TAURI__ !== 'undefined';
const invoke = tauriAvailable
  ? (window as any).__TAURI__!.core.invoke
  : async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
      console.log('[station-mock]', cmd, args);
      return null;
    };

/** Result from a station_call Tauri command. */
export interface StationCallResult {
  success: boolean;
  tool: string;
  result: unknown;
  error: string | null;
  duration_ms: number;
  rpc_id: number;
}

/** Station health response. */
export interface StationHealth {
  status: string;
  tools: number;
  configs: number;
  courses: number;
  version: string;
  telemetry: {
    uptime_seconds: number;
    total_calls: number;
    calls_per_minute: number;
    error_rate_pct: number;
    slo_status: string;
  };
}

/** Tool definition from station_tools. */
export interface StationTool {
  name: string;
  description?: string;
  inputSchema?: unknown;
}

// ── Station API ─────────────────────────────────────────────────

/** Check Station health. */
export async function health(): Promise<StationHealth | null> {
  try {
    return await invoke('station_health') as StationHealth;
  } catch (e) {
    toasts.warn('Station Unreachable', String(e));
    return null;
  }
}

/** List all available Station tools. */
export async function tools(): Promise<StationTool[]> {
  try {
    return (await invoke('station_tools') as StationTool[]) || [];
  } catch (e) {
    toasts.warn('Station Tools Failed', String(e));
    return [];
  }
}

/** Call a Station tool directly. Returns structured result. */
export async function call(tool: string, args: Record<string, unknown> = {}): Promise<StationCallResult | null> {
  try {
    const result = await invoke('station_call', { tool, args }) as StationCallResult;
    if (result && result.success) {
      toasts.success(tool, result.duration_ms + 'ms');
    } else if (result) {
      toasts.warn(tool + ' failed', result.error || 'Unknown error');
    }
    return result;
  } catch (e) {
    toasts.warn('Station Call Failed', String(e));
    return null;
  }
}

// ── Results Panel ───────────────────────────────────────────────

let resultsPanelEl: HTMLElement | null = null;
let resultsBodyEl: HTMLElement | null = null;
const resultHistory: StationCallResult[] = [];
const MAX_RESULTS = 10;

/** Mount the results panel into a container. */
export function mountResultsPanel(container: HTMLElement): void {
  resultsPanelEl = document.createElement('div');
  resultsPanelEl.className = 'station-results';
  Object.assign(resultsPanelEl.style, {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    background: '#0d1117',
    border: '1px solid #1a2233',
    borderRadius: '6px',
    padding: '6px',
    marginTop: '4px',
    maxHeight: '250px',
    overflow: 'hidden',
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  });

  // Header
  const header = document.createElement('div');
  Object.assign(header.style, {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    borderBottom: '1px solid #1a2233',
    paddingBottom: '3px',
  });

  const title = document.createElement('span');
  title.textContent = 'STATION RESULTS';
  Object.assign(title.style, { color: '#c084fc', fontWeight: 'bold', fontSize: '9px' });

  const clearBtn = document.createElement('span');
  clearBtn.textContent = 'clear';
  Object.assign(clearBtn.style, { color: '#556677', fontSize: '9px', cursor: 'pointer' });
  clearBtn.addEventListener('click', () => {
    resultHistory.length = 0;
    renderResults();
  });

  header.appendChild(title);
  header.appendChild(clearBtn);
  resultsPanelEl.appendChild(header);

  // Body (scrollable)
  resultsBodyEl = document.createElement('div');
  Object.assign(resultsBodyEl.style, { flex: '1', overflowY: 'auto', maxHeight: '200px' });
  resultsPanelEl.appendChild(resultsBodyEl);

  container.appendChild(resultsPanelEl);
}

/** Add a result and re-render. */
export function pushResult(result: StationCallResult): void {
  resultHistory.unshift(result);
  if (resultHistory.length > MAX_RESULTS) resultHistory.pop();
  renderResults();
}

function renderResults(): void {
  if (!resultsBodyEl) return;
  while (resultsBodyEl.firstChild) resultsBodyEl.removeChild(resultsBodyEl.firstChild);

  if (resultHistory.length === 0) {
    const empty = document.createElement('div');
    empty.textContent = 'No results yet. Call a Station tool to see output here.';
    Object.assign(empty.style, { color: '#556677', fontSize: '10px', padding: '8px 0' });
    resultsBodyEl.appendChild(empty);
    return;
  }

  for (const r of resultHistory) {
    const entry = document.createElement('div');
    Object.assign(entry.style, {
      borderBottom: '1px solid #111827',
      padding: '4px 0',
    });

    // Header line: icon + tool + duration
    const headerLine = document.createElement('div');
    Object.assign(headerLine.style, { display: 'flex', gap: '6px', alignItems: 'center' });

    const icon = document.createElement('span');
    icon.textContent = r.success ? '\u2713' : '\u2717';
    icon.style.color = r.success ? '#34d399' : '#f87171';

    const toolName = document.createElement('span');
    toolName.textContent = r.tool;
    toolName.style.color = '#93c5fd';

    const dur = document.createElement('span');
    dur.textContent = r.duration_ms + 'ms';
    dur.style.color = '#556677';

    headerLine.appendChild(icon);
    headerLine.appendChild(toolName);
    headerLine.appendChild(dur);
    entry.appendChild(headerLine);

    // Result body (collapsible JSON)
    if (r.success && r.result) {
      const body = document.createElement('pre');
      const text = typeof r.result === 'string' ? r.result : JSON.stringify(r.result, null, 2);
      // Truncate long results
      body.textContent = text.length > 500 ? text.substring(0, 497) + '...' : text;
      Object.assign(body.style, {
        color: '#e0e8f0',
        fontSize: '9px',
        margin: '2px 0 0 16px',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        maxHeight: '80px',
        overflow: 'auto',
        background: '#111827',
        padding: '4px',
        borderRadius: '3px',
      });
      entry.appendChild(body);
    } else if (r.error) {
      const errEl = document.createElement('div');
      errEl.textContent = r.error;
      Object.assign(errEl.style, { color: '#f87171', fontSize: '9px', marginLeft: '16px' });
      entry.appendChild(errEl);
    }

    resultsBodyEl.appendChild(entry);
  }
}

// ── Quick Station Calls (pre-built workflows) ───────────────────

/** Search FAERS adverse events for a drug. */
export async function searchFaers(drug: string): Promise<StationCallResult | null> {
  return await call('api_fda_gov_search_adverse_events', { drug, limit: 10 });
}

/** Get drug label from DailyMed. */
export async function getDrugLabel(drug: string): Promise<StationCallResult | null> {
  return await call('dailymed_nlm_nih_gov_search_drugs', { name: drug });
}

/** Compute PRR for a drug-event pair. */
export async function computePrr(drug: string, event: string): Promise<StationCallResult | null> {
  return await call('calculate_nexvigilant_com_compute_prr', { drug, event });
}

/** Compute ROR for a drug-event pair. */
export async function computeRor(drug: string, event: string): Promise<StationCallResult | null> {
  return await call('calculate_nexvigilant_com_compute_ror', { drug, event });
}

/** Get Station health. */
export async function stationHealth(): Promise<StationCallResult | null> {
  return await call('nexvigilant_station_health', {});
}
