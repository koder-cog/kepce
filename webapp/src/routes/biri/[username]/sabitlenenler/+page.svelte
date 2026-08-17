<script>
  import { globalState } from "@/state.svelte.js";
  import { onMount } from "svelte";
  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Loader from "@/components/ui/Loader.svelte";
  import { showToast } from "@/components/ui/toast.js";
  import { page } from "$app/stores";

  let username = $derived($page.params.username);
  let contentLoading = $state(true);
  let dashboardStats = $state(null);

  $effect(() => {
    if (username) loadDashboardStats();
  });

  async function loadDashboardStats() {
    contentLoading = true;
    try {
      dashboardStats = await api.getProfileDashboardStats(username);
    } catch (err) {
      console.error("Dashboard stats error:", err);
    } finally {
      contentLoading = false;
    }
  }

  let allMeals = $derived.by(() => {
    if (!dashboardStats) return [];
    const pinned = dashboardStats.pinned_meals || [];
    const favs = dashboardStats.favorite_meals || [];
    const map = new Map();
    [...pinned, ...favs].forEach((m) => {
      const id = m.dish_id || m.id;
      if (id && !map.has(id)) {
        map.set(id, { ...m, dish_id: id });
      }
    });
    return Array.from(map.values());
  });

  async function handleUnpin(dishId) {
    try {
      await Promise.allSettled([
        api.togglePinned(dishId),
        api.toggleFavorite(dishId),
      ]);
      if (dashboardStats) {
        if (dashboardStats.pinned_meals) {
          dashboardStats.pinned_meals = dashboardStats.pinned_meals.filter(
            (d) => (d.dish_id || d.id) !== dishId,
          );
        }
        if (dashboardStats.favorite_meals) {
          dashboardStats.favorite_meals = dashboardStats.favorite_meals.filter(
            (d) => (d.dish_id || d.id) !== dishId,
          );
        }
      }
      if (globalState.favorites) {
        globalState.favorites = globalState.favorites.filter(
          (id) => id !== dishId,
        );
      }
      showToast("Sabitlenenlerden kaldırıldı");
    } catch (err) {
      showToast(err.message, "error");
    }
  }

  let isOwner = $derived(globalState?.user?.username === username);
</script>

{#if contentLoading}
  <div class="profile-grid-loader">
    <Loader size={48} />
  </div>
{:else if allMeals.length > 0}
  <div class="profile-dish-grid">
    {#each allMeals as dish}
      <div class="profile-dish-card">
        <div class="profile-dish-card__main">
          <div class="profile-dish-card__icon">
            {@html icon("starFilled", 18)}
          </div>
          <span class="profile-dish-card__name">{dish.name}</span>
        </div>
        {#if isOwner}
          <button
            class="profile-dish-unpin-btn btn--squish"
            title="Kaldır"
            onclick={() => handleUnpin(dish.dish_id)}
          >
            {@html icon("close", 14)}
          </button>
        {/if}
      </div>
    {/each}
  </div>
{:else}
  <EmptyState
    iconName="star"
    title="Sabitlenen Yemek Yok"
    desc={`@${username} henüz bir yemeği beğenmemiş.`}
  />
{/if}
