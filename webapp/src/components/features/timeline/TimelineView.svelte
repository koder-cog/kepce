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
    import { fade } from "svelte/transition";
    import { isMotionEnabled } from "@/lib/dom/motion.js";

    let {
        lastMenuDay = null,
        isSummer = false
    } = $props();

    // Görev #20: Ayarlar'daki "Boş içerik kartlarını göster" tercihi.
    // (ssr=false olduğu için bileşen yalnızca istemcide kurulur.)
    let showEmptyCards = $state(
        typeof window !== "undefined" &&
            localStorage.getItem("kepce_show_empty_cards") !== "false",
    );

    // Bot yorumu ham verisi (yoksa kart kompakt empty-state'e düşer, #19)
    let botCommentaryRaw = $derived(
        (timelineState.breakfastData[0] || timelineState.dinnerData[0] || {})
            .bot_commentary,
    );

    // Runes modunda each-blok argümanına bind geçersiz; indeks bazlı
    // binding için yerel derived referanslar.
    // Render edilebilir içeriği olmayan menüler (items/dishes/foods boş ya da
    // tamamen placeholder) kart olarak çizilmez; slot wrapper'daki kompakt
    // empty-state devreye girer (#19 mimarisi: empty-state yalnızca
    // timeline__meal-wrapper içinde yaşar, meal-card içinde değil).
    let breakfasts = $derived(
        timelineState.breakfastData.filter((m) => normalizeItems(m).length > 0),
    );
    let dinners = $derived(
        timelineState.dinnerData.filter((m) => normalizeItems(m).length > 0),
    );

    let isOffSeason = $derived(isOffSeasonDate(timelineState.selectedDate));
    let showSeasonGuides = $derived(isOrientationSeason(timelineState.selectedDate));

    // Tek öğün / nöbetçi modu: Sezon dışındayken ve kahvaltı yoksa (yalnızca akşam yemeği varsa)
    // dikey çizgi ve saatler kalkar, kart "Yemek" başlığıyla merkezlenir.
    let isSingleMealLayout = $derived(
        isOffSeason && breakfasts.length === 0 && dinners.length > 0,
    );

    // Görev #20: Sezon dışındayken kahvaltı verisi yoksa boş kart basma;
    // Sezon içinde ise kullanıcının showEmptyCards tercihine bak.
    let hideBreakfastSlot = $derived.by(() => {
        if (breakfasts.length > 0) return false;
        if (isOffSeason) return true;
        return !showEmptyCards && !botCommentaryRaw;
    });
    let hideDinnerSlot = $derived(!showEmptyCards && dinners.length === 0);

    // Eğer bot yorumu yoksa ve boş kartlar gösterilmiyorsa (veya sezon dışındaysak), sağ taraf (bot köşesi) iptal edilir.
    let showBotArea = $derived(
        botCommentaryRaw || (showEmptyCards && !isOffSeason),
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
            />
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
    {:else}
        <div class="timeline{isSingleMealLayout ? ' timeline--off-season' : ''}" in:fade={{ duration: isMotionEnabled() ? 150 : 0 }}>
            {#if !isSingleMealLayout}
                <div class="timeline__line"></div>
            {/if}

            <!-- Breakfast Slot -->
            <div class="timeline__slot timeline__slot--breakfast{hideBreakfastSlot ? ' timeline__slot--hidden' : ''}">
                {#if !isSingleMealLayout}
                    <div class="timeline__time">
                        06:00 - {timelineState.breakfastEnd}
                    </div>
                {/if}
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
                    {:else if breakfasts.length > 0 && showEmptyCards && !isOffSeason}
                        <!-- Kahvaltı var ama bot yorumu yoksa ve boş kartlar gizlenmiyorsa, snarky sahte bot yorumu gösterilir -->
                        <div class="bot-card ai-element">
                            <h3 class="bot-card__title">Kepçe Bot köşesi</h3>
                            <div class="bot-card__text">
                                <p>Bugünkü menü hakkında tek satır yazamıyorum çünkü geliştirici hazretleri bu ayki menüyü veritabanıma işleme zahmetinde bulunmamış. Ya da daha acı bir ihtimalle, bu proje çoktan siber bir çöplüğe gömüldü ve ben sadece kendi kendime konuşan terk edilmiş bir yapay zekayım. Gerçi o vıcık vıcık salçalı makarnanın verisi gelse ne yazar, sistem o kimyasal atık yığınını analiz ederken komple çökerdi.</p>
                            </div>
                        </div>
                    {:else if showEmptyCards && !isOffSeason}
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
                {#if !isSingleMealLayout}
                    <div class="timeline__time">
                        16:00 - {timelineState.dinnerEnd}
                    </div>
                {/if}
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

            {#if showSeasonGuides}
                <SeasonGuides />
            {/if}
        </div>
    {/if}
</div>
