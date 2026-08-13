<script>
  import { afterNavigate } from '$app/navigation';
  import { api } from '@/api/index.js';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import Loader from '@/components/ui/Loader.svelte';
  import { icon } from '@/components/ui/icons.js';
  import { showToast } from '@/components/ui/toast.js';
  import * as ui from '@/components/ui/forms.js';

  let isHealthLoading = $state(true);
  let healthData = $state(null);
  let healthError = $state(null);
  let verifyResults = $state(null);
  
  let incidents = $state([]);
  let isIncidentsLoading = $state(true);

  async function loadHealth() {
    isHealthLoading = true;
    healthError = null;
    verifyResults = null;
    try {
      healthData = await api.getSystemHealth();
    } catch (err) {
      healthError = err.message || 'Sistem durumu alınamadı.';
    } finally {
      isHealthLoading = false;
    }
  }

  async function loadIncidents() {
    isIncidentsLoading = true;
    try {
      incidents = await api.getIncidents();
    } catch (err) {
      console.error('Olaylar yüklenemedi:', err);
    } finally {
      isIncidentsLoading = false;
    }
  }

  afterNavigate(() => {
    loadHealth();
    loadIncidents();
  });

  async function handleVerifyIntegrity() {
    verifyResults = 'loading';
    try {
      const res = await api.verifyTree();
      verifyResults = { success: true, data: res };
    } catch (err) {
      verifyResults = { success: false, message: err.message };
    }
  }

  function getSeverityVariant(sev) {
    if (sev === 'Yüksek') return 'danger';
    if (sev === 'Orta') return 'warning';
    return 'primary';
  }

  function getStatusVariant(st) {
    if (st === 'Çözüldü') return 'success';
    if (st === 'Araştırılıyor') return 'warning';
    return 'secondary';
  }
</script>

<svelte:head>
  <title>Sistem Sağlığı - Moderasyon - Kepçe</title>
</svelte:head>

<div>
  {#if isHealthLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if healthError}
    <EmptyState statusCode={500} desc={healthError} />
  {:else if healthData}
    <!-- Sayfa Başı Aksiyon Çubuğu -->
    <div class="admin-top-action-bar u-flex u-items-center u-justify-between u-mb-lg u-gap-sm">
      <div class="u-min-w-0">
        <h1 class="u-text-lg u-font-bold u-mb-xs">Sistem Sağlığı</h1>
        <div class="u-text-sm u-color-muted u-flex u-items-center u-gap-xs u-flex-wrap">
          Sistem Durumu: 
          {@html ui.createBadge({ label: healthData.status === 'healthy' ? 'Sağlıklı' : 'Sorunlu', variant: healthData.status === 'healthy' ? 'success' : 'danger', size: 'sm' })}
        </div>
      </div>
      <button class="btn btn--primary u-flex-shrink-0 btn-admin-top-action" onclick={handleVerifyIntegrity}>
        <span class="u-hidden-mobile">Bütünlüğü doğrula</span>
        <span class="u-hidden-desktop">{@html icon("system", 16)}</span>
      </button>
    </div>

    <!-- Bütünlük Taraması Sonuçları (Geri Bildirim) -->
    <section id="verify-results">
      {#if verifyResults === 'loading'}
        <div class="stats-placeholder u-mb-lg"><Loader size={48} /></div>
      {:else if verifyResults}
        <div class="u-mb-lg">
          {#if verifyResults.success}
            {#if verifyResults.data.is_valid}
              <div class="admin-alert admin-alert--success">
                <strong>Sistem bütünlüğü onaylandı!</strong>
                Tüm {healthData.total_nodes || verifyResults.data.node_count || 0} node başarıyla doğrulandı. Hiçbir bozulma saptanmadı.
              </div>
            {:else}
              <div class="admin-alert admin-alert--error">
                <strong>Bozulma Saptandı!</strong>
                {verifyResults.data.corrupted_count ?? 0} adet node geçersiz hash değerine sahip.
                <div class="u-mt-xs">
                  <code>{@html (verifyResults.data.corrupted_hashes || []).join('<br>')}</code>
                </div>
              </div>
            {/if}
          {:else}
            <div class="admin-alert admin-alert--error">
              <strong>Hata Oluştu!</strong> {verifyResults.message}
            </div>
          {/if}
        </div>
      {/if}
    </section>

    <!-- Genel Durum (Fingerprint) -->
    <div class="admin-preview-card u-p-md u-mb-lg u-flex u-items-center u-justify-between u-flex-wrap u-gap-sm">
      <span class="u-font-bold u-text-sm">Global Fingerprint (Root Hash)</span>
      <code class="u-text-xs u-color-muted wrap-text">{healthData.global_fingerprint || '-'}</code>
    </div>

    <h2 class="u-text-sm u-font-bold u-color-muted u-mb-sm u-mt-lg">Node İstatistikleri</h2>

    <!-- Hero Total Card -->
    <div class="admin-preview-card u-p-lg u-text-center u-mb-md hero-total-card">
      <div class="u-text-xs u-color-muted u-font-bold u-mb-xs">TOPLAM AKTİF NODE</div>
      <div class="u-text-4xl u-font-black hero-total-number">{Object.values(healthData.node_counts || {}).reduce((a, b) => a + b, 0)}</div>
    </div>

    <!-- 4-Column Breakdown Grid -->
    <div class="admin-filter-grid-4 u-gap-sm u-mb-lg">
      {#each Object.entries(healthData.node_counts || {}) as [label, count]}
        <div class="admin-preview-card u-p-md u-text-center">
          <div class="u-text-xs u-color-muted u-font-bold u-mb-xs">{label.toUpperCase()}</div>
          <div class="u-text-2xl u-font-black">{count}</div>
        </div>
      {/each}
    </div>

    <!-- 5. Active headers in table format -->
    <h2 class="u-text-sm u-font-bold u-color-muted u-mb-sm u-mt-lg">Aktif Başlıklar (Heads)</h2>
    <div class="admin-table-wrapper admin-table-wrapper--no-scroll">
      <table class="admin-table admin-table--hybrid">
        <thead>
          <tr>
            <th>Key</th>
            <th>Hash</th>
            <th class="col-actions">Aksiyonlar</th>
          </tr>
        </thead>
        <tbody>
          {#if healthData.heads && healthData.heads.length > 0}
            {#each healthData.heads as head}
              <tr data-id={head.key}>
                <td><div class="admin-table-cell--primary">{head.key}</div></td>
                <td><div class="admin-table-cell--meta"><code>{head.hash.substring(0, 16)}...</code></div></td>
                <td class="col-actions">
                  <div class="u-flex u-items-center u-gap-xs u-justify-end">
                    <button type="button" class="btn-icon" aria-label="Görüntüle" title="Görüntüle">
                      {@html icon('search', 16)}
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          {:else}
            <tr>
              <td colspan="3"><div class="u-text-center u-color-muted u-p-md">Kayıt bulunamadı.</div></td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>

    <!-- 6. Active incidents -->
    <h2 class="u-text-sm u-font-bold u-color-muted u-mb-sm u-mt-lg">Aktif Olaylar</h2>
    {#if isIncidentsLoading}
      <div class="stats-placeholder u-p-md"><Loader size={32} /></div>
    {:else}
      <div class="admin-table-wrapper admin-table-wrapper--no-scroll">
        <table class="admin-table admin-table--hybrid">
          <thead>
            <tr>
              <th>Olay</th>
              <th>Önem</th>
              <th>Durum</th>
              <th class="col-actions">Aksiyonlar</th>
            </tr>
          </thead>
          <tbody>
            {#if incidents.length > 0}
              {#each incidents as incident}
                <tr data-id={incident.id}>
                  <td><div class="admin-table-cell--primary">{incident.title}</div></td>
                  <td>{@html ui.createBadge({ label: (incident.severity || incident.impact || '').toUpperCase(), variant: getSeverityVariant(incident.severity || incident.impact), size: 'sm' })}</td>
                  <td>{@html ui.createBadge({ label: incident.status, variant: getStatusVariant(incident.status), size: 'sm' })}</td>
                  <td class="col-actions">
                    <div class="u-flex u-items-center u-gap-xs u-justify-end">
                      <a href="/moderasyon/altyapi/olaylar" class="btn-icon" aria-label="İncele" title="İncele">
                        {@html icon('search', 16)}
                      </a>
                    </div>
                  </td>
                </tr>
              {/each}
            {:else}
              <tr>
                <td colspan="4"><div class="u-text-center u-color-muted u-p-md">Aktif olay bulunmamaktadır.</div></td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>

<style>
  .hero-total-card {
    background: var(--color-surface-sunken);
    border: none;
  }
  .hero-total-number {
    color: var(--color-primary);
  }
  .wrap-text {
    word-break: break-all;
    min-width: 0;
  }
</style>
