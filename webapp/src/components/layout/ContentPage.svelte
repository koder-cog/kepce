<script>
  import "@/styles/pages/_content.css";
  /**
   * ContentPage - Görev #52 + #53
   *
   * Statik içerik sayfaları (Hakkında, SSS, Gizlilik, Koşullar vb.) için
   * standart şablon.
   */
  let {
    title,
    dateLabel = null,
    archiveLink = null,
    isArchived = false,
    children,
  } = $props();
</script>

<article class="content-page">
  <header class="content-page__header">
    <h1 class="content-page__title">{title}</h1>
    <div class="content-page__meta">
      {#if dateLabel}
        <time class="content-page__date">{dateLabel}</time>
      {/if}
      {#if archiveLink && !isArchived}
        <a href={archiveLink} class="content-page__archive-link"
          >Geçmiş sürümler</a
        >
      {/if}
      {#if isArchived && archiveLink}
        <div class="content-page__archive-notice">
          Bu bir arşivlenmiş sürümdür. <a href={archiveLink} class="text-link"
            >Tüm sürümleri görüntüleyin</a
          >.
        </div>
      {/if}
    </div>
  </header>
  <div class="content-page__body">
    {@render children()}
  </div>
</article>
