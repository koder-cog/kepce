<script>
  import { formatUrlBreadcrumb } from "$lib/search/searchHelpers.js";
  import { searchPreferences } from "@/stores/searchPreferences.svelte.js";

  let {
    results = [],
    onSelectImage = () => {},
  } = $props();
</script>

<div class="c-search-images-grid">
  {#each results as item, idx}
    {@const ratio =
      item.width && item.height ? `${item.width} / ${item.height}` : "4 / 3"}
    <div class="c-search-image-item" style="--index: {idx}">
      <button
        type="button"
        class="c-search-image-card"
        style="--aspect-ratio: {ratio}"
        onclick={(e) => onSelectImage(item, e)}
        aria-label={item.title}
      >
        {#if item.thumbnailSrc || item.imgSrc}
          <img
            src={item.thumbnailSrc || item.imgSrc}
            alt={item.title}
            class="c-search-image-card__img"
            loading="lazy"
            decoding="async"
            onerror={(e) => {
              e.currentTarget
                .closest(".c-search-image-item")
                ?.classList.add("is-img-error");
            }}
          />
        {/if}
      </button>
      <div class="c-search-image-meta">
        <span class="c-search-image-meta__title" title={item.title}>
          {item.title}
        </span>
        <a
          href={item.url}
          target={searchPreferences.linkTarget}
          rel={searchPreferences.linkRel}
          class="c-search-image-meta__source"
        >
          {formatUrlBreadcrumb(item.url)}
        </a>
      </div>
    </div>
  {/each}
</div>
