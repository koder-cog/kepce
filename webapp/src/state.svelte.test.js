import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock SvelteKit and API dependencies
vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

vi.mock('./api/index.js', () => ({
  api: {
    getMe: vi.fn(),
    getFavorites: vi.fn(),
    logout: vi.fn(),
  },
}));

// Mock browser APIs
if (typeof document === 'undefined') {
  global.document = {
    cookie: '',
    body: {
      classList: {
        add: vi.fn(),
        remove: vi.fn(),
        toggle: vi.fn(),
      },
    },
  };
}

import { globalState, authActions } from './state.svelte.js';
import { api } from './api/index.js';
import { goto } from '$app/navigation';

describe('Global Auth State', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    globalState.user = null;
    globalState.isModerator = false;
    globalState.favorites = [];
    globalState.isReady = false;
  });

  it('should initialize with default states', () => {
    expect(globalState.user).toBeNull();
    expect(globalState.isModerator).toBe(false);
    expect(globalState.favorites).toEqual([]);
    expect(globalState.isReady).toBe(false);
  });

  it('should logout and clear state', async () => {
    globalState.user = { id: 1, username: 'test' };
    globalState.isModerator = true;
    globalState.favorites = [1, 2];

    await authActions.logout();

    expect(globalState.user).toBeNull();
    expect(globalState.isModerator).toBe(false);
    expect(globalState.favorites).toEqual([]);
    expect(api.logout).toHaveBeenCalled();
    expect(goto).toHaveBeenCalledWith('/');
  });
});
