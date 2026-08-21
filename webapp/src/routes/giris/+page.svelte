<script>
  import "@/styles/pages/_auth.css";
  import { goto } from "$app/navigation";
  import { globalState, authActions } from "@/state.svelte.js";

  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  let username = $state("");
  let password = $state("");
  let remember = $state(false);
  let showPassword = $state(false);
  let honeypot = $state("");

  let isLoading = $state(false);
  let errorMsg = $state("");
  let formError = $state(false);

  let isPasswordlessMode = $state(false);
  let isPasswordlessSuccess = $state(false);

  const user = $derived(globalState?.user);

  async function handleSubmit(e) {
    e.preventDefault();
    if (honeypot) return; // Spam protection

    isLoading = true;

    if (isPasswordlessMode) {
      try {
        await api.passwordless(username);
        isPasswordlessSuccess = true;
        errorMsg = "";
        formError = false;
      } catch (err) {
        errorMsg = err.message || "Bir hata oluştu.";
        formError = true;
      } finally {
        isLoading = false;
      }
      return;
    }

    try {
      await api.login(username, password, remember);
      await authActions.refreshUser();

      formError = false;
      errorMsg = "";

      const urlParams = new URLSearchParams(window.location.search);
      const redirectPath = urlParams.get("redirect") || "/";
      goto(redirectPath);
    } catch (err) {
      errorMsg =
        err.message || "Bir şeyleri hatalı girdin, tekrar dene istersen?";
      formError = true;
      isLoading = false;
    }
  }

  function handleLogout() {
    authActions.logout();
  }
</script>

<Seo
  title="Giriş Yap - Kepçe"
  description="Kepçe hesabınıza giriş yapın; favori yemeklerinizi kaydedin, menülere yorum yapın ve bildirimleri takip edin."
/>

{#if user}
  <div class="empty-state-container">
    <EmptyState
      statusCode={403}
      title={"403: Zaten Buradasın"}
      desc={`@${user.username} olarak zaten giriş yapmış durumdasın. Başka bir hesapla girmek istiyorsan önce çıkış yapmalısın.`}
    >
      <a href="/" data-link class="btn btn--secondary">Ana sayfaya sığın</a>
      <button type="button" class="btn btn--primary" onclick={handleLogout}
        >Çıkış yap</button
      >
    </EmptyState>
  </div>
{:else}
  <div class="auth-form-container">
    <h1 class="auth-page__title u-mb-lg">Giriş</h1>

    <div class="auth-social-buttons">
      <button
        type="button"
        class="btn btn--secondary auth-social-btn"
        onclick={() => (window.location.href = "/api/v1/auth/google/login")}
      >
        Google Hesabı ile giriş yap
      </button>
    </div>

    <div class="auth-divider">veya</div>

    {#if errorMsg}
      <div id="login-error-container">
        <div class="auth-error" role="alert">{errorMsg}</div>
      </div>
    {/if}

    <form class="auth-form" onsubmit={handleSubmit}>
      <!-- Honeypot field for spam protection -->
      <input
        type="text"
        name="website"
        tabindex="-1"
        autocomplete="off"
        class="u-honeypot"
        bind:value={honeypot}
      />

      {#if isPasswordlessSuccess}
        <div class="passwordless-success u-mb-md">
          <div class="passwordless-success__icon">
            {@html icon("send", 40) || icon("mail", 40) || icon("check", 40)}
          </div>
          <h3 class="passwordless-success__title">Bağlantı Yola Çıktı!</h3>
          <p class="passwordless-success__desc">
            Eğer <strong>{username}</strong> sistemde kayıtlıysa, giriş bağlantını
            e-postana gönderdik.
          </p>
          <p class="passwordless-success__note">
            Lütfen gelen kutunu (ve ne olur ne olmaz spam klasörünü) kontrol et.
          </p>
        </div>
        <button
          type="button"
          class="btn btn--secondary auth-submit"
          onclick={() => {
            isPasswordlessSuccess = false;
            isPasswordlessMode = false;
          }}
        >
          Şifreyle Girişe Dön
        </button>
      {:else}
        <div
          class="form-group form-group--floating"
          class:form-group--error={formError}
        >
          <input
            type="text"
            id="username"
            bind:value={username}
            required
            autocomplete={isPasswordlessMode ? "email" : "username"}
            placeholder=" "
          />
          <label class="form-label" for="username"
            >{isPasswordlessMode
              ? "Kayıtlı E-posta Adresiniz"
              : "E-posta veya kullanıcı adı"}<span class="form-required-mark"
              >*</span
            ></label
          >
        </div>

        {#if !isPasswordlessMode}
          <div
            class="form-group form-group--floating"
            class:form-group--error={formError}
          >
            <input
              type={showPassword ? "text" : "password"}
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
              aria-label={showPassword ? "Şifreyi gizle" : "Şifreyi göster"}
              onclick={() => (showPassword = !showPassword)}
            >
              {@html icon(showPassword ? "eyeOff" : "eye", 20)}
            </button>
          </div>

          <label class="auth-remember">
            <input
              type="checkbox"
              class="c-input-hidden"
              bind:checked={remember}
            />
            <div class="c-switch">
              <div class="c-switch__handle"></div>
            </div>
            <span class="auth-remember__text">Unutma bunları</span>
          </label>
        {/if}

        <button
          type="submit"
          class="btn btn--primary auth-submit"
          disabled={isLoading}
        >
          {#if isLoading}
            {isPasswordlessMode ? "Gönderiliyor..." : "Giriş yapılıyor..."}
          {:else}
            {isPasswordlessMode
              ? "Giriş Bağlantısı Gönder"
              : "Giriş yapmaya çabala"}
          {/if}
        </button>

        <button
          type="button"
          class="btn btn--secondary auth-submit u-mt-sm"
          onclick={() => {
            isPasswordlessMode = !isPasswordlessMode;
            formError = false;
            errorMsg = "";
          }}
          disabled={isLoading}
        >
          {isPasswordlessMode ? "Şifre ile giriş yap" : "Şifresiz giriş yap"}
        </button>
      {/if}
    </form>

    <div class="auth-footer">
      <h3 class="auth-footer__title">Giremeyiş</h3>
      <div class="auth-footer__links">
        <a href="/sifre-yenile" class="auth-footer__link" data-link
          >Şifremi unuttum</a
        >
        <a href="/kayit" class="auth-footer__link" data-link
          >Kayıtlı kullanıcı olunası</a
        >
      </div>
    </div>
  </div>
{/if}

<style>
  .passwordless-success {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 2rem 1.5rem;
    text-align: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  }

  .passwordless-success__icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    background-color: var(--color-success-soft);
    color: var(--color-success);
    border-radius: 50%;
    margin-bottom: 1rem;
  }

  .passwordless-success__title {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-text);
    margin-bottom: 0.5rem;
  }

  .passwordless-success__desc {
    color: var(--color-text);
    font-size: 1rem;
    line-height: 1.5;
    margin-bottom: 1rem;
  }

  .passwordless-success__note {
    font-size: 0.875rem;
    color: var(--color-muted);
  }
</style>
