import { api } from './api/index.js';
import { goto, invalidateAll } from '$app/navigation';
import { getCookie, clearLoggedCookie } from './utils/cookie.js';

const initialHasSession = typeof document !== 'undefined' ? getCookie('kepce_logged_in') === 'true' : false;
const cachedUser = (typeof window !== 'undefined' && initialHasSession) ? JSON.parse(localStorage.getItem('kepce_user_cache') || 'null') : null;

const initialPaginationMode = typeof window !== 'undefined' ? (localStorage.getItem('sayfalamaModu') || 'sayfali') : 'sayfali';
const initialIsApp = typeof navigator !== 'undefined' ? navigator.userAgent.includes('KepceMobileApp') : false;

export const globalState = $state({
  user: cachedUser,
  isModerator: false,
  favorites: [],
  isReady: false,
  hasSession: initialHasSession,
  devMode: false,
  paginationMode: initialPaginationMode,
  isApp: initialIsApp,
});

export function setPaginationMode(mode) {
  const validMode = mode === 'akici' ? 'akici' : 'sayfali';
  globalState.paginationMode = validMode;
  if (typeof window !== 'undefined') {
    localStorage.setItem('sayfalamaModu', validMode);
  }
}

export const authActions = {
  async refreshUser() {
    globalState.hasSession = getCookie('kepce_logged_in') === 'true';
    if (!globalState.hasSession) {
      globalState.user = null;
      if (typeof document !== 'undefined') {
        document.body.classList.remove('is-logged-in');
      }
      globalState.isReady = true;
      return;
    }

    if (!globalState.user && typeof window !== 'undefined') {
      try {
        const cached = localStorage.getItem('kepce_user_cache');
        if (cached) {
          const parsed = JSON.parse(cached);
          if (parsed && typeof parsed === 'object') {
            globalState.user = parsed;
            globalState.isModerator = parsed.is_admin || false;
          }
        }
      } catch (_) {}
    }

    try {
      globalState.user = await api.getMe();
      if (typeof window !== 'undefined') {
        localStorage.setItem('kepce_user_cache', JSON.stringify(globalState.user));
      }
      globalState.isModerator = globalState.user?.is_admin || false;
      if (globalState.user) {
        globalState.favorites = await api.getFavorites();
      }
    } catch (err) {
      console.warn('Auth check failed:', err);
      const msg = String(err?.message || '');
      const isAuthError = err?.status === 401 || err?.status === 404 ||
        msg.includes('401') || msg.includes('404') ||
        msg.includes('Invalid or expired token') ||
        msg.includes('oturum süresi dolmuş') ||
        msg.includes('açık anahtar eksik') ||
        msg.includes('Kullanıcı bulunamadı');
      if (isAuthError) {
        globalState.user = null;
        globalState.isModerator = false;
        clearLoggedCookie();
        if (typeof window !== 'undefined') localStorage.removeItem('kepce_user_cache');
        globalState.hasSession = false;
        // Oturum geçersizse state'i temizle ve kullanıcıyı giriş sayfasına yönlendir.
        if (typeof window !== 'undefined' && !window.location.pathname.startsWith('/giris')) {
          const currentPath = window.location.pathname + window.location.search;
          goto(`/giris?redirect=${encodeURIComponent(currentPath)}`);
        }
      }
    } finally {
      if (typeof document !== 'undefined') {
        document.body.classList.toggle('is-logged-in', !!globalState.user);
      }
      globalState.isReady = true;
    }
  },

  async triggerLogin(reason = null) {
    // Misafir etkileşimlerinde sayfa yönlendirmesi yerine hızlı giriş modalı açılır.
    // Dynamic import, modüller arasındaki döngüsel bağımlılığı önler.
    if (typeof window === 'undefined') return;
    const { openAuthGate } = await import('./components/features/auth-gate.js');
    openAuthGate({ reason });
  },

  async logout() {
    try {
      await api.logout();
    } catch (e) {
      console.warn('Sunucudan çıkış yapılamadı, yerel state temizleniyor...', e);
    } finally {
      globalState.user = null;
      globalState.isModerator = false;
      globalState.favorites = [];
      globalState.hasSession = false;
      clearLoggedCookie();
      if (typeof window !== 'undefined') localStorage.removeItem('kepce_user_cache');
      if (typeof document !== 'undefined') document.body.classList.remove('is-logged-in');
      try {
        await invalidateAll();
      } catch (err) {}
      goto('/');
    }
  }
};
