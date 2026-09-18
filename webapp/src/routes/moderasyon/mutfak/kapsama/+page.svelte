<script>
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { icon } from "@/components/ui/icons.js";
  import { showToast } from "@/components/ui/toast.js";

  const PILOT_CITIES = new Set([
    "istanbul", "ankara", "izmir", "konya", "eskisehir", "bursa", "antalya"
  ]);

  const ACTIVE_CITIES = new Set([
    "istanbul", "ankara", "izmir", "antalya", "canakkale", "erzurum",
    "eskisehir", "gaziantep", "isparta", "kahramanmaras", "karabuk",
    "kirklareli", "konya", "sakarya", "sivas", "trabzon"
  ]);

  const MONTH_NAMES = [
    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
    "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"
  ];

  const DAY_NAMES = ["Paz", "Pzt", "Sal", "Çar", "Per", "Cum", "Cmt"];

  let today = new Date();
  let selectedYear = $state(today.getFullYear());
  let selectedMonth = $state(today.getMonth() + 1);

  let currentYear = today.getFullYear();
  let currentMonth = today.getMonth() + 1;
  let currentDay = today.getDate();

  let activeFilter = $state("all"); // 'all' | 'active' | 'pilot'
  let searchQuery = $state("");

  let isLoading = $state(true);
  let errorMsg = $state(null);
  let coverageData = $state(null);

  async function loadCoverage() {
    isLoading = true;
    errorMsg = null;
    try {
      const res = await api.getKitchenCoverage(selectedYear, selectedMonth);
      coverageData = res;
    } catch (err) {
      errorMsg = err.message || "Kapsama verisi yüklenemedi.";
      showToast(errorMsg, "error");
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    loadCoverage();
  });

  function changeMonth(delta) {
    let newMonth = selectedMonth + delta;
    let newYear = selectedYear;
    if (newMonth < 1) {
      newMonth = 12;
      newYear -= 1;
    } else if (newMonth > 12) {
      newMonth = 1;
      newYear += 1;
    }
    selectedMonth = newMonth;
    selectedYear = newYear;
    loadCoverage();
  }

  function goToCurrentMonth() {
    selectedYear = currentYear;
    selectedMonth = currentMonth;
    loadCoverage();
  }

  let filteredCities = $derived.by(() => {
    if (!coverageData || !coverageData.cities) return [];
    let list = coverageData.cities;

    if (activeFilter === "pilot") {
      list = list.filter((c) => PILOT_CITIES.has(c.slug));
    } else if (activeFilter === "active") {
      list = list.filter((c) => ACTIVE_CITIES.has(c.slug));
    }

    if (searchQuery.trim()) {
      const q = searchQuery.trim().toLocaleLowerCase("tr-TR");
      list = list.filter(
        (c) =>
          c.name.toLocaleLowerCase("tr-TR").includes(q) ||
          c.slug.includes(q) ||
          String(c.id).includes(q)
      );
    }

    return list;
  });

  function getDayName(year, month, day) {
    const d = new Date(year, month - 1, day);
    return DAY_NAMES[d.getDay()];
  }

  function pad(num) {
    return num < 10 ? `0${num}` : String(num);
  }

  function getCellInfo(city, day) {
    if (!city.days || !city.days[day]) {
      return {
        status: "empty",
        label: "·",
        title: `${city.name} - ${day} ${MONTH_NAMES[selectedMonth - 1]}: Menü bulunamadı`,
      };
    }

    const dayData = city.days[day];
    const hasBreakfast = !!dayData.breakfast;
    const hasDinner = !!dayData.dinner;

    if (hasBreakfast && hasDinner) {
      const bApp = dayData.breakfast.is_approved ? "Onaylı" : "Bekliyor";
      const dApp = dayData.dinner.is_approved ? "Onaylı" : "Bekliyor";
      return {
        status: "full",
        label: "✓",
        title: `${city.name} - ${day} ${MONTH_NAMES[selectedMonth - 1]}: Kahvaltı (${bApp}), Akşam (${dApp})`,
      };
    } else if (hasBreakfast || hasDinner) {
      const meal = hasBreakfast ? "Kahvaltı" : "Akşam";
      return {
        status: "partial",
        label: "1",
        title: `${city.name} - ${day} ${MONTH_NAMES[selectedMonth - 1]}: Yalnızca ${meal} menüsü mevcut`,
      };
    }

    return {
      status: "empty",
      label: "·",
      title: `${city.name} - ${day} ${MONTH_NAMES[selectedMonth - 1]}: Menü bulunamadı`,
    };
  }

  function handleCellClick(city, day) {
    const dateStr = `${selectedYear}-${pad(selectedMonth)}-${pad(day)}`;
    goto(`/moderasyon/mutfak/tabela?sehir=${city.slug}&gun=${dateStr}`);
  }
</script>

<svelte:head>
  <title>Kapsama Matrisi - Moderasyon - Kepçe</title>
</svelte:head>

<div class="admin-header u-mb-lg">
  <div>
    <h2 class="admin-title">Kapsama Matrisi</h2>
    <p class="admin-subtitle">
      Türkiye genelindeki 81 ilin günlük menü doluluğu ve onay durumu
    </p>
  </div>

  <div class="u-flex u-flex-align-center u-gap-sm">
    <button
      class="btn btn--secondary btn--squish"
      onclick={() => changeMonth(-1)}
      title="Önceki Ay"
      aria-label="Önceki Ay"
    >
      {@html icon("chevronLeft", 16)}
    </button>
    <strong class="u-text-sm u-px-sm">
      {selectedYear} {MONTH_NAMES[selectedMonth - 1]}
    </strong>
    <button
      class="btn btn--secondary btn--squish"
      onclick={() => changeMonth(1)}
      title="Sonraki Ay"
      aria-label="Sonraki Ay"
    >
      {@html icon("chevronRight", 16)}
    </button>
    {#if selectedYear !== currentYear || selectedMonth !== currentMonth}
      <button
        class="btn btn--secondary btn--squish u-ml-xs"
        onclick={goToCurrentMonth}
      >
        Bu Ay
      </button>
    {/if}
  </div>
</div>

<div class="coverage-controls">
  <div class="coverage-filters">
    <button
      class="btn btn--squish"
      class:btn--primary={activeFilter === "all"}
      class:btn--secondary={activeFilter !== "all"}
      onclick={() => (activeFilter = "all")}
    >
      Tüm İller (81)
    </button>
    <button
      class="btn btn--squish"
      class:btn--primary={activeFilter === "active"}
      class:btn--secondary={activeFilter !== "active"}
      onclick={() => (activeFilter = "active")}
    >
      Aktif İller ({ACTIVE_CITIES.size})
    </button>
    <button
      class="btn btn--squish"
      class:btn--primary={activeFilter === "pilot"}
      class:btn--secondary={activeFilter !== "pilot"}
      onclick={() => (activeFilter = "pilot")}
    >
      Pilot İller ({PILOT_CITIES.size})
    </button>

    <div class="u-ml-sm">
      <input
        type="text"
        class="input input--sm"
        placeholder="İl ara..."
        bind:value={searchQuery}
      />
    </div>
  </div>

  <div class="coverage-legend">
    <div class="coverage-legend__item">
      <span class="coverage-legend__dot coverage-legend__dot--full"></span>
      <span>Tam (Kahvaltı & Akşam)</span>
    </div>
    <div class="coverage-legend__item">
      <span class="coverage-legend__dot coverage-legend__dot--partial"></span>
      <span>Kısmi (Tek Öğün)</span>
    </div>
    <div class="coverage-legend__item">
      <span class="coverage-legend__dot coverage-legend__dot--empty"></span>
      <span>Boş</span>
    </div>
  </div>
</div>

{#if isLoading}
  <div class="stats-placeholder">
    <Loader size={48} />
  </div>
{:else if errorMsg}
  <EmptyState statusCode={500} desc={errorMsg} />
{:else if !coverageData || filteredCities.length === 0}
  <EmptyState
    iconName="search"
    title="Şehir Bulunamadı"
    desc="Arama kriterlerine uyan şehir kaydı bulunamadı."
  />
{:else}
  <div
    class="coverage-matrix-container"
    role="region"
    aria-label="Aylık Menü Kapsama Matrisi"
  >
    <table class="coverage-table">
      <caption class="sr-only">
        {selectedYear} {MONTH_NAMES[selectedMonth - 1]} Ayı Menü Kapsama Tablosu
      </caption>
      <thead>
        <tr>
          <th scope="col">Şehir ({filteredCities.length})</th>
          {#each Array(coverageData.days_in_month) as _, i}
            {@const day = i + 1}
            {@const isToday =
              selectedYear === currentYear &&
              selectedMonth === currentMonth &&
              day === currentDay}
            <th scope="col" class:coverage-cell--today={isToday}>
              <div>{day}</div>
              <div class="u-color-muted coverage-subtext">
                {getDayName(selectedYear, selectedMonth, day)}
              </div>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each filteredCities as city (city.id)}
          <tr>
            <th scope="row">
              <span class="u-color-muted u-mr-xs">#{city.id}</span>
              <a
                href="/{city.slug}"
                target="_blank"
                class="u-link u-color-text"
                title="{city.name} sayfasını yeni sekmede aç"
              >
                {city.name}
              </a>
            </th>
            {#each Array(coverageData.days_in_month) as _, i}
              {@const day = i + 1}
              {@const cell = getCellInfo(city, day)}
              {@const isToday =
                selectedYear === currentYear &&
                selectedMonth === currentMonth &&
                day === currentDay}
              <td>
                <button
                  type="button"
                  class="coverage-cell coverage-cell--{cell.status}"
                  class:coverage-cell--today={isToday}
                  title={cell.title}
                  onclick={() => handleCellClick(city, day)}
                >
                  {cell.label}
                </button>
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
