/**
 * KYK Yurt Sezon Takvimi
 * 
 * - Temmuz ve Ağustos ayları istisnasız tüm yıllarda YAZ / NÖBETÇİ sezonudur (kahvaltı yoktur, sadece akşam yemeği).
 * - Eylül ayındaki sezon açılış tarihleri GSB KYK takvimine göre Eylül ayının 2. veya 3. Pazartesi günü başlar.
 * - Açılış gününe kadar nöbetçi yurt düzeni devam eder (yalnızca akşam yemeği verilir).
 */

export const OFF_SEASON_CALENDAR = {
  2024: { start: '07-01', open: '09-16' }, // 16 Eylül 2024 Pazartesi
  2025: { start: '07-01', open: '09-15' }, // 15 Eylül 2025 Pazartesi
  2026: { start: '07-01', open: '09-14' }, // 14 Eylül 2026 Pazartesi
  2027: { start: '07-01', open: '09-13' }, // 13 Eylül 2027 Pazartesi
  2028: { start: '07-01', open: '09-11' }, // 11 Eylül 2028 Pazartesi
  2029: { start: '07-01', open: '09-17' }, // 17 Eylül 2029 Pazartesi
  2030: { start: '07-01', open: '09-16' }, // 16 Eylül 2030 Pazartesi
};

/**
 * Verilen yıl için 15 Eylül'e en yakın Pazartesi gününü (GSB KYK standart açılış tarihi) hesaplar.
 * @param {number} year
 * @returns {number} Eylül ayındaki gün (örn: 13, 14, 15, 16, 17, 18)
 */
export function getClosestMondayToSept15(year) {
  const sept15 = new Date(year, 8, 15);
  const dayOfWeek = sept15.getDay(); // 0: Pazar, 1: Pazartesi, ..., 6: Cumartesi

  // 15 Eylül'e en yakın Pazartesi (1) gününe olan mesafe
  let diff = 0;
  if (dayOfWeek === 1) diff = 0;        // Pazartesi -> 15
  else if (dayOfWeek === 2) diff = -1;  // Salı -> 14
  else if (dayOfWeek === 3) diff = -2;  // Çarşamba -> 13
  else if (dayOfWeek === 4) diff = -3;  // Perşembe -> 12
  else if (dayOfWeek === 5) diff = -4;  // Cuma -> 11
  else if (dayOfWeek === 6) diff = 2;   // Cumartesi -> 17
  else if (dayOfWeek === 0) diff = 1;   // Pazar -> 16

  return 15 + diff;
}

/**
 * Verilen tarihin sezon dışı (nöbetçi yurt dönemi) olup olmadığını döner.
 * @param {Date|string} dateInput - Kontrol edilecek tarih
 * @returns {boolean}
 */
export function isOffSeasonDate(dateInput) {
  if (!dateInput) return false;
  const d = dateInput instanceof Date ? dateInput : new Date(dateInput);
  if (isNaN(d.getTime())) return false;

  const year = d.getFullYear();
  const month = d.getMonth() + 1; // 1-12
  const day = d.getDate();

  // Temmuz (7) ve Ağustos (8) her yıl garanti sezon dışıdır
  if (month === 7 || month === 8) {
    return true;
  }

  // Eylül (9) ayı için takvim kontrolü
  if (month === 9) {
    const config = OFF_SEASON_CALENDAR[year];
    if (config && config.open) {
      const [openMonth, openDay] = config.open.split('-').map(Number);
      if (openMonth === 9) {
        return day < openDay;
      }
    }
    // Konfigürasyonda yoksa 15 Eylül'e en yakın Pazartesi esas alınır
    return day < getClosestMondayToSept15(year);
  }

  return false;
}

/**
 * Oryantasyon rehber kartlarının (Beslenme Yardımı, Yemekhane Saatleri)
 * gösterileceği dönem: 1 Temmuz – 31 Ekim (Temmuz, Ağustos, Eylül, Ekim).
 * @param {Date|string} dateInput - Kontrol edilecek tarih
 * @returns {boolean}
 */
export function isOrientationSeason(dateInput) {
  if (!dateInput) return false;
  const d = dateInput instanceof Date ? dateInput : new Date(dateInput);
  if (isNaN(d.getTime())) return false;

  const month = d.getMonth() + 1;
  return [7, 8, 9, 10].includes(month);
}
