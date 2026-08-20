<script>
  import pricingData from "@/lib/data/pricing/istanbul_2025_2026.json";

  // Svelte 5 State
  let selectedMeal = $state("dinner"); // "breakfast" | "dinner"
  let allowanceInput = $state(pricingData.defaultAllowances.dinner);
  let searchQuery = $state("");
  let selectedCategory = $state("all");
  let tray = $state({}); // { [itemId]: quantity }

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

  // Filtered items
  let filteredItems = $derived.by(() => {
    const q = searchQuery.trim().toLocaleLowerCase("tr-TR");
    return pricingData.items.filter((item) => {
      // Category check
      if (selectedCategory !== "all" && item.category !== selectedCategory) {
        return false;
      }
      // Meal type relevance (breakfast vs dinner vs all)
      if (item.mealType !== "all" && item.mealType !== selectedMeal) {
        // If user is searching specifically, allow finding across meals, but otherwise prioritize selected meal
        if (!q) return false;
      }
      // Search query check
      if (q) {
        const matchName = item.name.toLocaleLowerCase("tr-TR").includes(q);
        const matchPortion = item.portion.toLocaleLowerCase("tr-TR").includes(q);
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
    trayItems.reduce((acc, curr) => acc + curr.itemTotal, 0)
  );

  let difference = $derived(totalTrayPrice - allowanceInput);
  let isUnderQuota = $derived(difference <= 0);
  let totalTrayCount = $derived(
    Object.values(tray).reduce((acc, qty) => acc + qty, 0)
  );
</script>

<section class="pricing-calc" aria-labelledby="pricing-calc-title">
  <div class="pricing-calc__header">
    <div class="pricing-calc__title-group">
      <h2 id="pricing-calc-title" class="pricing-calc__title">
        Resmi Alakart Fiyat Tarifesi & Fiş Hesaplayıcı
      </h2>
      <p class="pricing-calc__subtitle">
        {pricingData.cityName} KYK yurtları ({pricingData.period}) resmi tavan fiyatları. Ürünleri seçerek kasada ödeyeceğiniz ek farkı hesaplayabilirsiniz.
      </p>
    </div>
    <div class="pricing-calc__badge">
      <span>{pricingData.effectiveDate} Tarifesi</span>
    </div>
  </div>

  <!-- Öğün & Kota Ayarı -->
  <div class="pricing-calc__controls-box">
    <div class="pricing-calc__meal-toggle">
      <span class="pricing-calc__control-label">Öğün Seçimi:</span>
      <div class="pricing-calc__pill-group" role="tablist">
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
          Akşam Yemeği (105 TL)
        </button>
      </div>
    </div>

    <div class="pricing-calc__allowance-input">
      <label for="allowance-input" class="pricing-calc__control-label">
        Fiş Kotanız (TL):
      </label>
      <input
        id="allowance-input"
        type="number"
        min="0"
        max="500"
        step="1"
        bind:value={allowanceInput}
        class="pricing-calc__input"
      />
    </div>
  </div>

  <!-- Arama ve Kategori Filtresi -->
  <div class="pricing-calc__filter-bar">
    <div class="pricing-calc__search">
      <svg class="pricing-calc__search-icon" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input
        type="text"
        placeholder="Yemek veya içecek ara (Örn: Tost, Pide, Ayran)..."
        bind:value={searchQuery}
        class="pricing-calc__search-input"
      />
      {#if searchQuery}
        <button
          type="button"
          class="pricing-calc__search-clear"
          onclick={() => (searchQuery = "")}
          aria-label="Aramayı temizle"
        >
          ✕
        </button>
      {/if}
    </div>

    <div class="pricing-calc__categories" role="tablist">
      {#each pricingData.categories as cat}
        <button
          type="button"
          role="tab"
          aria-selected={selectedCategory === cat.id}
          class="pricing-calc__category-btn {selectedCategory === cat.id ? 'is-active' : ''}"
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
            <div class="pricing-calc__item-name">{item.name}</div>
            <div class="pricing-calc__item-portion">{item.portion}</div>
          </div>
          <div class="pricing-calc__item-action">
            <span class="pricing-calc__item-price">{item.price.toFixed(2)} TL</span>
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
                aria-label="Tepsiye ekle"
              >
                +
              </button>
            </div>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Canlı Tepsi ve Hesaplama Paneli -->
  {#if totalTrayCount > 0}
    <div class="pricing-calc__summary">
      <div class="pricing-calc__summary-header">
        <div class="pricing-calc__summary-title">
          <span>🍽️ Seçilenler ({totalTrayCount} Ürün)</span>
          <button type="button" class="pricing-calc__btn-clear" onclick={clearTray}>
            Tepsiyi Sıfırla
          </button>
        </div>
        <div class="pricing-calc__summary-chips">
          {#each trayItems as item (item.id)}
            <span class="pricing-calc__summary-chip">
              {item.name} <strong class="chip-qty">x{item.qty}</strong> ({item.itemTotal.toFixed(2)} TL)
              <button
                type="button"
                class="chip-remove"
                onclick={() => removeItem(item.id)}
                aria-label="Kaldır"
              >
                ✕
              </button>
            </span>
          {/each}
        </div>
      </div>

      <div class="pricing-calc__summary-footer">
        <div class="pricing-calc__metric">
          <span class="metric-label">Toplam Tutar:</span>
          <strong class="metric-val">{totalTrayPrice.toFixed(2)} TL</strong>
        </div>

        <div class="pricing-calc__metric">
          <span class="metric-label">Fiş Kotası:</span>
          <span class="metric-val">{allowanceInput.toFixed(2)} TL</span>
        </div>

        <div class="pricing-calc__metric-status {isUnderQuota ? 'is-success' : 'is-warning'}">
          {#if isUnderQuota}
            <span class="status-badge is-green">✓ Fiş Limitiniz Karşılıyor</span>
            <span class="status-sub">0.00 TL Ek Ödeme (Kalan: {Math.abs(difference).toFixed(2)} TL)</span>
          {:else}
            <span class="status-badge is-orange">⚠️ Limit Aşımı</span>
            <span class="status-sub">Kasada Ödenecek Fark: <strong>+{difference.toFixed(2)} TL</strong></span>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</section>

<style>
  .pricing-calc {
    margin-top: 2.5rem;
    padding: 1.5rem;
    background: var(--color-bg-surface, #ffffff);
    border: 1px solid var(--color-border, #e5e7eb);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  }

  :global(.dark) .pricing-calc {
    background: var(--color-bg-surface, #1e1e20);
    border-color: var(--color-border, #2e2e32);
  }

  .pricing-calc__header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
  }

  .pricing-calc__title {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0 0 0.35rem 0;
    color: var(--color-text-primary, #111827);
  }

  :global(.dark) .pricing-calc__title {
    color: var(--color-text-primary, #f3f4f6);
  }

  .pricing-calc__subtitle {
    font-size: 0.875rem;
    color: var(--color-text-secondary, #6b7280);
    margin: 0;
    line-height: 1.4;
  }

  .pricing-calc__badge {
    background: var(--color-bg-secondary, #f3f4f6);
    border: 1px solid var(--color-border, #e5e7eb);
    padding: 0.35rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary, #4b5563);
    white-space: nowrap;
  }

  :global(.dark) .pricing-calc__badge {
    background: var(--color-bg-secondary, #28282c);
    border-color: var(--color-border, #3a3a40);
    color: #9ca3af;
  }

  .pricing-calc__controls-box {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    background: var(--color-bg-secondary, #f9fafb);
    padding: 1rem;
    border-radius: var(--radius-md, 8px);
    margin-bottom: 1.25rem;
    flex-wrap: wrap;
  }

  :global(.dark) .pricing-calc__controls-box {
    background: var(--color-bg-secondary, #18181a);
  }

  .pricing-calc__meal-toggle,
  .pricing-calc__allowance-input {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .pricing-calc__control-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary, #374151);
  }

  :global(.dark) .pricing-calc__control-label {
    color: var(--color-text-primary, #d1d5db);
  }

  .pricing-calc__pill-group {
    display: flex;
    background: var(--color-bg-surface, #ffffff);
    padding: 3px;
    border-radius: 8px;
    border: 1px solid var(--color-border, #d1d5db);
  }

  :global(.dark) .pricing-calc__pill-group {
    background: #242428;
    border-color: #38383e;
  }

  .pricing-calc__pill-btn {
    padding: 0.35rem 0.85rem;
    border-radius: 6px;
    border: none;
    background: transparent;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-secondary, #6b7280);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .pricing-calc__pill-btn.is-active {
    background: var(--color-primary, #2563eb);
    color: #ffffff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .pricing-calc__input {
    width: 80px;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--color-border, #d1d5db);
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 600;
    text-align: center;
    background: var(--color-bg-surface, #ffffff);
    color: var(--color-text-primary, #111827);
  }

  :global(.dark) .pricing-calc__input {
    background: #242428;
    border-color: #38383e;
    color: #f3f4f6;
  }

  .pricing-calc__filter-bar {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1.25rem;
  }

  .pricing-calc__search {
    position: relative;
    display: flex;
    align-items: center;
  }

  .pricing-calc__search-icon {
    position: absolute;
    left: 0.85rem;
    color: var(--color-text-secondary, #9ca3af);
    pointer-events: none;
  }

  .pricing-calc__search-input {
    width: 100%;
    padding: 0.65rem 2.2rem 0.65rem 2.5rem;
    border: 1px solid var(--color-border, #d1d5db);
    border-radius: var(--radius-md, 8px);
    font-size: 0.875rem;
    background: var(--color-bg-surface, #ffffff);
    color: var(--color-text-primary, #111827);
    outline: none;
    transition: border-color 0.15s ease;
  }

  .pricing-calc__search-input:focus {
    border-color: var(--color-primary, #2563eb);
  }

  :global(.dark) .pricing-calc__search-input {
    background: #18181a;
    border-color: #2e2e32;
    color: #f3f4f6;
  }

  .pricing-calc__search-clear {
    position: absolute;
    right: 0.75rem;
    background: none;
    border: none;
    color: #9ca3af;
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.2rem;
  }

  .pricing-calc__categories {
    display: flex;
    gap: 0.4rem;
    overflow-x: auto;
    padding-bottom: 4px;
    scrollbar-width: thin;
  }

  .pricing-calc__category-btn {
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-border, #e5e7eb);
    border-radius: 9999px;
    background: var(--color-bg-surface, #ffffff);
    font-size: 0.8125rem;
    color: var(--color-text-secondary, #4b5563);
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .pricing-calc__category-btn:hover {
    border-color: #d1d5db;
    background: #f9fafb;
  }

  .pricing-calc__category-btn.is-active {
    background: #374151;
    color: #ffffff;
    border-color: #374151;
  }

  :global(.dark) .pricing-calc__category-btn {
    background: #1e1e20;
    border-color: #2e2e32;
    color: #9ca3af;
  }

  :global(.dark) .pricing-calc__category-btn.is-active {
    background: #4b5563;
    color: #ffffff;
    border-color: #4b5563;
  }

  .pricing-calc__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 0.75rem;
    max-height: 480px;
    overflow-y: auto;
    padding-right: 4px;
    margin-bottom: 1.5rem;
  }

  .pricing-calc__item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 0.9rem;
    background: var(--color-bg-secondary, #f9fafb);
    border: 1px solid var(--color-border, #e5e7eb);
    border-radius: 8px;
    transition: border-color 0.15s ease;
  }

  .pricing-calc__item.is-selected {
    border-color: var(--color-primary, #2563eb);
    background: #eff6ff;
  }

  :global(.dark) .pricing-calc__item {
    background: #18181a;
    border-color: #28282c;
  }

  :global(.dark) .pricing-calc__item.is-selected {
    background: #1e293b;
    border-color: #3b82f6;
  }

  .pricing-calc__item-name {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary, #1f2937);
  }

  :global(.dark) .pricing-calc__item-name {
    color: var(--color-text-primary, #e5e7eb);
  }

  .pricing-calc__item-portion {
    font-size: 0.75rem;
    color: var(--color-text-secondary, #6b7280);
    margin-top: 2px;
  }

  .pricing-calc__item-action {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .pricing-calc__item-price {
    font-size: 0.875rem;
    font-weight: 700;
    color: var(--color-text-primary, #111827);
  }

  :global(.dark) .pricing-calc__item-price {
    color: #f9fafb;
  }

  .pricing-calc__counter {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .pricing-calc__btn-count {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: 1px solid var(--color-border, #d1d5db);
    background: var(--color-bg-surface, #ffffff);
    color: var(--color-text-primary, #374151);
    font-size: 1rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .pricing-calc__btn-count.is-add {
    background: var(--color-primary, #2563eb);
    color: #ffffff;
    border-color: var(--color-primary, #2563eb);
  }

  :global(.dark) .pricing-calc__btn-count {
    background: #28282c;
    border-color: #38383e;
    color: #d1d5db;
  }

  :global(.dark) .pricing-calc__btn-count.is-add {
    background: #2563eb;
    color: #ffffff;
    border-color: #2563eb;
  }

  .pricing-calc__count-badge {
    font-size: 0.8125rem;
    font-weight: 700;
    min-width: 16px;
    text-align: center;
  }

  .pricing-calc__empty {
    grid-column: 1 / -1;
    padding: 2rem;
    text-align: center;
    color: var(--color-text-secondary, #6b7280);
    font-size: 0.875rem;
  }

  /* Summary Drawer / Box */
  .pricing-calc__summary {
    background: var(--color-bg-surface, #ffffff);
    border: 2px solid var(--color-border, #d1d5db);
    border-radius: var(--radius-lg, 12px);
    padding: 1.25rem;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  }

  :global(.dark) .pricing-calc__summary {
    background: #18181a;
    border-color: #3a3a40;
  }

  .pricing-calc__summary-header {
    margin-bottom: 1rem;
    border-bottom: 1px dashed var(--color-border, #e5e7eb);
    padding-bottom: 0.75rem;
  }

  :global(.dark) .pricing-calc__summary-header {
    border-color: #2e2e32;
  }

  .pricing-calc__summary-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.9375rem;
    font-weight: 700;
    margin-bottom: 0.65rem;
  }

  .pricing-calc__btn-clear {
    background: none;
    border: none;
    color: #ef4444;
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
  }

  .pricing-calc__summary-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .pricing-calc__summary-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background: var(--color-bg-secondary, #f3f4f6);
    padding: 0.25rem 0.6rem;
    border-radius: 6px;
    font-size: 0.8125rem;
    color: var(--color-text-primary, #374151);
  }

  :global(.dark) .pricing-calc__summary-chip {
    background: #242428;
    color: #e5e7eb;
  }

  .chip-qty {
    color: var(--color-primary, #2563eb);
  }

  .chip-remove {
    background: none;
    border: none;
    color: #9ca3af;
    cursor: pointer;
    font-size: 0.75rem;
    padding: 0;
    margin-left: 2px;
  }

  .chip-remove:hover {
    color: #ef4444;
  }

  .pricing-calc__summary-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .pricing-calc__metric {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .metric-label {
    font-size: 0.75rem;
    color: var(--color-text-secondary, #6b7280);
    font-weight: 500;
  }

  .metric-val {
    font-size: 1.125rem;
    font-weight: 700;
  }

  .pricing-calc__metric-status {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .status-badge {
    font-size: 0.8125rem;
    font-weight: 700;
    padding: 0.25rem 0.65rem;
    border-radius: 9999px;
  }

  .status-badge.is-green {
    background: #dcfce7;
    color: #166534;
  }

  :global(.dark) .status-badge.is-green {
    background: #14532d;
    color: #86efac;
  }

  .status-badge.is-orange {
    background: #ffedd5;
    color: #9a3412;
  }

  :global(.dark) .status-badge.is-orange {
    background: #7c2d12;
    color: #fdba74;
  }

  .status-sub {
    font-size: 0.8125rem;
    color: var(--color-text-secondary, #4b5563);
  }

  :global(.dark) .status-sub {
    color: #9ca3af;
  }

  @media (max-width: 640px) {
    .pricing-calc {
      padding: 1rem;
    }
    .pricing-calc__controls-box,
    .pricing-calc__summary-footer {
      flex-direction: column;
      align-items: flex-start;
    }
    .pricing-calc__metric-status {
      align-items: flex-start;
      margin-top: 0.5rem;
      width: 100%;
    }
  }
</style>
