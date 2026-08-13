<script>
  import { onMount } from 'svelte';
  import { api } from '@/api/index.js';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import Loader from '@/components/ui/Loader.svelte';
  import { icon } from '@/components/ui/icons.js';
  import { showToast } from '@/components/ui/toast.js';
  import { createModal } from '@/components/features/modal.js';
  import Dropdown from '@/components/features/Dropdown.svelte';
  import Modal from '@/components/ui/Modal.svelte';

  let isTagsLoading = $state(true);
  let tagsData = $state([]);
  let tagsError = $state(null);
  let groupedTags = $derived(tagsData.reduce((acc, t) => {
    if (!acc[t.category]) acc[t.category] = [];
    acc[t.category].push(t);
    return acc;
  }, {}));

  async function loadTags() {
    isTagsLoading = true;
    tagsError = null;
    try {
      tagsData = await api.getTags();
    } catch (err) {
      tagsError = err.message || 'Etiketler yüklenemedi.';
    } finally {
      isTagsLoading = false;
    }
  }

  onMount(() => {
    loadTags();
  });

  let isTagModalOpen = $state(false);
  let activeTagId = $state(null);
  let tagFormState = $state({
    name: '',
    category: 'sentiment',
    sort_order: 0
  });

  function handleTagModal(tag = null) {
    activeTagId = tag ? tag.id : null;
    tagFormState = {
      name: tag ? tag.name : '',
      category: tag ? tag.category : 'sentiment',
      sort_order: tag ? tag.sort_order : 0
    };
    isTagModalOpen = true;
  }

  async function submitTag() {
    try {
      if (activeTagId) {
        await api.updateTag(activeTagId, tagFormState);
      } else {
        await api.createTag(tagFormState);
      }
      showToast('Etiket kaydedildi.');
      isTagModalOpen = false;
      loadTags();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  function handleDeleteTag(id) {
    createModal({
      title: 'Etiketi Sil',
      iconHtml: icon('trash', 24),
      iconColor: 'danger',
      contentHtml: '<p>Bu etiketi silmek istediğine emin misin?</p>',
      buttons: [
        { label: 'İptal', variant: 'secondary' },
        {
          label: 'Sil', variant: 'danger', onClick: async () => {
            try {
              await api.deleteTag(id);
              showToast('Etiket silindi.');
              loadTags();
            } catch (err) {
              showToast(err.message, 'error');
            }
          }
        }
      ]
    });
  }
</script>

<svelte:head>
  <title>Etiketler - Moderasyon - Kepçe</title>
</svelte:head>

{#if isTagsLoading}
  <div class="stats-placeholder">
    <Loader size={48} />
  </div>
{:else if tagsError}
  <EmptyState statusCode={500} desc={tagsError} />
{:else}
  <div class="card">
    <div class="card__header u-flex u-justify-between u-items-center">
      <div>
        <h3 class="card__title">Etiket Yönetimi</h3>
        <p class="u-color-muted u-text-sm">Sistemin kullandığı tüm dinamik etiketler burada. Dikkatli düzenle.</p>
      </div>
      <button class="btn btn--primary btn--sm" onclick={() => handleTagModal()}>{@html icon('plus', 16)} Yeni Etiket</button>
    </div>
    <div class="card__body">
      {#each Object.entries(groupedTags) as [cat, list]}
        <div class="u-mb-lg">
          <h5 class="u-mb-sm u-text-bold u-text-capitalize">{cat} Kategorisi</h5>
          <div class="admin-table-wrapper">
            <table class="admin-table">
              <thead>
                <tr>
                  <th>Etiket</th>
                  <th>Sıra (Sort Order)</th>
                  <th class="col-actions">İşlemler</th>
                </tr>
              </thead>
              <tbody>
                {#each list as tag (tag.id)}
                  <tr>
                    <td><strong>{tag.name}</strong></td>
                    <td>{tag.sort_order}</td>
                    <td class="col-actions">
                      <div class="u-flex u-gap-sm u-justify-end">
                        <button class="btn btn--xs btn--secondary edit-tag" onclick={() => handleTagModal(tag)}>Düzenle</button>
                        <button class="btn btn--xs btn--primary delete-tag" onclick={() => handleDeleteTag(tag.id)}>Sil</button>
                      </div>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}

{#if isTagModalOpen}
<Modal options={{ title: activeTagId ? 'Etiketi Düzenle' : 'Yeni Etiket Ekle', iconHtml: icon('tag', 24) }} onClose={() => (isTagModalOpen = false)}>
  {#snippet children()}
    <div class="form-group form-group--floating u-mb-md">
      <input id="tag-name" type="text" class="form-input" placeholder=" " bind:value={tagFormState.name}>
      <label for="tag-name" class="form-label">Etiket İsmi</label>
    </div>
    <div class="form-group u-mb-md">
      <div class="u-display-block u-mb-xs u-text-sm u-color-muted">Kategori</div>
      <Dropdown
        options={[
          { value: 'sentiment', label: 'Duygu (Sentiment)' },
          { value: 'content', label: 'İçerik' },
          { value: 'dietary', label: 'Diyet' }
        ]}
        bind:value={tagFormState.category}
      />
    </div>
    <div class="form-group form-group--floating">
      <input id="tag-sort-order" type="number" class="form-input" placeholder=" " bind:value={tagFormState.sort_order}>
      <label for="tag-sort-order" class="form-label">Sıralama (Sort Order)</label>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isTagModalOpen = false}>İptal</button>
    <button class="btn btn--primary" onclick={submitTag}>Kaydet</button>
  {/snippet}
</Modal>
{/if}
