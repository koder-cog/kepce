/**
 * Kepçe Motion Utility
 * Centralized motion logic respecting user preferences and system settings.
 */

export const isMotionEnabled = () => {
  // SSR/prerender sırasında `window` ve `localStorage` tanımsız; statik
  // derleme patlamasın diye varsayılan olarak "açık" dönüyoruz (canlı
  // session'da zaten gerçek değerlere bakılıyor).
  if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
    return true;
  }
  // Check in-app setting (default to true)
  const userPref = localStorage.getItem('kepce_animations') !== 'false';

  // Check system-level reduced motion preference
  const systemPref = !window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  return userPref && systemPref;
};

/**
 * Returns the duration in milliseconds, or 0 if motion is disabled.
 * @param {number} ms - The default duration in ms.
 * @returns {number}
 */
export const getDuration = (ms) => isMotionEnabled() ? ms : 0;

/**
 * A promise-based delay that respects motion settings.
 * @param {number} ms - The default delay in ms.
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
    options.duration = isEnabled ? options.duration : 0;
  } else {
    options = { duration: isEnabled ? 300 : 0 };
  }
  
  return element.animate(keyframes, options);
};

/**
 * Smart Loading Wrapper
 * Prevents flicker by adding a grace period and minimum duration.
 * @param {Function} task - The async function to execute.
 * @param {Function} onLoading - Called if the task takes longer than the threshold.
 * @param {Function} onComplete - Called with the task results.
 * @param {Object} options - Custom durations.
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
