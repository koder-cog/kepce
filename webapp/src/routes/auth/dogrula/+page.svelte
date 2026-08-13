<svelte:head>
  <title>E-Posta Doğrulanıyor... - Kepçe</title>
</svelte:head>

<script>
  import "@/styles/pages/_auth.css";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "@/api/index.js";
  import { globalState, authActions } from "@/state.svelte.js";

  let status = $state("loading"); // loading, success, already_verified, error
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
      const result = await api.verifyEmail(token);
      await authActions.refreshUser();

      status = result?.status === "already_verified" ? "already_verified" : "success";
      setTimeout(() => {
        goto("/");
      }, 3000);
    } catch (err) {
      status = "error";
      errorMsg =
        err.message ||
        "Doğrulama bağlantısının süresi dolmuş veya geçersiz olabilir.";
    }
  });
</script>

<div class="auth-form-container">
  <h2 class="auth-page__title u-mb-lg">E-Posta Doğrulama</h2>

  <div class="c-card passwordless-card">
    {#if status === "loading"}
      <p class="u-color-muted u-mb-sm">
        Bağlantı doğrulanıyor, lütfen bekleyin...
      </p>
      <div class="spinner"></div>
    {:else if status === "success"}
      <div class="auth-success passwordless-success">
        <p class="u-font-bold u-mb-xs">Doğrulama Başarılı!</p>
        <p class="u-text-sm">E-posta adresiniz doğrulandı. Ana sayfaya yönlendiriliyorsunuz...</p>
      </div>
    {:else if status === "already_verified"}
      <div class="auth-info passwordless-success">
        <p class="u-font-bold u-mb-xs">Zaten Onaylısınız!</p>
        <p class="u-text-sm">E-postanız zaten onaylı, harika! Hiçbir işlem yapmanıza gerek yok. Ana sayfaya yönlendiriliyorsunuz...</p>
      </div>
    {:else if status === "error"}
      <div class="auth-error passwordless-error" role="alert">
        {errorMsg}
      </div>
      <a href="/giris" class="c-btn c-btn--primary">Giriş Sayfasına Dön</a>
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
