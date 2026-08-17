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

  function handleCommentAction(action, comment) {
    const commentId = comment.hash;
    const menuId = comment.menu?.id || comment.menu_id || "";

    if (action === "reply") {
      const shortId = commentId.substring(0, 7);
      goto(`/yorumlar/${menuId}/${shortId}`);
    }
  }
</script>

{#if contentLoading}
  <div class="profile-grid-loader">
    <Loader size={48} />
  </div>
{:else if !dashboardStats?.favorite_comments?.length}
  <EmptyState
    iconName="voteUpFilled"
    title="Henüz Beğeni Yok"
    desc={`@${username} henüz birilerinin yorumlarını beğenmemiş.`}
  />
{:else}
  <div class="profile-activity-list">
    {#each dashboardStats.favorite_comments as c}
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
                    (e.target.outerHTML = icon("avatarEmpty", 32))}
                />
              {:else}
                {@html icon("avatarEmpty", 32)}
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
                  href="/yorumlar/{c.menu?.id || c.menu_id || ''}/{c.hash}"
                  data-link
                  class="comment-card__date">{timeAgo(c.created_at)}</a
                >
              </div>
            </div>
            <div class="comment-card__body">
              <p class="comment-card__text">{c.comment}</p>
            </div>
            <div class="comment-node__actions">
              <div class="comment-node__vote">
                <span
                  class="vote-count {score > 0
                    ? 'positive'
                    : score < 0
                      ? 'negative'
                      : ''}">{score}</span
                >
              </div>
              <button
                class="action-btn btn--squish"
                title="Yanıtla"
                onclick={() => handleCommentAction("reply", c)}
              >
                {@html icon("chat", 14)}
              </button>
            </div>
          </div>
        </div>
      </div>
    {/each}
  </div>
{/if}
