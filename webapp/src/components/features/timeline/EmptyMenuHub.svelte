<script>
    import { icon } from "@/components/ui/icons.js";
    import EmptyState from "@/components/ui/EmptyState.svelte";
    import { CITY_MAP, formatFullTurkishDate } from "@/utils/turkish.js";
    import { normalizeItems } from "@/utils/menu.js";
    import { timelineState } from "@/stores/timeline.svelte.js";
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
</script>

<div class="empty-hub">
    <!-- 1. Durum Bildirimi -->
    {#if isSummer}
        <EmptyState
            iconName="icecream"
            title="Yaz Sezonu"
            desc="Temmuz ve ağustos aylarında yemekhane hizmeti verilmemektedir. Nöbetçi yurtlar yerel düzenleme yapabilir."
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

            <button
                type="button"
                class="empty-hub__menu-body"
                onclick={() => timelineState.selectDate(lastMenuDay.date)}
                aria-label="{formatFullTurkishDate(lastMenuDay.date)} menüsünü görüntüle"
            >
                {#if lastBreakfast}
                    <div class="empty-hub__meal-row">
                        <span class="empty-hub__meal-label">Kahvaltı:</span>
                        <span class="empty-hub__meal-items"
                            >{lastBreakfast.itemsText}</span
                        >
                        {#if lastBreakfast.totalCount > 0}
                            <span class="empty-hub__count-badge"
                                >({lastBreakfast.totalCount} çeşit)</span
                            >
                        {/if}
                    </div>
                {/if}

                {#if lastDinner}
                    <div class="empty-hub__meal-row">
                        <span class="empty-hub__meal-label">Akşam Yemeği:</span>
                        <span class="empty-hub__meal-items"
                            >{lastDinner.itemsText}</span
                        >
                        {#if lastDinner.totalCount > 0}
                            <span class="empty-hub__count-badge"
                                >({lastDinner.totalCount} çeşit)</span
                            >
                        {/if}
                    </div>
                {/if}
            </button>

            <div class="empty-hub__archive-wrapper">
                <a
                    href="/arsiv"
                    class="btn btn--primary u-w-full btn--squish empty-hub__archive-btn"
                    data-link
                >
                    <span>Arşiv</span>
                    {@html icon("chevronRight", 18)}
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
        padding-bottom: var(--space-xs);
        border-bottom: 1px solid var(--color-border-light);
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

    .empty-hub__menu-body {
        display: flex;
        flex-direction: column;
        gap: var(--space-xs);
        padding: var(--space-sm) 0;
        text-decoration: none;
        color: var(--color-text);
        font-size: var(--text-sm);
        line-height: var(--leading-relaxed);
        border-radius: var(--radius-md);
        background: none;
        border: none;
        text-align: left;
        cursor: pointer;
        font-family: inherit;
        width: 100%;
        transition: color var(--dur-fast) var(--ease-standard);
    }

    .empty-hub__menu-body:hover {
        color: var(--color-accent-primary);
    }

    .empty-hub__menu-body:active {
        transform: scale(0.98);
    }

    .empty-hub__meal-row {
        color: inherit;
    }

    .empty-hub__meal-label {
        font-weight: var(--font-weight-bold);
        margin-right: var(--space-2xs);
    }

    .empty-hub__meal-items {
        color: inherit;
    }

    .empty-hub__count-badge {
        color: var(--color-muted);
        font-size: var(--text-xs);
        margin-left: var(--space-2xs);
    }

    .empty-hub__archive-wrapper {
        margin-top: var(--space-xs);
    }

    .empty-hub__archive-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: var(--space-xs);
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
