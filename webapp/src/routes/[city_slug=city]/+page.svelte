<script>
    import { timelineState } from "@/stores/timeline.svelte.js";
    import { setCurrentCity } from "@/stores/city.svelte.js";
    import DailyHeader from "@/components/features/timeline/DailyHeader.svelte";
    import CalendarSelector from "@/components/features/timeline/CalendarSelector.svelte";
    import TimelineView from "@/components/features/timeline/TimelineView.svelte";
    import { CITY_MAP } from "@/utils/turkish.js";
    import Seo from "@/components/ui/Seo.svelte";
    import { onMount } from "svelte";

    let { data } = $props();

    let citySlug = $derived(data?.citySlug || "istanbul");
    let cityName = $derived(CITY_MAP[citySlug] || citySlug);

    onMount(() => {
        if (citySlug) {
            setCurrentCity(citySlug);
        }
        timelineState.init();
    });

    let pageTitle = $derived(`${cityName} KYK Yemek Menüsü | Kepçe`);
    let pageDescription = $derived(
        `${cityName} KYK yurtlarında bugün çıkan sabah kahvaltısı ve akşam yemeği menüsü. Reklamsız, güncel yemek listeleri ve öğrenci yorumları.`
    );
    let canonicalUrl = $derived(`https://kepce.org/${citySlug}`);
    let ogImage = $derived(`https://kepce.org/api/v1/public/og/city/${citySlug}`);

    let menuSchema = $derived.by(() => {
        const menus = timelineState.menusState || [];
        if (!menus || menus.length === 0) return null;

        const sections = menus.map((m) => {
            const dishes = m.items || m.dishes || [];
            const dishNames = dishes.map((d) => (typeof d === "string" ? d : d.name)).filter(Boolean);
            const mealTitle = m.meal_type === "breakfast" ? "Sabah Kahvaltısı" : "Akşam Yemeği";
            return {
                "@type": "MenuSection",
                name: mealTitle,
                hasMenuItem: dishNames.map((name) => ({
                    "@type": "MenuItem",
                    name: name,
                })),
            };
        });

        return {
            "@context": "https://schema.org",
            "@graph": [
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
                {
                    "@type": "Menu",
                    "@id": `https://kepce.org/${citySlug}#menu`,
                    name: `${cityName} KYK Günlük Yemek Menüsü`,
                    inLanguage: "tr-TR",
                    hasMenuSection: sections,
                },
            ],
        };
    });
</script>

<Seo
    title={pageTitle}
    description={pageDescription}
    image={ogImage}
    canonical={canonicalUrl}
    schema={menuSchema}
/>

<h1 class="sr-only">
    {cityName} KYK Yurt Yemek Menüsü - Günlük Yemek Listesi
</h1>

<DailyHeader />
<CalendarSelector />
<TimelineView />
