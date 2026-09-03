import { describe, it, expect } from 'vitest';
import {
  isOffSeasonDate,
  isOrientationSeason,
  getClosestMondayToSept15,
} from './season.js';

describe('season utilities', () => {
  it('July and August are always off-season', () => {
    expect(isOffSeasonDate('2026-07-15')).toBe(true);
    expect(isOffSeasonDate('2026-08-31')).toBe(true);
    expect(isOffSeasonDate(new Date(2025, 6, 1))).toBe(true); // July
    expect(isOffSeasonDate(new Date(2027, 7, 20))).toBe(true); // August
  });

  it('September respects opening calendar', () => {
    // 2026 opens 09-14
    expect(isOffSeasonDate('2026-09-03')).toBe(true);
    expect(isOffSeasonDate('2026-09-13')).toBe(true);
    expect(isOffSeasonDate('2026-09-14')).toBe(false);
    expect(isOffSeasonDate('2026-09-20')).toBe(false);

    // 2025 opened 09-15
    expect(isOffSeasonDate('2025-09-14')).toBe(true);
    expect(isOffSeasonDate('2025-09-15')).toBe(false);
  });

  it('In-season months return false', () => {
    expect(isOffSeasonDate('2026-10-01')).toBe(false);
    expect(isOffSeasonDate('2026-11-15')).toBe(false);
    expect(isOffSeasonDate('2026-03-10')).toBe(false);
  });

  it('Orientation season covers July through October', () => {
    expect(isOrientationSeason('2026-07-01')).toBe(true);
    expect(isOrientationSeason('2026-08-15')).toBe(true);
    expect(isOrientationSeason('2026-09-03')).toBe(true);
    expect(isOrientationSeason('2026-10-31')).toBe(true);
    expect(isOrientationSeason('2026-11-01')).toBe(false);
    expect(isOrientationSeason('2026-06-30')).toBe(false);
  });

  it('Calculates closest Monday to September 15 accurately', () => {
    expect(getClosestMondayToSept15(2024)).toBe(16); // 16 Eylül 2024 Pazartesi
    expect(getClosestMondayToSept15(2025)).toBe(15); // 15 Eylül 2025 Pazartesi
    expect(getClosestMondayToSept15(2026)).toBe(14); // 14 Eylül 2026 Pazartesi
    expect(getClosestMondayToSept15(2027)).toBe(13); // 13 Eylül 2027 Pazartesi
    expect(getClosestMondayToSept15(2028)).toBe(11); // 11 Eylül 2028 Pazartesi (2. Pazartesi tercihi)
    expect(getClosestMondayToSept15(2029)).toBe(17); // 17 Eylül 2029 Pazartesi
    expect(getClosestMondayToSept15(2030)).toBe(16); // 16 Eylül 2030 Pazartesi
  });
});
