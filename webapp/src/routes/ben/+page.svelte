<script>
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { globalState, authActions } from "@/state.svelte.js";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Seo from "@/components/ui/Seo.svelte";

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

<Seo title="Ben • Kepçe" description="Kepçe kullanıcı profiliniz, favorileriniz ve ayarlarınız." />

<div class="ben-container">
  {#if checked && !globalState.user}
    <EmptyState
      title="Oturum Açın"
      desc="Yorum yapmak, menüleri puanlamak, favori yemeklerinizi kaydetmek ve rozetler kazanmak için Kepçe hesabınıza giriş yapın."
      iconName="user"
    >
      <div class="ben-actions">
        <a href="/giris" class="btn btn--primary">Giriş Yap</a>
        <a href="/kayit" class="btn btn--secondary">Hesap Oluştur</a>
      </div>
    </EmptyState>
  {/if}
</div>

<style>
  .ben-container {
    padding: var(--space-xl) var(--space-md);
    min-height: 65vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .ben-actions {
    display: flex;
    gap: var(--space-md);
    justify-content: center;
    margin-top: var(--space-md);
    flex-wrap: wrap;
  }
</style>
