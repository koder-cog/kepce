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
    expect(h?.text).toMatch(/Atatürk|Atam/);

    // 2024 yılı % 4 = 0 olduğu için index 0'daki vefat sayacı metnini seçer (2024 - 1938 = 86)
    const h2024 = getHolidayInfo('2024-11-10');
    expect(h2024?.text).toContain('86. yıl dönümünde');
  });

  it('2026 Ramazan Bayramı Arefesini ve bayram günlerini tanımalı ve religious ton dönmeli', () => {
    const arefe = getHolidayInfo('2026-03-19');
    expect(arefe?.tone).toBe('religious');
    expect(arefe?.name).toBe('Ramazan Bayramı Arefesi');
    expect(arefe?.text).toBeTruthy();

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

  it('2026 Kurban Bayramı Arefesini ve bayram günlerini (1-4) tanımalı', () => {
    const arefe = getHolidayInfo('2026-05-26');
    expect(arefe?.tone).toBe('religious');
    expect(arefe?.name).toBe('Kurban Bayramı Arefesi');
    expect(arefe?.text).toBeTruthy();

    const k1 = getHolidayInfo('2026-05-27');
    expect(k1?.tone).toBe('religious');
    expect(k1?.name).toBe('Kurban Bayramı');
    expect(k1?.text).toBeTruthy();

    const k4 = getHolidayInfo('2026-05-30');
    expect(k4?.tone).toBe('religious');
    expect(k4?.text.toLowerCase()).toContain('bayram');

    const kAfter = getHolidayInfo('2026-05-31');
    expect(kAfter).toBeNull();
  });

  it('2027-05-19 çifte bayram override kaydını öncelikle getirmeli', () => {
    const doubleHoliday = getHolidayInfo('2027-05-19');
    expect(doubleHoliday?.tone).toBe('national');
    expect(doubleHoliday?.text.toLowerCase()).toContain('çifte bayram');
  });

  it('2029-02-14 sevgililer günü + ramazan çakışmasını override olarak getirmeli', () => {
    const val = getHolidayInfo('2029-02-14');
    expect(val?.tone).toBe('religious');
    expect(val?.text.toLowerCase()).toContain('gül buketiyle bayram çikolatasının');
  });

  it('2027-03-08 kadınlar günü + ramazan arefesi override kaydını getirmeli', () => {
    const val = getHolidayInfo('2027-03-08');
    expect(val?.tone).toBe('national');
    expect(val?.name).toBe('Dünya Kadınlar Günü ve Ramazan Arefesi');
    expect(val?.text.toLowerCase()).toContain('kadınların 8 mart');
  });

  it('2029-04-23 23 nisan + kurban arefesi override kaydını getirmeli', () => {
    const val = getHolidayInfo('2029-04-23');
    expect(val?.tone).toBe('national');
    expect(val?.name).toBe('23 Nisan ve Kurban Arefesi');
    expect(val?.text.toLowerCase()).toContain('bayram neşesi ile arefe telaşının');
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
