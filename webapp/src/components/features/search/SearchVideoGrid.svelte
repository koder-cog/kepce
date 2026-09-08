<script>
  import {
    getYoutubeEmbedUrl,
    getYoutubeThumbnail,
    formatUrlBreadcrumb,
  } from "$lib/search/searchHelpers.js";
  import { searchPreferences } from "@/stores/searchPreferences.svelte.js";

  let {
    results = [],
    activeVideoEmbed = $bindable(null),
  } = $props();
</script>

<div class="c-search-videos-grid">
  {#each results as item, idx}
    {@const embedUrl = getYoutubeEmbedUrl(item.url)}
    {@const ytThumb = getYoutubeThumbnail(item.url)}
    {@const thumbUrl = item.thumbnailSrc || item.thumbnail || item.imgSrc || ytThumb}
    <article class="c-search-video-card" style="--index: {idx}">
      {#if activeVideoEmbed === item.url && embedUrl}
        <div class="c-search-video-embed-wrap">
          <iframe
            src={embedUrl}
            title={item.title}
            frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
            allowfullscreen
            class="c-search-video-iframe"
          ></iframe>
        </div>
      {:else}
        <div class="c-search-video-thumb-wrap">
          {#if thumbUrl}
            <img
              src={thumbUrl}
              alt={item.title}
              class="c-search-video-thumb"
              loading="lazy"
              decoding="async"
              onerror={(e) => {
                if (ytThumb && e.currentTarget.src !== ytThumb) {
                  e.currentTarget.src = ytThumb;
                }
              }}
            />
          {/if}
          {#if embedUrl}
            <button
              type="button"
              class="c-search-video-play-btn"
              onclick={() => (activeVideoEmbed = item.url)}
              aria-label="Videoyu Oynat"
              title="Doğrudan Oynat"
            >
              ▶
            </button>
          {/if}
        </div>
      {/if}
      <div class="c-search-video-info">
        <a
          href={item.url}
          target={searchPreferences.linkTarget}
          rel={searchPreferences.linkRel}
          class="c-search-video-title"
        >
          {item.title}
        </a>
        <p class="c-search-video-desc">{item.content}</p>
        <span class="c-search-video-source"
          >{formatUrlBreadcrumb(item.url)}</span
        >
      </div>
    </article>
  {/each}
</div>
