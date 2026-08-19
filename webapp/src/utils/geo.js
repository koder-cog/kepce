/**
 * Geolocation utility — detect user's city from coordinates or IP.
 */
import { api } from '../api/index.js';

// All 81 Turkish provinces with their approximate center coordinates
const TURKEY_PROVINCE_COORDS = {
  adana:          { lat: 37.0000, lng: 35.3213 },
  adiyaman:       { lat: 37.7648, lng: 38.2786 },
  afyonkarahisar: { lat: 38.7507, lng: 30.5567 },
  agri:           { lat: 39.7217, lng: 43.0567 },
  amasya:         { lat: 40.6534, lng: 35.8331 },
  ankara:         { lat: 39.9334, lng: 32.8597 },
  antalya:        { lat: 36.8969, lng: 30.7133 },
  artvin:         { lat: 41.1828, lng: 41.8183 },
  aydin:          { lat: 37.8560, lng: 27.8416 },
  balikesir:      { lat: 39.6484, lng: 27.8826 },
  bilecik:        { lat: 40.1451, lng: 29.9799 },
  bingol:         { lat: 38.8854, lng: 40.4983 },
  bitlis:         { lat: 38.4006, lng: 42.1095 },
  bolu:           { lat: 40.7358, lng: 31.6061 },
  burdur:         { lat: 37.7203, lng: 30.2908 },
  bursa:          { lat: 40.1826, lng: 29.0665 },
  canakkale:      { lat: 40.1553, lng: 26.4142 },
  cankiri:        { lat: 40.6013, lng: 33.6134 },
  corum:          { lat: 40.5506, lng: 34.9556 },
  denizli:        { lat: 37.7765, lng: 29.0864 },
  diyarbakir:     { lat: 37.9144, lng: 40.2306 },
  edirne:         { lat: 41.6772, lng: 26.5557 },
  elazig:         { lat: 38.6810, lng: 39.2264 },
  erzincan:       { lat: 39.7500, lng: 39.5000 },
  erzurum:        { lat: 39.9055, lng: 41.2658 },
  eskisehir:      { lat: 39.7767, lng: 30.5206 },
  gaziantep:      { lat: 37.0662, lng: 37.3833 },
  giresun:        { lat: 40.9128, lng: 38.3895 },
  gumushane:      { lat: 40.4600, lng: 39.4700 },
  hakkari:        { lat: 37.5833, lng: 43.7333 },
  hatay:          { lat: 36.2023, lng: 36.1613 },
  isparta:        { lat: 37.7648, lng: 30.5566 },
  mersin:         { lat: 36.8121, lng: 34.6415 },
  istanbul:       { lat: 41.0082, lng: 28.9784 },
  izmir:          { lat: 38.4237, lng: 27.1428 },
  kars:           { lat: 40.6013, lng: 43.0975 },
  kastamonu:      { lat: 41.3887, lng: 33.7827 },
  kayseri:        { lat: 38.7312, lng: 35.4787 },
  kirklareli:     { lat: 41.7333, lng: 27.2167 },
  kirsehir:       { lat: 39.1425, lng: 34.1709 },
  kocaeli:        { lat: 40.8533, lng: 29.8815 },
  konya:          { lat: 37.8715, lng: 32.4846 },
  kutahya:        { lat: 39.4167, lng: 29.9833 },
  malatya:        { lat: 38.3554, lng: 38.3335 },
  manisa:         { lat: 38.6191, lng: 27.4289 },
  kahramanmaras:  { lat: 37.5858, lng: 36.9371 },
  mardin:         { lat: 37.3212, lng: 40.7245 },
  mugla:          { lat: 37.2153, lng: 28.3636 },
  mus:            { lat: 38.7432, lng: 41.5064 },
  nevsehir:       { lat: 38.6244, lng: 34.7144 },
  nigde:          { lat: 37.9667, lng: 34.6833 },
  ordu:           { lat: 40.9839, lng: 37.8764 },
  rize:           { lat: 41.0201, lng: 40.5234 },
  sakarya:        { lat: 40.7569, lng: 30.3783 },
  samsun:         { lat: 41.2867, lng: 36.3300 },
  siirt:          { lat: 37.9333, lng: 41.9500 },
  sinop:          { lat: 42.0231, lng: 35.1531 },
  sivas:          { lat: 39.7477, lng: 37.0179 },
  tekirdag:       { lat: 40.9833, lng: 27.5167 },
  tokat:          { lat: 40.3167, lng: 36.5500 },
  trabzon:        { lat: 41.0027, lng: 39.7168 },
  tunceli:        { lat: 39.1079, lng: 39.5401 },
  sanliurfa:      { lat: 37.1674, lng: 38.7955 },
  usak:           { lat: 38.6823, lng: 29.4082 },
  van:            { lat: 38.5012, lng: 43.3730 },
  yozgat:         { lat: 39.8181, lng: 34.8147 },
  zonguldak:      { lat: 41.4564, lng: 31.7987 },
  aksaray:        { lat: 38.3687, lng: 34.0370 },
  bayburt:        { lat: 40.2552, lng: 40.2249 },
  karaman:        { lat: 37.1759, lng: 33.2287 },
  kirikkale:      { lat: 39.8468, lng: 33.5153 },
  batman:         { lat: 37.8812, lng: 41.1294 },
  sirnak:         { lat: 37.5164, lng: 42.4593 },
  bartin:         { lat: 41.6344, lng: 32.3375 },
  ardahan:        { lat: 41.1105, lng: 42.7022 },
  igdir:          { lat: 39.9196, lng: 44.0450 },
  yalova:         { lat: 40.6500, lng: 29.2667 },
  karabuk:        { lat: 41.2061, lng: 32.6204 },
  kilis:          { lat: 36.7184, lng: 37.1212 },
  osmaniye:       { lat: 37.0742, lng: 36.2478 },
  duzce:          { lat: 40.8438, lng: 31.1565 },
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
 * Find user's city from GPS coordinates and match against active city slugs.
 * Prompts user for permission.
 * @param {string[]} availableSlugs - List of city slugs that have active menu data
 * @returns {Promise<{success: boolean, slug?: string, unsupported?: boolean}|null>}
 */
export function detectCityPrecise(availableSlugs) {
  return new Promise((resolve) => {
    if (typeof navigator === 'undefined' || !navigator.geolocation) {
      resolve(null);
      return;
    }

    navigator.geolocation.getCurrentPosition(
      (pos) => {
        const { latitude, longitude } = pos.coords;
        let nearestSlug = null;
        let minDist = Infinity;

        // 1. Aşama: 81 il arasından en yakın gerçek Türkiye ilini bul
        for (const [slug, coords] of Object.entries(TURKEY_PROVINCE_COORDS)) {
          const dist = haversineDistance(latitude, longitude, coords.lat, coords.lng);
          if (dist < minDist) {
            minDist = dist;
            nearestSlug = slug;
          }
        }

        // Türkiye sınırları dışındaysa (en yakın il 150 km'den uzaksa) veya bulunamadıysa
        if (!nearestSlug || minDist > 150) {
          resolve(null);
          return;
        }

        // 2. Aşama: Tespit edilen il sistemde (menüsü olan iller arasında) var mı kontrol et
        const isSupported = Array.isArray(availableSlugs) && availableSlugs.includes(nearestSlug);

        if (isSupported) {
          resolve({ success: true, slug: nearestSlug });
        } else {
          resolve({ success: false, unsupported: true, slug: nearestSlug });
        }
      },
      () => resolve(null), // permission denied or error
      { timeout: 8000, enableHighAccuracy: false }
    );
  });
}

// Deprecated: Alias for backward compatibility if needed
export const detectCity = detectCityPrecise;
