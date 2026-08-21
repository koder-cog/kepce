<script>
  import "@/styles/pages/_auth.css";
  import { api } from "@/api/index.js";
  import Seo from "@/components/ui/Seo.svelte";

  let email = $state("");
  let isLoading = $state(false);
  let errorMsg = $state("");
  let isSuccess = $state(false);
  let formError = $state(false);

  async function handleSubmit(e) {
    e.preventDefault();
    errorMsg = "";
    formError = false;

    if (!email.includes("@")) {
      errorMsg = "Geçerli bir e-posta giriniz.";
      formError = true;
      return;
    }

    isLoading = true;

    try {
      await api.forgotPassword(email);

      isSuccess = true;
      const { showToast } = await import("@/components/ui/toast.js");
      showToast("Talimatlar gönderildi!");
    } catch (err) {
      errorMsg = err.message || "Bir hata oluştu. Tekrar deneyebilir misin?";
      formError = true;
    } finally {
      isLoading = false;
    }
  }
</script>

<Seo
  title="Şifremi Unuttum - Kepçe"
  description="Kepçe şifre sıfırlama talebi."
  noindex={true}
/>

<h1 class="auth-page__title">Şifre Yenileme</h1>

<div class="auth-form-container">
  <div id="reset-message-container">
    {#if isSuccess}
      <div class="auth-success">
        Eğer bu e-posta adresi sistemimizde kayıtlıysa, şifre sıfırlama
        talimatlarını gönderdik. Lütfen kutunu (ve spam klasörünü) kontrol et.
      </div>
    {/if}
    {#if errorMsg && !formError}
      <div class="auth-error">{errorMsg}</div>
    {/if}
  </div>

  {#if !isSuccess}
    <form class="auth-form" onsubmit={handleSubmit}>
      <div
        class="form-group form-group--floating"
        class:form-group--error={formError}
        data-error={errorMsg}
      >
        <input
          type="email"
          id="reset-email"
          bind:value={email}
          required
          autocomplete="email"
          placeholder=" "
        />
        <label class="form-label" for="reset-email"
          >E-postam şuydu<span class="form-required-mark">*</span></label
        >
      </div>

      <button
        type="submit"
        class="btn btn--primary auth-submit"
        disabled={isLoading}
      >
        {isLoading ? "Gönderiliyor..." : "Geri kalanı neydi yahu"}
      </button>
    </form>
  {/if}

  <div class="auth-footer">
    <h3 class="auth-footer__title">Hatırladın mı:</h3>
    <div class="auth-footer__links">
      <a href="/giris" class="auth-footer__link" data-link>Giriş yapılası</a>
      <a href="/kayit" class="auth-footer__link" data-link
        >Kayıtlı kullanıcı olunası</a
      >
    </div>
  </div>
</div>
