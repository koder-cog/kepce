<script>
  import Modal from '@/components/ui/Modal.svelte';
  import { api } from '@/api/index.js';
  import { globalState } from '@/state.svelte.js';
  import { showToast } from '@/components/ui/toast.js';
  import { goto } from '$app/navigation';

  let { onClose, onPostpone } = $props();

  let loading = $state(false);

  let remainingDays = $derived.by(() => {
    if (!globalState.user?.consent_deadline_at) return 0;
    const deadline = new Date(globalState.user.consent_deadline_at);
    const now = new Date();
    const diffTime = deadline.getTime() - now.getTime();
    return Math.max(0, Math.ceil(diffTime / (1000 * 60 * 60 * 24)));
  });

  let isExpired = $derived(remainingDays <= 0);

  async function handleAccept() {
    loading = true;
    try {
      const updatedUser = await api.giveCrossBorderConsent();
      globalState.user = {
        ...globalState.user,
        consent_cross_border: true,
        consent_cross_border_at: updatedUser.consent_cross_border_at || new Date().toISOString()
      };
      if (typeof window !== 'undefined') {
        localStorage.setItem('kepce_user_cache', JSON.stringify(globalState.user));
      }
      showToast('Yurt dışı barındırma onayınız kaydedildi.', 'success');
      onClose();
    } catch (err) {
      showToast('Onay kaydedilirken bir hata oluştu. Lütfen tekrar deneyin.', 'error');
    } finally {
      loading = false;
    }
  }

  function handlePostpone() {
    if (isExpired) return;
    if (onPostpone) {
      onPostpone();
    } else {
      onClose();
    }
  }

  function handleDeleteAccount() {
    onClose();
    goto('/ayarlar#hesap-silme');
  }
</script>

<Modal
  options={{
    title: 'Yurt Dışı Barındırma Onayı',
    disableEscape: isExpired,
    disableHistory: true,
    footerClass: 'c-modal__footer--stack'
  }}
  onClose={() => {
    if (!isExpired) {
      handlePostpone();
    }
  }}
>
  {#snippet children()}
    <p>
      Kepçe'nin sunucuları Fransa'da barınıyor. Sitedeki menüleri ve fiyatları incelemek için üyelik
      gerekmiyor ancak oy vermek, yorum yapmak veya fotoğraf yüklemek için açtığın hesabın verileri
      ile yasal trafik kayıtları mevzuat gereği teknik olarak yurt dışına aktarılmış sayılıyor.
    </p>
    <p class="u-mt-sm">
      {#if !isExpired}
        KVKK Madde 9 uyarınca muhtemel riskler hakkında bilgilendirilerek hesabını bu şekilde
        kullanabilmen için {remainingDays} gün içinde açık rıza vermen gerekiyor. Verilerin kimseye satılmaz
        veya pazarlama için kullanılmaz. Ayrıntılara ve yasal haklarına dilediğin zaman aydınlatma metninden
        bakabilirsin.
      {:else}
        KVKK Madde 9 uyarınca muhtemel riskler hakkında bilgilendirilerek hesabını bu şekilde
        kullanabilmen için açık rıza vermen gerekiyor. Verilerin kimseye satılmaz veya pazarlama
        için kullanılmaz. Ayrıntılara ve yasal haklarına dilediğin zaman aydınlatma metninden
        bakabilirsin.
      {/if}
    </p>
  {/snippet}

  {#snippet footer()}
    <button class="btn btn--primary btn--full" onclick={handleAccept} disabled={loading}>
      Onaylıyorum
    </button>
    {#if !isExpired}
      <button class="btn btn--secondary btn--full" onclick={handlePostpone} disabled={loading}>
        Sonra hatırlat
      </button>
    {/if}
    <button class="btn btn--danger btn--full" onclick={handleDeleteAccount} disabled={loading}>
      Hesabımı sil
    </button>
  {/snippet}
</Modal>
