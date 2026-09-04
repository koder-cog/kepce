/**
 * Kepçe Ara - Arama Tercihleri Reaktif Store (Svelte 5 Runes)
 * ==========================================================
 * Tüm arama ayarlarını tek merkezden yönetir, localStorage ile senkronize tutar.
 */

const DEFAULT_ENGINES = {
  google: true,
  bing: true,
  duckduckgo: true,
  brave: true,
  startpage: false,
  wikipedia: true,
  wikidata: true,
  wikihow: false,
  reddit: true,
  github: true,
  stackoverflow: true,
  youtube: true,
  vimeo: false,
  unsplash: true,
};

class SearchPreferencesStore {
  defaultCategory = $state("general");
  language = $state("tr");
  autocomplete = $state("duckduckgo");
  faviconResolver = $state("duckduckgo");
  safeSearch = $state("1");
  pluginCalculator = $state(true);
  pluginSelfInfo = $state(true);
  pluginTimezones = $state(true);
  pluginUnitConverter = $state(true);

  theme = $state("sistem");
  openInNewTab = $state(false);
  compactResults = $state(false);
  infiniteScroll = $state(false);
  resultsPerPage = $state("10");

  httpMethod = $state("POST");
  trackerRemover = $state(true);
  hideQueryInTitle = $state(false);
  engines = $state({ ...DEFAULT_ENGINES });

  isLoaded = $state(false);

  // Derived properties for link behavior
  linkTarget = $derived(this.openInNewTab ? "_blank" : undefined);
  linkRel = $derived(this.openInNewTab ? "noopener noreferrer" : undefined);

  init() {
    if (typeof window === "undefined" || this.isLoaded) return;

    try {
      this.defaultCategory = localStorage.getItem("kepce_search_category") || "general";
      this.language = localStorage.getItem("kepce_search_lang") || "tr";
      this.autocomplete = localStorage.getItem("kepce_search_autocomplete") || "duckduckgo";
      this.faviconResolver = localStorage.getItem("kepce_search_favicons") || "duckduckgo";
      this.safeSearch = localStorage.getItem("kepce_search_safe") || "1";
      this.pluginCalculator = localStorage.getItem("kepce_search_plugin_calc") !== "false";
      this.pluginSelfInfo = localStorage.getItem("kepce_search_plugin_ip") !== "false";
      this.pluginTimezones = localStorage.getItem("kepce_search_plugin_time") !== "false";
      this.pluginUnitConverter = localStorage.getItem("kepce_search_plugin_unit") !== "false";

      this.theme = localStorage.getItem("renkTercihi") || "sistem";
      this.openInNewTab = localStorage.getItem("kepce_search_new_tab") === "true";
      this.compactResults = localStorage.getItem("kepce_search_compact") === "true";
      this.infiniteScroll = localStorage.getItem("kepce_search_infinite") === "true";
      this.resultsPerPage = localStorage.getItem("kepce_search_per_page") || "10";

      this.httpMethod = localStorage.getItem("kepce_search_method") || "POST";
      this.trackerRemover = localStorage.getItem("kepce_search_tracker_remover") !== "false";
      this.hideQueryInTitle = localStorage.getItem("kepce_search_hide_title") === "true";

      const enginesRaw = localStorage.getItem("kepce_search_engines");
      if (enginesRaw) {
        this.engines = { ...DEFAULT_ENGINES, ...JSON.parse(enginesRaw) };
      }

      if (this.compactResults) {
        document.documentElement.classList.add("is-compact-results");
      }
    } catch (_) {}

    this.isLoaded = true;
  }

  setOpenInNewTab(val) {
    this.openInNewTab = Boolean(val);
    if (typeof window !== "undefined") {
      try {
        localStorage.setItem("kepce_search_new_tab", String(this.openInNewTab));
      } catch (_) {}
    }
  }

  setCompactResults(val) {
    this.compactResults = Boolean(val);
    if (typeof window !== "undefined") {
      try {
        localStorage.setItem("kepce_search_compact", String(this.compactResults));
        document.documentElement.classList.toggle("is-compact-results", this.compactResults);
      } catch (_) {}
    }
  }

  setTheme(newTheme) {
    this.theme = newTheme;
    if (typeof window !== "undefined") {
      try {
        localStorage.setItem("renkTercihi", newTheme);
        if (typeof window.applyTheme === "function") {
          window.applyTheme(newTheme);
        }
      } catch (_) {}
    }
  }

  setLanguage(val) {
    this.language = val;
    if (typeof window !== "undefined") {
      try {
        localStorage.setItem("kepce_search_lang", val);
      } catch (_) {}
    }
  }

  setSafeSearch(val) {
    this.safeSearch = val;
    if (typeof window !== "undefined") {
      try {
        localStorage.setItem("kepce_search_safe", val);
      } catch (_) {}
    }
  }

  setFaviconResolver(val) {
    this.faviconResolver = val;
    if (typeof window !== "undefined") {
      try {
        localStorage.setItem("kepce_search_favicons", val);
      } catch (_) {}
    }
  }

  setInfiniteScroll(val) {
    this.infiniteScroll = Boolean(val);
    if (typeof window !== "undefined") {
      try {
        localStorage.setItem("kepce_search_infinite", String(this.infiniteScroll));
      } catch (_) {}
    }
  }

  persistAll() {
    if (typeof window === "undefined") return;
    try {
      localStorage.setItem("kepce_search_category", this.defaultCategory);
      localStorage.setItem("kepce_search_lang", this.language);
      localStorage.setItem("kepce_search_autocomplete", this.autocomplete);
      localStorage.setItem("kepce_search_favicons", this.faviconResolver);
      localStorage.setItem("kepce_search_safe", this.safeSearch);
      localStorage.setItem("kepce_search_plugin_calc", String(this.pluginCalculator));
      localStorage.setItem("kepce_search_plugin_ip", String(this.pluginSelfInfo));
      localStorage.setItem("kepce_search_plugin_time", String(this.pluginTimezones));
      localStorage.setItem("kepce_search_plugin_unit", String(this.pluginUnitConverter));

      localStorage.setItem("renkTercihi", this.theme);
      localStorage.setItem("kepce_search_new_tab", String(this.openInNewTab));
      localStorage.setItem("kepce_search_compact", String(this.compactResults));
      localStorage.setItem("kepce_search_infinite", String(this.infiniteScroll));
      localStorage.setItem("kepce_search_per_page", this.resultsPerPage);

      localStorage.setItem("kepce_search_method", this.httpMethod);
      localStorage.setItem("kepce_search_tracker_remover", String(this.trackerRemover));
      localStorage.setItem("kepce_search_hide_title", String(this.hideQueryInTitle));
      localStorage.setItem("kepce_search_engines", JSON.stringify(this.engines));

      document.documentElement.classList.toggle("is-compact-results", this.compactResults);
    } catch (_) {}
  }
}

export const searchPreferences = new SearchPreferencesStore();
