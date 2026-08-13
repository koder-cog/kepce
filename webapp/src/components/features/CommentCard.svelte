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
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import CommentList from "./CommentList.svelte";
    import ActionMenu from "./ActionMenu.svelte";
    import { openSpamInfoModal } from "../../lib/dom/spam-modal.js";

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
    let rawNickname = $derived(isUserDeleted ? null : comment.user?.nickname);
    let userName = $derived(
        isUserDeleted ? "Silinmiş" : rawNickname || "Kepçe Kullanıcısı",
    );
    let isLinkable = $derived(
        rawNickname &&
            rawNickname.toLowerCase() !== "silinmiş" &&
            rawNickname.toLowerCase() !== "anonim",
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
        goto(`/yorumlar/${menuId}/${shortId}`);
    }

    function handleShare(e) {
        if (e) e.stopPropagation();
        const shortId = comment.id.substring(0, 7);
        const url = `${window.location.origin}/yorumlar/${menuId}/${shortId}`;
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
        if (isOwn) return;

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

    function countAllDescendants(node) {
        if (!node.children || node.children.length === 0) return 0;
        let count = node.children.length;
        for (const child of node.children) {
            count += countAllDescendants(child);
        }
        return count;
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_mouse_events_have_key_events -->
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
        {#if comment.user?.avatar_url && !isUserDeleted}
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
                data-link
                class="comment-node__avatar-link">{@render avatarSnippet()}</a
            >
        {:else}
            <div class="comment-node__avatar-link">
                {@render avatarSnippet()}
            </div>
        {/if}
        {#if hasChildren && depth < MAX_INDENT_DEPTH - 1}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="comment-node__line-hitbox" onclick={toggleCollapse}>
                <div class="comment-node__line"></div>
            </div>
        {/if}
    </div>
    <div class="comment-node__right">
        <div class="comment-node__header">
            {#if isLinkable}
                <a
                    href="/biri/{rawNickname}"
                    data-link
                    class="comment-node__user">{userName}</a
                >
            {:else}
                <span class="comment-node__user">{userName}</span>
            {/if}
            {#if isStructured}
                <span
                    class="comment-node__badge--structured"
                    data-tooltip="Bu yorum, kullanıcının klavye kullanacak kaloriye sahip olmaması sebebiyle el değmeden üretilmiştir."
                    >{@html icon("puzzle", 12)}</span
                >
            {/if}

            <span class="comment-node__dot">·</span>
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span class="comment-node__time" onclick={handleFocus}
                >{timeAgo(comment.created_at)}</span
            >
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="comment-node__header-toggle-area"
                onclick={toggleCollapse}
            ></div>
        </div>

        <div class="comment-node__body">
            {#if isDeleted}
                <div class="comment-node__content">
                    <span class="u-opacity-dim italic text-sm"
                        >[{isAdminDeleted
                            ? "Yorum admin tarafından kaldırıldı"
                            : "Yorum kullanıcı tarafından silindi"}]</span
                    >
                </div>
            {:else}
                <div class="comment-node__content">
                    <div
                        bind:this={textContainer}
                        class="comment-node__text {isExpanded
                            ? ''
                            : 'comment-node__text--clamped'}"
                    >
                        {@html sanitizeText(comment.comment || "")}
                    </div>
                    {#if isOverflowing && !isExpanded}
                        <button
                            class="btn-read-more"
                            onclick={(e) => {
                                e.stopPropagation();
                                isExpanded = true;
                            }}>Devamını oku...</button
                        >
                    {/if}
                    {#if comment.tags}
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
                                : ''} {isOwn ? 'is-disabled' : ''}"
                            data-vote="up"
                            onclick={(e) => handleVote("up", e)}
                            title={isOwn
                                ? "Kendi yorumunuza oy veremezsiniz"
                                : ""}
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
                                : ''} {isOwn ? 'is-disabled' : ''}"
                            data-vote="down"
                            onclick={(e) => handleVote("down", e)}
                            title={isOwn
                                ? "Kendi yorumunuza oy veremezsiniz"
                                : ""}
                        >
                            {@html icon(
                                reaction.my_vote === "down"
                                    ? "voteDownFilled"
                                    : "voteDown",
                                16,
                            )}
                        </button>
                    </div>
                    <button
                        class="action-btn"
                        onclick={toggleReply}
                        title="Yanıtla"
                    >
                        {@html icon("chat", 14)}
                        <span class="action-btn__text">Yanıtla</span>
                    </button>
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
                            ...(!isOwn ? [
                                { label: "Şikayet et", onClick: (e) => handleDropdownAction("report", e) },
                                ...(comment.user?.nickname && comment.user?.nickname !== "anonim" && comment.user?.nickname !== "silinmiş" ? [
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
</div>

<style>
    .comment-node__text--clamped {
        display: -webkit-box;
        line-clamp: 6;
        -webkit-line-clamp: 6;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }
    .btn-read-more {
        background: none;
        border: none;
        color: var(--color-primary, #1d9bd1);
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        padding: 4px 0 0 0;
        margin-top: 4px;
        text-align: left;
    }
    .btn-read-more:hover {
        text-decoration: underline;
    }
</style>
