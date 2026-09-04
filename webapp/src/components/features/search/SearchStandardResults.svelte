<script>
  import {
    getDomain,
    getFaviconUrl,
    formatUrlBreadcrumb,
    formatDateSnippet,
    highlightQuery,
  } from "$lib/search/searchHelpers.js";
  import { searchPreferences } from "@/stores/searchPreferences.svelte.js";

  let {
    results = [],
    query = "",
    selectedResultIndex = -1,
  } = $props();
</script>

{#each results as item, idx}
  {@const favicon = getFaviconUrl(
    item.url,
    searchPreferences.faviconResolver,
  )}
  {@const dateBadge = formatDateSnippet(item.publishedDate)}
  <article
    id="search-result-{idx}"
    class="c-search-item"
    class:is-keyboard-selected={idx === selectedResultIndex}
    data-index={idx}
    style="--index: {idx}"
  >
    <a
      href={item.url}
      target={searchPreferences.linkTarget}
      rel={searchPreferences.linkRel}
      class="c-search-item__header"
    >
      {#if favicon}
        <img
          src={favicon}
          alt=""
          class="c-search-item__favicon"
          loading="lazy"
        />
      {/if}
      <span class="c-search-item__domain">{getDomain(item.url)}</span>
      <span class="c-search-item__breadcrumb"
        >{formatUrlBreadcrumb(item.url)}</span
      >
    </a>
    <a
      href={item.url}
      target={searchPreferences.linkTarget}
      rel={searchPreferences.linkRel}
      class="c-search-item__title"
    >
      {@html highlightQuery(item.title, query)}
    </a>
    <p class="c-search-item__content">
      {#if dateBadge}
        <span class="c-search-item__date-badge">{dateBadge} -</span>
      {/if}
      {@html highlightQuery(item.content, query)}
    </p>
  </article>
{/each}
