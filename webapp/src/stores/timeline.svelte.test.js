import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@/api/index.js', () => ({
  api: {
    getMenusByDate: vi.fn(),
  },
}));

vi.mock('@/stores/city.svelte.js', () => ({
  getCurrentCity: vi.fn(() => 'istanbul'),
  setCurrentCity: vi.fn(),
  getCitiesData: vi.fn(async () => [{ id: 1, name: 'İstanbul', slug: 'istanbul', has_celiac: false }]),
}));

vi.mock('@/lib/dom/motion.js', () => ({
  wait: vi.fn(),
  getDuration: vi.fn((d) => d),
  runNextTick: vi.fn((cb) => cb()),
  isMotionEnabled: vi.fn(() => true),
}));

import { createTimelineStore } from './timeline.svelte.js';
import { api } from '@/api/index.js';

describe('Timeline Store - Vote State & Hydration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    document.cookie = '';
  });

  it('preserves existing user vote when setPrerenderedData is called with anonymous SSR data', () => {
    const store = createTimelineStore();

    // İlk durum: Kullanıcı oy vermiş
    store.setPrerenderedData([
      { id: 101, meal_type: 'dinner', my_vote: 'negative', rating_sum: -1, vote_count: 1, items: [{ raw_name: 'Çorba' }] },
    ], 'istanbul', '2026-09-18');

    expect(store.dinnerData[0].my_vote).toBe('negative');

    // SPA navigasyonu veya sunucudan yeni SSR verisi geldiğinde (anonim / my_vote: null)
    store.setPrerenderedData([
      { id: 101, meal_type: 'dinner', my_vote: null, rating_sum: -1, vote_count: 1, items: [{ raw_name: 'Çorba' }] },
    ], 'istanbul', '2026-09-18');

    // Mevcut aktif oy korunmalı
    expect(store.dinnerData[0].my_vote).toBe('negative');
  });

  it('hydrates user votes from API when user has active session', async () => {
    const store = createTimelineStore();
    document.cookie = 'kepce_logged_in=true; path=/';

    // SSR'dan gelen anonim veriyle başlat
    store.setPrerenderedData([
      { id: 202, meal_type: 'dinner', my_vote: null, rating_sum: -1, vote_count: 1, items: [{ raw_name: 'Pilav' }] },
    ], 'istanbul', '2026-09-18');

    // API'den oturumlu kullanıcının oyu döner
    api.getMenusByDate.mockResolvedValueOnce([
      { id: 202, meal_type: 'dinner', my_vote: 'negative', rating_sum: -1, vote_count: 1, items: [{ raw_name: 'Pilav' }] },
    ]);

    await store.hydrateUserVotes();

    expect(api.getMenusByDate).toHaveBeenCalledWith('istanbul', expect.any(String), 'standard', { noCache: true });
    expect(store.dinnerData[0].my_vote).toBe('negative');
    expect(store.dinnerData[0].rating_sum).toBe(-1);
  });

  it('clears my_vote when auth-changed event indicates user logout', () => {
    const store = createTimelineStore();

    store.setPrerenderedData([
      { id: 303, meal_type: 'dinner', my_vote: 'positive', rating_sum: 5, vote_count: 5, items: [{ raw_name: 'Köfte' }] },
    ], 'istanbul', '2026-09-18');

    expect(store.dinnerData[0].my_vote).toBe('positive');

    // Çıkış yapıldığında auth-changed tetiklenir
    window.dispatchEvent(new CustomEvent('auth-changed', { detail: { user: null } }));

    expect(store.dinnerData[0].my_vote).toBeNull();
  });
});
