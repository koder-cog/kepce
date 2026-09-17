<script>
  import { onMount } from 'svelte';
  import { api } from '@/api/index.js';
  import Loader from '@/components/ui/Loader.svelte';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import { showToast } from '@/components/ui/toast.js';
  import { icon } from '@/components/ui/icons.js';
  import Dropdown from '@/components/features/Dropdown.svelte';

  let submissions = $state([]);
  let isLoading = $state(true);
  let statusFilter = $state('pending');
  let isUpdating = $state(null);

  const STATUS_OPTIONS = [
    { label: 'Onay Bekleyenler', value: 'pending' },
    { label: 'Onaylananlar', value: 'approved' },
    { label: 'Reddedilenler', value: 'rejected' },
    { label: 'Tümü', value: '' },
  ];

  async function fetchSubmissions() {
    isLoading = true;
    try {
      submissions = await api.getSubmissions(statusFilter);
    } catch (err) {
      showToast(err.message || 'Gönderimler yüklenemedi.', 'error');
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    fetchSubmissions();
  });

  function handleFilterChange(newVal) {
    statusFilter = newVal;
    fetchSubmissions();
  }

  async function updateStatus(id, newStatus) {
    isUpdating = id;
    try {
      await api.updateSubmissionStatus(id, newStatus);
      showToast(newStatus === 'approved' ? 'Gönderim onaylandı.' : 'Gönderim reddedildi.');
      fetchSubmissions();
    } catch (err) {
      showToast(err.message || 'İşlem başarısız.', 'error');
    } finally {
      isUpdating = null;
    }
  }

  async function handleBanUser(userId, username) {
    if (!confirm(`${username} adlı kullanıcıyı yasaklamak istediğinizden emin misiniz?`)) {
      return;
    }
    try {
      await api.banUser(userId);
      showToast(`${username} kullanıcısı yasaklandı.`, 'danger');
      fetchSubmissions();
    } catch (err) {
      showToast(err.message || 'Kullanıcı yasaklanamadı.', 'error');
    }
  }
</script>

<svelte:head>
  <title>Menü Gönderimleri - Moderasyon - Kepçe</title>
</svelte:head>

<div class="admin-filter-bar u-mb-md">
  <div class="admin-filter-grid-4">
    <div class="dev-filter-group">
      <span class="admin-filter-label">Durum Filtresi</span>
      <Dropdown
        options={STATUS_OPTIONS}
        bind:value={statusFilter}
        onChange={handleFilterChange}
      />
    </div>
  </div>
</div>

{#if isLoading}
  <div class="stats-placeholder">
    <Loader size={48} />
  </div>
{:else if submissions.length === 0}
  <EmptyState
    iconName="menu"
    title="Gönderim Bulunamadı"
    desc="Seçilen filtreye uygun menü gönderimi bulunmuyor."
  />
{:else}
  <div class="admin-table-wrapper">
    <table class="admin-table admin-table--hybrid">
      <thead>
        <tr>
          <th>ID</th>
          <th>Gönderen</th>
          <th>Şehir</th>
          <th>Dönem</th>
          <th>Dosyalar & Not</th>
          <th>Tarih</th>
          <th>Durum</th>
          <th class="col-actions">Aksiyonlar</th>
        </tr>
      </thead>
      <tbody>
        {#each submissions as sub (sub.id)}
          <tr class={sub.user_is_banned ? 'u-opacity-50' : ''}>
            <td>#{sub.id}</td>
            <td>
              {#if sub.username}
                <div class="u-flex u-flex-col u-gap-2xs">
                  <span class="u-font-bold">{sub.username}</span>
                  {#if sub.user_is_banned}
                    <span class="admin-slot-dish-tag admin-slot-dish-tag--alt">Yasaklı</span>
                  {:else if sub.user_id}
                    <button
                      type="button"
                      class="u-bg-transparent u-border-none u-color-danger u-cursor-pointer u-text-xs u-p-0 u-text-left"
                      onclick={() => handleBanUser(sub.user_id, sub.username)}
                    >
                      Kullanıcıyı Yasakla
                    </button>
                  {/if}
                </div>
              {:else}
                <span class="u-color-muted">Misafir (Anonim)</span>
              {/if}
            </td>
            <td><strong>{sub.city_slug}</strong></td>
            <td>{sub.month}/{sub.year}</td>
            <td class="u-max-w-xs u-truncate" title={sub.notes || ''}>
              {sub.notes || '-'}
            </td>
            <td>
              {sub.created_at ? new Date(sub.created_at).toLocaleDateString('tr-TR') : '-'}
            </td>
            <td>
              {#if sub.status === 'pending'}
                <span class="admin-slot-dish-tag admin-slot-dish-tag--alt">Bekliyor</span>
              {:else if sub.status === 'approved'}
                <span class="admin-slot-dish-tag">Onaylandı</span>
              {:else}
                <span class="admin-slot-dish-tag admin-slot-dish-tag--alt">Reddedildi</span>
              {/if}
            </td>
            <td class="col-actions">
              <div class="u-flex u-items-center u-gap-xs">
                {#if sub.status !== 'approved'}
                  <button
                    type="button"
                    class="btn btn--2xs btn--primary btn--squish"
                    disabled={isUpdating === sub.id}
                    onclick={() => updateStatus(sub.id, 'approved')}
                  >
                    Onayla
                  </button>
                {/if}
                {#if sub.status !== 'rejected'}
                  <button
                    type="button"
                    class="btn btn--2xs btn--secondary btn--squish"
                    disabled={isUpdating === sub.id}
                    onclick={() => updateStatus(sub.id, 'rejected')}
                  >
                    Reddet
                  </button>
                {/if}
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
