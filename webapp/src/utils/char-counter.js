/**
 * Textarea Karakter Sayacı Utility
 *
 * Kullanım:
 *   import { initCharCounter } from '../utils/char-counter.js';
 *   initCharCounter(textareaEl);
 *
 * Textarea'nın `maxlength` attribute'u varsa sayacı otomatik gösterir.
 * Wrapper'ı `position: relative` olarak ayarlar veya zaten öyleyse kullanır.
 * Sayaç elemanı `data-char-counter` attribute'uyla işaretlenir - bu sayede
 * modal yeniden render edilse bile çift inject olmaz.
 *
 * @param {HTMLTextAreaElement} textarea
 * @param {Object} [opts]
 * @param {number} [opts.limit]   maxlength attribute'u yoksa kullanılacak limit
 * @param {Function} [opts.onUpdate]  (count, limit, isOver) => void  ek callback
 * @returns {{ destroy: Function }}  event listener'ı temizlemek için
 */
export function initCharCounter(textarea, opts = {}) {
  const limit = opts.limit ?? parseInt(textarea.getAttribute('maxlength'), 10);
  if (!limit || isNaN(limit)) return { destroy: () => { } };

  // Inject sadece bir kez yapılır
  let counter = textarea.parentElement?.querySelector('[data-char-counter]');
  if (!counter) {
    counter = document.createElement('span');
    counter.setAttribute('data-char-counter', '');
    counter.className = 'c-char-counter';
    textarea.parentElement?.appendChild(counter);
  }

  const update = () => {
    const count = textarea.value.length;
    const isOver = count > limit;
    counter.textContent = `${count} / ${limit}`;
    counter.classList.toggle('c-char-counter--over', isOver);
    opts.onUpdate?.(count, limit, isOver);
  };

  // İlk render
  update();

  textarea.addEventListener('input', update);
  return {
    destroy: () => textarea.removeEventListener('input', update)
  };
}
