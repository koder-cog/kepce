<script>
    import { timelineState } from "@/stores/timeline.svelte.js";
    import DailyHeader from "@/components/features/timeline/DailyHeader.svelte";
    import CalendarSelector from "@/components/features/timeline/CalendarSelector.svelte";
    import TimelineView from "@/components/features/timeline/TimelineView.svelte";
    import { CITY_MAP } from "@/utils/turkish.js";
    import Seo from "@/components/ui/Seo.svelte";
    import { onMount } from "svelte";

    import { untrack } from "svelte";

    let { data } = $props();

    // SSR / Init: data'dan gelen prerender menülerini hemen store'a aktar.
    // Senkron çağrı hem SSR prerender'da hem ilk client render'da çalışır.
    untrack(() => {
        if (data?.prerenderedCity) {
            timelineState.setPrerenderedData(
                data.prerenderedMenus || [],
                data.prerenderedCity,
                data.prerenderedDate,
            );
        }
    });

    onMount(() => {
        // Görev #24: URL üzerinden diyet modu zorlaması (?diyet=celiac).
        // Çölyaksız bir ayda bile mod korunur ve "çölyak menüsü yok"
        // empty-state'i gösterilir.
        const params = new URLSearchParams(window.location.search);
        const dietParam = params.get("diyet") || params.get("diet");
        if (dietParam === "celiac" || dietParam === "colyak") {
            timelineState.forceDietMode("celiac");
        }
        timelineState.init();
    });

    const turkishMonths = [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
        "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"
    ];
    const now = new Date();
    const todayTurkishStr = `${now.getDate()} ${turkishMonths[now.getMonth()]} ${now.getFullYear()}`;

    let cityName = $derived(
        CITY_MAP[timelineState.currentCity] || timelineState.currentCity,
    );

    let pageTitle = $derived(
        timelineState.currentCity
            ? `${cityName} KYK Yemek Menüsü (Günlük Tabldot) | Kepçe`
            : "KYK Yemek Menüsü - Bugün KYK'da Ne Yemek Var? | Kepçe",
    );

    let pageDescription = $derived(
        timelineState.currentCity
            ? `${todayTurkishStr} ${cityName} KYK yurt yemekhane menüsü: Güncel kahvaltı ve akşam tabldot listesi, kalori ve beslenme yardımı detayları.`
            : `${todayTurkishStr} KYK yurtlarında çıkan kahvaltı ve akşam yemeği menüsü. 81 il güncel tabldot listeleri, kalori ve beslenme yardımı detayları.`,
    );
    let ogImage = $derived(
        timelineState.currentCity
            ? `https://kepce.org/api/v1/public/og/city/${timelineState.currentCity}`
            : "https://kepce.org/og_image.png",
    );

    let canonicalUrl = "https://kepce.org";

    let menuSchema = $derived.by(() => {
        const menus = timelineState.menusState || [];
        const baseGraphs = [
            {
                "@type": "WebSite",
                "@id": "https://kepce.org/#website",
                url: "https://kepce.org/",
                name: "Kepçe",
                description:
                    "Bugün KYK'da Ne Yemek Var? Günlük KYK Yurt Menüleri",
                inLanguage: "tr-TR",
            },
            {
                "@type": "Organization",
                "@id": "https://kepce.org/#organization",
                name: "Kepçe",
                url: "https://kepce.org/",
                logo: "https://kepce.org/icon-512.png",
            },
        ];

        if (menus && menus.length > 0) {
            const sections = menus.map((m) => {
                const dishes = m.items || m.dishes || [];
                const dishNames = dishes
                    .map((d) => (typeof d === "string" ? d : d.name))
                    .filter(Boolean);
                const mealTitle =
                    m.meal_type === "breakfast" ? "Kahvaltı" : "Akşam Yemeği";
                return {
                    "@type": "MenuSection",
                    name: mealTitle,
                    hasMenuItem: dishNames.map((name) => ({
                        "@type": "MenuItem",
                        name: name,
                    })),
                };
            });

            baseGraphs.push({
                "@type": "Menu",
                "@id": "https://kepce.org/#menu",
                name: `${cityName} KYK Günlük Yemek Menüsü`,
                inLanguage: "tr-TR",
                datePublished: `${now.toISOString().split("T")[0]}T00:00:00+03:00`,
                dateModified: now.toISOString(),
                hasMenuSection: sections,
            });
        }

        return {
            "@context": "https://schema.org",
            "@graph": baseGraphs,
        };
    });
</script>

<Seo
    title={pageTitle}
    description={pageDescription}
    image={ogImage}
    canonical={canonicalUrl}
    noindex={false}
    schema={menuSchema}
/>

<h1 class="sr-only">
    {timelineState.currentCity
        ? `${CITY_MAP[timelineState.currentCity] || timelineState.currentCity} KYK Yurt Yemek Menüsü`
        : "KYK Yurt Yemek Menüsü - Günlük Yemek Listesi"}
</h1>

<DailyHeader />
<CalendarSelector />
<TimelineView
    lastMenuDay={data?.lastMenuDay}
    isSummer={data?.isSummer}
/>

<nav class="city-seo-directory" aria-label="Şehirlere Göre KYK Yemek Menüleri">
    <h2 class="city-seo-directory__title">Şehirlere Göre KYK Menüleri</h2>
    <div class="city-seo-directory__grid">
        {#each Object.entries(CITY_MAP) as [slug, name]}
            <a href="/{slug}" class="city-seo-directory__link" title="{name} KYK Yemek Menüsü">
                {name}
            </a>
        {/each}
    </div>
    <div class="city-seo-directory__about">
        <p>
            Kepçe; 81 ildeki Gençlik ve Spor Bakanlığı (GSB) KYK yurtlarında çıkan günlük kahvaltı ve akşam yemeği tabldot menülerini, kalori değerlerini ve beslenme yardımı detaylarını reklamsız olarak sunar.
        </p>
    </div>
    <div class="city-seo-directory__guides">
        <a href="/kyk-yemek-saatleri" class="city-seo-directory__guide-link">
            2026-2027 KYK Yemek Saatleri (Sabah & Akşam) →
        </a>
        <a href="/kyk-beslenme-yardimi" class="city-seo-directory__guide-link">
            2026-2027 KYK Beslenme Yardımı Ücretleri →
        </a>
    </div>
</nav>
