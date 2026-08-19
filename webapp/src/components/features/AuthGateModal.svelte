<script>
  /**
   * AuthGateModal — Görev #13-15
   *
   * Misafir kullanıcı etkileşim gerektiren bir butona bastığında (yorum,
   * favori, oy vb.) sayfa yönlendirmesi yerine bu modal açılır. Hızlı giriş
   * başarılı olduğunda sayfa/içerik YENİLENMEZ; globalState reaktif olarak
   * güncellenir ve kullanıcının o anki girdileri (ör. yorum taslağı)
   * korunur (#15, #16 ile birlikte).
   */
  import Modal from '../ui/Modal.svelte';
  import { icon } from '../ui/icons.js';
  import { api } from '../../api/index.js';
  import { globalState, authActions } from '../../state.svelte.js';
  import { showToast } from '../ui/toast.js';
  import { goto } from '$app/navigation';

  let { onClose, reason = null } = $props();

  let modalRef = $state(null);
  let identifier = $state('');
  let password = $state('');
  let isLoading = $state(false);
  let errorMsg = $state('');

  let isPasswordlessMode = $state(false);
  let isPasswordlessSuccess = $state(false);

  // Modal.svelte'in çıkış animasyonlu close() metodunu dışarıya aç
  export function close() {
    modalRef?.close();
  }

  async function handleLogin(e) {
    e?.preventDefault();
    if (isLoading) return;
    if (!identifier.trim() || (!isPasswordlessMode && !password)) {
      errorMsg = 'E-posta/kullanıcı adı ve şifre alanları zorunludur.';
      return;
    }

    isLoading = true;
    errorMsg = '';

    if (isPasswordlessMode) {
      try {
        await api.passwordless(identifier.trim());
        isPasswordlessSuccess = true;
      } catch (err) {
        errorMsg = err.message || 'Bir hata oluştu.';
      } finally {
        isLoading = false;
      }
      return;
    }

    try {
      await api.login(identifier.trim(), password);
      // Sayfa yenilenmeden state güncellenir (#15)
      await authActions.refreshUser();
      const name = globalState.user?.username;
      showToast(name ? `Tekrar hoş geldin, @${name}!` : 'Giriş yapıldı!', 'success');
      close();
    } catch (err) {
      errorMsg = err.message || 'Bir şeyleri hatalı girdin, tekrar dene istersen?';
      isLoading = false;
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
  onClose={onClose}
>
  {#snippet children()}
    <p class="u-text-sm u-color-muted">
      {reason ?? 'Bu işlemi gerçekleştirmek için giriş yapman gerekiyor.'} 
      Hesabın yok mu? <a href="/kayit" class="u-link u-font-bold" onclick={(e) => { e.preventDefault(); handleRegister(); }}>Hemen kayıt ol.</a>
    </p>

    <form id="auth-gate-form" class="c-modal__form-group u-mt-md" onsubmit={handleLogin}>
      {#if errorMsg}
        <div class="auth-error" id="auth-gate-error" role="alert">{errorMsg}</div>
      {/if}

      {#if isPasswordlessSuccess}
        <div class="passwordless-success u-mb-md">
          <div class="passwordless-success__icon">
            {@html icon('send', 40) || icon('mail', 40) || icon('check', 40)}
          </div>
          <h3 class="passwordless-success__title">Bağlantı Yola Çıktı!</h3>
          <p class="passwordless-success__desc">
            Eğer <strong>{identifier}</strong> sistemde kayıtlıysa, giriş bağlantını e-postana gönderdik.
          </p>
          <p class="passwordless-success__note">
            Lütfen gelen kutunu kontrol et.
          </p>
        </div>
      {:else}
        <div class="form-group form-group--floating" class:form-group--error={!!errorMsg}>
          <input
            type="text"
            id="auth-gate-identifier"
            class="form-input"
            placeholder=" "
            bind:value={identifier}
            autocomplete={isPasswordlessMode ? "email" : "username"}
            aria-describedby={errorMsg ? 'auth-gate-error' : undefined}
          />
          <label class="form-label" for="auth-gate-identifier">{isPasswordlessMode ? 'Kayıtlı e-posta adresiniz' : 'E-posta veya kullanıcı adı'}</label>
        </div>

        {#if !isPasswordlessMode}
          <div class="form-group form-group--floating" class:form-group--error={!!errorMsg}>
            <input
              type="password"
              id="auth-gate-password"
              class="form-input"
              placeholder=" "
              bind:value={password}
              autocomplete="current-password"
              aria-describedby={errorMsg ? 'auth-gate-error' : undefined}
            />
            <label class="form-label" for="auth-gate-password">Şifre</label>
          </div>
        {/if}
      {/if}
    </form>
  {/snippet}

  {#snippet footer()}
    {#if isPasswordlessSuccess}
      <button type="button" class="btn btn--secondary" onclick={close}>Kapat</button>
    {:else}
      <button type="button" class="btn btn--secondary" onclick={close} disabled={isLoading}>
        Kalsın
      </button>
      <button type="submit" form="auth-gate-form" class="btn btn--primary" disabled={isLoading}>
        {#if isLoading}
          {isPasswordlessMode ? 'Gönderiliyor...' : 'Giriş yapılıyor...'}
        {:else}
          {isPasswordlessMode ? 'Link Gönder' : 'Giriş yap'}
        {/if}
      </button>
    {/if}
  {/snippet}
</Modal>

