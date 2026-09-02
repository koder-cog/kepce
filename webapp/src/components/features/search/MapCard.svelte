<script>
  import { icon } from "@/components/ui/icons.js";

  let {
    lat = 0,
    lon = 0,
    zoom = 13,
    isExpanded = $bindable(false),
    title = "Harita",
  } = $props();

  let googleEmbedUrl = $derived(
    lat && lon
      ? `https://maps.google.com/maps?q=${lat},${lon}&hl=tr&z=${zoom}&output=embed`
      : ""
  );

  let googleMapsUrl = $derived(
    lat && lon
      ? `https://www.google.com/maps/search/?api=1&query=${lat},${lon}`
      : ""
  );

  function toggleExpand(e) {
    e.stopPropagation();
    isExpanded = !isExpanded;
  }
</script>

<div
  class="c-knowledge-tile c-knowledge-tile--map"
  class:is-expanded={isExpanded}
  role="region"
  aria-label="Google Harita Görünümü"
>
  {#if googleEmbedUrl}
    <iframe
      src={googleEmbedUrl}
      title="{title} Google Harita"
      class="c-map-iframe"
      loading="lazy"
      referrerpolicy="no-referrer"
      sandbox="allow-scripts allow-same-origin allow-popups"
    ></iframe>
  {/if}

  <!-- Sağ Üst: Genişlet / Daralt (Morfoz) Butonu -->
  <div class="c-map-ctrl-top-right">
    <button
      type="button"
      class="c-map-action-btn c-map-action-btn--expand"
      onclick={toggleExpand}
      title={isExpanded ? "Haritayı Daralt" : "Haritayı Genişlet"}
      aria-label={isExpanded ? "Haritayı Daralt" : "Haritayı Genişlet"}
    >
      {#if isExpanded}
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M4 14h6v6M20 10h-6V4M14 10l7-7M10 14L3 21" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" />
        </svg>
      {/if}
    </button>
  </div>

  <!-- Sağ Alt: Dış Bağlantı (Google Haritalar) Butonu -->
  <div class="c-map-ctrl-bottom-right">
    {#if googleMapsUrl}
      <a
        href={googleMapsUrl}
        target="_blank"
        rel="noopener noreferrer"
        class="c-map-action-btn"
        title="Google Haritalar'da Aç"
        aria-label="Google Haritalar'da Aç"
      >
        {@html icon("externalLink", 14)}
      </a>
    {/if}
  </div>
</div>
