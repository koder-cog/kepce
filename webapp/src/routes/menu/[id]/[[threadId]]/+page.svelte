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
    import { CITY_MAP, formatFullTurkishDate } from "@/utils/turkish.js";
    import { onMount, onDestroy, untrack } from "svelte";
    import CommentList from "@/components/features/CommentList.svelte";
    import Seo from "@/components/ui/Seo.svelte";

    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    let params = $derived(page.params);
    let { data } = $props();

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

    // SSR: +page.js load()'un sunucuda çektiği menüyü hemen state'e aktar.
    // Böylece yemek isimleri, title ve Menu JSON-LD ilk HTML'de hazır olur.
    untrack(() => {
        if (data?.menu) {
            menu = data.menu;
            isLoading = false;
        }
    });

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
        // SSR'da menü zaten gömülüyse loader gösterme; sessizce tazele.
        if (!menu) isLoading = true;
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

    // Haftanin gunu basligi kirletiyor; sade gun-ay-yil yeterli.
    let formattedDate = $derived(
        menu?.date ? formatFullTurkishDate(menu.date) : ""
    );

    // Gün sayfası konsolidasyonu: canonical sinyalleri /{sehir}/{tarih}'e gider.
    let dayUrl = $derived(
        data?.dayUrl || (targetCitySlug && menu?.date ? `/${targetCitySlug}/${menu.date}` : null),
    );

    // Görünür özet paragrafı (FAQ JSON-LD bilinçli olarak kullanılmıyor).
    let summaryText = $derived.by(() => {
        if (!menu) return "";
        const dishes = (menu.items || menu.dishes || [])
            .map((d) => (typeof d === "string" ? d : d.raw_name ?? d.master_data?.name ?? d.name))
            .filter(Boolean);
        if (!dishes.length) return "";
        return `${formattedDate || menu.date} ${targetCityName} KYK ${mealLabel} menüsünde ${dishes.join(", ")} var.`;
    });

    let mealLabel = $derived(
        menu?.meal_type === "breakfast"
            ? "Kahvaltı"
            : menu?.meal_type === "dinner"
              ? "Akşam Yemeği"
              : "",
    );

    let menuSchema = $derived.by(() => {
        if (!menu) return null;
        const dishes = menu.items || menu.dishes || [];
        const dishEntries = dishes
            .map((d) => {
                if (typeof d === "string") return { name: d, calories: null };
                return {
                    name: d.raw_name ?? d.master_data?.name ?? d.name,
                    calories: d.master_data?.estimated_calories ?? d.calories ?? null,
                };
            })
            .filter((d) => d.name);
        const cityName = CITY_MAP[targetCitySlug] || targetCitySlug || "KYK";
        const dateLabel = formattedDate || menu.date || "";

        const votes = flattenPureVotes(allComments || []);
        let ratingValue = null;
        let reviewCount = votes.length;
        if (reviewCount > 0) {
            const scoreMap = { lezzetli: 5, guzel: 5, normal: 3.5, orta: 3, kotu: 2, berbat: 1, zehir: 1 };
            const totalScore = votes.reduce((acc, v) => acc + (scoreMap[v.voteType] || 3.5), 0);
            ratingValue = (totalScore / reviewCount).toFixed(1);
        }

        const menuObj = {
            "@type": "Menu",
            name: `${dateLabel} ${cityName} KYK ${mealLabel} Menüsü`,
            description: `${dateLabel} tarihli ${cityName} KYK yurt menüsü: ${dishEntries.map((d) => d.name).join(", ")}`,
            inLanguage: "tr-TR",
            hasMenuItem: dishEntries.map((d) => ({
                "@type": "MenuItem",
                name: d.name,
                ...(d.calories ? { description: `Yaklaşık ${d.calories} kcal` } : {}),
            })),
        };

        if (reviewCount >= 1 && ratingValue) {
            menuObj.aggregateRating = {
                "@type": "AggregateRating",
                ratingValue: ratingValue,
                bestRating: "5",
                worstRating: "1",
                ratingCount: reviewCount,
            };
        }

        return {
            "@context": "https://schema.org",
            "@graph": [
                {
                    "@type": "BreadcrumbList",
                    itemListElement: [
                        { "@type": "ListItem", position: 1, name: "Ana Sayfa", item: "https://kepce.org/" },
                        ...(targetCitySlug
                            ? [{ "@type": "ListItem", position: 2, name: cityName, item: `https://kepce.org/${targetCitySlug}` }]
                            : []),
                        { "@type": "ListItem", position: targetCitySlug ? 3 : 2, name: `${dateLabel} ${cityName} KYK ${mealLabel} Menüsü` },
                    ],
                },
                menuObj,
            ],
        };
    });
</script>

<h1 class="sr-only">
    {menu
        ? `${formattedDate || menu.date} ${CITY_MAP[targetCitySlug] || targetCitySlug || ""} KYK ${mealLabel} Menüsü Detayı`
        : "KYK Yemek Menüsü Detayı ve Yorumları"}
</h1>

{#if menu && summaryText}
    <p class="day-summary">{summaryText}</p>
{/if}

{#if menu && dayUrl}
    <div class="day-link-row">
        <a href={dayUrl} data-link class="btn btn--secondary btn--sm">
            {@html icon("cards", 16)} Tüm öğünleri gör
        </a>
        {#if data?.prevDate && targetCitySlug}
            <a
                href="/{targetCitySlug}/{data.prevDate}"
                data-link
                class="btn btn--secondary btn--sm"
            >
                {@html icon("chevronLeft", 16)} Önceki Gün
            </a>
        {/if}
        {#if data?.nextDate && targetCitySlug}
            <a
                href="/{targetCitySlug}/{data.nextDate}"
                data-link
                class="btn btn--secondary btn--sm"
            >
                Sonraki Gün {@html icon("chevronRight", 16)}
            </a>
        {/if}
    </div>
{/if}

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
        ? `${formattedDate || menu.date} ${CITY_MAP[targetCitySlug] || targetCitySlug || ""} KYK ${mealLabel} Menüsü | Kepçe`
        : "KYK Yemek Menüsü Detayı | Kepçe"}
    description={menu
        ? `${formattedDate || menu.date} ${CITY_MAP[targetCitySlug] || targetCitySlug || "KYK"} ${mealLabel} menüsü detayları, besin değerleri ve öğrenci yorumları.`
        : "KYK yurt yemek menüsü detayları ve öğrenci değerlendirmeleri."}
    image={ogImageUrl}
    canonical={dayUrl ? `https://kepce.org${dayUrl}` : `https://kepce.org/menu/${menuId}`}
    schema={menuSchema}
/>

<style>
    .day-summary {
        margin: 0 0 var(--spacing-sm, 12px);
        padding: var(--spacing-sm, 12px) var(--spacing-md, 16px);
        border-radius: 12px;
        background: var(--bg-elevated, rgba(255, 255, 255, 0.04));
        color: var(--text-secondary, inherit);
        font-size: 0.95rem;
        line-height: 1.6;
    }

    .day-link-row {
        display: flex;
        gap: var(--spacing-sm, 12px);
        margin: 0 0 var(--spacing-md, 16px);
    }
</style>
