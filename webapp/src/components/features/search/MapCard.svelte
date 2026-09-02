<script>
  import { onMount } from "svelte";
  import { icon } from "@/components/ui/icons.js";

  let {
    lat = 0,
    lon = 0,
    zoom = 12,
    osmUrl = ""
  } = $props();

  let canvasEl = $state(null);
  let isDark = $state(false);

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
    if (!canvasEl || !lat || !lon) return;

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
    const xFrac = lon2tileFrac(lon, zoom);
    const yFrac = lat2tileFrac(lat, zoom);
    const n = 2 ** zoom;

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
        const tileUrl = `https://${sub}.basemaps.cartocdn.com/rastertiles/${styleName}/${zoom}/${wrappedX}/${ty}@2x.png`;

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

  onMount(() => {
    isDark = checkDarkMode();
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

<div class="c-knowledge-tile c-knowledge-tile--map">
  <canvas bind:this={canvasEl} class="c-map-canvas"></canvas>

  <!-- Merkez Vektör Pin -->
  <div class="c-map-marker-anchor" aria-hidden="true">
    <div class="c-map-marker-pulse"></div>
    <div class="c-map-marker-pin">
      <div class="c-map-marker-dot"></div>
    </div>
  </div>

  <!-- Sağ Alt Harita Büyütme Butonu -->
  <div class="c-knowledge-tile__map-overlay">
    <a
      href={osmUrl || `https://www.openstreetmap.org/?mlat=${lat}&mlon=${lon}#map=${zoom}/${lat}/${lon}`}
      target="_blank"
      rel="noopener noreferrer"
      class="c-knowledge-tile__map-btn"
      title="Haritada İncele"
    >
      {@html icon("externalLink", 14)}
    </a>
  </div>
</div>
