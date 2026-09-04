<script>
  import Modal from "@/components/ui/Modal.svelte";
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { icon } from "@/components/ui/icons.js";
  import { searchPreferences } from "@/stores/searchPreferences.svelte.js";
  import { onMount } from "svelte";

  let { isOpen = $bindable(false) } = $props();

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
    searchPreferences.init();
  });

  function resetSettings() {
    searchPreferences.setTheme("sistem");
    searchPreferences.setLanguage("tr");
    searchPreferences.setSafeSearch("1");
    searchPreferences.setOpenInNewTab(false);
    searchPreferences.setCompactResults(false);
    searchPreferences.setFaviconResolver("duckduckgo");
    searchPreferences.setInfiniteScroll(false);
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
              value={searchPreferences.theme}
              onChange={(val) => searchPreferences.setTheme(val)}
            />
          </div>

          <!-- Kompakt Sonuçlar (Yazı solda, Switch sağda) -->
          <label class="c-search-settings-row" for="search-compact-switch">
            <span class="c-search-settings-label">Kompakt sonuçlar</span>
            <input
              type="checkbox"
              id="search-compact-switch"
              class="c-input-hidden"
              checked={searchPreferences.compactResults}
              onchange={(e) => searchPreferences.setCompactResults(e.currentTarget.checked)}
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
              checked={searchPreferences.faviconResolver !== "none" && searchPreferences.faviconResolver !== "off"}
              onchange={(e) => searchPreferences.setFaviconResolver(e.currentTarget.checked ? "duckduckgo" : "off")}
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
                value={searchPreferences.language}
                options={LANG_OPTIONS}
                onChange={(val) => searchPreferences.setLanguage(val)}
              />
            </div>
          </div>

          <!-- Güvenli Arama -->
          <div class="c-search-settings-row">
            <span class="c-search-settings-label">Güvenli Arama</span>
            <div class="c-search-settings-control">
              <Dropdown
                variant="ghost"
                value={searchPreferences.safeSearch}
                options={SAFE_OPTIONS}
                onChange={(val) => searchPreferences.setSafeSearch(val)}
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
              checked={searchPreferences.openInNewTab}
              onchange={(e) => searchPreferences.setOpenInNewTab(e.currentTarget.checked)}
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
              checked={searchPreferences.infiniteScroll}
              onchange={(e) => searchPreferences.setInfiniteScroll(e.currentTarget.checked)}
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
