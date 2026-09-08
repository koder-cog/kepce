/**
 * Calendar component for monthly view.
 */

const DAYS_TR = ['Pzt', 'Sal', 'Çar', 'Per', 'Cum', 'Cmt', 'Paz'];
const MONTHS_TR = [
  'Ocak', 'Şubat', 'Mart', 'Nisan', 'Mayıs', 'Haziran',
  'Temmuz', 'Ağustos', 'Eylül', 'Ekim', 'Kasım', 'Aralık',
];

export function getMonthName(month) {
  return MONTHS_TR[month - 1] || '';
}

/**
 * Gönderilen tarihin şimdiki zamana göre ne kadar süre önce olduğunu Türkçe formatında döndürür.
 * @param {string|number|Date} dateStr - Tarih verisi
 * @returns {string} Süre açıklaması (örn: "3 dakika önce", "dün", vb.)
 */
export function timeAgo(dateStr) {
  if (!dateStr) return '';
  let date;
  if (typeof dateStr === 'number') {
    date = new Date(dateStr);
  } else if (typeof dateStr === 'string') {
    date = new Date(dateStr.endsWith('Z') || dateStr.includes('+') ? dateStr : dateStr + 'Z');
  } else if (dateStr instanceof Date) {
    date = dateStr;
  } else {
    return '';
  }
  
  if (isNaN(date.getTime())) return '';

  const now = new Date();
  const seconds = Math.floor((now - date) / 1000);

  if (seconds < 60) return 'şimdi';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} dakika önce`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} saat önce`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days} gün önce`;
  const weeks = Math.floor(days / 7);
  if (weeks < 4) return `${weeks} hafta önce`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${Math.max(1, months)} ay önce`;

  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}.${m}.${d}`;
}

const TURKISH_MONTHS_MAP = {
  ocak: 1, subat: 2, şubat: 2, mart: 3, nisan: 4, mayis: 5, mayıs: 5,
  haziran: 6, temmuz: 7, agustos: 8, ağustos: 8, eylul: 9, eylül: 9,
  ekim: 10, kasim: 11, kasım: 11, aralik: 12, aralık: 12
};

function isValidYmd(year, month, day) {
  if (year < 2000 || year > 2100) return false;
  if (month < 1 || month > 12) return false;
  if (day < 1 || day > 31) return false;
  const d = new Date(Date.UTC(year, month - 1, day));
  return (
    d.getUTCFullYear() === year &&
    d.getUTCMonth() === month - 1 &&
    d.getUTCDate() === day
  );
}

function formatYmd(year, month, day) {
  return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}

/**
 * Kullanıcı arama sorgusundaki tarihi ayıklar.
 * Desteklenen kalıplar:
 * - 30.06.2026, 30/06/2026, 30-06-2026, 30.06, 30/6/26
 * - 30 haziran 2026, 30 haziran, 30 haziran'da, 30 hazirandaki
 * - 2026-06-30 (ISO)
 * - dün, dünkü, yarın, yarınki, bugün, bugünkü
 * 
 * @param {string} query Kullanıcı sorgusu
 * @param {string} referenceDateStr Varsayılan YYYY-MM-DD tarihi
 * @returns {string} Ayıklanan YYYY-MM-DD tarihi veya referenceDateStr
 */
export function extractQueryDate(query, referenceDateStr) {
  if (!query || typeof query !== 'string') return referenceDateStr;
  const q = query.toLowerCase().trim();
  const refYear = referenceDateStr ? parseInt(referenceDateStr.slice(0, 4), 10) : new Date().getFullYear();

  // Bağıl günler (dün, bugün, yarın)
  if (/(?:^|[^\p{L}\p{N}])(?:dün|dünkü)(?=[^\p{L}\p{N}]|$)/iu.test(q)) {
    const d = new Date((referenceDateStr || new Date().toISOString().split('T')[0]) + 'T12:00:00Z');
    d.setUTCDate(d.getUTCDate() - 1);
    return d.toISOString().split('T')[0];
  }
  if (/(?:^|[^\p{L}\p{N}])(?:yarın|yarinki|yarınki)(?=[^\p{L}\p{N}]|$)/iu.test(q)) {
    const d = new Date((referenceDateStr || new Date().toISOString().split('T')[0]) + 'T12:00:00Z');
    d.setUTCDate(d.getUTCDate() + 1);
    return d.toISOString().split('T')[0];
  }
  if (/(?:^|[^\p{L}\p{N}])(?:bugün|bugünkü)(?=[^\p{L}\p{N}]|$)/iu.test(q)) {
    return referenceDateStr;
  }

  // ISO formatı (YYYY-MM-DD)
  const isoMatch = q.match(/\b(20\d{2})-(0?[1-9]|1[0-2])-(0?[1-9]|[12]\d|3[01])\b/);
  if (isoMatch) {
    const y = parseInt(isoMatch[1], 10);
    const m = parseInt(isoMatch[2], 10);
    const d = parseInt(isoMatch[3], 10);
    if (isValidYmd(y, m, d)) return formatYmd(y, m, d);
  }

  // Doğal Türkçe ay ve gün kalıpları (örn. "30 Haziran", "30 Haziran'daki")
  const monthRegex = /(?:^|[^\p{L}\p{N}])(\d{1,2})\s*(?:nci|ncı|inci|ıncı|\.)?\s*(ocak|şubat|subat|mart|nisan|mayıs|mayis|haziran|temmuz|ağustos|agustos|eylül|eylul|ekim|kasım|kasim|aralık|aralik)(?:['’]?(?:daki|deki|taki|teki|da|de|ta|te|nın|nin|nun|nün|ın|in|un|ün|e|a))?(?:\s+(20\d{2}|\d{2}))?(?=[^\p{L}\p{N}]|$)/iu;
  const textMonthMatch = q.match(monthRegex);
  if (textMonthMatch) {
    const day = parseInt(textMonthMatch[1], 10);
    const month = TURKISH_MONTHS_MAP[textMonthMatch[2].toLowerCase()];
    let year = refYear;
    if (textMonthMatch[3]) {
      const rawYear = parseInt(textMonthMatch[3], 10);
      year = rawYear < 100 ? 2000 + rawYear : rawYear;
    }
    if (isValidYmd(year, month, day)) {
      return formatYmd(year, month, day);
    }
  }

  // Sayısal ayraçlı formatlar (DD.MM.YYYY, DD/MM/YYYY)
  const numMatch = q.match(/\b(\d{1,2})[./-](\d{1,2})(?:[./-](\d{2,4}))?\b/);
  if (numMatch) {
    const day = parseInt(numMatch[1], 10);
    const month = parseInt(numMatch[2], 10);
    let year = refYear;
    if (numMatch[3]) {
      const rawYear = parseInt(numMatch[3], 10);
      year = rawYear < 100 ? 2000 + rawYear : rawYear;
    }
    if (isValidYmd(year, month, day)) {
      return formatYmd(year, month, day);
    }
  }

  return referenceDateStr;
}


