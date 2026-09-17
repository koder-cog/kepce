<script>
  import "@/styles/pages/_menu-table.css";
  import { onMount, tick } from "svelte";
  import { api } from '@/api/index.js';
  import { getCitiesData } from "@/stores/city.svelte.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Loader from "@/components/ui/Loader.svelte";
  import { icon } from "@/components/ui/icons.js";
  import * as ui from "@/components/ui/forms.js";
  import { showToast } from "@/components/ui/toast.js";
  import Modal from "@/components/ui/Modal.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import ActionMenu from "@/components/features/ActionMenu.svelte";

  let cities = $state([]);
  let menuCityFilter = $state("");
  let menuYearFilter = $state(new Date().getFullYear().toString());
  let menuMonthFilter = $state((new Date().getMonth() + 1).toString().padStart(2, '0'));
  let menuStatusFilter = $state("");
  let displayLimit = $state(10);

  let isLoading = $state(true);
  let groupedMenus = $state({});
  let errorMsg = $state(null);

  // Modal State
  let isEditMenuModalOpen = $state(false);
  let editMenuTarget = $state(null);
  let editMenuNotice = $state("");
  let editMenuSourceType = $state("kepce-admin");
  let editMenuSlots = $state([]);
  let activeSlotTarget = $state({ slotIndex: 0, isAlternative: false });
  let editMenuSearchQuery = $state("");
  let editMenuSearchResults = $state([]);
  let searchTimeout;
  let currentLoadToken = 0;
  let currentSearchToken = 0;

  let isEditBotModalOpen = $state(false);
  let editBotTarget = $state(null);
  let editBotCommentText = $state("");

  // Yeni Menü Oluşturma State'i
  let isCreateMenuModalOpen = $state(false);
  let newMenuCityId = $state(null);
  let newMenuDate = $state(new Date().toISOString().split('T')[0]);
  let newMenuMealType = $state("breakfast");
  let newMenuSourceType = $state("kepce-admin");
  let newMenuNotice = $state("");
  let isCreatingMenu = $state(false);

  const currentYear = new Date().getFullYear();
  const yearOptions = [
    { label: "Tümü", value: "" },
    { label: (currentYear - 1).toString(), value: (currentYear - 1).toString() },
    { label: currentYear.toString(), value: currentYear.toString() },
    { label: (currentYear + 1).toString(), value: (currentYear + 1).toString() },
  ];

  const monthsTR = ["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"];

  let sortedGroupedMenus = $derived(
    Object.entries(groupedMenus)
      .sort(([cityA], [cityB]) => cityA.localeCompare(cityB, "tr"))
      .map(([city, monthsObj]) => {
        const sortedMonths = Object.entries(monthsObj)
          .sort(([monthA], [monthB]) => monthB.localeCompare(monthA)) // YYYY-MM ters sıralama
          .map(([monthKey, menus]) => {
            const [y, m] = monthKey.split("-");
            const monthName = `${monthsTR[parseInt(m, 10) - 1]} ${y}`;
            return { monthKey, monthName, menus };
          });
        return { city, months: sortedMonths };
      }),
  );

  let totalMonthsCount = $derived(
    sortedGroupedMenus.reduce((acc, cityGroup) => acc + cityGroup.months.length, 0),
  );

  let displayedMenus = $derived.by(() => {
    let count = 0;
    const result = [];
    for (const cityGroup of sortedGroupedMenus) {
      if (count >= displayLimit) break;
      const monthsToTake = cityGroup.months.slice(0, displayLimit - count);
      result.push({ city: cityGroup.city, months: monthsToTake });
      count += monthsToTake.length;
    }
    return result;
  });

  async function loadInitialData() {
    try {
      cities = await getCitiesData();
    } catch (err) { console.error(err); }
    fetchMenus();
  }

  onMount(() => {
    loadInitialData();
  });

  $effect(() => {
    return () => {
      clearTimeout(searchTimeout);
    };
  });

  async function fetchMenus() {
    isLoading = true;
    errorMsg = null;
    groupedMenus = {};
    
    let monthFilterStr = '';
    const token = ++currentLoadToken;
    if (menuYearFilter && menuMonthFilter) {
      monthFilterStr = `${menuYearFilter}-${menuMonthFilter}`;
    } else if (menuYearFilter) {
      monthFilterStr = menuYearFilter;
    }

    try {
      const menus = await api.getMenus(menuStatusFilter, menuCityFilter, monthFilterStr);
      if (token !== currentLoadToken) return;
      displayLimit = 10;
      if (menus.length > 0) {
        groupedMenus = menus.reduce((acc, menu) => {
          const city = menu.city?.name || 'Bilinmeyen Şehir';
          const [y, m] = menu.date.split('-');
          const monthKey = `${y}-${m}`;
          if (!acc[city]) acc[city] = {};
          if (!acc[city][monthKey]) acc[city][monthKey] = [];
          acc[city][monthKey].push(menu);
          return acc;
        }, {});
      }
    } catch (err) {
      if (token !== currentLoadToken) return;
      errorMsg = err.message || 'Menüler yüklenirken bir hata oluştu.';
    } finally {
      if (token === currentLoadToken) {
        isLoading = false;
      }
    }
  }

  function handleFilter() {
    fetchMenus();
  }

  async function handleEditMenuItems(menu) {
    editMenuTarget = menu;
    editMenuNotice = menu.notice || "";
    editMenuSourceType = menu.source_type || "kepce-admin";
    editMenuSearchQuery = "";
    editMenuSearchResults = [];
    isEditMenuModalOpen = true;
    try {
      const dishes = await api.getMenuDishIds(menu.id);
      const map = new Map();
      for (const d of dishes) {
        const idx = d.order_index ?? 0;
        // Aynı slot'ta birden fazla package_name olabilir (NORMAL + ÇÖLYAK), ama
        // admin düzenleyici tek bir package_name'i slot bazında tutar; NORMAL dışı
        // paketler ayrı slot olarak gelir.
        const key = `${idx}::${d.package_name ?? 'NORMAL'}`;
        if (!map.has(key)) {
          map.set(key, { order_index: idx, package_name: d.package_name ?? 'NORMAL', primary: null, alternatives: [] });
        }
        const slot = map.get(key);
        if (d.is_alternative) {
          slot.alternatives.push(d);
        } else {
          slot.primary = d;
        }
      }
      const sorted = Array.from(map.values()).sort((a, b) => a.order_index - b.order_index || a.package_name.localeCompare(b.package_name));
      editMenuSlots = sorted.length > 0 ? sorted : [{ order_index: 0, package_name: 'NORMAL', primary: null, alternatives: [] }];
      activeSlotTarget = { slotIndex: 0, isAlternative: false };
    } catch (err) {
      editMenuSlots = [{ order_index: 0, primary: null, alternatives: [] }];
      activeSlotTarget = { slotIndex: 0, isAlternative: false };
      showToast("Yemekler yüklenemedi", "error");
    }
  }

  function handleDishSearchInput(e) {
    editMenuSearchQuery = e.target.value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(async () => {
      const query = editMenuSearchQuery.trim();
      if (!query) { editMenuSearchResults = []; return; }
      const token = ++currentSearchToken;
      try {
        const results = await api.getDishStats(query);
        if (token !== currentSearchToken) return;
        editMenuSearchResults = results;
      } catch (err) { console.error(err); }
    }, 300);
  }

  let isApplyingTemplate = $state(false);

  async function applyBreakfastTemplate() {
    if (editMenuSlots.some((s) => s.primary)) {
      if (!confirm("Mevcut yemek sıraları standart kahvaltı şablonuyla değiştirilsin mi?")) {
        return;
      }
    }

    isApplyingTemplate = true;
    try {
      const templateItems = [
        'Haşlanmış Yumurta',
        'Beyaz Peynir',
        'Siyah Zeytin',
        'Reçel',
        'Ekmek'
      ];

      const newSlots = [];
      for (let i = 0; i < templateItems.length; i++) {
        const itemQuery = templateItems[i];
        let dishMatch = null;
        try {
          const results = await api.getDishStats(itemQuery);
          if (results && results.length > 0) {
            dishMatch = results.find((d) => d.name.toLowerCase().includes(itemQuery.toLowerCase())) || results[0];
          }
        } catch (_) {}

        if (dishMatch) {
          newSlots.push({
            order_index: i,
            package_name: 'NORMAL',
            primary: dishMatch,
            alternatives: []
          });
        }
      }

      if (newSlots.length > 0) {
        editMenuSlots = newSlots;
        activeSlotTarget = { slotIndex: 0, isAlternative: false };
        showToast(`${newSlots.length} kahvaltılık yemek şablondan yüklendi.`);
      } else {
        showToast("Kahvaltı yemekleri veritabanında arandı ancak eşleşme bulunamadı.", "warning");
      }
    } catch (err) {
      showToast("Şablon uygulanırken hata oluştu: " + err.message, "error");
    } finally {
      isApplyingTemplate = false;
    }
  }

  function addNewSlot() {
    const nextIdx = editMenuSlots.length;
    editMenuSlots.push({ order_index: nextIdx, package_name: 'NORMAL', primary: null, alternatives: [] });
    activeSlotTarget = { slotIndex: nextIdx, isAlternative: false };
  }

  function removeSlot(slotIndex) {
    editMenuSlots = editMenuSlots.filter((_, i) => i !== slotIndex);
    if (editMenuSlots.length === 0) {
      editMenuSlots = [{ order_index: 0, primary: null, alternatives: [] }];
    }
    activeSlotTarget = {
      slotIndex: Math.min(activeSlotTarget.slotIndex, editMenuSlots.length - 1),
      isAlternative: false,
    };
  }

  function setTargetForSlot(slotIndex, isAlternative) {
    activeSlotTarget = { slotIndex, isAlternative };
    tick().then(() => {
      const el = document.getElementById('dish-search');
      if (el) {
        el.focus({ preventScroll: false });
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    });
  }

  function addDishToSlot(dish) {
    let { slotIndex, isAlternative } = activeSlotTarget;
    if (slotIndex >= editMenuSlots.length) {
      addNewSlot();
      slotIndex = editMenuSlots.length - 1;
    }
    const currentSlot = editMenuSlots[slotIndex];
    if (isAlternative) {
      if (!currentSlot.alternatives.find((a) => a.id === dish.id)) {
        currentSlot.alternatives.push(dish);
        showToast(`Alternatif olarak eklendi (Sıra ${slotIndex + 1})`);
      }
    } else {
      currentSlot.primary = dish;
      showToast(`Ana yemek olarak eklendi (Sıra ${slotIndex + 1})`);
    }
  }

  function removePrimaryFromSlot(slotIndex) {
    if (editMenuSlots[slotIndex]) {
      editMenuSlots[slotIndex].primary = null;
    }
  }

  function removeAlternativeFromSlot(slotIndex, altDishId) {
    if (editMenuSlots[slotIndex]) {
      editMenuSlots[slotIndex].alternatives = editMenuSlots[slotIndex].alternatives.filter((a) => a.id !== altDishId);
    }
  }

  async function saveMenuDishes() {
    try {
      const items = [];
      editMenuSlots.forEach((slot, sIdx) => {
        const pkg = slot.package_name || 'NORMAL';
        if (slot.primary) {
          items.push({
            dish_id: slot.primary.id,
            order_index: sIdx,
            is_alternative: false,
            package_name: pkg,
          });
        }
        if (slot.alternatives && slot.alternatives.length > 0) {
          slot.alternatives.forEach((alt) => {
            items.push({
              dish_id: alt.id,
              order_index: sIdx,
              is_alternative: true,
              package_name: pkg,
            });
          });
        }
      });

      await api.updateMenuItems(editMenuTarget.id, {
        items,
        notice: editMenuNotice,
        source_type: editMenuSourceType,
      });
      showToast("Menü yemekleri ve ayarları güncellendi!");
      isEditMenuModalOpen = false;
      fetchMenus();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  function handleEditBotComment(menu) {
    editBotTarget = menu;
    editBotCommentText = menu.bot_commentary || "";
    isEditBotModalOpen = true;
  }

  async function saveBotComment() {
    try {
      await api.updateMenuCommentary(editBotTarget.id, editBotCommentText.trim());
      showToast("Bot yorumu başarıyla güncellendi!");
      isEditBotModalOpen = false;
      fetchMenus();
    } catch (err) {
      showToast(err.message, 'error');
    }
  }

  let isBulkProcessing = $state(false);
  let selectedMenuIds = $state(new Set());

  function toggleSelectMenu(id) {
    const next = new Set(selectedMenuIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedMenuIds = next;
  }

  function toggleSelectAllMonth(menus) {
    const allSelected = menus.length > 0 && menus.every((m) => selectedMenuIds.has(m.id));
    const next = new Set(selectedMenuIds);
    if (allSelected) {
      menus.forEach((m) => next.delete(m.id));
    } else {
      menus.forEach((m) => next.add(m.id));
    }
    selectedMenuIds = next;
  }

  async function handleBulkApprovePending(menus) {
    const pending = menus.filter((m) => m.status === 'pending');
    if (pending.length === 0) {
      showToast("Bu ayda onay bekleyen menü bulunamadı.");
      return;
    }
    isBulkProcessing = true;
    try {
      const res = await api.bulkUpdateMenuStatus(pending.map((m) => m.id), 'approved');
      showToast(`${res.updated_count} menü toplu onaylandı!`);
      fetchMenus();
    } catch (err) {
      showToast(err.message, 'error');
    } finally {
      isBulkProcessing = false;
    }
  }

  async function handleBulkRejectMonth(menus) {
    if (!confirm(`Bu aydaki ${menus.length} menünün tamamını reddetmek istediğinize emin misiniz?`)) {
      return;
    }
    isBulkProcessing = true;
    try {
      const res = await api.bulkUpdateMenuStatus(menus.map((m) => m.id), 'rejected');
      showToast(`${res.updated_count} menü toplu reddedildi.`, 'danger');
      fetchMenus();
    } catch (err) {
      showToast(err.message, 'error');
    } finally {
      isBulkProcessing = false;
    }
  }

  async function handleBulkApproveSelected(menus) {
    const selected = menus.filter((m) => selectedMenuIds.has(m.id));
    if (selected.length === 0) return;
    isBulkProcessing = true;
    try {
      const res = await api.bulkUpdateMenuStatus(selected.map((m) => m.id), 'approved');
      showToast(`${res.updated_count} seçili menü onaylandı!`);
      const next = new Set(selectedMenuIds);
      selected.forEach((m) => next.delete(m.id));
      selectedMenuIds = next;
      fetchMenus();
    } catch (err) {
      showToast(err.message, 'error');
    } finally {
      isBulkProcessing = false;
    }
  }

  async function handleBulkRejectSelected(menus) {
    const selected = menus.filter((m) => selectedMenuIds.has(m.id));
    if (selected.length === 0) return;
    if (!confirm(`Seçilen ${selected.length} menüyü reddetmek istediğinize emin misiniz?`)) {
      return;
    }
    isBulkProcessing = true;
    try {
      const res = await api.bulkUpdateMenuStatus(selected.map((m) => m.id), 'rejected');
      showToast(`${res.updated_count} seçili menü reddedildi.`, 'danger');
      const next = new Set(selectedMenuIds);
      selected.forEach((m) => next.delete(m.id));
      selectedMenuIds = next;
      fetchMenus();
    } catch (err) {
      showToast(err.message, 'error');
    } finally {
      isBulkProcessing = false;
    }
  }

  async function approveMenu(id) {
    try {
      await api.approveMenu(id);
      showToast("Menü onaylandı!");
      fetchMenus();
    } catch (err) { showToast(err.message, 'error'); }
  }

  async function rejectMenu(id) {
    try {
      await api.rejectMenu(id);
      showToast("Menü reddedildi.", "danger");
      fetchMenus();
    } catch (err) { showToast(err.message, 'error'); }
  }

  function openCreateMenuModal() {
    if (menuCityFilter) {
      const matched = cities.find((c) => c.slug === menuCityFilter);
      newMenuCityId = matched ? matched.id : (cities[0]?.id || 1);
    } else {
      newMenuCityId = cities[0]?.id || 1;
    }
    newMenuDate = new Date().toISOString().split('T')[0];
    newMenuMealType = "breakfast";
    newMenuSourceType = "kepce-admin";
    newMenuNotice = "";
    isCreateMenuModalOpen = true;
  }

  async function submitCreateMenu() {
    if (!newMenuCityId) return showToast("Lütfen bir şehir seçin", "error");
    if (!newMenuDate) return showToast("Lütfen bir tarih seçin", "error");
    if (!newMenuMealType) return showToast("Lütfen bir öğün seçin", "error");

    isCreatingMenu = true;
    try {
      const createdMenu = await api.createMenu({
        city_id: parseInt(newMenuCityId, 10),
        serve_date: newMenuDate,
        meal_type: newMenuMealType,
        source_type: newMenuSourceType.trim() || "kepce-admin",
        notice: newMenuNotice.trim() || null,
      });

      showToast("Menü başarıyla oluşturuldu");
      isCreateMenuModalOpen = false;

      const createdCity = cities.find((c) => c.id === parseInt(newMenuCityId, 10));
      if (createdCity) {
        menuCityFilter = createdCity.slug;
      }
      const [year, month] = newMenuDate.split('-');
      menuYearFilter = year;
      menuMonthFilter = month;

      await fetchMenus();

      if (createdMenu && createdMenu.id) {
        handleEditMenuItems(createdMenu);
      }
    } catch (err) {
      showToast(err.message || "Menü oluşturulamadı", "error");
    } finally {
      isCreatingMenu = false;
    }
  }
</script>

<svelte:head>
  <title>Menü Tabela Yönetimi - Moderasyon - Kepçe</title>
</svelte:head>

<div class="admin-filter-bar u-mb-md">
  <div class="admin-filter-grid-4">
    <div class="dev-filter-group">
      <span class="admin-filter-label">Şehir</span>
      <Dropdown
        options={[
          { label: "Tümü", value: "" },
          ...cities.map((c) => ({ label: c.name, value: c.slug })),
        ]}
        bind:value={menuCityFilter}
        onChange={handleFilter}
      />
    </div>

    <div class="dev-filter-group">
      <span class="admin-filter-label">Yıl</span>
      <Dropdown
        options={yearOptions}
        bind:value={menuYearFilter}
        onChange={handleFilter}
      />
    </div>

    <div class="dev-filter-group">
      <span class="admin-filter-label">Ay</span>
      <Dropdown
        options={[
          { label: "Tümü", value: "" },
          ...monthsTR.map((m, i) => ({
            label: m,
            value: (i + 1).toString().padStart(2, "0"),
          })),
        ]}
        bind:value={menuMonthFilter}
        onChange={handleFilter}
      />
    </div>

    <div class="dev-filter-group">
      <span class="admin-filter-label">Durum</span>
      <Dropdown
        options={[
          { label: "Tümü", value: "" },
          { label: "Onay Bekleyenler", value: "pending" },
          { label: "Onaylananlar", value: "approved" },
          { label: "Reddedilenler", value: "rejected" },
        ]}
        bind:value={menuStatusFilter}
        onChange={handleFilter}
      />
    </div>
  </div>

  <button
    type="button"
    class="btn btn--primary btn--squish u-flex-shrink-0"
    onclick={openCreateMenuModal}
  >
    <span class="u-hidden-mobile">Yeni Menü Ekle</span>
    <span class="u-hidden-desktop">{@html icon('plus', 16)}</span>
  </button>
</div>

<div id="menu-list-container" class="u-mt-lg">
  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState statusCode={500} desc={errorMsg} />
  {:else if Object.keys(groupedMenus).length === 0}
    <EmptyState
      iconName={"calendar"}
      title={"Kayıt Bulunamadı"}
      desc={"Seçilen filtrelere uygun menü bulunamadı."}
    />
  {:else}
    {#each displayedMenus as { city, months } (city)}
      <div class="u-mb-xl">
        <h3 class="u-mb-md">{city}</h3>
        {#each months as { monthKey, monthName, menus } (monthKey)}
          {@const pendingCount = menus.filter((m) => m.status === 'pending').length}
          {@const approvedCount = menus.filter((m) => m.status === 'approved').length}
          {@const rejectedCount = menus.filter((m) => m.status === 'rejected').length}
          {@const selectedInMonthCount = menus.filter((m) => selectedMenuIds.has(m.id)).length}
          {@const allSelectedInMonth = menus.length > 0 && selectedInMonthCount === menus.length}
          <div class="u-mb-lg">
            <div class="admin-month-header">
              <div class="admin-month-title-group">
                <h4 class="u-color-accent-primary">{monthName}</h4>
                <span class="u-text-xs u-color-muted">
                  {menus.length} Menü ({pendingCount} Bekleyen, {approvedCount} Onaylı, {rejectedCount} Reddedildi)
                </span>
              </div>
              <div class="admin-month-actions">
                {#if selectedInMonthCount > 0}
                  <button
                    type="button"
                    class="btn btn--sm btn--success btn--squish"
                    disabled={isBulkProcessing}
                    onclick={() => handleBulkApproveSelected(menus)}
                  >
                    Seçilenleri Onayla ({selectedInMonthCount})
                  </button>
                  <button
                    type="button"
                    class="btn btn--sm btn--danger btn--squish"
                    disabled={isBulkProcessing}
                    onclick={() => handleBulkRejectSelected(menus)}
                  >
                    Seçilenleri Reddet ({selectedInMonthCount})
                  </button>
                {:else}
                  {#if pendingCount > 0}
                    <button
                      type="button"
                      class="btn btn--sm btn--success btn--squish"
                      disabled={isBulkProcessing}
                      onclick={() => handleBulkApprovePending(menus)}
                    >
                      Bekleyenleri Onayla ({pendingCount})
                    </button>
                  {/if}
                  <button
                    type="button"
                    class="btn btn--sm btn--danger btn--squish"
                    disabled={isBulkProcessing}
                    onclick={() => handleBulkRejectMonth(menus)}
                  >
                    Tümünü Reddet
                  </button>
                {/if}
              </div>
            </div>
            <div class="admin-table-wrapper">
              <table class="admin-table admin-table--hybrid">
                <thead>
                  <tr>
                    <th>
                      <div class="u-flex u-items-center u-gap-xs">
                        <input
                          type="checkbox"
                          checked={allSelectedInMonth}
                          onchange={() => toggleSelectAllMonth(menus)}
                          title="Tümünü seç / kaldır"
                          aria-label="Bu aydaki tüm menüleri seç"
                        />
                        <span>Tarih</span>
                      </div>
                    </th>
                    <th>Öğün</th>
                    <th>Durum</th>
                    <th>Bot yorumu</th>
                    <th class="col-actions">Aksiyonlar</th>
                  </tr>
                </thead>
                <tbody>
                  {#each menus as menu (menu.id)}
                    <tr data-id={menu.id}>
                      <td>
                        <div class="u-flex u-items-center u-gap-xs">
                          <input
                            type="checkbox"
                            checked={selectedMenuIds.has(menu.id)}
                            onchange={() => toggleSelectMenu(menu.id)}
                            aria-label="Menü seç"
                          />
                          <div class="admin-table-cell--primary">
                            {(() => {
                              const [, m, d] = menu.date.split("-");
                              return `${parseInt(d, 10)} ${monthsTR[parseInt(m, 10) - 1]}`;
                            })()}
                          </div>
                        </div>
                      </td>
                      <td
                        ><span class="admin-table-cell--secondary"
                          >{menu.meal_type}</span
                        ></td
                      >
                      <td>
                        {@html ui.createBadge({
                          label:
                            menu.status === "approved"
                              ? "Onaylandı"
                              : menu.status === "rejected"
                                ? "Reddedildi"
                                : "Bekliyor",
                          variant:
                            menu.status === "approved"
                              ? "success"
                              : menu.status === "rejected"
                                ? "danger"
                                : "warning",
                          size: "sm",
                        })}
                      </td>
                      <td>
                        <div
                          class="admin-table-cell--meta u-text-sm"
                          title={menu.bot_commentary || ""}
                        >
                          <span
                            class="u-hidden-desktop u-text-xs u-color-muted u-mr-xs"
                            >Bot yorumu:</span
                          >
                          {#if menu.bot_commentary}
                            {sanitizeText(
                              menu.bot_commentary.substring(0, 30),
                            ) + (menu.bot_commentary.length > 30 ? "..." : "")}
                          {:else}
                            <span class="u-color-muted">Yok</span>
                          {/if}
                        </div>
                      </td>
                      <td class="col-actions">
                        <ActionMenu
                          items={[
                            { label: "Yemekleri Düzenle", onClick: () => handleEditMenuItems(menu) },
                            { label: "Bot yorumunu Düzenle", onClick: () => handleEditBotComment(menu) },
                            ...(menu.status !== "approved" ? [{ label: "Onayla", class: "u-color-text-success", onClick: () => approveMenu(menu.id) }] : []),
                            ...(menu.status !== "rejected" ? [{ label: "Reddet", class: "u-color-text-danger", onClick: () => rejectMenu(menu.id) }] : []),
                          ]}
                        />
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/each}
      </div>
    {/each}

    {#if displayLimit < totalMonthsCount}
      <div class="u-text-center u-mt-xl u-mb-xl">
        <button class="btn btn--secondary" onclick={() => (displayLimit += 10)}>
          Daha Fazla Göster
        </button>
      </div>
    {/if}
  {/if}
</div>

{#if isEditMenuModalOpen}
<Modal options={{ title: "Menü Yemeklerini & Ayarlarını Düzenle", iconHtml: icon('list', 24) }} onClose={() => (isEditMenuModalOpen = false)}>
  {#snippet children()}
    <div class="c-modal__form-group">
      <div class="u-flex u-items-center u-justify-between u-mb-xs">
        <span class="c-modal__label u-mb-0">Yemek Sıraları (Slotlar & Alternatifler)</span>
        <div class="u-flex u-items-center u-gap-xs">
          {#if editMenuTarget?.meal_type === 'breakfast' || !editMenuSlots.some((s) => s.primary)}
            <button
              type="button"
              class="btn btn--xs btn--ghost btn--squish"
              disabled={isApplyingTemplate}
              onclick={applyBreakfastTemplate}
              title="Standart KYK kahvaltı öğelerini (Yumurta, Peynir, Zeytin, Reçel, Ekmek) otomatik doldurur"
            >
              {isApplyingTemplate ? 'Yükleniyor...' : 'Kahvaltı Şablonu'}
            </button>
          {/if}
          <button type="button" class="btn btn--xs btn--secondary btn--squish" onclick={addNewSlot}>
            + Yeni Sıra Ekle
          </button>
        </div>
      </div>
      <div class="admin-slots-container">
        {#each editMenuSlots as slot, sIdx}
          <div class="admin-slot-card {activeSlotTarget.slotIndex === sIdx ? 'admin-slot-card--active' : ''}">
          <div class="admin-slot-header">
              <span class="admin-slot-title">Sıra {sIdx + 1}</span>
              <div class="u-flex u-items-center u-gap-xs">
                <select
                  class="c-modal__input u-text-xs u-py-2xs u-px-xs"
                  bind:value={slot.package_name}
                  title="Paket"
                >
                  <option value="NORMAL">NORMAL</option>
                  <option value="ÇÖLYAK MENÜSÜ">ÇÖLYAK MENÜSÜ</option>
                </select>
                {#if editMenuSlots.length > 1}
                  <button
                    type="button"
                    class="u-bg-transparent u-border-none u-color-muted u-cursor-pointer u-text-xs"
                    onclick={() => removeSlot(sIdx)}
                    title="Bu sırayı kaldır"
                  >
                    Sırayı Sil
                  </button>
                {/if}
              </div>
            </div>

            <div class="admin-slot-dishes">
              {#if slot.primary}
                <div class="admin-slot-dish-row">
                  <div class="admin-slot-dish-info">
                    <span class="u-font-bold">{slot.primary.name}</span>
                    <span class="admin-slot-dish-tag">Asıl</span>
                    {#if slot.primary.category}
                      <span class="admin-slot-dish-tag">{slot.primary.category}</span>
                    {/if}
                  </div>
                  <button
                    type="button"
                    class="u-bg-transparent u-border-none u-color-muted u-cursor-pointer u-font-bold"
                    onclick={() => removePrimaryFromSlot(sIdx)}
                    title="Yemeği kaldır"
                  >
                    ×
                  </button>
                </div>
              {:else}
                <div class="admin-slot-dish-row u-color-muted u-text-xs">
                  <span>Asıl yemek atanmadı</span>
                  <button
                    type="button"
                    class="btn btn--2xs btn--secondary btn--squish"
                    onclick={() => setTargetForSlot(sIdx, false)}
                  >
                    {activeSlotTarget.slotIndex === sIdx && !activeSlotTarget.isAlternative ? 'Seçiliyor...' : 'Asıl Ata'}
                  </button>
                </div>
              {/if}

              {#each slot.alternatives as alt}
                <div class="admin-slot-dish-row">
                  <div class="admin-slot-dish-info">
                    <span>{alt.name}</span>
                    <span class="admin-slot-dish-tag admin-slot-dish-tag--alt">Alternatif</span>
                    {#if alt.category}
                      <span class="admin-slot-dish-tag">{alt.category}</span>
                    {/if}
                  </div>
                  <button
                    type="button"
                    class="u-bg-transparent u-border-none u-color-muted u-cursor-pointer u-font-bold"
                    onclick={() => removeAlternativeFromSlot(sIdx, alt.id)}
                    title="Alternatifi kaldır"
                  >
                    ×
                  </button>
                </div>
              {/each}
            </div>

            <div class="admin-slot-actions">
              <button
                type="button"
                class="btn btn--2xs btn--ghost btn--squish"
                onclick={() => setTargetForSlot(sIdx, true)}
              >
                {activeSlotTarget.slotIndex === sIdx && activeSlotTarget.isAlternative ? 'Alternatif Aranıyor...' : '+ Alternatif Ekle'}
              </button>
              {#if slot.primary}
                <button
                  type="button"
                  class="btn btn--2xs btn--ghost btn--squish"
                  onclick={() => setTargetForSlot(sIdx, false)}
                >
                  {activeSlotTarget.slotIndex === sIdx && !activeSlotTarget.isAlternative ? 'Asıl Değiştiriliyor...' : 'Asıl Değiştir'}
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="c-modal__form-group">
      <div class="u-flex u-items-center u-justify-between u-mb-xs">
        <label for="dish-search" class="c-modal__label u-mb-0">
          Yemek Ekle:
          <span class="u-color-primary u-font-bold">
            Sıra {activeSlotTarget.slotIndex + 1} ({activeSlotTarget.isAlternative ? 'Alternatif' : 'Asıl Yemek'})
          </span>
        </label>
      </div>
      <div class="admin-search-wrapper">
        <input
          id="dish-search"
          type="text"
          class="c-modal__input"
          placeholder="Yemek ismi yaz..."
          bind:value={editMenuSearchQuery}
          oninput={handleDishSearchInput}
        />
      </div>
      {#if editMenuSearchResults.length > 0}
        <div class="admin-modal-search-results u-mt-xs">
          {#each editMenuSearchResults as d}
            <div class="dish-item-select">
              <div>
                <div class="u-text-sm u-font-bold">{d.name}</div>
                <div class="u-text-xs u-color-muted">ID: {d.id} | {d.category || 'Kategorisiz'}</div>
              </div>
              <button
                type="button"
                class="btn btn--xs btn--primary btn--squish add-dish-btn"
                onclick={() => addDishToSlot(d)}
              >
                Hedefe Ekle
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="c-modal__form-group">
      <label for="menu-notice" class="c-modal__label">Günün Özel Uyarısı</label>
      <input
        id="menu-notice"
        type="text"
        class="c-modal__input"
        placeholder="Örn: Bu menü il müdürlüğü onaylı olmayıp yurt yemekhanesi numune tepsisinden derlenmiştir."
        bind:value={editMenuNotice}
      />
      <span class="u-text-xs u-color-muted u-mt-2xs">Doluysa timeline gün kutusunda sarı uyarı kartı basılır.</span>
    </div>

    <div class="c-modal__form-group">
      <label for="menu-source-type" class="c-modal__label">Kaynak Türü</label>
      <input
        id="menu-source-type"
        type="text"
        class="c-modal__input"
        placeholder="kepce-admin, kepce-kullanici, yurtmenu.net..."
        bind:value={editMenuSourceType}
      />
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary btn--squish" onclick={() => (isEditMenuModalOpen = false)}>İptal</button>
    <button class="btn btn--primary btn--squish" onclick={saveMenuDishes}>Değişiklikleri Kaydet</button>
  {/snippet}
</Modal>
{/if}

{#if isEditBotModalOpen}
<Modal options={{ title: "Bot yorumunu Düzenle", iconHtml: icon('bot', 24) }} onClose={() => (isEditBotModalOpen = false)}>
  {#snippet children()}
    <div class="c-modal__form-group">
      <label for="bot-comment" class="c-modal__label">Kepçe Bot yorumu</label>
      <textarea id="bot-comment" class="c-modal__input" rows="5" placeholder="Kepçe Bot bu menü için ne desin?" bind:value={editBotCommentText}></textarea>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => (isEditBotModalOpen = false)}>İptal</button>
    <button class="btn btn--primary" onclick={saveBotComment}>Kaydet</button>
  {/snippet}
</Modal>
{/if}

{#if isCreateMenuModalOpen}
<Modal
  options={{ title: "Yeni Menü Oluştur", iconHtml: icon('calendar', 24) }}
  onClose={() => (isCreateMenuModalOpen = false)}
>
  {#snippet children()}
    <div class="c-modal__form-group">
      <label for="new-menu-city" class="c-modal__label">Şehir</label>
      <select
        id="new-menu-city"
        class="c-modal__input"
        bind:value={newMenuCityId}
      >
        {#each cities as c}
          <option value={c.id}>{c.name}</option>
        {/each}
      </select>
    </div>

    <div class="c-modal__form-group">
      <label for="new-menu-date" class="c-modal__label">Tarih</label>
      <input
        id="new-menu-date"
        type="date"
        class="c-modal__input"
        bind:value={newMenuDate}
      />
    </div>

    <div class="c-modal__form-group">
      <label for="new-menu-meal" class="c-modal__label">Öğün</label>
      <select
        id="new-menu-meal"
        class="c-modal__input"
        bind:value={newMenuMealType}
      >
        <option value="breakfast">Kahvaltı</option>
        <option value="dinner">Akşam Yemeği</option>
      </select>
    </div>

    <div class="c-modal__form-group">
      <label for="new-menu-source" class="c-modal__label">Kaynak Türü</label>
      <input
        id="new-menu-source"
        type="text"
        class="c-modal__input"
        bind:value={newMenuSourceType}
        placeholder="kepce-admin"
      />
    </div>

    <div class="c-modal__form-group">
      <label for="new-menu-notice" class="c-modal__label">Günün Uyarısı / Not (Opsiyonel)</label>
      <input
        id="new-menu-notice"
        type="text"
        class="c-modal__input"
        bind:value={newMenuNotice}
        placeholder="Örn: Hafta sonu nöbetçi yurt"
      />
    </div>
  {/snippet}
  {#snippet footer()}
    <button
      type="button"
      class="btn btn--secondary btn--squish"
      disabled={isCreatingMenu}
      onclick={() => (isCreateMenuModalOpen = false)}
    >
      İptal
    </button>
    <button
      type="button"
      class="btn btn--primary btn--squish"
      disabled={isCreatingMenu}
      onclick={submitCreateMenu}
    >
      {isCreatingMenu ? "Oluşturuluyor..." : "Menü Oluştur"}
    </button>
  {/snippet}
</Modal>
{/if}
