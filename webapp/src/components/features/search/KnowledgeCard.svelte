<script>
  import { icon } from "@/components/ui/icons.js";
  import MapCard from "./MapCard.svelte";

  let { infobox } = $props();

  let imgError = $state(false);
  let mapExpanded = $state(false);
  let orgMapExpanded = $state(false);

  function handleImgError() {
    imgError = true;
  }

  // Standart hava durumu GNOME ikon eşleştirmesi
  function getWeatherInfo(code) {
    if (code === 0) return { label: "Açık", iconName: "sun" };
    if ([1, 2, 3].includes(code)) return { label: "Parçalı Bulutlu", iconName: "cloudSun" };
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
  let hasImage = $derived(Boolean(infobox?.imgSrc) && !imgError);

  // Vikipedi bağlantısını bulma
  let wikiUrl = $derived.by(() => {
    if (!infobox) return "";
    const urls = infobox.urls || [];
    const wikiItem = urls.find((u) => u.title === "Vikipedi" || (u.url && u.url.includes("wikipedia.org")));
    if (wikiItem?.url) return wikiItem.url;
    if (urls.length > 0 && urls[0].url) return urls[0].url;
    if (infobox.id) return `https://tr.wikipedia.org/wiki/${encodeURIComponent(infobox.id)}`;
    return "";
  });

  // Dinamik alt başlık / unvan
  let subtitle = $derived.by(() => {
    if (entityType === "place") {
      if (place?.country) return place.country;
      const match = (infobox?.content || "").match(/^([^,.]+)/);
      return match ? match[1].trim() : "";
    }
    if (entityType === "person") {
      const text = infobox?.content || "";
      // Eğer infobox'ta açık meslek/unvan niteliği varsa onu tercih et
      const meslekAttr = (infobox?.attributes || []).find((a) =>
        ["mesleği", "meslek", "unvanı", "unvan", "uğraş"].includes((a.label || "").toLowerCase())
      );
      if (meslekAttr?.value) {
        return meslekAttr.value;
      }

      const parts = text.split(/[,–-]/);
      if (parts.length > 1 && parts[1].trim().length > 3 && parts[1].trim().length < 60) {
        const candidate = parts[1].trim();
        const firstSentence = text.split(".")[0] || "";
        // Eğer bu parça cümlenin içinde aynen yer alıyorsa alt başlığı boş geç, papağanlık yapma
        if (firstSentence.toLowerCase().includes(candidate.toLowerCase()) && text.length < 200) {
          return "";
        }
        return candidate;
      }
      return "";
    }
    return "";
  });

  // En önemli özet nitelikler (Grid simetrisi için tam 6, 4 veya 2 eleman)
  let displayAttributes = $derived.by(() => {
    if (!infobox?.attributes) return [];
    const attrs = infobox.attributes;
    if (attrs.length >= 6) return attrs.slice(0, 6);
    if (attrs.length >= 4) return attrs.slice(0, 4);
    if (attrs.length >= 2) return attrs.slice(0, 2);
    return attrs;
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
      <!-- 2A. Coğrafi Yer: Ferah Vitrin (Resim + Google Harita + Modern Hava Durumu) -->
      <div class="c-knowledge-showcase" class:is-map-expanded={mapExpanded} class:has-weather={!!weather}>
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

        <!-- Google Harita Kartı -->
        <MapCard
          lat={place.lat}
          lon={place.lon}
          zoom={12}
          title={infobox.title}
          bind:isExpanded={mapExpanded}
        />

        <!-- Hava Durumu Kartı (Tek ve Ferah Widget) -->
        {#if weather}
          {@const currentWeather = getWeatherInfo(weather.weatherCode)}
          <div class="c-knowledge-widget c-knowledge-widget--weather">
            <div class="c-knowledge-widget__head">
              <span class="c-knowledge-widget__title">Hava durumu</span>
              <span class="c-knowledge-weather-icon">
                {@html icon(currentWeather.iconName, 22)}
              </span>
            </div>

            <div class="c-knowledge-widget__temp-now">
              <span class="temp-val">{weather.currentTemp}°C</span>
              <span class="c-knowledge-widget__cond">{currentWeather.label}</span>
            </div>

            {#if weather.daily && weather.daily.length > 0}
              <div class="c-knowledge-widget__forecast">
                {#each weather.daily as day}
                  {@const dayWeather = getWeatherInfo(day.code)}
                  <div class="c-knowledge-forecast-item">
                    <span class="f-day">{formatDayName(day.date)}</span>
                    <span class="f-icon" title={dayWeather.label}>
                      {@html icon(dayWeather.iconName, 15)}
                    </span>
                    <span class="f-temp">{day.maxTemp}°</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Altta Açıklama, Vikipedi Bağlantısı ve Hap Bilgiler -->
      <div class="c-knowledge-details">
        {#if infobox.content}
          <p class="c-knowledge-details__summary">{infobox.content}</p>
        {/if}

        {#if wikiUrl}
          <a
            href={wikiUrl}
            target="_blank"
            rel="noopener noreferrer"
            class="c-knowledge-more-link"
          >
            <span>Devamını Vikipedi'de oku</span>
            <span class="c-knowledge-more-icon">{@html icon("externalLink", 12)}</span>
          </a>
        {/if}

        {#if displayAttributes.length > 0}
          <div class="c-knowledge-facts-grid" class:has-4-items={displayAttributes.length === 4}>
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
      <div class="c-knowledge-person-layout" class:has-no-image={!hasImage}>
        {#if hasImage}
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
            <p class="c-knowledge-details__summary">{infobox.content}</p>
          {/if}

          {#if wikiUrl}
            <a
              href={wikiUrl}
              target="_blank"
              rel="noopener noreferrer"
              class="c-knowledge-more-link"
            >
              <span>Devamını Vikipedi'de oku</span>
              <span class="c-knowledge-more-icon">{@html icon("externalLink", 12)}</span>
            </a>
          {/if}

          {#if displayAttributes.length > 0}
            <div class="c-knowledge-facts-grid" class:has-4-items={displayAttributes.length === 4}>
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
      <!-- 2C. Kurum: Medya + Yerleşke Google Haritası -->
      <div class="c-knowledge-org-showcase" class:is-map-expanded={orgMapExpanded}>
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
        <MapCard
          lat={place.lat}
          lon={place.lon}
          zoom={14}
          title={infobox.title}
          bind:isExpanded={orgMapExpanded}
        />
      </div>

      <div class="c-knowledge-details">
        {#if infobox.content}
          <p class="c-knowledge-details__summary">{infobox.content}</p>
        {/if}

        {#if wikiUrl}
          <a
            href={wikiUrl}
            target="_blank"
            rel="noopener noreferrer"
            class="c-knowledge-more-link"
          >
            <span>Devamını Vikipedi'de oku</span>
            <span class="c-knowledge-more-icon">{@html icon("externalLink", 12)}</span>
          </a>
        {/if}

        {#if displayAttributes.length > 0}
          <div class="c-knowledge-facts-grid" class:has-4-items={displayAttributes.length === 4}>
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
      <div class="c-knowledge-thing-layout" class:has-no-image={!hasImage}>
        {#if hasImage}
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
            <p class="c-knowledge-details__summary">{infobox.content}</p>
          {/if}

          {#if wikiUrl}
            <a
              href={wikiUrl}
              target="_blank"
              rel="noopener noreferrer"
              class="c-knowledge-more-link"
            >
              <span>Devamını Vikipedi'de oku</span>
              <span class="c-knowledge-more-icon">{@html icon("externalLink", 12)}</span>
            </a>
          {/if}

          {#if displayAttributes.length > 0}
            <div class="c-knowledge-facts-grid" class:has-4-items={displayAttributes.length === 4}>
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
  </article>
{/if}
