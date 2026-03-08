// chi-indicator.ts — χ (chi) Session Health Indicator
// Listens for Tauri `session-health` events and renders a pill in the title bar.
// No external dependencies. Vanilla DOM + CSS custom properties.

export type HealthBand = 'SpinningUp' | 'Healthy' | 'Depleting' | 'Critical';

export interface SessionHealth {
  chi: number;
  tau_acc: number;
  tau_wind: number;
  tau_disk: number;
  equilibrium: boolean;
  band: HealthBand;
}

// ── CSS (injected once into <head>) ─────────────────────────────────────────

const CSS = `
  :root {
    --chi-spinning-up: #60a5fa;
    --chi-healthy:     #4ade80;
    --chi-depleting:   #fb923c;
    --chi-critical:    #f87171;
    --chi-bg:          rgba(255,255,255,0.06);
    --chi-transition:  background-color 600ms ease, color 600ms ease,
                       box-shadow 600ms ease;
  }

  #chi-indicator {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 11px;
    font-family: inherit;
    background: var(--chi-bg);
    border: 1px solid transparent;
    cursor: default;
    transition: var(--chi-transition);
    position: relative;
    -webkit-app-region: no-drag;
  }

  #chi-indicator[data-band="SpinningUp"] {
    color: var(--chi-spinning-up);
    border-color: var(--chi-spinning-up);
    box-shadow: 0 0 6px var(--chi-spinning-up);
  }
  #chi-indicator[data-band="Healthy"] {
    color: var(--chi-healthy);
    border-color: var(--chi-healthy);
    box-shadow: 0 0 6px var(--chi-healthy);
  }
  #chi-indicator[data-band="Depleting"] {
    color: var(--chi-depleting);
    border-color: var(--chi-depleting);
    box-shadow: 0 0 6px var(--chi-depleting);
  }
  #chi-indicator[data-band="Critical"] {
    color: var(--chi-critical);
    border-color: var(--chi-critical);
    box-shadow: 0 0 8px var(--chi-critical);
    animation: chi-pulse 1.4s ease-in-out infinite;
  }

  @keyframes chi-pulse {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.65; }
  }

  .chi-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
    flex-shrink: 0;
  }

  /* Tooltip */
  #chi-tooltip {
    display: none;
    position: absolute;
    bottom: calc(100% + 6px);
    right: 0;
    background: #111822;
    border: 1px solid #2a3a4e;
    border-radius: 6px;
    padding: 8px 10px;
    font-size: 11px;
    color: #e0e8f0;
    white-space: nowrap;
    pointer-events: none;
    z-index: 9999;
    line-height: 1.7;
  }
  #chi-indicator:hover #chi-tooltip {
    display: block;
  }
`;

// ── Band label mapping ───────────────────────────────────────────────────────

const BAND_LABEL: Record<HealthBand, string> = {
  SpinningUp: 'spinning up',
  Healthy:    'healthy',
  Depleting:  'depleting',
  Critical:   'critical',
};

// ── DOM construction ─────────────────────────────────────────────────────────

function injectStyles(): void {
  if (document.getElementById('chi-indicator-styles')) return;
  const style = document.createElement('style');
  style.id = 'chi-indicator-styles';
  style.textContent = CSS;
  document.head.appendChild(style);
}

/** Build a single text node row: `label + value` */
function row(label: string, value: string): Text {
  return document.createTextNode(`${label} ${value}`);
}

function buildTooltipContent(health: SessionHealth): DocumentFragment {
  const { chi, band, tau_acc, tau_wind, tau_disk, equilibrium } = health;
  const frag = document.createDocumentFragment();

  // Line 1: χ value and band name
  const line1 = document.createElement('div');
  const strong = document.createElement('strong');
  strong.textContent = `\u03C7 = ${chi.toFixed(4)}`;   // χ = …
  line1.appendChild(strong);
  line1.appendChild(document.createTextNode(` \u2014 ${BAND_LABEL[band]}`)); // — band
  frag.appendChild(line1);

  // Line 2: torques
  const line2 = document.createElement('div');
  line2.appendChild(row('\u03C4_acc', tau_acc.toFixed(3)));   // τ_acc
  line2.appendChild(document.createTextNode('\u00A0\u00A0'));
  line2.appendChild(row('\u03C4_wind', tau_wind.toFixed(3)));
  line2.appendChild(document.createTextNode('\u00A0\u00A0'));
  line2.appendChild(row('\u03C4_disk', tau_disk.toFixed(3)));
  frag.appendChild(line2);

  // Line 3: equilibrium
  const line3 = document.createElement('div');
  line3.appendChild(document.createTextNode(
    `Torques: ${equilibrium ? 'balanced' : 'unbalanced'}`,
  ));
  frag.appendChild(line3);

  return frag;
}

function buildElement(): HTMLElement {
  const pill = document.createElement('span');
  pill.id = 'chi-indicator';
  pill.setAttribute('data-band', 'SpinningUp');

  const dot = document.createElement('span');
  dot.className = 'chi-dot';
  dot.setAttribute('aria-hidden', 'true');

  const label = document.createElement('span');
  label.id = 'chi-label';
  label.textContent = '\u03C7 --';   // χ --

  const tooltip = document.createElement('div');
  tooltip.id = 'chi-tooltip';
  tooltip.textContent = 'Awaiting health data\u2026';   // …

  pill.appendChild(dot);
  pill.appendChild(label);
  pill.appendChild(tooltip);

  return pill;
}

// ── State update ─────────────────────────────────────────────────────────────

function applyHealth(pill: HTMLElement, health: SessionHealth): void {
  const { chi, band } = health;

  const label = pill.querySelector<HTMLElement>('#chi-label');
  const tooltip = pill.querySelector<HTMLElement>('#chi-tooltip');

  if (label) {
    label.textContent = `\u03C7 ${chi.toFixed(2)}`;
  }

  pill.setAttribute('data-band', band);

  if (tooltip) {
    // Replace tooltip content with safe DOM nodes — no innerHTML
    while (tooltip.firstChild) tooltip.removeChild(tooltip.firstChild);
    tooltip.appendChild(buildTooltipContent(health));
  }
}

// ── Tauri event listener ─────────────────────────────────────────────────────

type UnlistenFn = () => void;

declare global {
  interface Window {
    __TAURI__?: {
      core: { invoke: (cmd: string, args?: unknown) => Promise<unknown> };
      event: {
        listen: <T>(
          event: string,
          handler: (e: { payload: T }) => void,
        ) => Promise<UnlistenFn>;
      };
    };
  }
}

// ── Public API ───────────────────────────────────────────────────────────────

/**
 * Mount the χ health indicator into `container` and begin listening for
 * Tauri `session-health` events.  Returns an unlisten cleanup function.
 *
 * @param container - DOM element that receives the pill (e.g. `.title-bar .right`)
 */
export async function mountChiIndicator(
  container: HTMLElement,
): Promise<UnlistenFn> {
  injectStyles();

  const pill = buildElement();
  container.prepend(pill);   // leftmost item in the right section

  const tauriAvailable = typeof window.__TAURI__ !== 'undefined';

  if (!tauriAvailable) {
    // Dev-mode stub: cycle through bands every 3 s for visual testing
    const stubs: SessionHealth[] = [
      { chi: 0.01, band: 'SpinningUp', tau_acc: 0.02, tau_wind: 0.01, tau_disk: 0.01, equilibrium: false },
      { chi: 0.12, band: 'Healthy',    tau_acc: 0.12, tau_wind: 0.09, tau_disk: 0.07, equilibrium: true  },
      { chi: 0.45, band: 'Depleting',  tau_acc: 0.28, tau_wind: 0.11, tau_disk: 0.06, equilibrium: false },
      { chi: 0.75, band: 'Critical',   tau_acc: 0.50, tau_wind: 0.15, tau_disk: 0.10, equilibrium: false },
    ];
    let idx = 0;
    const tick = (): void => {
      const stub = stubs[idx % stubs.length];
      if (stub) applyHealth(pill, stub);
      idx++;
    };
    tick();
    const timerId = window.setInterval(tick, 3000);
    return () => window.clearInterval(timerId);
  }

  // Production: listen for backend health events
  const unlisten = await window.__TAURI__!.event.listen<SessionHealth>(
    'session-health',
    (event) => {
      applyHealth(pill, event.payload);
    },
  );

  return unlisten;
}
