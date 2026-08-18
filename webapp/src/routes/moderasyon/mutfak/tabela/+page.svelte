<script>
  import "@/styles/pages/_menu-table.css";
  import { onMount } from "svelte";
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
  let editMenuSelectedDishes = $state([]);
  let editMenuSearchQuery = $state("");
  let editMenuSearchResults = $state([]);
  let searchTimeout;
  let currentLoadToken = 0;
  let currentSearchToken = 0;

  let isEditBotModalOpen = $state(false);
  let editBotTarget = $state(null);
  let editBotCommentText = $state("");

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
    editMenuSearchQuery = "";
    editMenuSearchResults = [];
    isEditMenuModalOpen = true;
    try {
      const dishes = await api.getMenuDishIds(menu.id);
      editMenuSelectedDishes = dishes;
    } catch (err) {
      editMenuSelectedDishes = [];
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

  function addDishToMenu(d) {
    if (!editMenuSelectedDishes.find((dish) => dish.id === d.id)) {
      editMenuSelectedDishes.push({ id: d.id, name: d.name });
      showToast("Yemek eklendi");
    }
  }

  function removeDishFromMenu(id) {
    editMenuSelectedDishes = editMenuSelectedDishes.filter((d) => d.id !== id);
  }

  async function saveMenuDishes() {
    try {
      const dishIds = editMenuSelectedDishes.map((d) => d.id);
      await api.updateMenuItems(editMenuTarget.id, dishIds);
      showToast("Menü yemekleri güncellendi!");
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
          <div class="u-mb-lg">
            <h4 class="u-mb-md u-color-accent-primary">{monthName}</h4>
            <div class="admin-table-wrapper">
              <table class="admin-table admin-table--hybrid">
                <thead>
                  <tr>
                    <th>Tarih</th>
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
                        <div class="admin-table-cell--primary">
                          {(() => {
                            const [, m, d] = menu.date.split("-");
                            return `${parseInt(d, 10)} ${monthsTR[parseInt(m, 10) - 1]}`;
                          })()}
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
<Modal options={{ title: "Menü Yemeklerini Düzenle", iconHtml: icon('list', 24) }} onClose={() => (isEditMenuModalOpen = false)}>
  {#snippet children()}
    <div class="c-modal__form-group">
      <div class="c-modal__label">Mevcut yemekler</div>
      <div class="admin-selected-dishes-box">
        {#if editMenuSelectedDishes.length === 0}
          <p class="u-color-muted u-text-xs">Henüz yemek eklenmedi.</p>
        {:else}
          {#each editMenuSelectedDishes as dish}
            <div class="chip active u-m-xs">
              {dish.name}
              <button class="remove-dish u-cursor-pointer u-ml-xs u-bg-transparent u-border-none" onclick={() => removeDishFromMenu(dish.id)}>×</button>
            </div>
          {/each}
        {/if}
      </div>
    </div>
    <div class="c-modal__form-group">
      <label for="dish-search" class="c-modal__label">Yemek ekle (arama)</label>
      <div class="admin-search-wrapper">
        <input id="dish-search" type="text" class="c-modal__input" placeholder="Yemek ismi yaz..." bind:value={editMenuSearchQuery} oninput={handleDishSearchInput}>
      </div>
      <div class="admin-modal-search-results">
        {#each editMenuSearchResults as d}
          <div class="dish-item-select">
            <div>
              <div class="u-text-sm u-font-bold">{d.name}</div>
              <div class="u-text-xs u-color-muted">ID: {d.id} | {d.category}</div>
            </div>
            <button class="btn btn--xs btn--primary add-dish-btn" onclick={() => addDishToMenu(d)}>Ekle</button>
          </div>
        {/each}
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={() => (isEditMenuModalOpen = false)}>İptal</button>
    <button class="btn btn--primary" onclick={saveMenuDishes}>Değişiklikleri kaydet</button>
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
