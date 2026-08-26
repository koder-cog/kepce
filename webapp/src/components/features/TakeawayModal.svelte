<script>
  import { globalState, authActions } from "../../state.svelte.js";
  import Modal from "../ui/Modal.svelte";
  import { icon } from "../ui/icons.js";
  import { api } from "../../api/index.js";
  import { showToast } from "../ui/toast.js";
  import { groupItems, normalizeItems } from "../../utils/menu.js";
  import { sanitizeText } from "../../utils/sanitize.js";

  let { takeawayMenu, takeawayId, takeawayLabel, currentCity, onClose } =
    $props();

  let controller = {};
  let pendingFavorites = new Set();

  // Normalize items using standard menu normalizer
  let normalizedItems = $derived.by(() => {
    if (!takeawayMenu) return [];
    const items = normalizeItems(takeawayMenu);
    // Eğer tüm menü sadece 1 satırdan ibaretse ve o da jenerik bir "Al Götür Menü X" başlığıysa boş duruma düşür
    if (items.length === 1) {
      const singleName = (items[0].name || "").trim().toLowerCase();
      if (/^al[- ]?g[öo]t[üu]r\s*(men[üu])?\s*\d*$/i.test(singleName)) {
        return [];
      }
    }
    return items;
  });

  let groupedItems = $derived(groupItems(normalizedItems));

  let noteLines = $derived.by(() => {
    const isIstanbul = currentCity === "istanbul";
    const isBreakfast =
      takeawayMenu &&
      (takeawayMenu.meal_type === "breakfast" ||
        (takeawayLabel && takeawayLabel.toLowerCase().includes("kahvaltı")));
    let notes = [];
    if (isIstanbul && isBreakfast) {
      notes.push("Al Götür Menü her gün saat 10:30'a kadar verilmektedir.");
      notes.push(
        "Al Götür Menü için alternatiflerden ikisi serviste bulundurularak, biri kahvaltı yerine verilir.",
      );
    }
    notes.push(
      "Verilen bilgilerin kesinliği hakkında Kepçe herhangi bir garanti sunmamaktadır.",
    );
    return notes;
  });

  let modalOptions = $derived({
    title:
      takeawayLabel || (takeawayMenu && takeawayMenu.name) || "Al Götür Menüsü",
    iconColor: "primary",
  });

  function isFav(dish) {
    if (!dish || typeof dish.id !== "number") return false;
    return (
      (globalState.favorites && globalState.favorites.includes(dish.id)) ||
      !!dish.my_favorite
    );
  }

  async function toggleFavorite(dish) {
    if (!globalState?.user) {
      authActions.triggerLogin();
      return;
    }
    const dishId = dish.id;
    if (typeof dishId !== "number") return;

    if (pendingFavorites.has(dishId)) return;
    pendingFavorites.add(dishId);

    const currentlyFav = isFav(dish);

    if (currentlyFav) {
      globalState.favorites = (globalState.favorites || []).filter(
        (id) => id !== dishId,
      );
      dish.my_favorite = false;
    } else {
      if (!globalState.favorites.includes(dishId)) {
        globalState.favorites = [...(globalState.favorites || []), dishId];
      }
      dish.my_favorite = true;
    }

    try {
      await api.toggleFavorite(dishId);
    } catch (err) {
      showToast(err.message, "error");
      if (currentlyFav) {
        if (!globalState.favorites.includes(dishId)) {
          globalState.favorites = [...(globalState.favorites || []), dishId];
        }
        dish.my_favorite = true;
      } else {
        globalState.favorites = (globalState.favorites || []).filter(
          (id) => id !== dishId,
        );
        dish.my_favorite = false;
      }
    } finally {
      pendingFavorites.delete(dishId);
    }
  }
</script>

<Modal options={modalOptions} {onClose} {controller}>
  {#if groupedItems.length === 0}
    <p class="c-takeaway-modal__empty">
      Bu al götür menüsünün içeriği henüz detaylandırılmamıştır. İlerleyen
      zamanlarda elimize geçerse sisteme eklenecektir.
    </p>
  {:else}
    <div class="meal-card__items">
      {#each groupedItems as item}
        {@const dishes =
          item.dishes && item.dishes.length > 0
            ? item.dishes
            : [{ id: item.id, name: item.name }]}
        {@const isAlternative = dishes.length > 1}

        <div
          class="meal-card__item-row {isAlternative
            ? 'meal-card__item-row--alternative'
            : ''}"
        >
          {#each dishes as dish, idx}
            {#if idx > 0}
              <div class="meal-card__dish-separator--yada">
                <span class="meal-card__dish-separator-text">- ya da -</span>
              </div>
            {/if}
            <div class="meal-card__item">
              <div class="meal-card__dish-part" data-dish-id={dish.id}>
                <div class="meal-card__dish-info-wrapper">
                  <span class="meal-card__dish-name"
                    >{sanitizeText(dish.name)}</span
                  >
                  {#if dish.weight || dish.calories}
                    <div class="text-xs color-muted weight-info">
                      {#if dish.weight}{sanitizeText(dish.weight)}{/if}
                      {#if dish.weight && dish.calories}
                        &bull;
                      {/if}
                      {#if dish.calories}{dish.calories} kcal{/if}
                    </div>
                  {/if}
                </div>

                {#if typeof dish.id === "number" || dish.is_vegan || dish.is_vegetarian || dish.is_celiac}
                  <div class="meal-card__dish-actions">
                    {#if dish.is_celiac}
                      <div class="meal-card__diet-icon" data-tooltip="Çölyak">
                        {@html icon("wheat", 18)}
                      </div>
                    {/if}
                    {#if dish.is_vegan}
                      <div class="meal-card__diet-icon" data-tooltip="Vegan">
                        {@html icon("check", 18)}
                      </div>
                    {:else if dish.is_vegetarian}
                      <div
                        class="meal-card__diet-icon"
                        data-tooltip="Vejetaryen"
                      >
                        {@html icon("check", 18)}
                      </div>
                    {/if}
                    {#if typeof dish.id === "number"}
                      {@const activeFav = isFav(dish)}
                      <button
                        class="meal-card__star-btn {activeFav ? 'active' : ''}"
                        aria-label={activeFav
                          ? "Favorilerden çıkar"
                          : "Favorilere ekle"}
                        title={activeFav
                          ? "Favorilerden çıkar"
                          : "Favorilere ekle"}
                        onclick={(e) => {
                          e.stopPropagation();
                          toggleFavorite(dish);
                        }}
                      >
                        {@html icon(activeFav ? "starFilled" : "star", 18)}
                      </button>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/each}
    </div>

    {#if noteLines.length > 0}
      <div class="c-takeaway-modal__note">
        {#each noteLines as line}
          <p class="c-takeaway-modal__note-line">{line}</p>
        {/each}
      </div>
    {/if}
  {/if}

  {#snippet footer()}
    <button class="btn btn--primary" onclick={() => controller?.close()}
      >Kapat</button
    >
  {/snippet}
</Modal>
