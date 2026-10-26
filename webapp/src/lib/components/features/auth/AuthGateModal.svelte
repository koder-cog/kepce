<script>
  /**
   * AuthGateModal
   *
   * Misafir kullanıcı etkileşim gerektiren bir butona bastığında (yorum,
   * favori, oy vb.) sayfa yönlendirmesi yerine bu modal açılır. Hızlı giriş
   * başarılı olduğunda sayfa/içerik YENİLENMEZ; globalState reaktif olarak
   * güncellenir ve kullanıcının o anki girdileri korunur.
   */
  import Modal from '$lib/components/ui/Modal.svelte';
  import { icon } from '$lib/components/ui/icons.js';
  import { api } from '$lib/api/index.js';
  import { globalState, authActions } from '$lib/stores/appState.svelte.js';
  import { showToast } from '$lib/components/ui/toast.js';
  import { goto } from '$app/navigation';

  let { onClose, reason = null } = $props();

  let modalRef = $state(null);
  let identifier = $state('');
  let password = $state('');
  let isLoading = $state(false);
  let errorMsg = $state('');

  export function close() {
    modalRef?.close();
  }

  async function handleLogin(e) {
    e?.preventDefault();
    if (isLoading) return;

    if (!identifier.trim() || !password) {
      errorMsg = 'Kullanıcı adı ve şifre alanları zorunludur.';
      showToast(errorMsg, { type: 'error' });
      return;
    }

    isLoading = true;
    errorMsg = '';

    try {
      await api.login(identifier.trim(), password);
      await authActions.refreshUser();
      const name = globalState.user?.username;
      showToast(name ? `Tekrar hoş geldin, @${name}!` : 'Giriş yapıldı!', { type: 'success' });
      close();
    } catch (err) {
      errorMsg = err.message || 'Bir şeyleri hatalı girdin, tekrar dene istersen?';
      isLoading = false;
      showToast(errorMsg, { type: 'error' });
    }
  }

  function handleRegister() {
    const currentPath = window.location.pathname + window.location.search;
    goto(`/kayit?redirect=${encodeURIComponent(currentPath)}`);
    close();
  }
</script>

<Modal
  bind:this={modalRef}
  options={{ title: 'Giriş gerekli', iconHtml: icon('login', 24) }}
  {onClose}
>
  {#snippet children()}
    <p class="u-text-sm u-color-muted">
      {reason ?? 'Bu işlemi gerçekleştirmek için giriş yapman gerekiyor.'}
      Hesabın yok mu?
      <a
        href="/kayit"
        class="u-link u-font-bold"
        onclick={(e) => {
          e.preventDefault();
          handleRegister();
        }}>Hemen kayıt ol.</a
      >
    </p>

    <form id="auth-gate-form" class="c-modal__form-group u-mt-md" onsubmit={handleLogin}>
      <div
        class="form-group form-group--floating"
        class:form-group--error={!!errorMsg && !identifier.trim()}
        data-error={errorMsg && !identifier.trim() ? errorMsg : undefined}
      >
        <input
          type="text"
          id="auth-gate-identifier"
          placeholder=" "
          bind:value={identifier}
          autocomplete="username"
          required
        />
        <label class="form-label" for="auth-gate-identifier">Kullanıcı Adı<span class="form-required-mark">*</span></label>
      </div>

      <div
        class="form-group form-group--floating u-mt-md"
        class:form-group--error={!!errorMsg && !password}
        data-error={errorMsg && !password ? errorMsg : undefined}
      >
        <input
          type="password"
          id="auth-gate-password"
          placeholder=" "
          bind:value={password}
          autocomplete="current-password"
          required
        />
        <label class="form-label" for="auth-gate-password">Şifre<span class="form-required-mark">*</span></label>
      </div>
    </form>
  {/snippet}

  {#snippet footer()}
    <button type="button" class="btn btn--secondary btn--squish" onclick={close} disabled={isLoading}>
      Kalsın
    </button>
    <button type="submit" form="auth-gate-form" class="btn btn--primary btn--squish" disabled={isLoading}>
      {isLoading ? 'Giriş yapılıyor...' : 'Giriş yap'}
    </button>
  {/snippet}
</Modal>
