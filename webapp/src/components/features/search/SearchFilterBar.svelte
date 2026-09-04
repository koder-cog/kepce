<script>
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { icon } from "@/components/ui/icons.js";
  import { CATEGORIES, REGION_OPTIONS } from "$lib/search/searchHelpers.js";

  let {
    activeCategory = "general",
    data = {},
    hasActiveFilters = false,
    onCategoryChange = () => {},
    onCategoryHover = () => {},
    onFilterChange = () => {},
    onClearAllFilters = () => {},
  } = $props();
</script>

<div class="c-search-subbar-wrap">
  <!-- 1. Kategori Adası (Kepçe SegmentedControl) -->
  <div class="c-search-categories-island">
    <SegmentedControl
      options={CATEGORIES.map((c) => ({ value: c.id, label: c.label }))}
      value={activeCategory}
      onChange={(catId) => onCategoryChange(catId)}
      onHover={(catId) => onCategoryHover(catId)}
    />
  </div>

  <!-- 2. Filtreler (Kepçe Dropdown Ghost - Kategoriye Duyarlı Zengin Filtreler) -->
  <div class="c-search-pill-filters">
    <!-- Ortak Filtreler: Dil ve Güvenli Arama -->
    <Dropdown
      variant="ghost"
      value={data.language || "tr"}
      options={REGION_OPTIONS}
      onChange={(val) => onFilterChange("dil", val)}
    />

    <Dropdown
      variant="ghost"
      value={data.safeSearch || "1"}
      options={[
        { value: "1", label: "Güvenli arama: Orta" },
        { value: "2", label: "Güvenli arama: Katı" },
        { value: "0", label: "Güvenli arama: Kapalı" },
      ]}
      onChange={(val) => onFilterChange("guvenli", val)}
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
        onChange={(val) => onFilterChange("zaman", val)}
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
        onChange={(val) => onFilterChange("dosya", val)}
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
        onChange={(val) => onFilterChange("site", val)}
      />

      <button
        type="button"
        class="c-search-filter-btn"
        class:is-active={data.verbatim}
        onclick={() => onFilterChange("tam", !data.verbatim)}
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
        onChange={(val) => onFilterChange("format", val)}
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
        onChange={(val) => onFilterChange("boyut", val)}
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
        onChange={(val) => onFilterChange("renk", val)}
      />

      <Dropdown
        variant="ghost"
        value={data.imgLicense || ""}
        options={[
          { value: "", label: "Lisans: Tümü" },
          { value: "cc", label: "Lisans: Creative Commons" },
          { value: "commercial", label: "Lisans: Ticari Kullanım" },
        ]}
        onChange={(val) => onFilterChange("lisans", val)}
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
        onChange={(val) => onFilterChange("sure", val)}
      />

      <Dropdown
        variant="ghost"
        value={data.videoQuality || ""}
        options={[
          { value: "", label: "Kalite: Tüm kaliteler" },
          { value: "hd", label: "Kalite: Yüksek Kalite (HD/4K)" },
          { value: "sd", label: "Kalite: Standart (SD)" },
        ]}
        onChange={(val) => onFilterChange("kalite", val)}
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
        onChange={(val) => onFilterChange("platform", val)}
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
        onChange={(val) => onFilterChange("sirala", val)}
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
        onChange={(val) => onFilterChange("dil_prog", val)}
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
        onChange={(val) => onFilterChange("kaynak", val)}
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
        onChange={(val) => onFilterChange("erisim", val)}
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
        onChange={(val) => onFilterChange("yil", val)}
      />
    {/if}

    <!-- Filtreleri Temizle Butonu -->
    {#if hasActiveFilters}
      <button
        type="button"
        class="c-search-filter-btn c-search-filter-clear"
        onclick={onClearAllFilters}
        title="Tüm filtreleri sıfırla"
      >
        {@html icon("close", 12)}
        <span>Filtreleri Temizle</span>
      </button>
    {/if}
  </div>
</div>
