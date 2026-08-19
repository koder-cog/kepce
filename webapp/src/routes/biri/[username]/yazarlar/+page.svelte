<script>
  import { onMount } from "svelte";
  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Loader from "@/components/ui/Loader.svelte";
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
</script>

{#if contentLoading}
  <div class="profile-grid-loader">
    <Loader size={48} />
  </div>
{:else}
  {#if !dashboardStats?.favorite_authors?.length}
    <EmptyState
      iconName="user"
      title="Favori Yazar Yok"
      desc={`@${username} henüz başka bir kullanıcının yorumunu beğenmemiş.`}
    />
  {:else}
    <div class="profile-authors-grid">
      {#each dashboardStats.favorite_authors as acc, idx}
        <div class="contributor-card">
          <div class="contributor-card__left">
            <div class="contributor-card__rank {idx < 3 ? `contributor-card__rank--${idx + 1}` : ''}">
              {#if idx === 0 || idx === 1 || idx === 2}
                {@html icon("trophy", 16)}
              {:else}
                {idx + 1}
              {/if}
            </div>
            <a href="/biri/{acc.username}" data-link class="contributor-card__avatar" title="@{acc.username}">
              {#if acc.avatar_url}
                <img
                  src={api.getAvatarUrl(acc.avatar_url)}
                  alt="@{acc.username}"
                  onerror={(e) => {
                    e.target.outerHTML = icon("avatarEmpty", 40);
                  }}
                />
              {:else}
                {@html icon("avatarEmpty", 40)}
              {/if}
            </a>
            <a href="/biri/{acc.username}" data-link class="contributor-card__name">
              @{acc.username || "Anonim"}
            </a>
          </div>
          <div class="contributor-card__value">
            <span>{acc.favorite_count} Beğeni</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}
