<script>
    import { timelineState } from "@/stores/timeline.svelte.js";
    import MenuCard from "@/components/features/MenuCard.svelte";
    import { icon } from "@/components/ui/icons.js";
    import { sanitizeText } from "@/utils/sanitize.js";
    import { normalizeItems } from "@/utils/menu.js";
    import { showToast } from "@/components/ui/toast.js";
    import { openBotReportModal } from "@/components/features/report-modal.js";
    import Skeleton from "@/components/ui/Skeleton.svelte";
    import EmptyState from "@/components/ui/EmptyState.svelte";
    import EmptyMenuHub from "@/components/features/timeline/EmptyMenuHub.svelte";
    import SeasonGuides from "@/components/features/timeline/SeasonGuides.svelte";
    import { isOffSeasonDate, isOrientationSeason } from "@/utils/season.js";
    import { getBotPlaceholderComment } from "@/utils/botPlaceholders.js";
    import { fade } from "svelte/transition";
    import { isMotionEnabled } from "@/lib/dom/motion.js";
    import { CITY_MAP, formatFullTurkishDate } from "@/utils/turkish.js";
    import { onMount } from "svelte";

    let {
        lastMenuDay = null,
        isSummer = false,
        isOffSeason: propIsOffSeason = undefined
    } = $props();

    // Ayarlar sayfasındaki "Boş içerik kartlarını göster" kullanıcı tercihi (varsayılan: true).
    let showEmptyCards = $state(
        typeof window !== "undefined"
            ? localStorage.getItem("kepce_show_empty_cards") !== "false"
            : true,
    );

    // Bot yorumu ham verisi (yoksa kart kompakt empty-state'e düşer)
    let botCommentaryRaw = $derived(
        (timelineState.breakfastData[0] || timelineState.dinnerData[0] || {})
            .bot_commentary,
    );

    // Runes modunda each-blok argümanına bind geçersiz; indeks bazlı
    // binding için yerel derived referanslar.
    // Render edilebilir içeriği olmayan menüler (items/dishes/foods boş ya da
    // tamamen placeholder) kart olarak çizilmez; slot wrapper'daki kompakt
    // empty-state devreye girer (empty-state yalnızca timeline__meal-wrapper içinde yaşar).
    let breakfasts = $derived(
        timelineState.breakfastData.filter((m) => normalizeItems(m).length > 0),
    );
    let dinners = $derived(
        timelineState.dinnerData.filter((m) => normalizeItems(m).length > 0),
    );

    let isOffSeason = $derived(
        propIsOffSeason !== undefined ? propIsOffSeason : isOffSeasonDate(timelineState.selectedDate)
    );
    let showSeasonGuides = $derived(isOrientationSeason(timelineState.selectedDate));

    // Tek öğün / nöbetçi modu: Sezon dışındayken ve kahvaltı yoksa (yalnızca akşam yemeği varsa)
    // dikey çizgi ve saatler kalkar, kart "Yemek" başlığıyla merkezlenir.
    let isSingleMealLayout = $derived(
        isOffSeason && breakfasts.length === 0 && dinners.length > 0,
    );

    // Çift menü varsa sezon dışı olsa bile varsayılan sezon içi görünüme (60/40 bot köşesi) döner.
    let botPlaceholder = $derived(
        breakfasts.length > 0 && showEmptyCards && !isSingleMealLayout
            ? getBotPlaceholderComment(timelineState.selectedDate)
            : null
    );

    // Sezon dışındayken kahvaltı verisi yoksa boş kart basma;
    // Sezon içinde ise kullanıcının showEmptyCards tercihine bak.
    let hideBreakfastSlot = $derived.by(() => {
        if (breakfasts.length > 0) return false;
        if (isOffSeason) return true;
        return !showEmptyCards && !botCommentaryRaw;
    });
    let hideDinnerSlot = $derived(!showEmptyCards && dinners.length === 0);

    // Eğer bot yorumu yoksa ve boş kartlar gösterilmiyorsa (veya tek menülü nöbetçi düzenindeysek), sağ taraf (bot köşesi) iptal edilir.
    let showBotArea = $derived(
        botCommentaryRaw || (showEmptyCards && !isSingleMealLayout),
    );

    function renderBotCommentary(raw, currentDate) {
        if (!raw) return "";
        try {
            if (raw.trim().startsWith("{")) {
                const data = JSON.parse(raw);
                if (data.gunler && Array.isArray(data.gunler)) {
                    const day = currentDate?.getDate();
                    const month = currentDate?.getMonth();
                    let match = null;
                    if (day && month !== undefined) {
                        match = data.gunler.find((g) => {
                            const dayMatch = g.tarih.match(/(\d+)/);
                            return dayMatch && parseInt(dayMatch[1]) === day;
                        });
                    }
                    if (!match && data.gunler.length > 0)
                        match = data.gunler[0];
                    if (match) return `<p>${sanitizeText(match.yorum)}</p>`;
                }
                if (data.yorum) return `<p>${sanitizeText(data.yorum)}</p>`;
            }
        } catch {}
        return `<p>${sanitizeText(raw)}</p>`;
    }

    let showingAlternatives = $state(null);
    let currentCityAndDate = $derived(`${timelineState.currentCity}:${timelineState.selectedDateString}`);
    let prevCityAndDate = $state("");

    function openAlternativeView(mealType, alts) {
        showingAlternatives = { mealType, alts };
        if (typeof window !== "undefined") {
            const url = new URL(window.location.href);
            url.searchParams.set("kaynak", "diger");
            if (mealType) url.searchParams.set("ogun", mealType);
            window.history.pushState({ kepceAlternativeView: true }, "", url.toString());
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    }

    function closeAlternativeView() {
        showingAlternatives = null;
        if (typeof window !== "undefined") {
            const url = new URL(window.location.href);
            url.searchParams.delete("kaynak");
            url.searchParams.delete("ogun");
            window.history.pushState({}, "", url.toString());
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    }

    $effect(() => {
        if (prevCityAndDate && currentCityAndDate !== prevCityAndDate && showingAlternatives) {
            showingAlternatives = null;
            if (typeof window !== "undefined") {
                const url = new URL(window.location.href);
                url.searchParams.delete("kaynak");
                url.searchParams.delete("ogun");
                window.history.replaceState({}, "", url.toString());
            }
        }
        prevCityAndDate = currentCityAndDate;
    });

    onMount(() => {
        const checkUrlParams = () => {
            if (typeof window === "undefined") return;
            const url = new URL(window.location.href);
            if (url.searchParams.get("kaynak") === "diger") {
                const ogun = url.searchParams.get("ogun");
                if (ogun === "breakfast" && breakfasts[0]?.alternatives?.length > 0) {
                    showingAlternatives = { mealType: "breakfast", alts: breakfasts[0].alternatives };
                } else if (ogun === "dinner" && dinners[0]?.alternatives?.length > 0) {
                    showingAlternatives = { mealType: "dinner", alts: dinners[0].alternatives };
                } else {
                    const allAlts = [
                        ...(breakfasts[0]?.alternatives || []),
                        ...(dinners[0]?.alternatives || []),
                    ];
                    if (allAlts.length > 0) {
                        showingAlternatives = { mealType: "all", alts: allAlts };
                    }
                }
            } else {
                showingAlternatives = null;
            }
        };

        checkUrlParams();
        window.addEventListener("popstate", checkUrlParams);
        return () => {
            window.removeEventListener("popstate", checkUrlParams);
        };
    });
</script>

<div id="meals-container" class:is-updating={timelineState.isUpdating}>
    {#if !timelineState.currentCity}
        <div class="empty-state-container" in:fade={{ duration: isMotionEnabled() ? 150 : 0 }}>
            <EmptyState
                iconName={"info"}
                title={"Lütfen bir şehir seçin"}
                desc={"Bugünün menüsünü görmek için önce yurdunuzun bulunduğu şehri seçmelisiniz."}
            />
        </div>
    {:else if timelineState.isLoading}
        <Skeleton type="timeline" />
    {:else if timelineState.errorState}
        <div class="empty-state-container" in:fade={{ duration: isMotionEnabled() ? 150 : 0 }}>
            <EmptyState
                statusCode={timelineState.errorState.statusCode}
                desc={timelineState.errorState.desc}
            >
                <button
                    type="button"
                    class="btn btn--secondary btn--sm btn--squish"
                    onclick={() => timelineState.reload()}
                >
                    <span>Tekrar Dene</span>
                </button>
            </EmptyState>
        </div>
    {:else if breakfasts.length === 0 && dinners.length === 0}
        <div class="timeline-empty-wrapper" in:fade={{ duration: isMotionEnabled() ? 150 : 0 }}>
            <EmptyMenuHub
                citySlug={timelineState.currentCity}
                date={timelineState.selectedDateString}
                isSummer={isSummer || isOffSeason}
                {lastMenuDay}
            />
        </div>
    {:else if timelineState.currentDietMode === "celiac" && !timelineState.breakfastData.some((m) => m.items?.length > 0) && !timelineState.dinnerData.some((m) => m.items?.length > 0)}
        <div class="empty-state-container" in:fade={{ duration: isMotionEnabled() ? 150 : 0 }}>
            <EmptyState
                iconName={"wheat"}
                title={"Bugün çölyak menüsü yok"}
                desc={"Seçtiğin tarih için herhangi bir çölyak menüsü bulamadık."}
            />
        </div>
    {:else if showingAlternatives}
        <div class="alternate-view" in:fade={{ duration: isMotionEnabled() ? 150 : 0 }}>
            <div class="alternate-view__nav">
                <button
                    type="button"
                    class="btn btn--secondary btn--sm btn--squish"
                    onclick={closeAlternativeView}
                >
                    {@html icon("chevronLeft", 16)}
                    <span>Günün Menüsüne Dön</span>
                </button>
                <span class="text-sm color-muted">
                    {formatFullTurkishDate(timelineState.selectedDate)} • {CITY_MAP[timelineState.currentCity] || timelineState.currentCity}
                </span>
            </div>

            <header class="content-page__header">
                <h2 class="content-page__title">Bazı kaynaklar böyle demektedir</h2>
                <p class="color-muted">Bu tarih için diğer kaynaklarda aşağıdaki menü listesi bildirilmiştir:</p>
            </header>

            <div class="alternate-view__stack">
                {#each showingAlternatives.alts as altMenu (altMenu.id || altMenu.source_type)}
                    <MenuCard
                        menu={altMenu}
                        options={{
                            isAlternative: true,
                            dietMode: timelineState.currentDietMode,
                        }}
                    />
                {/each}
            </div>
        </div>
    {:else}
        <div class="timeline{isSingleMealLayout ? ' timeline--off-season' : ''}" in:fade={{ duration: isMotionEnabled() ? 150 : 0 }}>
            <!-- Sezon dışı + tek menü varsa (breakfast=0, dinner>0) off-season layout devreye girer.
                 Sezon dışı olsa da breakfast varsa (çift menü) normal layout kalır. -->
            <div class="timeline__line"></div>

            <!-- Breakfast Slot -->
            <div class="timeline__slot timeline__slot--breakfast{hideBreakfastSlot ? ' timeline__slot--hidden' : ''}">
                <div class="timeline__time">
                    06:00 - {timelineState.breakfastEnd}
                </div>
                <div class="timeline__content {showBotArea ? 'timeline__content--60-40' : ''}">
                    <div class="timeline__meal-wrapper">
                        {#if breakfasts.length > 0}
                            {#each breakfasts as m, i (m.id)}
                                <MenuCard
                                    menu={breakfasts[i]}
                                    options={{
                                        dietMode: timelineState.currentDietMode,
                                        takeaways: [],
                                    }}
                                />
                                {#if breakfasts[i].alternatives?.length > 0}
                                    <div class="meal-alternate-row">
                                        <button
                                            type="button"
                                            class="meal-card__takeaway-btn"
                                            onclick={() => openAlternativeView("breakfast", breakfasts[i].alternatives)}
                                        >
                                            <span class="meal-card__dish-name">Kahvaltı için başka kaynakların dedikleri</span>
                                            <div class="meal-card__dish-actions">
                                                {@html icon("chevronRight", 18)}
                                            </div>
                                        </button>
                                    </div>
                                {/if}
                            {/each}
                        {:else if showEmptyCards && !isOffSeason}
                            <!-- #19: Kartı tamamen gizlemek yerine kompakt empty-state -->
                            <EmptyState
                                compact
                                iconName={"menuMissing"}
                                title={"Kahvaltı yok"}
                                desc={"Bu öğün için henüz menü bilgisine ulaşamadık."}
                            />
                        {/if}
                    </div>

                    {#if botCommentaryRaw}
                        <div class="bot-card ai-element">
                            <h3 class="bot-card__title">Kepçe Bot köşesi</h3>
                            <div class="bot-card__text">
                                {@html renderBotCommentary(
                                    botCommentaryRaw,
                                    timelineState.selectedDate,
                                )}
                            </div>
                            <div class="bot-card__footer">
                                <a
                                    href="/sss#kepce-bot-nedir"
                                    class="bot-card__link"
                                    data-link>Kepçe Bot nedir?</a
                                >
                                <button
                                    class="meal-card__action-btn"
                                    onclick={() => {
                                        const botCardMenu =
                                            timelineState.breakfastData[0] ||
                                            timelineState.dinnerData[0];
                                        if (botCardMenu)
                                            openBotReportModal(botCardMenu);
                                        else
                                            showToast(
                                                "Hata bildirebileceğiniz bir menü bulunamadı.",
                                                "error",
                                            );
                                    }}
                                    data-tooltip="Hata bildir"
                                    aria-label="Hata bildir"
                                >
                                    {@html icon("warning", 18)}
                                </button>
                            </div>
                        </div>
                    {:else if botPlaceholder}
                        <!-- Kahvaltı var ama bot yorumu yoksa (ve anma günü değilse), placeholder bot yorumu gösterilir -->
                        <div class="bot-card ai-element">
                            <h3 class="bot-card__title">Kepçe Bot köşesi</h3>
                            <div class="bot-card__text">
                                <p>{botPlaceholder}</p>
                            </div>
                        </div>
                    {:else if showEmptyCards && !isOffSeason && breakfasts.length === 0}
                        <!-- Kahvaltı da yok, bot yorumu da yok ve boş kartlar gizlenmiyor -->
                        <div class="bot-card ai-element">
                            <EmptyState
                                compact
                                iconName={"info"}
                                title={"Bot yorumu yok"}
                                desc={"Bu öğün için menü verisi girilmediğinden dolayı bot değerlendirmesi yapılamıyor."}
                            />
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Dinner Slot -->
            <div class="timeline__slot timeline__slot--dinner{hideDinnerSlot ? ' timeline__slot--hidden' : ''}">
                <div class="timeline__time">
                    16:00 - {timelineState.dinnerEnd}
                </div>
                <div class="timeline__content timeline__content--100">
                    <div class="timeline__meal-wrapper">
                        {#if dinners.length > 0}
                            {#each dinners as m, i (m.id)}
                                <MenuCard
                                    menu={dinners[i]}
                                    options={{
                                        dietMode: timelineState.currentDietMode,
                                        takeaways: [],
                                        isOffSeason: isSingleMealLayout,
                                    }}
                                />
                                {#if dinners[i].alternatives?.length > 0}
                                    <div class="meal-alternate-row">
                                        <button
                                            type="button"
                                            class="meal-card__takeaway-btn"
                                            onclick={() => openAlternativeView("dinner", dinners[i].alternatives)}
                                        >
                                            <span class="meal-card__dish-name">{isSingleMealLayout ? "Yemek" : "Akşam yemeği"} için başka kaynakların dedikleri</span>
                                            <div class="meal-card__dish-actions">
                                                {@html icon("chevronRight", 18)}
                                            </div>
                                        </button>
                                    </div>
                                {/if}
                            {/each}
                        {:else if showEmptyCards}
                            <EmptyState
                                compact
                                iconName={"menuMissing"}
                                title={"Akşam yemeği yok"}
                                desc={"Bu öğün için henüz menü bilgisine ulaşamadık."}
                            />
                        {/if}
                    </div>
                </div>
            </div>

        </div>

        {#if showSeasonGuides}
            <SeasonGuides />
        {/if}
    {/if}
</div>
