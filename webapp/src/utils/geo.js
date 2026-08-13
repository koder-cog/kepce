/**
 * Geolocation utility — detect user's city from coordinates or IP.
 */
import { api } from '../api/index.js';

// Major Turkish cities with KYK dorms and their approximate coordinates
const CITY_COORDS = {
  ankara:    { lat: 39.9334, lng: 32.8597 },
  istanbul:  { lat: 41.0082, lng: 28.9784 },
  izmir:     { lat: 38.4237, lng: 27.1428 },
  bursa:     { lat: 40.1826, lng: 29.0665 },
  antalya:   { lat: 36.8969, lng: 30.7133 },
  adana:     { lat: 37.0000, lng: 35.3213 },
  konya:     { lat: 37.8715, lng: 32.4846 },
  gaziantep: { lat: 37.0662, lng: 37.3833 },
  kayseri:   { lat: 38.7312, lng: 35.4787 },
  eskisehir: { lat: 39.7767, lng: 30.5206 },
  trabzon:   { lat: 41.0027, lng: 39.7168 },
  samsun:    { lat: 41.2867, lng: 36.3300 },
  erzurum:   { lat: 39.9055, lng: 41.2658 },
  diyarbakir:{ lat: 37.9144, lng: 40.2306 },
  malatya:   { lat: 38.3554, lng: 38.3335 },
  van:       { lat: 38.5012, lng: 43.3730 },
  denizli:   { lat: 37.7765, lng: 29.0864 },
  mersin:    { lat: 36.8121, lng: 34.6415 },
  manisa:    { lat: 38.6191, lng: 27.4289 },
  balikesir: { lat: 39.6484, lng: 27.8826 },
};

function haversineDistance(lat1, lng1, lat2, lng2) {
  const R = 6371; // km
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLng = ((lng2 - lng1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) ** 2 +
    Math.cos((lat1 * Math.PI) / 180) *
    Math.cos((lat2 * Math.PI) / 180) *
    Math.sin(dLng / 2) ** 2;
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}

/**
 * Detect user's city SILENTLY using backend IP detection.
 * Does NOT prompt for permission.
 */
export async function detectCitySilent() {
  try {
    const city = await api.detectCity();
    return city && city.city_slug ? city.city_slug : null;
  } catch (err) {
    console.error('[Geo] Silent detection failed:', err);
    return null;
  }
}

export async function detectCityIP() {
  try {
    const city = await api.detectCity();
    if (city && city.source === 'cloudflare' && city.city_slug) {
      return city.city_slug;
    }
    return null;
  } catch (err) {
    return null;
  }
}

/**
 * Find the nearest city slug from user's geolocation (GPS).
 * Prompts user for permission.
 * @param {string[]} availableSlugs - List of city slugs that have data
 * @returns {Promise<string|null>} nearest city slug or null
 */
export function detectCityPrecise(availableSlugs) {
  return new Promise((resolve) => {
    if (!navigator.geolocation) {
      resolve(null);
      return;
    }

    navigator.geolocation.getCurrentPosition(
      (pos) => {
        const { latitude, longitude } = pos.coords;
        let nearest = null;
        let minDist = Infinity;

        for (const slug of availableSlugs) {
          const coords = CITY_COORDS[slug];
          if (!coords) continue;
          const dist = haversineDistance(latitude, longitude, coords.lat, coords.lng);
          if (dist < minDist) {
            minDist = dist;
            nearest = slug;
          }
        }

        // If no match in CITY_COORDS, or distance > 200km, fall back
        if (!nearest || minDist > 200) {
          resolve(availableSlugs[0] || null);
        } else {
          resolve(nearest);
        }
      },
      () => resolve(null), // permission denied or error
      { timeout: 5000 }
    );
  });
}

// Deprecated: Alias for backward compatibility if needed during migration
export const detectCity = detectCityPrecise;
