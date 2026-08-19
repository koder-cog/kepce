<script>
  import { globalState, authActions } from "@/state.svelte.js";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import { timeAgo } from "@/utils/date.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import ActionMenu from "@/components/features/ActionMenu.svelte";
  import { createModal } from "@/components/features/modal.js";
  import { showToast } from "@/components/ui/toast.js";
  import Loader from "@/components/ui/Loader.svelte";
  import { getCommentContextHtml } from "@/utils/turkish.js";
  import { sanitizeText } from "@/utils/sanitize.js";
  import { initCharCounter } from "@/utils/char-counter.js";
  import { page } from "$app/stores";

  let username = $derived($page.params.username);
  let contentLoading = $state(true);
  let contentError = $state(null);
  let commentsData = $state([]);
  let commentsSort = $state("new");
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
      const res = await api.getUserComments(
        username,
        commentsSort,
        limit,
        offset,
      );
      if (currentTabToken !== token) return;

      const newData = Array.isArray(res) ? res : res?.items || res?.data || [];
      if (isLoadMore) {
        commentsData = [...commentsData, ...newData];
      } else {
        commentsData = newData;
      }

      const total =
        res?.total_items ?? res?.total ?? (Array.isArray(res) ? res.length : 0);
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
    if (globalState.user.id === commentObj.user?.id) {
      showToast("Kendi yorumuna oy veremezsin.", "warning");
      return;
    }
    if (
      commentObj.is_blocked ||
      commentObj.user?.nickname === "Engellenmiş" ||
      commentObj.user?.nickname === "Engellemiş"
    ) {
      showToast("Engellenen içeriklere oy verilemez.", "warning");
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
    const commentId = comment.id || comment.hash;
    const menuId = comment.menu_id || comment.menu?.id || "";
    const requiresLogin = ["reply", "delete", "report"].includes(action);
    if (requiresLogin && !globalState?.user) {
      authActions.triggerLogin();
      return;
    }

    const shortId =
      typeof commentId === "string" ? commentId.substring(0, 7) : commentId;

    if (action === "reply") {
      goto(`/yorumlar/${menuId}/${shortId}`);
    } else if (action === "share") {
      const url = `${window.location.origin}/yorumlar/${menuId}/${shortId}`;
      navigator.clipboard
        .writeText(url)
        .then(() => showToast("Yorum linki kopyalandı!"));
    } else if (action === "report") {
      createModal({
        title: "Yorumu Şikayet et",
        contentHtml:
          "<p>Bu yorumun topluluk kurallarını ihlal ettiğini mi düşünüyorsun?</p>",
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
    } else if (action === "edit") {
      const currentText = comment.comment || "";
      const modalObj = createModal({
        title: "Yorumu Düzenle",
        iconHtml: icon("edit", 24),
        contentHtml: `
          <div class="c-modal__form-group">
            <div class="form-group">
              <textarea id="profile-edit-comment-input" rows="5" maxlength="500" placeholder="Yorumunu güncelle...">${sanitizeText(currentText)}</textarea>
            </div>
          </div>
        `,
        buttons: [
          { label: "Vazgeç", variant: "secondary" },
          {
            label: "Güncelle",
            variant: "primary",
            onClick: async (modalEl) => {
              const newText = modalEl
                .querySelector("#profile-edit-comment-input")
                .value.trim();
              if (!newText) {
                showToast("Yorum boş bırakılamaz.", "warning");
                return false;
              }
              try {
                const res = await api.updateComment(commentId, newText);
                comment.comment = res.comment;
                comment.is_edited = res.is_edited;
                showToast("Yorumun güncellendi!", "success");
                return true;
              } catch (e) {
                showToast(e.message || "Yorum güncellenemedi.", "error");
                return false;
              }
            },
          },
        ],
      });

      const textarea = modalObj.modal.querySelector(
        "#profile-edit-comment-input",
      );
      const saveBtn = modalObj.modal.querySelector(".btn--primary");
      initCharCounter(textarea, {
        onUpdate: (_count, _limit, isOver) => {
          saveBtn.disabled = isOver || textarea.value.trim().length === 0;
        },
      });
      textarea.focus();
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
  <div class="u-text-center u-p-2xl">
    <EmptyState
      iconName="warning"
      title="Hata"
      desc={contentError}
      actionLabel="Tekrar Dene"
      onAction={loadTabContent}
    />
  </div>
{:else if commentsData.length === 0}
  <div class="u-text-center u-p-2xl">
    <EmptyState
      iconName="chat"
      title="Henüz Yorum Yok"
      desc="Bu kullanıcı henüz bir menüye veya yemeğe yorum yapmamış."
    />
  </div>
{:else}
  <div class="profile-comments-list">
    <div class="profile-comments-header u-mb-md u-flex u-flex-justify-end">
      <Dropdown
        bind:value={commentsSort}
        options={[
          { value: "new", label: "En Yeni" },
          { value: "top", label: "En Beğenilen" },
        ]}
        onchange={handleSortChange}
      />
    </div>

    <div class="comment-card-list">
      {#each commentsData as c, idx}
        {@const commentKey = c.id || c.hash || `comment-${idx}`}
        {@const menuId = c.menu_id || c.menu?.id || ""}
        {@const threadTarget = c.id ? c.id.substring(0, 7) : c.hash || ""}
        {@const commentHref = menuId
          ? `/yorumlar/${menuId}/${threadTarget}`
          : `/yorumlar?thread=${threadTarget}`}
        {@const isOwnComment = globalState?.user?.id === c.user?.id}
        {@const isBlockedComment =
          c.is_blocked ||
          c.user?.nickname === "Engellenmiş" ||
          c.user?.nickname === "Engellemiş"}
        {@const isVoteDisabled = isOwnComment || isBlockedComment}
        {@const isAdmin = globalState?.user?.role === "admin"}
        {@const score = c.reaction_summary
          ? c.reaction_summary.up - c.reaction_summary.down
          : 0}
        <div class="comment-card" style="--stagger-idx: {idx}">
          <div class="comment-card__main-col">
            <div class="comment-card__header">
              <div class="comment-card__meta">
                <span class="comment-card__action-text"
                  >{@html getCommentContextHtml(c)}</span
                >
                <span class="comment-card__dot">·</span>
                <a href={commentHref} data-link class="comment-card__date"
                  >{timeAgo(c.created_at)}</a
                >
                {#if c.is_edited}
                  <span
                    class="comment-node__edited"
                    title="Bu yorum daha sonra düzenlendi">(düzenlendi)</span
                  >
                {/if}
              </div>
            </div>
            <div class="comment-card__body">
              <p class="comment-card__text">{c.comment}</p>
            </div>
            <div class="comment-node__actions">
              <div class="comment-node__vote">
                <button
                  class="vote-btn btn--squish {c.reaction_summary &&
                  c.reaction_summary.my_vote === 'up'
                    ? 'is-active'
                    : ''} {isVoteDisabled ? 'is-disabled' : ''}"
                  data-vote="up"
                  disabled={isVoteDisabled}
                  title={isOwnComment
                    ? "Kendi yorumuna oy veremezsin"
                    : isBlockedComment
                      ? "Engellenen içeriklere oy verilemez"
                      : "Beğen"}
                  onclick={() =>
                    !isVoteDisabled && reactToComment(c.hash || c.id, "up", c)}
                >
                  {@html icon(
                    c.reaction_summary && c.reaction_summary.my_vote === "up"
                      ? "voteUpFilled"
                      : "voteUp",
                    16,
                  )}
                </button>
                <span
                  class="vote-count {score > 0
                    ? 'positive'
                    : score < 0
                      ? 'negative'
                      : ''}">{score}</span
                >
                <button
                  class="vote-btn btn--squish {c.reaction_summary &&
                  c.reaction_summary.my_vote === 'down'
                    ? 'is-active'
                    : ''} {isVoteDisabled ? 'is-disabled' : ''}"
                  data-vote="down"
                  disabled={isVoteDisabled}
                  title={isOwnComment
                    ? "Kendi yorumuna oy veremezsin"
                    : isBlockedComment
                      ? "Engellenen içeriklere oy verilemez"
                      : "Beğenme"}
                  onclick={() =>
                    !isVoteDisabled &&
                    reactToComment(c.hash || c.id, "down", c)}
                >
                  {@html icon(
                    c.reaction_summary && c.reaction_summary.my_vote === "down"
                      ? "voteDownFilled"
                      : "voteDown",
                    16,
                  )}
                </button>
              </div>
              <button
                class="action-btn btn--squish"
                title="Yanıtla"
                onclick={() => handleCommentAction("reply", c)}
              >
                {@html icon("chat", 14)}
                <span class="action-btn__text">Yoruma git</span>
              </button>
              <button
                class="action-btn btn--squish"
                title="Paylaş"
                onclick={() => handleCommentAction("share", c)}
              >
                {@html icon("share", 14)}
                <span class="action-btn__text">Paylaş</span>
              </button>

              <ActionMenu
                triggerClass="action-btn btn--squish"
                triggerTitle="Daha fazla seçenek"
                items={[
                  ...(isOwnComment
                    ? [
                        {
                          label: "Düzenle",
                          onClick: () => handleCommentAction("edit", c),
                        },
                      ]
                    : []),
                  ...(!isOwnComment
                    ? [
                        {
                          label: "Şikayet et",
                          onClick: () => handleCommentAction("report", c),
                        },
                      ]
                    : []),
                  ...(isOwnComment || isAdmin
                    ? [
                        {
                          label: "Sil",
                          variant: "danger",
                          onClick: () => handleCommentAction("delete", c),
                        },
                      ]
                    : []),
                ]}
              />
            </div>
          </div>
        </div>
      {/each}
      {#if hasMore}
        <div class="u-text-center u-mt-md">
          <button
            class="c-btn c-btn--secondary"
            onclick={loadMore}
            disabled={isLoadingMore}
          >
            {isLoadingMore ? "Yükleniyor..." : "Daha Fazla Yükle"}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
