<script>
    import "@/styles/pages/_comments.css";
    import { api } from "@/api/index.js";
    import { icon } from "@/components/ui/icons.js";
    import MenuCard from "@/components/features/MenuCard.svelte";
    import CommentInput from "@/components/features/CommentInput.svelte";
    import CommentList from "@/components/features/CommentList.svelte";
    import Loader from "@/components/ui/Loader.svelte";
    import EmptyState from "@/components/ui/EmptyState.svelte";
    import Seo from "@/components/ui/Seo.svelte";
    import { CITY_MAP, formatFullTurkishDate } from "@/utils/turkish.js";
    import { setCurrentCity } from "@/stores/city.svelte.js";
    import { onMount, untrack } from "svelte";

    let { data } = $props();

    let citySlug = $derived(data?.citySlug || "istanbul");
    let cityName = $derived(CITY_MAP[citySlug] || citySlug);
    let date = $derived(data?.date || "");
    let menus = $derived(data?.menus || []);
    let formattedDate = $derived(date ? formatFullTurkishDate(date) : "");

    untrack(() => {
        if (citySlug) setCurrentCity(citySlug);
    });

    // Her öğünün yorumları menu-bazlıdır; gün sayfası ikisini agregasyonla
    // gösterir (veri modeli değişmez, /menu/[id] derin linkleri çalışmaya devam eder).
    let commentsByMenu = $state({});

    async function loadComments(menuId) {
        if (commentsByMenu[menuId]) return commentsByMenu[menuId];
        try {
            const res = await api.getMenuComments(menuId);
            const list = Array.isArray(res) ? res : (res?.comments || []);
            commentsByMenu = { ...commentsByMenu, [menuId]: list };
            return list;
        } catch {
            commentsByMenu = { ...commentsByMenu, [menuId]: [] };
            return [];
        }
    }

    onMount(() => {
        for (const m of menus) {
            if (m?.id) loadComments(m.id);
        }
    });

    function dishNames(menu) {
        return (menu.items || menu.dishes || [])
            .map((d) => (typeof d === "string" ? d : d.raw_name ?? d.master_data?.name ?? d.name))
            .filter(Boolean);
    }

    function mealLabel(menu) {
        return menu?.meal_type === "breakfast" ? "Kahvaltı" : "Akşam Yemeği";
    }

    // Görünür özet paragrafı: yemek listesinden üretilen doğal metin.
    // (FAQ JSON-LD bilinçli olarak KULLANILMIYOR - şablonik schema spam riski.)
    let summaryText = $derived.by(() => {
        if (!menus.length) return "";
        const parts = menus.map(
            (m) => `${mealLabel(m)}: ${dishNames(m).join(", ")}`,
        );
        return `${formattedDate} ${cityName} KYK yurt menüsü - ${parts.join(" | ")}.`;
    });

    let daySchema = $derived.by(() => {
        if (!menus.length) return null;
        const dayUrl = `https://kepce.org/${citySlug}/${date}`;

        const menuObjects = menus.map((m) => {
            const dishes = dishNames(m);
            return {
                "@type": "Menu",
                name: `${formattedDate} ${cityName} KYK ${mealLabel(m)} Menüsü`,
                description: `${formattedDate} tarihli ${cityName} KYK yurt menüsü: ${dishes.join(", ")}`,
                inLanguage: "tr-TR",
                hasMenuItem: dishes.map((name) => ({
                    "@type": "MenuItem",
                    name,
                })),
            };
        });

        return {
            "@context": "https://schema.org",
            "@graph": [
                {
                    "@type": "BreadcrumbList",
                    itemListElement: [
                        { "@type": "ListItem", position: 1, name: "Ana Sayfa", item: "https://kepce.org/" },
                        { "@type": "ListItem", position: 2, name: cityName, item: `https://kepce.org/${citySlug}` },
                        { "@type": "ListItem", position: 3, name: `${formattedDate} ${cityName} KYK Menüsü`, item: dayUrl },
                    ],
                },
                ...menuObjects,
            ],
        };
    });

    let ogImage = $derived(
        menus[0]?.id
            ? `https://kepce.org/api/v1/public/og/menu/${menus[0].id}`
            : "https://kepce.org/og_image.png",
    );
</script>

<Seo
    title={formattedDate
        ? `${formattedDate} ${cityName} KYK Menüsü | Kepçe`
        : "KYK Gün Menüsü | Kepçe"}
    description={summaryText ||
        `${cityName} KYK yurt yemekhane gün menüsü, kahvaltı ve akşam yemeği listesi.`}
    image={ogImage}
    canonical={`https://kepce.org/${citySlug}/${date}`}
    schema={daySchema}
/>

<h1 class="sr-only">
    {formattedDate} {cityName} KYK Yurt Menüsü - Kahvaltı ve Akşam Yemeği Listesi
</h1>

<div class="comments-page">
    <div id="menu-header-container">
        <header class="comments-page__header">
            <div class="comments-page__header-top">
                <a
                    href="/{citySlug}"
                    data-link
                    class="comments-page__back-link"
                >
                    <span class="comments-page__back-icon">
                        {@html icon("chevronLeft", 18)}
                    </span>
                    <span class="comments-page__back-text">{cityName}</span>
                </a>

                <nav class="day-nav" aria-label="Gün navigasyonu">
                    {#if data?.prevDate && citySlug}
                        <a
                            href="/{citySlug}/{data.prevDate}"
                            data-link
                            class="day-nav__btn"
                            title="Önceki Gün"
                            aria-label="Önceki Gün"
                        >
                            {@html icon("chevronLeft", 14)}
                            <span class="day-nav__btn-text">Önceki Gün</span>
                        </a>
                    {/if}
                    {#if data?.nextDate && citySlug}
                        <a
                            href="/{citySlug}/{data.nextDate}"
                            data-link
                            class="day-nav__btn"
                            title="Sonraki Gün"
                            aria-label="Sonraki Gün"
                        >
                            <span class="day-nav__btn-text">Sonraki Gün</span>
                            {@html icon("chevronRight", 14)}
                        </a>
                    {/if}
                </nav>
            </div>

            {#if formattedDate}
                <h1 class="comments-page__title">{formattedDate}</h1>
            {/if}
        </header>

        {#each menus as menu, mi (menu.id)}
            <MenuCard bind:menu={menus[mi]} options={{ hideComment: true }} />

            <div class="comments-section">
                <div class="comments-section-title">
                    <h2>{mealLabel(menu)} Yorumları</h2>
                </div>

                {#if menu.id}
                    <CommentInput menuObj={menu} parentId={null} />

                    <div id="comments-list-container">
                        {#if commentsByMenu[menu.id] === undefined}
                            <div class="stats-placeholder"><Loader size={32} /></div>
                        {:else if commentsByMenu[menu.id].length === 0}
                            <EmptyState
                                iconName={"info"}
                                title={"Yorum Yok"}
                                desc={"Henüz yorum yapılmamış. Düşüncelerini ilk sen paylaş!"}
                            />
                        {:else}
                            <CommentList
                                comments={commentsByMenu[menu.id]}
                                menuId={menu.id}
                                onloadData={() => loadComments(menu.id)}
                            />
                        {/if}
                    </div>
                {/if}
            </div>
        {/each}

        {#if data?.prevDate || data?.nextDate}
            <nav class="day-nav" aria-label="Gün navigasyonu">
                {#if data?.prevDate}
                    <a
                        href="/{citySlug}/{data.prevDate}"
                        data-link
                        class="btn btn--secondary btn--sm"
                    >
                        {@html icon("chevronLeft", 16)} Önceki Gün
                    </a>
                {:else}
                    <div></div>
                {/if}
                {#if data?.nextDate}
                    <a
                        href="/{citySlug}/{data.nextDate}"
                        data-link
                        class="btn btn--secondary btn--sm"
                    >
                        Sonraki Gün {@html icon("chevronRight", 16)}
                    </a>
                {/if}
            </nav>
        {/if}
    </div>
</div>

<style>
    .day-nav {
        display: flex;
        justify-content: space-between;
        gap: var(--spacing-sm, 12px);
        margin: var(--spacing-lg, 24px) 0;
    }
</style>
