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

  // Change meal type and reset default allowance
  function handleMealChange(type) {
    selectedMeal = type;
    allowanceInput = pricingData.defaultAllowances[type];
  }

  // Tray actions
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
  }

  // ── Knapsack / Smart Combination Generators ──
  function applyZeroDiffCombo() {
    if (selectedMeal === "breakfast") {
      // 45 TL Breakfast: Kaşarlı Tost (36 TL) + Ayran (8 TL) = 44 TL
      tray = {
        "kasarli-tost": 1,
        ayran: 1,
      };
    } else {
      // 105 TL Dinner: Tavuk Döner (80 TL) + Ayran (8 TL) + Turşu (12 TL) + Su (5 TL) = 105.00 TL (Exact 0 TL diff!)
      tray = {
        "tavuk-doner": 1,
        ayran: 1,
        tursu: 1,
        su: 1,
      };
    }
    showToast("Sıfır fark kombinasyonu tepsiye eklendi.", "success");
  }

  function applyProteinCombo() {
    if (selectedMeal === "breakfast") {
      // Breakfast protein: Dana Sucuklu Çift Yumurta (50 TL) or Sahanda Çift Yumurta (27 TL) + Süt (15 TL) = 42 TL
      tray = {
        "sahanda-cift-yumurta": 1,
        sut: 1,
      };
    } else {
      // Dinner protein: Kemiksiz Tavuk (80 TL) + Yoğurt (16 TL) + Ayran (8 TL) = 104 TL
      tray = {
        "kemiksiz-tavuk-yemegi": 1,
        yogurt: 1,
        ayran: 1,
      };
    }
    showToast("Protein odaklı menü tepsiye eklendi.", "success");
  }

  function applyClassicCombo() {
    if (selectedMeal === "breakfast") {
      // Breakfast classic: Gözleme (47 TL)
      tray = {
        gozleme: 1,
      };
    } else {
      // Dinner classic: Lahmacun (55 TL) + Cevizli Baklava (39 TL) + Ayran (8 TL) = 102 TL
      tray = {
        lahmacun: 1,
        "baklava-cevizli": 1,
        ayran: 1,
      };
    }
    showToast("Büfe klasiği menü tepsiye eklendi.", "success");
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

  function copyShareLink() {
    const url = getShareUrl();
    navigator.clipboard.writeText(url).then(() => {
      showToast("Menü bağlantısı panoya kopyalandı.", "success");
    });
  }

  function shareWhatsApp() {
    const itemsText = trayItems
      .map((i) => `- ${i.name} (${i.qty} adet, ${i.itemTotal.toFixed(2)} TL)`)
      .join("\n");
    const statusText = isUnderQuota
      ? `Limit içi (0 TL ek ödeme, Kalan: ${Math.abs(difference).toFixed(2)} TL)`
      : `Kasada ödenecek fark: +${difference.toFixed(2)} TL`;
    const message = `Kepçe Alakart Menü Setup'ım:\n${itemsText}\nToplam: ${totalTrayPrice.toFixed(2)} TL / Fiş Kotası: ${allowanceInput.toFixed(2)} TL\n${statusText}\n\n${getShareUrl()}`;
    const waUrl = `https://api.whatsapp.com/send?text=${encodeURIComponent(message)}`;
    window.open(waUrl, "_blank");
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
    <div>
      <h2 id="pricing-calc-title" class="pricing-calc__title">
        Resmi Fiyat Tarifesi ve Fiş Hesaplayıcı
      </h2>
      <p class="pricing-calc__subtitle">
        {pricingData.cityName} KYK yurtları {pricingData.period} dönemi resmi tavan
        fiyatlarıdır.
      </p>
    </div>
  </div>

  <!-- Öğün & Kota Ayarı -->
  <div class="pricing-calc__controls">
    <div class="pricing-calc__meal-toggle" role="tablist">
      <button
        type="button"
        role="tab"
        aria-selected={selectedMeal === "breakfast"}
        class="pricing-calc__pill-btn {selectedMeal === 'breakfast'
          ? 'is-active'
          : ''}"
        onclick={() => handleMealChange("breakfast")}
      >
        Kahvaltı (45 TL)
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={selectedMeal === "dinner"}
        class="pricing-calc__pill-btn {selectedMeal === 'dinner'
          ? 'is-active'
          : ''}"
        onclick={() => handleMealChange("dinner")}
      >
        Akşam (105 TL)
      </button>
    </div>

    <div class="pricing-calc__quota">
      <label for="allowance-input" class="pricing-calc__quota-label">
        Kota
      </label>
      <input
        id="allowance-input"
        type="number"
        min="0"
        max="500"
        step="1"
        bind:value={allowanceInput}
        class="pricing-calc__quota-input"
      />
      <span class="pricing-calc__quota-unit">TL</span>
    </div>
  </div>

  <!-- Hızlı Kombinasyon / Knapsack Butonları -->
  <div class="pricing-calc__presets">
    <span class="pricing-calc__presets-label">Hazır Menüler:</span>
    <button
      type="button"
      class="pricing-calc__preset-btn"
      onclick={applyZeroDiffCombo}
    >
      Sıfır Farkla Doldur
    </button>
    <button
      type="button"
      class="pricing-calc__preset-btn"
      onclick={applyProteinCombo}
    >
      Maksimum Protein
    </button>
    <button
      type="button"
      class="pricing-calc__preset-btn"
      onclick={applyClassicCombo}
    >
      Büfe Klasiği
    </button>
  </div>

  <!-- Arama ve Kategori Filtresi -->
  <div class="pricing-calc__filter">
    <div class="pricing-calc__search">
      <input
        type="text"
        placeholder="Yemek veya içecek ara (tost, pide, ayran...)"
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

    <div class="pricing-calc__categories" role="tablist">
      {#each pricingData.categories as cat}
        <button
          type="button"
          role="tab"
          aria-selected={selectedCategory === cat.id}
          class="pricing-calc__category-btn {selectedCategory === cat.id
            ? 'is-active'
            : ''}"
          onclick={() => (selectedCategory = cat.id)}
        >
          {cat.name}
        </button>
      {/each}
    </div>
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
            <span class="pricing-calc__item-price"
              >{item.price.toFixed(2)} TL</span
            >
            <div class="pricing-calc__counter">
              {#if qty > 0}
                <button
                  type="button"
                  class="pricing-calc__btn-count"
                  onclick={() => removeItem(item.id)}
                  aria-label="Bir azalt"
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

  <!-- Sayfa İçi Tepsi ve Hesaplama Paneli -->
  {#if totalTrayCount > 0}
    <div class="pricing-calc__summary">
      <div class="pricing-calc__summary-header">
        <span class="pricing-calc__summary-title"
          >Seçilen Ürünler ({totalTrayCount})</span
        >
        <div class="pricing-calc__summary-actions">
          <button
            type="button"
            class="pricing-calc__btn-share"
            onclick={shareWhatsApp}
          >
            WhatsApp
          </button>
          <button
            type="button"
            class="pricing-calc__btn-share"
            onclick={copyShareLink}
          >
            Linki Kopyala
          </button>
          <button
            type="button"
            class="pricing-calc__btn-clear"
            onclick={clearTray}
          >
            Sıfırla
          </button>
        </div>
      </div>

      <div class="pricing-calc__summary-chips">
        {#each trayItems as item (item.id)}
          <span class="pricing-calc__summary-chip">
            <span>{item.name}</span>
            <span class="chip-qty">{item.qty} adet</span>
            <span class="chip-price">{item.itemTotal.toFixed(2)} TL</span>
            <button
              type="button"
              class="chip-remove"
              onclick={() => removeItem(item.id)}
            >
              Sil
            </button>
          </span>
        {/each}
      </div>

      <!-- Kota Progress Bar -->
      <div class="pricing-calc__progress-bar">
        <div
          class="pricing-calc__progress-fill {isUnderQuota
            ? 'is-ok'
            : 'is-warn'}"
          style="width: {progressPercent}%;"
        ></div>
      </div>

      <div class="pricing-calc__summary-footer">
        <div class="pricing-calc__totals">
          <div class="pricing-calc__total-line">
            <span>Tutar:</span>
            <strong>{totalTrayPrice.toFixed(2)} TL</strong>
          </div>
          <div class="pricing-calc__total-line is-muted">
            <span>Kota:</span>
            <span>{allowanceInput.toFixed(2)} TL</span>
          </div>
        </div>

        <div class="pricing-calc__result">
          {#if isUnderQuota}
            <span class="pricing-calc__status-tag is-ok">Limit İçi</span>
            <span class="pricing-calc__result-text"
              >0.00 TL ek ödeme ({Math.abs(difference).toFixed(2)} TL kaldı)</span
            >
          {:else}
            <span class="pricing-calc__status-tag is-warn">Limit Aşımı</span>
            <span class="pricing-calc__result-text"
              >Fark: +{difference.toFixed(2)} TL</span
            >
          {/if}
        </div>
      </div>
    </div>

    <!-- Sticky Bottom Bar (Ekran Kaydırıldığında Kasada Sürpriz Rezillik Önleyici) -->
    <aside class="pricing-calc__sticky-bar" aria-label="Seçim Özeti">
      <div class="pricing-calc__sticky-info">
        <span class="pricing-calc__sticky-count">{totalTrayCount} Ürün</span>
        <span class="pricing-calc__sticky-total"
          >{totalTrayPrice.toFixed(2)} TL</span
        >
        {#if isUnderQuota}
          <span class="pricing-calc__status-tag is-ok">0 TL Fark</span>
        {:else}
          <span class="pricing-calc__status-tag is-warn"
            >+{difference.toFixed(2)} TL</span
          >
        {/if}
      </div>
      <div class="pricing-calc__sticky-actions">
        <button
          type="button"
          class="pricing-calc__btn-pill-action"
          onclick={shareWhatsApp}
        >
          WhatsApp
        </button>
        <button
          type="button"
          class="pricing-calc__btn-pill-action"
          onclick={clearTray}
        >
          Sıfırla
        </button>
      </div>
    </aside>
  {/if}
</section>
