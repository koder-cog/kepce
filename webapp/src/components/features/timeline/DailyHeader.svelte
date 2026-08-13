<script>
    import { timelineState } from "@/stores/timeline.svelte.js";
    import CitySelector from "@/components/features/CitySelector.svelte";
    import { icon } from "@/components/ui/icons.js";
    import { onMount } from "svelte";

    import SegmentedControl from "@/components/ui/SegmentedControl.svelte";

    const MONTHS = [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
        "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık",
    ];

    function handleDietSelect(mode) {
        timelineState.selectDietMode(mode);
    }

</script>

<div class="daily-header">
    <div class="month-nav-wrapper">
        <div class="month-nav">
            <button class="month-nav__btn" aria-label="Önceki ay" onclick={timelineState.prevMonth} disabled={!timelineState.canPrevMonth}>
                {@html icon("chevronLeft", 20)}
            </button>
            <span class="month-nav__label">{MONTHS[timelineState.viewMonth]} {timelineState.viewYear}</span>
            <button class="month-nav__btn" aria-label="Sonraki ay" onclick={timelineState.nextMonth} disabled={!timelineState.canNextMonth}>
                {@html icon("chevronRight", 20)}
            </button>
        </div>
    </div>

    <div class="header-controls">
        <SegmentedControl
            class="view-toggle"
            bind:value={timelineState.viewType}
            variant="icons"
            options={[
                { value: "timeline", icon: icon("cards", 18), tooltip: "Zaman çizelgesi" },
                { value: "calendar", icon: icon("calendar", 18), tooltip: "Takvim görünümü" }
            ]}
        />

        <div class="diet-mode-selector {timelineState.isDietVisible ? '' : 'is-hidden'}">
            <SegmentedControl
                value={timelineState.currentDietMode}
                variant="responsive"
                id="diet-mode-switcher"
                options={[
                    { value: "standard", icon: icon("utensils", 18), label: "Standart" },
                    { value: "celiac", icon: icon("wheat", 18), label: "Çölyak" }
                ]}
                onChange={(mode) => handleDietSelect(mode)}
            />
        </div>
    </div>

    <div id="city-selector-container">
        <!--
            CitySelector artık otomatik tespit ettiği şehri ve kullanıcının seçtiği
            şehri `onChange` callback'i üzerinden bildiriyor. `$bindable` + getter/setter
            kombinasyonu Svelte 5'te güvenilir olmadığı için bu açık callback yaklaşımını
            kullanıyoruz; böylece `timelineState.currentCity` güncelleniyor ve
            `loadMenus()` zinciri tetikleniyor.
        -->
        <CitySelector
            value={timelineState.currentCity}
            cities={timelineState.cities}
            onChange={(city) => { timelineState.currentCity = city; }}
        />
    </div>
</div>
