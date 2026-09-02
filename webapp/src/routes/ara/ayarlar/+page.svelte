<script>
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { icon } from "@/components/ui/icons.js";
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { showToast } from "@/components/ui/toast.js";

  let isSubdomain = $derived(
    $page.url.hostname.startsWith("ara.") ||
      $page.url.hostname === "ara.localhost",
  );
  let basePath = $derived(isSubdomain ? "" : "/ara");

  // ── State ──────────────────────────────────────────────────
  let activeTab = $state("general");

  // 1. Genel
  let defaultCategory = $state("general");
  let language = $state("tr");
  let autocomplete = $state("duckduckgo");
  let faviconResolver = $state("duckduckgo");
  let safeSearch = $state("1");
  let pluginCalculator = $state(true);
  let pluginSelfInfo = $state(true);
  let pluginTimezones = $state(true);
  let pluginUnitConverter = $state(true);

  // 2. Görünüm
  let theme = $state("sistem");
  let openInNewTab = $state(false);
  let compactResults = $state(false);
  let infiniteScroll = $state(false);
  let resultsPerPage = $state("10");

  // 3. Gizlilik
  let httpMethod = $state("POST");
  let trackerRemover = $state(true);
  let hideQueryInTitle = $state(false);

  // 4. Arama Motorları
  let engines = $state({
    // Genel
    google: true,
    bing: true,
    duckduckgo: true,
    brave: true,
    startpage: false,
    // Ansiklopedi
    wikipedia: true,
    wikidata: true,
    wikihow: false,
    // Topluluk & Kod
    reddit: true,
    github: true,
    stackoverflow: true,
    // Medya
    youtube: true,
    vimeo: false,
    unsplash: true,
  });

  // 5. İçe/Dışa Aktarma
  let importHashInput = $state("");
  let preferencesHash = $state("");
  let shareUrl = $state("");

  const TABS = [
    { value: "general", label: "Genel" },
    { value: "ui", label: "Görünüm" },
    { value: "engines", label: "Motorlar" },
    { value: "privacy", label: "Gizlilik" },
    { value: "data", label: "Veri ve Aktarma" },
  ];

  const CATEGORY_OPTIONS = [
    { value: "general", label: "Genel" },
    { value: "images", label: "Görseller" },
    { value: "videos", label: "Videolar" },
    { value: "news", label: "Haberler" },
    { value: "it", label: "BT ve Kod" },
    { value: "science", label: "Bilim" },
  ];

  const THEME_OPTIONS = [
    { value: "sistem", icon: icon("system", 18), label: "Sistem" },
    { value: "acik", icon: icon("sun", 18), label: "Açık" },
    { value: "koyu", icon: icon("moon", 18), label: "Koyu" },
  ];

  const LANG_OPTIONS = [
    { value: "tr", label: "Türkçe (tr)" },
    { value: "en", label: "English (en)" },
    { value: "all", label: "Tüm Diller" },
  ];

  const AUTOCOMPLETE_OPTIONS = [
    { value: "duckduckgo", label: "DuckDuckGo" },
    { value: "google", label: "Google" },
    { value: "wikipedia", label: "Vikipedi" },
    { value: "off", label: "Kapalı" },
  ];

  const FAVICON_OPTIONS = [
    { value: "duckduckgo", label: "DuckDuckGo" },
    { value: "google", label: "Google" },
    { value: "off", label: "Kapalı" },
  ];

  const SAFE_OPTIONS = [
    { value: "0", label: "Kapalı" },
    { value: "1", label: "Orta" },
    { value: "2", label: "Katı" },
  ];

  const RESULTS_PER_PAGE_OPTIONS = [
    { value: "10", label: "10 Sonuç" },
    { value: "20", label: "20 Sonuç" },
    { value: "50", label: "50 Sonuç" },
  ];

  const HTTP_METHOD_OPTIONS = [
    { value: "POST", label: "POST (Gizlilik Odaklı)" },
    { value: "GET", label: "GET (URL Paylaşılabilir)" },
  ];

  function getSerializedState() {
    return {
      defaultCategory,
      language,
      autocomplete,
      faviconResolver,
      safeSearch,
      pluginCalculator,
      pluginSelfInfo,
      pluginTimezones,
      pluginUnitConverter,
      theme,
      openInNewTab,
      compactResults,
      infiniteScroll,
      resultsPerPage,
      httpMethod,
      trackerRemover,
      hideQueryInTitle,
      engines,
    };
  }

  function updateHashes() {
    try {
      const data = getSerializedState();
      const base64 = btoa(unescape(encodeURIComponent(JSON.stringify(data))));
      preferencesHash = base64;
      if (typeof window !== "undefined") {
        shareUrl = `${window.location.origin}${basePath || "/ara"}/ayarlar?pref=${base64}`;
      }
    } catch (e) {}
  }

  function applyState(data) {
    if (!data) return;
    if (data.defaultCategory) defaultCategory = data.defaultCategory;
    if (data.language) language = data.language;
    if (data.autocomplete) autocomplete = data.autocomplete;
    if (data.faviconResolver) faviconResolver = data.faviconResolver;
    if (data.safeSearch) safeSearch = data.safeSearch;
    if (data.pluginCalculator !== undefined)
      pluginCalculator = data.pluginCalculator;
    if (data.pluginSelfInfo !== undefined) pluginSelfInfo = data.pluginSelfInfo;
    if (data.pluginTimezones !== undefined)
      pluginTimezones = data.pluginTimezones;
    if (data.pluginUnitConverter !== undefined)
      pluginUnitConverter = data.pluginUnitConverter;

    if (data.theme) {
      theme = data.theme;
      handleThemeChange(data.theme);
    }
    if (data.openInNewTab !== undefined) openInNewTab = data.openInNewTab;
    if (data.compactResults !== undefined) {
      compactResults = data.compactResults;
      document.documentElement.classList.toggle(
        "is-compact-results",
        compactResults,
      );
    }
    if (data.infiniteScroll !== undefined) infiniteScroll = data.infiniteScroll;
    if (data.resultsPerPage) resultsPerPage = data.resultsPerPage;
    if (data.httpMethod) httpMethod = data.httpMethod;
    if (data.trackerRemover !== undefined) trackerRemover = data.trackerRemover;
    if (data.hideQueryInTitle !== undefined)
      hideQueryInTitle = data.hideQueryInTitle;

    if (data.engines) {
      engines = { ...engines, ...data.engines };
    }

    persistAll();
    updateHashes();
  }

  function persistAll() {
    try {
      localStorage.setItem("kepce_search_category", defaultCategory);
      localStorage.setItem("kepce_search_lang", language);
      localStorage.setItem("kepce_search_autocomplete", autocomplete);
      localStorage.setItem("kepce_search_favicons", faviconResolver);
      localStorage.setItem("kepce_search_safe", safeSearch);
      localStorage.setItem(
        "kepce_search_plugin_calc",
        String(pluginCalculator),
      );
      localStorage.setItem("kepce_search_plugin_ip", String(pluginSelfInfo));
      localStorage.setItem("kepce_search_plugin_time", String(pluginTimezones));
      localStorage.setItem(
        "kepce_search_plugin_unit",
        String(pluginUnitConverter),
      );

      localStorage.setItem("renkTercihi", theme);
      localStorage.setItem("kepce_search_new_tab", String(openInNewTab));
      localStorage.setItem("kepce_search_compact", String(compactResults));
      localStorage.setItem("kepce_search_infinite", String(infiniteScroll));
      localStorage.setItem("kepce_search_per_page", resultsPerPage);

      localStorage.setItem("kepce_search_method", httpMethod);
      localStorage.setItem(
        "kepce_search_tracker_remover",
        String(trackerRemover),
      );
      localStorage.setItem("kepce_search_hide_title", String(hideQueryInTitle));

      localStorage.setItem("kepce_search_engines", JSON.stringify(engines));
    } catch (e) {}
  }

  onMount(() => {
    try {
      // 1. URL'den dış tercih içe aktarma
      const urlPref =
        $page.url.searchParams.get("pref") ||
        $page.url.searchParams.get("preferences");
      if (urlPref) {
        try {
          const jsonStr = decodeURIComponent(escape(atob(urlPref)));
          const data = JSON.parse(jsonStr);
          applyState(data);
          showToast("Arama tercihleri bağlantıdan başarıyla içe aktarıldı.", {
            type: "success",
          });
          return;
        } catch (err) {
          showToast("Geçersiz tercih bağlantısı.", { type: "error" });
        }
      }

      // 2. Yerel depolamadan yükleme
      defaultCategory =
        localStorage.getItem("kepce_search_category") || "general";
      language = localStorage.getItem("kepce_search_lang") || "tr";
      autocomplete =
        localStorage.getItem("kepce_search_autocomplete") || "duckduckgo";
      faviconResolver =
        localStorage.getItem("kepce_search_favicons") || "duckduckgo";
      safeSearch = localStorage.getItem("kepce_search_safe") || "1";
      pluginCalculator =
        localStorage.getItem("kepce_search_plugin_calc") !== "false";
      pluginSelfInfo =
        localStorage.getItem("kepce_search_plugin_ip") !== "false";
      pluginTimezones =
        localStorage.getItem("kepce_search_plugin_time") !== "false";
      pluginUnitConverter =
        localStorage.getItem("kepce_search_plugin_unit") !== "false";

      theme = localStorage.getItem("renkTercihi") || "sistem";
      openInNewTab = localStorage.getItem("kepce_search_new_tab") === "true";
      compactResults = localStorage.getItem("kepce_search_compact") === "true";
      infiniteScroll = localStorage.getItem("kepce_search_infinite") === "true";
      resultsPerPage = localStorage.getItem("kepce_search_per_page") || "10";

      httpMethod = localStorage.getItem("kepce_search_method") || "POST";
      trackerRemover =
        localStorage.getItem("kepce_search_tracker_remover") !== "false";
      hideQueryInTitle =
        localStorage.getItem("kepce_search_hide_title") === "true";

      const savedEngines = localStorage.getItem("kepce_search_engines");
      if (savedEngines) {
        engines = { ...engines, ...JSON.parse(savedEngines) };
      }

      updateHashes();
    } catch (e) {}
  });

  function handleThemeChange(newTheme) {
    theme = newTheme;
    try {
      localStorage.setItem("renkTercihi", newTheme);
      if (
        typeof window !== "undefined" &&
        typeof window.applyTheme === "function"
      ) {
        window.applyTheme(newTheme);
      }
      updateHashes();
    } catch (e) {}
  }

  function handleGenericChange() {
    persistAll();
    updateHashes();
  }

  function toggleEngine(engineKey, enabled) {
    engines[engineKey] = enabled;
    handleGenericChange();
  }

  function resetAll() {
    defaultCategory = "general";
    language = "tr";
    autocomplete = "duckduckgo";
    faviconResolver = "duckduckgo";
    safeSearch = "1";
    pluginCalculator = true;
    pluginSelfInfo = true;
    pluginTimezones = true;
    pluginUnitConverter = true;

    handleThemeChange("sistem");
    openInNewTab = false;
    compactResults = false;
    infiniteScroll = false;
    resultsPerPage = "10";
    document.documentElement.classList.remove("is-compact-results");

    httpMethod = "POST";
    trackerRemover = true;
    hideQueryInTitle = false;

    engines = {
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

    persistAll();
    updateHashes();
    showToast("Tüm ayarlar varsayılana sıfırlandı.", { type: "info" });
  }

  async function copyText(text, successMsg) {
    try {
      await navigator.clipboard.writeText(text);
      showToast(successMsg, { type: "success" });
    } catch (e) {
      showToast("Kopyalama başarısız oldu.", { type: "error" });
    }
  }

  function importCustomHash() {
    if (!importHashInput.trim()) return;
    try {
      const jsonStr = decodeURIComponent(escape(atob(importHashInput.trim())));
      const data = JSON.parse(jsonStr);
      applyState(data);
      importHashInput = "";
      showToast("Tercihler başarıyla içe aktarıldı.", { type: "success" });
    } catch (err) {
      showToast("Geçersiz tercih hash verisi.", { type: "error" });
    }
  }
</script>

<svelte:head>
  <title>Ayarlar | Kepçe Ara</title>
  <meta
    name="description"
    content="Arama motoru tercihleri, motor yönetimi ve veri aktarımı."
  />
</svelte:head>

<div class="settings-page c-search-adv-page" id="settings-page">
  <h1 class="settings-page__title">Ayarlar</h1>

  <!-- ── 5'li Sekme Seçici ────────────────────────────────── -->
  <div class="c-search-adv-tabs-box">
    <SegmentedControl
      options={TABS}
      value={activeTab}
      onChange={(t) => (activeTab = t)}
    />
  </div>

  <!-- ── 1. GENEL (General) ───────────────────────────────── -->
  {#if activeTab === "general"}
    <section class="settings-section">
      <h2 class="settings-section__heading">Arama Tercihleri</h2>
      <div class="c-boxed-list">
        <!-- Varsayılan Kategori -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Varsayılan kategori</div>
            <div class="c-list-row__desc">
              Arama başladığında seçili gelen alan
            </div>
          </div>
          <div class="c-list-row__control">
            <Dropdown
              variant="ghost"
              value={defaultCategory}
              options={CATEGORY_OPTIONS}
              onChange={(val) => {
                defaultCategory = val;
                handleGenericChange();
              }}
            />
          </div>
        </label>

        <!-- Arama Dili -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Arama dili</div>
            <div class="c-list-row__desc">Sonuçların öncelikli dili</div>
          </div>
          <div class="c-list-row__control">
            <Dropdown
              variant="ghost"
              value={language}
              options={LANG_OPTIONS}
              onChange={(val) => {
                language = val;
                handleGenericChange();
              }}
            />
          </div>
        </label>

        <!-- Otomatik Tamamlama -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Otomatik tamamlama</div>
            <div class="c-list-row__desc">
              Arama kutusunda öneri getiren servis
            </div>
          </div>
          <div class="c-list-row__control">
            <Dropdown
              variant="ghost"
              value={autocomplete}
              options={AUTOCOMPLETE_OPTIONS}
              onChange={(val) => {
                autocomplete = val;
                handleGenericChange();
              }}
            />
          </div>
        </label>

        <!-- Favicon Çözümleyici -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Site simgeleri (Favicon)</div>
            <div class="c-list-row__desc">
              Sonuçların yanında site simgelerini gösterir
            </div>
          </div>
          <div class="c-list-row__control">
            <Dropdown
              variant="ghost"
              value={faviconResolver}
              options={FAVICON_OPTIONS}
              onChange={(val) => {
                faviconResolver = val;
                handleGenericChange();
              }}
            />
          </div>
        </label>

        <!-- Güvenli Arama -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Güvenli arama (SafeSearch)</div>
            <div class="c-list-row__desc">
              Yetişkin içerik filtreleme düzeyi
            </div>
          </div>
          <div class="c-list-row__control">
            <Dropdown
              variant="ghost"
              value={safeSearch}
              options={SAFE_OPTIONS}
              onChange={(val) => {
                safeSearch = val;
                handleGenericChange();
              }}
            />
          </div>
        </label>
      </div>
    </section>

    <!-- Özel Sorgular & Eklentiler -->
    <section class="settings-section">
      <h2 class="settings-section__heading">Anında Yanıt Eklentileri</h2>
      <div class="c-boxed-list">
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="plug-calc"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Hesap makinesi</div>
            <div class="c-list-row__desc">
              24 * 15 gibi matematiksel işlemlerde sonucu anında gösterir
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="plug-calc"
              class="c-input-hidden"
              checked={pluginCalculator}
              onchange={(e) => {
                pluginCalculator = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="plug-ip"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">IP ve bağlantı bilgisi</div>
            <div class="c-list-row__desc">
              "ip" veya "tarayıcı" sorgularında ağ detaylarını gösterir
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="plug-ip"
              class="c-input-hidden"
              checked={pluginSelfInfo}
              onchange={(e) => {
                pluginSelfInfo = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="plug-time"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Saat ve zaman dilimleri</div>
            <div class="c-list-row__desc">
              "tokyo saati" veya "saat kaç" sorgularında yerel saati gösterir
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="plug-time"
              class="c-input-hidden"
              checked={pluginTimezones}
              onchange={(e) => {
                pluginTimezones = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="plug-unit"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Birim ve kur çevirici</div>
            <div class="c-list-row__desc">
              "100 usd" veya "50 km" gibi çevirilerde anında hesaplar
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="plug-unit"
              class="c-input-hidden"
              checked={pluginUnitConverter}
              onchange={(e) => {
                pluginUnitConverter = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>

    <!-- ── 2. GÖRÜNÜM (UI) ─────────────────────────────────── -->
  {:else if activeTab === "ui"}
    <section class="settings-section">
      <h2 class="settings-section__heading">Tema ve Düzen</h2>
      <div class="c-boxed-list">
        <!-- Tema -->
        <div class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Tema</div>
            <div class="c-list-row__desc">Ara yüz renk teması</div>
          </div>
          <div class="c-list-row__control c-list-row__control--flexible">
            <SegmentedControl
              value={theme}
              variant="responsive"
              options={THEME_OPTIONS}
              onChange={handleThemeChange}
            />
          </div>
        </div>

        <!-- Kompakt Sonuçlar -->
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="ui-compact"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Kompakt sonuçlar</div>
            <div class="c-list-row__desc">
              Sonuç kartları arasındaki boşluğu daraltır
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="ui-compact"
              class="c-input-hidden"
              checked={compactResults}
              onchange={(e) => {
                compactResults = e.currentTarget.checked;
                document.documentElement.classList.toggle(
                  "is-compact-results",
                  compactResults,
                );
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <!-- Yeni Sekmede Aç -->
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="ui-new-tab"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Bağlantıları yeni sekmede aç</div>
            <div class="c-list-row__desc">
              Arama sonuçlarını yeni tarayıcı sekmesinde açar
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="ui-new-tab"
              class="c-input-hidden"
              checked={openInNewTab}
              onchange={(e) => {
                openInNewTab = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <!-- Sonsuz Kaydırma -->
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="ui-infinite"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Sonsuz kaydırma</div>
            <div class="c-list-row__desc">
              Sayfa sonuna gelince yeni sonuçları otomatik yükler
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="ui-infinite"
              class="c-input-hidden"
              checked={infiniteScroll}
              onchange={(e) => {
                infiniteScroll = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <!-- Sayfa Başına Sonuç -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Sayfa başına sonuç</div>
            <div class="c-list-row__desc">
              Her sayfada listelenecek sonuç sayısı
            </div>
          </div>
          <div class="c-list-row__control">
            <Dropdown
              variant="ghost"
              value={resultsPerPage}
              options={RESULTS_PER_PAGE_OPTIONS}
              onChange={(val) => {
                resultsPerPage = val;
                handleGenericChange();
              }}
            />
          </div>
        </label>
      </div>
    </section>

    <!-- ── 3. MOTORLAR (Engines) ────────────────────────────── -->
  {:else if activeTab === "engines"}
    <section class="settings-section">
      <h2 class="settings-section__heading">Genel Arama Motorları</h2>
      <div class="c-boxed-list">
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-google"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Google <span class="c-search-bang-pill">!g</span>
            </div>
            <div class="c-list-row__desc">Web arama sonuçları</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-google"
              class="c-input-hidden"
              checked={engines.google}
              onchange={(e) => toggleEngine("google", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-bing"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Bing <span class="c-search-bang-pill">!b</span>
            </div>
            <div class="c-list-row__desc">
              Microsoft web ve görsel arama indeksi
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-bing"
              class="c-input-hidden"
              checked={engines.bing}
              onchange={(e) => toggleEngine("bing", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-duckduckgo"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              DuckDuckGo <span class="c-search-bang-pill">!ddg</span>
            </div>
            <div class="c-list-row__desc">Gizlilik odaklı web arama motoru</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-duckduckgo"
              class="c-input-hidden"
              checked={engines.duckduckgo}
              onchange={(e) =>
                toggleEngine("duckduckgo", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-brave"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Brave Search <span class="c-search-bang-pill">!brave</span>
            </div>
            <div class="c-list-row__desc">Bağımsız web arama dizini</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-brave"
              class="c-input-hidden"
              checked={engines.brave}
              onchange={(e) => toggleEngine("brave", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-startpage"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Startpage <span class="c-search-bang-pill">!sp</span>
            </div>
            <div class="c-list-row__desc">Anonim Google arama sonuçları</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-startpage"
              class="c-input-hidden"
              checked={engines.startpage}
              onchange={(e) =>
                toggleEngine("startpage", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>

    <section class="settings-section">
      <h2 class="settings-section__heading">Ansiklopedi ve Bilgi</h2>
      <div class="c-boxed-list">
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-wikipedia"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Vikipedi <span class="c-search-bang-pill">!w</span>
            </div>
            <div class="c-list-row__desc">Özgür ansiklopedi maddeleri</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-wikipedia"
              class="c-input-hidden"
              checked={engines.wikipedia}
              onchange={(e) =>
                toggleEngine("wikipedia", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-wikidata"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Wikidata <span class="c-search-bang-pill">!wd</span>
            </div>
            <div class="c-list-row__desc">Açık yapılandırılmış veri tabanı</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-wikidata"
              class="c-input-hidden"
              checked={engines.wikidata}
              onchange={(e) =>
                toggleEngine("wikidata", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>

    <section class="settings-section">
      <h2 class="settings-section__heading">Topluluk ve Kod</h2>
      <div class="c-boxed-list">
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-reddit"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Reddit <span class="c-search-bang-pill">!r</span>
            </div>
            <div class="c-list-row__desc">
              Topluluk tartışmaları ve gönderiler
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-reddit"
              class="c-input-hidden"
              checked={engines.reddit}
              onchange={(e) => toggleEngine("reddit", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-github"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              GitHub <span class="c-search-bang-pill">!gh</span>
            </div>
            <div class="c-list-row__desc">
              Açık kaynak kod depoları ve dokümanlar
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-github"
              class="c-input-hidden"
              checked={engines.github}
              onchange={(e) => toggleEngine("github", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-stackoverflow"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Stack Overflow <span class="c-search-bang-pill">!so</span>
            </div>
            <div class="c-list-row__desc">Yazılım soru ve cevapları</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-stackoverflow"
              class="c-input-hidden"
              checked={engines.stackoverflow}
              onchange={(e) =>
                toggleEngine("stackoverflow", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>

    <section class="settings-section">
      <h2 class="settings-section__heading">Medya</h2>
      <div class="c-boxed-list">
        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-youtube"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              YouTube <span class="c-search-bang-pill">!yt</span>
            </div>
            <div class="c-list-row__desc">Video arama sonuçları</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-youtube"
              class="c-input-hidden"
              checked={engines.youtube}
              onchange={(e) => toggleEngine("youtube", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="eng-unsplash"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              Unsplash <span class="c-search-bang-pill">!unsplash</span>
            </div>
            <div class="c-list-row__desc">Ücretsiz stok fotoğraflar</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="eng-unsplash"
              class="c-input-hidden"
              checked={engines.unsplash}
              onchange={(e) =>
                toggleEngine("unsplash", e.currentTarget.checked)}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>

    <!-- ── 4. GİZLİLİK (Privacy) ────────────────────────────── -->
  {:else if activeTab === "privacy"}
    <section class="settings-section">
      <h2 class="settings-section__heading">Arama Yöntemi</h2>
      <div class="c-boxed-list">
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">HTTP yöntemi</div>
            <div class="c-list-row__desc">
              POST yöntemi sorguların tarayıcı geçmişine yazılmasını önler
            </div>
          </div>
          <div class="c-list-row__control">
            <Dropdown
              variant="ghost"
              value={httpMethod}
              options={HTTP_METHOD_OPTIONS}
              onChange={(val) => {
                httpMethod = val;
                handleGenericChange();
              }}
            />
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="priv-tracker"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              İzleyici temizleyici (Tracker Remover)
            </div>
            <div class="c-list-row__desc">
              Bağlantılardaki utm_ ve fbclid gibi izleme etiketlerini temizler
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="priv-tracker"
              class="c-input-hidden"
              checked={trackerRemover}
              onchange={(e) => {
                trackerRemover = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <label
          class="c-list-row c-list-row--clickable c-list-row--tall"
          for="priv-title"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Sekme başlığında sorguyu gizle</div>
            <div class="c-list-row__desc">
              Sekme başlığında arama terimini göstermez
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="priv-title"
              class="c-input-hidden"
              checked={hideQueryInTitle}
              onchange={(e) => {
                hideQueryInTitle = e.currentTarget.checked;
                handleGenericChange();
              }}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>

    <!-- ── 5. VERİ & AKTARMA (Data & Cookies) ────────────────── -->
  {:else if activeTab === "data"}
    <section class="settings-section">
      <h2 class="settings-section__heading">Tercihleri Dışa Aktar</h2>
      <div class="c-boxed-list">
        <!-- Hash Kopyalama -->
        <div class="c-list-row c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Tercih Kodu</div>
            <div class="c-list-row__desc">
              Mevcut ayarlarınızı içeren metin dizesi
            </div>
            <div class="c-search-adv-hash-preview">{preferencesHash}</div>
          </div>
          <div class="c-list-row__control">
            <button
              type="button"
              class="btn btn--sm btn--secondary btn--squish"
              onclick={() =>
                copyText(preferencesHash, "Tercih kodu kopyalandı.")}
            >
              Kopyala
            </button>
          </div>
        </div>

        <!-- Paylaşım URL'si -->
        <div class="c-list-row c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Paylaşım Bağlantısı</div>
            <div class="c-list-row__desc">
              Ayarlarınızı başka cihazda açan bağlantı
            </div>
          </div>
          <div class="c-list-row__control">
            <button
              type="button"
              class="btn btn--sm btn--secondary btn--squish"
              onclick={() =>
                copyText(shareUrl, "Paylaşım bağlantısı kopyalandı.")}
            >
              Bağlantıyı Kopyala
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h2 class="settings-section__heading">Tercihleri İçe Aktar</h2>
      <div class="c-boxed-list">
        <div class="c-list-row c-list-row--tall">
          <div class="c-list-row__info u-flex-1">
            <div class="c-list-row__title">Tercih Kodunu Yapıştırın</div>
            <div class="c-list-row__desc">
              Kopyaladığınız tercih metnini buraya yapıştırıp içe aktarın
            </div>
            <div class="u-mt-xs">
              <input
                type="text"
                class="c-input"
                placeholder="Tercih kodunu buraya yapıştırın..."
                bind:value={importHashInput}
              />
            </div>
          </div>
          <div class="c-list-row__control u-self-end">
            <button
              type="button"
              class="btn btn--sm btn--primary btn--squish"
              onclick={importCustomHash}
              disabled={!importHashInput.trim()}
            >
              İçe Aktar
            </button>
          </div>
        </div>
      </div>
    </section>
  {/if}

  <!-- ── Alt Eylem Butonları ───────────────────────────────── -->
  <div class="c-search-adv-footer-actions">
    <a href={basePath || "/"} class="btn btn--secondary btn--squish">
      {@html icon("chevronLeft", 16)}
      <span>Aramaya Dön</span>
    </a>

    <button type="button" class="btn btn--ghost btn--squish" onclick={resetAll}>
      Varsayılanlara Sıfırla
    </button>
  </div>
</div>
