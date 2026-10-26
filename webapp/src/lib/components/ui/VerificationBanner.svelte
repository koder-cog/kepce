<script>
  import { goto } from '$app/navigation';
  import { globalState } from '$lib/stores/appState.svelte.js';
  import AnnouncementBanner from './AnnouncementBanner.svelte';

  function handleNavigateSettings() {
    goto('/ayarlar');
  }
</script>

{#if globalState.user && !globalState.user.is_verified}
  <AnnouncementBanner
    id={`unverified-banner-${globalState.user.id}`}
    text="Kurtarma kodunu teyit edin"
    ctaText="Teyit Et"
    theme="accent-primary"
    onCtaClick={handleNavigateSettings}
  />
{/if}
