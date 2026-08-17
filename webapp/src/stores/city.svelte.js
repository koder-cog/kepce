import { api } from '@/api/index.js';

let currentCity = $state(typeof window !== 'undefined' ? localStorage.getItem('kepce_city') || 'istanbul' : 'istanbul');
let onCityChangeListeners = [];

// ── Şehir Listesi (Stale-While-Revalidate Önbellek) ──────────
const CACHE_KEY = 'kepce_cities_cache';
let citiesData = $state([]);
let citiesLoaded = $state(false);
let citiesPromise = null;

/**
 * Şehir listesini önbellekten anında döndürür, arka planda API'den günceller.
 * İlk çağrıda önbellek yoksa API yanıtını bekler.
 * @returns {Promise<Array<{slug: string, name: string}>>}
 */
export function getCitiesData() {
  if (citiesPromise) return citiesPromise;

  // 1. localStorage'dan hemen yükle (sıfır gecikme)
  if (typeof window !== 'undefined') {
    try {
      const cached = localStorage.getItem(CACHE_KEY);
      if (cached) {
        const parsed = JSON.parse(cached);
        if (Array.isArray(parsed) && parsed.length > 0) {
          citiesData = parsed;
          citiesLoaded = true;
        }
      }
    } catch (_) { /* bozuk cache, yok say */ }
  }

  // 2. Arka planda API'den güncel listeyi çek
  citiesPromise = api.getCities()
    .then(fresh => {
      if (Array.isArray(fresh) && fresh.length > 0) {
        citiesData = fresh;
        citiesLoaded = true;
        if (typeof window !== 'undefined') {
          localStorage.setItem(CACHE_KEY, JSON.stringify(fresh));
        }
      }
      return citiesData;
    })
    .catch(err => {
      console.error('[CityStore] API çağrısı başarısız:', err);
      // Cache varsa onu kullanmaya devam et
      if (citiesData.length > 0) {
        citiesLoaded = true;
      }
      return citiesData;
    });

  // 3. Eğer cache'den yüklenmişse anında dön, yoksa promise'i beklet
  if (citiesLoaded) {
    return Promise.resolve(citiesData);
  }
  return citiesPromise;
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
