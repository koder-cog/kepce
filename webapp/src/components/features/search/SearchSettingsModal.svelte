<script>
  import Modal from "@/components/ui/Modal.svelte";
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { icon } from "@/components/ui/icons.js";
  import { onMount } from "svelte";

  let { isOpen = $bindable(false) } = $props();

  let theme = $state("sistem");
  let language = $state("tr");
  let safeSearch = $state("1");
  let openInNewTab = $state(false);
  let compactResults = $state(false);
  let showFavicons = $state(true);
  let infiniteScroll = $state(false);

  const THEME_OPTIONS = [
    { value: "sistem", label: "Sistem" },
    { value: "acik", label: "Açık" },
    { value: "koyu", label: "Koyu" },
  ];

  const LANG_OPTIONS = [
    { value: "tr", label: "Türkçe" },
    { value: "en", label: "English" },
    { value: "all", label: "Tüm Diller" },
  ];

  const SAFE_OPTIONS = [
    { value: "1", label: "Orta" },
    { value: "2", label: "Katı" },
    { value: "0", label: "Kapalı" },
  ];

  onMount(() => {
    try {
      theme = localStorage.getItem("renkTercihi") || "sistem";
      language = localStorage.getItem("kepce_search_lang") || "tr";
      safeSearch = localStorage.getItem("kepce_search_safe") || "1";
      openInNewTab = localStorage.getItem("kepce_search_new_tab") === "true";
      compactResults = localStorage.getItem("kepce_search_compact") === "true";
      showFavicons = localStorage.getItem("kepce_search_favicons") !== "false";
      infiniteScroll = localStorage.getItem("kepce_search_infinite") === "true";
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
    } catch (e) {}
  }

  function updateLanguage(newLang) {
    language = newLang;
    try {
      localStorage.setItem("kepce_search_lang", newLang);
    } catch (e) {}
  }

  function updateSafeSearch(newSafe) {
    safeSearch = newSafe;
    try {
      localStorage.setItem("kepce_search_safe", newSafe);
    } catch (e) {}
  }

  function updateOpenInNewTab(val) {
    openInNewTab = val;
    try {
      localStorage.setItem("kepce_search_new_tab", String(val));
    } catch (e) {}
  }

  function updateCompactResults(val) {
    compactResults = val;
    try {
      localStorage.setItem("kepce_search_compact", String(val));
      document.documentElement.classList.toggle("is-compact-results", val);
    } catch (e) {}
  }

  function updateShowFavicons(val) {
    showFavicons = val;
    try {
      localStorage.setItem("kepce_search_favicons", String(val));
    } catch (e) {}
  }

  function updateInfiniteScroll(val) {
    infiniteScroll = val;
    try {
      localStorage.setItem("kepce_search_infinite", String(val));
    } catch (e) {}
  }

  function resetSettings() {
    handleThemeChange("sistem");
    updateLanguage("tr");
    updateSafeSearch("1");
    updateOpenInNewTab(false);
    updateCompactResults(false);
    updateShowFavicons(true);
    updateInfiniteScroll(false);
  }
</script>

{#if isOpen}
  <Modal
    options={{
      title: "Ayarlar",
      iconHtml: icon("settings", 24),
    }}
    onClose={() => (isOpen = false)}
  >
    {#snippet children()}
      <div class="c-search-settings-flow">
        <!-- ── 1. Görünüm (Appearance) ─────────────────────── -->
        <section class="c-search-settings-section">
          <h4>Görünüm</h4>

          <!-- Tema Seçimi -->
          <div class="c-search-settings-theme-ctrl">
            <SegmentedControl
              options={THEME_OPTIONS}
              value={theme}
              onChange={handleThemeChange}
            />
          </div>

          <!-- Kompakt Sonuçlar (Yazı solda, Switch sağda) -->
          <label class="c-search-settings-row" for="search-compact-switch">
            <span class="c-search-settings-label">Kompakt sonuçlar</span>
            <input
              type="checkbox"
              id="search-compact-switch"
              class="c-input-hidden"
              checked={compactResults}
              onchange={(e) => updateCompactResults(e.currentTarget.checked)}
            />
            <span class="c-switch" aria-hidden="true">
              <span class="c-switch__handle"></span>
            </span>
          </label>

          <!-- Site Simgeleri (Yazı solda, Switch sağda) -->
          <label class="c-search-settings-row" for="search-favicons-switch">
            <span class="c-search-settings-label">Site simgelerini göster</span>
            <input
              type="checkbox"
              id="search-favicons-switch"
              class="c-input-hidden"
              checked={showFavicons}
              onchange={(e) => updateShowFavicons(e.currentTarget.checked)}
            />
            <span class="c-switch" aria-hidden="true">
              <span class="c-switch__handle"></span>
            </span>
          </label>
        </section>

        <!-- ── 2. Genel (General) ─────────────────────────── -->
        <section class="c-search-settings-section">
          <h4>Genel</h4>

          <!-- Arama Dili -->
          <div class="c-search-settings-row">
            <span class="c-search-settings-label">Arama Dili</span>
            <div class="c-search-settings-control">
              <Dropdown
                variant="ghost"
                value={language}
                options={LANG_OPTIONS}
                onChange={updateLanguage}
              />
            </div>
          </div>

          <!-- Güvenli Arama -->
          <div class="c-search-settings-row">
            <span class="c-search-settings-label">Güvenli Arama</span>
            <div class="c-search-settings-control">
              <Dropdown
                variant="ghost"
                value={safeSearch}
                options={SAFE_OPTIONS}
                onChange={updateSafeSearch}
              />
            </div>
          </div>

          <!-- Bağlantıları Yeni Sekmede Aç (Yazı solda, Switch sağda) -->
          <label class="c-search-settings-row" for="search-new-tab-switch">
            <span class="c-search-settings-label"
              >Bağlantıları yeni sekmede aç</span
            >
            <input
              type="checkbox"
              id="search-new-tab-switch"
              class="c-input-hidden"
              checked={openInNewTab}
              onchange={(e) => updateOpenInNewTab(e.currentTarget.checked)}
            />
            <span class="c-switch" aria-hidden="true">
              <span class="c-switch__handle"></span>
            </span>
          </label>

          <!-- Sonsuz Kaydırma (Yazı solda, Switch sağda) -->
          <label class="c-search-settings-row" for="search-infinite-switch">
            <span class="c-search-settings-label">Sonsuz kaydırma</span>
            <input
              type="checkbox"
              id="search-infinite-switch"
              class="c-input-hidden"
              checked={infiniteScroll}
              onchange={(e) => updateInfiniteScroll(e.currentTarget.checked)}
            />
            <span class="c-switch" aria-hidden="true">
              <span class="c-switch__handle"></span>
            </span>
          </label>
        </section>

        <!-- ── 3. Tüm Ayarlar / Gelişmiş Bağlantı ─────────── -->
        <section class="c-search-settings-advanced-link">
          <a href="/ara/ayarlar" class="c-search-settings-all-link">
            <span>Gelişmiş Ayarlar</span>
            {@html icon("externalLink", 14)}
          </a>
        </section>
      </div>
    {/snippet}

    {#snippet footer()}
      <button type="button" class="btn btn--secondary" onclick={resetSettings}>
        Sıfırla
      </button>
      <button
        type="button"
        class="btn btn--primary"
        onclick={() => (isOpen = false)}
      >
        Kapat
      </button>
    {/snippet}
  </Modal>
{/if}
