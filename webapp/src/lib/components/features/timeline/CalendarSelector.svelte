<script>
  import { onMount, onDestroy } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { isMotionEnabled } from '$lib/dom/motion.js';
  import { timelineState } from '$lib/stores/timeline.svelte.js';
  import { getHolidayInfo } from '$lib/domain/calendar/holidays.js';
  import { icon } from '$lib/components/ui/icons.js';

  const FULL_WEEKDAYS = ['Pazar', 'Pazartesi', 'Salı', 'Çarşamba', 'Perşembe', 'Cuma', 'Cumartesi'];
  const MID_WEEKDAYS = ['Pazar', 'Pztsi', 'Salı', 'Çarş', 'Perş', 'Cuma', 'Cmtsi'];
  const WEEKDAYS = ['Paz', 'Pzt', 'Sal', 'Çar', 'Per', 'Cum', 'Cmt'];

  let daySelectorWrapper = $state(null);
  let contentHeight = $state(0);

  const now = new Date();

  onMount(() => {
    window.addEventListener('timeline-scroll-to-active', handleScrollToActive);
    return () => {
      window.removeEventListener('timeline-scroll-to-active', handleScrollToActive);
    };
  });

  // Takvim seçimi yalnızca client-side filtre; URL değişmez. Gün sayfalarına
  // yalnızca link üzerinden gidilir (/[sehir]/[tarih] menüsüz günde 404).
  function selectDate(day, e) {
    if (e && (e.ctrlKey || e.metaKey || e.shiftKey)) return;
    if (e) e.preventDefault();
    timelineState.selectDate(day);
  }

  function getDayHref(day) {
    const city = timelineState.currentCity || 'istanbul';
    const m = String(timelineState.viewMonth + 1).padStart(2, '0');
    const d = String(day).padStart(2, '0');
    return `/${city}?gun=${timelineState.viewYear}-${m}-${d}`;
  }

  function getHolidayIcon(theme, tone) {
    if (theme === 'ramadan' || theme === 'sacrifice' || tone === 'religious') {
      return 'moon';
    }
    if (theme === 'national' || tone === 'national') {
      return 'starFilled';
    }
    if (theme === 'new-year') {
      return 'starFilled';
    }
    if (theme === 'solemn' || tone === 'solemn') {
      return 'leafFall';
    }
    return 'starFilled';
  }

  function getHolidays(year, month, day) {
    const m = String(month + 1).padStart(2, '0');
    const d = String(day).padStart(2, '0');
    const yyyymmdd = `${year}-${m}-${d}`;
    const info = getHolidayInfo(yyyymmdd);
    return info ? [info] : [];
  }

  function handleScrollToActive(e) {
    if (timelineState.viewType === 'calendar') return;
    const forceCenter = e.detail?.forceCenter;
    if (!daySelectorWrapper) return;

    requestAnimationFrame(() => {
      if (!daySelectorWrapper) return;

      const activeItem =
        daySelectorWrapper.querySelector('.day-selector__item--active') ||
        daySelectorWrapper.querySelector('.day-selector__item--today');

      if (activeItem) {
        const container = activeItem.parentElement;
        if (!container) return;

        const behavior = isMotionEnabled() ? 'smooth' : 'auto';
        const rect = activeItem.getBoundingClientRect();
        const containerRect = container.getBoundingClientRect();
        const isVisible = rect.left >= containerRect.left && rect.right <= containerRect.right;

        if (forceCenter || !isVisible) {
          const targetLeft =
            activeItem.offsetLeft - container.clientWidth / 2 + activeItem.offsetWidth / 2;
          container.scrollTo({ left: targetLeft, behavior });
        }
      }
    });
  }

  function getIstanbulTodayString() {
    return new Intl.DateTimeFormat('en-CA', {
      timeZone: 'Europe/Istanbul',
      year: 'numeric',
      month: '2-digit',
      day: '2-digit'
    }).format(new Date());
  }

  let showHolidayBanner = $state(
    typeof window !== 'undefined'
      ? localStorage.getItem('kepce_show_holiday_banner') !== 'false'
      : true
  );

  onMount(() => {
    window.addEventListener('timeline-scroll-to-active', handleScrollToActive);
    const handleSettingChange = (e) => {
      showHolidayBanner = e.detail?.show ?? true;
    };
    window.addEventListener('holiday-banner-setting-changed', handleSettingChange);
    return () => {
      window.removeEventListener('timeline-scroll-to-active', handleScrollToActive);
      window.removeEventListener('holiday-banner-setting-changed', handleSettingChange);
    };
  });

  let todayString = $derived(
    timelineState.serverToday || getIstanbulTodayString()
  );
  let todayHolidayInfo = $derived(
    todayString ? getHolidayInfo(todayString) : null
  );
</script>

<div
  class="day-selector-wrapper"
  class:is-updating={timelineState.isUpdating}
  bind:this={daySelectorWrapper}
  style={contentHeight ? `--content-height: ${contentHeight}px;` : ''}
>
  {#key `${timelineState.viewYear}-${timelineState.viewMonth}`}
    <div
      class="day-selector-inner"
      bind:clientHeight={contentHeight}
      in:fly={{
        x: isMotionEnabled() ? timelineState.monthNavDirection * 24 : 0,
        duration: isMotionEnabled() ? 180 : 0,
        easing: cubicOut
      }}
    >
      {#key timelineState.viewType}
        <div
          class="day-selector {timelineState.viewType === 'calendar'
            ? 'day-selector--calendar'
            : ''}"
          in:fade={{ duration: isMotionEnabled() ? 130 : 0 }}
        >
          {#if timelineState.viewType === 'calendar'}
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
                {@const isToday = timelineState.serverTodayParts
                  ? day === timelineState.serverTodayParts.day &&
                    timelineState.viewMonth === timelineState.serverTodayParts.month &&
                    timelineState.viewYear === timelineState.serverTodayParts.year
                  : day === now.getDate() &&
                    timelineState.viewMonth === now.getMonth() &&
                    timelineState.viewYear === now.getFullYear()}
                {@const dayHols = getHolidays(timelineState.viewYear, timelineState.viewMonth, day)}
                {@const tooltipText =
                  dayHols.length > 0 ? dayHols.map((h) => h.name).join(', ') : undefined}
                <a
                  href={getDayHref(day)}
                  data-link
                  class="calendar-grid__day {isSelected
                    ? 'calendar-grid__day--selected'
                    : ''} {isToday ? 'calendar-grid__day--today' : ''} {isWeekend
                    ? 'calendar-grid__day--weekend'
                    : ''} {dayHols.length > 0 ? 'calendar-grid__day--holiday' : ''}"
                  data-tooltip={tooltipText}
                  onclick={(e) => selectDate(day, e)}
                  onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      selectDate(day, e);
                    }
                  }}
                >
                  <span class="calendar-grid__date">{day}</span>
                  {#if dayHols.length > 0 || isWeekend}
                    <div class="indicator-container">
                      {#each dayHols as holiday}
                        <span
                          class="indicator-dot indicator-dot--holiday"
                          style="--holiday-color: var(--color-holiday-{holiday.tone}-border);"
                        ></span>
                      {/each}
                      {#if isWeekend}
                        <span class="indicator-dot indicator-dot--weekend"></span>
                      {/if}
                    </div>
                  {/if}
                </a>
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
              {@const isToday = timelineState.serverTodayParts
                ? day === timelineState.serverTodayParts.day &&
                  timelineState.viewMonth === timelineState.serverTodayParts.month &&
                  timelineState.viewYear === timelineState.serverTodayParts.year
                : day === now.getDate() &&
                  timelineState.viewMonth === now.getMonth() &&
                  timelineState.viewYear === now.getFullYear()}
              {@const dayHols = getHolidays(timelineState.viewYear, timelineState.viewMonth, day)}
              {@const tooltipText =
                dayHols.length > 0 ? dayHols.map((h) => h.name).join(', ') : undefined}
              <a
                href={getDayHref(day)}
                data-link
                class="day-selector__item {isSelected ? 'day-selector__item--active' : ''} {isToday
                  ? 'day-selector__item--today'
                  : ''} {isWeekend ? 'day-selector__item--weekend' : ''} {dayHols.length > 0
                  ? 'day-selector__item--holiday'
                  : ''}"
                data-tooltip={tooltipText}
                onclick={(e) => selectDate(day, e)}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    selectDate(day, e);
                  }
                }}
              >
                <span class="day-selector__weekday">{WEEKDAYS[d.getDay()]}</span>
                <span class="day-selector__date">{day}</span>
                <div class="indicator-container">
                  {#each dayHols as holiday}
                    <span
                      class="indicator-dot indicator-dot--holiday"
                      style="--holiday-color: var(--color-holiday-{holiday.tone}-border);"
                    ></span>
                  {/each}
                  {#if isWeekend}
                    <span class="indicator-dot indicator-dot--weekend"></span>
                  {/if}
                </div>
              </a>
            {/each}
          {/if}
        </div>
      {/key}

      {#if todayHolidayInfo && showHolidayBanner}
        {@const holidayIcon = getHolidayIcon(todayHolidayInfo.theme, todayHolidayInfo.tone)}
        <aside
          class="day-selector__holiday-banner day-selector__holiday-banner--{todayHolidayInfo.theme} day-selector__holiday-banner--tone-{todayHolidayInfo.tone}"
          aria-label={todayHolidayInfo.name}
        >
          {#if holidayIcon}
            <span class="day-selector__holiday-icon" aria-hidden="true">
              {@html icon(holidayIcon, 20)}
            </span>
          {/if}
          <p class="day-selector__holiday-text">
            {todayHolidayInfo.text}
          </p>
        </aside>
      {/if}
    </div>
  {/key}
</div>
