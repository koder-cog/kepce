import { parseDateKey } from './specialDates.js';
import holidaysData from './holidays.json';

/**
 * İki UTC tarihi arasındaki gün farkını döner (dst/saat dilimi sapmalarından arındırılmış).
 *
 * @param {number} y1
 * @param {number} m1
 * @param {number} d1
 * @param {number} y2
 * @param {number} m2
 * @param {number} d2
 * @returns {number}
 */
function getDayDiff(y1, m1, d1, y2, m2, d2) {
  const dA = Date.UTC(y1, m1 - 1, d1);
  const dB = Date.UTC(y2, m2 - 1, d2);
  return Math.round((dA - dB) / 86400000);
}

/**
 * Verilen tarihin bayram, anma veya özel kutlama kartı bilgisini döner.
 *
 * @param {Date|string} date
 * @returns {{ name: string, tone: 'national' | 'religious' | 'solemn', text: string } | null}
 */
export function getHolidayInfo(date) {
  const { mmdd, year, month, day } = parseDateKey(date);
  const yyyymmdd = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;

  // 1. Manuel İstisna ve Çifte Bayram Önceliği (Overrides)
  const override = holidaysData.overrides?.[yyyymmdd];
  if (override) {
    return {
      name: override.name,
      tone: override.tone,
      text: override.message
    };
  }

  // 2. Sabit Güneş Takvimi Günleri (Milli Bayramlar ve Anmalar)
  const fixedHoliday = holidaysData.fixed?.[mmdd];
  if (fixedHoliday) {
    const list = fixedHoliday.messages || [];
    let text = list.length > 0 ? list[year % list.length] : (fixedHoliday.message || fixedHoliday.name);
    const republicYear = year - 1923;
    text = text.replace('{republicYear}', String(republicYear));

    return {
      name: fixedHoliday.name,
      tone: fixedHoliday.tone,
      text
    };
  }

  // 3. Dini Bayramlar (Öncelik: schedule tablosu)
  const yearSchedule = holidaysData.schedule?.[String(year)];

  if (yearSchedule?.ramadan) {
    const [rM, rD] = yearSchedule.ramadan.split('-').map(Number);
    const diff = getDayDiff(year, month, day, year, rM, rD);
    if (diff >= 0 && diff <= 2) {
      const dayKey = `day_${diff + 1}`;
      const list = holidaysData.messages?.ramadan?.[dayKey] || [];
      const text = list.length > 0 ? list[year % list.length] : 'Ramazan Bayramı kutlu olsun!';
      return {
        name: 'Ramazan Bayramı',
        tone: 'religious',
        text
      };
    }
  }

  if (yearSchedule?.sacrifice) {
    const [sM, sD] = yearSchedule.sacrifice.split('-').map(Number);
    const diff = getDayDiff(year, month, day, year, sM, sD);
    if (diff >= 0 && diff <= 3) {
      const dayKey = `day_${diff + 1}`;
      const list = holidaysData.messages?.sacrifice?.[dayKey] || [];
      const text = list.length > 0 ? list[year % list.length] : 'Kurban Bayramı kutlu olsun!';
      return {
        name: 'Kurban Bayramı',
        tone: 'religious',
        text
      };
    }
  }

  // 4. Gelecek Yıllar İçin Otomatik Fallback (2031+ ve tanımlanmamış yıllar)
  if (!yearSchedule) {
    try {
      const dt = new Date(Date.UTC(year, month - 1, day));
      const fmt = new Intl.DateTimeFormat('en-u-ca-islamic-umalqura', { month: 'numeric', day: 'numeric' });
      const parts = fmt.formatToParts(dt);
      const hMonth = parseInt(parts.find((p) => p.type === 'month')?.value || '0', 10);
      const hDay = parseInt(parts.find((p) => p.type === 'day')?.value || '0', 10);

      // Ramazan Bayramı: 10. ay (Şevval), 1-3. günler
      if (hMonth === 10 && hDay >= 1 && hDay <= 3) {
        const dayKey = `day_${hDay}`;
        const list = holidaysData.messages?.ramadan?.[dayKey] || [];
        const text = list.length > 0 ? list[year % list.length] : 'Ramazan Bayramı kutlu olsun!';
        return {
          name: 'Ramazan Bayramı',
          tone: 'religious',
          text
        };
      }

      // Kurban Bayramı: 12. ay (Zilhicce), 10-13. günler
      if (hMonth === 12 && hDay >= 10 && hDay <= 13) {
        const dayIndex = hDay - 9;
        const dayKey = `day_${dayIndex}`;
        const list = holidaysData.messages?.sacrifice?.[dayKey] || [];
        const text = list.length > 0 ? list[year % list.length] : 'Kurban Bayramı kutlu olsun!';
        return {
          name: 'Kurban Bayramı',
          tone: 'religious',
          text
        };
      }
    } catch {
      // Intl desteklenmiyorsa sessizce geç
    }
  }

  return null;
}
