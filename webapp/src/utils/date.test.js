import { describe, it, expect } from 'vitest';
import { getMonthName, timeAgo, extractQueryDate } from './date.js';

describe('getMonthName', () => {
  it('should return correct Turkish month names', () => {
    expect(getMonthName(1)).toBe('Ocak');
    expect(getMonthName(7)).toBe('Temmuz');
    expect(getMonthName(12)).toBe('Aralık');
    expect(getMonthName(13)).toBe('');
  });
});

describe('timeAgo', () => {
  it('should format relative times in Turkish', () => {
    const now = new Date();
    
    // Now
    expect(timeAgo(now)).toBe('şimdi');

    // Minutes ago
    const fiveMinutesAgo = new Date(now.getTime() - 5 * 60 * 1000);
    expect(timeAgo(fiveMinutesAgo)).toBe('5 dakika önce');

    // Hours ago
    const threeHoursAgo = new Date(now.getTime() - 3 * 60 * 60 * 1000);
    expect(timeAgo(threeHoursAgo)).toBe('3 saat önce');

    // Days ago
    const twoDaysAgo = new Date(now.getTime() - 2 * 24 * 60 * 60 * 1000);
    expect(timeAgo(twoDaysAgo)).toBe('2 gün önce');

    // Handle invalid inputs
    expect(timeAgo(null)).toBe('');
    expect(timeAgo('invalid-date')).toBe('');
  });
});

describe('extractQueryDate', () => {
  const refDate = '2026-09-07';

  it('ayraçlı sayısal formatları doğru ayıklar (30.06.2026, 30/06/2026, 30-06-2026)', () => {
    expect(extractQueryDate('30.06.2026 istanbul kyk yemek', refDate)).toBe('2026-06-30');
    expect(extractQueryDate('istanbul kyk yemek 30/06/2026', refDate)).toBe('2026-06-30');
    expect(extractQueryDate('30-06-2026 ankara menüsü', refDate)).toBe('2026-06-30');
    expect(extractQueryDate('30.6.2026 izmir', refDate)).toBe('2026-06-30');
    expect(extractQueryDate('30.06.26 bursa kyk', refDate)).toBe('2026-06-30');
    expect(extractQueryDate('30.06 antalya kyk', refDate)).toBe('2026-06-30');
  });

  it('Türkçe ay isimlerini ve çekim eklerini doğru ayıklar', () => {
    expect(extractQueryDate('30 haziran 2026 istanbul kyk yemek', refDate)).toBe('2026-06-30');
    expect(extractQueryDate('30 haziran kyk yemek', refDate)).toBe('2026-06-30');
    expect(extractQueryDate("30 haziran'da istanbul yemek", refDate)).toBe('2026-06-30');
    expect(extractQueryDate('30 hazirandaki tabldot listesi', refDate)).toBe('2026-06-30');
    expect(extractQueryDate('15 subat 2026 ankara', refDate)).toBe('2026-02-15');
    expect(extractQueryDate('1 ocak konya menü', refDate)).toBe('2026-01-01');
  });

  it('ISO formatını doğru ayıklar', () => {
    expect(extractQueryDate('2026-06-30 istanbul kyk', refDate)).toBe('2026-06-30');
  });

  it('bağıl günleri (dün, yarın, bugün) referans tarihe göre hesaplar', () => {
    expect(extractQueryDate('dün istanbul kyk yemek', refDate)).toBe('2026-09-06');
    expect(extractQueryDate('dünkü menü ankara', refDate)).toBe('2026-09-06');
    expect(extractQueryDate('yarın izmir yemekhane', refDate)).toBe('2026-09-08');
    expect(extractQueryDate('yarınki tabldot', refDate)).toBe('2026-09-08');
    expect(extractQueryDate('bugün kyk menüsü', refDate)).toBe('2026-09-07');
    expect(extractQueryDate('bugünkü yemekler', refDate)).toBe('2026-09-07');
  });

  it('geçersiz tarihlerde veya tarih bulunmadığında referans tarihi korur', () => {
    expect(extractQueryDate('istanbul kyk yemek', refDate)).toBe(refDate);
    expect(extractQueryDate('31 şubat 2026 menü', refDate)).toBe(refDate);
    expect(extractQueryDate('99.99.2026 istanbul', refDate)).toBe(refDate);
    expect(extractQueryDate('', refDate)).toBe(refDate);
    expect(extractQueryDate(null, refDate)).toBe(refDate);
  });
});
