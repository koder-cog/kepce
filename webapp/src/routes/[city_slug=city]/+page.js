import { CITY_MAP } from '@/utils/turkish.js';

export const prerender = true;

/**
 * Entry generator for SvelteKit static prerendering.
 * Generates prerender routes for all 81 Turkish cities.
 * 
 * @type {import('./$types').EntryGenerator}
 */
export function entries() {
    return Object.keys(CITY_MAP).map(slug => ({ city_slug: slug }));
}

/** @type {import('./$types').PageLoad} */
export function load({ params }) {
    return {
        citySlug: params.city_slug
    };
}
