<script>
    import "@/styles/pages/_comments.css";
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
    import Seo from "@/components/ui/Seo.svelte";

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
    let focalNode = $derived(focusId ? findNode(allComments, focusId) : null);
    let finalComments = $derived(focalNode ? [focalNode] : allComments);

    $effect(() => {
        if (focalNode && typeof document !== "undefined") {
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
            }, 100);
        }
    });

    let targetCitySlug = $derived.by(() => {
        if (menu?.city?.slug) return menu.city.slug;
        if (menu?.city_slug) return menu.city_slug;
        if (typeof menu?.city === "string") return menu.city;
        if (menu?.city?.name || menu?.city_name) {
            const nameToMatch = menu.city?.name || menu.city_name;
            const entry = Object.entries(CITY_MAP).find(
                ([_, name]) => name === nameToMatch,
            );
            if (entry) return entry[0];
        }
        return getCurrentCity();
    });

    let targetCityName = $derived.by(() => {
        if (menu?.city?.name) return menu.city.name;
        if (menu?.city_name) return menu.city_name;
        if (targetCitySlug && CITY_MAP[targetCitySlug])
            return CITY_MAP[targetCitySlug];
        if (targetCitySlug) {
            return (
                targetCitySlug.charAt(0).toLocaleUpperCase("tr-TR") +
                targetCitySlug.slice(1)
            );
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
                result.push({
                    id: n.id,
                    voteType: n.sentiment === "report" ? "report" : n.content,
                });
            }
            if (n.children && n.children.length > 0) {
                result = result.concat(flattenPureVotes(n.children));
            }
        }
        return result;
    }

    async function loadData() {
        if (!menuId) return;
        isLoading = true;
        errorState = null;

        try {
            const [menuData, commentsData] = await Promise.all([
                api.getMenu(menuId),
                api.getMenuComments(menuId).catch(() => []),
            ]);

            if (!menuData) {
                errorState = {
                    statusCode: 404,
                    desc: "Menü bulunamadı veya silinmiş.",
                };
                return;
            }

            menu = menuData;
            allComments = Array.isArray(commentsData)
                ? commentsData
                : (commentsData?.comments || []);
        } catch (err) {
            console.error("Menü yüklenirken hata oluştu:", err);
            errorState = {
                statusCode: err.status || 500,
                desc: err.message || "Menü verileri yüklenirken bir sorun oluştu.",
            };
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        loadData();
    });

    let ogImageUrl = $derived(
        focusId
            ? `https://kepce.org/api/v1/public/og/thread/${focusId}`
            : menuId
              ? `https://kepce.org/api/v1/public/og/menu/${menuId}`
              : "https://kepce.org/og_image.png"
    );
</script>

{#if isLoading}
    <div class="comments-page">
        <div id="menu-header-container">
            <div class="comments-page__header">
                <a
                    href="/"
                    onclick={handleBack}
                    data-link
                    class="comments-page__back-link"
                >
                    <span class="comments-page__back-icon">
                        {@html icon("chevronLeft", 24)}
                    </span>
                    <span class="comments-page__back-text"
                        >{targetCityName}</span
                    >
                </a>
            </div>
            <div class="stats-placeholder"><Loader size={48} /></div>
        </div>

        <div class="comments-section">
            <div class="comments-section-title">
                <h2>{focusId ? "Tartışma" : "Yorumlar"}</h2>
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
                <a
                    href="/"
                    onclick={handleBack}
                    data-link
                    class="comments-page__back-link"
                >
                    <span class="comments-page__back-icon">
                        {@html icon("chevronLeft", 24)}
                    </span>
                    <span class="comments-page__back-text"
                        >{targetCityName}</span
                    >
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
                                href="/menu/{menu.id}"
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

<Seo
    title={menu
        ? `${menu.date || menuId} Yemek Menüsü - Kepçe`
        : "Menü Detayı - Kepçe"}
    description={menu
        ? `${menu.date || menuId} tarihli KYK yurt yemek menüsü, besin değerleri ve öğrenci değerlendirmeleri.`
        : "KYK yurt yemek menüsü detayları ve öğrenci değerlendirmeleri."}
    image={ogImageUrl}
/>
