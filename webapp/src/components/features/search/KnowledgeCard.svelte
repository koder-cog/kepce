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

  let entityType = $derived(infobox?.entityType || "thing");
  let place = $derived(infobox?.placeInfo);
  let weather = $derived(place?.weather);
  let hasValidPlace = $derived(entityType === "place" && !!place?.lat && !!place?.lon);
  let hasValidOrgMap = $derived(entityType === "organization" && !!place?.lat && !!place?.lon);

  let typeBadgeLabel = $derived.by(() => {
    if (entityType === "place") return "Coğrafi Konum";
    if (entityType === "person") return "Biyografi";
    if (entityType === "organization") return "Kurum";
    return "Bilgi";
  });

  // Dinamik alt başlık / unvan
  let subtitle = $derived.by(() => {
    if (entityType === "place") {
      if (place?.country) return place.country;
      const match = (infobox.content || "").match(/^([^,.]+)/);
      return match ? match[1].trim() : "Coğrafi Konum";
    }
    if (entityType === "person") {
      const text = infobox.content || "";
      const parts = text.split(/[,–-]/);
      if (parts.length > 1 && parts[1].trim().length > 3 && parts[1].trim().length < 60) {
        return parts[1].trim();
      }
      return "Tarihî Şahsiyet / Biyografi";
    }
    if (entityType === "organization") {
      return "Kurum / Kuruluş";
    }
    return "Kavram / Nesne";
  });

  // Kişi için en önemli 4 özet nitelik
  let personQuickStats = $derived.by(() => {
    if (entityType !== "person" || !infobox?.attributes) return [];
    return infobox.attributes.slice(0, 4);
  });
</script>

{#if infobox}
  <article class="c-hig-knowledge c-hig-knowledge--{entityType}" aria-label="Bilgi Kartı">
    <!-- ── 1. ÜST BAŞLIK (Bütünleşik Kart İçi Başlık) ──────────── -->
    <header class="c-hig-knowledge__header">
      <div class="c-hig-knowledge__title-group">
        <h1 class="c-hig-knowledge__title">{infobox.title}</h1>
        <p class="c-hig-knowledge__subtitle">{subtitle}</p>
      </div>
      <div class="c-hig-knowledge__badges">
        <span class="c-hig-knowledge__badge c-hig-knowledge__badge--type">
          {typeBadgeLabel}
        </span>
        {#if infobox.engine}
          <span class="c-hig-knowledge__badge">
            {infobox.engine === "wikipedia"
              ? "Vikipedi"
              : infobox.engine === "wikidata"
                ? "Wikidata"
                : infobox.engine}
          </span>
        {/if}
      </div>
    </header>

    <!-- ── 2. GÖVDE VE VİTRİN DÜZENİ (Varlık Türüne Göre) ───────── -->
    {#if hasValidPlace}
      <!-- 2A. Coğrafi Yer: Üstte 3 Parçalı Vitrin + Altta Genel Bakış -->
      <div class="c-hig-knowledge__showcase">
        <!-- Fotoğraf -->
        <div class="c-hig-tile c-hig-tile--photo">
          {#if infobox.imgSrc && !imgError}
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-hig-tile__img"
              loading="lazy"
              onerror={handleImgError}
            />
          {:else}
            <div class="c-hig-tile__fallback">
              <span>🏛️</span>
            </div>
          {/if}
        </div>

        <!-- Harita Karosu -->
        <div class="c-hig-tile c-hig-tile--map">
          <iframe
            title="Harita - {infobox.title}"
            class="c-hig-tile__map-iframe"
            src="https://www.openstreetmap.org/export/embed.html?bbox={place.lon - 0.08}%2C{place.lat - 0.04}%2C{place.lon + 0.08}%2C{place.lat + 0.04}&layer=mapnik&marker={place.lat}%2C{place.lon}"
            loading="lazy"
            tabindex="-1"
          ></iframe>
          <div class="c-hig-tile__map-overlay">
            <span class="c-hig-tile__map-pin">📍 {infobox.title}</span>
            <a
              href="https://www.openstreetmap.org/?mlat={place.lat}&mlon={place.lon}#map=12/{place.lat}/{place.lon}"
              target="_blank"
              rel="noopener noreferrer"
              class="c-hig-tile__map-btn"
              title="Haritada Büyüt"
            >
              ↗
            </a>
          </div>
        </div>

        <!-- Sağ İkili Yığın (Hava Durumu + Nasıl Gidilir) -->
        <div class="c-hig-stack">
          {#if weather}
            <div class="c-hig-widget c-hig-widget--weather">
              <div class="c-hig-widget__head">
                <span class="c-hig-widget__title">Hava durumu</span>
                <span class="c-hig-widget__icon">{getWeatherInfo(weather.weatherCode).icon}</span>
              </div>
              <div class="c-hig-widget__temp-now">
                {weather.currentTemp}°C
                <span class="c-hig-widget__cond">{getWeatherInfo(weather.weatherCode).label}</span>
              </div>
              {#if weather.daily && weather.daily.length > 0}
                <div class="c-hig-widget__forecast">
                  {#each weather.daily as day}
                    <div class="c-hig-forecast-item">
                      <span class="f-day">{formatDayName(day.date)}</span>
                      <span class="f-icon">{getWeatherInfo(day.code).icon}</span>
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
            class="c-hig-widget c-hig-widget--action"
          >
            <div class="c-hig-widget__action-text">
              <span class="c-hig-widget__title">Nasıl gidilir?</span>
              <span class="c-hig-widget__subtext">Rota ve yol tarifi al</span>
            </div>
            <span class="c-hig-widget__arrow">›</span>
          </a>
        </div>
      </div>

      <!-- Altta Detaylar -->
      <div class="c-hig-details">
        {#if infobox.content}
          <p class="c-hig-details__summary">
            {infobox.content}
            {#if infobox.urls && infobox.urls.length > 0}
              <a
                href={infobox.urls[0].url}
                target="_blank"
                rel="noopener noreferrer"
                class="c-hig-details__inline-source"
              >
                Vikipedi
              </a>
            {/if}
          </p>
        {/if}

        {#if infobox.attributes && infobox.attributes.length > 0}
          <div class="c-hig-attr-chips">
            {#each infobox.attributes.slice(0, 6) as attr}
              <div class="c-hig-attr-chip">
                <span class="c-hig-attr-chip__label">{attr.label}</span>
                <span class="c-hig-attr-chip__value">{attr.value}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    {:else if entityType === "person"}
      <!-- 2B. Kişi / Biyografi: Sol Portre + Sağ Künye & Biyografi -->
      <div class="c-hig-person-layout">
        {#if infobox.imgSrc && !imgError}
          <div class="c-hig-person-portrait">
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-hig-person-portrait__img"
              loading="lazy"
              onerror={handleImgError}
            />
          </div>
        {/if}

        <div class="c-hig-person-content">
          {#if infobox.content}
            <p class="c-hig-details__summary">
              {infobox.content}
              {#if infobox.urls && infobox.urls.length > 0}
                <a
                  href={infobox.urls[0].url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="c-hig-details__inline-source"
                >
                  Vikipedi
                </a>
              {/if}
            </p>
          {/if}

          {#if personQuickStats.length > 0}
            <div class="c-hig-attr-chips">
              {#each personQuickStats as stat}
                <div class="c-hig-attr-chip">
                  <span class="c-hig-attr-chip__label">{stat.label}</span>
                  <span class="c-hig-attr-chip__value">{stat.value}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>

    {:else if hasValidOrgMap}
      <!-- 2C. Kurum / Üniversite: Logo/Kampüs + Yerleşke Haritası -->
      <div class="c-hig-knowledge__org-showcase">
        {#if infobox.imgSrc && !imgError}
          <div class="c-hig-tile c-hig-tile--org-media">
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-hig-tile__img"
              loading="lazy"
              onerror={handleImgError}
            />
          </div>
        {/if}
        <div class="c-hig-tile c-hig-tile--map">
          <iframe
            title="Yerleşke - {infobox.title}"
            class="c-hig-tile__map-iframe"
            src="https://www.openstreetmap.org/export/embed.html?bbox={place.lon - 0.05}%2C{place.lat - 0.03}%2C{place.lon + 0.05}%2C{place.lat + 0.03}&layer=mapnik&marker={place.lat}%2C{place.lon}"
            loading="lazy"
            tabindex="-1"
          ></iframe>
          <div class="c-hig-tile__map-overlay">
            <span class="c-hig-tile__map-pin">🏛️ {infobox.title}</span>
            <a
              href="https://www.openstreetmap.org/?mlat={place.lat}&mlon={place.lon}#map=14/{place.lat}/{place.lon}"
              target="_blank"
              rel="noopener noreferrer"
              class="c-hig-tile__map-btn"
              title="Haritada İncele"
            >
              ↗
            </a>
          </div>
        </div>
      </div>

      <div class="c-hig-details">
        {#if infobox.content}
          <p class="c-hig-details__summary">{infobox.content}</p>
        {/if}
        {#if infobox.attributes && infobox.attributes.length > 0}
          <div class="c-hig-attr-chips">
            {#each infobox.attributes.slice(0, 6) as attr}
              <div class="c-hig-attr-chip">
                <span class="c-hig-attr-chip__label">{attr.label}</span>
                <span class="c-hig-attr-chip__value">{attr.value}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    {:else}
      <!-- 2D. Nesne / Kavram: Sol Medya + Sağ Tanım & Nitelikler -->
      <div class="c-hig-thing-layout">
        {#if infobox.imgSrc && !imgError}
          <div class="c-hig-thing-media">
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-hig-thing-media__img"
              loading="lazy"
              onerror={handleImgError}
            />
          </div>
        {/if}
        <div class="c-hig-thing-content">
          {#if infobox.content}
            <p class="c-hig-details__summary">
              {infobox.content}
              {#if infobox.urls && infobox.urls.length > 0}
                <a
                  href={infobox.urls[0].url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="c-hig-details__inline-source"
                >
                  Vikipedi
                </a>
              {/if}
            </p>
          {/if}
          {#if infobox.attributes && infobox.attributes.length > 0}
            <div class="c-hig-attr-chips">
              {#each infobox.attributes.slice(0, 6) as attr}
                <div class="c-hig-attr-chip">
                  <span class="c-hig-attr-chip__label">{attr.label}</span>
                  <span class="c-hig-attr-chip__value">{attr.value}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ── 3. ALT DIŞ BAĞLANTI HAPLARI ───────────────────────────── -->
    {#if infobox.urls && infobox.urls.length > 0}
      <footer class="c-hig-knowledge__footer">
        {#each infobox.urls.slice(0, 5) as link}
          <a
            href={link.url}
            target="_blank"
            rel="noopener noreferrer"
            class="c-hig-pill-btn"
          >
            <span>{link.title || "Kaynak"}</span>
            <span class="c-hig-pill-arrow">↗</span>
          </a>
        {/each}
      </footer>
    {/if}
  </article>
{/if}
