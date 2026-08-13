<script>
    import { onMount, onDestroy } from "svelte";
    import { timelineState } from "@/stores/timeline.svelte.js";
    import holidays from "$lib/data/holidays.json";

    const FULL_WEEKDAYS = [
        "Pazar", "Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi",
    ];
    const MID_WEEKDAYS = [
        "Pazar", "Pztsi", "Salı", "Çarş", "Perş", "Cuma", "Cmtsi",
    ];
    const WEEKDAYS = ["Paz", "Pzt", "Sal", "Çar", "Per", "Cum", "Cmt"];

    let daySelectorWrapper;
    let contentHeight = $state(0);

    const now = new Date();

    onMount(() => {
        window.addEventListener('timeline-scroll-to-active', handleScrollToActive);
        return () => {
            window.removeEventListener('timeline-scroll-to-active', handleScrollToActive);
        };
    });

    function selectDate(day) {
        timelineState.selectDate(day);
    }

    function getHolidays(year, month, day) {
        const m = String(month + 1).padStart(2, '0');
        const d = String(day).padStart(2, '0');
        const mmdd = `${m}-${d}`;
        const yyyymmdd = `${year}-${m}-${d}`;

        const dayHolidays = [];
        
        if (holidays[yyyymmdd]) {
            Array.isArray(holidays[yyyymmdd]) ? dayHolidays.push(...holidays[yyyymmdd]) : dayHolidays.push(holidays[yyyymmdd]);
        }
        if (holidays[mmdd]) {
            Array.isArray(holidays[mmdd]) ? dayHolidays.push(...holidays[mmdd]) : dayHolidays.push(holidays[mmdd]);
        }
        
        return dayHolidays;
    }

    function handleScrollToActive(e) {
        const forceCenter = e.detail?.forceCenter;
        if (!daySelectorWrapper) return;
        const activeItem = 
            daySelectorWrapper.querySelector(".day-selector__item--active") ||
            daySelectorWrapper.querySelector(".day-selector__item--today");
        if (activeItem) {
            if (forceCenter) {
                activeItem.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "center" });
            } else {
                const container = activeItem.parentElement;
                const rect = activeItem.getBoundingClientRect();
                const containerRect = container.getBoundingClientRect();
                const isVisible = rect.left >= containerRect.left && rect.right <= containerRect.right;
                if (!isVisible) {
                    activeItem.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "center" });
                }
            }
        }
    }
</script>

<div
    class="day-selector-wrapper"
    bind:this={daySelectorWrapper}
    style={contentHeight ? `--content-height: ${contentHeight}px;` : ""}
>
    <div class="day-selector-inner" bind:clientHeight={contentHeight}>
        <div class="day-selector {timelineState.viewType === 'calendar' ? 'day-selector--calendar' : ''}">
            {#if timelineState.viewType === "calendar"}
                <div class="calendar-grid">
                    {#each [1, 2, 3, 4, 5, 6, 0] as d}
                        <div class="calendar-grid__weekday">
                            <span class="u-show-desktop">{FULL_WEEKDAYS[d]}</span>
                            <span class="u-show-tablet">{MID_WEEKDAYS[d]}</span>
                            <span class="u-show-mobile">{WEEKDAYS[d]}</span>
                        </div>
                    {/each}
                    {#each Array(timelineState.firstDayOffset) as _}
                        <div class="calendar-grid__day calendar-grid__day--empty"></div>
                    {/each}
                    {#each Array(timelineState.daysInMonth) as _, i}
                        {@const day = i + 1}
                        {@const d = new Date(timelineState.viewYear, timelineState.viewMonth, day)}
                        {@const isWeekend = d.getDay() === 0 || d.getDay() === 6}
                        {@const isSelected =
                            day === timelineState.selectedDate.getDate() &&
                            timelineState.viewMonth === timelineState.selectedDate.getMonth() &&
                            timelineState.viewYear === timelineState.selectedDate.getFullYear()}
                        {@const isToday =
                            day === new Date().getDate() &&
                            timelineState.viewMonth === new Date().getMonth() &&
                            timelineState.viewYear === new Date().getFullYear()}
                        {@const dayHols = getHolidays(timelineState.viewYear, timelineState.viewMonth, day)}
                        {@const tooltipText = dayHols.length > 0 ? dayHols.map(h => h.name).join(', ') : undefined}
                        <div
                            class="calendar-grid__day {isSelected ? 'calendar-grid__day--selected' : ''} {isToday ? 'calendar-grid__day--today' : ''} {isWeekend ? 'calendar-grid__day--weekend' : ''} {dayHols.length > 0 ? 'calendar-grid__day--holiday' : ''}"
                            data-tooltip={tooltipText}
                            role="button"
                            tabindex="0"
                            onclick={() => selectDate(day)}
                            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectDate(day); } }}
                        >
                            <span class="calendar-grid__date">{day}</span>
                            {#if dayHols.length > 0 || isWeekend}
                                <div class="indicator-container">
                                    {#each dayHols as holiday}
                                        <span class="indicator-dot indicator-dot--holiday" style="--holiday-color: var(--color-{holiday.theme});"></span>
                                    {/each}
                                    {#if isWeekend}
                                        <span class="indicator-dot indicator-dot--weekend"></span>
                                    {/if}
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>
            {:else}
                {#each Array(timelineState.daysInMonth) as _, i}
                    {@const day = i + 1}
                    {@const d = new Date(timelineState.viewYear, timelineState.viewMonth, day)}
                    {@const isWeekend = d.getDay() === 0 || d.getDay() === 6}
                    {@const isSelected =
                        day === timelineState.selectedDate.getDate() &&
                        timelineState.viewMonth === timelineState.selectedDate.getMonth() &&
                        timelineState.viewYear === timelineState.selectedDate.getFullYear()}
                    {@const isToday =
                        day === now.getDate() &&
                        timelineState.viewMonth === now.getMonth() &&
                        timelineState.viewYear === now.getFullYear()}
                    {@const dayHols = getHolidays(timelineState.viewYear, timelineState.viewMonth, day)}
                    {@const tooltipText = dayHols.length > 0 ? dayHols.map(h => h.name).join(', ') : undefined}
                    <div
                        class="day-selector__item {isSelected ? 'day-selector__item--active' : ''} {isToday ? 'day-selector__item--today' : ''} {isWeekend ? 'day-selector__item--weekend' : ''} {dayHols.length > 0 ? 'day-selector__item--holiday' : ''}"
                        data-tooltip={tooltipText}
                        role="button"
                        tabindex="0"
                        onclick={() => selectDate(day)}
                        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectDate(day); } }}
                    >
                        <span class="day-selector__weekday">{WEEKDAYS[d.getDay()]}</span>
                        <span class="day-selector__date">{day}</span>
                        <div class="indicator-container">
                            {#each dayHols as holiday}
                                <span class="indicator-dot indicator-dot--holiday" style="--holiday-color: var(--color-{holiday.theme});"></span>
                            {/each}
                            {#if isWeekend}
                                <span class="indicator-dot indicator-dot--weekend"></span>
                            {/if}
                        </div>
                    </div>
                {/each}
            {/if}
        </div>
    </div>
</div>
