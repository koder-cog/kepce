<script>
    import { api } from "@/api/index.js";
    import { icon } from "@/components/ui/icons.js";
    import MenuCard from "@/components/features/MenuCard.svelte";
    import CommentInput from "@/components/features/CommentInput.svelte";
    import Loader from "@/components/ui/Loader.svelte";
    import EmptyState from "@/components/ui/EmptyState.svelte";
    import { isMotionEnabled } from "@/lib/dom/motion.js";
    import { getCurrentCity, setCurrentCity } from "@/stores/city.svelte.js";
    import { CITY_MAP } from "@/utils/turkish.js";
    import { onMount, onDestroy } from "svelte";
    import CommentList from "@/components/features/CommentList.svelte";

    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    let params = $derived(page.params);

    let menuId = $derived(params?.id);
    let focusId = $derived(
        params?.threadId || page.url.searchParams.get("thread"),
    );

    let isLoading = $state(true);
    let errorState = $state(null);
    let menu = $state(null);
    let allComments = $state([]);
    let finalComments = $state([]);
    let focalNode = $state(null);

    let targetCitySlug = $derived.by(() => {
        if (menu?.city?.slug) return menu.city.slug;
        if (menu?.city_slug) return menu.city_slug;
        if (typeof menu?.city === "string") return menu.city;
        if (menu?.city?.name || menu?.city_name) {
            const nameToMatch = menu.city?.name || menu.city_name;
            const entry = Object.entries(CITY_MAP).find(([_, name]) => name === nameToMatch);
            if (entry) return entry[0];
        }
        return getCurrentCity();
    });
    
    let targetCityName = $derived.by(() => {
        if (menu?.city?.name) return menu.city.name;
        if (menu?.city_name) return menu.city_name;
        if (targetCitySlug && CITY_MAP[targetCitySlug]) return CITY_MAP[targetCitySlug];
        if (targetCitySlug) {
            return targetCitySlug.charAt(0).toLocaleUpperCase("tr-TR") + targetCitySlug.slice(1);
        }
        return "İstanbul";
    });

    function handleBack(e) {
        e.preventDefault();
        
        if (targetCitySlug && targetCitySlug !== getCurrentCity()) {
            setCurrentCity(targetCitySlug);
        }
        
        goto("/");
    }

    function findNode(nodes, id) {
        for (const n of nodes) {
            if (n.id === id || n.id.substring(0, 7) === id) return n;
            if (n.children) {
                const found = findNode(n.children, id);
                if (found) return found;
            }
        }
        return null;
    }

    function flattenPureVotes(nodes) {
        let result = [];
        for (const n of nodes) {
            if (n.user?.nickname === "kepce_bot" || n.sentiment === "report") {
                continue;
            }
            if (!n.comment || n.comment.trim() === "") {
                if (n.children && n.children.length > 0) {
                    result.push(...flattenPureVotes(n.children));
                }
            } else {
                if (n.children) {
                    n.children = flattenPureVotes(n.children);
                }
                result.push(n);
            }
        }
        return result;
    }

    async function loadData() {
        if (!menuId) return;

        try {
            isLoading = true;
            errorState = null;

            let currentDietMode = "standard";
            if (typeof window !== "undefined") {
                currentDietMode =
                    localStorage.getItem("kepce_diet_mode") || "standard";
            }

            const [fetchedMenu, fetchedComments] = await Promise.all([
                api.getMenu(menuId, currentDietMode),
                api.getMenuComments(menuId),
            ]);

            menu = fetchedMenu;

            // Oy event'leri yoruma dönüştüğü için (pure votes) ağaç doğrulamasında yer almalı,
            // bu yüzden render aşamasında gizliyoruz (filter ile).
            allComments = flattenPureVotes(fetchedComments || []);

            if (focusId) {
                focalNode = findNode(allComments, focusId);
                finalComments = focalNode ? [focalNode] : allComments;
            } else {
                focalNode = null;
                finalComments = allComments;
            }

            isLoading = false;

            if (focalNode) {
                setTimeout(() => {
                    const target = document.querySelector(
                        `#comment-${focalNode.id}`,
                    );
                    if (target) {
                        target.scrollIntoView({
                            behavior: isMotionEnabled() ? "smooth" : "auto",
                            block: "start",
                        });
                    }
                }, 300);
            }
        } catch (err) {
            isLoading = false;
            errorState = {
                statusCode: err.status || 500,
                desc: err.message,
            };
        }
    }

    $effect(() => {
        if (menuId) {
            loadData();
        }
    });

    function onCommentSubmitted(e) {
        if (e.detail.menuId == menuId) {
            loadData();
        }
    }

    onMount(() => {
        window.addEventListener("comment-submitted", onCommentSubmitted);
    });

    onDestroy(() => {
        window.removeEventListener("comment-submitted", onCommentSubmitted);
    });
</script>

{#if !menuId}
    <div class="empty-state-container">
        <EmptyState
            iconName={"warning"}
            title={"Geçersiz Menü"}
            desc={"Hangi menüye bakmak istediğini anlayamadık."}
        />
    </div>
{:else if isLoading}
    <div class="comments-page">
        <div id="menu-header-container">
            <div class="empty-state-container">
                <Loader size={48} />
            </div>
        </div>
        <div class="comments-section">
            <div class="comments-section-title">
                <h2>Yorumlar</h2>
            </div>
            <div id="comments-list-container">
                <div class="stats-placeholder"><Loader size={48} /></div>
            </div>
        </div>
    </div>
{:else if errorState}
    <div class="empty-state-container">
        <EmptyState statusCode={errorState.statusCode} desc={errorState.desc}>
            <a href="/" data-link class="btn btn--primary">Ana sayfaya dön</a>
        </EmptyState>
    </div>
{:else}
    <div class="comments-page">
        <div id="menu-header-container">
            <div class="comments-page__header">
                <!--
                    Köprü: `/` (ana sayfa). Önceden `/{city.slug}` kullanılıyordu,
                    ancak projede `/{slug}/+page.svelte` rota dosyası yok; bu nedenle
                    SvelteKit 404 dönüyordu. Ana sayfa zaten seçili şehre göre
                    menüleri yüklüyor; şehir bilgisi yalnızca etiket olarak gösteriliyor.
                -->
                <a href="/" onclick={handleBack} data-link class="comments-page__back-link">
                    <span class="comments-page__back-icon">
                        {@html icon("chevronLeft", 24)}
                    </span>
                    <span class="comments-page__back-text">{targetCityName}</span>
                </a>
            </div>

            <MenuCard bind:menu options={{ hideComment: true }} />
        </div>

        <div class="comments-section">
            <div class="comments-section-title">
                <h2>{focusId ? "Tartışma" : "Yorumlar"}</h2>
            </div>

            {#if !focusId && menu}
                <CommentInput menuObj={menu} parentId={focusId} />
            {/if}

            <div id="comments-list-container">
                {#if finalComments.length === 0}
                    <EmptyState
                        iconName={"info"}
                        title={"Yorum Yok"}
                        desc={"Burada biz haşlanmış yumurtalardan başka kimse yok."}
                    />
                {:else}
                    {#if focusId}
                        <div class="comments-page__focus-nav u-mb-md">
                            <a
                                href="/yorumlar/{menu.id}"
                                data-link
                                class="btn btn--secondary btn--sm focus-return-btn"
                            >
                                {@html icon("arrowLeft", 16)} Tüm tartışmaya geri
                                dön
                            </a>
                        </div>
                    {/if}

                    <CommentList
                        comments={finalComments}
                        menuId={menu.id}
                        onloadData={loadData}
                    />
                {/if}
            </div>
        </div>
    </div>
{/if}

<svelte:head>
    <title>{menu ? `${menu.date || menuId} Yorumları - Kepçe` : "Yorumlar - Kepçe"}</title>
</svelte:head>
