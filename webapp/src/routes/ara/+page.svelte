<script>
  import { onMount } from "svelte";
  import { goto, preloadData } from "$app/navigation";
  import { page, navigating } from "$app/stores";
  import { icon } from "@/components/ui/icons.js";
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import SearchInfoModal from "@/components/features/search/SearchInfoModal.svelte";
  import SearchSettingsModal from "@/components/features/search/SearchSettingsModal.svelte";
  import KnowledgeCard from "@/components/features/search/KnowledgeCard.svelte";
  import AnswerCard from "@/components/features/search/AnswerCard.svelte";
  import { BANG_DEFINITIONS } from "$lib/search/bangs.js";

  let { data } = $props();

  let searchInput = $state("");
  let searchInputEl = $state(null);
  let isInfoOpen = $state(false);
  let isSettingsOpen = $state(false);
  let activeCategory = $state("general");

  $effect(() => {
    activeCategory = data.category || "general";
  });

  let randomShortcuts = $state(
    BANG_DEFINITIONS.slice(0, 4).map((b) => ({ prefix: b.prefix, label: b.name }))
  );

  let searchHistory = $state([]);
  let isHistoryOpen = $state(false);
  let selectedImage = $state(null);
  let isImageLightboxOpen = $state(false);
  let activeVideoEmbed = $state(null);

  function loadHistory() {
    try {
      const raw = localStorage.getItem("kepce_search_history");
      if (raw) {
        searchHistory = JSON.parse(raw);
      }
    } catch {
      searchHistory = [];
    }
  }

  function saveToHistory(q) {
    const term = q.trim();
    if (!term || term.startsWith("!")) return;
    try {
      let list = searchHistory.filter((item) => item.toLowerCase() !== term.toLowerCase());
      list.unshift(term);
      list = list.slice(0, 8);
      searchHistory = list;
      localStorage.setItem("kepce_search_history", JSON.stringify(list));
    } catch {}
  }

  function clearHistory() {
    searchHistory = [];
    try {
      localStorage.removeItem("kepce_search_history");
    } catch {}
  }

  function removeHistoryItem(term, e) {
    e.stopPropagation();
    searchHistory = searchHistory.filter((t) => t !== term);
    try {
      localStorage.setItem("kepce_search_history", JSON.stringify(searchHistory));
    } catch {}
  }

  function openImageLightbox(item, e) {
    e.preventDefault();
    selectedImage = item;
    isImageLightboxOpen = true;
  }

  function closeImageLightbox() {
    selectedImage = null;
    isImageLightboxOpen = false;
  }

  function getYoutubeEmbedUrl(url) {
    if (!url) return null;
    const m = url.match(/(?:youtube\.com\/(?:watch\?v=|embed\/)|youtu\.be\/)([a-zA-Z0-9_-]{11})/);
    if (m) {
      return `https://www.youtube-nocookie.com/embed/${m[1]}?autoplay=1`;
    }
    return null;
  }

  onMount(() => {
    loadHistory();
    const shuffled = [...BANG_DEFINITIONS]
      .sort(() => 0.5 - Math.random())
      .slice(0, 4)
      .map((b) => ({ prefix: b.prefix, label: b.name }));
    randomShortcuts = shuffled;
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

  let selectedResultIndex = $state(-1);

  const CATEGORIES = [
    { id: "general", label: "Web" },
    { id: "images", label: "Görseller" },
    { id: "videos", label: "Videolar" },
    { id: "news", label: "Haberler" },
    { id: "it", label: "Kod" },
    { id: "science", label: "Akademi" },
    { id: "map", label: "Haritalar" },
  ];

  function handleGlobalKeydown(e) {
    const activeEl = document.activeElement;
    const isInputActive =
      activeEl &&
      (activeEl.tagName === "INPUT" ||
        activeEl.tagName === "TEXTAREA" ||
        activeEl.isContentEditable);

    if (e.key === "/" && !isInputActive) {
      e.preventDefault();
      if (searchInputEl) {
        searchInputEl.focus();
        searchInputEl.select();
      }
      return;
    }

    if (isInputActive) {
      if (e.key === "Escape") {
        activeEl.blur();
        isSuggestionsOpen = false;
        isHistoryOpen = false;
      }
      return;
    }

    if (data.results && data.results.length > 0) {
      if (e.key === "j" || e.key === "ArrowDown") {
        e.preventDefault();
        selectedResultIndex = Math.min(data.results.length - 1, selectedResultIndex + 1);
        scrollSelectedResult();
      } else if (e.key === "k" || e.key === "ArrowUp") {
        e.preventDefault();
        selectedResultIndex = Math.max(0, selectedResultIndex - 1);
        scrollSelectedResult();
      } else if (e.key === "Enter" && selectedResultIndex >= 0) {
        const target = data.results[selectedResultIndex];
        if (target?.url) {
          window.open(target.url, "_blank", "noopener,noreferrer");
        }
      } else if (e.key === "Escape") {
        selectedResultIndex = -1;
      }
    }
  }

  function scrollSelectedResult() {
    const el = document.querySelector(`.c-search-item[data-index="${selectedResultIndex}"]`);
    if (el) {
      el.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }

  let suggestions = $state([]);
  let isSuggestionsOpen = $state(false);
  let selectedSuggestionIndex = $state(-1);
  let debounceTimeout = null;

  async function fetchSuggestions(query) {
    const q = query.trim();
    if (!q || q.length < 2) {
      suggestions = [];
      isSuggestionsOpen = false;
      return;
    }

    try {
      const res = await fetch(`${basePath}/autocompleter?q=${encodeURIComponent(q)}`);
      if (res.ok) {
        const list = await res.json();
        if (Array.isArray(list) && list.length > 0) {
          suggestions = list;
          isSuggestionsOpen = true;
          selectedSuggestionIndex = -1;
          return;
        }
      }
    } catch {
      // ignore
    }
    suggestions = [];
    isSuggestionsOpen = false;
  }

  function handleInput(e) {
    const val = e.target.value;
    searchInput = val;
    isHistoryOpen = !val;
    clearTimeout(debounceTimeout);
    debounceTimeout = setTimeout(() => {
      fetchSuggestions(val);
    }, 150);
  }

  function handleFocus() {
    if (!searchInput.trim() && searchHistory.length > 0) {
      isHistoryOpen = true;
    }
  }

  function handleKeydown(e) {
    if (isSuggestionsOpen && suggestions.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        selectedSuggestionIndex = (selectedSuggestionIndex + 1) % suggestions.length;
        searchInput = suggestions[selectedSuggestionIndex];
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedSuggestionIndex = (selectedSuggestionIndex - 1 + suggestions.length) % suggestions.length;
        searchInput = suggestions[selectedSuggestionIndex];
      } else if (e.key === "Escape") {
        isSuggestionsOpen = false;
        selectedSuggestionIndex = -1;
      }
    }
  }

  function selectSuggestion(item) {
    searchInput = item;
    isSuggestionsOpen = false;
    isHistoryOpen = false;
    handleSearch();
  }

  function handleBlur() {
    setTimeout(() => {
      isSuggestionsOpen = false;
      isHistoryOpen = false;
    }, 200);
  }

  function buildSearchUrl(params) {
    const qs = params.toString();
    const prefix = isSubdomain ? "" : "/ara";
    return `${prefix}${qs ? `?${qs}` : ""}` || "/";
  }

  function handleSearch(e) {
    e?.preventDefault();
    isSuggestionsOpen = false;
    isHistoryOpen = false;
    const q = searchInput.trim();
    if (!q) return;

    saveToHistory(q);

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

    goto(buildSearchUrl(params));
  }

  function handleCategoryChange(catId) {
    activeCategory = catId;
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
    goto(buildSearchUrl(params));
  }

  function handleCategoryHover(catId) {
    if (!data.query) return;
    const params = new URLSearchParams();
    params.set("q", data.query);
    if (catId !== "general") params.set("kategori", catId);
    if (data.language && data.language !== "tr") params.set("dil", data.language);
    if (data.timeRange) params.set("zaman", data.timeRange);
    if (data.safeSearch && data.safeSearch !== "1") params.set("guvenli", data.safeSearch);
    preloadData(buildSearchUrl(params));
  }

  function getDomain(rawUrl) {
    try {
      const u = new URL(rawUrl);
      return u.hostname.replace(/^www\./, "");
    } catch {
      return "";
    }
  }

  function getFaviconUrl(rawUrl) {
    const domain = getDomain(rawUrl);
    if (!domain) return "";
    return `https://icons.duckduckgo.com/ip3/${domain}.ico`;
  }

  function formatDateSnippet(dateStr) {
    if (!dateStr) return "";
    try {
      const d = new Date(dateStr);
      if (isNaN(d.getTime())) return "";
      return d.toLocaleDateString("tr-TR", { day: "numeric", month: "short", year: "numeric" });
    } catch {
      return "";
    }
  }

  function escapeHtml(str) {
    return String(str || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  function highlightQuery(text, query) {
    if (!text) return "";
    if (!query) return escapeHtml(text);

    const tokens = query
      .trim()
      .split(/\s+/)
      .filter((t) => t.length > 1 && !t.startsWith("!"));
    if (tokens.length === 0) return escapeHtml(text);

    const escapedTokens = tokens.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
    const regex = new RegExp(`(${escapedTokens.join("|")})`, "gi");

    const safe = escapeHtml(text);
    return safe.replace(regex, "<strong>$1</strong>");
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

    goto(buildSearchUrl(params));
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
    return buildSearchUrl(params);
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

<svelte:window onkeydown={handleGlobalKeydown} />

<svelte:head>
  <link rel="preconnect" href="https://icons.duckduckgo.com" />
  <link rel="dns-prefetch" href="https://icons.duckduckgo.com" />
  <link
    rel="search"
    type="application/opensearchdescription+xml"
    title="Kepçe Ara"
    href="/opensearch.xml"
  />
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
        {@html icon("logoExperimental", null, "Kepçe")}
      </div>

      <form class="c-search-box" onsubmit={handleSearch}>
        <input
          type="search"
          class="c-search-box__input"
          placeholder="İnterneti kurcala..."
          value={searchInput}
          oninput={handleInput}
          onkeydown={handleKeydown}
          onblur={handleBlur}
          bind:this={searchInputEl}
          aria-label="Ara"
          autocomplete="off"
        />
        <button
          type="submit"
          class="c-search-box__submit"
          aria-label="Ara"
          title="Ara"
        >
          {@html icon("search", 20)}
        </button>

        {#if isSuggestionsOpen && suggestions.length > 0}
          <ul class="c-search-autocomplete" role="listbox">
            {#each suggestions as item, idx}
              <li
                class="c-search-autocomplete__item"
                class:is-selected={idx === selectedSuggestionIndex}
                onmousedown={() => selectSuggestion(item)}
                role="option"
                aria-selected={idx === selectedSuggestionIndex}
              >
                <span class="c-search-autocomplete__icon">
                  {@html icon("search", 16)}
                </span>
                <span>{item}</span>
              </li>
            {/each}
          </ul>
        {/if}
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
          {@html icon("logoSmallExperimental", 36, "Kepçe Logosu")}
        </a>

        <div class="c-search-results__search-wrap">
          <form class="c-search-box" onsubmit={handleSearch}>
            <input
              type="search"
              class="c-search-box__input"
              placeholder="Ara..."
              value={searchInput}
              oninput={handleInput}
              onkeydown={handleKeydown}
              onblur={handleBlur}
              aria-label="Ara"
              autocomplete="off"
            />
            <button
              type="submit"
              class="c-search-box__submit"
              aria-label="Ara"
              title="Ara"
            >
              {@html icon("search", 20)}
            </button>

            <!-- Yerel Arama Geçmişi (Arama çubuğu boş ve odaklıyken) -->
            {#if isHistoryOpen && searchHistory.length > 0 && !searchInput}
              <ul class="c-search-autocomplete c-search-history-panel" role="listbox">
                <li class="c-search-history-header">
                  <span>Son Aramalar</span>
                  <button type="button" class="c-search-history-clear" onmousedown={clearHistory}>
                    Temizle
                  </button>
                </li>
                {#each searchHistory as item}
                  <li
                    class="c-search-autocomplete__item c-search-history__item"
                    onmousedown={() => selectSuggestion(item)}
                    role="option"
                    tabindex="-1"
                    aria-selected="false"
                  >
                    <span class="c-search-autocomplete__icon">
                      {@html icon("clock", 16)}
                    </span>
                    <span class="c-search-history__text">{item}</span>
                    <button
                      type="button"
                      class="c-search-history__remove"
                      onmousedown={(e) => removeHistoryItem(item, e)}
                      aria-label="Sil"
                      title="Sil"
                    >
                      ✕
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}

            {#if isSuggestionsOpen && suggestions.length > 0}
              <ul class="c-search-autocomplete" role="listbox">
                {#each suggestions as item, idx}
                  <li
                    class="c-search-autocomplete__item"
                    class:is-selected={idx === selectedSuggestionIndex}
                    onmousedown={() => selectSuggestion(item)}
                    role="option"
                    aria-selected={idx === selectedSuggestionIndex}
                  >
                    <span class="c-search-autocomplete__icon">
                      {@html icon("search", 16)}
                    </span>
                    <span>{item}</span>
                  </li>
                {/each}
              </ul>
            {/if}
          </form>
        </div>

        <div class="c-search-results__topbar-actions">
          <a
            href={basePath ? `${basePath}/rss?q=${encodeURIComponent(data.query)}&kategori=${data.category || 'general'}` : `/ara/rss?q=${encodeURIComponent(data.query)}&kategori=${data.category || 'general'}`}
            target="_blank"
            rel="noopener noreferrer"
            class="c-search-icon-btn"
            aria-label="RSS Akışı"
            title="Arama Sonuçlarını RSS Beslemesi Olarak Al"
          >
            {@html icon("rss", 20)}
          </a>
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
          value={activeCategory}
          onChange={(catId) => handleCategoryChange(catId)}
          onHover={(catId) => handleCategoryHover(catId)}
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

    <!-- Sonuçlar Yükleme Çubuğu -->
    {#if $navigating}
      <div class="c-search-loading-bar" aria-hidden="true"></div>
    {/if}

    <!-- Sonuçlar Gövdesi -->
    <main class="c-search-body" class:is-loading={Boolean($navigating)}>
      <!-- 1. Hızlı Anlık Yanıt (Döviz, Hesap Makinesi - Yalnızca Web sekmesinde) -->
      {#if data.answer && (data.category === "general" || !data.category)}
        <div class="c-search-top-answer">
          <AnswerCard answer={data.answer} />
        </div>
      {/if}

      <!-- 2. Üst Bilgi Kartı (Yalnızca Web sekmesinde) -->
      {#if data.infoboxes && data.infoboxes.length > 0 && (data.category === "general" || !data.category)}
        <div class="c-search-top-knowledge">
          <KnowledgeCard infobox={data.infoboxes[0]} />
        </div>
      {/if}

      <!-- Sonuç Listesi -->
      <section class="c-search-list">
        {#if data.error}
          <div class="card u-p-lg">
            <p class="u-text-sm u-color-danger">{data.error}</p>
          </div>
        {:else if data.results.length === 0 && !data.answer && (!data.infoboxes || data.infoboxes.length === 0)}
          <div class="card u-p-lg">
            <p class="u-text-sm">
              <strong>{data.query}</strong> ile ilgili sonuç bulunamadı.
            </p>
          </div>
        {:else if data.category === "images"}
          <!-- Görsel Sonuçları Duvarı (Masonry Grid) -->
          <div class="c-search-images-grid">
            {#each data.results as item}
              <div class="c-search-image-item">
                <button
                  type="button"
                  class="c-search-image-card"
                  onclick={(e) => openImageLightbox(item, e)}
                  aria-label={item.title}
                >
                  {#if item.imgSrc}
                    <img
                      src={item.imgSrc}
                      alt={item.title}
                      class="c-search-image-card__img"
                      loading="lazy"
                    />
                  {/if}
                </button>
                <div class="c-search-image-meta">
                  <span class="c-search-image-meta__title" title={item.title}>
                    {item.title}
                  </span>
                  <a
                    href={item.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="c-search-image-meta__source"
                  >
                    {formatUrlBreadcrumb(item.url)}
                  </a>
                </div>
              </div>
            {/each}
          </div>
        {:else if data.category === "videos"}
          <!-- Video Sonuçları ve Gömülü Oynatıcı -->
          <div class="c-search-videos-grid">
            {#each data.results as item}
              {@const embedUrl = getYoutubeEmbedUrl(item.url)}
              <article class="c-search-video-card">
                {#if activeVideoEmbed === item.url && embedUrl}
                  <div class="c-search-video-embed-wrap">
                    <iframe
                      src={embedUrl}
                      title={item.title}
                      frameborder="0"
                      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                      allowfullscreen
                      class="c-search-video-iframe"
                    ></iframe>
                  </div>
                {:else}
                  <div class="c-search-video-thumb-wrap">
                    {#if item.thumbnail || item.imgSrc}
                      <img
                        src={item.thumbnail || item.imgSrc}
                        alt={item.title}
                        class="c-search-video-thumb"
                        loading="lazy"
                      />
                    {/if}
                    {#if embedUrl}
                      <button
                        type="button"
                        class="c-search-video-play-btn"
                        onclick={() => (activeVideoEmbed = item.url)}
                        aria-label="Videoyu Oynat"
                        title="Doğrudan Oynat"
                      >
                        ▶
                      </button>
                    {/if}
                  </div>
                {/if}
                <div class="c-search-video-info">
                  <a
                    href={item.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="c-search-video-title"
                  >
                    {item.title}
                  </a>
                  <p class="c-search-video-desc">{item.content}</p>
                  <span class="c-search-video-source">{formatUrlBreadcrumb(item.url)}</span>
                </div>
              </article>
            {/each}
          </div>
        {:else}
          <!-- Standart Web / Haber / Kod / Akademi Sonuçları -->
          {#each data.results as item, idx}
            {@const favicon = getFaviconUrl(item.url)}
            {@const dateBadge = formatDateSnippet(item.publishedDate)}
            <article
              class="c-search-item"
              class:is-keyboard-selected={idx === selectedResultIndex}
              data-index={idx}
            >
              <a
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                class="c-search-item__header"
              >
                {#if favicon}
                  <img
                    src={favicon}
                    alt=""
                    class="c-search-item__favicon"
                    loading="lazy"
                    onerror={(e) => { e.currentTarget.style.display = 'none'; }}
                  />
                {/if}
                <span class="c-search-item__domain">{getDomain(item.url)}</span>
                <span class="c-search-item__breadcrumb">{formatUrlBreadcrumb(item.url)}</span>
              </a>
              <a
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                class="c-search-item__title"
              >
                {@html highlightQuery(item.title, data.query)}
              </a>
              <p class="c-search-item__content">
                {#if dateBadge}
                  <span class="c-search-item__date-badge">{dateBadge} —</span>
                {/if}
                {@html highlightQuery(item.content, data.query)}
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

<!-- Görsel Büyüteç / Lightbox Modalı -->
{#if isImageLightboxOpen && selectedImage}
  <div
    class="c-search-lightbox-backdrop"
    onclick={closeImageLightbox}
    onkeydown={(e) => e.key === "Escape" && closeImageLightbox()}
    role="presentation"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="c-search-lightbox-content"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Görsel Önizleme"
      tabindex="-1"
    >
      <button
        type="button"
        class="c-search-lightbox-close"
        onclick={closeImageLightbox}
        aria-label="Kapat"
        title="Kapat (Esc)"
      >
        ✕
      </button>

      <div class="c-search-lightbox-img-wrap">
        <img
          src={selectedImage.imgSrc || selectedImage.thumbnail}
          alt={selectedImage.title}
          class="c-search-lightbox-img"
        />
      </div>

      <div class="c-search-lightbox-sidebar">
        <h3 class="c-search-lightbox-title">{selectedImage.title}</h3>
        <p class="c-search-lightbox-domain">{formatUrlBreadcrumb(selectedImage.url)}</p>

        <div class="c-search-lightbox-actions">
          <a
            href={selectedImage.url}
            target="_blank"
            rel="noopener noreferrer"
            class="btn btn--primary btn--block"
          >
            Sayfayı Ziyaret Et
          </a>
          {#if selectedImage.imgSrc}
            <a
              href={selectedImage.imgSrc}
              target="_blank"
              rel="noopener noreferrer"
              class="btn btn--secondary btn--block"
            >
              Tam Boyut Görsel
            </a>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
