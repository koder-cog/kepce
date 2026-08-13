<script>
  import { globalState, authActions } from "@/state.svelte.js";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import { timeAgo } from "@/utils/date.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { createModal } from "@/components/features/modal.js";
  import { showToast } from "@/components/ui/toast.js";
  import Loader from "@/components/ui/Loader.svelte";
  import { getCommentContextHtml } from "@/utils/turkish.js";
  import { page } from "$app/stores";

  let username = $derived($page.params.username);
  let contentLoading = $state(true);
  let contentError = $state(null);
  let commentsData = $state([]);
  let commentsSort = $state("new");
  let openMenuId = $state(null);
  let currentTabToken = 0;

  // Pagination
  let limit = 20;
  let offset = $state(0);
  let hasMore = $state(false);
  let isLoadingMore = $state(false);

  $effect(() => {
    if (username) {
        offset = 0;
        loadTabContent();
    }
  });

  async function loadTabContent(isLoadMore = false) {
    if (!isLoadMore) {
        contentLoading = true;
        offset = 0;
        commentsData = [];
    } else {
        isLoadingMore = true;
    }
    contentError = null;
    const token = ++currentTabToken;

    try {
      const res = await api.getUserComments(username, commentsSort, limit, offset);
      if (currentTabToken !== token) return;
      
      const newData = Array.isArray(res) ? res : (res?.items || res?.data || []);
      if (isLoadMore) {
          commentsData = [...commentsData, ...newData];
      } else {
          commentsData = newData;
      }
      
      const total = res?.total_items ?? res?.total ?? (Array.isArray(res) ? res.length : 0);
      hasMore = commentsData.length < total && newData.length > 0;
      
    } catch (err) {
      if (currentTabToken !== token) return;
      contentError = err.message;
    } finally {
      if (currentTabToken === token) {
          contentLoading = false;
          isLoadingMore = false;
      }
    }
  }

  function loadMore() {
      if (isLoadingMore || !hasMore) return;
      offset += limit;
      loadTabContent(true);
  }

  function handleSortChange() {
    offset = 0;
    loadTabContent();
  }

  async function reactToComment(hash, type, commentObj) {
    if (!globalState?.user) {
      authActions.triggerLogin();
      return;
    }
    try {
      await api.voteComment(hash, type);
      if (!commentObj.reaction_summary) {
        commentObj.reaction_summary = { up: 0, down: 0 };
      }
      if (type === "up") commentObj.reaction_summary.up++;
      else commentObj.reaction_summary.down++;
      showToast("Reaksiyon kaydedildi.", "success");
    } catch (e) {
      showToast(e.message, "error");
    }
  }

  function handleCommentAction(action, comment) {
    const commentId = comment.hash;
    const menuId = comment.menu?.id || comment.menu_id || "";
    const requiresLogin = ["reply", "delete", "report"].includes(action);
    if (requiresLogin && !globalState?.user) {
      authActions.triggerLogin();
      return;
    }

    if (action === "reply") {
      const shortId = commentId.substring(0, 7);
      goto(`/yorumlar/${menuId}/${shortId}`);
    } else if (action === "share") {
      const shortId = commentId.substring(0, 7);
      const url = `${window.location.origin}/yorumlar/${menuId}/${shortId}`;
      navigator.clipboard
        .writeText(url)
        .then(() => showToast("Yorum linki kopyalandı!"));
    } else if (action === "report") {
      createModal({
        title: "Yorumu Şikayet et",
        contentHtml: "<p>Bu yorumun topluluk kurallarını ihlal ettiğini mi düşünüyorsun?</p>",
        buttons: [
          { label: "Vazgeç", variant: "secondary" },
          {
            label: "Şikayet et",
            variant: "danger",
            onClick: async () => {
              try {
                await api.reportComment(commentId);
                showToast("Şikayetin alındı.");
              } catch (e) {
                showToast(e.message, "error");
              }
            },
          },
        ],
      });
    } else if (action === "delete") {
      createModal({
        title: "Yorumu Sil",
        contentHtml: "<p>Bu yorumu silmek istediğine emin misin?</p>",
        buttons: [
          { label: "Vazgeç", variant: "secondary" },
          {
            label: "Sil",
            variant: "danger",
            onClick: async () => {
              try {
                await api.deleteComment(commentId);
                showToast("Yorum silindi.");
                loadTabContent();
              } catch (e) {
                showToast(e.message, "error");
              }
            },
          },
        ],
      });
    }
  }
</script>

{#if contentLoading}
  <div class="profile-grid-loader">
    <Loader size={48} />
  </div>
{:else if contentError}
  <div class="profile-activity-list">
    <EmptyState iconName="info" title="Hata Oluştu" desc={contentError} />
  </div>
{:else}
  <div class="profile-comments-header">
    <div class="profile-comments-sort">
      <Dropdown 
        options={[
          { label: "En Yeni", value: "new" },
          { label: "En Çok Beğenilen", value: "top" }
        ]}
        bind:value={commentsSort}
        onChange={handleSortChange}
      />
    </div>
  </div>
  {#if commentsData.length === 0}
    <div class="profile-activity-list">
      <EmptyState
        iconName="chat"
        title="Yorum Yok"
        desc={`@${username} henüz yorum yazmamış.`}
      />
    </div>
  {:else}
    <div class="profile-activity-list">
      {#each commentsData as c, idx}
        {@const commentKey = c.hash || c.id || `comment-${idx}`}
        {@const isOwnComment = globalState?.user?.id === c.user?.id}
        {@const isAdmin = globalState?.user?.role === "admin"}
        {@const score = c.reaction_summary ? c.reaction_summary.up - c.reaction_summary.down : 0}
        <div class="comment-card {openMenuId === commentKey ? 'has-open-menu' : ''}" style="--stagger-idx: {idx}">
          <div class="comment-card__main-col">
            <div class="comment-card__header">
              <div class="comment-card__meta">
                <span class="comment-card__action-text">{@html getCommentContextHtml(c)}</span>
                <span class="comment-card__dot">·</span>
                <a href="/yorumlar/{c.menu?.id || c.menu_id || ''}/{c.hash}" data-link class="comment-card__date">{timeAgo(c.created_at)}</a>
              </div>
            </div>
            <div class="comment-card__body">
              <p class="comment-card__text">{c.comment}</p>
            </div>
            <div class="comment-node__actions">
              <div class="comment-node__vote">
                <button
                  class="vote-btn btn--squish {c.reaction_summary && c.reaction_summary.my_vote === 'up' ? 'is-active' : ''}"
                  data-vote="up"
                  title="Beğen"
                  onclick={() => reactToComment(c.hash, "up", c)}
                >
                  {@html icon(c.reaction_summary && c.reaction_summary.my_vote === "up" ? "voteUpFilled" : "voteUp", 16)}
                </button>
                <span class="vote-count {score > 0 ? 'positive' : score < 0 ? 'negative' : ''}">{score}</span>
                <button
                  class="vote-btn btn--squish {c.reaction_summary && c.reaction_summary.my_vote === 'down' ? 'is-active' : ''}"
                  data-vote="down"
                  title="Beğenme"
                  onclick={() => reactToComment(c.hash, "down", c)}
                >
                  {@html icon(c.reaction_summary && c.reaction_summary.my_vote === "down" ? "voteDownFilled" : "voteDown", 16)}
                </button>
              </div>
              <button class="action-btn btn--squish" title="Yanıtla" onclick={() => handleCommentAction("reply", c)}>
                {@html icon("chat", 14)}
                <span class="action-btn__text">Yanıtla</span>
              </button>
              <button class="action-btn btn--squish" title="Paylaş" onclick={() => handleCommentAction("share", c)}>
                {@html icon("share", 14)}
                <span class="action-btn__text">Paylaş</span>
              </button>

              <div class="comment-dropdown-wrapper">
                <button class="action-btn btn--squish" title="Daha fazla seçenek" onclick={(e) => { e.stopPropagation(); openMenuId = openMenuId === commentKey ? null : commentKey; }}>
                  {@html icon("more", 16)}
                </button>
                {#if openMenuId === commentKey}
                  <div class="c-menu c-menu--open comment-dropdown-menu">
                    {#if !isOwnComment}
                      <button class="c-menu__item" onclick={(e) => { e.stopPropagation(); openMenuId = null; handleCommentAction("report", c); }}>Şikayet et</button>
                    {/if}
                    {#if isOwnComment || isAdmin}
                      {#if !isOwnComment}<div class="c-menu__divider"></div>{/if}
                      <button class="c-menu__item c-menu__item--danger" onclick={(e) => { e.stopPropagation(); openMenuId = null; handleCommentAction("delete", c); }}>Sil</button>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </div>
      {/each}
      {#if hasMore}
        <div class="u-text-center u-mt-md">
          <button class="c-btn c-btn--secondary" onclick={loadMore} disabled={isLoadingMore}>
            {isLoadingMore ? 'Yükleniyor...' : 'Daha Fazla Yükle'}
          </button>
        </div>
      {/if}
    </div>
  {/if}
{/if}
