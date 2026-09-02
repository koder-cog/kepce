<script>
  import { onMount } from "svelte";
  import { icon } from "@/components/ui/icons.js";

  let {
    lat = 0,
    lon = 0,
    zoom = 12,
    isExpanded = $bindable(false),
    osmUrl = ""
  } = $props();

  let canvasEl = $state(null);
  let isDark = $state(false);

  let currentZoom = $state(12);
  let centerLat = $state(0);
  let centerLon = $state(0);

  $effect(() => {
    centerLat = lat;
    centerLon = lon;
    currentZoom = zoom;
  });

  // Sürükleme (Drag / Pan) durumu
  let isDragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;
  let dragStartLat = 0;
  let dragStartLon = 0;

  function lon2tileFrac(lonDeg, z) {
    const n = 2 ** z;
    return ((lonDeg + 180) / 360) * n;
  }

  function lat2tileFrac(latDeg, z) {
    const n = 2 ** z;
    const latRad = (latDeg * Math.PI) / 180;
    return ((1 - Math.log(Math.tan(latRad) + 1 / Math.cos(latRad)) / Math.PI) / 2) * n;
  }

  function checkDarkMode() {
    if (typeof document === "undefined") return false;
    const theme = document.documentElement.getAttribute("data-theme");
    if (theme === "dark") return true;
    if (theme === "light") return false;
    return window.matchMedia?.("(prefers-color-scheme: dark)").matches || false;
  }

  function renderMap() {
    if (!canvasEl || !centerLat || !centerLon) return;

    const ctx = canvasEl.getContext("2d");
    if (!ctx) return;

    const rect = canvasEl.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const width = rect.width || 320;
    const height = rect.height || 200;

    canvasEl.width = width * dpr;
    canvasEl.height = height * dpr;
    ctx.scale(dpr, dpr);

    const tileSize = 256;
    const z = currentZoom;
    const xFrac = lon2tileFrac(centerLon, z);
    const yFrac = lat2tileFrac(centerLat, z);
    const n = 2 ** z;

    const centerX = width / 2;
    const centerY = height / 2;
    const centerPxX = xFrac * tileSize;
    const centerPxY = yFrac * tileSize;

    const minX = Math.floor((centerPxX - centerX) / tileSize);
    const maxX = Math.floor((centerPxX + centerX) / tileSize);
    const minY = Math.floor((centerPxY - centerY) / tileSize);
    const maxY = Math.floor((centerPxY + centerY) / tileSize);

    const styleName = isDark ? "dark_all" : "voyager";
    const subdomains = ["a", "b", "c", "d"];

    for (let ty = minY; ty <= maxY; ty++) {
      if (ty < 0 || ty >= n) continue;
      for (let tx = minX; tx <= maxX; tx++) {
        const wrappedX = ((tx % n) + n) % n;
        const sub = subdomains[(wrappedX + ty) % subdomains.length];
        const tileUrl = `https://${sub}.basemaps.cartocdn.com/rastertiles/${styleName}/${z}/${wrappedX}/${ty}@2x.png`;

        const img = new Image();
        img.crossOrigin = "anonymous";
        img.src = tileUrl;
        img.onload = () => {
          const posX = tx * tileSize - centerPxX + centerX;
          const posY = ty * tileSize - centerPxY + centerY;
          ctx.drawImage(img, posX, posY, tileSize, tileSize);
        };
      }
    }
  }

  function handleMouseDown(e) {
    if (e.button !== 0) return;
    isDragging = true;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    dragStartLat = centerLat;
    dragStartLon = centerLon;
  }

  function handleMouseMove(e) {
    if (!isDragging) return;
    const dx = e.clientX - dragStartX;
    const dy = e.clientY - dragStartY;

    const tileSize = 256;
    const n = 2 ** currentZoom;
    const dLonFrac = -dx / tileSize;
    const dLatFrac = -dy / tileSize;

    const startXFrac = lon2tileFrac(dragStartLon, currentZoom);
    const startYFrac = lat2tileFrac(dragStartLat, currentZoom);

    const curXFrac = startXFrac + dLonFrac;
    const curYFrac = startYFrac + dLatFrac;

    centerLon = (curXFrac / n) * 360 - 180;
    const sinhVal = Math.sinh(Math.PI * (1 - (2 * curYFrac) / n));
    centerLat = (Math.atan(sinhVal) * 180) / Math.PI;

    renderMap();
  }

  function handleMouseUp() {
    isDragging = false;
  }

  function handleTouchStart(e) {
    if (e.touches.length !== 1) return;
    isDragging = true;
    dragStartX = e.touches[0].clientX;
    dragStartY = e.touches[0].clientY;
    dragStartLat = centerLat;
    dragStartLon = centerLon;
  }

  function handleTouchMove(e) {
    if (!isDragging || e.touches.length !== 1) return;
    const dx = e.touches[0].clientX - dragStartX;
    const dy = e.touches[0].clientY - dragStartY;

    const tileSize = 256;
    const n = 2 ** currentZoom;
    const dLonFrac = -dx / tileSize;
    const dLatFrac = -dy / tileSize;

    const startXFrac = lon2tileFrac(dragStartLon, currentZoom);
    const startYFrac = lat2tileFrac(dragStartLat, currentZoom);

    const curXFrac = startXFrac + dLonFrac;
    const curYFrac = startYFrac + dLatFrac;

    centerLon = (curXFrac / n) * 360 - 180;
    const sinhVal = Math.sinh(Math.PI * (1 - (2 * curYFrac) / n));
    centerLat = (Math.atan(sinhVal) * 180) / Math.PI;

    renderMap();
  }

  function handleTouchEnd() {
    isDragging = false;
  }

  function toggleExpand() {
    isExpanded = !isExpanded;
    setTimeout(() => {
      renderMap();
    }, 50);
  }

  function zoomIn() {
    if (currentZoom < 18) {
      currentZoom += 1;
      renderMap();
    }
  }

  function zoomOut() {
    if (currentZoom > 3) {
      currentZoom -= 1;
      renderMap();
    }
  }

  function resetCenter() {
    centerLat = lat;
    centerLon = lon;
    currentZoom = zoom;
    renderMap();
  }

  onMount(() => {
    isDark = checkDarkMode();
    centerLat = lat;
    centerLon = lon;
    currentZoom = zoom;
    renderMap();

    const observer = new MutationObserver(() => {
      const nextDark = checkDarkMode();
      if (nextDark !== isDark) {
        isDark = nextDark;
        renderMap();
      }
    });

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"]
    });

    const handleResize = () => {
      renderMap();
    };
    window.addEventListener("resize", handleResize);

    return () => {
      observer.disconnect();
      window.removeEventListener("resize", handleResize);
    };
  });
</script>

<svelte:window
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  ontouchmove={handleTouchMove}
  ontouchend={handleTouchEnd}
/>

<div
  class="c-knowledge-tile c-knowledge-tile--map"
  class:is-expanded={isExpanded}
  class:is-dragging={isDragging}
  role="region"
  aria-label="Etkileşimli Harita"
>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <canvas
    bind:this={canvasEl}
    class="c-map-canvas"
    onmousedown={handleMouseDown}
    ontouchstart={handleTouchStart}
    ondblclick={zoomIn}
  ></canvas>

  <!-- Merkez Vektör Pin -->
  <div class="c-map-marker-anchor" aria-hidden="true">
    <div class="c-map-marker-pulse"></div>
    <div class="c-map-marker-pin">
      <div class="c-map-marker-dot"></div>
    </div>
  </div>

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
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M4 14h6v6M20 10h-6V4M14 10l7-7M10 14L3 21" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" />
        </svg>
      {/if}
    </button>
  </div>

  <!-- Sağ Alt: Yakınlaştırma (+ / -) ve Dış Bağlantı Butonları -->
  <div class="c-map-ctrl-bottom-right">
    {#if isExpanded}
      <div class="c-map-zoom-group">
        <button
          type="button"
          class="c-map-zoom-btn"
          onclick={zoomIn}
          title="Yakınlaştır"
          aria-label="Yakınlaştır"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M12 5v14M5 12h14" />
          </svg>
        </button>
        <button
          type="button"
          class="c-map-zoom-btn"
          onclick={zoomOut}
          title="Uzaklaştır"
          aria-label="Uzaklaştır"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M5 12h14" />
          </svg>
        </button>
      </div>

      <button
        type="button"
        class="c-map-action-btn"
        onclick={resetCenter}
        title="Merkeze Dön"
        aria-label="Merkeze Dön"
      >
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 2v3M12 19v3M2 12h3M19 12h3" />
        </svg>
      </button>
    {/if}

    <a
      href={osmUrl || `https://www.openstreetmap.org/?mlat=${centerLat}&mlon=${centerLon}#map=${currentZoom}/${centerLat}/${centerLon}`}
      target="_blank"
      rel="noopener noreferrer"
      class="c-map-action-btn"
      title="OpenStreetMap'te Aç"
    >
      {@html icon("externalLink", 14)}
    </a>
  </div>
</div>
