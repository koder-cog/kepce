import { CITY_MAP } from '../utils/turkish.js';

/**
 * Validates if the route parameter is a valid Turkish city slug.
 * Used by SvelteKit route matching [city_slug=city] to prevent collision with static routes.
 * 
 * @param {string} param
 * @returns {boolean}
 */
export function match(param) {
  return typeof param === 'string' && param.toLowerCase() in CITY_MAP;
}
