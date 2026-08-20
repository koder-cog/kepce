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
    <div>
      <h2 id="pricing-calc-title" class="pricing-calc__title">
        Resmi Fiyat Tarifesi ve Fiş Hesaplayıcı
      </h2>
      <p class="pricing-calc__subtitle">
        {pricingData.cityName} KYK yurtları {pricingData.period} dönemi resmi tavan fiyatlarıdır.
      </p>
    </div>
    <span class="pricing-calc__badge">{pricingData.effectiveDate}</span>
  </div>

  <!-- Öğün & Kota Ayarı -->
  <div class="pricing-calc__controls">
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

  <!-- Tepsi ve Hesaplama Paneli -->
  {#if totalTrayCount > 0}
    <div class="pricing-calc__summary">
      <div class="pricing-calc__summary-header">
        <span class="pricing-calc__summary-title">Seçilen Ürünler ({totalTrayCount})</span>
        <button type="button" class="pricing-calc__btn-clear" onclick={clearTray}>
          Sıfırla
        </button>
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
            <span class="pricing-calc__result-text">0.00 TL ek ödeme</span>
          {:else}
            <span class="pricing-calc__status-tag is-warn">Limit Aşımı</span>
            <span class="pricing-calc__result-text">Fark: +{difference.toFixed(2)} TL</span>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</section>

<style>
  .pricing-calc {
    margin-top: var(--space-2xl, 2.5rem);
    padding: var(--space-lg, 1.25rem);
    background: var(--color-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-card, 16px);
    color: var(--color-text);
  }

  .pricing-calc__header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-md, 1rem);
    margin-bottom: var(--space-md, 1rem);
    flex-wrap: wrap;
  }

  .pricing-calc__title {
    font-family: var(--font-body);
    font-size: var(--text-h3, 1.125rem);
    font-weight: var(--font-weight-bold, 700);
    margin: 0 0 0.25rem 0 !important;
    color: var(--color-text-primary);
  }

  .pricing-calc__subtitle {
    font-size: var(--text-xs, 0.8125rem);
    color: var(--color-text-secondary);
    margin: 0 !important;
  }

  .pricing-calc__badge {
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border-light);
    padding: 0.25rem 0.6rem;
    border-radius: var(--radius-full, 9999px);
    font-size: var(--text-xs, 0.75rem);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .pricing-calc__controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md, 1rem);
    background: var(--color-surface-sunken);
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-md, 10px);
    margin-bottom: var(--space-md, 1rem);
    flex-wrap: wrap;
  }

  .pricing-calc__meal-toggle {
    display: flex;
    gap: 4px;
    background: var(--color-card);
    padding: 2px;
    border-radius: var(--radius-full, 9999px);
    border: 1px solid var(--color-border);
  }

  .pricing-calc__pill-btn {
    padding: 0.3rem 0.75rem;
    border-radius: var(--radius-full, 9999px);
    border: none;
    background: transparent;
    font-family: var(--font-body);
    font-size: var(--text-xs, 0.75rem);
    font-weight: var(--font-weight-medium, 600);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__pill-btn.is-active {
    background: var(--color-accent-primary);
    color: var(--color-text-on-dark, #ffffff);
  }

  .pricing-calc__quota {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .pricing-calc__quota-label {
    font-size: var(--text-xs, 0.75rem);
    color: var(--color-text-secondary);
  }

  .pricing-calc__quota-input {
    width: 64px;
    padding: 0.25rem 0.4rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm, 6px);
    font-family: var(--font-body);
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-bold, 700);
    text-align: center;
    background: var(--color-card);
    color: var(--color-text-primary);
    outline: none;
  }

  .pricing-calc__quota-input:focus {
    border-color: var(--color-accent-primary);
  }

  .pricing-calc__quota-unit {
    font-size: var(--text-xs, 0.75rem);
    color: var(--color-text-secondary);
  }

  .pricing-calc__filter {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: var(--space-md, 1rem);
  }

  .pricing-calc__search {
    position: relative;
    display: flex;
    align-items: center;
  }

  .pricing-calc__search-input {
    width: 100%;
    padding: 0.5rem 4.5rem 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md, 8px);
    font-family: var(--font-body);
    font-size: var(--text-xs, 0.8125rem);
    background: var(--color-surface-sunken);
    color: var(--color-text-primary);
    outline: none;
  }

  .pricing-calc__search-input:focus {
    border-color: var(--color-accent-primary);
  }

  .pricing-calc__search-clear {
    position: absolute;
    right: 0.6rem;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: var(--text-xs, 0.75rem);
    padding: 0.2rem;
  }

  .pricing-calc__categories {
    display: flex;
    gap: 0.35rem;
    overflow-x: auto;
    padding-bottom: 2px;
    scrollbar-width: none;
  }

  .pricing-calc__category-btn {
    padding: 0.25rem 0.65rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full, 9999px);
    background: var(--color-surface-sunken);
    font-family: var(--font-body);
    font-size: var(--text-xs, 0.75rem);
    color: var(--color-text-secondary);
    cursor: pointer;
    white-space: nowrap;
    transition: all var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__category-btn.is-active {
    background: var(--color-accent-primary);
    color: var(--color-text-on-dark, #ffffff);
    border-color: var(--color-accent-primary);
  }

  .pricing-calc__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.5rem;
    max-height: 380px;
    overflow-y: auto;
    padding-right: 2px;
    margin-bottom: var(--space-md, 1rem);
  }

  .pricing-calc__item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.55rem 0.75rem;
    background: var(--color-surface-variant);
    border: 1px solid var(--color-border-light);
    border-radius: var(--radius-sm, 8px);
    transition: border-color var(--dur-fast, 0.15s) ease;
  }

  .pricing-calc__item.is-selected {
    border-color: var(--color-accent-primary);
    background: var(--color-accent-subtle);
  }

  .pricing-calc__item-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .pricing-calc__item-name {
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-medium, 600);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pricing-calc__item-portion {
    font-size: 0.7rem;
    color: var(--color-text-secondary);
  }

  .pricing-calc__item-action {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .pricing-calc__item-price {
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-bold, 700);
    color: var(--color-accent-text);
  }

  .pricing-calc__counter {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .pricing-calc__btn-count {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm, 4px);
    border: 1px solid var(--color-border);
    background: var(--color-card);
    color: var(--color-text-primary);
    font-size: 0.875rem;
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

  .pricing-calc__count-badge {
    font-size: var(--text-xs, 0.75rem);
    font-weight: var(--font-weight-bold, 700);
    min-width: 14px;
    text-align: center;
    color: var(--color-accent-text);
  }

  .pricing-calc__empty {
    grid-column: 1 / -1;
    padding: var(--space-xl, 1.5rem);
    text-align: center;
    color: var(--color-text-secondary);
    font-size: var(--text-xs, 0.8125rem);
  }

  /* Summary Box */
  .pricing-calc__summary {
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md, 10px);
    padding: var(--space-md, 1rem);
  }

  .pricing-calc__summary-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .pricing-calc__summary-title {
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-bold, 700);
    color: var(--color-text-primary);
  }

  .pricing-calc__btn-clear {
    background: none;
    border: none;
    color: var(--color-error, #d2564a);
    font-size: var(--text-xs, 0.75rem);
    cursor: pointer;
    padding: 0;
  }

  .pricing-calc__btn-clear:hover {
    text-decoration: underline;
  }

  .pricing-calc__summary-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-bottom: 0.75rem;
  }

  .pricing-calc__summary-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    background: var(--color-card);
    border: 1px solid var(--color-border-light);
    padding: 0.2rem 0.45rem;
    border-radius: var(--radius-sm, 4px);
    font-size: var(--text-xs, 0.75rem);
    color: var(--color-text-primary);
  }

  .chip-qty {
    color: var(--color-accent-text);
  }

  .chip-price {
    color: var(--color-text-secondary);
  }

  .chip-remove {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.7rem;
    padding: 0 0 0 2px;
  }

  .chip-remove:hover {
    color: var(--color-error, #d2564a);
  }

  .pricing-calc__summary-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 0.5rem;
    border-top: 1px solid var(--color-border-light);
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .pricing-calc__totals {
    display: flex;
    gap: 1rem;
    font-size: var(--text-xs, 0.8125rem);
  }

  .pricing-calc__total-line {
    display: flex;
    gap: 0.3rem;
  }

  .pricing-calc__total-line.is-muted {
    color: var(--color-text-secondary);
  }

  .pricing-calc__result {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .pricing-calc__status-tag {
    font-size: var(--text-xs, 0.75rem);
    font-weight: var(--font-weight-bold, 700);
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-full, 9999px);
  }

  .pricing-calc__status-tag.is-ok {
    background: rgba(173, 209, 138, 0.2);
    color: var(--color-success-text);
  }

  .pricing-calc__status-tag.is-warn {
    background: rgba(236, 191, 127, 0.2);
    color: var(--color-warning-text);
  }

  .pricing-calc__result-text {
    font-size: var(--text-xs, 0.8125rem);
    font-weight: var(--font-weight-bold, 700);
    color: var(--color-text-primary);
  }
</style>
