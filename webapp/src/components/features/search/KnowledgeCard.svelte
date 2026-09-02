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
  let isPlaceEntity = $derived(!!place?.lat && !!place?.lon);
</script>

{#if infobox}
  <section class="c-google-knowledge" aria-label="Bilgi Kartı">
    <!-- ── 1. ÜST BAŞLIK (Google Tarzı Büyük Başlık & Alt Başlık) ── -->
    <header class="c-google-knowledge__header">
      <div class="c-google-knowledge__title-group">
        <h1 class="c-google-knowledge__title">{infobox.title}</h1>
        {#if place?.country}
          <p class="c-google-knowledge__subtitle">{place.country}</p>
        {/if}
      </div>
      {#if infobox.engine}
        <span class="c-google-knowledge__badge">
          {infobox.engine === "wikipedia"
            ? "Vikipedi"
            : infobox.engine === "wikidata"
              ? "Wikidata"
              : infobox.engine}
        </span>
      {/if}
    </header>

    <!-- ── 2. VİTRİN KARTLARI (Yer/Coğrafya ise 3 Kartlı Grid) ── -->
    {#if isPlaceEntity}
      <div class="c-google-knowledge__showcase">
        <!-- Kart 1: Fotoğraf -->
        <div class="c-google-showcase-tile c-google-showcase-tile--photo">
          {#if infobox.imgSrc && !imgError}
            <img
              src={infobox.imgSrc}
              alt={infobox.title}
              class="c-google-showcase-tile__img"
              loading="lazy"
              onerror={handleImgError}
            />
          {:else}
            <div class="c-google-showcase-tile__fallback">
              <span>🏛️</span>
            </div>
          {/if}
        </div>

        <!-- Kart 2: Harita -->
        <div class="c-google-showcase-tile c-google-showcase-tile--map">
          <iframe
            title="Harita - {infobox.title}"
            class="c-google-showcase-tile__map-iframe"
            src="https://www.openstreetmap.org/export/embed.html?bbox={place.lon - 0.08}%2C{place.lat - 0.04}%2C{place.lon + 0.08}%2C{place.lat + 0.04}&layer=mapnik&marker={place.lat}%2C{place.lon}"
            loading="lazy"
            tabindex="-1"
          ></iframe>
          <div class="c-google-showcase-tile__map-overlay">
            <span class="c-google-showcase-tile__map-pin">📍 {infobox.title}</span>
            <a
              href="https://www.openstreetmap.org/?mlat={place.lat}&mlon={place.lon}#map=12/{place.lat}/{place.lon}"
              target="_blank"
              rel="noopener noreferrer"
              class="c-google-showcase-tile__map-btn"
              title="Haritayı Büyüt"
            >
              ↗
            </a>
          </div>
        </div>

        <!-- Kart 3: Sağ İkili Yığın (Hava Durumu + Nasıl Gidilir) -->
        <div class="c-google-showcase-stack">
          <!-- Hava Durumu Kartı -->
          {#if weather}
            <div class="c-google-stat-card c-google-stat-card--weather">
              <div class="c-google-stat-card__head">
                <span class="c-google-stat-card__title">Hava durumu</span>
                <span class="c-google-stat-card__icon">{getWeatherInfo(weather.weatherCode).icon}</span>
              </div>
              <div class="c-google-stat-card__temp-now">
                {weather.currentTemp}°C
                <span class="c-google-stat-card__cond">{getWeatherInfo(weather.weatherCode).label}</span>
              </div>
              {#if weather.daily && weather.daily.length > 0}
                <div class="c-google-stat-card__forecast">
                  {#each weather.daily as day}
                    <div class="c-google-forecast-item">
                      <span class="f-day">{formatDayName(day.date)}</span>
                      <span class="f-icon">{getWeatherInfo(day.code).icon}</span>
                      <span class="f-temp">{day.maxTemp}°</span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Nasıl Gidilir Kartı -->
          <a
            href="https://www.openstreetmap.org/directions?to={place.lat}%2C{place.lon}"
            target="_blank"
            rel="noopener noreferrer"
            class="c-google-stat-card c-google-stat-card--action"
          >
            <div class="c-google-stat-card__action-text">
              <span class="c-google-stat-card__title">Nasıl gidilir?</span>
              <span class="c-google-stat-card__subtext">Harita üzerinde rota ve yol tarifi al</span>
            </div>
            <span class="c-google-stat-card__arrow">›</span>
          </a>
        </div>
      </div>
    {:else if infobox.imgSrc && !imgError}
      <!-- Genel Varlık (Yer değilse): Tek Geniş Hero Fotoğrafı -->
      <div class="c-google-knowledge__single-hero">
        <img
          src={infobox.imgSrc}
          alt={infobox.title}
          class="c-google-knowledge__single-img"
          loading="lazy"
          onerror={handleImgError}
        />
      </div>
    {/if}

    <!-- ── 3. ALT BİLGİ VE HAKKINDA BÖLÜMÜ ──────────────────── -->
    <div class="c-google-knowledge__details">
      <div class="c-google-knowledge__section-title">Hakkında</div>
      {#if infobox.content}
        <p class="c-google-knowledge__summary">
          {infobox.content}
          {#if infobox.urls && infobox.urls.length > 0}
            <a
              href={infobox.urls[0].url}
              target="_blank"
              rel="noopener noreferrer"
              class="c-google-knowledge__inline-source"
            >
              Vikipedi
            </a>
          {/if}
        </p>
      {/if}

      <!-- Nitelikler Izgarası (Attributes) -->
      {#if infobox.attributes && infobox.attributes.length > 0}
        <div class="c-google-knowledge__attrs-grid">
          {#each infobox.attributes.slice(0, 6) as attr}
            <div class="c-google-attr-item">
              <span class="c-google-attr-label">{attr.label}</span>
              <span class="c-google-attr-value">{attr.value}</span>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Bağlantılar / Hap Butonlar -->
      {#if infobox.urls && infobox.urls.length > 0}
        <div class="c-google-knowledge__pills">
          {#each infobox.urls.slice(0, 5) as link}
            <a
              href={link.url}
              target="_blank"
              rel="noopener noreferrer"
              class="c-google-pill-btn"
            >
              <span>{link.title || "Kaynak"}</span>
              <span class="c-google-pill-arrow">↗</span>
            </a>
          {/each}
        </div>
      {/if}
    </div>
  </section>
{/if}
