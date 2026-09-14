<script>
    import { icon } from "@/components/ui/icons.js";
    import EmptyState from "@/components/ui/EmptyState.svelte";
    import { CITY_MAP, formatFullTurkishDate } from "@/utils/turkish.js";
    import { normalizeItems } from "@/utils/menu.js";
    import SeasonGuides from "@/components/features/timeline/SeasonGuides.svelte";
    import { isOrientationSeason } from "@/utils/season.js";

    let {
        citySlug = "istanbul",
        date = "",
        isSummer = false,
        lastMenuDay = null,
    } = $props();

    let cityName = $derived(CITY_MAP[citySlug] || citySlug);
    let showGuides = $derived(isSummer || isOrientationSeason(date));

    // Son menü öğelerini özetle
    let lastBreakfast = $derived.by(() => {
        if (!lastMenuDay?.menus) return null;
        const b = lastMenuDay.menus.find((m) => m.meal_type === "breakfast");
        if (!b) return null;
        const items = normalizeItems(b)
            .map((i) => i.name)
            .filter(Boolean);
        return {
            itemsText:
                items.slice(0, 4).join(", ") + (items.length > 4 ? "..." : ""),
            totalCount: items.length,
        };
    });

    let lastDinner = $derived.by(() => {
        if (!lastMenuDay?.menus) return null;
        const d = lastMenuDay.menus.find(
            (m) => m.meal_type === "dinner" || m.meal_type !== "breakfast",
        );
        if (!d) return null;
        const items = normalizeItems(d)
            .map((i) => i.name)
            .filter(Boolean);
        return {
            itemsText:
                items.slice(0, 4).join(", ") + (items.length > 4 ? "..." : ""),
            totalCount: items.length,
        };
    });
    let selectedMonth = $derived.by(() => {
        if (!date) return 0;
        const d = new Date(date);
        return isNaN(d.getTime()) ? 0 : d.getMonth() + 1;
    });

    let isRealSummer = $derived(selectedMonth === 7 || selectedMonth === 8);
    let isSeptemberPreSeason = $derived(selectedMonth === 9 && isSummer);
</script>

<div class="empty-hub">
    <!-- 1. Durum Bildirimi -->
    {#if isRealSummer}
        <EmptyState
            iconName="icecream"
            title="Yaz Sezonu"
            desc="Temmuz ve ağustos aylarında yemekhane hizmeti verilmemektedir. Nöbetçi yurtlar yerel düzenleme yapabilir."
        />
    {:else if isSeptemberPreSeason}
        <EmptyState
            iconName="calendar"
            title="Yeni Dönem Hazırlığı"
            desc="KYK yurt yemekhaneleri yeni eğitim-öğretim dönemiyle birlikte hizmete açılacaktır. Nöbetçi yurtlar yerel düzenleme yapabilir."
        />
    {:else}
        <EmptyState
            iconName="ghost"
            title="Bugün bişii yok"
            desc="Seçtiğin tarih için herhangi bir menü bilgisi bulamadık. Belki de aşçı abla istifa etmiştir."
        >
            <div class="empty-hub__action">
                <a
                    href="/menu-gonder"
                    class="btn btn--primary u-w-full btn--squish"
                    data-link
                >
                    Menü gönder
                </a>
            </div>
        </EmptyState>
    {/if}

    <!-- 2. Yaz Sezonu ve Oryantasyon Rehber Kartları -->
    {#if showGuides}
        <SeasonGuides />
    {/if}

    <!-- 3. Eldeki Son Menü Kartı -->
    {#if lastMenuDay}
        <section class="card empty-hub__menu-card">
            <div class="empty-hub__menu-header">
                <h3 class="empty-hub__menu-title">Eldeki Son Menü</h3>
                <span class="empty-hub__menu-date"
                    >{formatFullTurkishDate(lastMenuDay.date)}</span
                >
            </div>

            <div class="empty-hub__meals-list">
                {#if lastBreakfast}
                    <div class="empty-hub__meal-card">
                        <div class="empty-hub__meal-head">
                            <span class="empty-hub__meal-badge">Kahvaltı</span>
                            {#if lastBreakfast.totalCount > 0}
                                <span class="empty-hub__count-badge"
                                    >({lastBreakfast.totalCount} çeşit)</span
                                >
                            {/if}
                        </div>
                        <div class="empty-hub__meal-content">
                            <span class="empty-hub__meal-items"
                                >{lastBreakfast.itemsText}</span
                            >
                        </div>
                    </div>
                {/if}

                {#if lastDinner}
                    <div class="empty-hub__meal-card">
                        <div class="empty-hub__meal-head">
                            <span class="empty-hub__meal-badge">Akşam Yemeği</span>
                            {#if lastDinner.totalCount > 0}
                                <span class="empty-hub__count-badge"
                                    >({lastDinner.totalCount} çeşit)</span
                                >
                            {/if}
                        </div>
                        <div class="empty-hub__meal-content">
                            <span class="empty-hub__meal-items"
                                >{lastDinner.itemsText}</span
                            >
                        </div>
                    </div>
                {/if}
            </div>

            <div class="empty-hub__archive-wrapper">
                <a
                    href="/arsiv"
                    class="btn--row-action empty-hub__archive-btn"
                    data-link
                >
                    <span class="btn--row-action__label">Arşiv</span>
                    <div class="btn--row-action__icon">
                        {@html icon("chevronRight", 18)}
                    </div>
                </a>
            </div>
        </section>
    {/if}
</div>

<style>
    .empty-hub {
        display: flex;
        flex-direction: column;
        gap: var(--space-lg);
        max-width: 680px;
        margin: var(--space-md) auto var(--space-2xl);
        width: 100%;
    }

    /* ── Action ─────────────────────────────────────────────── */
    .empty-hub__action {
        width: 100%;
        max-width: 360px;
        margin: var(--space-md) auto 0;
    }

    /* ── Eldeki Son Menü Kartı ───────────────────────────────── */
    .empty-hub__menu-card {
        display: flex;
        flex-direction: column;
        gap: var(--space-md);
        padding: var(--space-xl);
    }

    .empty-hub__menu-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .empty-hub__menu-title {
        font-family: var(--font-display);
        font-size: var(--text-lg);
        color: var(--color-text);
        margin: 0;
        letter-spacing: -0.01em;
    }

    .empty-hub__menu-date {
        font-size: var(--text-xs);
        font-weight: var(--font-weight-semibold);
        color: var(--color-muted);
    }

    .empty-hub__meals-list {
        display: flex;
        flex-direction: column;
        gap: var(--space-sm);
    }

    .empty-hub__meal-card {
        background: var(--color-surface-sunken);
        border: 1px solid var(--color-border-light);
        border-radius: var(--radius-md);
        padding: var(--space-md);
        display: flex;
        flex-direction: column;
        gap: var(--space-xs);
    }

    .empty-hub__meal-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-sm);
    }

    .empty-hub__meal-badge {
        font-family: var(--font-display);
        font-size: var(--text-sm);
        font-weight: var(--font-weight-bold);
        color: var(--color-text);
        letter-spacing: -0.01em;
    }

    .empty-hub__count-badge {
        font-size: var(--text-xs);
        color: var(--color-muted);
        font-weight: var(--font-weight-medium);
    }

    .empty-hub__meal-content {
        font-size: var(--text-sm);
        line-height: var(--leading-relaxed);
        color: var(--color-text-secondary);
    }

    .empty-hub__meal-items {
        color: inherit;
        word-break: break-word;
    }

    .empty-hub__archive-wrapper {
        padding-top: var(--space-sm);
        border-top: 1px dashed var(--color-border-light);
    }

    @media (max-width: 600px) {
        .empty-hub {
            gap: var(--space-md);
        }

        .empty-hub__menu-card {
            padding: var(--space-md);
        }
    }
</style>
