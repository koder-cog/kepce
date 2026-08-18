<script>
  import "@/styles/pages/_auth.css";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "@/api/index.js";
  import { globalState, authActions } from "@/state.svelte.js";
  import { showToast } from "@/components/ui/toast.js";
  import Seo from "@/components/ui/Seo.svelte";

  let status = $state("loading"); // loading, success, error
  let errorMsg = $state("");

  onMount(async () => {
    const urlParams = new URLSearchParams(window.location.search);
    const token = urlParams.get("token");

    if (!token) {
      status = "error";
      errorMsg = "Geçersiz veya eksik doğrulama bağlantısı.";
      return;
    }

    try {
      await api.passwordlessLogin(token);
      await authActions.refreshUser();

      const name = globalState.user?.username;
      showToast(name ? `Hoş geldin, @${name}!` : "Giriş yapıldı!", "success");

      status = "success";
      setTimeout(() => {
        goto("/");
      }, 1500);
    } catch (err) {
      status = "error";
      errorMsg =
        err.message ||
        "Giriş bağlantısının süresi dolmuş veya geçersiz olabilir.";
    }
  });
</script>

<Seo
  title="Şifresiz Giriş - Kepçe"
  description="Kepçe şifresiz giriş bağlantısı doğrulanıyor."
  noindex={true}
/>

<div class="auth-form-container">
  <h2 class="auth-page__title u-mb-lg">Şifresiz Giriş</h2>

  <div class="c-card passwordless-card">
    {#if status === "loading"}
      <p class="u-color-muted u-mb-sm">
        Bağlantı doğrulanıyor, lütfen bekleyin...
      </p>
      <div class="spinner"></div>
    {:else if status === "success"}
      <div class="auth-success passwordless-success">
        <p class="u-font-bold u-mb-xs">Giriş Başarılı!</p>
        <p class="u-text-sm">Ana sayfaya yönlendiriliyorsunuz...</p>
      </div>
    {:else if status === "error"}
      <div class="auth-error passwordless-error" role="alert">
        {errorMsg}
      </div>
      <a href="/giris" class="btn btn--primary">Giriş Sayfasına Dön</a>
    {/if}
  </div>
</div>

<style>
  .passwordless-card {
    text-align: center;
    padding: 2rem;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto;
  }

  .passwordless-success {
    background: var(--color-success-soft);
    color: var(--color-success);
    padding: 1rem;
    border-radius: var(--radius-md);
  }

  .passwordless-error {
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
