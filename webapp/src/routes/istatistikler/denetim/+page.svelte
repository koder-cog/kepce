<script>
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import Seo from "@/components/ui/Seo.svelte";

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
      // NOTE: getModerationActivity may not support selectedTimeframe yet, but we pass it anyway.
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
</script>

<Seo
  title="Denetim İstatistikleri - Kepçe"
  description="Kepçe şeffaflık raporu, moderasyon hareketleri ve içerik denetim istatistikleri."
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

    <div class="audit-sections">
      <section class="stat-card">
        <h3 class="stats-section-title">Sistem denetim özeti</h3>
        <div class="audit-system-list">
          <div class="audit-system-item" use:actionStagger={0}>
            <div class="audit-system-item__info">
              <strong>Kaldırılan yorumlar</strong>
              <span class="u-text-xs u-color-muted"
                >Kural ihlali sebebiyle kaldırılan yorumlar</span
              >
            </div>
            <span class="audit-status-label active"
              >{(contentData.deleted_comments || 0).toLocaleString(
                "tr-TR",
              )}</span
            >
          </div>
        </div>
      </section>

      <section class="stat-card">
        <h3 class="stats-section-title">Son işlemler</h3>
        <div class="audit-system-list">
          {#if !contentData.recent_actions || contentData.recent_actions.length === 0}
            <EmptyState
              iconName={"shieldCheck"}
              title={"İşlem Yok"}
              desc={"Son zamanlarda bir denetim işlemi yapılmadı."}
            />
          {:else}
            {#each contentData.recent_actions as act, idx}
              {@const isBan = act.action_type === "ban"}
              <div class="audit-system-item" use:actionStagger={idx + 2}>
                <div class="audit-system-item__info">
                  <strong>@{sanitizeText(act.nickname)}</strong>
                  <span class="u-text-xs u-color-muted"
                    >{isBan
                      ? "Topluluk kuralları ihlali"
                      : "Şüpheli aktivite"}</span
                  >
                </div>
                <span class="audit-status-label" class:active={!isBan}>
                  <span
                    class="audit-status-dot"
                    class:audit-status-dot--error={isBan}
                    class:audit-status-dot--warning={!isBan}
                  ></span>
                  {sanitizeText(act.action)}
                </span>
              </div>
            {/each}
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>
