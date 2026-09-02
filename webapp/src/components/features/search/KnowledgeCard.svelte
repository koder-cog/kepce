<script>
  import { icon } from "@/components/ui/icons.js";
  import MapCard from "./MapCard.svelte";

  let { infobox } = $props();

  let imgError = $state(false);

  function handleImgError() {
    imgError = true;
  }

  // Standart hava durumu GNOME ikon eşleştirmesi
  function getWeatherInfo(code) {
    if (code === 0) return { label: "Açık", iconName: "sun" };
    if ([1, 2, 3].includes(code)) return { label: "Bulutlu", iconName: "cloudSun" };
    if ([45, 48].includes(code)) return { label: "Sisli", iconName: "fog" };
    if ([51, 53, 55, 61, 63, 65, 80, 81, 82].includes(code)) return { label: "Yağmurlu", iconName: "rain" };
    if ([71, 73, 75, 85, 86].includes(code)) return { label: "Karlı", iconName: "snow" };
    if ([95, 96, 99].includes(code)) return { label: "Fırtına", iconName: "storm" };
    return { label: "Açık", iconName: "sun" };
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
      <!-- 2A. Coğrafi Yer: 3 Parçalı Vitrin (Resim + Retina Harita + Hava Durumu) -->
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
              {@html icon("image", 40)}
            </div>
          {/if}
        </div>

        <!-- Yüksek Çözünürlüklü Vektör Harita -->
        <MapCard lat={place.lat} lon={place.lon} zoom={12} />

        <!-- Sağ İkili Yığın (Hava Durumu + Nasıl Gidilir) -->
        <div class="c-knowledge-stack">
          {#if weather}
            {@const currentWeather = getWeatherInfo(weather.weatherCode)}
            <div class="c-knowledge-widget c-knowledge-widget--weather">
              <div class="c-knowledge-widget__head">
                <span class="c-knowledge-widget__title">Hava durumu</span>
                <span class="c-knowledge-weather-icon">
                  {@html icon(currentWeather.iconName, 20)}
                </span>
              </div>
              <div class="c-knowledge-widget__temp-now">
                {weather.currentTemp}°C
                <span class="c-knowledge-widget__cond">{currentWeather.label}</span>
              </div>
              {#if weather.daily && weather.daily.length > 0}
                <div class="c-knowledge-widget__forecast">
                  {#each weather.daily as day}
                    {@const dayWeather = getWeatherInfo(day.code)}
                    <div class="c-knowledge-forecast-item">
                      <span class="f-day">{formatDayName(day.date)}</span>
                      <span class="f-icon" title={dayWeather.label}>
                        {@html icon(dayWeather.iconName, 14)}
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
              {@html icon("chevronRight", 18)}
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
      <!-- 2C. Kurum: Medya + Yerleşke Haritası -->
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
        <MapCard lat={place.lat} lon={place.lon} zoom={14} />
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

    <!-- ── 3. ALT DIŞ BAĞLANTI HAPLARI ───────────────────────────── -->
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
            <span class="c-knowledge-pill-arrow">
              {@html icon("externalLink", 11)}
            </span>
          </a>
        {/each}
      </footer>
    {/if}
  </article>
{/if}
