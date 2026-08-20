import { api } from './api/index.js';
import { goto } from '$app/navigation';
import { getCookie, clearLoggedCookie } from './utils/cookie.js';

const initialHasSession = typeof document !== 'undefined' ? getCookie('kepce_logged_in') === 'true' : false;
const cachedUser = typeof window !== 'undefined' ? JSON.parse(localStorage.getItem('kepce_user_cache') || 'null') : null;

const initialPaginationMode = typeof window !== 'undefined' ? (localStorage.getItem('sayfalamaModu') || 'sayfali') : 'sayfali';

export const globalState = $state({
  user: cachedUser,
  isModerator: false,
  favorites: [],
  isReady: false,
  hasSession: initialHasSession,
  devMode: false,
  paginationMode: initialPaginationMode,
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
      globalState.user = null;
      globalState.isModerator = false;
      const isAuthError = err.status === 401 || err.status === 404 ||
        err.message.includes('401') || err.message.includes('404') ||
        err.message.includes('Invalid or expired token') ||
        err.message.includes('oturum süresi dolmuş') ||
        err.message.includes('açık anahtar eksik') ||
        err.message.includes('Kullanıcı bulunamadı');
      if (isAuthError) {
        clearLoggedCookie();
        if (typeof window !== 'undefined') localStorage.removeItem('kepce_user_cache');
        globalState.hasSession = false;
        // Eski kodda kullanıcı burada sessizce düşürülüyordu; UI sonsuz
        // "yükleniyor" durumunda kalıyordu. Şimdi login sayfasına yönlendir.
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
    // Görev #13-15: Misafir etkileşimlerinde sayfa yönlendirmesi yerine
    // hızlı giriş modalı açılır; kullanıcının sayfa state'i korunur.
    // Dynamic import: auth-gate → AuthGateModal → state.svelte.js
    // döngüsel bağımlılığını kırar.
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
      if (typeof window !== 'undefined') localStorage.removeItem('kepce_user_cache');
      if (typeof document !== 'undefined') document.body.classList.remove('is-logged-in');
      goto('/');
    }
  }
};
