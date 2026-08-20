<script>
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import { timeAgo } from "@/utils/date.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import PieChart from "@/components/ui/PieChart.svelte";
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
      const data = await api.getModerationActivity(selectedTimeframe);
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

  function getCategoryLabel(category) {
    switch (category) {
      case "toxicity":
        return "Hakaret";
      case "spam":
        return "Spam";
      case "misinformation":
        return "Hata";
      default:
        return "Kural Dışı";
    }
  }
</script>

<Seo
  title="Denetim İstatistikleri - Kepçe"
  description="Kepçe şeffaflık raporu, moderasyon hareketleri ve içerik denetim istatistikleri."
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
    {@const deletedComments = contentData?.deleted_comments || 0}
    {@const resolutionRate = contentData?.resolution_rate}
    {@const resolutionLabel =
      resolutionRate === null || resolutionRate === undefined
        ? "—"
        : `%${resolutionRate}`}
    {@const categories = contentData?.category_distribution || []}
    {@const recentActions = contentData?.recent_actions || []}

    <!-- Üst 4'lü Metrik Grid -->
    <div class="audit-metrics-grid">
      <!-- Metric 1: Çözüm Oranı -->
      <div class="audit-metric-card" use:actionStagger={0}>
        <span class="audit-metric-card__label">Çözüm oranı</span>
        <span class="audit-metric-card__value">{resolutionLabel}</span>
        <span class="audit-metric-card__desc">Çözümlenen / toplam şikayet oranı</span>
        {#if resolutionRate !== null}
          <div class="audit-progress-track">
            <div
              class="audit-progress-fill"
              use:actionPulseBar={{ width: resolutionRate }}
            ></div>
          </div>
        {/if}
      </div>

      <!-- Metric 2: Müdahale Edilen -->
      <div class="audit-metric-card" use:actionStagger={1}>
        <span class="audit-metric-card__label">Müdahale edilen</span>
        <span class="audit-metric-card__value">{resolvedCount.toLocaleString("tr-TR")}</span>
        <span class="audit-metric-card__desc">İşlem yapılan şikayet</span>
      </div>

      <!-- Metric 3: İncelemede -->
      <div class="audit-metric-card" use:actionStagger={2}>
        <span class="audit-metric-card__label">İncelemede</span>
        <span
          class="audit-metric-card__value"
          class:warning={pendingCount > 0}
        >
          {pendingCount.toLocaleString("tr-TR")}
        </span>
        <span class="audit-metric-card__desc">Bekleyen aktif şikayet</span>
      </div>

      <!-- Metric 4: Kaldırılan Yorumlar -->
      <div class="audit-metric-card" use:actionStagger={3}>
        <span class="audit-metric-card__label">Kaldırılan yorumlar</span>
        <span class="audit-metric-card__value">{deletedComments.toLocaleString("tr-TR")}</span>
        <span class="audit-metric-card__desc">Kural ihlali sebebiyle silinen</span>
      </div>
    </div>

    <!-- Alt İki Kolonlu Grid: Sol Pasta Grafiği / Sağ Son Moderasyon Hareketleri -->
    <div class="audit-grid-layout">
      <!-- Sol: İhlal Kategorileri Dağılımı (PieChart) -->
      <section class="stat-card" use:actionStagger={4}>
        <h3 class="stats-section-title">İhlal kategorileri dağılımı</h3>
        <PieChart data={categories} title="Şikayet" size={170} />
      </section>

      <!-- Sağ: Son Moderasyon Hareketleri -->
      <section class="stat-card" use:actionStagger={5}>
        <h3 class="stats-section-title">Son denetim hareketleri</h3>
        <div class="audit-system-list">
          {#if recentActions.length === 0}
            <EmptyState
              iconName={"shieldCheck"}
              title={"İşlem Yok"}
              desc={"Bu zaman aralığında bir denetim hareketi kaydedilmedi."}
            />
          {:else}
            {#each recentActions as act, idx}
              <div class="audit-system-item" use:actionStagger={idx + 6}>
                <div class="audit-system-item__info">
                  <span class="audit-system-item__title">{sanitizeText(act.action)}</span>
                  <div class="audit-system-item__meta">
                    <span>{timeAgo(act.created_at)}</span>
                  </div>
                </div>
                <span class="audit-system-badge audit-system-badge--{act.category || 'general'}">
                  {getCategoryLabel(act.category)}
                </span>
              </div>
            {/each}
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>
