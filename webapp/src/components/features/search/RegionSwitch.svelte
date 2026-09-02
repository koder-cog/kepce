<!--
  Kepçe Ara - Bölge & Dil Switch Bileşeni
  DuckDuckGo tarzı toggle switch ve standart Kepçe Dropdown'ı bir arada sunar.
  Apple HIG gereği metin etiketi olan alanlarda gereksiz tooltip bulunmaz, bayrak emojileri içermez.
-->
<script>
  import Dropdown from "@/components/features/Dropdown.svelte";

  let {
    value = "tr",
    onChange = () => {},
  } = $props();

  const REGION_OPTIONS = [
    { value: "tr", label: "Türkiye" },
    { value: "all", label: "Tüm diller" },
    { value: "en-US", label: "Amerika Birleşik Devletleri" },
    { value: "en-GB", label: "Birleşik Krallık" },
    { value: "de-DE", label: "Almanya" },
    { value: "fr-FR", label: "Fransa" },
    { value: "es-ES", label: "İspanya" },
    { value: "it-IT", label: "İtalya" },
    { value: "ru-RU", label: "Rusya" },
    { value: "ja-JP", label: "Japonya" },
    { value: "zh-CN", label: "Çin" },
    { value: "nl-NL", label: "Hollanda" },
    { value: "az", label: "Azerbaycan" },
    { value: "pt-BR", label: "Brezilya" },
    { value: "en-CA", label: "Kanada" },
    { value: "ko-KR", label: "Güney Kore" },
    { value: "ar-SA", label: "Suudi Arabistan" },
    { value: "el-GR", label: "Yunanistan" },
    { value: "pl-PL", label: "Polonya" },
    { value: "sv-SE", label: "İsveç" },
  ];

  let isEnabled = $derived(value !== "all" && Boolean(value));
  let lastActiveRegion = $state("tr");

  function handleToggle(e) {
    e.stopPropagation();
    if (isEnabled) {
      lastActiveRegion = value || "tr";
      onChange("all");
    } else {
      onChange(lastActiveRegion || "tr");
    }
  }

  function handleSelect(val) {
    if (val !== "all") {
      lastActiveRegion = val;
    }
    onChange(val);
  }
</script>

<div class="c-region-control">
  <label class="c-region-switch-wrap squish-effect">
    <input
      type="checkbox"
      class="c-input-hidden"
      checked={isEnabled}
      onchange={handleToggle}
      aria-label="Bölge filtresi"
    />
    <span class="c-switch c-switch--sm">
      <span class="c-switch__handle"></span>
    </span>
  </label>

  <Dropdown
    variant="ghost"
    value={value || "tr"}
    options={REGION_OPTIONS}
    onChange={handleSelect}
  />
</div>
