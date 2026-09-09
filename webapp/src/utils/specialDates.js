import specialDatesData from "$lib/data/specialDates.json";

/**
 * Gelen tarihi yerel saat dilimi sapmalarından etkilenmeden yıl, ay, gün ve MM-DD formatına ayırır.
 *
 * @param {Date|string} date
 * @returns {{ mmdd: string, year: number, month: number, day: number }}
 */
export function parseDateKey(date) {
    if (typeof date === "string") {
        const parts = date.split("T")[0].split("-").map(Number);
        if (parts.length === 3 && !parts.some(isNaN)) {
            const mm = String(parts[1]).padStart(2, "0");
            const dd = String(parts[2]).padStart(2, "0");
            return { mmdd: `${mm}-${dd}`, year: parts[0], month: parts[1], day: parts[2] };
        }
    }
    const dt = date instanceof Date && !isNaN(date) ? date : new Date();
    const mm = String(dt.getMonth() + 1).padStart(2, "0");
    const dd = String(dt.getDate()).padStart(2, "0");
    return {
        mmdd: `${mm}-${dd}`,
        year: dt.getFullYear(),
        month: dt.getMonth() + 1,
        day: dt.getDate()
    };
}

/**
 * Verilen tarihin 10 Kasım Atatürk'ü Anma Günü (matem/saygı günü) olup olmadığını döner.
 *
 * @param {Date|string} date
 * @returns {boolean}
 */
export function isMourningDay(date) {
    const { mmdd } = parseDateKey(date);
    return mmdd === "11-10";
}

/**
 * Verilen tarihin her ayın 6 ile 10'u arasındaki KYK Burs/Kredi ödeme dönemi olup olmadığını döner.
 *
 * @param {Date|string} date
 * @returns {boolean}
 */
export function isScholarshipPeriod(date) {
    const { day } = parseDateKey(date);
    return day >= 6 && day <= 10;
}

/**
 * Seçili tarih ve şehir için varsa yerel kurtuluş, anma veya milli gün bilgisini döner.
 *
 * @param {Date|string} date
 * @param {string} [citySlug]
 * @returns {{ name: string, isMourning?: boolean, isCelebration?: boolean, isScholarship?: boolean, isLocal?: boolean } | null}
 */
export function getSpecialDayInfo(date, citySlug = "") {
    const { mmdd } = parseDateKey(date);

    // 1. Şehre özel yerel kurtuluş / anma günü önceliği
    if (citySlug && specialDatesData.cities[citySlug]) {
        const localEvent = specialDatesData.cities[citySlug][mmdd];
        if (localEvent) {
            return {
                name: localEvent,
                isLocal: true
            };
        }
    }

    // 2. Milli ve resmi günler
    const nationalEvent = specialDatesData.national[mmdd];
    if (nationalEvent) {
        return {
            name: nationalEvent.name,
            isMourning: Boolean(nationalEvent.isMourning),
            isCelebration: Boolean(nationalEvent.isCelebration),
            isLocal: false
        };
    }

    // 3. Öğrenci döngüsü: Her ayın 6-10'u burs yatış dönemi
    if (isScholarshipPeriod(date)) {
        return {
            name: "KYK Burs/Kredi Yatış Dönemi",
            isScholarship: true,
            isLocal: false
        };
    }

    return null;
}
