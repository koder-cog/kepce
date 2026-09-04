<script>
  import { onMount } from "svelte";
  import { goto, preloadData } from "$app/navigation";
  import { page, navigating } from "$app/stores";
  import { icon } from "@/components/ui/icons.js";
  import SearchInfoModal from "@/components/features/search/SearchInfoModal.svelte";
  import SearchSettingsModal from "@/components/features/search/SearchSettingsModal.svelte";
  import KnowledgeCard from "@/components/features/search/KnowledgeCard.svelte";
  import AnswerCard from "@/components/features/search/AnswerCard.svelte";
  import KepceDirectCard from "@/components/features/search/KepceDirectCard.svelte";
  import SearchHome from "@/components/features/search/SearchHome.svelte";
  import SearchTopBar from "@/components/features/search/SearchTopBar.svelte";
  import SearchFilterBar from "@/components/features/search/SearchFilterBar.svelte";
  import SearchImageGrid from "@/components/features/search/SearchImageGrid.svelte";
  import SearchVideoGrid from "@/components/features/search/SearchVideoGrid.svelte";
  import SearchStandardResults from "@/components/features/search/SearchStandardResults.svelte";
  import SearchImageLightbox from "@/components/features/search/SearchImageLightbox.svelte";
  import SearchFooter from "@/components/features/search/SearchFooter.svelte";
  import { BANG_DEFINITIONS } from "$lib/search/bangs.js";
  import { searchPreferences } from "@/stores/searchPreferences.svelte.js";
  import {
    buildSearchUrl,
    checkInstantPreview,
    isAnswerPluginAllowed,
  } from "$lib/search/searchHelpers.js";

  let { data } = $props();

  let searchInput = $state("");
  let searchInputEl = $state(null);
  let isInfoOpen = $state(false);
  let isSettingsOpen = $state(false);
  let activeCategory = $state("general");

  $effect(() => {
    activeCategory = data.category || "general";
  });

  let searxData = $state(null);
  let isSearxLoading = $state(false);
  let isNavigatingToResults = $state(false);
  let scanStep = $state(0);
  let instantPreview = $state(null);

  $effect(() => {
    if (data.streamed?.searxData) {
      isSearxLoading = true;
      searxData = null;
      scanStep = 0;
      const startTime = Date.now();
      const interval = setInterval(() => {
        const elapsed = Date.now() - startTime;
        if (elapsed < 350) {
          scanStep = 0;
        } else if (elapsed < 750) {
          scanStep = 1;
        } else if (elapsed < 1200) {
          scanStep = 2;
        } else {
          scanStep = 3;
        }
      }, 100);

      data.streamed.searxData
        .then((res) => {
          clearInterval(interval);
          scanStep = 4;
          searxData = res;
          isSearxLoading = false;
          isNavigatingToResults = false;
        })
        .catch(() => {
          clearInterval(interval);
          searxData = {
            results: [],
            infoboxes: [],
            suggestions: [],
            corrections: [],
            error: "Arama servisi şu anda yanıt vermiyor.",
          };
          isSearxLoading = false;
          isNavigatingToResults = false;
        });

      return () => clearInterval(interval);
    } else {
      searxData = {
        results: data.results || [],
        infoboxes: data.infoboxes || [],
        suggestions: data.suggestions || [],
        corrections: data.corrections || [],
        answer: data.answer,
        numberOfResults: data.numberOfResults || 0,
        error: data.error,
      };
      isSearxLoading = false;
      isNavigatingToResults = false;
      scanStep = 4;
    }
  });

  let currentResults = $derived(searxData?.results || data.results || []);
  let currentInfoboxes = $derived(searxData?.infoboxes || data.infoboxes || []);
  let currentCorrections = $derived(
    searxData?.corrections || data.corrections || [],
  );
  let currentError = $derived(searxData?.error || data.error);

  let randomShortcuts = $state(
    BANG_DEFINITIONS.slice(0, 4).map((b) => ({
      prefix: b.prefix,
      label: b.name,
    })),
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
      let list = searchHistory.filter(
        (item) => item.toLowerCase() !== term.toLowerCase(),
      );
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
    e?.preventDefault();
    e?.stopPropagation();
    searchHistory = searchHistory.filter((t) => t !== term);
    try {
      localStorage.setItem(
        "kepce_search_history",
        JSON.stringify(searchHistory),
      );
    } catch {}
    if (searchHistory.length === 0) {
      isHistoryOpen = false;
    }
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

  onMount(() => {
    searchPreferences.init();
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
        isSuggestionsOpen = false;
        isHistoryOpen = false;
        instantPreview = null;
        if (searchInputEl) searchInputEl.blur();
      }
      return;
    }

    const items = currentResults || [];
    if (items.length === 0) return;

    if (e.key === "j" || e.key === "ArrowDown") {
      e.preventDefault();
      selectedResultIndex = Math.min(items.length - 1, selectedResultIndex + 1);
      scrollToSelectedResult();
    } else if (e.key === "k" || e.key === "ArrowUp") {
      e.preventDefault();
      selectedResultIndex = Math.max(0, selectedResultIndex - 1);
      scrollToSelectedResult();
    } else if (e.key === "Enter" && selectedResultIndex >= 0) {
      const selectedItem = items[selectedResultIndex];
      if (selectedItem?.url) {
        if (searchPreferences.openInNewTab) {
          window.open(selectedItem.url, "_blank", "noopener,noreferrer");
        } else {
          window.location.href = selectedItem.url;
        }
      }
    } else if (e.key === "Escape") {
      selectedResultIndex = -1;
    }
  }

  function scrollToSelectedResult() {
    if (selectedResultIndex < 0) return;
    const el = document.getElementById(`search-result-${selectedResultIndex}`);
    if (el) {
      el.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }

  let suggestions = $state([]);
  let isSuggestionsOpen = $state(false);
  let selectedSuggestionIndex = $state(-1);
  let debounceTimeout = null;

  async function fetchSuggestions(query) {
    if (searchPreferences.autocomplete === "off") {
      suggestions = [];
      isSuggestionsOpen = false;
      return;
    }

    const q = query.trim();
    if (!q) {
      suggestions = [];
      isSuggestionsOpen = false;
      return;
    }

    if (q.startsWith("!")) {
      const bangTerm = q.toLowerCase();
      const matchedBangs = BANG_DEFINITIONS.filter(
        (b) =>
          b.prefix.toLowerCase().startsWith(bangTerm) ||
          b.name.toLowerCase().includes(bangTerm.replace(/^!/, "")),
      )
        .slice(0, 6)
        .map((b) => ({
          isBang: true,
          prefix: b.prefix,
          label: b.name,
          displayText: `${b.prefix} ${b.name}`,
        }));

      if (matchedBangs.length > 0) {
        suggestions = matchedBangs;
        isSuggestionsOpen = true;
        selectedSuggestionIndex = -1;
        return;
      }
    }

    if (q.length < 2) {
      suggestions = [];
      isSuggestionsOpen = false;
      return;
    }

    try {
      const motorParam = searchPreferences.autocomplete
        ? `&motor=${encodeURIComponent(searchPreferences.autocomplete)}`
        : "";
      const res = await fetch(
        `${basePath}/autocompleter?q=${encodeURIComponent(q)}${motorParam}`,
      );
      if (res.ok) {
        const list = await res.json();
        if (Array.isArray(list) && list.length > 0) {
          suggestions = list.map((item) =>
            typeof item === "string"
              ? { isBang: false, displayText: item }
              : item,
          );
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
    instantPreview = checkInstantPreview(val, searchPreferences);
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
    if (e.key === "Escape") {
      isSuggestionsOpen = false;
      isHistoryOpen = false;
      instantPreview = null;
      selectedSuggestionIndex = -1;
      if (searchInputEl) searchInputEl.blur();
      return;
    }

    if (isSuggestionsOpen && suggestions.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        selectedSuggestionIndex =
          (selectedSuggestionIndex + 1) % suggestions.length;
        const current = suggestions[selectedSuggestionIndex];
        searchInput = current?.isBang
          ? current.prefix
          : current?.displayText || current || "";
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedSuggestionIndex =
          (selectedSuggestionIndex - 1 + suggestions.length) %
          suggestions.length;
        const current = suggestions[selectedSuggestionIndex];
        searchInput = current?.isBang
          ? current.prefix
          : current?.displayText || current || "";
      }
    }
  }

  function selectSuggestion(item) {
    if (typeof item === "object" && item?.isBang) {
      selectBang(item.prefix);
      isSuggestionsOpen = false;
      return;
    }
    const val = typeof item === "string" ? item : item?.displayText || "";
    searchInput = val;
    isSuggestionsOpen = false;
    isHistoryOpen = false;
    instantPreview = null;
    handleSearch();
  }

  function handleBlur() {
    setTimeout(() => {
      isSuggestionsOpen = false;
      isHistoryOpen = false;
      instantPreview = null;
    }, 200);
  }

  function handleWindowClick(e) {
    const target = e.target;
    if (
      target &&
      !target.closest(".c-search-bar") &&
      !target.closest(".c-search-home-form") &&
      !target.closest(".c-search-box")
    ) {
      isSuggestionsOpen = false;
      isHistoryOpen = false;
      instantPreview = null;
    }
  }

  function handleSearch(e) {
    e?.preventDefault();
    isSuggestionsOpen = false;
    isHistoryOpen = false;
    const q = searchInput.trim();
    if (!q) return;

    if (data.isHome) {
      isNavigatingToResults = true;
    }

    saveToHistory(q);

    const params = new URLSearchParams();
    params.set("q", q);

    const targetCategory = data.isHome
      ? searchPreferences.defaultCategory
      : data.category || searchPreferences.defaultCategory || "general";
    if (targetCategory && targetCategory !== "general") {
      params.set("kategori", targetCategory);
    }

    const targetLang = data.isHome
      ? searchPreferences.language
      : data.language || searchPreferences.language || "tr";
    if (targetLang && targetLang !== "tr") {
      params.set("dil", targetLang);
    }

    if (data.timeRange) {
      params.set("zaman", data.timeRange);
    }

    const targetSafe = data.isHome
      ? searchPreferences.safeSearch
      : data.safeSearch || searchPreferences.safeSearch || "1";
    if (targetSafe && targetSafe !== "1") {
      params.set("guvenli", targetSafe);
    }

    appendActiveCategoryFilters(params);

    goto(buildSearchUrl(params, isSubdomain));
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
    goto(buildSearchUrl(params, isSubdomain));
  }

  function handleCategoryHover(catId) {
    if (!data.query) return;
    const params = new URLSearchParams();
    params.set("q", data.query);
    if (catId !== "general") params.set("kategori", catId);
    if (data.language && data.language !== "tr")
      params.set("dil", data.language);
    if (data.timeRange) params.set("zaman", data.timeRange);
    if (data.safeSearch && data.safeSearch !== "1")
      params.set("guvenli", data.safeSearch);
    preloadData(buildSearchUrl(params, isSubdomain));
  }

  let hasActiveFilters = $derived(
    Boolean(
      (data.language && data.language !== "tr") ||
        data.timeRange ||
        (data.safeSearch && data.safeSearch !== "1") ||
        data.fileType ||
        data.siteFilter ||
        data.verbatim ||
        data.imgFormat ||
        data.imgSize ||
        data.imgColor ||
        data.imgLicense ||
        data.videoDuration ||
        data.videoQuality ||
        data.videoPlatform ||
        data.newsSort ||
        data.codeLang ||
        data.codePlatform ||
        data.scholarAccess ||
        data.scholarYear,
    ),
  );

  function appendActiveCategoryFilters(params) {
    if (data.fileType) params.set("dosya", data.fileType);
    if (data.siteFilter) params.set("site", data.siteFilter);
    if (data.verbatim) params.set("tam", "1");
    if (data.imgFormat) params.set("format", data.imgFormat);
    if (data.imgSize) params.set("boyut", data.imgSize);
    if (data.imgColor) params.set("renk", data.imgColor);
    if (data.imgLicense) params.set("lisans", data.imgLicense);
    if (data.videoDuration) params.set("sure", data.videoDuration);
    if (data.videoQuality) params.set("kalite", data.videoQuality);
    if (data.videoPlatform) params.set("platform", data.videoPlatform);
    if (data.newsSort) params.set("sirala", data.newsSort);
    if (data.codeLang) params.set("dil_prog", data.codeLang);
    if (data.codePlatform) params.set("kaynak", data.codePlatform);
    if (data.scholarAccess) params.set("erisim", data.scholarAccess);
    if (data.scholarYear) params.set("yil", data.scholarYear);
    return params;
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

    const filters = {
      dosya: key === "dosya" ? value : data.fileType,
      site: key === "site" ? value : data.siteFilter,
      tam: key === "tam" ? (value ? "1" : "") : data.verbatim ? "1" : "",
      format: key === "format" ? value : data.imgFormat,
      boyut: key === "boyut" ? value : data.imgSize,
      renk: key === "renk" ? value : data.imgColor,
      lisans: key === "lisans" ? value : data.imgLicense,
      sure: key === "sure" ? value : data.videoDuration,
      kalite: key === "kalite" ? value : data.videoQuality,
      platform: key === "platform" ? value : data.videoPlatform,
      sirala: key === "sirala" ? value : data.newsSort,
      dil_prog: key === "dil_prog" ? value : data.codeLang,
      kaynak: key === "kaynak" ? value : data.codePlatform,
      erisim: key === "erisim" ? value : data.scholarAccess,
      yil: key === "yil" ? value : data.scholarYear,
    };

    for (const [k, v] of Object.entries(filters)) {
      if (v) params.set(k, v);
    }

    goto(buildSearchUrl(params, isSubdomain));
  }

  function clearAllFilters() {
    const params = new URLSearchParams();
    params.set("q", data.query);
    if (data.category && data.category !== "general") {
      params.set("kategori", data.category);
    }
    goto(buildSearchUrl(params, isSubdomain));
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
    appendActiveCategoryFilters(params);
    if (pageNum > 1) params.set("sayfa", String(pageNum));
    return buildSearchUrl(params, isSubdomain);
  }

  let searchPageNumbers = $derived.by(() => {
    const current = Math.max(1, data.page || 1);
    if (current <= 3) {
      return [1, 2, 3, 4, 5];
    }
    return [current - 2, current - 1, current, current + 1, current + 2];
  });
</script>

<svelte:window onkeydown={handleGlobalKeydown} onclick={handleWindowClick} />

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
    <title
      >{searchPreferences.hideQueryInTitle
        ? "Kepçe Ara"
        : `${data.query} | Kepçe Ara`}</title
    >
    <meta name="description" content="{data.query} arama sonuçları." />
  {/if}
</svelte:head>

{#if data.isHome}
  <SearchHome
    isNavigating={$navigating || isNavigatingToResults}
    {searchInput}
    bind:searchInputEl
    {suggestions}
    {isSuggestionsOpen}
    {selectedSuggestionIndex}
    {randomShortcuts}
    {basePath}
    onSearch={handleSearch}
    onInput={handleInput}
    onKeydown={handleKeydown}
    onBlur={handleBlur}
    onSelectSuggestion={selectSuggestion}
    onSelectBang={selectBang}
    onOpenInfo={() => (isInfoOpen = true)}
    onOpenSettings={() => (isSettingsOpen = true)}
  />
{:else}
  <!-- ── 2. ARAMA SONUÇLARI EKRANI (Results) ────────────────── -->
  <div class="c-search-results">
    <!-- Üst Arama Çubuğu -->
    <SearchTopBar
      {basePath}
      query={data.query}
      category={data.category}
      {searchInput}
      bind:searchInputEl
      isLoading={$navigating || isSearxLoading}
      {searchHistory}
      {isHistoryOpen}
      {suggestions}
      {isSuggestionsOpen}
      {selectedSuggestionIndex}
      {instantPreview}
      onSearch={handleSearch}
      onInput={handleInput}
      onFocus={handleFocus}
      onKeydown={handleKeydown}
      onBlur={handleBlur}
      onSelectSuggestion={selectSuggestion}
      onClearHistory={clearHistory}
      onRemoveHistoryItem={removeHistoryItem}
      onOpenInfo={() => (isInfoOpen = true)}
      onOpenSettings={() => (isSettingsOpen = true)}
    />

    <!-- Kategori ve Filtre Çubuğu -->
    <SearchFilterBar
      {activeCategory}
      {data}
      {hasActiveFilters}
      onCategoryChange={handleCategoryChange}
      onCategoryHover={handleCategoryHover}
      onFilterChange={handleFilterChange}
      onClearAllFilters={clearAllFilters}
    />

    <!-- Sonuçlar Yükleme Çubuğu -->
    {#if $navigating}
      <div class="c-search-loading-bar" aria-hidden="true"></div>
    {/if}

    <!-- Sonuçlar Gövdesi -->
    <main class="c-search-body" class:is-loading={Boolean($navigating)}>
      <!-- 0. Kepçe Doğrudan Platform Sonucu (Menü / Araç / Arşiv) -->
      {#if data.category === "general" || !data.category}
        <KepceDirectCard card={data.kepceCard} />
      {/if}

      <!-- 1. Hızlı Anlık Yanıt (Döviz, Hesap Makinesi - Yalnızca Web sekmesinde) -->
      {#if data.answer && (data.category === "general" || !data.category) && isAnswerPluginAllowed(data.answer, searchPreferences)}
        <div class="c-search-top-answer">
          <AnswerCard answer={data.answer} />
        </div>
      {/if}

      <!-- 2. Üst Bilgi Kartı (Yalnızca Web sekmesinde) -->
      {#if currentInfoboxes && currentInfoboxes.length > 0 && (data.category === "general" || !data.category)}
        <div class="c-search-top-knowledge">
          <KnowledgeCard infobox={currentInfoboxes[0]} />
        </div>
      {/if}

      <!-- Yazım Hatası Düzeltmesi (Bunu mu demek istediniz?) -->
      {#if currentCorrections && currentCorrections.length > 0}
        <div class="c-search-spelling-correction">
          <span>Bunu mu demek istediniz:</span>
          {#each currentCorrections as correction, i}
            <a
              href={buildSearchUrl(
                new URLSearchParams({
                  q: correction,
                  kategori: data.category || "general",
                }),
                isSubdomain,
              )}
              class="c-search-spelling-correction__link"
            >
              {correction}
            </a>
            {#if i < currentCorrections.length - 1},
            {/if}
          {/each}
          <span>?</span>
        </div>
      {/if}

      <!-- Sonuç Listesi -->
      <section class="c-search-list">
        {#if isSearxLoading}
          <!-- Canlı Motor Takipçisi -->
          <div class="c-search-live-tracker" aria-live="polite">
            <div class="c-search-live-tracker__header">
              <span class="c-search-live-tracker__pulse"></span>
              <span class="c-search-live-tracker__status">
                {#if scanStep === 0}
                  Vikipedi ve ansiklopedi taranıyor...
                {:else if scanStep === 1}
                  Bing ve web dizinleri taranıyor...
                {:else if scanStep === 2}
                  DuckDuckGo ve bağımsız kaynaklar taranıyor...
                {:else if scanStep === 3}
                  Sonuçlar analiz ediliyor ve sıralanıyor...
                {:else}
                  Sonuçlar hazırlandı!
                {/if}
              </span>
            </div>
            <div class="c-search-live-chips">
              <span
                class="c-search-chip"
                class:is-done={scanStep > 0}
                class:is-active={scanStep === 0}
              >
                {#if scanStep > 0}
                  <span class="c-search-chip__icon"
                    >{@html icon("check", 12)}</span
                  >
                {:else if scanStep === 0}
                  <span class="c-search-chip__spinner" aria-hidden="true"
                  ></span>
                {:else}
                  <span class="c-search-chip__bullet" aria-hidden="true"></span>
                {/if}
                Vikipedi
              </span>
              <span
                class="c-search-chip"
                class:is-done={scanStep > 1}
                class:is-active={scanStep === 1}
              >
                {#if scanStep > 1}
                  <span class="c-search-chip__icon"
                    >{@html icon("check", 12)}</span
                  >
                {:else if scanStep === 1}
                  <span class="c-search-chip__spinner" aria-hidden="true"
                  ></span>
                {:else}
                  <span class="c-search-chip__bullet" aria-hidden="true"></span>
                {/if}
                Bing
              </span>
              <span
                class="c-search-chip"
                class:is-done={scanStep > 2}
                class:is-active={scanStep === 2}
              >
                {#if scanStep > 2}
                  <span class="c-search-chip__icon"
                    >{@html icon("check", 12)}</span
                  >
                {:else if scanStep === 2}
                  <span class="c-search-chip__spinner" aria-hidden="true"
                  ></span>
                {:else}
                  <span class="c-search-chip__bullet" aria-hidden="true"></span>
                {/if}
                DuckDuckGo
              </span>
              <span
                class="c-search-chip"
                class:is-done={scanStep >= 3}
                class:is-active={scanStep === 3}
              >
                {#if scanStep >= 3}
                  <span class="c-search-chip__icon"
                    >{@html icon("check", 12)}</span
                  >
                {:else if scanStep === 3}
                  <span class="c-search-chip__spinner" aria-hidden="true"
                  ></span>
                {:else}
                  <span class="c-search-chip__bullet" aria-hidden="true"></span>
                {/if}
                Sıralama
              </span>
            </div>
          </div>

          <div class="c-search-skeletons" aria-hidden="true">
            {#each [1, 2, 3] as _}
              <div class="c-search-skeleton-card">
                <div class="c-search-skeleton-line is-source"></div>
                <div class="c-search-skeleton-line is-title"></div>
                <div class="c-search-skeleton-line is-desc-1"></div>
                <div class="c-search-skeleton-line is-desc-2"></div>
              </div>
            {/each}
          </div>
        {:else if currentError}
          <div class="card u-p-lg">
            <p class="u-text-sm u-color-danger">{currentError}</p>
          </div>
        {:else if currentResults.length === 0 && !data.answer && !data.kepceCard && (!currentInfoboxes || currentInfoboxes.length === 0)}
          <div class="c-search-no-results">
            <div class="c-search-no-results__icon">
              {@html icon("search", 32)}
            </div>
            <h2 class="c-search-no-results__title">
              "{data.query}" ile ilgili hiçbir sonuç bulunamadı.
            </h2>
            <ul class="c-search-no-results__tips">
              <li>Tüm kelimelerin doğru yazıldığından emin olun.</li>
              <li>
                Daha genel veya farklı anahtar sözcükler kullanmayı deneyin.
              </li>
              {#if data.language && data.language !== "all"}
                <li>
                  Bölge filtresini genişletin:
                  <button
                    type="button"
                    class="c-search-inline-btn"
                    onclick={() => handleFilterChange("dil", "all")}
                  >
                    Tüm Dillerde / Küresel Ara
                  </button>
                </li>
              {/if}
              {#if data.category && data.category !== "general"}
                <li>
                  <button
                    type="button"
                    class="c-search-inline-btn"
                    onclick={() => handleCategoryChange("general")}
                  >
                    Genel Web Sonuçlarına Dön
                  </button>
                </li>
              {/if}
            </ul>
          </div>
        {:else if data.category === "images"}
          <!-- Görsel Sonuçları Duvarı (Masonry Grid) -->
          <SearchImageGrid
            results={currentResults}
            onSelectImage={openImageLightbox}
          />
        {:else if data.category === "videos"}
          <!-- Video Sonuçları ve Gömülü Oynatıcı -->
          <SearchVideoGrid
            results={currentResults}
            bind:activeVideoEmbed
          />
        {:else}
          <!-- Standart Web / Haber / Kod / Akademi Sonuçları -->
          <SearchStandardResults
            results={currentResults}
            query={data.query}
            {selectedResultIndex}
          />
        {/if}

        <!-- Sayfalama -->
        {#if currentResults.length > 0}
          <nav class="pagination c-search-pagination" aria-label="Sayfalama">
            <a
              href={getPageUrl(Math.max(1, (data.page || 1) - 1))}
              class="pagination__btn pagination__btn--prev"
              class:is-disabled={(data.page || 1) <= 1}
              aria-disabled={(data.page || 1) <= 1}
            >
              {@html icon("chevronLeft", 18)}
              <span class="pagination__btn-text">Önceki</span>
            </a>

            <ul class="pagination__list">
              {#each searchPageNumbers as p}
                <li class="pagination__item">
                  {#if p === data.page}
                    <span class="pagination__btn is-active" aria-current="page"
                      >{p}</span
                    >
                  {:else}
                    <a
                      href={getPageUrl(p)}
                      class="pagination__btn"
                      aria-label="Sayfa {p}">{p}</a
                    >
                  {/if}
                </li>
              {/each}
            </ul>

            <a
              href={getPageUrl((data.page || 1) + 1)}
              class="pagination__btn pagination__btn--next"
            >
              <span class="pagination__btn-text">Sonraki</span>
              {@html icon("chevronRight", 18)}
            </a>
          </nav>
        {/if}
      </section>
    </main>

    <!-- 4 Kolonlu Dengeli Arama Footer'ı -->
    <SearchFooter {basePath} />
  </div>
{/if}

<!-- Modallar -->
<SearchInfoModal bind:isOpen={isInfoOpen} />
<SearchSettingsModal bind:isOpen={isSettingsOpen} />

<!-- Görsel Büyüteç / Lightbox Modalı -->
<SearchImageLightbox
  isOpen={isImageLightboxOpen}
  image={selectedImage}
  onClose={closeImageLightbox}
/>
