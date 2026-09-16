/**
 * Kepçe Motion Utility
 * Centralized motion logic respecting user preferences and system settings.
 */

// ── Design Tokens (motion.css ile birebir senkronize) ───────────
export const DURATION = {
  instant: 100,
  fast: 200,
  default: 350,
  medium: 500,
  slow: 650,
  slower: 900
};

export const EASING = {
  // Spatial - Layout ve pozisyon değişimleri
  spatialFast: 'cubic-bezier(0.42, 1.67, 0.21, 0.9)',
  spatialDefault: 'cubic-bezier(0.38, 1.21, 0.22, 1)',
  spatialSlow: 'cubic-bezier(0.39, 1.29, 0.35, 0.98)',

  // Effects - Opaklık, renk, gölge (Giriş/Çıkış için harika)
  effectsFast: 'cubic-bezier(0.31, 0.94, 0.34, 1)',
  effectsDefault: 'cubic-bezier(0.25, 1, 0.35, 1)',
  effectsSlow: 'cubic-bezier(0.2, 1, 0.4, 1)',

  // Standard / Material
  standard: 'cubic-bezier(0.4, 0, 0.2, 1)',
  entrance: 'cubic-bezier(0, 0, 0.2, 1)',
  exit: 'cubic-bezier(0.4, 0, 1, 1)',

  // Tactile & Expressive
  tactile: 'cubic-bezier(0.15, 0, 0, 1)',
  springBounce: 'cubic-bezier(0.34, 1.56, 0.64, 1)', // Mikro etkileşimler için (ikon, buton)
  expressive: 'cubic-bezier(0.175, 0.885, 0.32, 1.275)'
};

// ── State & Checks ─────────────────────────────────────────────
export const isMotionEnabled = () => {
  if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
    return true;
  }
  const userPref = localStorage.getItem('kepce_animations') !== 'false';
  const systemPref = !window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  return userPref && systemPref;
};

/**
 * Returns duration in ms respecting motion preferences.
 * Sayı (350) veya token stringi ('default', 'fast') kabul eder.
 * @param {number|keyof typeof DURATION} val
 * @returns {number}
 */
export const getDuration = (val = 'default') => {
  const ms = typeof val === 'string' ? (DURATION[val] ?? DURATION.default) : val;
  return isMotionEnabled() ? ms : 0;
};

/**
 * A promise-based delay that respects motion settings.
 * @param {number|keyof typeof DURATION} ms
 * @returns {Promise<void>}
 */
export const wait = (ms) => new Promise(resolve => setTimeout(resolve, getDuration(ms)));

/**
 * Runs a function in the next animation frame if motion is enabled,
 * otherwise runs it immediately.
 * @param {Function} fn 
 */
export const runNextTick = (fn) => {
  if (isMotionEnabled()) {
    requestAnimationFrame(fn);
  } else {
    fn();
  }
};

/**
 * A wrapper for element.animate() that respects motion settings.
 * @param {HTMLElement} element - The element to animate.
 * @param {Keyframe[]|PropertyIndexedKeyframes} keyframes - Animation keyframes.
 * @param {number|KeyframeAnimationOptions} options - Animation options.
 * @returns {Animation}
 */
export const animate = (element, keyframes, options) => {
  const isEnabled = isMotionEnabled();

  if (typeof options === 'number') {
    options = { duration: isEnabled ? options : 0 };
  } else if (options) {
    const rawDuration = typeof options.duration === 'string'
      ? (DURATION[options.duration] ?? DURATION.default)
      : (options.duration ?? DURATION.default);

    options.duration = isEnabled ? rawDuration : 0;
  } else {
    options = { duration: isEnabled ? DURATION.default : 0 };
  }

  return element.animate(keyframes, options);
};

/**
 * Smart Loading Wrapper
 * Prevents flicker by adding a grace period and minimum duration.
 */
export async function smartLoad(task, onLoading, onComplete, options = {}) {
  const { threshold = 180, minDuration = 400 } = options;
  const startTime = Date.now();
  let isLoadingShown = false;

  const loadingTimeout = setTimeout(() => {
    isLoadingShown = true;
    onLoading();
  }, getDuration(threshold));

  try {
    const result = await task();
    clearTimeout(loadingTimeout);

    if (isLoadingShown) {
      const elapsed = Date.now() - (startTime + getDuration(threshold));
      const remaining = Math.max(0, getDuration(minDuration) - elapsed);
      if (remaining > 0) await new Promise(r => setTimeout(r, remaining));
    }

    onComplete(result);
  } catch (err) {
    clearTimeout(loadingTimeout);
    throw err;
  }
}