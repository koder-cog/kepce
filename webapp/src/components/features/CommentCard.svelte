<!-- 
  Kepçe Webapp — Bileşen: CommentCard
  ===================================
  
  Tek bir yorumu ve ona ait reaksiyon/yanıt formunu gösterir.
-->

<script>
    import { api } from "../../api/index.js";
    import { icon } from "../ui/icons.js";
    import { globalState, authActions } from "../../state.svelte.js";
    import { sanitizeText } from "../../utils/sanitize.js";
    import { showToast } from "../ui/toast.js";
    import { timeAgo } from "../../utils/date.js";
    import CommentList from "./CommentList.svelte";
    import ActionMenu from "./ActionMenu.svelte";
    import { openSpamInfoModal } from "../../lib/dom/spam-modal.js";
    import { createModal } from "./modal.js";
    import { initCharCounter } from "../../utils/char-counter.js";

    let { comment, depth = 0, menuId, onloadData } = $props();

    const MAX_INDENT_DEPTH = 5;
    let currentDepth = $derived(Math.min(depth, MAX_INDENT_DEPTH));

    let isDeleted = $derived(
        Boolean(comment.is_deleted || comment.deleted || comment.deleted_at),
    );
    let isAdminDeleted = $derived(comment.deletion_type === "admin");
    let isStructured = $derived(
        Boolean(
            comment.is_tabldot ||
                (comment.tags &&
                    Array.isArray(comment.tags) &&
                    comment.tags.some(
                        (t) => t === "tabldot" || t.tag_id === "tabldot",
                    )) ||
                (comment.tag_ids &&
                    Array.isArray(comment.tag_ids) &&
                    comment.tag_ids.some((t) => t === "tabldot")),
        ),
    );

    let isUserDeleted = $derived(isDeleted && !isAdminDeleted);
    let isBlocked = $derived(
        Boolean(
            comment.is_blocked ||
                comment.user?.nickname === "Engellenmiş" ||
                comment.user?.nickname === "Engellemiş",
        ),
    );
    let rawNickname = $derived(isUserDeleted ? null : comment.user?.nickname);
    let userName = $derived(
        isUserDeleted ? "Silinmiş" : rawNickname || "Kepçe Kullanıcısı",
    );
    let isLinkable = $derived(
        rawNickname &&
            rawNickname.toLowerCase() !== "silinmiş" &&
            rawNickname.toLowerCase() !== "anonim" &&
            rawNickname !== "Engellenmiş" &&
            rawNickname !== "Engellemiş",
    );

    // Avatar rendering snippet'a taşındı (XSS önlemi)

    let isOwn = $derived(globalState?.user?.id === comment.user?.id);
    let reaction = $derived(
        comment.reaction_summary || { up: 0, down: 0, my_vote: null },
    );
    let score = $derived((reaction.up || 0) - (reaction.down || 0));
    let hasChildren = $derived(comment.children && comment.children.length > 0);
    let shouldAutoCollapse = $derived(isDeleted && !hasChildren);

    // svelte-ignore state_referenced_locally
    let isCollapsed = $state(
        isDeleted && !(comment.children && comment.children.length > 0),
    );
    let replying = $state(false);
    let replyText = $state("");

    let isExpanded = $state(false);
    let textContainer = $state(null);
    let isOverflowing = $state(false);

    $effect(() => {
        isCollapsed = shouldAutoCollapse;
    });

    $effect(() => {
        if (textContainer && !isExpanded) {
            isOverflowing =
                textContainer.scrollHeight > textContainer.clientHeight;
        }
    });

    function toggleCollapse(e) {
        if (e) e.stopPropagation();
        isCollapsed = !isCollapsed;
    }

    function handleFocus(e) {
        if (e) e.stopPropagation();
        const shortId = comment.id.substring(0, 7);
        goto(`/menu/${menuId}/${shortId}`);
    }

    function handleShare(e) {
        if (e) e.stopPropagation();
        const shortId = comment.id.substring(0, 7);
        const url = `${window.location.origin}/menu/${menuId}/${shortId}`;
        navigator.clipboard
            .writeText(url)
            .then(() => showToast("Yorum linki kopyalandı!"));
    }

    function toggleReply(e) {
        if (e) e.stopPropagation();
        if (!globalState?.user) {
            authActions.triggerLogin();
            return;
        }
        if (!globalState?.user?.is_verified) {
            showToast("Yorum yapabilmek için e-postanızı onaylamalısınız.", {
                type: "warning",
            });
            return;
        }
        replying = !replying;
        replyText = "";
    }

    async function submitReply() {
        if (!replyText.trim()) return;
        if (!globalState?.user?.is_verified) {
            showToast("Yorum yapabilmek için e-postanızı onaylamalısınız.", {
                type: "warning",
            });
            return;
        }
        try {
            await api.postComment(menuId, replyText, comment.id);
            replying = false;
            replyText = "";
            if (onloadData) await onloadData();
        } catch (err) {
            const isSpam = err.message && err.message.includes("spam");
            showToast(err.message, {
                type: "error",
                action: isSpam
                    ? {
                          text: "Bilgi",
                          callback: () => openSpamInfoModal(),
                      }
                    : null,
            });
        }
    }

    async function handleVote(type, e) {
        if (e) e.stopPropagation();
        if (!globalState?.user) {
            authActions.triggerLogin();
            return;
        }

        if (!globalState?.user?.is_verified) {
            showToast("Oy verebilmek için e-postanızı onaylamalısınız.", {
                type: "warning",
            });
            return;
        }
        if (isOwn) {
            showToast("Kendi yorumunuza oy veremezsiniz.", "warning");
            return;
        }
        if (isBlocked) {
            showToast("Engellenen içeriklere oy verilemez.", "warning");
            return;
        }

        const isRemoving = reaction.my_vote === type;
        const oppositeType = type === "up" ? "down" : "up";
        const isFlipping = reaction.my_vote === oppositeType;

        const newUp =
            reaction.up +
            (type === "up" ? (isRemoving ? -1 : 1) : isFlipping ? -1 : 0);
        const newDown =
            reaction.down +
            (type === "down" ? (isRemoving ? -1 : 1) : isFlipping ? -1 : 0);

        const previousReaction = { ...reaction };

        comment.reaction_summary = {
            up: newUp,
            down: newDown,
            my_vote: isRemoving ? null : type,
        };

        try {
            const result = await api.voteComment(comment.id, type);
            if (result && result.reaction_summary) {
                comment.reaction_summary = result.reaction_summary;
            }
        } catch (err) {
            showToast(err.message, "error");
            comment.reaction_summary = previousReaction;
        }
    }

    async function handleDropdownAction(action, e) {
        if (e) e.stopPropagation();
        if (!globalState?.user) {
            authActions.triggerLogin();
            return;
        }

        try {
            if (action === "report") {
                await api.reportComment(comment.id);
                showToast("Şikayetin alındı, moderatörlerimize ilettik.");
            } else if (action === "block") {
                await api.blockUser(comment.user.id);
                showToast(`${comment.user.nickname} engellendi.`);
            } else if (action === "delete") {
                await api.deleteComment(comment.id);
                if (onloadData) await onloadData();
            } else if (action === "purge") {
                await api.purgeComment(comment.id);
                if (onloadData) await onloadData();
            }
        } catch (err) {
            showToast(err.message, "error");
        }
    }

    function openEditCommentModal() {
        const currentText = comment.comment || "";
        const modalObj = createModal({
            title: "Yorumu Düzenle",
            iconHtml: icon("edit", 24),
            contentHtml: `
                <div class="c-modal__form-group">
                    <div class="form-group">
                        <textarea id="edit-comment-input" rows="5" maxlength="500" placeholder="Yorumunu güncelle...">${sanitizeText(currentText)}</textarea>
                    </div>
                </div>
            `,
            buttons: [
                { label: "Vazgeç", variant: "secondary" },
                {
                    label: "Güncelle",
                    variant: "primary",
                    onClick: async (modalEl) => {
                        const newText = modalEl.querySelector("#edit-comment-input").value.trim();
                        if (!newText) {
                            showToast("Yorum boş bırakılamaz.", "warning");
                            return false;
                        }
                        try {
                            const res = await api.updateComment(comment.id, newText);
                            comment.comment = res.comment;
                            comment.is_edited = res.is_edited;
                            showToast("Yorumun güncellendi!", "success");
                            if (onloadData) await onloadData();
                            return true;
                        } catch (err) {
                            showToast(err.message || "Yorum güncellenemedi.", "error");
                            return false;
                        }
                    },
                },
            ],
        });

        const textarea = modalObj.modal.querySelector("#edit-comment-input");
        const saveBtn = modalObj.modal.querySelector(".btn--primary");
        initCharCounter(textarea, {
            onUpdate: (_count, _limit, isOver) => {
                saveBtn.disabled = isOver || textarea.value.trim().length === 0;
            },
        });
        textarea.focus();
    }

    function countAllDescendants(node) {
        if (!node.children || node.children.length === 0) return 0;
        let count = node.children.length;
        for (const child of node.children) {
            count += countAllDescendants(child);
        }
        return count;
    }
</script>

<div
    class="comment-node {depth >= MAX_INDENT_DEPTH
        ? 'is-max-depth'
        : ''} {isCollapsed ? 'is-collapsed' : ''} {comment.user?.nickname ===
    'kepce_bot'
        ? 'comment-kepce_bot'
        : ''}"
    id="comment-{comment.id}"
    data-id={comment.id}
    data-depth={currentDepth}
>
    {#snippet avatarSnippet()}
        {#if comment.user?.avatar_url && !isUserDeleted && !isBlocked}
            <img
                src={api.getAvatarUrl(comment.user.avatar_url)}
                alt=""
                onerror={(e) => (e.target.outerHTML = icon("avatarEmpty", 32))}
            />
        {:else}
            {@html icon("avatarEmpty", 32)}
        {/if}
    {/snippet}

    <div class="comment-node__left">
        {#if isLinkable}
            <a
                href="/biri/{rawNickname}"
                class="comment-node__avatar"
                aria-label="{rawNickname} profilini görüntüle"
                data-link
            >
                {@render avatarSnippet()}
            </a>
        {:else}
            <div class="comment-node__avatar">
                {@render avatarSnippet()}
            </div>
        {/if}
        {#if hasChildren}
            <button
                class="comment-node__thread-line"
                onclick={toggleCollapse}
                aria-label={isCollapsed
                    ? "Yorum yanıtlarını genişlet"
                    : "Yorum yanıtlarını daralt"}
            ></button>
        {/if}
    </div>

    <div class="comment-node__right">
        <div class="comment-node__header">
            {#if isLinkable}
                <a
                    href="/biri/{rawNickname}"
                    class="comment-node__author"
                    data-link
                >
                    {userName}
                </a>
            {:else}
                <span class="comment-node__author">{userName}</span>
            {/if}

            {#if comment.user?.level}
                <span class="comment-node__level"
                    >Lvl {comment.user.level}</span
                >
            {/if}

            <span class="comment-node__dot">·</span>
            <button
                class="comment-node__time comment-node__time-btn"
                onclick={handleFocus}
                title="Tartışmaya odaklan"
            >
                {timeAgo(comment.created_at)}
            </button>

            {#if comment.is_edited}
                <span class="comment-node__edited" title="Bu yorum daha sonra düzenlendi">(düzenlendi)</span>
            {/if}

            {#if hasChildren}
                <button
                    class="comment-node__collapse-btn"
                    onclick={toggleCollapse}
                    title={isCollapsed ? "Genişlet" : "Daralt"}
                >
                    {@html icon(isCollapsed ? "plus" : "minus", 14)}
                    {#if isCollapsed}
                        <span class="collapsed-count"
                            >({countAllDescendants(comment)} yanıt)</span
                        >
                    {/if}
                </button>
            {/if}
        </div>

        {#if !isCollapsed}
            <div class="comment-node__content">
                <div
                    class="comment-node__text-container {isExpanded
                        ? 'is-expanded'
                        : ''}"
                    bind:this={textContainer}
                >
                    <p
                        class="comment-node__text {isDeleted
                            ? 'is-deleted'
                            : ''} {isBlocked ? 'is-blocked-text' : ''}"
                    >
                        {comment.comment || ""}
                    </p>
                    {#if isOverflowing && !isExpanded}
                        <button
                            class="comment-node__more-btn"
                            onclick={() => (isExpanded = true)}
                        >
                            Devamını oku
                        </button>
                    {/if}
                    {#if comment.tags && Array.isArray(comment.tags) && comment.tags.length > 0}
                        <div class="comment-node__tags u-mt-xs">
                            {#each comment.tags as tag}
                                {#if tag === "tabldot" || tag.tag_id === "tabldot"}
                                    <span class="comment-tag-badge"
                                        >Tabldot</span
                                    >
                                {:else if tag.name || typeof tag === "string"}
                                    <span class="comment-tag-badge"
                                        >{@html sanitizeText(
                                            tag.name || tag,
                                        )}</span
                                    >
                                {/if}
                            {/each}
                        </div>
                    {/if}
                </div>

                <div class="comment-node__actions">
                    <div class="comment-node__vote">
                        <button
                            class="vote-btn {reaction.my_vote === 'up'
                                ? 'is-active'
                                : ''} {isOwn || isBlocked ? 'is-disabled' : ''}"
                            data-vote="up"
                            disabled={isOwn || isBlocked}
                            onclick={(e) => handleVote("up", e)}
                            title={isOwn
                                ? "Kendi yorumunuza oy veremezsiniz"
                                : isBlocked
                                  ? "Engellenen içeriklere oy verilemez"
                                  : "Beğen"}
                        >
                            {@html icon(
                                reaction.my_vote === "up"
                                    ? "voteUpFilled"
                                    : "voteUp",
                                16,
                            )}
                        </button>
                        <span
                            class="vote-count {reaction.my_vote === 'up'
                                ? 'positive'
                                : reaction.my_vote === 'down'
                                  ? 'negative'
                                  : score > 0
                                    ? 'positive'
                                    : score < 0
                                      ? 'negative'
                                      : ''}">{score}</span
                        >
                        <button
                            class="vote-btn {reaction.my_vote === 'down'
                                ? 'is-active'
                                : ''} {isOwn || isBlocked ? 'is-disabled' : ''}"
                            data-vote="down"
                            disabled={isOwn || isBlocked}
                            onclick={(e) => handleVote("down", e)}
                            title={isOwn
                                ? "Kendi yorumunuza oy veremezsiniz"
                                : isBlocked
                                  ? "Engellenen içeriklere oy verilemez"
                                  : "Beğenme"}
                        >
                            {@html icon(
                                reaction.my_vote === "down"
                                    ? "voteDownFilled"
                                    : "voteDown",
                                16,
                            )}
                        </button>
                    </div>
                    {#if !isBlocked}
                        <button
                            class="action-btn"
                            onclick={toggleReply}
                            title="Yanıtla"
                        >
                            {@html icon("chat", 14)}
                            <span class="action-btn__text">Yanıtla</span>
                        </button>
                    {/if}
                    <button
                        class="action-btn"
                        onclick={handleShare}
                        title="Paylaş"
                    >
                        {@html icon("share", 14)}
                        <span class="action-btn__text">Paylaş</span>
                    </button>
                    <ActionMenu
                        triggerClass="action-btn"
                        triggerTitle="Daha fazla seçenek"
                        items={[
                            ...(isOwn && !isDeleted ? [
                                { label: "Düzenle", onClick: () => openEditCommentModal() }
                            ] : []),
                            ...(!isOwn ? [
                                { label: "Şikayet et", onClick: (e) => handleDropdownAction("report", e) },
                                ...(comment.user?.nickname && !["anonim", "silinmiş", "Engellenmiş", "Engellemiş"].includes(comment.user?.nickname) ? [
                                    { label: "Kullanıcıyı engelle", onClick: (e) => handleDropdownAction("block", e) }
                                ] : [])
                            ] : []),
                            ...(!isOwn && globalState?.user?.role === "admin" ? [{ divider: true }] : []),
                            ...(isOwn || globalState?.user?.role === "admin" ? [
                                { label: "Sil", variant: "danger", onClick: (e) => handleDropdownAction("delete", e) }
                            ] : []),
                            ...(globalState?.user?.role === "admin" ? [
                                { label: "Kalıcı sil", variant: "danger", onClick: (e) => handleDropdownAction("purge", e) }
                            ] : [])
                        ]}
                    />
                </div>

                {#if replying}
                    <div class="comment-reply-form-container">
                        <div class="comment-reply-form active u-mt-md">
                            <!-- svelte-ignore a11y_autofocus -->
                            <textarea
                                bind:value={replyText}
                                autofocus
                                placeholder="Yanıtınızı buraya yazın..."
                                class="comment-panel__textarea"
                            ></textarea>
                            <div
                                class="u-flex u-flex-justify-end u-flex-gap-sm u-mt-sm"
                            >
                                <button
                                    class="btn btn--secondary btn--sm"
                                    onclick={() => (replying = false)}
                                    >Vazgeç</button
                                >
                                <button
                                    class="btn btn--primary btn--sm"
                                    onclick={submitReply}>Yanıtla</button
                                >
                            </div>
                        </div>
                    </div>
                {/if}
            </div>
        {/if}

        {#if hasChildren}
            {#if depth >= MAX_INDENT_DEPTH - 1}
                {#if countAllDescendants(comment) > 0}
                    <div class="comment-node__more-replies">
                        <button
                            class="btn-more-replies"
                            onclick={handleFocus}
                        >
                            {@html icon("plusCircle", 16)}
                            <span
                                >{countAllDescendants(comment)} yanıtı daha gör</span
                            >
                        </button>
                    </div>
                {/if}
            {:else}
                <div class="comment-node__replies">
                    <CommentList
                        comments={comment.children}
                        depth={depth + 1}
                        {menuId}
                        {onloadData}
                    />
                </div>
            {/if}
        {/if}
    </div>
</div>
