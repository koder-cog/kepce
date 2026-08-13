import { describe, it, expect } from 'vitest';
import { getMonthName, timeAgo } from './date.js';

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
