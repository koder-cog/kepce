<script>
  import "@/styles/pages/_auth.css";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "@/api/index.js";
  import { showToast } from "@/components/ui/toast.js";
  import Seo from "@/components/ui/Seo.svelte";

  let token = $state("");
  let password = $state("");
  let repeatPassword = $state("");
  let showPassword = $state(false);
  let showRepeatPassword = $state(false);

  let isLoading = $state(false);
  let errorMsg = $state("");
  let formError = $state(false);
  let isSuccess = $state(false);

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    token = urlParams.get("token") || "";

    if (!token) {
      errorMsg =
        "Şifre sıfırlama anahtarı eksik veya geçersiz. Lütfen yeni bir bağlantı talep et.";
      formError = true;
    }
  });

  async function handleSubmit(e) {
    e.preventDefault();
    errorMsg = "";
    formError = false;

    if (!token) {
      errorMsg =
        "Şifre sıfırlama anahtarı eksik. Lütfen yeni bir bağlantı talep et.";
      formError = true;
      return;
    }

    if (password.length < 8) {
      errorMsg = "Şifre en az 8 karakter olmalıdır.";
      formError = true;
      return;
    }

    if (password !== repeatPassword) {
      errorMsg = "Şifreler uyuşmuyor.";
      formError = true;
      return;
    }

    isLoading = true;

    try {
      await api.resetPassword(token, password);
      isSuccess = true;
      showToast(
        "Şifreniz başarıyla sıfırlandı. Giriş yapabilirsiniz!",
        "success",
      );

      setTimeout(() => {
        goto("/giris");
      }, 2000);
    } catch (err) {
      errorMsg =
        err.message || "Şifre sıfırlanamadı. Bağlantı süresi dolmuş olabilir.";
      formError = true;
    } finally {
      isLoading = false;
    }
  }
</script>

<Seo
  title="Yeni Şifre Belirleme - Kepçe"
  description="Yeni şifrenizi belirleyin."
  noindex={true}
/>

<h2 class="auth-page__title">Yeni Şifre Belirleme</h2>

<div class="auth-form-container">
  <div id="reset-message-container">
    {#if isSuccess}
      <div class="auth-success">
        Şifreniz başarıyla güncellendi! Giriş sayfasına yönlendiriliyorsunuz...
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
        class:form-group--error={formError && errorMsg.includes("Şifre")}
        data-error={errorMsg}
      >
        <input
          type={showPassword ? "text" : "password"}
          id="new-password"
          bind:value={password}
          required
          placeholder=" "
        />
        <label class="form-label" for="new-password"
          >Yeni Şifre<span class="form-required-mark">*</span></label
        >
        <button
          type="button"
          class="btn-toggle-password"
          onclick={() => (showPassword = !showPassword)}
        >
          {#if showPassword}Gizle{:else}Göster{/if}
        </button>
      </div>

      <div
        class="form-group form-group--floating"
        class:form-group--error={formError && errorMsg.includes("uyuş")}
        data-error={errorMsg}
      >
        <input
          type={showRepeatPassword ? "text" : "password"}
          id="repeat-password"
          bind:value={repeatPassword}
          required
          placeholder=" "
        />
        <label class="form-label" for="repeat-password"
          >Yeni Şifre (Tekrar)<span class="form-required-mark">*</span></label
        >
        <button
          type="button"
          class="btn-toggle-password"
          onclick={() => (showRepeatPassword = !showRepeatPassword)}
        >
          {#if showRepeatPassword}Gizle{:else}Göster{/if}
        </button>
      </div>

      {#if errorMsg && formError}
        <div class="auth-error u-mb-md">{errorMsg}</div>
      {/if}

      <button
        type="submit"
        class="btn btn--primary auth-submit"
        disabled={isLoading || !token}
      >
        {isLoading ? "Güncelleniyor..." : "Şifremi Güncelle"}
      </button>
    </form>
  {/if}

  <div class="auth-footer">
    <div class="auth-footer__links">
      <a href="/giris" class="auth-footer__link" data-link
        >Giriş sayfasına dön</a
      >
    </div>
  </div>
</div>
