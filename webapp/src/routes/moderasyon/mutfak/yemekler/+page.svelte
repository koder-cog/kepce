<script>
  import { onMount } from "svelte";
  import { api } from '@/api/index.js';
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Loader from "@/components/ui/Loader.svelte";
  import { icon } from "@/components/ui/icons.js";
  import * as ui from "@/components/ui/forms.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import ActionMenu from "@/components/features/ActionMenu.svelte";
  import Modal from "@/components/ui/Modal.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import { showToast } from "@/components/ui/toast.js";

  let searchQuery = $state("");
  let isLoading = $state(true);
  let errorMsg = $state(null);
  let sortState = $state({ column: "usage", asc: false });
  let dishes = $state([]);
  let searchTimeout;
  let currentLoadToken = 0;

  // --- Modal States ---
  let selectedDish = $state(null);
  let isInfoModalOpen = $state(false);
  let isAddModalOpen = $state(false);
  let isEditModalOpen = $state(false);
  let isMergeModalOpen = $state(false);
  let isDetachModalOpen = $state(false);
  let isSplitModalOpen = $state(false);
  let isDeleteModalOpen = $state(false);

  // Edit Modal Specific States
  let editDishState = $state({
    name: "",
    category: "",
    weight: "",
    calories: "",
    quickPrep: false,
    prePrepared: false,
    constraints: []
  });
  
  // Split & Merge Specific
  let splitDelimiter = $state("/");
  let mergeSearchQuery = $state("");
  let targetDishId = $state(null);
  let mergeTargetName = $state("");
  let mergeSearchResults = $state([]);
  let mergeSearchTimeout;

  function toggleConstraint(c) {
    if (editDishState.constraints.includes(c)) {
      editDishState.constraints = editDishState.constraints.filter(x => x !== c);
    } else {
      editDishState.constraints.push(c);
    }
  }

  function getConstraintStyle(constraint) {
    switch (constraint) {
      case "Vegan":
        return "--pill-bg: rgba(var(--color-accent-positive-rgb), 0.1); --pill-color: var(--color-accent-positive); --pill-border: rgba(var(--color-accent-positive-rgb), 0.2);";
      case "Vejetaryen":
        return "--pill-bg: rgba(var(--color-accent-primary-rgb), 0.1); --pill-color: var(--color-accent-primary); --pill-border: rgba(var(--color-accent-primary-rgb), 0.2);";
      case "Gluten-free":
        return "--pill-bg: rgba(var(--color-warning-rgb), 0.1); --pill-color: var(--color-warning); --pill-border: rgba(var(--color-warning-rgb), 0.2);";
      case "Yüksek Protein":
        return "--pill-bg: rgba(var(--color-accent-secondary-rgb), 0.1); --pill-color: var(--color-accent-secondary); --pill-border: rgba(var(--color-accent-secondary-rgb), 0.2);";
      case "Çiğ":
        return "--pill-bg: rgba(var(--color-accent-tertiary-rgb), 0.1); --pill-color: var(--color-accent-tertiary); --pill-border: rgba(var(--color-accent-tertiary-rgb), 0.2);";
      default:
        return "";
    }
  }

  const categoryConfig = {
    'soup': { label: 'Çorba', color: 'var(--color-accent-tertiary)' },
    'main': { label: 'Ana Yemek', color: 'var(--color-accent-primary)' },
    'side': { label: 'Ara Sıcak / Garnitür', color: 'var(--color-accent-secondary)' },
    'dessert': { label: 'Tatlı / Meyve', color: 'var(--color-warning)' },
    'drink': { label: 'İçecek', color: 'var(--color-accent-positive)' },
    'bread': { label: 'Unlu Mamül', color: 'var(--color-text-secondary)' },
    'extra': { label: 'Ekstra', color: 'var(--color-text-muted)' },
  };

  function getCategoryLabel(cat) {
    return categoryConfig[cat]?.label || 'Belirtilmemiş';
  }

  function getCategoryColor(cat) {
    return categoryConfig[cat]?.color || 'var(--color-border)';
  }

  function getDishActions(dish) {
    const actions = [
      { label: "Düzenle", onClick: () => handleEditDish(dish) },
      { label: "Başka Yemekle Birleştir", onClick: () => handleMerge(dish) },
      { label: "İsme göre böl", onClick: () => handleSplitString(dish) },
    ];

    if (dish.aliases && dish.aliases.length > 0) {
      actions.push({
        label: "Bağlantıları Ayır",
        onClick: () => handleDetachAliases(dish),
      });
    }

    actions.push({
      label: "Sil",
      variant: "danger",
      onClick: () => handleDeleteDish(dish),
    });

    return actions;
  }

  async function fetchDishes(query = '') {
    isLoading = true;
    errorMsg = null;
    dishes = [];
    const token = ++currentLoadToken;
    try {
      const data = await api.getDishStats(query);
      if (token !== currentLoadToken) return;
      dishes = data;
    } catch (err) {
      if (token !== currentLoadToken) return;
      errorMsg = err.message || 'Yemekler yüklenirken bir hata oluştu.';
    } finally {
      if (token === currentLoadToken) {
        isLoading = false;
      }
    }
  }

  onMount(() => {
    fetchDishes();
  });

  $effect(() => {
    return () => {
      clearTimeout(searchTimeout);
    };
  });

  function handleSearchInput(e) {
    searchQuery = e.target.value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      fetchDishes(searchQuery.trim());
    }, 300);
  }

  function handleSort(column) {
    if (sortState.column === column) {
      sortState.asc = !sortState.asc;
    } else {
      sortState.column = column;
      sortState.asc = true;
    }
  }

  let sortedDishes = $derived(
    [...dishes].sort((a, b) => {
      let valA, valB;
      if (sortState.column === "name") {
        valA = a.name.toLocaleLowerCase("tr-TR");
        valB = b.name.toLocaleLowerCase("tr-TR");
      } else if (sortState.column === "category") {
        valA = a.category || "";
        valB = b.category || "";
      } else {
        valA = a.usage_count;
        valB = b.usage_count;
      }

      if (valA < valB) return sortState.asc ? -1 : 1;
      if (valA > valB) return sortState.asc ? 1 : -1;
      return 0;
    }),
  );

  function handleInfoModal(dish) {
    selectedDish = dish;
    isInfoModalOpen = true;
  }

  function handleAddDish() {
    editDishState = {
      name: searchQuery,
      category: "",
    };
    isAddModalOpen = true;
  }

  async function submitAddDish() {
    if (!editDishState.name) return showToast('İsim zorunlu', 'error');
    try {
      await api.createDish(editDishState.name, editDishState.category || null);
      showToast('Yemek eklendi!');
      isAddModalOpen = false;
      fetchDishes(searchQuery.trim());
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  function handleEditDish(dish) {
    selectedDish = dish;
    editDishState = {
      name: dish.name || "",
      category: dish.category || "",
      weight: dish.weight || "",
      calories: dish.calories || "",
      quickPrep: dish.quickPrep || false,
      prePrepared: dish.prePrepared || false,
      constraints: [...(dish.constraints || [])]
    };
    isEditModalOpen = true;
  }

  async function submitEditDish() {
    const newName = editDishState.name.trim();
    const newCategory = editDishState.category.trim();
    const newConstraints = editDishState.constraints;

    const payload = {};
    if (newName && newName !== selectedDish.name) payload.name = newName;
    if (newCategory !== selectedDish.category) payload.category = newCategory;
    
    const oldConstraints = selectedDish.constraints || [];
    if (JSON.stringify([...newConstraints].sort()) !== JSON.stringify([...oldConstraints].sort())) {
      payload.constraints = newConstraints;
    }

    if (Object.keys(payload).length === 0) return (isEditModalOpen = false);

    try {
      const res = await api.updateDish(selectedDish.id, payload);
      if (res && res.error_code === 'DUPLICATE_NAME') {
        showToast(`Bu isimde bir yemek zaten var (#${res.existing_dish.id}). Birleştirmeyi dene.`);
      } else {
        showToast('Yemek güncellendi!');
        isEditModalOpen = false;
        fetchDishes(searchQuery.trim());
      }
    } catch (err) {
      if (err.message && err.message.includes('ix_dishes_name')) {
        showToast('Aynı isimde iki yemek olamaz. Birleşim önerilir.', 'error');
      } else {
        showToast(err.message, 'error');
      }
    }
  }

  function handleMerge(dish) {
    selectedDish = dish;
    mergeSearchQuery = "";
    targetDishId = null;
    mergeTargetName = "";
    mergeSearchResults = [];
    isMergeModalOpen = true;
  }

  function handleMergeSearchInput(e) {
    mergeSearchQuery = e.target.value;
    clearTimeout(mergeSearchTimeout);
    mergeSearchTimeout = setTimeout(async () => {
      const query = mergeSearchQuery.trim();
      if (!query) { mergeSearchResults = []; return; }
      try {
        const results = await api.getDishStats(query);
        mergeSearchResults = results.filter(d => d.id !== selectedDish.id);
      } catch (err) { console.error(err); }
    }, 300);
  }

  function selectMergeTarget(id, name) {
    targetDishId = id;
    mergeTargetName = name;
    mergeSearchQuery = "";
    mergeSearchResults = [];
  }

  async function submitMerge() {
    if (!targetDishId) return showToast('Lütfen hedef yemeği seçin!', 'error');
    if (targetDishId === selectedDish.id) return showToast('Bir yemeği kendisiyle birleştiremezsiniz.', 'error');
    try {
      await api.mergeDishes(selectedDish.id, targetDishId);
      showToast('Yemekler başarıyla birleştirildi!');
      isMergeModalOpen = false;
      fetchDishes(searchQuery.trim());
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  function handleDetachAliases(dish) {
    if (!dish.aliases || dish.aliases.length === 0) return;
    selectedDish = dish;
    isDetachModalOpen = true;
  }

  async function submitDetachAlias(aliasId, aliasName) {
    if (confirm(`'${aliasName}' adlı yemeği ayırmak istediğinize emin misiniz?`)) {
      try {
        await api.detachDish(aliasId);
        showToast('Yemek başarıyla ayrıldı!');
        selectedDish.aliases = selectedDish.aliases.filter(a => a.id !== aliasId);
        if (selectedDish.aliases.length === 0) isDetachModalOpen = false;
        fetchDishes(searchQuery.trim());
      } catch (err) {
        showToast(err.message, 'error');
      }
    }
  }

  function handleSplitString(dish) {
    selectedDish = dish;
    splitDelimiter = "/";
    isSplitModalOpen = true;
  }

  async function submitSplitString() {
    try {
      await api.splitDish(selectedDish.id, splitDelimiter || "/");
      showToast('Yemek başarıyla bölündü!');
      isSplitModalOpen = false;
      fetchDishes(searchQuery.trim());
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  function handleDeleteDish(dish) {
    selectedDish = dish;
    isDeleteModalOpen = true;
  }

  async function submitDeleteDish() {
    try {
      await api.deleteDish(selectedDish.id);
      showToast('Yemek başarıyla silindi.', 'success');
      isDeleteModalOpen = false;
      fetchDishes(searchQuery.trim());
    } catch (err) {
      showToast('Silinemedi: ' + err.message, 'error');
    }
  }
</script>

<svelte:head>
  <title>Yemek - Moderasyon - Kepçe</title>
</svelte:head>

<div class="u-flex u-items-center u-justify-between u-mb-md u-gap-md">
  <div class="admin-search-bar u-flex-grow">
    <span class="admin-search-bar__icon">
      {@html icon("search", 16)}
    </span>
    <input
      type="text"
      class="admin-search-bar__input"
      placeholder="Yemek ismi ara (örn: Pilav, Tavuk...)"
      autocomplete="off"
      value={searchQuery}
      oninput={handleSearchInput}
    />
  </div>
  <button
    class="btn btn--primary u-flex-shrink-0 btn-admin-top-action"
    onclick={handleAddDish}
  >
    <span class="u-hidden-mobile">Yemek ekle</span>
    <span class="u-hidden-desktop">{@html icon("plus", 16)}</span>
  </button>
</div>

<!-- SADECE MOBİL İÇİN SIRALAMA ÇUBUĞU (900px altı) -->
<div class="u-hidden-desktop u-mb-lg u-gap-sm u-items-center">
  <div class="u-flex-grow mobile-sort-dropdown">
    <Dropdown
      options={[
        { label: "Yemek ismine Göre", value: "name" },
        { label: "Kategoriye Göre", value: "category" },
        { label: "Kullanıma Göre", value: "usage" },
      ]}
      bind:value={sortState.column}
    />
  </div>
  <button
    class="btn btn--secondary btn--icon-only u-flex-shrink-0 btn-admin-sort"
    onclick={() => {
      sortState.asc = !sortState.asc;
    }}
  >
    {@html icon(sortState.asc ? "chevronUp" : "chevronDown", 16)}
  </button>
</div>

<div id="dish-list-container">
  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState statusCode={500} desc={errorMsg} />
  {:else if dishes.length === 0}
    <EmptyState
      iconName={"ghost"}
      title={"Sonuç Yok"}
      desc={"Böyle bir yemek bulamadık. Yanlış mı yazdın?"}
    />
  {:else}
    <div class="admin-table-wrapper admin-table-wrapper--no-scroll">
      <table class="admin-table admin-table--hybrid" id="dish-table">
        <thead>
          <tr>
            <th class="sortable {sortState.column === 'name' ? (sortState.asc ? 'sort-asc' : 'sort-desc') : ''}" onclick={() => handleSort("name")}>Yemek ismi</th>
            <th class="sortable {sortState.column === 'category' ? (sortState.asc ? 'sort-asc' : 'sort-desc') : ''}" onclick={() => handleSort("category")}>Kategori</th>
            <th>Kısıtlamalar</th>
            <th class="sortable {sortState.column === 'usage' ? (sortState.asc ? 'sort-asc' : 'sort-desc') : ''}" onclick={() => handleSort("usage")}>Kullanım</th>
            <th class="col-actions">Aksiyonlar</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedDishes as dish (dish.id)}
            <tr data-id={dish.id}>
              <td>
                <div class="admin-table-cell--primary">{dish.name}</div>
                {#if dish.aliases?.length > 0}
                  {#if dish.aliases.length === 1}
                    <div class="u-text-xs u-color-text-muted u-mt-xs">Bağlı: {dish.aliases[0].name}</div>
                  {:else}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                    <details class="admin-aliases-expander u-mt-xs u-cursor-pointer" onclick={(e) => e.stopPropagation()}>
                      <summary class="u-text-xs u-color-text-muted u-font-medium admin-aliases-summary">
                        {dish.aliases.length} Bağlantı {@html icon('chevronDown', 12)}
                      </summary>
                      <div class="u-pl-md u-mt-xs u-text-xs u-color-text-muted u-flex u-flex-col u-gap-xs admin-aliases-list">
                        {#each dish.aliases as a}
                          <div class="u-flex u-items-center u-justify-between">
                            <span>{a.name}</span>
                            <span>ID: {a.id}</span>
                          </div>
                        {/each}
                      </div>
                    </details>
                  {/if}
                {/if}
              </td>
              <td>
                <span class="admin-table-cell--secondary">{getCategoryLabel(dish.category)}</span>
              </td>
              <td>
                {#if dish.constraints && dish.constraints.length > 0}
                  <span class="admin-table-cell--pill-group">
                    {#each dish.constraints as constraint}
                      <span class="admin-table-cell--pill" style={getConstraintStyle(constraint)}>{constraint}</span>
                    {/each}
                  </span>
                {:else}
                  <span class="u-text-muted">-</span>
                {/if}
              </td>
              <td><span class="admin-table-cell--meta">{dish.usage_count} kez</span></td>
              <td class="col-actions">
                <div class="u-flex u-items-center u-gap-xs u-justify-end">
                  <button class="btn-icon" onclick={() => handleInfoModal(dish)} aria-label="Detayları Gör">
                    {@html icon('info', 16)}
                  </button>
                  <ActionMenu items={getDishActions(dish)} />
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- MODALS -->
{#if isInfoModalOpen}
<Modal options={{ title: `${selectedDish?.name} Detayları`, iconHtml: icon('info', 24) }} onClose={() => (isInfoModalOpen = false)}>
  {#snippet children()}
    {#if selectedDish}
      <div class="u-mb-lg">
        <div>
          <div class="u-text-xs u-color-muted u-font-bold">KATEGORİ</div>
          <div class="u-mt-xs u-font-medium">{getCategoryLabel(selectedDish.category)}</div>
        </div>
        {#if selectedDish.constraints?.length > 0}
          <div class="u-mt-md">
            <div class="u-text-xs u-color-muted u-font-bold">KISITLAMALAR</div>
            <div class="admin-table-cell--pill-group u-mt-xs">
              {#each selectedDish.constraints as c}
                <span class="admin-table-cell--pill" style={getConstraintStyle(c)}>{c}</span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
      <div class="u-mb-lg">
        <div class="u-text-xs u-color-muted u-font-bold u-mb-xs">METRİKLER</div>
        <div class="c-boxed-list">
          <div class="c-list-row u-flex u-justify-between u-items-center">
            <span class="u-text-sm u-color-muted u-font-medium">Kullanım</span>
            <span class="u-font-bold u-color-text">{selectedDish.usage_count} kez</span>
          </div>
          {#if selectedDish.weight}
            <div class="c-list-row u-flex u-justify-between u-items-center">
              <span class="u-text-sm u-color-muted u-font-medium">Porsiyon</span>
              <span class="u-font-bold u-color-text">{selectedDish.weight}</span>
            </div>
          {/if}
          {#if selectedDish.calories}
            <div class="c-list-row u-flex u-justify-between u-items-center">
              <span class="u-text-sm u-color-muted u-font-medium">Kalori</span>
              <span class="u-font-bold u-color-text">{selectedDish.calories} kcal</span>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isInfoModalOpen = false}>Kapat</button>
  {/snippet}
</Modal>
{/if}

{#if isAddModalOpen}
<Modal options={{ title: "Yeni Yemek Ekle", iconHtml: icon('plus', 24) }} onClose={() => (isAddModalOpen = false)}>
  {#snippet children()}
    <div class="c-modal__form-group">
      <label for="add-dish-name" class="c-modal__label">Yemek ismi</label>
      <input id="add-dish-name" type="text" class="c-modal__input" bind:value={editDishState.name}>
    </div>
    <div class="c-modal__form-group">
      <div class="c-modal__label">Kategori</div>
      <Dropdown
        options={[
          { value: "", label: "Belirtilmemiş" },
          ...Object.entries(categoryConfig).map(([k, v]) => ({ value: k, label: v.label }))
        ]}
        bind:value={editDishState.category}
      />
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isAddModalOpen = false}>İptal</button>
    <button class="btn btn--primary" onclick={submitAddDish}>Ekle</button>
  {/snippet}
</Modal>
{/if}

{#if isEditModalOpen}
<Modal options={{ title: "Yemek Düzenle" }} onClose={() => (isEditModalOpen = false)}>
  {#snippet children()}
    <div class="c-modal__form-group">
      <label for="edit-dish-name" class="c-modal__label">Yemek ismi</label>
      <input id="edit-dish-name" type="text" class="c-modal__input" bind:value={editDishState.name}>
      <p class="c-modal__help">İsim değişirse, eski isim otomatik olarak bir takma ad olarak kaydedilir.</p>
    </div>
    <div class="c-modal__form-group">
      <div class="c-modal__label">Kategori</div>
      <Dropdown
        options={[
          { value: "", label: "Belirtilmemiş" },
          ...Object.entries(categoryConfig).map(([k, v]) => ({ value: k, label: v.label }))
        ]}
        bind:value={editDishState.category}
      />
    </div>
    <div class="c-modal__form-group">
      <div class="c-modal__label">Kısıtlamalar</div>
      <div class="u-mt-xs admin-grid-half u-gap-xs">
        {#each ["Vegan", "Vejetaryen", "Gluten-free", "Yüksek Protein", "Çiğ"] as c}
          <label class="form-switch-row u-cursor-pointer u-py-xs">
            <input type="checkbox" class="c-input-hidden" checked={editDishState.constraints.includes(c)} onchange={() => toggleConstraint(c)}>
            <div class="c-switch"><div class="c-switch__handle"></div></div>
            <span class="form-switch-row__text u-ml-sm">{c}</span>
          </label>
        {/each}
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isEditModalOpen = false}>İptal</button>
    <button class="btn btn--primary" onclick={submitEditDish}>Değişiklikleri kaydet</button>
  {/snippet}
</Modal>
{/if}

{#if isMergeModalOpen}
<Modal options={{ title: "Yemek Birleştir" }} onClose={() => (isMergeModalOpen = false)}>
  {#snippet children()}
    {#if selectedDish}
      <p class="u-mb-md u-text-sm"><strong>{selectedDish.name}</strong> yemeğini hedef yemeğe aktarıp birleştireceksiniz.</p>
      
      {#if !targetDishId}
        <div class="c-modal__form-group" id="merge-search-group">
          <label for="merge-search-input" class="c-modal__label">Hedef yemeği ara</label>
          <div class="admin-search-wrapper">
            <input id="merge-search-input" type="text" class="c-modal__input" bind:value={mergeSearchQuery} oninput={handleMergeSearchInput} placeholder="Hedef yemek ismi yaz...">
          </div>
          <div class="admin-modal-search-results">
            {#each mergeSearchResults as d}
              <div class="dish-item-select">
                <div>
                  <div class="u-text-sm u-font-bold">{d.name}</div>
                  <div class="u-text-xs u-color-muted">ID: {d.id} | {getCategoryLabel(d.category)}</div>
                </div>
                <button class="btn btn--xs btn--primary select-target-btn" onclick={() => selectMergeTarget(d.id, d.name)}>Seç</button>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div id="merge-target-selection" class="u-mt-md">
          <div class="c-modal__label">Seçilen hedef yemek</div>
          <div class="admin-selected-dishes-box">
            <div class="chip active u-m-0">
              <span>{mergeTargetName}</span>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span class="remove-dish u-cursor-pointer u-ml-xs" onclick={() => { targetDishId = null; mergeTargetName = ""; }}>×</span>
            </div>
          </div>
          <p class="u-mt-md u-text-xs u-color-negative">DİKKAT: Bu işlem geri alınamaz. "<strong>{sanitizeText(selectedDish.name)}</strong>" silinecek ve tüm kullanım istatistikleri hedef yemeğe aktarılacak.</p>
        </div>
      {/if}
    {/if}
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isMergeModalOpen = false}>İptal</button>
    <button class="btn btn--danger" disabled={!targetDishId} onclick={submitMerge}>Birleştir</button>
  {/snippet}
</Modal>
{/if}

{#if isDetachModalOpen}
<Modal options={{ title: "Bağlantıları Ayır", iconHtml: icon('split', 24) }} onClose={() => (isDetachModalOpen = false)}>
  {#snippet children()}
    {#if selectedDish && selectedDish.aliases}
      <div class="c-boxed-list u-mb-md">
        <div class="u-p-md admin-list-section-header">
          <p class="u-text-sm u-font-bold">Mevcut bağlantılar (takma adlar)</p>
          <p class="u-text-xs u-opacity-80 u-mt-xs">Bağlı takma adları bağımsız birer kayda dönüştürür.</p>
        </div>
        <div class="u-flex u-flex-col">
          {#each selectedDish.aliases as a}
            <div class="c-list-row admin-list-row--flush u-flex u-items-center u-justify-between">
              <div>
                <span class="u-font-medium">{a.name}</span>
                <span class="u-text-xs u-color-text-muted u-ml-xs">(ID: {a.id})</span>
              </div>
              <button class="btn btn--secondary btn--sm" onclick={() => submitDetachAlias(a.id, a.name)}>Ayır</button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isDetachModalOpen = false}>Kapat</button>
  {/snippet}
</Modal>
{/if}

{#if isSplitModalOpen}
<Modal options={{ title: "İsme göre böl", iconHtml: icon('split', 24) }} onClose={() => (isSplitModalOpen = false)}>
  {#snippet children()}
    {#if selectedDish}
      <div class="c-boxed-list">
        <div class="u-p-md admin-list-section-header">
          <p class="u-text-sm u-font-bold">İsme göre böl</p>
          <p class="u-text-xs u-opacity-80 u-mt-xs">Birleşik metinleri ayırarak yeni kayıtlar oluşturur.</p>
        </div>
        <div class="u-p-md">
          <p class="u-text-sm u-mb-md"><strong>{selectedDish.name}</strong> kaydını hangi karaktere göre böleceksiniz?</p>
          <div class="u-flex u-items-center u-gap-md">
            <input type="text" class="c-modal__input admin-input--narrow" bind:value={splitDelimiter}>
            <button class="btn btn--primary" onclick={submitSplitString}>İsme göre böl</button>
          </div>
        </div>
      </div>
    {/if}
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isSplitModalOpen = false}>Kapat</button>
  {/snippet}
</Modal>
{/if}

{#if isDeleteModalOpen}
<Modal options={{ title: "Yemeği Sil", iconHtml: icon('trash', 24), iconColor: 'danger' }} onClose={() => (isDeleteModalOpen = false)}>
  {#snippet children()}
    {#if selectedDish}
      <p><strong>{selectedDish.name}</strong> adlı yemeği kalıcı olarak silmek üzeresiniz. Bu işlem geri alınamaz.</p>
    {/if}
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => isDeleteModalOpen = false}>İptal</button>
    <button class="btn btn--danger" onclick={submitDeleteDish}>Kalıcı sil</button>
  {/snippet}
</Modal>
{/if}

<style>
  /* Native details element oklarını gizle */
  .admin-aliases-expander summary::-webkit-details-marker {
    display: none;
  }

  /* Özel Number Input oklarını gizle (Modal için global) */
  :global(.hide-spinners::-webkit-inner-spin-button), 
  :global(.hide-spinners::-webkit-outer-spin-button) { 
    -webkit-appearance: none !important; 
    margin: 0 !important; 
  }
  :global(.hide-spinners) {
    -moz-appearance: textfield !important;
    appearance: none !important;
  }
</style>
