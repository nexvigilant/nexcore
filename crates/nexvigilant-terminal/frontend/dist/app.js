// NexVigilant Terminal — Frontend Application
// Safe DOM construction (no innerHTML with dynamic content)

const tauriAvailable = typeof window.__TAURI__ !== 'undefined';

const invoke = tauriAvailable
  ? window.__TAURI__.core.invoke
  : async function mockInvoke(cmd, args) {
      const mocks = {
        cloud_list_services: [
          { name: 'nexcore-mcp', port: 0, state: 'running', health: 'healthy', uptime_secs: 3600, restarts: 0 },
          { name: 'nexcore-api', port: 3030, state: 'running', health: 'healthy', uptime_secs: 3600, restarts: 0 },
        ],
        cloud_overview: { total_services: 2, healthy: 2, unhealthy: 0, platform: 'nexcore' },
        cloud_events: [],
        shell_list_apps: [
          { id: 'terminal', name: 'Terminal', state: 'running', icon: null },
          { id: 'observatory', name: 'Observatory', state: 'stopped', icon: null },
        ],
        shell_status: { state: 'running', user: 'matthew', active_apps: 1, notifications: 0 },
        terminal_create_session: { id: 'ses-001', mode: 'hybrid', status: 'active', cols: 120, rows: 40, mcp_calls: 0, ai_tokens: 0 },
        terminal_send_input: 'OK',
      };
      return mocks[cmd] || null;
    };

// -- State --
let currentSession = null;
let currentMode = 'hybrid';

// -- Safe DOM helpers --
function createElement(tag, className, textContent) {
  const el = document.createElement(tag);
  if (className) el.className = className;
  if (textContent) el.textContent = textContent;
  return el;
}

function clearChildren(el) {
  while (el.firstChild) el.removeChild(el.firstChild);
}

// -- Terminal --
const terminalInput = document.getElementById('terminalInput');
const terminalOutput = document.getElementById('terminalOutput');

function addLine(text, cls) {
  const div = createElement('div', 'terminal-line ' + (cls || ''), text);
  terminalOutput.appendChild(div);
  div.scrollIntoView({ behavior: 'smooth' });
}

terminalInput.addEventListener('keydown', async function(e) {
  if (e.key !== 'Enter') return;
  const cmd = terminalInput.value.trim();
  if (!cmd) return;
  terminalInput.value = '';
  addLine('nexvig> ' + cmd, '');

  if (cmd === '/help') {
    addLine('Commands:', 'info');
    addLine('  /mode <shell|regulatory|ai|hybrid>  Switch terminal mode', 'muted');
    addLine('  /services                           List cloud services', 'muted');
    addLine('  /dashboard                          Switch to dashboard', 'muted');
    addLine('  /clear                              Clear terminal', 'muted');
    addLine('  nexvig> <query>                     MCP regulatory query', 'muted');
    addLine('  @claude <message>                   AI mode query', 'muted');
    return;
  }
  if (cmd === '/clear') {
    clearChildren(terminalOutput);
    return;
  }
  if (cmd === '/services') {
    try {
      const svcs = await invoke('cloud_list_services');
      svcs.forEach(function(s) {
        const icon = s.health === 'healthy' ? '\u2713' : '\u2717';
        addLine('  ' + icon + ' ' + s.name.padEnd(20) + ' :' + String(s.port).padEnd(6) + ' ' + s.state,
          s.health === 'healthy' ? 'system' : 'error');
      });
    } catch (err) {
      addLine('Error: ' + err, 'error');
    }
    return;
  }
  if (cmd.startsWith('/mode ')) {
    const mode = cmd.slice(6).trim();
    try {
      await invoke('terminal_switch_mode', { sessionId: currentSession ? currentSession.id : '', mode: mode });
      currentMode = mode;
      updateModeUI();
      addLine('Mode switched to: ' + mode, 'info');
    } catch (err) {
      addLine('Error: ' + err, 'error');
    }
    return;
  }
  if (currentSession) {
    try {
      const result = await invoke('terminal_send_input', { sessionId: currentSession.id, data: cmd });
      if (result) addLine(result, '');
    } catch (err) {
      addLine('Error: ' + err, 'error');
    }
  }
});

// -- Tab switching --
function switchTab(tab) {
  document.querySelectorAll('.tab').forEach(function(t) { t.classList.remove('active'); });
  var tabEl = document.querySelector('[data-tab="' + tab + '"]');
  if (tabEl) tabEl.classList.add('active');

  document.getElementById('terminalPanel').style.display = tab === 'terminal' ? 'block' : 'none';
  document.getElementById('dashboardPanel').className = 'dashboard-panel' + (tab === 'dashboard' ? ' active' : '');
  document.getElementById('eventsPanel').className = 'dashboard-panel' + (tab === 'events' ? ' active' : '');

  if (tab === 'dashboard') loadDashboard();
  if (tab === 'events') loadEvents();
}
// Expose to onclick
window.switchTab = switchTab;

// -- Mode UI --
function updateModeUI() {
  var badge = document.getElementById('modeBadge');
  badge.textContent = currentMode.toUpperCase();
  badge.className = 'mode-badge ' + currentMode;
  var prompts = { shell: '$ ', regulatory: 'nexvig> ', ai: '@claude ', hybrid: 'nexvig> ' };
  document.getElementById('promptText').textContent = prompts[currentMode] || 'nexvig> ';
}

// -- Dashboard --
async function loadDashboard() {
  try {
    var overview = await invoke('cloud_overview');
    var grid = document.getElementById('metricGrid');
    clearChildren(grid);

    var metrics = [
      { label: 'Total Services', value: overview.total_services, cls: '' },
      { label: 'Healthy', value: overview.healthy, cls: 'green' },
      { label: 'Unhealthy', value: overview.unhealthy, cls: overview.unhealthy > 0 ? 'red' : 'green' },
      { label: 'Platform', value: overview.platform, cls: '', small: true },
    ];
    metrics.forEach(function(m) {
      var card = createElement('div', 'metric-card');
      card.appendChild(createElement('div', 'metric-label', m.label));
      var val = createElement('div', 'metric-value' + (m.cls ? ' ' + m.cls : ''), String(m.value));
      if (m.small) val.style.fontSize = '16px';
      card.appendChild(val);
      grid.appendChild(card);
    });
  } catch (err) {
    console.error('Dashboard load error:', err);
  }
}

// -- Events --
async function loadEvents() {
  try {
    var events = await invoke('cloud_events', { limit: 50 });
    var list = document.getElementById('eventList');
    clearChildren(list);
    if (!events || events.length === 0) {
      var li = createElement('li', 'event-item');
      li.appendChild(createElement('span', 'event-time', '--:--'));
      li.appendChild(createElement('span', 'event-service', 'system'));
      li.appendChild(createElement('span', '', 'No events recorded yet'));
      list.appendChild(li);
      return;
    }
    events.forEach(function(ev) {
      var li = createElement('li', 'event-item');
      li.appendChild(createElement('span', 'event-time', ev.timestamp));
      li.appendChild(createElement('span', 'event-service', ev.service));
      li.appendChild(createElement('span', '', ev.message));
      list.appendChild(li);
    });
  } catch (err) {
    console.error('Events load error:', err);
  }
}

// -- Sidebar --
async function loadSidebar() {
  try {
    var services = await invoke('cloud_list_services');
    var serviceList = document.getElementById('serviceList');
    clearChildren(serviceList);
    services.forEach(function(s) {
      var item = createElement('div', 'service-item');
      var dot = createElement('span', 'status-indicator ' + (s.health === 'healthy' ? 'green' : 'red'));
      item.appendChild(dot);
      item.appendChild(createElement('span', 'name', s.name));
      item.appendChild(createElement('span', 'port', s.port ? String(s.port) : 'stdio'));
      serviceList.appendChild(item);
    });

    var apps = await invoke('shell_list_apps');
    var appList = document.getElementById('appList');
    clearChildren(appList);
    apps.forEach(function(a) {
      var item = createElement('div', 'service-item');
      item.appendChild(createElement('span', 'status-indicator ' + (a.state === 'running' ? 'green' : 'amber')));
      item.appendChild(createElement('span', 'name', a.name));
      item.addEventListener('click', function() { launchApp(a.id); });
      appList.appendChild(item);
    });

    var overview = await invoke('cloud_overview');
    document.getElementById('cloudStatus').textContent = 'Cloud: ' + overview.healthy + '/' + overview.total_services + ' healthy';
  } catch (err) {
    console.error('Sidebar load error:', err);
  }
}

async function launchApp(appId) {
  try {
    await invoke('shell_launch_app', { appId: appId });
    addLine('Launched: ' + appId, 'system');
    loadSidebar();
  } catch (err) {
    addLine('Failed to launch ' + appId + ': ' + err, 'error');
  }
}
window.launchApp = launchApp;

// -- Init --
async function init() {
  await loadSidebar();
  currentSession = await invoke('terminal_create_session', { mode: 'hybrid' });
  if (currentSession) {
    document.getElementById('sessionId').textContent = 'Session: ' + currentSession.id;
    document.getElementById('sessionCount').textContent = 'Sessions: 1';
  }
  addLine('Session created. Ready.', 'system');
  terminalInput.focus();
}

init();
