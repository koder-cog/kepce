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
                data.prerenderedDate
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

    let cityName = $derived(CITY_MAP[timelineState.currentCity] || timelineState.currentCity);

    let pageTitle = $derived(
        timelineState.currentCity
            ? `${cityName} KYK Yemek Menüsü | Kepçe`
            : "Kepçe | Bugün KYK'da Ne Yemek Var?",
    );

    let pageDescription = $derived(
        timelineState.currentCity
            ? `${cityName} KYK yurtlarında bugün çıkan sabah kahvaltısı ve akşam yemeği menüsü. Reklamsız, güncel yemek listeleri ve öğrenci yorumları.`
            : "Bugün KYK yurtlarında çıkan sabah kahvaltısı ve akşam yemeği menüsü. Reklamsız, güncel yemekhane listeleri ve öğrenci değerlendirmeleri.",
    );
    let ogImage = $derived(
        timelineState.currentCity
            ? `https://kepce.org/api/v1/public/og/city/${timelineState.currentCity}`
            : "https://kepce.org/og_image.png",
    );
</script>

<Seo title={pageTitle} description={pageDescription} image={ogImage} />

<h1 class="sr-only">
    {timelineState.currentCity
        ? `${CITY_MAP[timelineState.currentCity] || timelineState.currentCity} KYK Yurt Yemek Menüsü`
        : "KYK Yurt Yemek Menüsü - Günlük Yemek Listesi"}
</h1>

<DailyHeader />
<CalendarSelector />
<TimelineView />
