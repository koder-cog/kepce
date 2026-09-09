import { istanbulToday } from '@/lib/server/api.js';
import { getSpecialDayInfo, isMourningDay } from '@/utils/specialDates.js';

/**
 * Sunucu tarafında Türkiye saat dilimi (Europe/Istanbul - UTC+3) ile bugünün tarihini
 * ve ortam bağlamını (matem, özel gün) sağlar. İstemci cihaz saati sapmalarına
 * karşı güvenilir referans noktasıdır.
 *
 * @type {import('./$types').LayoutServerLoad}
 */
export async function load() {
    const serverToday = istanbulToday();
    const isMourning = isMourningDay(serverToday);
    const specialDay = getSpecialDayInfo(serverToday);

    return {
        serverToday,
        isMourning,
        specialDay
    };
}
