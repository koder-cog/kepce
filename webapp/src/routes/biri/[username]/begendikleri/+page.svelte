<script>
  import { globalState, authActions } from "@/state.svelte.js";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import { timeAgo } from "@/utils/date.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Loader from "@/components/ui/Loader.svelte";
  import { showToast } from "@/components/ui/toast.js";
  import { page } from "$app/stores";

  import Pagination from "@/components/ui/Pagination.svelte";

  let username = $derived($page.params.username);
  let paginationMode = $derived(globalState.paginationMode || "sayfali");
  let urlPage = $derived(parseInt($page.url.searchParams.get("sayfa") || "1", 10) || 1);

  let contentLoading = $state(true);
  let dashboardStats = $state(null);
  let currentPage = $state(1);
  let limit = 20;

  let allFavorites = $derived(dashboardStats?.favorite_comments || []);
  let totalItems = $derived(allFavorites.length);
  let totalPages = $derived(Math.ceil(totalItems / limit) || 1);

  let paginatedFavorites = $derived.by(() => {
    if (paginationMode === "sayfali") {
      const start = (currentPage - 1) * limit;
      return allFavorites.slice(start, start + limit);
    }
    return allFavorites;
  });

  $effect(() => {
    if (username) loadDashboardStats();
  });

  $effect(() => {
    currentPage = Math.max(1, Math.min(urlPage, totalPages));
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

  function handlePageChange(newPage) {
    currentPage = newPage;
    const url = new URL(window.location.href);
    if (newPage > 1) {
      url.searchParams.set("sayfa", String(newPage));
    } else {
      url.searchParams.delete("sayfa");
    }
    goto(url.pathname + url.search, { keepFocus: true, noScroll: false });
  }

  function handleCommentAction(action, comment) {
    const commentId = comment.hash;
    const menuId = comment.menu?.id || comment.menu_id || "";

    if (action === "reply") {
      const shortId = commentId.substring(0, 7);
      goto(`/menu/${menuId}/${shortId}`);
    }
  }
</script>

{#if contentLoading}
  <div class="profile-grid-loader">
    <Loader size={48} />
  </div>
{:else if !allFavorites.length}
  <EmptyState
    iconName="voteUpFilled"
    title="Henüz Beğeni Yok"
    desc={`@${username} henüz birilerinin yorumlarını beğenmemiş.`}
  />
{:else}
  {#if paginationMode === "sayfali" && totalPages > 1}
    <div class="profile-comments-header u-mb-md u-flex u-flex-justify-end">
      <Pagination
        compact={true}
        page={currentPage}
        {totalPages}
        {totalItems}
        onPageChange={handlePageChange}
      />
    </div>
  {/if}

  <div class="profile-activity-list">
    {#each paginatedFavorites as c, idx}
      {@const commentKey = c.id || c.hash || `comment-${idx}`}
      {@const menuId = c.menu_id || c.menu?.id || ""}
      {@const threadTarget = c.id ? c.id.substring(0, 7) : (c.hash || "")}
      {@const commentHref = menuId ? `/menu/${menuId}/${threadTarget}` : `/menu?thread=${threadTarget}`}
      {@const score = c.reaction_summary
        ? c.reaction_summary.up - c.reaction_summary.down
        : 0}
      <div class="comment-card">
        <div class="comment-card__inner">
          <div class="comment-card__avatar-col">
            <a
              href="/biri/{c.user?.nickname}"
              class="comment-card__avatar"
              data-link
            >
              {#if c.user?.avatar_url}
                <img
                  src={api.getAvatarUrl(c.user.avatar_url)}
                  alt=""
                  onerror={(e) =>
                    (e.target.outerHTML = icon("avatarEmpty", 40))}
                />
              {:else}
                {@html icon("avatarEmpty", 40)}
              {/if}
            </a>
          </div>
          <div class="comment-card__main-col">
            <div class="comment-card__header">
              <div class="comment-card__meta">
                <span class="comment-card__action-text">
                  <a href="/biri/{c.user?.nickname}" data-link
                    ><strong>@{c.user?.nickname}</strong></a
                  >
                </span>
                {#if c.is_tabldot || (c.tags && c.tags.length > 0) || (c.tag_ids && c.tag_ids.length > 0)}
                  <span
                    class="comment-node__badge--structured"
                    data-tooltip="Tabldot / Yapılandırılmış yorum"
                    >{@html icon("puzzle", 12)}</span
                  >
                {/if}
                <span class="comment-card__dot">·</span>
                <a
                  href={commentHref}
                  data-link
                  class="comment-card__date">{timeAgo(c.created_at)}</a
                >
              </div>
            </div>
            <div class="comment-card__body">
              <p class="comment-card__text">{c.comment}</p>
            </div>
          </div>
        </div>
      </div>
    {/each}
  </div>

  {#if paginationMode === "sayfali" && totalPages > 1}
    <Pagination
      page={currentPage}
      {totalPages}
      {totalItems}
      onPageChange={handlePageChange}
    />
  {/if}
{/if}
