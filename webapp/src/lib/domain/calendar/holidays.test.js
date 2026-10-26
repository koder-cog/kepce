import { describe, it, expect } from 'vitest';
import { getHolidayInfo } from './holidays.js';

describe('holidays Resolver', () => {
  it('29 Ekim için Cumhuriyet yılı dinamik hesaplanmalı ve national ton dönmeli', () => {
    const h2026 = getHolidayInfo('2026-10-29');
    expect(h2026).not.toBeNull();
    expect(h2026?.tone).toBe('national');
    expect(h2026?.name).toBe('Cumhuriyet Bayramı');
    expect(h2026?.text).toContain('103. yılı');

    const h2030 = getHolidayInfo('2030-10-29');
    expect(h2030?.text).toContain('107. yılı');
  });

  it('10 Kasım için solemn tonunda Atatürk anma mesajı dönmeli', () => {
    const h = getHolidayInfo('2026-11-10');
    expect(h).not.toBeNull();
    expect(h?.tone).toBe('solemn');
    expect(h?.name).toBe("Atatürk'ü Anma Günü");
    expect(h?.text).toContain('09:05');
  });

  it('2026 Ramazan Bayramı günlerini (1-3) tanımalı ve religious ton dönmeli', () => {
    const d1 = getHolidayInfo('2026-03-20');
    expect(d1?.tone).toBe('religious');
    expect(d1?.name).toBe('Ramazan Bayramı');
    expect(d1?.text).toBeTruthy();

    const d2 = getHolidayInfo('2026-03-21');
    expect(d2?.tone).toBe('religious');
    expect(d2?.text).toBeTruthy();

    const d3 = getHolidayInfo('2026-03-22');
    expect(d3?.tone).toBe('religious');
    expect(d3?.text).toBeTruthy();

    // Bayram sonrası sıradan gün
    const dAfter = getHolidayInfo('2026-03-23');
    expect(dAfter).toBeNull();
  });

  it('2026 Kurban Bayramı günlerini (1-4) tanımalı', () => {
    const k1 = getHolidayInfo('2026-05-27');
    expect(k1?.tone).toBe('religious');
    expect(k1?.name).toBe('Kurban Bayramı');
    expect(k1?.text).toBeTruthy();

    const k4 = getHolidayInfo('2026-05-30');
    expect(k4?.tone).toBe('religious');
    expect(k4?.text).toContain('bayram');

    const kAfter = getHolidayInfo('2026-05-31');
    expect(kAfter).toBeNull();
  });

  it('2027-05-19 çifte bayram override kaydını öncelikle getirmeli', () => {
    const doubleHoliday = getHolidayInfo('2027-05-19');
    expect(doubleHoliday?.tone).toBe('national');
    expect(doubleHoliday?.text).toContain('çifte bayramımız kutlu olsun');
  });

  it('2029-02-14 sevgililer günü + ramazan çakışmasını override olarak getirmeli', () => {
    const val = getHolidayInfo('2029-02-14');
    expect(val?.tone).toBe('religious');
    expect(val?.text).toContain('sevgililer günü ile ramazan bayramı aynı güne denk geldi');
  });

  it('2031 yılında schedule tanımlanmamış olsa bile otomatik fallback ile Ramazan ve Kurbanı tanımalı', () => {
    // 2031-01-24: 1 Shawwal 1452 (Ramazan 1. gün)
    const h2031 = getHolidayInfo('2031-01-24');
    expect(h2031).not.toBeNull();
    expect(h2031?.name).toBe('Ramazan Bayramı');
    expect(h2031?.tone).toBe('religious');
  });

  it('Sıradan bir günde null dönmeli', () => {
    expect(getHolidayInfo('2026-10-26')).toBeNull();
    expect(getHolidayInfo('2026-06-15')).toBeNull();
  });
});
