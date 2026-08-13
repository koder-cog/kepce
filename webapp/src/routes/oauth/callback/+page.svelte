<svelte:head>
  <title>Giriş Yapılıyor... - Kepçe</title>
</svelte:head>

<script>
  import { goto } from '$app/navigation';
  import { globalState, authActions } from '@/state.svelte.js';

  import Loader from '@/components/ui/Loader.svelte';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import { onMount } from 'svelte';

  let isError = $state(false);
  let errorDescription = $state('Google ile giriş başarısız oldu. Lütfen tekrar deneyin.');

  onMount(async () => {
    const urlParams = new URLSearchParams(window.location.search);
    const oauthError = urlParams.get('error');
    if (oauthError) {
      isError = true;
      if (oauthError === 'access_denied') {
        errorDescription = 'Giriş işlemi iptal edildi veya reddedildi.';
      }
      return;
    }

    const isNew = urlParams.get('is_new') === 'true';
    
    try {
      if (authActions.refreshUser) {
        await authActions.refreshUser();
      }

      if (!globalState.user) {
        isError = true;
        return;
      }

      setTimeout(() => {
        if (isNew) {
          sessionStorage.setItem('kepce_is_new_oauth', 'true');
          goto('/profili-tamamla');
        } else {
          goto('/');
        }
      }, 1000);
    } catch (err) {
      console.error('OAuth callback login check failed:', err);
      isError = true;
    }
  });
</script>

{#if isError}
  <EmptyState statusCode={400}
    title={'Geçersiz İstek'} desc={errorDescription} actionHtml={'<a href="/giris" data-link class="btn btn--primary">Giriş Sayfasına Dön</a>'} />
{:else}
  <div class="fade-in verification-container">
    <Loader size={64} />
    <div class="verification-status">
      <p class="verification-status__title">Giriş yapılıyor...</p>
      <p class="verification-status__desc">Lütfen bekleyin, yönlendiriliyorsunuz.</p>
    </div>
  </div>
{/if}
