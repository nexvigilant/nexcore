// Session Feed — Live multi-session Claude Code monitoring via SSE
// Connects to session-dashboard-server.py on localhost:9800

const SESSION_FEED_PORT = 9800;
const SESSION_FEED_URL = `http://localhost:${SESSION_FEED_PORT}`;

interface ToolEvent {
  type: 'tool' | 'verdict' | 'session_start';
  session_id?: string;
  session_color?: string;
  tool?: string;
  file?: string;
  command?: string;
  verdict?: string;
  proposition?: string;
  started_at?: string;
  cwd?: string;
}

const TOOL_COLORS: Record<string, string> = {
  Read: '#38bdf8', Grep: '#a78bfa', Glob: '#818cf8',
  Edit: '#34d399', Write: '#4ade80', Bash: '#f97316',
  MCP: '#fbbf24', Agent: '#f87171', Other: '#64748b',
};

function classify(name: string): string {
  if (['Read', 'Grep', 'Glob', 'Edit', 'Write', 'Bash', 'Agent'].includes(name)) return name;
  if (name.startsWith('mcp__') || name.startsWith('MCP') || name === 'ToolSearch') return 'MCP';
  return 'Other';
}

interface SessionState {
  sessionId: string;
  shortId: string;
  color: string;
  startTime: number;
  totalTools: number;
  filesModified: Set<string>;
  counts: Record<string, number>;
  verdict: string | null;
  proposition: string | null;
  feedItems: ToolEvent[];
}

// All tracked sessions
const sessions = new Map<string, SessionState>();
let activeSessionId: string | null = null;
let evtSource: EventSource | null = null;

function getOrCreateSession(sid: string, color?: string, startedAt?: string): SessionState {
  let s = sessions.get(sid);
  if (!s) {
    s = {
      sessionId: sid,
      shortId: sid.substring(0, 8),
      color: color || '#38bdf8',
      startTime: startedAt ? new Date(startedAt).getTime() : Date.now(),
      totalTools: 0,
      filesModified: new Set(),
      counts: { Read: 0, Grep: 0, Glob: 0, Edit: 0, Write: 0, Bash: 0, MCP: 0, Agent: 0, Other: 0 } as Record<string, number>,
      verdict: null,
      proposition: null,
      feedItems: [],
    };
    sessions.set(sid, s);
  }
  return s;
}

function formatElapsed(ms: number): string {
  const secs = Math.floor(ms / 1000);
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function getVerdictColor(verdict: string): string {
  const colors: Record<string, string> = {
    fully_demonstrated: '#34d399',
    partially_demonstrated: '#fbbf24',
    not_demonstrated: '#f87171',
    unmeasured: '#94a3b8',
  };
  return colors[verdict] || '#94a3b8';
}

// ── DOM Updates ────────────────────────────────────────────────

function renderSessionTabs(): void {
  const tabContainer = document.getElementById('sf-tabs');
  if (!tabContainer) return;

  tabContainer.innerHTML = '';

  // Sort: live sessions first, then by start time desc
  const sorted = [...sessions.values()].sort((a, b) => {
    const aLive = !a.verdict;
    const bLive = !b.verdict;
    if (aLive !== bLive) return aLive ? -1 : 1;
    return b.startTime - a.startTime;
  });

  for (const s of sorted) {
    const tab = document.createElement('div');
    tab.className = 'sf-tab' + (s.sessionId === activeSessionId ? ' active' : '');
    const dotColor = s.verdict ? getVerdictColor(s.verdict) : s.color;
    const dotAnim = s.verdict ? '' : 'animation:pulse 2s infinite;';
    tab.innerHTML = `<span class="sf-dot" style="background:${dotColor};${dotAnim}"></span><span>${s.shortId}</span><span class="sf-tab-count">${s.totalTools}</span>`;
    tab.addEventListener('click', () => {
      activeSessionId = s.sessionId;
      renderSessionTabs();
      renderActiveSession();
    });
    tabContainer.appendChild(tab);
  }
}

function renderActiveSession(): void {
  if (!activeSessionId) return;
  const s = sessions.get(activeSessionId);
  if (!s) return;

  const set = (id: string, val: string | number): void => {
    const e = document.getElementById(id);
    if (e) e.textContent = String(val);
  };

  set('sf-tools', s.totalTools);
  set('sf-files', s.filesModified.size);
  set('sf-reads', (s.counts['Read'] || 0) + (s.counts['Grep'] || 0) + (s.counts['Glob'] || 0));
  set('sf-writes', (s.counts['Edit'] || 0) + (s.counts['Write'] || 0));
  set('sf-bash', s.counts['Bash'] || 0);
  set('sf-mcp', s.counts['MCP'] || 0);

  // Update elapsed
  set('sf-elapsed', formatElapsed(Date.now() - s.startTime));

  // Activity bar
  const bar = document.getElementById('sf-bar');
  if (bar && s.totalTools > 0) {
    let html = '';
    for (const [k, v] of Object.entries(s.counts)) {
      if (v === 0) continue;
      const pct = Math.max((v / s.totalTools) * 100, 3);
      html += `<div style="width:${pct}%;background:${TOOL_COLORS[k] || '#64748b'};height:100%;transition:width 0.3s" title="${k} ${v}"></div>`;
    }
    bar.innerHTML = html;
  }

  // Connection dot
  const dot = document.getElementById('sf-conn-dot');
  if (dot) {
    if (s.verdict) {
      dot.style.background = getVerdictColor(s.verdict);
      dot.style.animation = 'none';
    } else {
      dot.style.background = s.color;
      dot.style.animation = 'pulse 2s infinite';
    }
  }

  // Verdict banner
  const verdictEl = document.getElementById('sf-verdict');
  if (verdictEl) {
    if (s.verdict) {
      const vc = getVerdictColor(s.verdict);
      verdictEl.innerHTML = `<span style="color:${vc};font-weight:700">${s.verdict.replace(/_/g, ' ').toUpperCase()}</span>`;
      if (s.proposition) {
        verdictEl.innerHTML += `<span class="sf-prop">${s.proposition}</span>`;
      }
      verdictEl.style.borderColor = vc;
      verdictEl.style.display = 'block';
    } else {
      verdictEl.style.display = 'none';
    }
  }
}

function addFeedItem(sid: string, data: ToolEvent): void {
  // Only render if this is the active session
  if (sid !== activeSessionId) return;

  const list = document.getElementById('sf-feed-list');
  if (!list) return;

  const cat = classify(data.tool || '');
  const color = TOOL_COLORS[cat] || TOOL_COLORS['Other'];
  const now = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });

  let label = data.tool || 'event';
  if (data.file) {
    label = data.file.split('/').pop() || label;
  } else if (data.tool === 'Bash' && data.command) {
    label = data.command.substring(0, 40);
  }

  const item = document.createElement('div');
  item.className = 'sf-item';
  item.innerHTML = `
    <span class="sf-dot" style="background:${color}"></span>
    <span class="sf-label">${label}</span>
    <span class="sf-time">${now}</span>
  `;
  list.insertBefore(item, list.firstChild);
  while (list.children.length > 50) {
    list.removeChild(list.lastChild as Node);
  }
}

function updateConnectionStatus(connected: boolean): void {
  const dot = document.getElementById('sf-conn-dot');
  if (!dot) return;
  if (connected) {
    dot.style.background = '#34d399';
    dot.style.animation = 'pulse 2s infinite';
    dot.title = 'Connected to session feed';
  } else {
    dot.style.background = '#f87171';
    dot.style.animation = 'none';
    dot.title = 'Session feed disconnected';
  }
}

// ── SSE Connection ─────────────────────────────────────────────

function connectSSE(): void {
  if (evtSource) {
    evtSource.close();
  }

  const source = new EventSource(`${SESSION_FEED_URL}/stream`);
  evtSource = source;

  source.onopen = (): void => {
    updateConnectionStatus(true);
  };

  source.onmessage = (e: MessageEvent): void => {
    let data: ToolEvent;
    try {
      data = JSON.parse(e.data) as ToolEvent;
    } catch {
      return;
    }

    const sid = data.session_id || '_default';

    if (data.type === 'tool') {
      const s = getOrCreateSession(sid, data.session_color);
      s.totalTools++;
      const cat = classify(data.tool || '');
      s.counts[cat] = (s.counts[cat] || 0) + 1;
      if (data.file) s.filesModified.add(data.file);
      s.feedItems.push(data);
      if (s.feedItems.length > 200) s.feedItems.shift();

      // Auto-select first session
      if (!activeSessionId) {
        activeSessionId = sid;
      }
      addFeedItem(sid, data);
      renderSessionTabs();
      if (sid === activeSessionId) renderActiveSession();

    } else if (data.type === 'verdict') {
      const s = getOrCreateSession(sid, data.session_color);
      s.verdict = data.verdict || 'unmeasured';
      s.proposition = data.proposition || null;
      renderSessionTabs();
      if (sid === activeSessionId) renderActiveSession();

    } else if (data.type === 'session_start') {
      const s = getOrCreateSession(sid, data.session_color, data.started_at);
      // New session auto-activates
      activeSessionId = sid;
      // Clear feed list for new session view
      const list = document.getElementById('sf-feed-list');
      if (list) list.innerHTML = '';
      renderSessionTabs();
      renderActiveSession();

      // Notify via title bar flash
      const section = document.getElementById('sf-section');
      if (section) {
        section.style.borderLeft = `2px solid ${s.color}`;
        setTimeout(() => { section.style.borderLeft = ''; }, 2000);
      }
    }
  };

  source.onerror = (): void => {
    updateConnectionStatus(false);
    source.close();
    evtSource = null;
    setTimeout(connectSSE, 5000);
  };
}

// ── Elapsed Timer ──────────────────────────────────────────────

function startElapsedTimer(): void {
  setInterval(() => {
    if (activeSessionId) {
      const s = sessions.get(activeSessionId);
      if (s) {
        const el = document.getElementById('sf-elapsed');
        if (el) el.textContent = formatElapsed(Date.now() - s.startTime);
      }
    }
    // Update tab counts periodically
    renderSessionTabs();
  }, 1000);
}

// ── Mount ────────────────────────────────────────────��─────────

export function mountSessionFeed(): void {
  fetch(`${SESSION_FEED_URL}/health`)
    .then((res) => {
      if (res.ok) {
        connectSSE();
        startElapsedTimer();
        const section = document.getElementById('sf-section');
        if (section) section.style.display = 'flex';
        const panel = document.getElementById('side-panel');
        if (panel && panel.classList.contains('hidden')) {
          panel.classList.remove('hidden');
        }
      }
    })
    .catch(() => {
      setTimeout(mountSessionFeed, 10000);
    });
}
