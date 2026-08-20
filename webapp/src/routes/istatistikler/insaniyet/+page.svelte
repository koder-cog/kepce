<script>
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import { onMount } from "svelte";

  let selectedTimeframe = $state("");
  let timeframes = [
    { value: "", label: "Tümü" },
    { value: "daily", label: "Dün" },
    { value: "weekly", label: "Geçen Hafta" },
    { value: "monthly", label: "Geçen Ay" },
    { value: "yearly", label: "Geçen Yıl" },
  ];

  let isLoading = $state(true);
  let errorMsg = $state(null);
  let errorCode = $state(null);
  let contentData = $state(null);
  let currentLoadToken = 0;

  onMount(() => {
    loadContent();
  });

  async function loadContent() {
    isLoading = true;
    errorMsg = null;
    contentData = null;

    const token = ++currentLoadToken;
    try {
      const data = await api.getHumanityStats(selectedTimeframe);
      if (token !== currentLoadToken) return;
      contentData = data;
    } catch (err) {
      if (token !== currentLoadToken) return;
      errorMsg = err.message || "Bir hata oluştu.";
      errorCode = err.status || 500;
    } finally {
      if (token === currentLoadToken) {
        isLoading = false;
      }
    }
  }

  function handleTimeframeChange() {
    loadContent();
  }

  function actionStagger(node, idx) {
    node.style.setProperty("--stagger-idx", idx);
  }

  function actionPulseBar(node, { width }) {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        node.style.setProperty("--width", `${width}%`);
      });
    });
    return {
      update({ width: newWidth }) {
        node.style.setProperty("--width", `${newWidth}%`);
      },
    };
  }
</script>

<Seo
  title="İnsaniyet ve Topluluk Tablosu - Kepçe"
  description="Kepçe topluluğu insaniyet metrikleri, yardımseverlik ve etkileşim istatistikleri."
  image="https://kepce.org/api/v1/public/og/page/istatistikler"
/>

<div class="c-tab-content">
  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState statusCode={errorCode} desc={errorMsg} />
  {:else if contentData}
    <div class="comments-header-actions">
      <Dropdown
        options={timeframes}
        bind:value={selectedTimeframe}
        onChange={handleTimeframeChange}
        placeholder="Zaman Aralığı"
      />
    </div>
    {@const resolvedCount = contentData?.resolved_reports || 0}
    {@const pendingCount = contentData?.pending_reports || 0}
    {@const totalReports = contentData?.total_reports || 0}
    {@const resolutionRate = contentData?.resolution_rate}
    {@const resolutionLabel =
      resolutionRate === null || resolutionRate === undefined
        ? "—"
        : `%${resolutionRate}`}
    {@const contributors = contentData?.contributors || []}

    <div class="humanity-container">
      <div class="audit-metrics-grid">
        <!-- Metric 1 -->
        <div class="audit-metric-card" use:actionStagger={0}>
          <span class="audit-metric-card__label">Çözüm oranı</span>
          <span class="audit-metric-card__value">{resolutionLabel}</span>
          <span class="audit-metric-card__desc"
            >Çözümlenen / toplam rapor oranı</span
          >
          {#if resolutionRate !== null}
            <div class="audit-progress-track">
              <div
                class="audit-progress-fill"
                use:actionPulseBar={{ width: resolutionRate }}
              ></div>
            </div>
          {/if}
        </div>
        <!-- Metric 2 -->
        <div class="audit-metric-card" use:actionStagger={1}>
          <span class="audit-metric-card__label">Çözümlenen</span>
          <span class="audit-metric-card__value"
            >{resolvedCount.toLocaleString("tr-TR")}</span
          >
          <span class="audit-metric-card__desc">Onaylanan hata bildirimi</span>
        </div>
        <!-- Metric 3 -->
        <div class="audit-metric-card" use:actionStagger={2}>
          <span class="audit-metric-card__label">İncelemede</span>
          <span
            class="audit-metric-card__value"
            class:warning={pendingCount > 0}
            >{pendingCount.toLocaleString("tr-TR")}</span
          >
          <span class="audit-metric-card__desc">Bekleyen hata bildirimi</span>
        </div>
      </div>

      {#if contributors.length === 0}
        <EmptyState
          iconName={"menuMissing"}
          title={"Yok Bişii"}
          desc={"Henüz bir açık bulan da çıkmadı."}
        />
      {:else}
        <section class="stat-card">
          <h2 class="stats-section-title">Hata çözen cengaverler</h2>
          <div class="contributor-list">
            {#each contributors as c, idx}
              <div class="contributor-card" use:actionStagger={idx + 3}>
                <div class="contributor-card__rank">{idx + 1}</div>
                <div class="contributor-card__avatar">
                  {#if c.avatar_url}
                    <img
                      src={api.getAvatarUrl(c.avatar_url)}
                      alt={sanitizeText(c.nickname)}
                    />
                  {:else}
                    {@html icon("user", 24)}
                  {/if}
                </div>
                <div class="contributor-card__info">
                  <span class="contributor-card__name"
                    >{sanitizeText(c.nickname)}</span
                  >
                </div>
                <div class="contributor-card__value">
                  {c.resolved_count} kez
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/if}
    </div>
  {/if}
</div>
