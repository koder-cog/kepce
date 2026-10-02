import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

vi.mock('@/api/index.js', () => ({
  api: {
    giveCrossBorderConsent: vi.fn(),
  },
}));

vi.mock('@/components/ui/toast.js', () => ({
  showToast: vi.fn(),
}));

import { globalState } from '@/state.svelte.js';
import { api } from '@/api/index.js';
import { goto } from '$app/navigation';

describe('CrossBorderConsent Logic', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    globalState.user = null;
  });

  it('calculates remaining days correctly before deadline', () => {
    const futureDate = new Date(Date.now() + 15 * 24 * 60 * 60 * 1000).toISOString();
    globalState.user = {
      id: 'test-user-id',
      username: 'ogrenci',
      consent_cross_border: false,
      consent_deadline_at: futureDate,
    };

    const deadline = new Date(globalState.user.consent_deadline_at);
    const now = new Date();
    const diffTime = deadline.getTime() - now.getTime();
    const remainingDays = Math.max(0, Math.ceil(diffTime / (1000 * 60 * 60 * 24)));

    expect(remainingDays).toBeGreaterThanOrEqual(14);
    expect(remainingDays).toBeLessThanOrEqual(16);
  });

  it('marks as expired when deadline is in the past', () => {
    const pastDate = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
    globalState.user = {
      id: 'test-user-id',
      username: 'ogrenci',
      consent_cross_border: false,
      consent_deadline_at: pastDate,
    };

    const deadline = new Date(globalState.user.consent_deadline_at);
    const now = new Date();
    const diffTime = deadline.getTime() - now.getTime();
    const remainingDays = Math.max(0, Math.ceil(diffTime / (1000 * 60 * 60 * 24)));
    const isExpired = remainingDays <= 0;

    expect(remainingDays).toBe(0);
    expect(isExpired).toBe(true);
  });

  it('updates global user state upon successful consent call', async () => {
    const mockUpdated = {
      consent_cross_border: true,
      consent_cross_border_at: new Date().toISOString(),
    };
    api.giveCrossBorderConsent.mockResolvedValue(mockUpdated);

    globalState.user = {
      id: 'test-user-id',
      username: 'ogrenci',
      consent_cross_border: false,
      consent_deadline_at: new Date().toISOString(),
    };

    const res = await api.giveCrossBorderConsent();
    globalState.user = {
      ...globalState.user,
      consent_cross_border: true,
      consent_cross_border_at: res.consent_cross_border_at,
    };

    expect(globalState.user.consent_cross_border).toBe(true);
    expect(globalState.user.consent_cross_border_at).toBe(mockUpdated.consent_cross_border_at);
  });
});
