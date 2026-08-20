<script>
  import { onMount } from "svelte";
  import pricingData from "@/lib/data/pricing/istanbul_2025_2026.json";
  import { showToast } from "@/components/ui/toast.js";

  // Svelte 5 State
  let selectedMeal = $state("dinner"); // "breakfast" | "dinner"
  let allowanceInput = $state(pricingData.defaultAllowances.dinner);
  let searchQuery = $state("");
  let selectedCategory = $state("all");
  let tray = $state({}); // { [itemId]: quantity }
  let isDrawerOpen = $state(false);

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

  // ── Smart Presets ──
  function applyPreset(type) {
    if (type === "zero") {
      if (selectedMeal === "breakfast") {
        tray = { "kasarli-tost": 1, ayran: 1 };
      } else {
        tray = { "tavuk-doner": 1, ayran: 1, tursu: 1, su: 1 };
      }
      showToast("Sıfır fark menüsü seçildi.", "success");
    } else if (type === "protein") {
      if (selectedMeal === "breakfast") {
        tray = { "sahanda-cift-yumurta": 1, sut: 1 };
      } else {
        tray = { "kemiksiz-tavuk-yemegi": 1, yogurt: 1, ayran: 1 };
      }
      showToast("Protein menüsü seçildi.", "success");
    } else if (type === "classic") {
      if (selectedMeal === "breakfast") {
        tray = { gozleme: 1 };
      } else {
        tray = { lahmacun: 1, "baklava-cevizli": 1, ayran: 1 };
      }
      showToast("Büfe klasiği menü seçildi.", "success");
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
    const itemsText = trayItems
      .map((i) => `- ${i.name} (${i.qty} adet)`)
      .join("\n");
    const statusText = isUnderQuota
      ? `Tam limittesin (0 TL fark)`
      : `+${difference.toFixed(0)} TL cepten`;
    const shareText = `Kepçe KYK Alakart Menüm:\n${itemsText}\nToplam: ${totalTrayPrice.toFixed(0)} TL (${statusText})\n${url}`;

    if (navigator.share) {
      try {
        await navigator.share({
          title: "Kepçe KYK Alakart Menüm",
          text: shareText,
          url,
        });
        return;
      } catch {
        // Fallback to clipboard
      }
    }

    navigator.clipboard.writeText(url).then(() => {
      showToast("Menü bağlantısı panoya kopyalandı.", "success");
    });
  }

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

<section class="pricing-calc" aria-labelledby="pricing-calc-title">
  <div class="pricing-calc__header">
    <h2 id="pricing-calc-title" class="pricing-calc__title">
      Resmi Fiyat Tarifesi ve Fiş Hesaplayıcı
    </h2>
    <p class="pricing-calc__subtitle">
      {pricingData.cityName} KYK yurtları {pricingData.period} dönemi resmi tavan fiyatlarıdır.
    </p>
  </div>

  <!-- Satır 1: Öğün Geçişi ve Arama Çubuğu -->
  <div class="pricing-calc__top-bar">
    <div class="pricing-calc__meal-toggle" role="tablist">
      <button
        type="button"
        role="tab"
        aria-selected={selectedMeal === "breakfast"}
        class="pricing-calc__pill-btn {selectedMeal === 'breakfast' ? 'is-active' : ''}"
        onclick={() => handleMealChange("breakfast")}
      >
        Kahvaltı (45 TL)
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={selectedMeal === "dinner"}
        class="pricing-calc__pill-btn {selectedMeal === 'dinner' ? 'is-active' : ''}"
        onclick={() => handleMealChange("dinner")}
      >
        Akşam (105 TL)
      </button>
    </div>

    <div class="pricing-calc__search">
      <input
        type="text"
        placeholder="Yemek veya içecek ara..."
        bind:value={searchQuery}
        class="pricing-calc__search-input"
      />
      {#if searchQuery}
        <button
          type="button"
          class="pricing-calc__search-clear"
          onclick={() => (searchQuery = "")}
        >
          Temizle
        </button>
      {/if}
    </div>
  </div>

  <!-- Satır 2: Akıllı Kombinasyonlar ve Kategoriler -->
  <div class="pricing-calc__pills-row" role="tablist">
    <button
      type="button"
      class="pricing-calc__chip is-preset"
      onclick={() => applyPreset("zero")}
    >
      Sıfır Fark
    </button>
    <button
      type="button"
      class="pricing-calc__chip is-preset"
      onclick={() => applyPreset("protein")}
    >
      Protein
    </button>
    <button
      type="button"
      class="pricing-calc__chip is-preset"
      onclick={() => applyPreset("classic")}
    >
      Büfe Klasiği
    </button>
    {#each pricingData.categories as cat}
      <button
        type="button"
        role="tab"
        aria-selected={selectedCategory === cat.id}
        class="pricing-calc__chip {selectedCategory === cat.id ? 'is-active' : ''}"
        onclick={() => (selectedCategory = cat.id)}
      >
        {cat.name}
      </button>
    {/each}
  </div>

  <!-- Ürün Listesi Grid -->
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
            <span class="pricing-calc__item-price">{item.price.toFixed(2)} TL</span>
            <div class="pricing-calc__counter">
              {#if qty > 0}
                <button
                  type="button"
                  class="pricing-calc__btn-count"
                  onclick={() => removeItem(item.id)}
                  aria-label="Azalt"
                >
                  -
                </button>
                <span class="pricing-calc__count-badge">{qty}</span>
              {/if}
              <button
                type="button"
                class="pricing-calc__btn-count is-add"
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

  <!-- Yapışkan Alt Bar (Tek ve Net Doğruluk Kaynağı) -->
  {#if totalTrayCount > 0}
    <div class="pricing-calc__sticky-wrap">
      <aside class="pricing-calc__sticky-bar" aria-label="Seçim Özeti">
        <!-- 3px İnce İlerleme Çizgisi -->
        <div class="pricing-calc__progress-line">
          <div
            class="pricing-calc__progress-fill {isUnderQuota ? 'is-ok' : 'is-warn'}"
            style="width: {progressPercent}%;"
          ></div>
        </div>

        <div class="pricing-calc__sticky-content">
          <div class="pricing-calc__sticky-summary">
            <span class="pricing-calc__sticky-count">{totalTrayCount} ürün</span>
            <span class="pricing-calc__sticky-dot">•</span>
            <span class="pricing-calc__sticky-total">{totalTrayPrice.toFixed(0)} TL</span>
            {#if isUnderQuota}
              {#if difference === 0}
                <span class="pricing-calc__sticky-tag is-ok">Tam limittesin (0 TL fark)</span>
              {:else}
                <span class="pricing-calc__sticky-tag is-ok">{Math.abs(difference).toFixed(0)} TL kaldı</span>
              {/if}
            {:else}
              <span class="pricing-calc__sticky-tag is-warn">+{difference.toFixed(0)} TL cepten</span>
            {/if}
          </div>

          <div class="pricing-calc__sticky-actions">
            <button
              type="button"
              class="pricing-calc__btn-text-action"
              onclick={() => (isDrawerOpen = !isDrawerOpen)}
            >
              {isDrawerOpen ? 'Kapat' : 'Detay'}
            </button>
            <button
              type="button"
              class="pricing-calc__btn-icon"
              onclick={handleShare}
              aria-label="Paylaş"
              title="Paylaş"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="18" cy="5" r="3"></circle>
                <circle cx="6" cy="12" r="3"></circle>
                <circle cx="18" cy="19" r="3"></circle>
                <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"></line>
                <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"></line>
              </svg>
            </button>
            <button
              type="button"
              class="pricing-calc__btn-icon is-danger"
              onclick={clearTray}
              aria-label="Sıfırla"
              title="Sıfırla"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"></polyline>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
              </svg>
            </button>
          </div>
        </div>

        <!-- Açılır/Kapanır Zarif Detay Çekmecesi -->
        {#if isDrawerOpen}
          <div class="pricing-calc__drawer">
            <div class="pricing-calc__drawer-list">
              {#each trayItems as item (item.id)}
                <span class="pricing-calc__drawer-chip">
                  <span>{item.name}</span>
                  <strong>x{item.qty}</strong>
                  <span>({item.itemTotal.toFixed(0)} TL)</span>
                  <button
                    type="button"
                    onclick={() => removeItem(item.id)}
                    aria-label="Kaldır"
                  >
                    Sil
                  </button>
                </span>
              {/each}
            </div>
          </div>
        {/if}
      </aside>
    </div>
  {/if}
</section>
