/**
 * Kepçe - HTML Sanitizer Utility.
 *
 * Wraps DOMPurify to prevent XSS attacks when rendering user-generated
 * content via innerHTML. All user-facing text (comments, nicknames, etc.)
 * MUST pass through `sanitize()` before being interpolated into templates.
 *
 * Usage:
 *   import { sanitize } from '../utils/sanitize.js';
 *   container.innerHTML = `<p>${sanitize(userText)}</p>`;
 */
import DOMPurify from 'dompurify';

/**
 * Sanitize a string of potentially dangerous HTML.
 * Strips all script tags, event handlers, and dangerous attributes.
 *
 * @param {string} dirty - Raw user input that may contain HTML/scripts.
 * @param {object} [config] - Optional DOMPurify config overrides.
 * @returns {string} Clean, safe HTML string.
 */
export function sanitize(dirty, config = {}) {
  if (dirty == null) return '';
  const str = String(dirty);
  if (typeof DOMPurify !== 'undefined' && typeof DOMPurify.sanitize === 'function') {
    return DOMPurify.sanitize(str, {
      ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'br', 'span'],
      ALLOWED_ATTR: ['class'],
      ...config,
    });
  }
  // SSR Fallback: Tehlikeli script ve handler'ları temizle
  return str
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    .replace(/\bon\w+\s*=\s*(?:'[^']*'|"[^"]*"|[^\s>]+)/gi, '');
}

/**
 * Sanitize for plain text output - strips ALL HTML tags.
 * Use this when inserting into contexts where no HTML is expected.
 *
 * @param {string} dirty - Raw user input.
 * @returns {string} Plain text with all HTML removed.
 */
export function sanitizeText(dirty) {
  if (dirty == null) return '';
  const str = String(dirty);
  if (typeof DOMPurify !== 'undefined' && typeof DOMPurify.sanitize === 'function') {
    return DOMPurify.sanitize(str, { ALLOWED_TAGS: [], ALLOWED_ATTR: [] });
  }
  // SSR Fallback: Tüm HTML etiketlerini temizle
  return str.replace(/<[^>]*>?/gm, '');
}

