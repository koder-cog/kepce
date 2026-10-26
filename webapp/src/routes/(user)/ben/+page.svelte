<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { globalState, authActions } from '$lib/stores/appState.svelte.js';
  import { icon } from '$lib/components/ui/icons.js';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Seo from '$lib/components/ui/Seo.svelte';

  let checked = $state(false);

  onMount(async () => {
    await authActions.refreshUser();
    if (globalState.user?.username) {
      goto(`/biri/${globalState.user.username}`, { replaceState: true });
    } else {
      checked = true;
    }
  });
</script>

<Seo
  title="Profilim | Kepçe"
  description="Kepçe kullanıcı profiliniz, favorileriniz ve ayarlarınız."
/>

<h1 class="sr-only">Profilim</h1>

<div class="centered-content u-p-xl">
  {#if checked && !globalState.user}
    <EmptyState
      title="Oturum Açın"
      desc="Yorum yapmak, menüleri puanlamak ve favori yemeklerinizi kaydetmek için Kepçe hesabınıza giriş yapın."
      iconName="user"
    >
      <div class="u-flex u-gap-md u-flex-justify-center u-flex-wrap u-mt-md">
        <a href="/giris" class="btn btn--primary">Giriş Yap</a>
        <a href="/kayit" class="btn btn--secondary">Hesap Oluştur</a>
      </div>
    </EmptyState>
  {/if}
</div>
