<script>
  let { infobox } = $props();

  let imgError = $state(false);

  function handleImgError() {
    imgError = true;
  }

  // Hava durumu SVG ikonları
  function getWeatherInfo(code) {
    if (code === 0) return { label: "Açık", iconType: "sun" };
    if ([1, 2, 3].includes(code)) return { label: "Bulutlu", iconType: "cloudSun" };
    if ([45, 48].includes(code)) return { label: "Sisli", iconType: "fog" };
    if ([51, 53, 55, 61, 63, 65, 80, 81, 82].includes(code)) return { label: "Yağmurlu", iconType: "rain" };
    if ([71, 73, 75, 85, 86].includes(code)) return { label: "Karlı", iconType: "snow" };
    if ([95, 96, 99].includes(code)) return { label: "Fırtına", iconType: "storm" };
    return { label: "Açık", iconType: "sun" };
  }

  function formatDayName(dateStr) {
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString("tr-TR", { weekday: "short" });
    } catch {
      return "";
    }
  }

  let entityType = $derived(infobox?.entityType || "thing");
  let place = $derived(infobox?.placeInfo);
  let weather = $derived(place?.weather);
  let hasValidPlace = $derived(entityType === "place" && !!place?.lat && !!place?.lon);
  let hasValidOrgMap = $derived(entityType === "organization" && !!place?.lat && !!place?.lon);

  // Dinamik alt başlık / unvan
  let subtitle = $derived.by(() => {
    if (entityType === "place") {
      if (place?.country) return place.country;
      const match = (infobox.content || "").match(/^([^,.]+)/);
      return match ? match[1].trim() : "";
    }
    if (entityType === "person") {
      const text = infobox.content || "";
      const parts = text.split(/[,–-]/);
      if (parts.length > 1 && parts[1].trim().length > 3 && parts[1].trim().length < 60) {
        return parts[1].trim();
      }
      return "";
    }
    return "";
  });

  // En önemli 4 ila 6 özet nitelik
  let displayAttributes = $derived.by(() => {
    if (!infobox?.attributes) return [];
    return infobox.attributes.slice(0, 6);
  });
</script>

{#if infobox}
  <article class="c-knowledge-panel" aria-label="Bilgi Kartı">
    <!-- ── 1. ÜST BAŞLIK ────────────────────────────────────────── -->
    <header class="c-knowledge-header">
      <div class="c-knowledge-title-group">
        <h1 class="c-knowledge-title">{infobox.title}</h1>
        {#if subtitle}
          <p class="c-knowledge-subtitle">{subtitle}</p>
        {/if}
      </div>
    </header>

    <!-- ── 2. VİTRİN VE İÇERİK DÜZENİ ───────────────────────────── -->
    {#if hasValidPlace}
      <!-- 2A. Coğrafi Yer: 3 Parçalı Vitrin (Resim + Harita + Hava Durumu) -->
      <div class="c-knowledge-showcase">
        <!-- Fotoğraf -->
        <div class="c-knowledge-tile c-knowledge-tile--photo">
          {#if infobox.imgSrc && !imgError}
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-knowledge-tile__img"
              loading="lazy"
              onerror={handleImgError}
            />
          {:else}
            <div class="c-knowledge-tile__fallback">
              <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M3 21h18M3 10h18M5 10v11M19 10v11M9 10v11M15 10v11M12 3L2 10h20L12 3z" />
              </svg>
            </div>
          {/if}
        </div>

        <!-- Harita Karosu -->
        <div class="c-knowledge-tile c-knowledge-tile--map">
          <iframe
            title="Harita - {infobox.title}"
            class="c-knowledge-tile__map-iframe"
            src="https://www.openstreetmap.org/export/embed.html?bbox={place.lon - 0.08}%2C{place.lat - 0.04}%2C{place.lon + 0.08}%2C{place.lat + 0.04}&layer=mapnik&marker={place.lat}%2C{place.lon}"
            loading="lazy"
            tabindex="-1"
          ></iframe>
          <div class="c-knowledge-tile__map-overlay">
            <span class="c-knowledge-tile__map-pin">
              <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 21s-6-5.33-6-10a6 6 0 0 1 12 0c0 4.67-6 10-6 10z" />
                <circle cx="12" cy="11" r="2.5" />
              </svg>
              <span>{infobox.title}</span>
            </span>
            <a
              href="https://www.openstreetmap.org/?mlat={place.lat}&mlon={place.lon}#map=12/{place.lat}/{place.lon}"
              target="_blank"
              rel="noopener noreferrer"
              class="c-knowledge-tile__map-btn"
              title="Haritada Büyüt"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M7 17L17 7M7 7h10v10" />
              </svg>
            </a>
          </div>
        </div>

        <!-- Sağ İkili Yığın (Hava Durumu + Nasıl Gidilir) -->
        <div class="c-knowledge-stack">
          {#if weather}
            <div class="c-knowledge-widget c-knowledge-widget--weather">
              <div class="c-knowledge-widget__head">
                <span class="c-knowledge-widget__title">Hava durumu</span>
                <span class="c-knowledge-weather-icon">
                  {#if getWeatherInfo(weather.weatherCode).iconType === "sun"}
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="12" r="4" />
                      <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
                    </svg>
                  {:else if getWeatherInfo(weather.weatherCode).iconType === "cloudSun"}
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 2v2M4.93 4.93l1.41 1.41M2 12h2M17.66 6.34l1.41-1.41" />
                      <path d="M17.5 19H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 12 4.5 4.5 0 0 1 17.5 19z" />
                    </svg>
                  {:else if getWeatherInfo(weather.weatherCode).iconType === "rain"}
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M17.5 14H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 7 4.5 4.5 0 0 1 17.5 14z" />
                      <path d="M8 17l-1 3M12 17l-1 3M16 17l-1 3" />
                    </svg>
                  {:else if getWeatherInfo(weather.weatherCode).iconType === "snow"}
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M17.5 14H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 7 4.5 4.5 0 0 1 17.5 14z" />
                      <path d="M8 18h.01M12 18h.01M16 18h.01M10 21h.01M14 21h.01" />
                    </svg>
                  {:else if getWeatherInfo(weather.weatherCode).iconType === "storm"}
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M17.5 13H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 6 4.5 4.5 0 0 1 17.5 13z" />
                      <path d="M13 13l-3 5h4l-2 5" />
                    </svg>
                  {:else}
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M4 14h16M4 18h16M6 10h12" />
                    </svg>
                  {/if}
                </span>
              </div>
              <div class="c-knowledge-widget__temp-now">
                {weather.currentTemp}°C
                <span class="c-knowledge-widget__cond">{getWeatherInfo(weather.weatherCode).label}</span>
              </div>
              {#if weather.daily && weather.daily.length > 0}
                <div class="c-knowledge-widget__forecast">
                  {#each weather.daily as day}
                    {@const dayWeather = getWeatherInfo(day.code)}
                    <div class="c-knowledge-forecast-item">
                      <span class="f-day">{formatDayName(day.date)}</span>
                      <span class="f-icon" title={dayWeather.label}>
                        {#if dayWeather.iconType === "sun"}
                          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="4" />
                            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2" />
                          </svg>
                        {:else if dayWeather.iconType === "cloudSun"}
                          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M17.5 19H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 12 4.5 4.5 0 0 1 17.5 19z" />
                          </svg>
                        {:else if dayWeather.iconType === "rain"}
                          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M17.5 14H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 7 4.5 4.5 0 0 1 17.5 14z" />
                            <path d="M8 17l-1 2M12 17l-1 2M16 17l-1 2" />
                          </svg>
                        {:else if dayWeather.iconType === "snow"}
                          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M17.5 14H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 7 4.5 4.5 0 0 1 17.5 14z" />
                            <path d="M9 18h.01M15 18h.01" />
                          </svg>
                        {:else if dayWeather.iconType === "storm"}
                          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M17.5 13H9a5 5 0 0 1-1.2-9.85A6 6 0 0 1 19.36 6 4.5 4.5 0 0 1 17.5 13z" />
                            <path d="M13 13l-2 4h3l-1 4" />
                          </svg>
                        {:else}
                          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M4 14h16M6 10h12" />
                          </svg>
                        {/if}
                      </span>
                      <span class="f-temp">{day.maxTemp}°</span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <a
            href="https://www.openstreetmap.org/directions?to={place.lat}%2C{place.lon}"
            target="_blank"
            rel="noopener noreferrer"
            class="c-knowledge-widget c-knowledge-widget--action"
          >
            <div class="c-knowledge-widget__action-text">
              <span class="c-knowledge-widget__title">Nasıl gidilir?</span>
              <span class="c-knowledge-widget__subtext">Rota ve yol tarifi al</span>
            </div>
            <div class="c-knowledge-widget__action-icon">
              <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M9 18l6-6-6-6" />
              </svg>
            </div>
          </a>
        </div>
      </div>

      <!-- Altta Detaylar ve Sindirilebilir Hap Bilgiler -->
      <div class="c-knowledge-details">
        {#if infobox.content}
          <p class="c-knowledge-details__summary">
            {infobox.content}
            {#if infobox.urls && infobox.urls.length > 0}
              <a
                href={infobox.urls[0].url}
                target="_blank"
                rel="noopener noreferrer"
                class="c-knowledge-details__inline-source"
              >
                Vikipedi
              </a>
            {/if}
          </p>
        {/if}

        {#if displayAttributes.length > 0}
          <div class="c-knowledge-facts-grid">
            {#each displayAttributes as attr}
              <div class="c-knowledge-fact-item">
                <span class="c-knowledge-fact-label">{attr.label}</span>
                <span class="c-knowledge-fact-value">{attr.value}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    {:else if entityType === "person"}
      <!-- 2B. Kişi / Biyografi: Sol Portre + Sağ Künye & Biyografi -->
      <div class="c-knowledge-person-layout">
        {#if infobox.imgSrc && !imgError}
          <div class="c-knowledge-person-portrait">
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-knowledge-person-portrait__img"
              loading="lazy"
              onerror={handleImgError}
            />
          </div>
        {/if}

        <div class="c-knowledge-person-content">
          {#if infobox.content}
            <p class="c-knowledge-details__summary">
              {infobox.content}
              {#if infobox.urls && infobox.urls.length > 0}
                <a
                  href={infobox.urls[0].url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="c-knowledge-details__inline-source"
                >
                  Vikipedi
                </a>
              {/if}
            </p>
          {/if}

          {#if displayAttributes.length > 0}
            <div class="c-knowledge-facts-grid">
              {#each displayAttributes as attr}
                <div class="c-knowledge-fact-item">
                  <span class="c-knowledge-fact-label">{attr.label}</span>
                  <span class="c-knowledge-fact-value">{attr.value}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>

    {:else if hasValidOrgMap}
      <!-- 2C. Kurum: Medya + Harita -->
      <div class="c-knowledge-org-showcase">
        {#if infobox.imgSrc && !imgError}
          <div class="c-knowledge-tile c-knowledge-tile--org-media">
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-knowledge-tile__img"
              loading="lazy"
              onerror={handleImgError}
            />
          </div>
        {/if}
        <div class="c-knowledge-tile c-knowledge-tile--map">
          <iframe
            title="Yerleşke - {infobox.title}"
            class="c-knowledge-tile__map-iframe"
            src="https://www.openstreetmap.org/export/embed.html?bbox={place.lon - 0.05}%2C{place.lat - 0.03}%2C{place.lon + 0.05}%2C{place.lat + 0.03}&layer=mapnik&marker={place.lat}%2C{place.lon}"
            loading="lazy"
            tabindex="-1"
          ></iframe>
          <div class="c-knowledge-tile__map-overlay">
            <span class="c-knowledge-tile__map-pin">
              <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 21s-6-5.33-6-10a6 6 0 0 1 12 0c0 4.67-6 10-6 10z" />
                <circle cx="12" cy="11" r="2.5" />
              </svg>
              <span>{infobox.title}</span>
            </span>
            <a
              href="https://www.openstreetmap.org/?mlat={place.lat}&mlon={place.lon}#map=14/{place.lat}/{place.lon}"
              target="_blank"
              rel="noopener noreferrer"
              class="c-knowledge-tile__map-btn"
              title="Haritada İncele"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M7 17L17 7M7 7h10v10" />
              </svg>
            </a>
          </div>
        </div>
      </div>

      <div class="c-knowledge-details">
        {#if infobox.content}
          <p class="c-knowledge-details__summary">{infobox.content}</p>
        {/if}
        {#if displayAttributes.length > 0}
          <div class="c-knowledge-facts-grid">
            {#each displayAttributes as attr}
              <div class="c-knowledge-fact-item">
                <span class="c-knowledge-fact-label">{attr.label}</span>
                <span class="c-knowledge-fact-value">{attr.value}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    {:else}
      <!-- 2D. Nesne / Kavram: Sol Medya + Sağ Tanım & Nitelikler -->
      <div class="c-knowledge-thing-layout">
        {#if infobox.imgSrc && !imgError}
          <div class="c-knowledge-thing-media">
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-knowledge-thing-media__img"
              loading="lazy"
              onerror={handleImgError}
            />
          </div>
        {/if}
        <div class="c-knowledge-thing-content">
          {#if infobox.content}
            <p class="c-knowledge-details__summary">
              {infobox.content}
              {#if infobox.urls && infobox.urls.length > 0}
                <a
                  href={infobox.urls[0].url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="c-knowledge-details__inline-source"
                >
                  Vikipedi
                </a>
              {/if}
            </p>
          {/if}
          {#if displayAttributes.length > 0}
            <div class="c-knowledge-facts-grid">
              {#each displayAttributes as attr}
                <div class="c-knowledge-fact-item">
                  <span class="c-knowledge-fact-label">{attr.label}</span>
                  <span class="c-knowledge-fact-value">{attr.value}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ── 3. ALT DIŞ BAĞLANTI HAPLARI (Yalnızca Resmî Site / Harita) ── -->
    {#if infobox.urls && infobox.urls.length > 1}
      <footer class="c-knowledge-footer">
        {#each infobox.urls.slice(1, 4) as link}
          <a
            href={link.url}
            target="_blank"
            rel="noopener noreferrer"
            class="c-knowledge-pill-btn"
          >
            <span>{link.title}</span>
            <svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2.5" class="c-knowledge-pill-arrow">
              <path d="M7 17L17 7M7 7h10v10" />
            </svg>
          </a>
        {/each}
      </footer>
    {/if}
  </article>
{/if}
