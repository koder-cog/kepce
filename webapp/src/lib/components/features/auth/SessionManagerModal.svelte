<script>
  import { onMount } from 'svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import { icon } from '$lib/components/ui/icons.js';
  import { api } from '$lib/api/index.js';
  import { showToast } from '$lib/components/ui/toast.js';

  let { onClose } = $props();

  let sessions = $state([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      sessions = await api.getSessions();
    } catch {
      showToast('Oturumlar yüklenemedi.', { type: 'error' });
    } finally {
      loading = false;
    }
  });

  async function handleRevoke(sessionId) {
    try {
      await api.revokeSession(sessionId);
      sessions = sessions.filter((s) => s.id !== sessionId);
      showToast('Oturum başarıyla sonlandırıldı.', { type: 'success' });
    } catch (e) {
      showToast(e.message || 'Oturum sonlandırılamadı.', { type: 'error' });
    }
  }

  function formatDate(dateStr) {
    if (!dateStr) return 'Bilinmiyor';
    try {
      return new Date(dateStr).toLocaleString('tr-TR', {
        day: 'numeric',
        month: 'short',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return dateStr;
    }
  }
</script>

<Modal options={{ title: 'Aktif oturumlar', iconHtml: icon('lockClose', 24) }} {onClose}>
  {#snippet children()}
    <div class="c-boxed-list">
      {#if loading}
        <div class="u-text-center u-p-md u-color-muted">Yükleniyor...</div>
      {:else if sessions.length === 0}
        <div class="u-text-center u-p-md u-color-muted">Aktif oturum bulunamadı.</div>
      {:else}
        {#each sessions as session (session.id)}
          <div class="c-list-row c-list-row--regular">
            <div class="c-list-row__info">
              <div class="c-list-row__title">
                {session.is_current ? 'Bu oturum' : 'Diğer oturum'}
              </div>
              <div class="c-list-row__desc">
                Son işlem: {formatDate(session.last_used_at)}
              </div>
            </div>
            {#if !session.is_current}
              <div class="c-list-row__control">
                <button
                  type="button"
                  class="btn btn--secondary btn--squish btn--danger"
                  onclick={() => handleRevoke(session.id)}
                >
                  Sonlandır
                </button>
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <button type="button" class="btn btn--secondary" onclick={onClose}>Kapat</button>
  {/snippet}
</Modal>
