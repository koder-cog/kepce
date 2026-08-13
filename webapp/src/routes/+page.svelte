<script>
    import { timelineState } from "@/stores/timeline.svelte.js";
    import DailyHeader from "@/components/features/timeline/DailyHeader.svelte";
    import CalendarSelector from "@/components/features/timeline/CalendarSelector.svelte";
    import TimelineView from "@/components/features/timeline/TimelineView.svelte";
    import { CITY_MAP } from "@/utils/turkish.js";
    import { onMount } from "svelte";

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
</script>

<svelte:head>
  {#if timelineState.currentCity}
    <title>{CITY_MAP[timelineState.currentCity] || timelineState.currentCity} Yurt Menüsü - Kepçe</title>
  {:else}
    <title>Yurt Menüsü - Kepçe</title>
  {/if}
</svelte:head>

<DailyHeader />
<CalendarSelector />
<TimelineView />
