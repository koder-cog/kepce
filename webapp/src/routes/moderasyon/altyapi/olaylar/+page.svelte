<script>
  import { onMount } from 'svelte';
  import { api } from '@/api/index.js';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import Loader from '@/components/ui/Loader.svelte';
  import { icon } from '@/components/ui/icons.js';
  import * as ui from '@/components/ui/forms.js';
  import { showToast } from '@/components/ui/toast.js';
  import { createModal } from '@/components/features/modal.js';
  import Dropdown from '@/components/features/Dropdown.svelte';
  import Modal from '@/components/ui/Modal.svelte';

  import Pagination from '@/components/ui/Pagination.svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  let isIncidentsLoading = $state(true);
  let incidentsData = $state([]);
  let incidentsError = $state(null);
  let currentLoadToken = 0;
  
  let activeIncidents = $derived(incidentsData.filter(i => (i.status || i.durum) !== 'resolved'));
  let pastIncidents = $derived(incidentsData.filter(i => (i.status || i.durum) === 'resolved'));

  // Pagination for past incidents
  let limit = 20;
  let urlPage = $derived(parseInt($page.url.searchParams.get("sayfa") || "1", 10) || 1);
  let currentPage = $state(1);
  let totalItems = $derived(pastIncidents.length);
  let totalPages = $derived(Math.ceil(totalItems / limit) || 1);

  $effect(() => {
    currentPage = Math.max(1, Math.min(urlPage, totalPages));
  });

  let paginatedPastIncidents = $derived.by(() => {
    const start = (currentPage - 1) * limit;
    return pastIncidents.slice(start, start + limit);
  });

  function handlePageChange(newPage) {
    currentPage = newPage;
    const url = new URL(window.location.href);
    if (newPage > 1) {
      url.searchParams.set("sayfa", String(newPage));
    } else {
      url.searchParams.delete("sayfa");
    }
    goto(url.pathname + url.search, { keepFocus: true, noScroll: false });
  }

  async function loadIncidents() {
    isIncidentsLoading = true;
    incidentsError = null;
    const token = ++currentLoadToken;
    try {
      const data = await api.getIncidents();
      if (token !== currentLoadToken) return;
      incidentsData = data;
    } catch (err) {
      if (token !== currentLoadToken) return;
      incidentsError = err.message || 'Olaylar yüklenemedi.';
    } finally {
      if (token === currentLoadToken) {
        isIncidentsLoading = false;
      }
    }
  }

  onMount(() => {
    loadIncidents();
  });

  let isIncidentModalOpen = $state(false);
  let newIncidentState = $state({
    component: 'API Sunucusu',
    title: '',
    message: '',
    impact: 'yavas'
  });

  function handleIncidentModal() {
    newIncidentState = {
      component: 'API Sunucusu',
      title: '',
      message: '',
      impact: 'yavas'
    };
    isIncidentModalOpen = true;
  }

  async function submitIncident() {
    try {
      await api.createIncident(newIncidentState);
      showToast('Olay başarıyla bildirildi.');
      isIncidentModalOpen = false;
      loadIncidents();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  async function handleResolveIncident(incident) {
    try {
      await api.updateIncident(incident.id, { status: 'resolved', resolved_at: new Date().toISOString() });
      showToast('Olay çözüldü olarak işaretlendi.', {
        type: 'success',
        timeout: 5000,
        action: {
          text: 'Geri Al',
          callback: async () => {
            try {
              await api.updateIncident(incident.id, { status: 'pending', resolved_at: null });
              showToast('İşlem geri alındı, olay yeniden açıldı.', 'success');
              loadIncidents();
            } catch (err) {
              showToast(err.message, 'error');
            }
          }
        }
      });
      loadIncidents();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  async function handleReopenIncident(incident) {
    try {
      await api.updateIncident(incident.id, { status: 'pending', resolved_at: null });
      showToast('Olay yeniden açıldı.', 'success');
      loadIncidents();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  function handleDeleteIncident(incident) {
    createModal({
      title: 'Olayı Sil',
      iconHtml: icon('trash', 24),
      iconColor: 'danger',
      contentHtml: '<p class="u-mb-0">Bu olayı silmek istediğinize emin misiniz? Bu işlem <strong>geri alınamaz</strong>.</p>',
      buttons: [
        { label: 'İptal', variant: 'secondary' },
        {
          label: 'Sil',
          variant: 'danger',
          onClick: async () => {
            try {
              await api.deleteIncident(incident.id);
              showToast('Olay başarıyla silindi.', 'success');
              loadIncidents();
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
  <title>Sistem Olayları - Moderasyon - Kepçe</title>
</svelte:head>

<div>
  {#if isIncidentsLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if incidentsError}
    <EmptyState statusCode={500} desc={incidentsError} />
  {:else}
    <!-- Sayfa Başı Aksiyon Çubuğu -->
    <div class="admin-top-action-bar u-flex u-items-center u-justify-between u-mb-lg u-gap-sm">
      <div class="u-min-w-0">
        <h1 class="u-text-lg u-font-bold u-mb-xs">Sistem Olayları</h1>
        <p class="u-text-sm u-color-muted u-mb-0 u-flex-wrap">
          Manuel kesinti ve yavaşlık bildirimlerini buradan yönetin.
        </p>
      </div>
      <button class="btn btn--primary u-flex-shrink-0 btn-admin-top-action" onclick={() => handleIncidentModal()}>
        <span class="u-hidden-mobile">Yeni Olay Bildir</span>
        <span class="u-hidden-desktop">{@html icon("plus", 16)}</span>
      </button>
    </div>

    <h2 class="u-text-sm u-font-bold u-color-muted u-mb-sm u-mt-lg">Aktif Olaylar</h2>
    {#if activeIncidents.length === 0}
      <div class="u-color-muted u-mb-lg u-text-sm">Şu an aktif bir olay yok.</div>
    {:else}
      <div class="admin-table-wrapper admin-table-wrapper--no-scroll u-mb-lg">
        <table class="admin-table admin-table--hybrid">
          <thead>
            <tr>
              <th>Bileşen</th>
              <th>Etki</th>
              <th>Başlık</th>
              <th>Başlangıç</th>
              <th class="col-actions"></th>
            </tr>
          </thead>
          <tbody>
            {#each activeIncidents as incident (incident.id)}
              <tr>
                <td data-label="Bileşen">
                  <div class="admin-table-cell--primary">{incident.component}</div>
                </td>
                <td data-label="Etki">
                  {@html ui.createBadge({ label: ((incident.impact || '').toUpperCase()), variant: (incident.impact) === 'kesinti' ? 'danger' : 'warning', size: 'sm' })}
                </td>
                <td data-label="Başlık">
                  <div class="admin-table-cell--meta">{incident.title}</div>
                </td>
                <td data-label="Başlangıç">
                  <div class="admin-table-cell--meta">{new Date(incident.created_at || incident.start_time || incident.started_at).toLocaleString('tr-TR')}</div>
                </td>
                <td class="col-actions">
                  <div class="u-flex u-items-center u-gap-xs u-justify-end">
                    <button type="button" class="btn-icon btn-icon--danger" aria-label="Sil" title="Sil" onclick={() => handleDeleteIncident(incident)}>
                      {@html icon('trash', 16)}
                    </button>
                    <button type="button" class="btn-icon" aria-label="Çözüldü İşaretle" title="Çözüldü İşaretle" onclick={() => handleResolveIncident(incident)}>
                      {@html icon('check', 16)}
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    <div class="u-flex u-flex-justify-between u-flex-align-center u-mb-sm u-mt-lg">
      <h2 class="u-text-sm u-font-bold u-color-muted">Geçmiş Olaylar</h2>
      {#if totalPages > 1}
        <Pagination
          compact={true}
          page={currentPage}
          {totalPages}
          {totalItems}
          onPageChange={handlePageChange}
        />
      {/if}
    </div>
    {#if pastIncidents.length === 0}
      <div class="u-color-muted u-text-sm">Geçmişte yaşanmış bir olay yok.</div>
    {:else}
      <div class="admin-table-wrapper admin-table-wrapper--no-scroll">
        <table class="admin-table admin-table--hybrid">
          <thead>
            <tr>
              <th>Bileşen</th>
              <th>Başlık</th>
              <th>Başlangıç</th>
              <th>Bitiş</th>
              <th class="col-actions"></th>
            </tr>
          </thead>
          <tbody>
            {#each paginatedPastIncidents as incident (incident.id)}
              <tr>
                <td data-label="Bileşen">
                  <div class="admin-table-cell--primary">{incident.component}</div>
                </td>
                <td data-label="Başlık">
                  <div class="admin-table-cell--meta">{incident.title}</div>
                </td>
                <td data-label="Başlangıç">
                  <div class="admin-table-cell--meta">{new Date(incident.created_at || incident.start_time || incident.started_at).toLocaleString('tr-TR')}</div>
                </td>
                <td data-label="Bitiş">
                  <div class="admin-table-cell--meta">{incident.resolved_at || incident.end_time || incident.ended_at ? new Date(incident.resolved_at || incident.end_time || incident.ended_at).toLocaleString('tr-TR') : '-'}</div>
                </td>
                <td class="col-actions">
                  <div class="u-flex u-items-center u-gap-xs u-justify-end">
                    <button type="button" class="btn-icon btn-icon--danger" aria-label="Sil" title="Sil" onclick={() => handleDeleteIncident(incident)}>
                      {@html icon('trash', 16)}
                    </button>
                    <button type="button" class="btn-icon" aria-label="Yeniden Aç" title="Yeniden Aç" onclick={() => handleReopenIncident(incident)}>
                      {@html icon('refresh', 16) || icon('rotate-ccw', 16) || icon('arrow-left', 16)}
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      {#if totalPages > 1}
        <Pagination
          page={currentPage}
          {totalPages}
          {totalItems}
          onPageChange={handlePageChange}
        />
      {/if}
    {/if}
  {/if}
</div>

{#if isIncidentModalOpen}
<Modal options={{ title: 'Yeni Olay (Incident) Bildir', iconHtml: icon('alert-triangle', 24) }} onClose={() => (isIncidentModalOpen = false)}>
  {#snippet children()}
    <div class="form-group u-mb-md">
      <div class="u-display-block u-mb-xs u-text-sm u-color-muted">Etkilenen Bileşen</div>
      <Dropdown
        options={[
          { value: 'API Sunucusu', label: 'API Sunucusu' },
          { value: 'Veritabanı', label: 'Veritabanı' },
          { value: 'Botlar', label: 'Botlar' }
        ]}
        bind:value={newIncidentState.component}
      />
    </div>
    <div class="form-group form-group--floating u-mb-md">
      <input id="incident-title" type="text" class="form-input" placeholder=" " bind:value={newIncidentState.title}>
      <label for="incident-title" class="form-label">Başlık</label>
    </div>
    <div class="form-group form-group--floating u-mb-md">
      <textarea id="incident-message" class="form-input" rows="3" placeholder=" " bind:value={newIncidentState.message}></textarea>
      <label for="incident-message" class="form-label">Mesaj/Açıklama</label>
    </div>
    <div class="form-group u-mb-md">
      <div class="u-display-block u-mb-xs u-text-sm u-color-muted">Etki Seviyesi</div>
      <Dropdown
        options={[
          { value: 'yavas', label: 'Yavaş (Degraded)' },
          { value: 'kesinti', label: 'Kesinti (Outage)' }
        ]}
        bind:value={newIncidentState.impact}
      />
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isIncidentModalOpen = false}>İptal</button>
    <button class="btn btn--primary" onclick={submitIncident}>Bildir</button>
  {/snippet}
</Modal>
{/if}
