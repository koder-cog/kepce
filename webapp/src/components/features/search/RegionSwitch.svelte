<!--
  Kepçe Ara - DuckDuckGo Tarzı Bölge & Dil Switch Bileşeni
  Toggle switch ile anında açılıp kapatılabilir, tıklandığında aranabilir ülke listesi açar.
-->
<script>
  import { icon } from "@/components/ui/icons.js";

  let {
    value = "tr",
    onChange = () => {},
  } = $props();

  const POPULAR_REGIONS = [
    { code: "tr", name: "Türkiye", flag: "🇹🇷" },
    { code: "all", name: "Tüm Diller / Küresel", flag: "🌐" },
    { code: "en-US", name: "Amerika Birleşik Devletleri", flag: "🇺🇸" },
    { code: "en-GB", name: "Birleşik Krallık", flag: "🇬🇧" },
    { code: "de-DE", name: "Almanya", flag: "🇩🇪" },
    { code: "fr-FR", name: "Fransa", flag: "🇫🇷" },
    { code: "es-ES", name: "İspanya", flag: "🇪🇸" },
    { code: "it-IT", name: "İtalya", flag: "🇮🇹" },
    { code: "ru-RU", name: "Rusya", flag: "🇷🇺" },
    { code: "ja-JP", name: "Japonya", flag: "🇯🇵" },
    { code: "zh-CN", name: "Çin", flag: "🇨🇳" },
    { code: "nl-NL", name: "Hollanda", flag: "🇳🇱" },
    { code: "az", name: "Azerbaycan", flag: "🇦🇿" },
    { code: "pt-BR", name: "Brezilya", flag: "🇧🇷" },
    { code: "en-CA", name: "Kanada", flag: "🇨🇦" },
    { code: "ko-KR", name: "Güney Kore", flag: "🇰🇷" },
    { code: "ar-SA", name: "Suudi Arabistan", flag: "🇸🇦" },
    { code: "el-GR", name: "Yunanistan", flag: "🇬🇷" },
    { code: "pl-PL", name: "Polonya", flag: "🇵🇱" },
    { code: "sv-SE", name: "İsveç", flag: "🇸🇪" },
  ];

  let isOpen = $state(false);
  let filterQuery = $state("");
  let lastActiveRegion = $state("tr");
  let dropdownEl = $state(null);

  // Switch durumu: "all" veya boş değilse bölge filtresi etkindir
  let isEnabled = $derived(value !== "all" && Boolean(value));

  let currentRegionInfo = $derived.by(() => {
    if (!isEnabled) {
      return { code: "all", name: "Tüm Diller", flag: "🌐" };
    }
    const found = POPULAR_REGIONS.find((r) => r.code === value);
    if (found) return found;
    return { code: value, name: value.toUpperCase(), flag: "📍" };
  });

  let filteredRegions = $derived.by(() => {
    const q = filterQuery.trim().toLowerCase();
    if (!q) return POPULAR_REGIONS;
    return POPULAR_REGIONS.filter(
      (r) =>
        r.name.toLowerCase().includes(q) ||
        r.code.toLowerCase().includes(q)
    );
  });

  function handleToggle(e) {
    e.stopPropagation();
    if (isEnabled) {
      // Kapatıldı -> Küresel
      lastActiveRegion = value || "tr";
      onChange("all");
    } else {
      // Açıldı -> Son aktif bölgeye dön
      onChange(lastActiveRegion || "tr");
    }
  }

  function handleSelect(code) {
    if (code !== "all") {
      lastActiveRegion = code;
    }
    isOpen = false;
    filterQuery = "";
    onChange(code);
  }

  function toggleDropdown(e) {
    e.stopPropagation();
    isOpen = !isOpen;
    if (isOpen) {
      filterQuery = "";
    }
  }

  function handleOutsideClick(e) {
    if (dropdownEl && !dropdownEl.contains(e.target)) {
      isOpen = false;
    }
  }
</script>

<svelte:window onclick={handleOutsideClick} />

<div class="c-region-control" bind:this={dropdownEl}>
  <!-- DDG Switch Düğmesi -->
  <button
    type="button"
    class="c-region-switch-btn"
    class:is-active={isEnabled}
    onclick={handleToggle}
    aria-label={isEnabled ? "Bölge filtresini kapat (Tüm diller)" : "Bölge filtresini aç"}
    title={isEnabled ? "Bölge filtresini kapat (Tüm diller)" : "Bölge filtresini aç"}
  >
    <span class="c-region-switch-handle"></span>
  </button>

  <!-- Bölge Adı ve Açılır Menü Butonu -->
  <button
    type="button"
    class="c-region-label-btn"
    class:is-dimmed={!isEnabled}
    onclick={toggleDropdown}
    aria-expanded={isOpen}
    aria-haspopup="listbox"
  >
    <span class="c-region-flag">{currentRegionInfo.flag}</span>
    <span class="c-region-name">{currentRegionInfo.name}</span>
    <span class="c-region-arrow" class:is-open={isOpen}>
      {@html icon("chevronDown", 14)}
    </span>
  </button>

  <!-- Açılır Bölge Seçim Paneli -->
  {#if isOpen}
    <div class="c-region-dropdown" role="listbox" tabindex="-1">
      <div class="c-region-search-wrap">
        <input
          type="text"
          class="c-region-search-input"
          placeholder="Ülke veya bölge ara..."
          bind:value={filterQuery}
          onclick={(e) => e.stopPropagation()}
        />
      </div>

      <ul class="c-region-list">
        {#each filteredRegions as region (region.code)}
          <li>
            <button
              type="button"
              class="c-region-item-btn"
              class:is-selected={region.code === (isEnabled ? value : "all")}
              onclick={() => handleSelect(region.code)}
              role="option"
              aria-selected={region.code === (isEnabled ? value : "all")}
            >
              <span class="c-region-item-flag">{region.flag}</span>
              <span class="c-region-item-name">{region.name}</span>
              {#if region.code === (isEnabled ? value : "all")}
                <span class="c-region-item-check">
                  {@html icon("check", 14)}
                </span>
              {/if}
            </button>
          </li>
        {/each}
        {#if filteredRegions.length === 0}
          <li class="c-region-empty">Eşleşen bölge bulunamadı</li>
        {/if}
      </ul>
    </div>
  {/if}
</div>
