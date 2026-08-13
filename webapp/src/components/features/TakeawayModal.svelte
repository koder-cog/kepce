<script>
  import { globalState, authActions } from '../../state.svelte.js';

  import Modal from '../ui/Modal.svelte';
  import { icon } from '../ui/icons.js';
  import { api } from '../../api/index.js';
  import { showToast } from '../ui/toast.js';
  import { groupItems } from '../../utils/menu.js';

  let { takeawayMenu, takeawayId, takeawayLabel, currentCity, onClose } = $props();

  let controller = {};
  
  let pendingFavorites = new Set();

  // Derived logic for items
  let menuItemsData = $derived.by(() => {
    let items = [];
    if (takeawayMenu && takeawayMenu.items && takeawayMenu.items.length > 0) {
        items = takeawayMenu.items.map(i => ({
            sort_order: i.order_index,
            name: i.master_data ? i.master_data.name : i.raw_name,
            id: i.master_data ? i.master_data.dish_id : null,
            is_alternative: i.is_alternative,
            dishes: [{
                id: i.master_data ? i.master_data.dish_id : null,
                name: i.master_data ? i.master_data.name : i.raw_name,
                is_vegan: i.master_data ? i.master_data.is_vegan : false,
                is_vegetarian: i.master_data ? i.master_data.is_vegetarian : false,
                is_celiac: i.master_data ? i.master_data.is_celiac : false,
                is_alternative: i.is_alternative
            }]
        }));
    } else if (takeawayMenu && takeawayMenu.slots && takeawayMenu.slots.length > 0) {
        items = takeawayMenu.slots.map(s => ({ dishes: s.alternatives || [] }));
    } else if (takeawayMenu && takeawayMenu.dishes && takeawayMenu.dishes.length > 0) {
        items = takeawayMenu.dishes.map(d => ({ dishes: [d] }));
    }
    
    // deeply clone items so reactivity works when we mutate dish.my_favorite
    return {
        items: JSON.parse(JSON.stringify(items)), 
        isMock: false
    };
  });

  let displayItems = $derived(menuItemsData.items);
  let isMocked = $derived(menuItemsData.isMock);

  let groupedItems = $derived(groupItems(displayItems));

  let noteLines = $derived.by(() => {
    const isIstanbul = currentCity === 'istanbul';
    const isBreakfast = takeawayMenu && takeawayMenu.meal_type === 'breakfast';
    let notes = [];
    if (isIstanbul && isBreakfast) {
        notes.push("Al Götür Menü her gün saat 10:30'a kadar verilmektedir.");
        notes.push("Al Götür Menü için alternatiflerden ikisi serviste bulundurularak, biri kahvaltı yerine verilir.");
    }
    notes.push("Verilen bilgilerin kesinliği hakkında Kepçe herhangi bir garanti sunmamaktadır.");
    return notes;
  });

  let modalOptions = $derived({
    title: takeawayLabel,
    iconColor: 'primary',
  });

  function isFav(dish) {
    if (!dish || typeof dish.id !== 'number') return false;
    return (globalState.favorites && globalState.favorites.includes(dish.id)) || !!dish.my_favorite;
  }

  async function toggleFavorite(dish) {
    if (!globalState?.user) {
        authActions.triggerLogin();
        return;
    }
    const dishId = dish.id;
    if (isMocked || typeof dishId !== 'number') return;
    
    if (pendingFavorites.has(dishId)) return;
    pendingFavorites.add(dishId);
    
    const currentlyFav = isFav(dish);

    if (currentlyFav) {
        globalState.favorites = (globalState.favorites || []).filter((id) => id !== dishId);
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
        showToast(err.message, 'error');
        if (currentlyFav) {
            if (!globalState.favorites.includes(dishId)) {
                globalState.favorites = [...(globalState.favorites || []), dishId];
            }
            dish.my_favorite = true;
        } else {
            globalState.favorites = (globalState.favorites || []).filter((id) => id !== dishId);
            dish.my_favorite = false;
        }
    } finally {
        pendingFavorites.delete(dishId);
    }
  }
</script>

<Modal options={modalOptions} {onClose} {controller}>
        {#if displayItems.length === 0}
            <p class="c-takeaway-modal__empty">Bu al götür menüsünün içeriği henüz detaylandırılmamıştır. İlerleyen zamanlarda sisteme eklenecektir.</p>
        {:else}
            <div class="meal-card__items">
                {#each groupedItems as item}
                    {@const dishes = item.dishes && item.dishes.length > 0 ? item.dishes : [{ id: `raw-${item.id}`, name: item.name }]}
                    <div class="meal-card__item-row">
                        <div class="meal-card__item">
                            {#each dishes as dish, idx}
                                <div class="meal-card__dish-part">
                                    {#if idx > 0}
                                        <span class="meal-card__dish-separator">/</span>
                                    {/if}
                                    <span class="meal-card__dish-name">{dish.name}</span>
                                    
                                    {#if typeof dish.id === 'number' && !isMocked}
                                        {@const activeFav = isFav(dish)}
                                        <div class="meal-card__dish-actions">
                                            <button
                                                class="meal-card__star-btn {activeFav ? 'active' : ''}"
                                                aria-label={activeFav ? 'Favorilerden çıkar' : 'Favorilere ekle'}
                                                title={activeFav ? 'Favorilerden çıkar' : 'Favorilere ekle'}
                                                onclick={(e) => { e.stopPropagation(); toggleFavorite(dish); }}
                                            >
                                                {@html icon(activeFav ? 'starFilled' : 'star', 18)}
                                            </button>
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
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
        <button class="btn btn--primary" onclick={() => controller?.close()}>Kapat</button>
    {/snippet}
</Modal>
