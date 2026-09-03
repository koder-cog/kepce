<script>
  import { onMount } from "svelte";
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import pricingData from "@/lib/data/pricing/istanbul_2025_2026.json";
  import { showToast } from "@/components/ui/toast.js";

  // Svelte 5 State
  let selectedMeal = $state("dinner"); // "breakfast" | "dinner"
  let allowanceInput = $state(pricingData.defaultAllowances.dinner);
  let searchQuery = $state("");
  let selectedCategory = $state("all");
  let tray = $state({}); // { [itemId]: quantity }
  let isDrawerOpen = $state(false);

  const mealOptions = [
    { value: "breakfast", label: "Kahvaltı" },
    { value: "dinner", label: "Akşam" },
  ];

  // Load from URL params if present
  onMount(() => {
    try {
      const params = new URLSearchParams(window.location.search);
      const mealParam = params.get("ogun");
      if (mealParam === "breakfast" || mealParam === "dinner") {
        selectedMeal = mealParam;
        allowanceInput = pricingData.defaultAllowances[mealParam];
      }
      const sepetParam = params.get("sepet");
      if (sepetParam) {
        const parsedTray = {};
        const pairs = sepetParam.split(",");
        for (const pair of pairs) {
          const [id, qtyStr] = pair.split(":");
          const qty = parseInt(qtyStr, 10);
          if (id && qty > 0 && pricingData.items.some((i) => i.id === id)) {
            parsedTray[id] = qty;
          }
        }
        if (Object.keys(parsedTray).length > 0) {
          tray = parsedTray;
        }
      }
    } catch {
      // Ignore URL parse errors
    }
  });

  function handleMealChange(type) {
    selectedMeal = type;
    allowanceInput = pricingData.defaultAllowances[type];
  }

  function addItem(item) {
    tray[item.id] = (tray[item.id] || 0) + 1;
  }

  function removeItem(itemId) {
    if (!tray[itemId]) return;
    if (tray[itemId] <= 1) {
      const next = { ...tray };
      delete next[itemId];
      tray = next;
    } else {
      tray[itemId] -= 1;
    }
  }

  function clearTray() {
    tray = {};
    isDrawerOpen = false;
  }
  function shuffle(array) {
    const arr = [...array];
    for (let i = arr.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [arr[i], arr[j]] = [arr[j], arr[i]];
    }
    return arr;
  }

  const CATEGORY_LABELS = {
    all: "Tümü",
    ana_yemek_etli: "Etli Yemekler",
    ana_yemek_tavuk: "Tavuk Yemekleri",
    ana_yemek_sebze: "Sebze Yemekleri",
    pilav_makarna: "Pilav & Makarna",
    corba: "Çorbalar",
    tatli: "Tatlılar",
    salata_meze: "Salata & Meze",
    kahvaltilik: "Kahvaltılık",
    pide_hamur: "Pide & Börek",
    icecek: "İçecekler",
    ekmek: "Ekmek & Hamur",
    meyve: "Meyveler",
    diger: "Diğer",
  };

  function getCategoryLabel(catKey) {
    if (!catKey) return "";
    if (CATEGORY_LABELS[catKey]) return CATEGORY_LABELS[catKey];
    return catKey.replace(/_/g, " ").replace(/\b\w/g, (l) => l.toUpperCase());
  }

  // ── Dynamic Themed Dice Presets (Randomized Knapsack Generators) ──
  function applyPreset(type) {
    const availableItems = pricingData.items.filter(
      (i) => i.mealType === "all" || i.mealType === selectedMeal,
    );
    const target = allowanceInput;

    if (type === "zero") {
      // Kesinlikle tam 0 TL fark hedefli kombinasyon uzayından rastgele seçim
      const shuffled = shuffle(availableItems);
      const exactCombos = [];

      for (let i = 0; i < shuffled.length; i++) {
        const a = shuffled[i];
        if (a.price === target) {
          exactCombos.push({ [a.id]: 1 });
        }
        for (let j = i + 1; j < shuffled.length; j++) {
          const b = shuffled[j];
          const sum2 = a.price + b.price;
          if (sum2 === target) {
            exactCombos.push({ [a.id]: 1, [b.id]: 1 });
          } else if (sum2 < target) {
            for (let k = j + 1; k < shuffled.length; k++) {
              const c = shuffled[k];
              const sum3 = sum2 + c.price;
              if (sum3 === target) {
                exactCombos.push({ [a.id]: 1, [b.id]: 1, [c.id]: 1 });
              } else if (sum3 < target) {
                for (let l = k + 1; l < shuffled.length; l++) {
                  const d = shuffled[l];
                  const sum4 = sum3 + d.price;
                  if (sum4 === target) {
                    exactCombos.push({
                      [a.id]: 1,
                      [b.id]: 1,
                      [c.id]: 1,
                      [d.id]: 1,
                    });
                  }
                }
              }
            }
          }
        }
      }

      if (exactCombos.length > 0) {
        tray = exactCombos[Math.floor(Math.random() * exactCombos.length)];
        showToast("Tam 0 TL fark ile menü oluşturuldu.", "success");
      } else {
        showToast(
          "Bu bütçeye tam 0 TL oturan bir kombinasyon bulunamadı.",
          "warning",
        );
      }
    } else if (type === "protein") {
      const proteinKeywords = [
        "tavuk",
        "köfte",
        "et",
        "yumurta",
        "yoğurt",
        "ayran",
        "süt",
        "peynir",
        "kavurma",
        "döner",
      ];
      const proteinItems = shuffle(
        availableItems.filter((i) =>
          proteinKeywords.some((k) =>
            i.name.toLocaleLowerCase("tr-TR").includes(k),
          ),
        ),
      );

      let currentTotal = 0;
      const combo = {};
      for (const item of proteinItems) {
        if (currentTotal + item.price <= target && !combo[item.id]) {
          combo[item.id] = 1;
          currentTotal += item.price;
        }
      }
      if (Object.keys(combo).length > 0) {
        tray = combo;
        showToast("Rastgele protein menüsü oluşturuldu.", "success");
      }
    } else if (type === "classic") {
      const classicKeywords = [
        "tost",
        "lahmacun",
        "pide",
        "gözleme",
        "ayran",
        "sandviç",
        "börek",
        "poğaça",
        "çay",
      ];
      const classicItems = shuffle(
        availableItems.filter((i) =>
          classicKeywords.some((k) =>
            i.name.toLocaleLowerCase("tr-TR").includes(k),
          ),
        ),
      );

      let currentTotal = 0;
      const combo = {};
      for (const item of classicItems) {
        if (currentTotal + item.price <= target && !combo[item.id]) {
          combo[item.id] = 1;
          currentTotal += item.price;
        }
      }
      if (Object.keys(combo).length > 0) {
        tray = combo;
        showToast("Rastgele büfe menüsü oluşturuldu.", "success");
      }
    } else if (type === "sweet") {
      const sweetKeywords = [
        "tatlı",
        "baklava",
        "pasta",
        "kek",
        "sütlaç",
        "çikolata",
        "kruvasan",
        "meyve",
        "kahve",
        "çay",
        "süt",
      ];
      const sweetItems = shuffle(
        availableItems.filter((i) =>
          sweetKeywords.some((k) =>
            i.name.toLocaleLowerCase("tr-TR").includes(k),
          ),
        ),
      );

      let currentTotal = 0;
      const combo = {};
      for (const item of sweetItems) {
        if (currentTotal + item.price <= target && !combo[item.id]) {
          combo[item.id] = 1;
          currentTotal += item.price;
        }
      }
      if (Object.keys(combo).length > 0) {
        tray = combo;
        showToast(
          "Rastgele tatlı & atıştırmalık menüsü oluşturuldu.",
          "success",
        );
      }
    } else if (type === "random") {
      const shuffled = shuffle(availableItems);
      let currentTotal = 0;
      const combo = {};
      for (const item of shuffled) {
        if (currentTotal + item.price <= target && !combo[item.id]) {
          combo[item.id] = 1;
          currentTotal += item.price;
        }
      }
      if (Object.keys(combo).length > 0) {
        tray = combo;
        showToast("Rastgele menü kombinasyonu oluşturuldu.", "success");
      }
    }
  }

  // ── Share Actions ──
  function getShareUrl() {
    const serialized = Object.entries(tray)
      .map(([id, qty]) => `${id}:${qty}`)
      .join(",");
    const url = new URL(window.location.href);
    url.searchParams.set("ogun", selectedMeal);
    if (serialized) {
      url.searchParams.set("sepet", serialized);
    } else {
      url.searchParams.delete("sepet");
    }
    return url.toString();
  }

  async function handleShare() {
    const url = getShareUrl();
    if (navigator.share) {
      try {
        await navigator.share({
          title: "Kepçe KYK Fiş Hesaplayıcı",
          text: `Bugünkü yurt menü seçimim (${totalTrayPrice.toFixed(0)} TL):`,
          url: url,
        });
        return;
      } catch (err) {
        if (err.name !== "AbortError") {
          console.warn("Share failed:", err);
        }
      }
    }

    navigator.clipboard.writeText(url).then(() => {
      showToast("Menü bağlantısı panoya kopyalandı.", "success");
    });
  }

  // Categories list
  let availableCategories = $derived.by(() => {
    const set = new Set(
      pricingData.items
        .filter((i) => i.mealType === "all" || i.mealType === selectedMeal)
        .map((i) => i.category)
        .filter(Boolean),
    );
    return ["all", ...Array.from(set)];
  });

  // Filtered items
  let filteredItems = $derived.by(() => {
    const q = searchQuery.trim().toLocaleLowerCase("tr-TR");
    return pricingData.items.filter((item) => {
      if (selectedCategory !== "all" && item.category !== selectedCategory) {
        return false;
      }
      if (item.mealType !== "all" && item.mealType !== selectedMeal) {
        if (!q) return false;
      }
      if (q) {
        const matchName = item.name.toLocaleLowerCase("tr-TR").includes(q);
        const matchPortion = item.portion
          .toLocaleLowerCase("tr-TR")
          .includes(q);
        return matchName || matchPortion;
      }
      return true;
    });
  });

  // Tray derived metrics
  let trayItems = $derived.by(() => {
    return Object.entries(tray)
      .map(([id, qty]) => {
        const item = pricingData.items.find((i) => i.id === id);
        return item ? { ...item, qty, itemTotal: item.price * qty } : null;
      })
      .filter(Boolean);
  });

  let totalTrayPrice = $derived(
    trayItems.reduce((acc, curr) => acc + curr.itemTotal, 0),
  );

  let difference = $derived(totalTrayPrice - allowanceInput);
  let isUnderQuota = $derived(difference <= 0);
  let totalTrayCount = $derived(
    Object.values(tray).reduce((acc, qty) => acc + qty, 0),
  );

  let progressPercent = $derived(
    allowanceInput > 0
      ? Math.min(Math.round((totalTrayPrice / allowanceInput) * 100), 100)
      : 0,
  );
</script>

<section
  class="disclaimer-card pricing-calc-card"
  aria-labelledby="pricing-calc-title"
>
  <div class="pricing-calc__header">
    <h2 id="pricing-calc-title" class="pricing-calc__title">Fiş Hesaplayıcı</h2>
    <p class="pricing-calc__subtitle">
      {pricingData.cityName} KYK yurtları {pricingData.period} dönemi resmi tavan
      fiyatlarıdır.
    </p>
  </div>

  <!-- Satır 1: SegmentedControl + Arama -->
  <div class="pricing-calc__top-bar">
    <SegmentedControl
      bind:value={selectedMeal}
      options={mealOptions}
      onChange={handleMealChange}
    />

    <div class="pricing-calc__search-wrap">
      <label
        for="pricing-search-input"
        class="u-hidden"
        style="display:none !important;">Yemek veya içecek ara</label
      >
      <input
        type="search"
        id="pricing-search-input"
        name="search"
        aria-label="Yemek veya içecek ara"
        placeholder="Yemek veya içecek ara..."
        bind:value={searchQuery}
      />
    </div>
  </div>

  <!-- Dinamik Zar Preset Butonları -->
  <div class="pricing-calc__presets-bar">
    <button
      type="button"
      class="btn btn--secondary btn--squish"
      onclick={() => applyPreset("zero")}
      title="Bütçeye tam oturan menü türet"
    >
      Sıfır Fark
    </button>
    <button
      type="button"
      class="btn btn--secondary btn--squish"
      onclick={() => applyPreset("protein")}
      title="Yüksek proteinli rastgele menü türet"
    >
      Protein
    </button>
    <button
      type="button"
      class="btn btn--secondary btn--squish"
      onclick={() => applyPreset("classic")}
      title="Büfe klasiği rastgele menü türet"
    >
      Büfe Klasiği
    </button>
    <button
      type="button"
      class="btn btn--secondary btn--squish"
      onclick={() => applyPreset("sweet")}
      title="Tatlı & atıştırmalık menüsü türet"
    >
      Tatlı
    </button>
    <button
      type="button"
      class="btn btn--secondary btn--squish"
      onclick={() => applyPreset("random")}
      title="Tamamen rastgele menü keşfet"
    >
      Rastgele
    </button>
  </div>

  <!-- Kategori Filtre Çipleri -->
  {#if availableCategories.length > 2}
    <div class="pricing-calc__chips-scroll" aria-label="Yemek kategorileri">
      {#each availableCategories as cat}
        <button
          type="button"
          class="btn btn--sm {selectedCategory === cat
            ? 'btn--primary'
            : 'btn--secondary'} btn--squish"
          onclick={() => (selectedCategory = cat)}
        >
          {getCategoryLabel(cat)}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Ürün Listesi Grid (Masaüstü: 2, Mobil: 1 Sütun) -->
  <div class="pricing-calc__grid">
    {#if filteredItems.length === 0}
      <div class="pricing-calc__empty">
        <p>Aradığınız kriterlere uygun ürün bulunamadı.</p>
      </div>
    {:else}
      {#each filteredItems as item (item.id)}
        {@const qty = tray[item.id] || 0}
        <div class="pricing-calc__item {qty > 0 ? 'is-selected' : ''}">
          <div class="pricing-calc__item-info">
            <span class="pricing-calc__item-name">{item.name}</span>
            <span class="pricing-calc__item-portion">{item.portion}</span>
          </div>
          <div class="pricing-calc__item-action">
            <span class="pricing-calc__item-price"
              >{item.price.toFixed(2)} TL</span
            >
            <div class="pricing-calc__item-counter">
              {#if qty > 0}
                <button
                  type="button"
                  class="btn btn--secondary btn--sm btn--icon-only"
                  onclick={() => removeItem(item.id)}
                  aria-label="Azalt"
                >
                  -
                </button>
                <span class="pricing-calc__item-qty">{qty}</span>
              {/if}
              <button
                type="button"
                class="btn btn--primary btn--sm btn--icon-only"
                onclick={() => addItem(item)}
                aria-label="Ekle"
              >
                +
              </button>
            </div>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  {#if totalTrayCount > 0}
    <div class="pricing-calc__sticky-wrap">
      <aside
        class="disclaimer-card pricing-calc__sticky-bar"
        aria-label="Seçim Özeti"
      >
        <!-- 3px İnce İlerleme Çizgisi -->
        <div class="pricing-calc__progress-line">
          <div
            class="pricing-calc__progress-fill {isUnderQuota
              ? 'is-ok'
              : 'is-warn'}"
            style="width: {progressPercent}%;"
          ></div>
        </div>

        <div class="pricing-calc__sticky-content">
          <div class="pricing-calc__sticky-summary">
            <span class="pricing-calc__sticky-count">{totalTrayCount} ürün</span
            >
            <span class="pricing-calc__sticky-dot">•</span>
            <span class="pricing-calc__sticky-total"
              >{totalTrayPrice.toFixed(0)} TL</span
            >
            {#if isUnderQuota}
              {#if difference === 0}
                <span class="pricing-calc__sticky-tag is-ok"
                  >Tam limittesin (0 TL fark)</span
                >
              {:else}
                <span class="pricing-calc__sticky-tag is-ok"
                  >{Math.abs(difference).toFixed(0)} TL kaldı</span
                >
              {/if}
            {:else}
              <span class="pricing-calc__sticky-tag is-warn"
                >+{difference.toFixed(0)} TL cepten</span
              >
            {/if}
          </div>

          <div class="pricing-calc__sticky-actions">
            <button
              type="button"
              class="btn btn--secondary btn--sm"
              onclick={() => (isDrawerOpen = !isDrawerOpen)}
            >
              {isDrawerOpen ? "Kapat" : "Detay"}
            </button>
            <button
              type="button"
              class="btn btn--secondary btn--sm btn--icon-only"
              onclick={handleShare}
              aria-label="Paylaş"
              title="Paylaş"
            >
              <svg
                viewBox="0 0 24 24"
                width="14"
                height="14"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <circle cx="18" cy="5" r="3"></circle>
                <circle cx="6" cy="12" r="3"></circle>
                <circle cx="18" cy="19" r="3"></circle>
                <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"></line>
                <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"></line>
              </svg>
            </button>
            <button
              type="button"
              class="btn btn--danger btn--sm btn--icon-only"
              onclick={clearTray}
              aria-label="Sıfırla"
              title="Sıfırla"
            >
              <svg
                viewBox="0 0 24 24"
                width="14"
                height="14"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <polyline points="3 6 5 6 21 6"></polyline>
                <path
                  d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                ></path>
              </svg>
            </button>
          </div>
        </div>

        <!-- Açılır/Kapanır Zarif Detay Çekmecesi (Büyütülmüş & Ferah) -->
        <div class="pricing-calc__drawer {isDrawerOpen ? 'is-open' : ''}">
          <div class="pricing-calc__drawer-inner">
            <div class="pricing-calc__drawer-list">
              {#each trayItems as item (item.id)}
                <span class="pricing-calc__drawer-chip">
                  <span>{item.name}</span>
                  <strong>x{item.qty}</strong>
                  <span class="chip-price"
                    >({item.itemTotal.toFixed(0)} TL)</span
                  >
                  <button
                    type="button"
                    onclick={() => removeItem(item.id)}
                    aria-label="Kaldır"
                  >
                    ✕
                  </button>
                </span>
              {/each}
            </div>
          </div>
        </div>
      </aside>
    </div>
  {/if}
</section>
