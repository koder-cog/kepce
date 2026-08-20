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
        {pricingData.cityName} KYK yurtları ({pricingData.period}) resmi tavan fiyatlarıdır. Ürünleri seçerek kasada ödeyeceğiniz ek farkı hesaplayabilirsiniz.
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
                  class="pricing-calc__btn-count is-sub"
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
    margin-top: var(--space-2xl, 2.5rem);
    padding: var(--space-xl, 1.5rem);
    background: var(--color-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-card, 16px);
    box-shadow: var(--shadow-sm, 0 1px 3px rgba(0, 0, 0, 0.05));
    color: var(--color-text);
  }

  .pricing-calc__header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-md, 1rem);
    margin-bottom: var(--space-lg, 1.25rem);
    flex-wrap: wrap;
  }

  .pricing-calc__title {
    font-family: var(--font-body);
    font-size: var(--text-h3, 1.25rem);
    font-weight: var(--font-weight-bold, 700);
    margin: 0 0 0.35rem 0 !important;
    color: var(--color-text-primary);
  }

  .pricing-calc__subtitle {
    font-size: var(--text-sm, 0.875rem);
    color: var(--color-text-secondary);
    margin: 0 !important;
    line-height: var(--leading-normal, 1.4);
  }

  .pricing-calc__badge {
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border-light);
    padding: 0.35rem 0.75rem;
    border-radius: var(--radius-full, 9999px);
    font-size: var(--text-xs, 0.75rem);
    font-weight: var(--font-weight-medium, 600);
    color: var(--color-accent-text);
    white-space: nowrap;
  }

  .pricing-calc__controls-box {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md, 1rem);
    background: var(--color-surface-sunken);
    padding: var(--space-md, 1rem);
    border-radius: var(--radius-md, 10px);
    border: 1px solid var(--color-border-light);
    margin-bottom: var(--space-lg, 1.25rem);
    flex-wrap: wrap;
  }

  .pricing-calc__meal-toggle,
  .pricing-calc__allowance-input {
    display: flex;
    align-items: center;
    gap: var(--space-sm, 0.75rem);
  }

  .pricing-calc__control-label {
    font-size: var(--text-sm, 0.875rem);
    font-weight: var(--font-weight-medium, 600);
    color: var(--color-text-primary);
  }

  .pricing-calc__pill-group {
    display: flex;
    background: var(--color-card);
    padding: 3px;
    border-radius: var(--radius-full, 9999px);
    border: 1px solid var(--color-border);
  }

  .pricing-calc__pill-btn {
    padding: 0.35rem 0.85rem;
    border-radius: var(--radius-full, 9999px);
    border: none;
    background: transparent;
    font-family: var(--font-body);
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-medium, 600);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__pill-btn.is-active {
    background: var(--color-accent-primary);
    color: var(--color-text-on-dark, #ffffff);
    box-shadow: var(--shadow-sm, 0 1px 2px rgba(0, 0, 0, 0.15));
  }

  .pricing-calc__input {
    width: 84px;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm, 6px);
    font-family: var(--font-body);
    font-size: var(--text-sm, 0.875rem);
    font-weight: var(--font-weight-bold, 700);
    text-align: center;
    background: var(--color-card);
    color: var(--color-text-primary);
    outline: none;
    transition: border-color var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__input:focus {
    border-color: var(--color-accent-primary);
  }

  .pricing-calc__filter-bar {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm, 0.75rem);
    margin-bottom: var(--space-lg, 1.25rem);
  }

  .pricing-calc__search {
    position: relative;
    display: flex;
    align-items: center;
  }

  .pricing-calc__search-icon {
    position: absolute;
    left: 0.85rem;
    color: var(--color-text-secondary);
    pointer-events: none;
  }

  .pricing-calc__search-input {
    width: 100%;
    padding: 0.65rem 2.2rem 0.65rem 2.5rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md, 10px);
    font-family: var(--font-body);
    font-size: var(--text-sm, 0.875rem);
    background: var(--color-surface-sunken);
    color: var(--color-text-primary);
    outline: none;
    transition: border-color var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__search-input:focus {
    border-color: var(--color-accent-primary);
  }

  .pricing-calc__search-clear {
    position: absolute;
    right: 0.75rem;
    background: none;
    border: none;
    color: var(--color-text-secondary);
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
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full, 9999px);
    background: var(--color-surface-sunken);
    font-family: var(--font-body);
    font-size: var(--text-xs, 0.8125rem);
    color: var(--color-text-secondary);
    cursor: pointer;
    white-space: nowrap;
    transition: all var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__category-btn:hover {
    border-color: var(--color-border-strong);
    color: var(--color-text-primary);
  }

  .pricing-calc__category-btn.is-active {
    background: var(--color-accent-primary);
    color: var(--color-text-on-dark, #ffffff);
    border-color: var(--color-accent-primary);
  }

  .pricing-calc__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 0.75rem;
    max-height: 480px;
    overflow-y: auto;
    padding-right: 4px;
    margin-bottom: var(--space-lg, 1.25rem);
  }

  .pricing-calc__item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 0.9rem;
    background: var(--color-surface-variant);
    border: 1px solid var(--color-border-light);
    border-radius: var(--radius-md, 10px);
    transition: border-color var(--dur-fast, 0.15s) ease, background var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__item.is-selected {
    border-color: var(--color-accent-primary);
    background: var(--color-accent-subtle);
  }

  .pricing-calc__item-name {
    font-size: var(--text-sm, 0.875rem);
    font-weight: var(--font-weight-medium, 600);
    color: var(--color-text-primary);
  }

  .pricing-calc__item-portion {
    font-size: var(--text-xs, 0.75rem);
    color: var(--color-text-secondary);
    margin-top: 2px;
  }

  .pricing-calc__item-action {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .pricing-calc__item-price {
    font-size: var(--text-sm, 0.875rem);
    font-weight: var(--font-weight-bold, 700);
    color: var(--color-accent-text);
  }

  .pricing-calc__counter {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .pricing-calc__btn-count {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm, 6px);
    border: 1px solid var(--color-border);
    background: var(--color-card);
    color: var(--color-text-primary);
    font-size: 1rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__btn-count.is-add {
    background: var(--color-accent-primary);
    color: var(--color-text-on-dark, #ffffff);
    border-color: var(--color-accent-primary);
  }

  .pricing-calc__btn-count.is-add:hover {
    background: var(--color-accent-primary-hover);
  }

  .pricing-calc__btn-count.is-sub:hover {
    border-color: var(--color-border-strong);
    color: var(--color-error);
  }

  .pricing-calc__count-badge {
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-bold, 700);
    min-width: 16px;
    text-align: center;
    color: var(--color-accent-text);
  }

  .pricing-calc__empty {
    grid-column: 1 / -1;
    padding: var(--space-2xl, 2rem);
    text-align: center;
    color: var(--color-text-secondary);
    font-size: var(--text-sm, 0.875rem);
  }

  /* Summary Drawer / Box */
  .pricing-calc__summary {
    background: var(--color-surface-elevated);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-card, 14px);
    padding: var(--space-lg, 1.25rem);
    box-shadow: var(--shadow-md, 0 4px 6px -1px rgba(0, 0, 0, 0.1));
  }

  .pricing-calc__summary-header {
    margin-bottom: var(--space-md, 1rem);
    border-bottom: 1px dashed var(--color-border);
    padding-bottom: var(--space-sm, 0.75rem);
  }

  .pricing-calc__summary-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--text-sm, 0.9375rem);
    font-weight: var(--font-weight-bold, 700);
    color: var(--color-text-primary);
    margin-bottom: var(--space-sm, 0.65rem);
  }

  .pricing-calc__btn-clear {
    background: none;
    border: none;
    color: var(--color-error, #d2564a);
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-medium, 600);
    cursor: pointer;
    padding: 0;
  }

  .pricing-calc__btn-clear:hover {
    text-decoration: underline;
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
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border-light);
    padding: 0.25rem 0.6rem;
    border-radius: var(--radius-sm, 6px);
    font-size: var(--text-xs, 0.8125rem);
    color: var(--color-text-primary);
  }

  .chip-qty {
    color: var(--color-accent-text);
  }

  .chip-remove {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.75rem;
    padding: 0;
    margin-left: 2px;
  }

  .chip-remove:hover {
    color: var(--color-error, #d2564a);
  }

  .pricing-calc__summary-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md, 1rem);
    flex-wrap: wrap;
  }

  .pricing-calc__metric {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .metric-label {
    font-size: var(--text-xs, 0.75rem);
    color: var(--color-text-secondary);
    font-weight: var(--font-weight-medium, 500);
  }

  .metric-val {
    font-size: var(--text-lg, 1.125rem);
    font-weight: var(--font-weight-bold, 700);
    color: var(--color-text-primary);
  }

  .pricing-calc__metric-status {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .status-badge {
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-bold, 700);
    padding: 0.25rem 0.65rem;
    border-radius: var(--radius-full, 9999px);
  }

  .status-badge.is-green {
    background: rgba(173, 209, 138, 0.15);
    color: var(--color-success-text);
    border: 1px solid var(--color-success);
  }

  .status-badge.is-orange {
    background: rgba(236, 191, 127, 0.15);
    color: var(--color-warning-text);
    border: 1px solid var(--color-warning);
  }

  .status-sub {
    font-size: var(--text-xs, 0.8125rem);
    color: var(--color-text-secondary);
  }

  .status-sub strong {
    color: var(--color-accent-text);
  }

  @media (max-width: 640px) {
    .pricing-calc {
      padding: var(--space-md, 1rem);
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
