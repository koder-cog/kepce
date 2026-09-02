<script>
    import { timelineState } from "@/stores/timeline.svelte.js";
    import { setCurrentCity } from "@/stores/city.svelte.js";
    import DailyHeader from "@/components/features/timeline/DailyHeader.svelte";
    import CalendarSelector from "@/components/features/timeline/CalendarSelector.svelte";
    import TimelineView from "@/components/features/timeline/TimelineView.svelte";
    import { CITY_MAP } from "@/utils/turkish.js";
    import Seo from "@/components/ui/Seo.svelte";
    import { onMount, untrack } from "svelte";

    let { data } = $props();

    let citySlug = $derived(data?.citySlug || "istanbul");
    let cityName = $derived(CITY_MAP[citySlug] || citySlug);

    // SSR: load()'un sunucuda çektiği menüleri hemen store'a aktar.
    // Yemek isimleri ve Menu JSON-LD ilk HTML'de hazır olur.
    untrack(() => {
        if (data?.menus) {
            timelineState.setPrerenderedData(
                data.menus,
                citySlug,
                data.date,
            );
        }
    });

    onMount(() => {
        if (citySlug) {
            setCurrentCity(citySlug);
        }
        timelineState.init();
    });

    const turkishMonths = [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
        "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"
    ];
    const now = new Date();
    const todayTurkishStr = `${now.getDate()} ${turkishMonths[now.getMonth()]} ${now.getFullYear()}`;

    let pageTitle = $derived(`Bugünkü ${cityName} KYK Yemek Menüsü (2026-2027) — Güncel Tabldot | Kepçe`);
    let pageDescription = $derived(
        `Bugünkü ${cityName} KYK yurt yemekhane menüsü: ${todayTurkishStr} kahvaltı ve akşam yemeği tabldot listesi, kalori ve beslenme yardımı detayları.`
    );
    let canonicalUrl = $derived(`https://kepce.org/${citySlug}`);
    let ogImage = $derived(`https://kepce.org/api/v1/public/og/city/${citySlug}`);

    let menuSchema = $derived.by(() => {
        const menus = timelineState.menusState || [];
        const isoNow = now.toISOString();
        const baseGraphs = [
            {
                "@type": "WebSite",
                "@id": "https://kepce.org/#website",
                url: "https://kepce.org/",
                name: "Kepçe",
                description: "Bugün KYK'da Ne Yemek Var? Günlük KYK Yurt Menüleri",
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
                const dishNames = dishes.map((d) => (typeof d === "string" ? d : d.name)).filter(Boolean);
                const mealTitle = m.meal_type === "breakfast" ? "Kahvaltı" : "Akşam Yemeği";
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
                "@id": `https://kepce.org/${citySlug}#menu`,
                name: `${cityName} KYK Günlük Yemek Menüsü`,
                inLanguage: "tr-TR",
                datePublished: `${now.toISOString().split("T")[0]}T00:00:00+03:00`,
                dateModified: isoNow,
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
    {cityName} KYK Yurt Yemek Menüsü - Günlük Yemek Listesi
</h1>

<DailyHeader />
<CalendarSelector />
<TimelineView
    lastMenuDay={data?.lastMenuDay}
    isSummer={data?.isSummer}
/>
