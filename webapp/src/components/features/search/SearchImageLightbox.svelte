<script>
  import { formatUrlBreadcrumb } from "$lib/search/searchHelpers.js";
  import { searchPreferences } from "@/stores/searchPreferences.svelte.js";

  let {
    isOpen = false,
    image = null,
    onClose = () => {},
  } = $props();
</script>

{#if isOpen && image}
  <div
    class="c-search-lightbox-backdrop"
    onclick={onClose}
    onkeydown={(e) => e.key === "Escape" && onClose()}
    role="presentation"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="c-search-lightbox-content"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Görsel Önizleme"
      tabindex="-1"
    >
      <button
        type="button"
        class="c-search-lightbox-close"
        onclick={onClose}
        aria-label="Kapat"
        title="Kapat (Esc)"
      >
        ✕
      </button>

      <div class="c-search-lightbox-img-wrap">
        <img
          src={image.imgSrc || image.thumbnail}
          alt={image.title}
          class="c-search-lightbox-img"
        />
      </div>

      <div class="c-search-lightbox-sidebar">
        <h3 class="c-search-lightbox-title">{image.title}</h3>
        <p class="c-search-lightbox-domain">
          {formatUrlBreadcrumb(image.url)}
        </p>

        <div class="c-search-lightbox-actions">
          <a
            href={image.url}
            target={searchPreferences.linkTarget}
            rel={searchPreferences.linkRel}
            class="btn btn--primary btn--block"
          >
            Sayfayı Ziyaret Et
          </a>
          {#if image.imgSrc}
            <a
              href={image.imgSrc}
              target="_blank"
              rel="noopener noreferrer"
              class="btn btn--secondary btn--block"
            >
              Tam Boyut Görsel
            </a>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
