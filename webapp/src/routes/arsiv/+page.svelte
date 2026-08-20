<script>
  import "@/styles/pages/_archive.css";
  import { api } from "@/api/index.js";
  import {
    getCurrentCity,
    setCurrentCity,
    getCitiesData,
  } from "@/stores/city.svelte.js";
  import { getMonthName } from "@/utils/date.js";
  import ArchiveRow from "@/components/features/ArchiveRow.svelte";
  import CitySelector from "@/components/features/CitySelector.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import Loader from "@/components/ui/Loader.svelte";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import { onMount } from "svelte";

  const WEEKDAYS = [
    "Pazar",
    "Pazartesi",
    "Salı",
    "Çarşamba",
    "Perşembe",
    "Cuma",
    "Cumartesi",
  ];
  const MONTHS = [
    "Ocak",
    "Şubat",
    "Mart",
    "Nisan",
    "Mayıs",
    "Haziran",
    "Temmuz",
    "Ağustos",
    "Eylül",
    "Ekim",
    "Kasım",
    "Aralık",
  ];

  let cities = $state([]);
  let selectedCity = $state(getCurrentCity());
  let selectedYear = $state("");
  let selectedMonth = $state("");
  let lastLoadedKey = $state("");
  let yearOptions = $state([]);

  let groupedMenus = $state(null);
  let errorMsg = $state(null);
  let errorCode = $state(null);
  let currentLoadToken = 0;

  let isLoading = $state(false);

  function formatArchiveDate(dateStr) {
    const d = new Date(dateStr);
    if (isNaN(d.getTime())) return dateStr;
    return `${d.getDate()} ${MONTHS[d.getMonth()]} ${d.getFullYear()}, ${WEEKDAYS[d.getDay()]}`;
  }

  let isSearchValid = $derived(
    !!(selectedCity && selectedYear && selectedMonth),
  );
  let currentKey = $derived(`${selectedCity}-${selectedYear}-${selectedMonth}`);
  let isButtonRefresh = $derived(isSearchValid && currentKey === lastLoadedKey);

  onMount(async () => {
    try {
      cities = await getCitiesData();
    } catch (err) {
      console.error("Failed to fetch cities:", err);
    }
  });

  $effect(() => {
    if (selectedCity) {
      const token = ++currentLoadToken;
      api
        .getArchiveYears(selectedCity)
        .then((years) => {
          if (token !== currentLoadToken) return;
          yearOptions = years.map((y) => ({
            value: String(y),
            label: String(y),
          }));
          // If selectedYear is not in yearOptions, clear it
          if (
            selectedYear &&
            !yearOptions.some((opt) => opt.value === selectedYear)
          ) {
            selectedYear = "";
            selectedMonth = "";
          }
        })
        .catch((err) => {
          if (token !== currentLoadToken) return;
          console.error("Failed to load archive years:", err);
          yearOptions = [];
        });
    } else {
      yearOptions = [];
    }
  });

  // ── Sonsuz spinner koruması ────────────────────────────────
  // Önceki kodda `finally { if (token === currentLoadToken) isLoading = false }`
  // kullanılıyordu. Eğer bir istek hiç çözülmezse (örn. kullanıcı aynı tuşa iki kez
  // basıp, eski istek takılırsa) veya bir başka istek token'ı geçersiz kılarsa,
  // `isLoading` sonsuza kadar `true` kalıyordu. Şimdi:
  //  - Spinner'ı GÖSTERMEK yalnızca en güncel istek için gecikmeli olarak tetiklenir
  //    (eski istekler ekrana geç ulaşırsa spinner patlamaz).
  //  - Spinner'ı GİZLEMEK her zaman tetiklenir; eğer arada yeni istek başladıysa
  //    onun setTimeout'ı tekrar `true` yapar, böylece spinner takılı kalmaz.
  //  - İstek çözülür çözülmez `AbortController` ile yenisi başladığında eski istek
  //    iptal edilir; cevap gelirse göz ardı edilir (yarış durumu önlenir).
  let abortController = null;

  async function handleLoad() {
    if (!isSearchValid) return;

    // Önceki isteği iptal et (yarış koşulu)
    if (abortController) abortController.abort();
    abortController = new AbortController();

    errorMsg = null;
    errorCode = null;
    groupedMenus = null;

    const token = ++currentLoadToken;
    const showLoadingTimeout = setTimeout(() => {
      if (currentLoadToken === token && !abortController.signal.aborted) {
        isLoading = true;
      }
    }, 150);

    try {
      const menus = await api.getMonthlyMenus(
        selectedCity,
        selectedYear,
        selectedMonth,
      );
      // Bu noktada ya abort edilmiş olabilir ya da daha yeni bir istek başlamış olabilir;
      // her iki durumda da state'i bozmadan erken çık.
      if (abortController.signal.aborted || currentLoadToken !== token) {
        return;
      }
      lastLoadedKey = currentKey;

      if (menus.length === 0) {
        groupedMenus = {};
      } else {
        const byDate = {};
        menus.forEach((m) => {
          if (!byDate[m.serve_date]) byDate[m.serve_date] = [];
          byDate[m.serve_date].push(m);
        });
        groupedMenus = byDate;
      }
    } catch (err) {
      if (abortController.signal.aborted || err?.name === "AbortError") return;
      errorMsg = err.message || "Bilinmeyen hata";
      errorCode = parseInt(err.message.match(/\d{3}/)?.[0]) || 500;
    } finally {
      clearTimeout(showLoadingTimeout);
      // Spinner gizlemeyi her zaman çalıştır; eğer arada yeni istek başladıysa
      // onun `setTimeout`'ı tekrar `true` yapar — UI tutarlı kalır.
      if (currentLoadToken === token) {
        isLoading = false;
      }
    }
  }

  let monthOptions = $derived.by(() => {
    if (!selectedYear) return [];

    const now = new Date();
    const isCurrentYear = parseInt(selectedYear) === now.getFullYear();
    const currentM = now.getMonth() + 1;
    const maxMonth = isCurrentYear ? currentM : 12;

    const opts = [];
    for (let m = 1; m <= maxMonth; m++) {
      opts.push({ value: String(m), label: getMonthName(m) });
    }
    return opts;
  });

  $effect(() => {
    if (selectedYear && selectedMonth) {
      const now = new Date();
      const isCurrentYear = parseInt(selectedYear) === now.getFullYear();
      const maxMonth = isCurrentYear ? now.getMonth() + 1 : 12;
      if (parseInt(selectedMonth) > maxMonth) {
        selectedMonth = "";
      }
    }
  });
</script>

<Seo
  title="Geçmiş Menü Arşivi - Kepçe"
  description="Geçmiş aylara ve yıllara ait KYK yurt yemek menüleri arşivi. Şehir ve tarih bazlı yemek listesi geçmişi."
/>

<div class="archive-header">
  <h1 class="archive-title">Arşiv</h1>
</div>

<div class="archive-controls" id="archive-controls">
  <!--
    Arşiv sayfası için şehir seçimi KASITLI olarak global currentCity'den
    bağımsızdır (localOnly) ve özel "Şehrinizi göremiyor musunuz?" CTA'sını
    göstermez (showSpecial=false). Diğer sayfalardaki dropdown davranışını
    etkilemez; her iki dropdown da kendi `bind:value` state'ine yazıp bağımsız
    yaşar.
  -->
  <CitySelector
    bind:value={selectedCity}
    {cities}
    variant="primary"
    showSpecial={false}
    localOnly={true}
  />

  <Dropdown options={yearOptions} bind:value={selectedYear} placeholder="Yıl" />

  <Dropdown
    options={monthOptions}
    bind:value={selectedMonth}
    placeholder="Ay"
    disabled={!selectedYear}
  />
  <button
    class="archive-controls__btn"
    disabled={!isSearchValid}
    onclick={handleLoad}
  >
    {isButtonRefresh ? "Yenile" : "Göster"}
  </button>
</div>

<div id="archive-results" class:u-opacity-dim={isLoading}>
  {#if isLoading}
    <div class="empty-state-container u-fade-in">
      <Loader size="m3-loader--md" />
    </div>
  {:else if errorMsg}
    <div class="empty-state-container u-fade-in">
      <EmptyState statusCode={errorCode} desc={errorMsg} />
    </div>
  {:else if groupedMenus}
    {#if Object.keys(groupedMenus).length === 0}
      <div class="empty-state-container u-fade-in">
        <EmptyState
          statusCode={404}
          title={"Yok böyle bişii."}
          desc={"Seçtiğiniz tarih aralığı için arşivde herhangi bir menü kaydı bulunamadı."}
        />
      </div>
    {:else}
      <div class="u-fade-in">
        {#each Object.entries(groupedMenus).sort() as [date, dayMenus]}
          {#if dayMenus.length > 0}
            <h3 class="archive-date-title">{formatArchiveDate(date)}</h3>
            <div class="meal-cards--list">
              {#each dayMenus as m}
                <ArchiveRow menu={m} />
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    {/if}
  {/if}
</div>

<Seo
  title="Menü Arşivi - Kepçe"
  description="Geçmiş KYK yurt yemekhanesi menüleri, besin değerleri ve öğrenci değerlendirme arşivi."
  image="https://kepce.org/api/v1/public/og/page/arsiv"
/>
