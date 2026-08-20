<script>
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import { getCommentContextHtml } from "@/utils/turkish.js";
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
      const data = await api.getGlobalTopComments(15, selectedTimeframe);
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
  title="En Beğenilen Öğrenci Yorumları - Kepçe"
  description="KYK yemekhaneleri hakkında en çok oy alan, öne çıkan öğrenci yorumları ve değerlendirmeleri."
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
    {#if contentData.length === 0}
      <EmptyState
        iconName={"chat"}
        title={"Yorum Bulunamadı"}
        desc={"Henüz bu kategoride gösterilecek bir yorum bulunmuyor."}
      />
    {:else}
      <div class="comments-stats-container">
        <div class="comments-grid">
          {#each contentData as c, idx}
            {@const dateStr = c.created_at
              ? new Date(c.created_at).toLocaleDateString("tr-TR", {
                  day: "numeric",
                  month: "long",
                  year: "numeric",
                  hour: "2-digit",
                  minute: "2-digit",
                })
              : ""}
            {@const netScore =
              c.net_score !== undefined
                ? c.net_score
                : (c.reaction_summary?.up || 0) -
                  (c.reaction_summary?.down || 0)}
            <div class="comment-card" use:actionStagger={idx}>
              <div class="comment-card__main-col">
                <div class="comment-card__header">
                  <div class="comment-card__meta">
                    <span class="comment-card__action-text"
                      >{@html getCommentContextHtml(c)}</span
                    >
                    <span class="comment-card__dot">·</span>
                    <span class="comment-card__date">{dateStr}</span>
                  </div>
                </div>
                <div class="comment-card__body">
                  <p class="comment-card__text">{sanitizeText(c.comment)}</p>
                </div>
                <div class="comment-node__actions">
                  <div class="comment-node__vote">
                    <button class="vote-btn" data-vote="up" title="Beğeni">
                      {@html icon("voteUp", 16)}
                    </button>
                    <span
                      class="vote-count"
                      class:positive={netScore > 0}
                      class:negative={netScore < 0}
                    >
                      {netScore}
                    </span>
                    <button class="vote-btn" data-vote="down" title="Beğenmeme">
                      {@html icon("voteDown", 16)}
                    </button>
                  </div>
                  <a class="action-btn" href="/menu/{c.menu_id}/{c.id}"
                    >Yanıtla</a
                  >
                  <button
                    class="action-btn"
                    onclick={() => {
                      navigator.clipboard.writeText(
                        window.location.origin +
                          "/menu/" +
                          c.menu_id +
                          "/" +
                          c.id,
                      );
                      window.showToast?.(
                        "Yorum bağlantısı panoya kopyalandı.",
                        "success",
                      );
                    }}>Paylaş</button
                  >
                  <button
                    class="action-btn"
                    onclick={() =>
                      window.showToast?.(
                        "İstatistikler sayfasından şikayet işlemi yapılamaz.",
                        "error",
                      )}>Şikayet</button
                  >
                  <button
                    class="action-btn"
                    onclick={async () => {
                      if (
                        confirm("Bu yorumu silmek istediğinize emin misiniz?")
                      ) {
                        await api.deleteComment(c.id);
                        location.reload();
                      }
                    }}>Sil</button
                  >
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>
