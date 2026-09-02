<script>
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { icon } from "@/components/ui/icons.js";
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import SearchInfoModal from "@/components/features/search/SearchInfoModal.svelte";
  import SearchSettingsModal from "@/components/features/search/SearchSettingsModal.svelte";

  let { data } = $props();

  let searchInput = $state("");
  let searchInputEl = $state(null);
  let isInfoOpen = $state(false);
  let isSettingsOpen = $state(false);

  const BANG_POOL = [
    { prefix: "!w", label: "Vikipedi" },
    { prefix: "!yt", label: "YouTube" },
    { prefix: "!gh", label: "GitHub" },
    { prefix: "!r", label: "Reddit" },
    { prefix: "!so", label: "Stack Overflow" },
    { prefix: "!m", label: "Harita" },
    { prefix: "!imdb", label: "IMDb" },
    { prefix: "!brave", label: "Brave" },
    { prefix: "!ddg", label: "DuckDuckGo" },
    { prefix: "!g", label: "Google" },
    { prefix: "!mdn", label: "MDN" },
    { prefix: "!npm", label: "npm" },
    { prefix: "!crates", label: "Crates.io" },
    { prefix: "!arch", label: "ArchWiki" },
    { prefix: "!unsplash", label: "Unsplash" },
  ];

  let randomShortcuts = $state(BANG_POOL.slice(0, 4));

  onMount(() => {
    const shuffled = [...BANG_POOL].sort(() => 0.5 - Math.random());
    randomShortcuts = shuffled.slice(0, 4);
  });

  function selectBang(prefix) {
    searchInput = prefix + " ";
    if (searchInputEl) {
      searchInputEl.focus();
    }
  }

  let isSubdomain = $derived(
    $page.url.hostname.startsWith("ara.") ||
      $page.url.hostname === "ara.localhost",
  );
  let basePath = $derived(isSubdomain ? "" : "/ara");

  $effect(() => {
    searchInput = data.query || "";
  });

  const CATEGORIES = [
    { id: "general", label: "Web" },
    { id: "images", label: "Görseller" },
    { id: "videos", label: "Videolar" },
    { id: "news", label: "Haberler" },
    { id: "map", label: "Haritalar" },
  ];

  function handleSearch(e) {
    e?.preventDefault();
    const q = searchInput.trim();
    if (!q) return;

    const params = new URLSearchParams();
    params.set("q", q);
    if (data.category && data.category !== "general") {
      params.set("kategori", data.category);
    }
    if (data.language && data.language !== "tr") {
      params.set("dil", data.language);
    }
    if (data.timeRange) {
      params.set("zaman", data.timeRange);
    }
    if (data.safeSearch && data.safeSearch !== "1") {
      params.set("guvenli", data.safeSearch);
    }

    goto(`${basePath}/?${params.toString()}`);
  }

  function handleCategoryChange(catId) {
    const params = new URLSearchParams();
    params.set("q", data.query);
    if (catId !== "general") {
      params.set("kategori", catId);
    }
    if (data.language && data.language !== "tr") {
      params.set("dil", data.language);
    }
    if (data.timeRange) {
      params.set("zaman", data.timeRange);
    }
    if (data.safeSearch && data.safeSearch !== "1") {
      params.set("guvenli", data.safeSearch);
    }
    goto(`${basePath}/?${params.toString()}`);
  }

  function handleFilterChange(key, value) {
    const params = new URLSearchParams();
    params.set("q", data.query);
    if (data.category && data.category !== "general") {
      params.set("kategori", data.category);
    }
    const currentLang = key === "dil" ? value : data.language;
    const currentTime = key === "zaman" ? value : data.timeRange;
    const currentSafe = key === "guvenli" ? value : data.safeSearch;

    if (currentLang && currentLang !== "tr") params.set("dil", currentLang);
    if (currentTime) params.set("zaman", currentTime);
    if (currentSafe && currentSafe !== "1") params.set("guvenli", currentSafe);

    goto(`${basePath}/?${params.toString()}`);
  }

  function getPageUrl(pageNum) {
    const params = new URLSearchParams();
    params.set("q", data.query);
    if (data.category && data.category !== "general")
      params.set("kategori", data.category);
    if (data.language && data.language !== "tr")
      params.set("dil", data.language);
    if (data.timeRange) params.set("zaman", data.timeRange);
    if (data.safeSearch && data.safeSearch !== "1")
      params.set("guvenli", data.safeSearch);
    if (pageNum > 1) params.set("sayfa", String(pageNum));
    return `${basePath}/?${params.toString()}`;
  }

  function formatUrlBreadcrumb(rawUrl) {
    try {
      const u = new URL(rawUrl);
      const parts = u.pathname.split("/").filter(Boolean);
      if (parts.length === 0) return u.origin;
      return `${u.origin} › ${parts.join(" › ")}`;
    } catch {
      return rawUrl;
    }
  }
</script>

<svelte:head>
  {#if data.isHome}
    <title>Kepçe Ara</title>
    <meta
      name="description"
      content="Gizlilik odaklı, açık kaynaklı meta arama motoru."
    />
  {:else}
    <title>{data.query} | Kepçe Ara</title>
    <meta name="description" content="{data.query} arama sonuçları." />
  {/if}
</svelte:head>

{#if data.isHome}
  <!-- ── 1. ANA ARAMA EKRANI (Home) ────────────────────────── -->
  <div class="c-search-home">
    <header class="c-search-home__header">
      <button
        type="button"
        class="c-search-icon-btn"
        onclick={() => (isInfoOpen = true)}
        aria-label="Hakkında"
        title="Hakkında"
      >
        {@html icon("info", 24)}
      </button>
      <button
        type="button"
        class="c-search-icon-btn"
        onclick={() => (isSettingsOpen = true)}
        aria-label="Ayarlar"
        title="Ayarlar"
      >
        {@html icon("settings", 24)}
      </button>
    </header>

    <main class="c-search-home__center">
      <!-- Stabil Logo -->
      <div class="c-search-home__logo" aria-label="Kepçe">
        {@html icon("logo", null, "Kepçe")}
      </div>

      <form class="c-search-box" onsubmit={handleSearch}>
        <input
          type="search"
          class="c-search-box__input"
          placeholder="İnterneti kurcala..."
          bind:value={searchInput}
          bind:this={searchInputEl}
          aria-label="Ara"
        />
        <button
          type="submit"
          class="c-search-box__submit"
          aria-label="Ara"
          title="Ara"
        >
          {@html icon("search", 20)}
        </button>
      </form>

      <!-- Hızlı Kısayol İpuçları (Dinamik Rastgele Havuz) -->
      <div class="c-search-home__shortcuts">
        {#each randomShortcuts as s}
          <button
            type="button"
            class="c-search-shortcut-pill"
            onclick={() => selectBang(s.prefix)}
          >
            <span class="pill-prefix">{s.prefix}</span> {s.label}
          </button>
        {/each}
      </div>
    </main>

    <!-- Minimalist Ana Sayfa Alt Şeridi -->
    <footer class="c-search-home-footer">
      <div class="c-search-home-footer__left">
        <a href={`${basePath}/ayarlar`} class="c-search-home-footer__link">Ayarlar</a>
        <a href={`${basePath}/gizlilik`} class="c-search-home-footer__link">Gizlilik</a>
        <a href={`${basePath}/kosullar`} class="c-search-home-footer__link">Koşullar</a>
        <a href={`${basePath}/iletisim`} class="c-search-home-footer__link">İletişim</a>
      </div>
      <div class="c-search-home-footer__right">
        <a
          href="https://github.com/searxng/searxng"
          target="_blank"
          rel="noopener noreferrer"
          class="c-search-home-footer__link">SearXNG</a
        >
        <a
          href="https://github.com/koder-cog/kepce"
          target="_blank"
          rel="noopener noreferrer"
          class="c-search-home-footer__link">Kaynak Kodu</a
        >
        <a
          href="https://www.gnu.org/licenses/agpl-3.0.html#license-text"
          target="_blank"
          rel="noopener noreferrer"
          class="c-search-home-footer__link c-search-home-footer__agpl"
          >AGPLv3</a
        >
      </div>
    </footer>
  </div>
{:else}
  <!-- ── 2. ARAMA SONUÇLARI EKRANI (Results) ────────────────── -->
  <div class="c-search-results">
    <!-- Üst Arama Çubuğu -->
    <header class="c-search-results__topbar">
      <div class="c-search-results__topbar-inner">
        <a
          href={basePath || "/"}
          class="c-search-results__brand"
          aria-label="Kepçe Ara"
          title="Kepçe Ara"
        >
          {@html icon("logoSmall", 36, "Kepçe Logosu")}
        </a>

        <div class="c-search-results__search-wrap">
          <form class="c-search-box" onsubmit={handleSearch}>
            <input
              type="search"
              class="c-search-box__input"
              placeholder="Ara..."
              bind:value={searchInput}
              aria-label="Ara"
            />
            <button
              type="submit"
              class="c-search-box__submit"
              aria-label="Ara"
              title="Ara"
            >
              {@html icon("search", 20)}
            </button>
          </form>
        </div>

        <div class="c-search-results__topbar-actions">
          <button
            type="button"
            class="c-search-icon-btn"
            onclick={() => (isInfoOpen = true)}
            aria-label="Hakkında"
            title="Hakkında"
          >
            {@html icon("info", 22)}
          </button>
          <button
            type="button"
            class="c-search-icon-btn"
            onclick={() => (isSettingsOpen = true)}
            aria-label="Ayarlar"
            title="Ayarlar"
          >
            {@html icon("settings", 22)}
          </button>
        </div>
      </div>
    </header>

    <!-- Kategori ve Filtre Çubuğu (SegmentedControl ve Dropdown Ghost) -->
    <div class="c-search-subbar-wrap">
      <!-- 1. Kategori Adası (Kepçe SegmentedControl) -->
      <div class="c-search-categories-island">
        <SegmentedControl
          options={CATEGORIES.map((c) => ({ value: c.id, label: c.label }))}
          value={data.category || "general"}
          onChange={(catId) => handleCategoryChange(catId)}
        />
      </div>

      <!-- 2. Filtreler (Kepçe Dropdown Ghost) -->
      <div class="c-search-pill-filters">
        <Dropdown
          variant="ghost"
          value={data.language || "tr"}
          options={[
            { value: "tr", label: "Türkiye" },
            { value: "all", label: "Tüm diller" },
          ]}
          onChange={(val) => handleFilterChange("dil", val)}
        />

        <Dropdown
          variant="ghost"
          value={data.safeSearch || "1"}
          options={[
            { value: "1", label: "Güvenli arama: Orta" },
            { value: "2", label: "Güvenli arama: Katı" },
            { value: "0", label: "Güvenli arama: Kapalı" },
          ]}
          onChange={(val) => handleFilterChange("guvenli", val)}
        />

        <Dropdown
          variant="ghost"
          value={data.timeRange || ""}
          options={[
            { value: "", label: "Tüm zamanlar" },
            { value: "day", label: "Son 24 saat" },
            { value: "week", label: "Son 1 hafta" },
            { value: "month", label: "Son 1 ay" },
            { value: "year", label: "Son 1 yıl" },
          ]}
          onChange={(val) => handleFilterChange("zaman", val)}
        />
      </div>
    </div>

    <!-- Sonuçlar Gövdesi -->
    <main class="c-search-body">
      <!-- Sol Kolon: Sonuç Listesi -->
      <section class="c-search-list">
        {#if data.error}
          <div class="card u-p-lg">
            <p class="u-text-sm u-color-danger">{data.error}</p>
          </div>
        {:else if data.results.length === 0}
          <div class="card u-p-lg">
            <p class="u-text-sm">
              <strong>{data.query}</strong> ile ilgili sonuç bulunamadı.
            </p>
          </div>
        {:else if data.category === "images"}
          <!-- Görsel Sonuçları Izgarası -->
          <div class="c-search-images-grid">
            {#each data.results as item}
              <a
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                class="c-search-image-card"
              >
                {#if item.imgSrc}
                  <img
                    src={item.imgSrc}
                    alt={item.title}
                    class="c-search-image-card__img"
                    loading="lazy"
                  />
                {/if}
                <div class="c-search-image-card__meta" title={item.title}>
                  {item.title}
                </div>
              </a>
            {/each}
          </div>
        {:else}
          <!-- Standart Web / Haber / Video Sonuçları -->
          {#each data.results as item}
            <article class="c-search-item">
              <a
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                class="c-search-item__url"
              >
                <span>{formatUrlBreadcrumb(item.url)}</span>
              </a>
              <a
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                class="c-search-item__title"
              >
                {item.title}
              </a>
              <p class="c-search-item__content">
                {item.content}
              </p>
            </article>
          {/each}
        {/if}

        <!-- Sayfalama -->
        {#if data.results.length > 0}
          <div class="c-search-pagination">
            {#if data.page > 1}
              <a
                href={getPageUrl(data.page - 1)}
                class="btn btn--sm btn--secondary"
                aria-label="Önceki"
              >
                Önceki
              </a>
            {/if}
            <span class="btn btn--sm btn--primary">
              {data.page}
            </span>
            <a
              href={getPageUrl(data.page + 1)}
              class="btn btn--sm btn--secondary"
              aria-label="Sonraki"
            >
              {data.page + 1}
            </a>
            <a
              href={getPageUrl(data.page + 2)}
              class="btn btn--sm btn--secondary"
              aria-label="Sayfa {data.page + 2}"
            >
              {data.page + 2}
            </a>
            <a
              href={getPageUrl(data.page + 1)}
              class="btn btn--sm btn--secondary"
              aria-label="Sonraki"
            >
              Sonraki
            </a>
          </div>
        {/if}
      </section>

      <!-- Sağ Kolon: Wikipedia Bilgi Kartı (Infobox) -->
      {#if data.infoboxes && data.infoboxes.length > 0}
        <aside class="c-search-infobox">
          <h3 class="c-search-infobox__title">{data.infoboxes[0].title}</h3>
          {#if data.infoboxes[0].imgSrc}
            <div class="c-search-infobox__img-wrap">
              <img
                src={data.infoboxes[0].imgSrc}
                alt={data.infoboxes[0].title}
                class="c-search-infobox__img"
              />
            </div>
          {/if}
          <p class="c-search-infobox__content">
            {data.infoboxes[0].content}
          </p>
          {#if data.infoboxes[0].urls && data.infoboxes[0].urls.length > 0}
            <a
              href={data.infoboxes[0].urls[0].url}
              target="_blank"
              rel="noopener noreferrer"
              class="c-search-infobox__link"
            >
              <span>Devamını Vikipedi'de oku</span>
              {@html icon("externalLink", 14)}
            </a>
          {/if}
        </aside>
      {/if}
    </main>

    <!-- 4 Kolonlu Dengeli Arama Footer'ı -->
    <footer class="site-footer">
      <div class="site-footer__inner">
        <div class="site-footer__brand">
          <div class="site-footer__brand-title">Kepçe Ara</div>
          <p class="site-footer__brand-desc">
            KYK menülerinden internetin derinliklerine. Kullanıcı verilerini
            satmayan, gizliliğinize saygılı açık kaynaklı meta arama motoru.
          </p>
        </div>

        <div class="site-footer__col">
          <div class="site-footer__col-title">SearXNG</div>
          <a
            href="https://github.com/searxng/searxng"
            target="_blank"
            rel="noopener noreferrer"
            class="site-footer__link">Kaynak kodu</a
          >
          <a
            href="https://github.com/searxng/searxng/issues"
            target="_blank"
            rel="noopener noreferrer"
            class="site-footer__link">Hata Bildirimi</a
          >
          <a
            href="https://searx.space/"
            target="_blank"
            rel="noopener noreferrer"
            class="site-footer__link">Motor İstatistikleri</a
          >
          <a
            href="https://searx.space/"
            target="_blank"
            rel="noopener noreferrer"
            class="site-footer__link">Diğer Açık Sunucular</a
          >
        </div>

        <div class="site-footer__col">
          <div class="site-footer__col-title">Yasal</div>
          <a
            href={`${basePath}/gizlilik`}
            class="site-footer__link">Gizlilik politikası</a
          >
          <a
            href={`${basePath}/kosullar`}
            class="site-footer__link">Kullanım koşulları</a
          >
          <a href={`${basePath}/iletisim`} class="site-footer__link"
            >İletişim & Geri Bildirim</a
          >
        </div>

        <div class="site-footer__col">
          <div class="site-footer__col-title">Bağlantılar</div>
          <a href={`${basePath}/ayarlar`} class="site-footer__link">Ayarlar</a>
          <a href="https://kepce.org" class="site-footer__link">Kepçe (Ana Site)</a>
          <a
            href="https://reddit.com/r/kepce"
            target="_blank"
            rel="noopener noreferrer"
            class="site-footer__link">Subreddit</a
          >
          <a
            href="https://twitter.com/kepceorg"
            target="_blank"
            rel="noopener noreferrer"
            class="site-footer__link">Twitter</a
          >
          <a
            href="https://instagram.com/kepceorg"
            target="_blank"
            rel="noopener noreferrer"
            class="site-footer__link">Instagram</a
          >
        </div>
      </div>

      <div class="site-footer__bottom">
        <div class="site-footer__agpl">
          <div
            class="site-footer__agpl-badge"
            role="img"
            aria-label="AGPLv3"
          ></div>
        </div>
      </div>
    </footer>
  </div>
{/if}

<!-- Modallar -->
<SearchInfoModal bind:isOpen={isInfoOpen} />
<SearchSettingsModal bind:isOpen={isSettingsOpen} />
