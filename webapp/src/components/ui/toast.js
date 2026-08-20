/**
 * Kepçe Toast System - Svelte State Manager
 */

const toastQueue = [];
let activeToast = null;
let subscribers = [];
let transitionTimeoutId = null;

export function subscribeToasts(fn) {
  subscribers.push(fn);
  fn({ activeToast, toastQueue });
  return () => {
    subscribers = subscribers.filter(f => f !== fn);
  };
}

function notify() {
  subscribers.forEach(fn => fn({ activeToast, toastQueue }));
}

function showNextToast() {
  if (activeToast) return;
  if (toastQueue.length === 0) return;

  const item = toastQueue.shift();
  _render(item);
}

function _render(item) {
  const timeoutMs = item.timeout || 4000;

  activeToast = {
    ...item,
    timeoutMs,
    isClosing: false
  };

  notify();

  activeToast.timeoutId = setTimeout(() => {
    dismissActiveToast();
  }, timeoutMs);
}

export function dismissActiveToast() {
  if (!activeToast || activeToast.isClosing) return;

  clearTimeout(activeToast.timeoutId);
  if (transitionTimeoutId) {
    clearTimeout(transitionTimeoutId);
  }
  activeToast.isClosing = true;
  notify();

  // Wait for the exit animation (200ms defined in CSS)
  transitionTimeoutId = setTimeout(() => {
    activeToast = null;
    transitionTimeoutId = null;
    notify();
    setTimeout(showNextToast, 50);
  }, 200);
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export function showToast(message, typeOrOptions = null, legacyAction = null) {
  let type = null;
  let timeout = 4000;
  let priority = 'normal';
  let action = null;

  if (typeof typeOrOptions === 'string') {
    type = typeOrOptions;
  } else if (typeOrOptions && typeof typeOrOptions === 'object') {
    type = typeOrOptions.type ?? type;
    timeout = typeOrOptions.timeout ?? timeout;
    priority = typeOrOptions.priority ?? priority;
    action = typeOrOptions.action ?? null;
  }

  if (!action && legacyAction && legacyAction.text && legacyAction.callback) {
    action = legacyAction;
  }

  if (type === 'error' || type === 'warning') {
    priority = 'high';
  }

  const item = { id: Date.now() + Math.random(), message, timeout, priority, action };

  if (priority === 'high' && activeToast && !activeToast.isClosing) {
    clearTimeout(activeToast.timeoutId);
    if (transitionTimeoutId) {
      clearTimeout(transitionTimeoutId);
    }
    activeToast.isClosing = true;
    notify();

    transitionTimeoutId = setTimeout(() => {
      activeToast = null;
      transitionTimeoutId = null;
      _render(item);
    }, 200);

    return;
  }

  toastQueue.push(item);
  showNextToast();
}

export function dismissAllToasts() {
  toastQueue.length = 0;
  if (transitionTimeoutId) {
    clearTimeout(transitionTimeoutId);
    transitionTimeoutId = null;
  }
  if (activeToast) {
    activeToast = null;
    notify();
  }
}

