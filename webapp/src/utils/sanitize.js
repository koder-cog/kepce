/**
 * Kepçe — HTML Sanitizer Utility.
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
 * @param {string} dirty — Raw user input that may contain HTML/scripts.
 * @param {object} [config] — Optional DOMPurify config overrides.
 * @returns {string} Clean, safe HTML string.
 */
export function sanitize(dirty, config = {}) {
  if (dirty == null) return '';
  return DOMPurify.sanitize(String(dirty), {
    ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'br', 'span'],
    ALLOWED_ATTR: ['class'],
    ...config,
  });
}

/**
 * Sanitize for plain text output — strips ALL HTML tags.
 * Use this when inserting into contexts where no HTML is expected.
 *
 * @param {string} dirty — Raw user input.
 * @returns {string} Plain text with all HTML removed.
 */
export function sanitizeText(dirty) {
  if (dirty == null) return '';
  return DOMPurify.sanitize(String(dirty), { ALLOWED_TAGS: [], ALLOWED_ATTR: [] });
}
