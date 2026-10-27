import { istanbulToday } from '$lib/server/api.js';
import { getSpecialDayInfo } from '$lib/domain/calendar/specialDates.js';
import { dev } from '$app/environment';

/**
 * Sunucu tarafında Türkiye saat dilimi (Europe/Istanbul - UTC+3) ile bugünün tarihini
 * ve ortam bağlamını (özel gün) sağlar. İstemci cihaz saati sapmalarına
 * karşı güvenilir referans noktasıdır.
 *
 * @type {import('./$types').LayoutServerLoad}
 */
export async function load({ url }) {
  const testToday = dev ? url.searchParams.get('test_today') : null;
  const serverToday = testToday || istanbulToday();
  const specialDay = getSpecialDayInfo(serverToday);

  return {
    serverToday,
    specialDay
  };
}
