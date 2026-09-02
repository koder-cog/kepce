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

      data.streamed.searxData.then((res) => {
        clearInterval(interval);
        scanStep = 4;
        searxData = res;
        isSearxLoading = false;
        isNavigatingToResults = false;
      }).catch(() => {
        clearInterval(interval);
        searxData = { results: [], infoboxes: [], suggestions: [], corrections: [], error: "Arama servisi şu anda yanıt vermiyor." };
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
  let currentCorrections = $derived(searxData?.corrections || data.corrections || []);
  let currentSuggestions = $derived(searxData?.suggestions || data.suggestions || []);
  let currentError = $derived(searxData?.error || data.error);

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
    e?.preventDefault();
    e?.stopPropagation();
    searchHistory = searchHistory.filter((t) => t !== term);
    try {
      localStorage.setItem("kepce_search_history", JSON.stringify(searchHistory));
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

  const REGION_OPTIONS = [
    { value: "all", label: "Tüm bölgeler" },
    { value: "tr", label: "Türkiye" },
    { value: "de-DE", label: "Almanya" },
    { value: "en-US", label: "Amerika Birleşik Devletleri" },
    { value: "az", label: "Azerbaycan" },
    { value: "en-GB", label: "Birleşik Krallık" },
    { value: "pt-BR", label: "Brezilya" },
    { value: "zh-CN", label: "Çin" },
    { value: "fr-FR", label: "Fransa" },
    { value: "ko-KR", label: "Güney Kore" },
    { value: "nl-NL", label: "Hollanda" },
    { value: "es-ES", label: "İspanya" },
    { value: "sv-SE", label: "İsveç" },
    { value: "it-IT", label: "İtalya" },
    { value: "ja-JP", label: "Japonya" },
    { value: "en-CA", label: "Kanada" },
    { value: "pl-PL", label: "Polonya" },
    { value: "ru-RU", label: "Rusya" },
    { value: "ar-SA", label: "Suudi Arabistan" },
    { value: "el-GR", label: "Yunanistan" },
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
        isSuggestionsOpen = false;
        isHistoryOpen = false;
        instantPreview = null;
        if (searchInputEl) searchInputEl.blur();
      }
      return;
    }

    // Arama sonuçları listesi klavye ile gezinti (j: aşağı, k: yukarı, Enter: aç, Esc: seçimi kaldır)
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
        window.open(selectedItem.url, "_blank", "noopener,noreferrer");
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
    const q = query.trim();
    if (!q) {
      suggestions = [];
      isSuggestionsOpen = false;
      return;
    }

    // 1. !bang kısayol tamamlama kontrolü
    if (q.startsWith("!")) {
      const bangTerm = q.toLowerCase();
      const matchedBangs = BANG_DEFINITIONS.filter(
        (b) =>
          b.prefix.toLowerCase().startsWith(bangTerm) ||
          b.name.toLowerCase().includes(bangTerm.replace(/^!/, ""))
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
      const res = await fetch(`${basePath}/autocompleter?q=${encodeURIComponent(q)}`);
      if (res.ok) {
        const list = await res.json();
        if (Array.isArray(list) && list.length > 0) {
          suggestions = list.map((item) =>
            typeof item === "string" ? { isBang: false, displayText: item } : item
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

  function checkInstantPreview(val) {
    const q = (val || "").trim().toLowerCase();
    if (!q) {
      instantPreview = null;
      return;
    }
    if (/^[\d\s.,+\-*/()^%]+$/.test(q) && /[+\-*/^%]/.test(q)) {
      try {
        const sanitized = q.replace(/,/g, ".").replace(/\^/g, "**");
        if (!/[a-zA-Z_$]/.test(sanitized)) {
          // eslint-disable-next-line no-new-func
          const result = Function(`'use strict'; return (${sanitized})`)();
          if (typeof result === "number") {
            let resText = "";
            if (isNaN(result)) {
              resText = "Tanımsız (0/0 belirsizliği)";
            } else if (!isFinite(result)) {
              resText = "Tanımsız (Sıfıra bölünemez)";
            } else {
              resText = result.toLocaleString("tr-TR", { maximumFractionDigits: 6 });
            }
            instantPreview = {
              badge: "Hesaplama",
              text: `${q} = ${resText}`,
            };
            return;
          }
        }
      } catch {}
    }
    const mCur = q.match(/^(\d+(?:[.,]\d+)?)\s*(dolar|usd|\$|euro|eur|€)/i);
    if (mCur) {
      const amt = parseFloat(mCur[1].replace(",", "."));
      const cur = mCur[2].toLowerCase();
      const rate = cur.includes("e") || cur.includes("€") ? 52.5 : 48.27;
      const total = amt * rate;
      instantPreview = {
        badge: "Döviz Tahmini",
        text: `≈ ${total.toLocaleString("tr-TR", { maximumFractionDigits: 2 })} ₺`,
      };
      return;
    }
    instantPreview = null;
  }

  function handleInput(e) {
    const val = e.target.value;
    searchInput = val;
    isHistoryOpen = !val;
    checkInstantPreview(val);
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
        selectedSuggestionIndex = (selectedSuggestionIndex + 1) % suggestions.length;
        const current = suggestions[selectedSuggestionIndex];
        searchInput = current?.isBang ? current.prefix : (current?.displayText || current || "");
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedSuggestionIndex = (selectedSuggestionIndex - 1 + suggestions.length) % suggestions.length;
        const current = suggestions[selectedSuggestionIndex];
        searchInput = current?.isBang ? current.prefix : (current?.displayText || current || "");
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
    if (target && !target.closest(".c-search-bar") && !target.closest(".c-search-home-form")) {
      isSuggestionsOpen = false;
      isHistoryOpen = false;
      instantPreview = null;
    }
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

    if (data.isHome) {
      isNavigatingToResults = true;
    }

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
      data.scholarYear
    )
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
      tam: key === "tam" ? (value ? "1" : "") : (data.verbatim ? "1" : ""),
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

    goto(buildSearchUrl(params));
  }

  function clearAllFilters() {
    const params = new URLSearchParams();
    params.set("q", data.query);
    if (data.category && data.category !== "general") {
      params.set("kategori", data.category);
    }
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
    appendActiveCategoryFilters(params);
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
    <title>{data.query} | Kepçe Ara</title>
    <meta name="description" content="{data.query} arama sonuçları." />
  {/if}
</svelte:head>

{#if data.isHome}
  <!-- ── 1. ANA ARAMA EKRANI (Home) ────────────────────────── -->
  <div class="c-search-home" class:is-navigating={$navigating || isNavigatingToResults}>
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
              {#if item.isBang}
                <li
                  class="c-search-autocomplete__item is-bang"
                  class:is-selected={idx === selectedSuggestionIndex}
                  onmousedown={() => selectSuggestion(item)}
                  role="option"
                  aria-selected={idx === selectedSuggestionIndex}
                >
                  <span class="c-search-autocomplete__bang-prefix">{item.prefix}</span>
                  <span class="c-search-autocomplete__bang-label">{item.label}</span>
                </li>
              {:else}
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
                  <span>{item.displayText || item}</span>
                </li>
              {/if}
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

            {#if $navigating || isSearxLoading}
              <div class="c-search-pulse-bar" aria-hidden="true"></div>
            {/if}

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

            {#if (isSuggestionsOpen && suggestions.length > 0) || instantPreview}
              <ul class="c-search-autocomplete" role="listbox">
                {#if instantPreview}
                  <li
                    class="c-search-autocomplete__item is-instant-preview"
                    onmousedown={() => handleSearch()}
                    role="option"
                    aria-selected="false"
                  >
                    <span class="c-search-autocomplete__preview-badge">{instantPreview.badge}</span>
                    <strong class="c-search-autocomplete__preview-text">{instantPreview.text}</strong>
                  </li>
                {/if}
                {#each suggestions as item, idx}
                  {#if item.isBang}
                    <li
                      class="c-search-autocomplete__item is-bang"
                      class:is-selected={idx === selectedSuggestionIndex}
                      onmousedown={() => selectSuggestion(item)}
                      role="option"
                      aria-selected={idx === selectedSuggestionIndex}
                    >
                      <span class="c-search-autocomplete__bang-prefix">{item.prefix}</span>
                      <span class="c-search-autocomplete__bang-label">{item.label}</span>
                    </li>
                  {:else}
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
                      <span>{item.displayText || item}</span>
                    </li>
                  {/if}
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
            aria-label="RSS Linki"
            title="RSS Linki"
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

      <!-- 2. Filtreler (Kepçe Dropdown Ghost - Kategoriye Duyarlı Zengin Filtreler) -->
      <div class="c-search-pill-filters">
        <!-- Ortak Filtreler: Dil ve Güvenli Arama -->
        <Dropdown
          variant="ghost"
          value={data.language || "tr"}
          options={REGION_OPTIONS}
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

        <!-- Zaman Filtresi (Kod hariç tüm sekmelerde geçerli) -->
        {#if activeCategory !== "it"}
          <Dropdown
            variant="ghost"
            value={data.timeRange || ""}
            options={[
              { value: "", label: "Zaman: Tümü" },
              { value: "day", label: "Son 24 saat" },
              { value: "week", label: "Son 1 hafta" },
              { value: "month", label: "Son 1 ay" },
              { value: "year", label: "Son 1 yıl" },
            ]}
            onChange={(val) => handleFilterChange("zaman", val)}
          />
        {/if}

        <!-- 🌐 Web Özel Filtreleri -->
        {#if activeCategory === "general"}
          <Dropdown
            variant="ghost"
            value={data.fileType || ""}
            options={[
              { value: "", label: "Format: Tüm dosyalar" },
              { value: "pdf", label: "Format: PDF (.pdf)" },
              { value: "docx", label: "Format: Word (.docx)" },
              { value: "pptx", label: "Format: Sunum (.pptx)" },
              { value: "xlsx", label: "Format: Tablo (.xlsx)" },
            ]}
            onChange={(val) => handleFilterChange("dosya", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.siteFilter || ""}
            options={[
              { value: "", label: "Kaynak: Tüm siteler" },
              { value: "edu.tr", label: "Kaynak: Üniversiteler (.edu.tr)" },
              { value: "gov.tr", label: "Kaynak: Kamu / Devlet (.gov.tr)" },
              { value: "org", label: "Kaynak: Vakıf / Dernek (.org)" },
            ]}
            onChange={(val) => handleFilterChange("site", val)}
          />

          <button
            type="button"
            class="c-search-filter-btn"
            class:is-active={data.verbatim}
            onclick={() => handleFilterChange("tam", !data.verbatim)}
            title="Sadece arama kelimelerini harfi harfine içeren sonuçları getirir"
          >
            <span>"Tam Eşleşme"</span>
          </button>
        {/if}

        <!-- 🖼️ Görseller Özel Filtreleri -->
        {#if activeCategory === "images"}
          <Dropdown
            variant="ghost"
            value={data.imgFormat || ""}
            options={[
              { value: "", label: "Tür: Tüm formatlar" },
              { value: "transparent", label: "Tür: Şeffaf (Transparan PNG)" },
              { value: "gif", label: "Tür: Hareketli (GIF)" },
              { value: "svg", label: "Tür: Vektörel (SVG)" },
              { value: "jpeg", label: "Tür: Fotoğraf (JPG)" },
            ]}
            onChange={(val) => handleFilterChange("format", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.imgSize || ""}
            options={[
              { value: "", label: "Boyut: Tümü" },
              { value: "large", label: "Boyut: Büyük (Duvar Kağıdı)" },
              { value: "medium", label: "Boyut: Orta" },
              { value: "icon", label: "Boyut: Küçük / İkon" },
            ]}
            onChange={(val) => handleFilterChange("boyut", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.imgColor || ""}
            options={[
              { value: "", label: "Renk: Tümü" },
              { value: "color", label: "Renk: Renkli" },
              { value: "monochrome", label: "Renk: Siyah-Beyaz" },
              { value: "transparent", label: "Renk: Saydam Zemin" },
            ]}
            onChange={(val) => handleFilterChange("renk", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.imgLicense || ""}
            options={[
              { value: "", label: "Lisans: Tümü" },
              { value: "cc", label: "Lisans: Creative Commons" },
              { value: "commercial", label: "Lisans: Ticari Kullanım" },
            ]}
            onChange={(val) => handleFilterChange("lisans", val)}
          />
        {/if}

        <!-- 🎬 Videolar Özel Filtreleri -->
        {#if activeCategory === "videos"}
          <Dropdown
            variant="ghost"
            value={data.videoDuration || ""}
            options={[
              { value: "", label: "Süre: Tüm süreler" },
              { value: "short", label: "Süre: Kısa (< 4 dk)" },
              { value: "medium", label: "Süre: Orta (4 - 20 dk)" },
              { value: "long", label: "Süre: Uzun (> 20 dk)" },
            ]}
            onChange={(val) => handleFilterChange("sure", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.videoQuality || ""}
            options={[
              { value: "", label: "Kalite: Tüm kaliteler" },
              { value: "hd", label: "Kalite: Yüksek Kalite (HD/4K)" },
              { value: "sd", label: "Kalite: Standart (SD)" },
            ]}
            onChange={(val) => handleFilterChange("kalite", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.videoPlatform || ""}
            options={[
              { value: "", label: "Platform: Tümü" },
              { value: "youtube", label: "Platform: YouTube" },
              { value: "vimeo", label: "Platform: Vimeo" },
              { value: "dailymotion", label: "Platform: Dailymotion" },
            ]}
            onChange={(val) => handleFilterChange("platform", val)}
          />
        {/if}

        <!-- 📰 Haberler Özel Filtreleri -->
        {#if activeCategory === "news"}
          <Dropdown
            variant="ghost"
            value={data.newsSort || ""}
            options={[
              { value: "", label: "Sıralama: Alakaya göre" },
              { value: "date", label: "Sıralama: Tarihe göre (En yeni)" },
            ]}
            onChange={(val) => handleFilterChange("sirala", val)}
          />
        {/if}

        <!-- 💻 Kod & IT Özel Filtreleri -->
        {#if activeCategory === "it"}
          <Dropdown
            variant="ghost"
            value={data.codeLang || ""}
            options={[
              { value: "", label: "Dil: Tüm diller" },
              { value: "rust", label: "Dil: Rust" },
              { value: "javascript", label: "Dil: JavaScript / TS" },
              { value: "python", label: "Dil: Python" },
              { value: "go", label: "Dil: Go" },
              { value: "cpp", label: "Dil: C / C++" },
            ]}
            onChange={(val) => handleFilterChange("dil_prog", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.codePlatform || ""}
            options={[
              { value: "", label: "Platform: Tümü" },
              { value: "github", label: "Platform: GitHub" },
              { value: "stackoverflow", label: "Platform: StackOverflow" },
              { value: "gitlab", label: "Platform: GitLab" },
            ]}
            onChange={(val) => handleFilterChange("kaynak", val)}
          />
        {/if}

        <!-- 🎓 Akademi Özel Filtreleri -->
        {#if activeCategory === "science"}
          <Dropdown
            variant="ghost"
            value={data.scholarAccess || ""}
            options={[
              { value: "", label: "Erişim: Tüm makaleler" },
              { value: "open", label: "Erişim: Açık Erişim (Tam PDF)" },
            ]}
            onChange={(val) => handleFilterChange("erisim", val)}
          />

          <Dropdown
            variant="ghost"
            value={data.scholarYear || ""}
            options={[
              { value: "", label: "Yıl: Tüm yıllar" },
              { value: "2026", label: "Yıl: 2026'dan beri" },
              { value: "2024", label: "Yıl: 2024'ten beri" },
              { value: "2020", label: "Yıl: 2020'den beri" },
            ]}
            onChange={(val) => handleFilterChange("yil", val)}
          />
        {/if}

        <!-- Filtreleri Temizle Butonu -->
        {#if hasActiveFilters}
          <button
            type="button"
            class="c-search-filter-btn c-search-filter-clear"
            onclick={clearAllFilters}
            title="Tüm filtreleri sıfırla"
          >
            {@html icon("close", 12)}
            <span>Filtreleri Temizle</span>
          </button>
        {/if}
      </div>
    </div>

    <!-- Sonuçlar Yükleme Çubuğu -->
    {#if $navigating}
      <div class="c-search-loading-bar" aria-hidden="true"></div>
    {/if}

    <!-- Sonuçlar Gövdesi -->
    <main class="c-search-body" class:is-loading={Boolean($navigating)}>
      <!-- 0. Kepçe Doğrudan Platform Sonucu (Menü / Araç / Arşiv) -->
      {#if data.kepceCard && (data.category === "general" || !data.category)}
        <aside class="c-kepce-direct-card" aria-label="Kepçe Sonucu">
          <div class="c-kepce-direct-card__header">
            <div class="c-kepce-direct-card__badge-group">
              <span class="c-kepce-direct-card__brand">🥣 KEPÇE DOĞRUDAN SONUÇ</span>
              <span class="c-kepce-direct-card__badge">{data.kepceCard.badge}</span>
            </div>
            <span class="c-kepce-direct-card__source">kepce.org</span>
          </div>
          <div class="c-kepce-direct-card__body">
            <h2 class="c-kepce-direct-card__title">
              <a href={data.kepceCard.href}>{data.kepceCard.title}</a>
            </h2>
            <p class="c-kepce-direct-card__subtitle">{data.kepceCard.subtitle}</p>
            <p class="c-kepce-direct-card__desc">{data.kepceCard.description}</p>
            {#if data.kepceCard.type === "city_menu"}
              <div class="c-kepce-direct-card__features">
                <span class="c-kepce-direct-card__feature-chip">🍲 4 Çeşit Tabldot Menü</span>
                <span class="c-kepce-direct-card__feature-chip">⏰ Sabah & Akşam Saatleri</span>
                <span class="c-kepce-direct-card__feature-chip">📊 Kalori & Fiyat Takibi</span>
              </div>
            {/if}
          </div>
          <div class="c-kepce-direct-card__footer">
            <a href={data.kepceCard.href} class="c-kepce-direct-card__btn">
              <span>{data.kepceCard.cta}</span>
              {@html icon("arrowRight", 14)}
            </a>
          </div>
        </aside>
      {/if}

      <!-- 1. Hızlı Anlık Yanıt (Döviz, Hesap Makinesi - Yalnızca Web sekmesinde) -->
      {#if data.answer && (data.category === "general" || !data.category)}
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
              href={buildSearchUrl(new URLSearchParams({ q: correction, kategori: data.category || 'general' }))}
              class="c-search-spelling-correction__link"
            >
              {correction}
            </a>
            {#if i < currentCorrections.length - 1}, {/if}
          {/each}
          <span>?</span>
        </div>
      {/if}

      <!-- Sonuç Listesi -->
      <section class="c-search-list">
        {#if isSearxLoading}
          <!-- Canlı Motor Takipçisi (Canlı Radar Pulse & Akıllı Şov) -->
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
              <span class="c-search-chip" class:is-done={scanStep > 0} class:is-active={scanStep === 0}>
                {#if scanStep > 0}
                  <span class="c-search-chip__icon">{@html icon("check", 12)}</span>
                {:else if scanStep === 0}
                  <span class="c-search-chip__spinner" aria-hidden="true"></span>
                {:else}
                  <span class="c-search-chip__bullet" aria-hidden="true"></span>
                {/if}
                Vikipedi
              </span>
              <span class="c-search-chip" class:is-done={scanStep > 1} class:is-active={scanStep === 1}>
                {#if scanStep > 1}
                  <span class="c-search-chip__icon">{@html icon("check", 12)}</span>
                {:else if scanStep === 1}
                  <span class="c-search-chip__spinner" aria-hidden="true"></span>
                {:else}
                  <span class="c-search-chip__bullet" aria-hidden="true"></span>
                {/if}
                Bing
              </span>
              <span class="c-search-chip" class:is-done={scanStep > 2} class:is-active={scanStep === 2}>
                {#if scanStep > 2}
                  <span class="c-search-chip__icon">{@html icon("check", 12)}</span>
                {:else if scanStep === 2}
                  <span class="c-search-chip__spinner" aria-hidden="true"></span>
                {:else}
                  <span class="c-search-chip__bullet" aria-hidden="true"></span>
                {/if}
                DuckDuckGo
              </span>
              <span class="c-search-chip" class:is-done={scanStep >= 3} class:is-active={scanStep === 3}>
                {#if scanStep >= 3}
                  <span class="c-search-chip__icon">{@html icon("check", 12)}</span>
                {:else if scanStep === 3}
                  <span class="c-search-chip__spinner" aria-hidden="true"></span>
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
            <h2 class="c-search-no-results__title">"{data.query}" ile ilgili hiçbir sonuç bulunamadı.</h2>
            <ul class="c-search-no-results__tips">
              <li>Tüm kelimelerin doğru yazıldığından emin olun.</li>
              <li>Daha genel veya farklı anahtar sözcükler kullanmayı deneyin.</li>
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
          <div class="c-search-images-grid">
            {#each currentResults as item, idx}
              {@const ratio = item.width && item.height ? `${item.width} / ${item.height}` : "4 / 3"}
              <div class="c-search-image-item" style="--index: {idx}">
                <button
                  type="button"
                  class="c-search-image-card"
                  style="--aspect-ratio: {ratio}"
                  onclick={(e) => openImageLightbox(item, e)}
                  aria-label={item.title}
                >
                  {#if item.thumbnailSrc || item.imgSrc}
                    <img
                      src={item.thumbnailSrc || item.imgSrc}
                      alt={item.title}
                      class="c-search-image-card__img"
                      loading="lazy"
                      decoding="async"
                      onerror={(e) => {
                        e.currentTarget.closest(".c-search-image-item")?.classList.add("is-img-error");
                      }}
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
            {#each currentResults as item, idx}
              {@const embedUrl = getYoutubeEmbedUrl(item.url)}
              <article class="c-search-video-card" style="--index: {idx}">
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
          {#each currentResults as item, idx}
            {@const favicon = getFaviconUrl(item.url)}
            {@const dateBadge = formatDateSnippet(item.publishedDate)}
            <article
              id="search-result-{idx}"
              class="c-search-item"
              class:is-keyboard-selected={idx === selectedResultIndex}
              data-index={idx}
              style="--index: {idx}"
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
        {#if currentResults.length > 0}
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
