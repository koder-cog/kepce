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


