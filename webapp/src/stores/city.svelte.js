import { api } from '@/api/index.js';
import { CITY_MAP } from '@/utils/turkish.js';

const DEFAULT_CITIES = Object.entries(CITY_MAP).map(([slug, name], i) => ({
  id: i + 1,
  slug,
  name,
  has_celiac: false
}));

let currentCity = $state(typeof window !== 'undefined' ? localStorage.getItem('kepce_city') || 'istanbul' : 'istanbul');
let onCityChangeListeners = [];

// ── Şehir Listesi (Stale-While-Revalidate Önbellek) ──────────
const CACHE_KEY = 'kepce_cities_cache';
let citiesData = $state(DEFAULT_CITIES);
let citiesLoaded = $state(true);
let citiesPromise = null;

/**
 * Şehir listesini önbellekten anında döndürür (varsayılan 81 il).
 * @returns {Promise<Array<{slug: string, name: string}>>}
 */
export function getCitiesData() {
  // 1. localStorage'dan hemen yükle (varsa zenginleştir)
  if (typeof window !== 'undefined') {
    try {
      const cached = localStorage.getItem(CACHE_KEY);
      if (cached) {
        const parsed = JSON.parse(cached);
        if (Array.isArray(parsed) && parsed.length > 0) {
          citiesData = parsed;
        }
      }
    } catch (_) { /* bozuk cache, yok say */ }
  }

  return Promise.resolve(citiesData);
}

/** Şehir listesini API'den tazelemek için çağrılır. */
export async function refreshCities() {
  try {
    const fresh = await api.getCities();
    if (Array.isArray(fresh) && fresh.length > 0) {
      citiesData = fresh;
      if (typeof window !== 'undefined') {
        localStorage.setItem(CACHE_KEY, JSON.stringify(fresh));
      }
    }
  } catch (_) {}
  return citiesData;
}

/** Reaktif şehir listesine erişim. */
export function getCities() {
  return citiesData;
}

/** Şehir listesinin yüklenip yüklenmediğini döndürür. */
export function isCitiesLoaded() {
  return citiesLoaded;
}

// ── Seçili Şehir Yönetimi ─────────────────────────────────────
export function getCurrentCity() {
  return currentCity;
}

export function setOnCityChange(callback) {
  onCityChangeListeners.push(callback);
}

export function setCurrentCity(slug) {
  currentCity = slug;
  if (typeof window !== 'undefined') {
    localStorage.setItem('kepce_city', slug);
  }
  onCityChangeListeners.forEach(cb => cb(slug));
}
