// NexVigilant Terminal — Toast Notification System
// Stackable notifications for system alerts, agent events, and signal detection.

export type ToastLevel = 'info' | 'success' | 'warning' | 'critical';

const ICONS: Record<ToastLevel, string> = {
  info: '\u2139',     // ℹ
  success: '\u2713',  // ✓
  warning: '\u26a0',  // ⚠
  critical: '\u2717', // ✗
};

const MAX_TOASTS = 6;
const DEFAULT_DURATION_MS = 5000;

let toastContainer: HTMLElement | null = null;

function getContainer(): HTMLElement {
  if (!toastContainer) {
    toastContainer = document.getElementById('toast-container');
    if (!toastContainer) {
      toastContainer = document.createElement('div');
      toastContainer.className = 'toast-container';
      toastContainer.id = 'toast-container';
      document.body.appendChild(toastContainer);
    }
  }
  return toastContainer;
}

/**
 * Show a toast notification.
 * Returns a dismiss function.
 */
export function toast(
  level: ToastLevel,
  title: string,
  message?: string,
  durationMs: number = DEFAULT_DURATION_MS,
): () => void {
  const container = getContainer();

  // Enforce max toasts — remove oldest
  while (container.children.length >= MAX_TOASTS) {
    const oldest = container.firstChild as HTMLElement | null;
    if (oldest) dismissToast(oldest);
  }

  const el = document.createElement('div');
  el.className = 'toast ' + level;

  const icon = document.createElement('span');
  icon.className = 'toast-icon';
  icon.textContent = ICONS[level];
  el.appendChild(icon);

  const body = document.createElement('div');
  body.className = 'toast-body';

  const titleEl = document.createElement('div');
  titleEl.className = 'toast-title';
  titleEl.textContent = title;
  body.appendChild(titleEl);

  if (message) {
    const msgEl = document.createElement('div');
    msgEl.className = 'toast-message';
    msgEl.textContent = message;
    body.appendChild(msgEl);
  }

  const timeEl = document.createElement('div');
  timeEl.className = 'toast-time';
  timeEl.textContent = new Date().toLocaleTimeString();
  body.appendChild(timeEl);

  el.appendChild(body);

  // Click to dismiss
  el.addEventListener('click', () => dismissToast(el));

  container.appendChild(el);

  // Auto-dismiss
  let timer: ReturnType<typeof setTimeout> | null = null;
  if (durationMs > 0) {
    timer = setTimeout(() => dismissToast(el), durationMs);
  }

  return () => {
    if (timer) clearTimeout(timer);
    dismissToast(el);
  };
}

function dismissToast(el: HTMLElement): void {
  if (el.classList.contains('dismissing')) return;
  el.classList.add('dismissing');
  setTimeout(() => el.remove(), 200);
}

// Convenience exports
export const info = (title: string, message?: string): (() => void) => toast('info', title, message);
export const success = (title: string, message?: string): (() => void) => toast('success', title, message);
export const warn = (title: string, message?: string): (() => void) => toast('warning', title, message);
export const critical = (title: string, message?: string): (() => void) => toast('critical', title, message, 0); // critical stays until dismissed
