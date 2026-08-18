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
 * Şehir listesini önbellekten anında döndürür, arka planda API'den günceller.
 * @returns {Promise<Array<{slug: string, name: string}>>}
 */
export function getCitiesData() {
  if (citiesPromise) return citiesPromise;

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

  // 2. Arka planda (idle) API'den güncel listeyi çek
  citiesPromise = new Promise((resolve) => {
    const fetchFresh = () => {
      api.getCities()
        .then(fresh => {
          if (Array.isArray(fresh) && fresh.length > 0) {
            citiesData = fresh;
            if (typeof window !== 'undefined') {
              localStorage.setItem(CACHE_KEY, JSON.stringify(fresh));
            }
          }
          resolve(citiesData);
        })
        .catch(() => resolve(citiesData));
    };

    if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
      window.requestIdleCallback(fetchFresh, { timeout: 3000 });
    } else {
      setTimeout(fetchFresh, 1500);
    }
  });

  return Promise.resolve(citiesData);
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
