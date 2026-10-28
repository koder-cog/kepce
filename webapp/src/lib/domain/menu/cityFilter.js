/**
 * Ay görünümüne göre şehir listesini dinamik filtreler.
 *
 * - Güncel ay veya gelecek (monthDiff <= 0): Yalnızca bu ay menüsü bulunan şehirler.
 * - Son 3 ay çemberi (1 <= monthDiff <= 3): Son 90 gün içinde menüsü bulunan şehirler.
 * - Arşiv veya eski aylar (monthDiff > 3): Tüm şehirler.
 *
 * Güvenlik koruması:
 * 1. Aktif seçili şehir (currentCity) hiçbir zaman listeden düşürülmez.
 * 2. Henüz latest_menu_date yüklenmemişse (ilk statik render veya çevrimdışı) tüm liste korunur.
 *
 * @param {Object} params
 * @param {Array<Object>} params.cities - Şehir nesneleri listesi (slug, name, latest_menu_date vb.)
 * @param {string} [params.currentCity] - Halihazırda seçili şehir slug'ı
 * @param {number} params.viewYear - Zaman çizelgesinde görüntülenen yıl
 * @param {number} params.viewMonth - Zaman çizelgesinde görüntülenen ay (0-11)
 * @param {Date} [params.today=new Date()] - Hesaplamada referans alınacak güncel tarih
 * @returns {Array<Object>} Filtrelenmiş şehir listesi
 */
export function filterCitiesByViewPeriod({
  cities = [],
  currentCity,
  viewYear,
  viewMonth,
  today = new Date()
}) {
  if (!Array.isArray(cities) || cities.length === 0) {
    return [];
  }

  const hasAnyDate = cities.some((c) => Boolean(c.latest_menu_date));
  if (!hasAnyDate) {
    return cities;
  }

  const todayYear = today.getFullYear();
  const todayMonth = today.getMonth();
  const monthDiff = (todayYear - viewYear) * 12 + (todayMonth - viewMonth);

  if (monthDiff > 3) {
    return cities;
  }

  const currentYearMonth = `${todayYear}-${String(todayMonth + 1).padStart(2, '0')}`;
  const todayTime = today.getTime();
  const MS_PER_DAY = 1000 * 60 * 60 * 24;

  let filtered = [];

  if (monthDiff <= 0) {
    filtered = cities.filter((c) => {
      if (!c.latest_menu_date) return false;
      return c.latest_menu_date.slice(0, 7) === currentYearMonth;
    });
  } else {
    filtered = cities.filter((c) => {
      if (!c.latest_menu_date) return false;
      const parts = c.latest_menu_date.slice(0, 10).split('-').map(Number);
      if (parts.length !== 3 || parts.some(isNaN)) return false;
      const menuDate = new Date(parts[0], parts[1] - 1, parts[2]);
      const diffDays = Math.floor((todayTime - menuDate.getTime()) / MS_PER_DAY);
      return diffDays <= 90;
    });
  }

  if (currentCity && !filtered.some((c) => c.slug === currentCity)) {
    const activeCityObj = cities.find((c) => c.slug === currentCity);
    if (activeCityObj) {
      filtered = [activeCityObj, ...filtered];
    }
  }

  return filtered;
}
