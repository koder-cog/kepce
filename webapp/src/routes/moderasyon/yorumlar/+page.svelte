<script>
  import { onMount } from 'svelte';
  import { api } from '@/api/index.js';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import Loader from '@/components/ui/Loader.svelte';
  import { icon } from '@/components/ui/icons.js';
  import { sanitizeText } from '@/utils/sanitize.js';
  import { showToast } from '@/components/ui/toast.js';
  import { createModal } from '@/components/features/modal.js';

  let searchQuery = $state('');
  let isLoading = $state(true);
  let comments = $state([]);
  let errorMsg = $state(null);
  let currentPage = $state(1);
  const limit = 20;
  let offset = $state(0);
  let hasMore = $state(false);
  let isLoadingMore = $state(false);

  let searchTimeout;

  async function fetchComments(query = '', isLoadMore = false) {
    if (!isLoadMore) {
        isLoading = true;
        offset = 0;
        comments = [];
    } else {
        isLoadingMore = true;
    }
    errorMsg = null;
    
    try {
      let res = await api.getAllVotes(query, limit, offset);
      let newVotes = Array.isArray(res) ? res : (res?.data || []);
      newVotes = newVotes.filter(v => v.sentiment !== 'report');
      
      if (isLoadMore) {
          comments = [...comments, ...newVotes];
      } else {
          comments = newVotes;
      }
      
      const total = res?.total !== undefined ? res.total : 0;
      hasMore = comments.length < total && newVotes.length > 0;
      
    } catch (err) {
      errorMsg = err.message || 'Yorumlar yüklenirken bir hata oluştu.';
    } finally {
      isLoading = false;
      isLoadingMore = false;
    }
  }

  onMount(() => {
    fetchComments();
  });

  function handleSearchInput(e) {
    searchQuery = e.target.value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      fetchComments(searchQuery.trim());
    }, 300);
  }

  async function approveComment(id) {
    try {
      await api.approveVote(id);
      showToast('Yorum yayına alındı.', 'success');
      fetchComments(searchQuery.trim());
    } catch (err) { showToast(err.message, 'error'); }
  }

  async function rejectComment(id) {
    try {
      await api.rejectVote(id);
      showToast('Yorum yayından kaldırıldı.');
      fetchComments(searchQuery.trim());
    } catch (err) { showToast(err.message, 'error'); }
  }

  function purgeComment(id) {
    createModal({
      title: 'Kalıcı Silme Onayı',
      iconHtml: icon('alert', 24),
      iconColor: 'danger',
      contentHtml: '<p>Bu yorum veritabanından tamamen silinecek. Emin misin?</p>',
      buttons: [
        { label: 'İptal', variant: 'secondary' },
        {
          label: 'Evet, Kalıcı Olarak Sil',
          variant: 'danger',
          onClick: async (close) => {
            try {
              await api.purgeVote(id);
              showToast('Yorum kalıcı olarak silindi.', 'danger');
              fetchComments(searchQuery.trim());
              close();
            } catch (err) { showToast(err.message); }
          }
        }
      ]
    });
  }

  function translateStatus(status, isDeleted) {
    if (isDeleted) return 'kaldırıldı';
    return status === 'published' ? 'yayımda' : status;
  }

  function translateSentiment(sentiment) {
    const map = {
      positive: 'olumlu',
      negative: 'olumsuz',
      warning: 'uyarı'
    };
    return map[sentiment] || sentiment;
  }
</script>

<svelte:head>
  <title>Yorumlar - Moderasyon - Kepçe</title>
</svelte:head>

<div class="u-flex u-items-center u-justify-between u-mb-md u-gap-md u-mt-md">
  <div class="admin-search-bar u-flex-grow">
    <span class="admin-search-bar__icon">
      {@html icon('search', 16)}
    </span>
    <input 
      type="text" 
      class="admin-search-bar__input u-text-base" 
      placeholder="Yorum veya kullanıcı ara..." 
      autocomplete="off"
      value={searchQuery}
      oninput={handleSearchInput}
    >
  </div>
</div>

<div id="comment-list-container" class="u-mt-md">
  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState statusCode={500} desc={errorMsg} />
  {:else if comments.length === 0}
    <EmptyState iconName={'check'} title={'Hiç Yorum Yok'} desc={'Sistemde henüz bir yorum bulunamadı.'} />
  {:else}
    <div class="comment-list">
      {#each comments as comment (comment.id)}
        <article class="comment-card" data-id={comment.id}>
          <header class="comment-card__header-group">
            <div class="comment-card__meta">
              <strong class="u-text-base u-color-text">{sanitizeText(comment.user?.username || 'Anonim')}</strong>
              <span class="u-color-muted u-text-sm">·</span>
              <span class="comment-card__date u-text-sm u-color-muted">{new Date(comment.created_at).toLocaleString('tr-TR')}</span>
              <span class="u-color-muted u-text-sm">·</span>
              <span class="u-text-sm u-color-muted">{comment.reaction_summary?.up || 0} beğeni</span>
            </div>
            <div class="comment-card__meta">
              <span class="comment-card__id c-link--subtle u-text-sm">#{comment.id.toString().substring(0, 8)}</span>
              <span class="u-color-muted u-text-sm">·</span>
              <span class="u-text-sm u-color-muted">{translateStatus(comment.status, comment.is_deleted)}</span>
              <span class="u-color-muted u-text-sm">·</span>
              <span class="u-text-sm u-color-muted">{translateSentiment(comment.sentiment)}</span>
            </div>
          </header>

          <div class="comment-card__body">
            <div class="comment-card__text">
              {sanitizeText(comment.comment || '')}
            </div>
          </div>

          <footer class="comment-card__actions">
            {#if comment.is_deleted}
              <button class="btn btn--sm btn--secondary" onclick={() => approveComment(comment.id)}>Yayına koy</button>
            {:else}
              <button class="btn btn--sm btn--secondary" onclick={() => rejectComment(comment.id)}>Yayından kaldır</button>
            {/if}
            <button class="btn btn--sm btn--danger" onclick={() => purgeComment(comment.id)}>Kalıcı olarak sil</button>
          </footer>
        </article>
      {/each}
    </div>
    
    {#if hasMore}
      <div class="u-text-center u-mt-md u-mb-lg">
        <button 
          class="btn btn--secondary btn--lg" 
          disabled={isLoadingMore}
          onclick={() => {
            offset += limit;
            fetchComments(searchQuery.trim(), true);
          }}
        >
          {#if isLoadingMore}
            Yükleniyor...
          {:else}
            Daha Fazla Yükle
          {/if}
        </button>
      </div>
    {/if}
  {/if}
</div>
