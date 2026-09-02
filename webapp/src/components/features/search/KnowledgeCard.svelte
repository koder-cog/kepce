<script>
  import { icon } from "@/components/ui/icons.js";

  let { infobox } = $props();

  let imgError = $state(false);

  function handleImgError() {
    imgError = true;
  }
</script>

{#if infobox}
  <aside class="c-knowledge-card" aria-label="Bilgi Kartı">
    <!-- ── Görsel (Hero Image) ──────────────────────────────── -->
    {#if infobox.imgSrc && !imgError}
      <div class="c-knowledge-card__hero">
        <img
          src={infobox.imgSrc}
          alt={infobox.title}
          class="c-knowledge-card__img"
          loading="lazy"
          onerror={handleImgError}
        />
      </div>
    {/if}

    <div class="c-knowledge-card__body">
      <!-- ── Başlık ve Rozet ───────────────────────────────── -->
      <div class="c-knowledge-card__header">
        <h2 class="c-knowledge-card__title">{infobox.title}</h2>
        {#if infobox.engine}
          <span class="c-knowledge-card__badge">
            {infobox.engine === "wikipedia"
              ? "Vikipedi"
              : infobox.engine === "wikidata"
                ? "Wikidata"
                : infobox.engine}
          </span>
        {/if}
      </div>

      <!-- ── Açıklama / Özet ───────────────────────────────── -->
      {#if infobox.content}
        <p class="c-knowledge-card__content">{infobox.content}</p>
      {/if}

      <!-- ── Nitelikler Tablosu (Attributes) ────────────────── -->
      {#if infobox.attributes && infobox.attributes.length > 0}
        <div class="c-knowledge-card__attrs">
          {#each infobox.attributes.slice(0, 8) as attr}
            <div class="c-knowledge-card__attr-row">
              <span class="c-knowledge-card__attr-label">{attr.label}</span>
              <span class="c-knowledge-card__attr-value">{attr.value}</span>
            </div>
          {/each}
        </div>
      {/if}

      <!-- ── Bağlantılar / Kaynaklar ────────────────────────── -->
      {#if infobox.urls && infobox.urls.length > 0}
        <div class="c-knowledge-card__links">
          {#each infobox.urls.slice(0, 4) as link}
            <a
              href={link.url}
              target="_blank"
              rel="noopener noreferrer"
              class="c-knowledge-card__link-pill"
            >
              <span>{link.title || "Kaynak"}</span>
              <span class="c-knowledge-card__link-icon">↗</span>
            </a>
          {/each}
        </div>
      {/if}
    </div>
  </aside>
{/if}
