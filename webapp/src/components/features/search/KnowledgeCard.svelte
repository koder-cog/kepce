<script>
  let { infobox } = $props();

  let imgError = $state(false);

  function handleImgError() {
    imgError = true;
  }

  function getWeatherInfo(code) {
    if (code === 0) return { label: "Açık", icon: "☀️" };
    if ([1, 2, 3].includes(code)) return { label: "Bulutlu", icon: "⛅" };
    if ([45, 48].includes(code)) return { label: "Sisli", icon: "🌫️" };
    if ([51, 53, 55, 61, 63, 65, 80, 81, 82].includes(code)) return { label: "Yağmurlu", icon: "🌧️" };
    if ([71, 73, 75, 85, 86].includes(code)) return { label: "Karlı", icon: "🌨️" };
    if ([95, 96, 99].includes(code)) return { label: "Fırtına", icon: "⛈️" };
    return { label: "Açık", icon: "☀️" };
  }

  function formatDayName(dateStr) {
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString("tr-TR", { weekday: "short" });
    } catch {
      return "";
    }
  }

  let place = $derived(infobox?.placeInfo);
  let weather = $derived(place?.weather);
</script>

{#if infobox}
  <aside class="c-knowledge-card" aria-label="Bilgi Kartı">
    <!-- ── 1. ÜST VİTRİN IZGARASI (2fr Resim + 1fr Harita + 1fr Hava Durumu) ── -->
    <div class="c-knowledge-card__showcase">
      <!-- Slot 1 (2fr): Kapak Fotoğrafı -->
      <div class="c-knowledge-card__hero" class:c-knowledge-card__hero--no-img={imgError || !infobox.imgSrc}>
        {#if infobox.imgSrc && !imgError}
          <img
            src={infobox.imgSrc}
            alt={infobox.title}
            class="c-knowledge-card__img"
            loading="lazy"
            onerror={handleImgError}
          />
        {:else}
          <div class="c-knowledge-card__hero-fallback">
            <span class="c-knowledge-card__hero-icon">🏛️</span>
          </div>
        {/if}
      </div>

      <!-- Slot 2 (1fr): Harita Arayüzü -->
      {#if place?.lat && place?.lon}
        <div class="c-knowledge-card__map-card">
          <iframe
            title="Harita - {infobox.title}"
            class="c-knowledge-card__map-iframe"
            src="https://www.openstreetmap.org/export/embed.html?bbox={place.lon - 0.05}%2C{place.lat - 0.03}%2C{place.lon + 0.05}%2C{place.lat + 0.03}&layer=mapnik&marker={place.lat}%2C{place.lon}"
            loading="lazy"
            tabindex="-1"
          ></iframe>
          <a
            href="https://www.openstreetmap.org/?mlat={place.lat}&mlon={place.lon}#map=12/{place.lat}/{place.lon}"
            target="_blank"
            rel="noopener noreferrer"
            class="c-knowledge-card__map-overlay-btn"
          >
            <span>Haritada Gör</span>
            <span class="c-knowledge-card__link-icon">↗</span>
          </a>
        </div>
      {:else}
        <!-- Harita verisi yoksa yedek nitelik paneli -->
        <div class="c-knowledge-card__stat-card">
          <span class="c-knowledge-card__stat-title">Temel Bilgiler</span>
          {#if infobox.attributes && infobox.attributes.length > 0}
            <div class="c-knowledge-card__stat-list">
              {#each infobox.attributes.slice(0, 3) as a}
                <div class="c-knowledge-card__stat-item">
                  <span class="stat-label">{a.label}</span>
                  <span class="stat-value">{a.value}</span>
                </div>
              {/each}
            </div>
          {:else}
            <p class="c-knowledge-card__stat-empty">Vikipedi ve Wikidata kaydı</p>
          {/if}
        </div>
      {/if}

      <!-- Slot 3 (1fr): Hava Durumu Kartı -->
      {#if weather}
        <div class="c-knowledge-card__weather-card">
          <div class="c-knowledge-card__weather-header">
            <span class="c-knowledge-card__weather-badge">Hava Durumu</span>
            <span class="c-knowledge-card__weather-icon">
              {getWeatherInfo(weather.weatherCode).icon}
            </span>
          </div>
          <div class="c-knowledge-card__weather-temp">
            {weather.currentTemp}°C
          </div>
          <div class="c-knowledge-card__weather-cond">
            {getWeatherInfo(weather.weatherCode).label}
          </div>

          <!-- 3 Günlük Tahmin -->
          {#if weather.daily && weather.daily.length > 0}
            <div class="c-knowledge-card__forecast-row">
              {#each weather.daily as day}
                <div class="c-knowledge-card__forecast-day">
                  <span class="day-name">{formatDayName(day.date)}</span>
                  <span class="day-icon">{getWeatherInfo(day.code).icon}</span>
                  <span class="day-temp">{day.maxTemp}°</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <!-- Hava durumu yoksa ek nitelik / kaynak kartı -->
        <div class="c-knowledge-card__stat-card">
          <span class="c-knowledge-card__stat-title">Kaynak</span>
          <div class="c-knowledge-card__stat-list">
            <div class="c-knowledge-card__stat-item">
              <span class="stat-label">Motor</span>
              <span class="stat-value">{infobox.engine || "Vikipedi"}</span>
            </div>
            {#if infobox.urls && infobox.urls.length > 0}
              <div class="c-knowledge-card__stat-item">
                <span class="stat-label">Bağlantı</span>
                <span class="stat-value">{infobox.urls[0].title || "Madde"}</span>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <!-- ── 2. ALT BİLGİ YAZISI VE NİTELİKLER ─────────────────── -->
    <div class="c-knowledge-card__details">
      <div class="c-knowledge-card__header">
        <div>
          <h2 class="c-knowledge-card__title">{infobox.title}</h2>
          {#if place?.country}
            <span class="c-knowledge-card__subtitle">{place.country}</span>
          {/if}
        </div>
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

      {#if infobox.content}
        <p class="c-knowledge-card__content">{infobox.content}</p>
      {/if}

      <!-- Nitelikler -->
      {#if infobox.attributes && infobox.attributes.length > 0}
        <div class="c-knowledge-card__attrs">
          {#each infobox.attributes.slice(0, 6) as attr}
            <div class="c-knowledge-card__attr-chip">
              <span class="c-knowledge-card__attr-label">{attr.label}</span>
              <span class="c-knowledge-card__attr-value">{attr.value}</span>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Hızlı Bağlantılar -->
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
