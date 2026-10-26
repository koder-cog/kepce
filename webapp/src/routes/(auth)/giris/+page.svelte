<script>
  import '@/styles/pages/_auth.css';
  import { goto } from '$app/navigation';
  import { globalState, authActions } from '$lib/stores/appState.svelte.js';

  import { api } from '$lib/api/index.js';
  import { icon } from '$lib/components/ui/icons.js';
  import { showToast } from '$lib/components/ui/toast.js';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Seo from '$lib/components/ui/Seo.svelte';

  let username = $state('');
  let password = $state('');
  let remember = $state(false);
  let showPassword = $state(false);
  let honeypot = $state('');

  let isLoading = $state(false);
  let errorMsg = $state('');
  let formError = $state(false);
  let isRedirecting = $state(false);

  const user = $derived(globalState?.user);

  function getSafeRedirectUrl() {
    if (typeof window === 'undefined') return '/';
    const urlParams = new URLSearchParams(window.location.search);
    const redirectParam = urlParams.get('redirect');

    if (
      redirectParam &&
      redirectParam.startsWith('/') &&
      !redirectParam.startsWith('//') &&
      !redirectParam.toLowerCase().startsWith('/\\')
    ) {
      return redirectParam;
    }

    if (document.referrer) {
      try {
        const refUrl = new URL(document.referrer);
        if (
          refUrl.origin === window.location.origin &&
          !refUrl.pathname.startsWith('/giris') &&
          !refUrl.pathname.startsWith('/kayit')
        ) {
          return refUrl.pathname + refUrl.search + refUrl.hash;
        }
      } catch (e) {}
    }

    return '/';
  }

  function handleGoBack() {
    if (typeof window !== 'undefined' && window.history.length > 1) {
      window.history.back();
    } else {
      goto('/');
    }
  }

  async function handleSubmit(e) {
    e.preventDefault();
    if (honeypot) return;

    if (!username.trim() || !password) {
      errorMsg = 'Kullanıcı adı ve şifre alanları zorunludur.';
      formError = true;
      showToast(errorMsg, { type: 'error' });
      return;
    }

    isLoading = true;

    try {
      await api.login(username.trim(), password, remember);
      isRedirecting = true;
      await authActions.refreshUser();

      formError = false;
      errorMsg = '';

      const targetPath = getSafeRedirectUrl();
      goto(targetPath);
    } catch (err) {
      isRedirecting = false;
      errorMsg = err.message || 'Bir şeyleri hatalı girdin, tekrar dene istersen?';
      formError = true;
      isLoading = false;
      showToast(errorMsg, { type: 'error' });
    }
  }

  function handleLogout() {
    authActions.logout();
  }
</script>

<Seo
  title="Giriş Yap | Kepçe"
  description="Kepçe hesabınıza giriş yapın, favori yemeklerinizi kaydedin, menülere yorum yapın ve bildirimleri takip edin."
/>

{#if globalState.isReady && user && !isRedirecting}
  <div class="empty-state-container">
    <EmptyState
      statusCode={403}
      title={'403: Zaten Buradasın'}
      desc={`@${user.username} olarak zaten giriş yapmış durumdasın. Başka bir hesapla girmek istiyorsan önce çıkış yapmalısın.`}
    >
      <button type="button" class="btn btn--secondary btn--squish" onclick={handleGoBack}
        >Önceki sayfaya dön</button
      >
      <a href="/" data-link class="btn btn--secondary btn--squish">Ana sayfa</a>
      <button type="button" class="btn btn--primary btn--squish" onclick={handleLogout}
        >Çıkış yap</button
      >
    </EmptyState>
  </div>
{:else}
  <div class="auth-form-container">
    <h1 class="auth-page__title u-mb-lg">Giriş</h1>

    <form class="auth-form" onsubmit={handleSubmit}>
      <!-- Honeypot field for spam protection -->
      <input
        type="text"
        id="login-website-honeypot"
        name="website"
        aria-label="Website"
        tabindex="-1"
        autocomplete="off"
        class="u-honeypot"
        bind:value={honeypot}
      />

      <div class="form-group">
        <div
          class="form-group--floating u-mb-0"
          class:form-group--error={formError && !username.trim()}
          data-error={formError && !username.trim() ? 'Kullanıcı adınızı giriniz' : undefined}
        >
          <input
            type="text"
            id="username"
            bind:value={username}
            required
            autocomplete="username"
            placeholder=" "
          />
          <label class="form-label" for="username"
            >Kullanıcı Adı<span class="form-required-mark">*</span></label
          >
        </div>
        <div class="form-help u-mt-xs">Kullanıcı adınızı giriniz.</div>
      </div>

      <div
        class="form-group form-group--floating"
        class:form-group--error={formError && !password}
        data-error={formError && !password ? 'Şifrenizi giriniz' : undefined}
      >
        <input
          type={showPassword ? 'text' : 'password'}
          id="password"
          bind:value={password}
          required
          autocomplete="current-password"
          placeholder=" "
        />
        <label class="form-label" for="password"
          >Şifre<span class="form-required-mark">*</span></label
        >
        <button
          type="button"
          class="password-toggle"
          aria-label={showPassword ? 'Şifreyi gizle' : 'Şifreyi göster'}
          onclick={() => (showPassword = !showPassword)}
        >
          {@html icon(showPassword ? 'eyeNotLooking' : 'eyeLooking', 20)}
        </button>
      </div>

      <label class="form-switch-row u-mt-sm">
        <input type="checkbox" class="c-input-hidden" bind:checked={remember} />
        <div class="c-switch">
          <div class="c-switch__handle"></div>
        </div>
        <span class="form-switch-row__text u-weight-bold">Unutma bunları</span>
      </label>

      <button
        type="submit"
        class="btn btn--primary btn--large u-w-full btn--squish u-mt-xl"
        disabled={isLoading}
      >
        {isLoading ? 'Giriş yapılıyor...' : 'Giriş yap'}
      </button>
    </form>

    <div class="auth-footer">
      <h2 class="auth-footer__title">Giremeyiş</h2>
      <div class="auth-footer__links">
        <a href="/sifre-yenile" class="auth-footer__link" data-link>Şifremi unuttum</a>
        <a href="/kayit" class="auth-footer__link" data-link>Kayıtlı kullanıcı olunası</a>
      </div>
    </div>
  </div>
{/if}
