<script>
    import { globalState, authActions } from "../../state.svelte.js";

    import { icon } from "../ui/icons.js";
    import { sanitizeText } from "../../utils/sanitize.js";
    import { groupItems, normalizeItems } from "../../utils/menu.js";
    import { api } from "../../api/index.js";
    import { showToast } from "../ui/toast.js";
    import { openReportModal, openMenuReportModal } from "./report-modal.js";
    import { openTakeawayModal } from "../../lib/dom/takeaway-modal.js";
    import { getCurrentCity } from "../../stores/city.svelte.js";

    let { menu = $bindable(), options = {} } = $props();

    let pendingFavorites = new Set();

    let dietMode = $derived(options.dietMode || "standard");
    let hideComment = $derived(options.hideComment || false);
    let takeaways = $derived(options.takeaways || []);

    // Yorum aksiyonu doğrudan menünün yorum akışına gider (/menu/[id]);
    // gün sayfası zaten tüm öğünleri tek yerde sunar.
    let commentUrl = $derived(menu?.id ? `/menu/${menu.id}` : null);

    let isBreakfast = $derived(menu.meal_type === "breakfast");
    let title = $derived(
        options.title ||
            (isBreakfast
                ? "Kahvaltı"
                : options.isOffSeason
                  ? "Yemek"
                  : "Akşam yemeği"),
    );

    // Kart altındaki tek satırlık kalori bilgisi: menüye ait VERİLEN ARALIK
    // esas alınır, uçlar eşitse "a - a" yerine "~a".
    let calorieText = $derived.by(() => {
        const min = menu.calorie_range_min;
        const max = menu.calorie_range_max;
        if (min && max) {
            return Number(min) === Number(max)
                ? `~${min} kcal`
                : `${min} - ${max} kcal`;
        }
        if (min) return `${min} kcal`;
        if (max) return `${max} kcal`;
        if (menu.calorie_range) return menu.calorie_range;
        if (menu.calculated_calories) return `~${menu.calculated_calories} kcal`;
        if (menu.total_calories) return `${sanitizeText(menu.total_calories)} kcal`;
        return "";
    });

    let ratingSum = $derived(menu.rating_sum || 0);
    let voteCount = $derived(menu.vote_count || 0);
    let myVote = $derived(menu.my_vote);

    let scoreClass = $derived(
        myVote === "positive"
            ? "positive"
            : myVote === "negative"
              ? "negative"
              : ratingSum > 0
                ? "positive"
                : ratingSum < 0
                  ? "negative"
                  : "",
    );

    const sourceMap = {
        kykyemek: {
            label: "Kaynak: Kykyemek.com",
            icon: "info",
            class: "disclaimer",
        },
        "kykyemek.com": {
            label: "Kaynak: Kykyemek.com",
            icon: "info",
            class: "disclaimer",
        },
        "kyk-yemek": {
            label: "Kaynak: Kykyemek.com",
            icon: "info",
            class: "disclaimer",
        },
        yurtmenu: {
            label: "Kaynak: Yurt Menü",
            icon: "info",
            class: "disclaimer",
        },
        "yurtmenu.net": {
            label: "Kaynak: Yurt Menü",
            icon: "info",
            class: "disclaimer",
        },
        kykmenu: {
            label: "Kaynak: Menü",
            icon: "info",
            class: "disclaimer",
        },
        "kykmenu.com.tr": {
            label: "Kaynak: Menü",
            icon: "info",
            class: "disclaimer",
        },
        "kykmenulistesi.com.tr": {
            label: "Kaynak: Menü",
            icon: "info",
            class: "disclaimer",
        },
        kepce: { label: "Kaynak: Kepçe", icon: "verified", class: "positive" },
        "kepce-admin": {
            label: "Kaynak: Kepçe",
            icon: "verified",
            class: "positive",
        },
        "kepce-kullanici": {
            label: "Kaynak: Kepçe",
            icon: "verified",
            class: "positive",
        },
        "kepce-anonim": {
            label: "Kaynak: Kepçe (Anonim)",
            icon: "info",
            class: "disclaimer",
        },
        anonim: {
            label: "Kaynak: Kepçe (Anonim)",
            icon: "info",
            class: "disclaimer",
        },
        unknown: {
            label: "Kaynak: Bilinmiyor",
            icon: "info-critical",
            class: "disclaimer",
        },
    };

    let isVerified = $derived(
        menu.verified === true ||
            menu.source_type === "kepce-admin" ||
            menu.source_type === "kepce-kullanici",
    );
    let sourceConfig = $derived(
        sourceMap[menu.source_type] ||
            (isVerified ? sourceMap["kepce"] : sourceMap["unknown"]),
    );
    let sourceLabel = $derived(sourceConfig.label);
    let sourceIcon = $derived(sourceConfig.icon);
    let sourceClass = $derived(`meal-card__source--${sourceConfig.class}`);

    let richTooltip = $derived(
        (() => {
            let nameObj = menu.meal_type === "breakfast" ? "kahvaltı" : "menü";
            let nameGenitive =
                menu.meal_type === "breakfast" ? "kahvaltının" : "menünün";
            if (
                menu.source_type === "kepce-admin" ||
                menu.source_type === "kepce" ||
                (!menu.source_type && menu.verified)
            ) {
                return `Bu ${nameObj}, Kepçe ekibinin (otomasyon) el emeği göz nurudur. Lakin yurdunuzun planları ve aşçının o günkü psikolojisi yüzünden tabağınızda başka bir şeyle karşılaşma ihtimali de mevcuttur.`;
            } else if (menu.source_type === "kepce-kullanici") {
                return `Bu ${nameObj}, KYK’nin derinliklerinden bilgi sızdıran isimsiz bir cengaverin yolladığı istihbarat ışığında, moderasyon ekibimiz tarafından deşifre edilip önünüze atılmıştır. Adeta bir KYK Leaks vakası.`;
            } else if (
                menu.source_type === "kykyemek" ||
                menu.source_type === "kykyemek.com" ||
                menu.source_type === "kyk-yemek" ||
                menu.source_type === "yurtmenu" ||
                menu.source_type === "yurtmenu.net" ||
                menu.source_type === "kykmenu" ||
                menu.source_type === "kykmenu.com.tr" ||
                menu.source_type === "kykmenulistesi.com.tr"
            ) {
                return `Bu ${nameObj}, harici platformlardan bot marifetiyle devşirilmiştir. Mutfağa bizzat sızamadığımız için tutarlılık garantisi veremiyoruz; menü tutarsa mucize tutmazsa fıtrat.`;
            } else if (
                menu.source_type === "kepce-anonim" ||
                menu.source_type === "anonim"
            ) {
                return `Bu ${nameObj}, sisteme giriş yapmamış bir Kepçe kullanıcısı tarafından bildirilmiştir. Bilgiye güvenmek istiyoruz ama mutfakta her an her şey yaşanabilir, temkini elden bırakmayın.`;
            } else {
                return `Bu ${nameGenitive} nereden geldiği, kimin hazırladığı veya bizim sisteme nasıl düştüğü hakkında en ufak bir fikrimiz yok. Muhtemelen deponun karanlık köşelerinde unutulan 3 yıllık salçaların hüznüyle kendi kendine spawn olmuş, boyut kapısı açılarak tepsinize düşmüş kozmik bir tabldot.`;
            }
        })(),
    );

    let forceUpdate = $state(0);
    let items = $derived.by(() => {
        forceUpdate;
        return groupItems(normalizeItems(menu));
    });
    let currentTakeaways = $derived(menu.takeaways || takeaways || []);

    async function handleVote(sentiment) {
        if (!globalState?.user) {
            authActions.triggerLogin();
            return;
        }

        if (!globalState?.user?.is_verified) {
            showToast("Oy verebilmek için e-postanızı onaylamalısınız.", {
                type: "warning",
            });
            return;
        }

        const type = sentiment === "positive" ? "up" : "down";
        const isRemoving = menu.my_vote === sentiment;
        const oppositeSentiment =
            sentiment === "positive" ? "negative" : "positive";
        const isFlipping = menu.my_vote === oppositeSentiment;

        let delta = 0;
        let voteDelta = 0;

        if (isRemoving) {
            delta = sentiment === "positive" ? -1 : 1;
            voteDelta = -1;
        } else if (isFlipping) {
            delta = sentiment === "positive" ? 2 : -2;
            voteDelta = 0;
        } else {
            delta = sentiment === "positive" ? 1 : -1;
            voteDelta = 1;
        }

        const oldRatingSum = menu.rating_sum;
        const oldVoteCount = menu.vote_count;
        const oldMyVote = menu.my_vote;

        // Optimistic UI
        menu.rating_sum = (menu.rating_sum || 0) + delta;
        menu.vote_count = (menu.vote_count || 0) + voteDelta;
        menu.my_vote = isRemoving ? null : sentiment;

        try {
            await api.voteMenu(menu.id, isRemoving ? "neutral" : sentiment);
        } catch (err) {
            showToast(err.message, "error");
            menu.rating_sum = oldRatingSum;
            menu.vote_count = oldVoteCount;
            menu.my_vote = oldMyVote;
        }
    }

    function handleTakeawayClick(takeawayMenu, takeawayLabel) {
        openTakeawayModal({
            takeawayMenu,
            takeawayId: takeawayMenu.id,
            takeawayLabel,
            currentCity: getCurrentCity(),
        });
    }

    function isFav(dish) {
        if (!dish || typeof dish.id !== "number") return false;
        return (
            (globalState.favorites &&
                globalState.favorites.includes(dish.id)) ||
            !!dish.my_favorite
        );
    }

    async function handleFavorite(dish) {
        if (!globalState?.user) {
            authActions.triggerLogin();
            return;
        }

        const dishId = dish.id;
        if (typeof dishId !== "number") return;
        if (pendingFavorites.has(dishId)) return;

        pendingFavorites.add(dishId);

        const currentlyFav = isFav(dish);

        // Optimistic update of globalState.favorites
        if (currentlyFav) {
            globalState.favorites = (globalState.favorites || []).filter(
                (id) => id !== dishId,
            );
            dish.my_favorite = false;
        } else {
            if (!globalState.favorites.includes(dishId)) {
                globalState.favorites = [
                    ...(globalState.favorites || []),
                    dishId,
                ];
            }
            dish.my_favorite = true;
        }

        if (menu.items) {
            const sourceItem = menu.items.find(
                (i) =>
                    (i.master_data ? i.master_data.dish_id : null) === dishId,
            );
            if (sourceItem) sourceItem.my_favorite = !currentlyFav;
        }
        if (menu.dishes) {
            const sourceDish = menu.dishes.find((d) => d.id === dishId);
            if (sourceDish) sourceDish.my_favorite = !currentlyFav;
        }
        forceUpdate++; // Trigger UI reactivity

        try {
            await api.toggleFavorite(dishId);
        } catch (err) {
            showToast(err.message, "error");
            // Revert on failure
            if (currentlyFav) {
                if (!globalState.favorites.includes(dishId)) {
                    globalState.favorites = [
                        ...(globalState.favorites || []),
                        dishId,
                    ];
                }
                dish.my_favorite = true;
            } else {
                globalState.favorites = (globalState.favorites || []).filter(
                    (id) => id !== dishId,
                );
                dish.my_favorite = false;
            }
            if (menu.items) {
                const sourceItem = menu.items.find(
                    (i) =>
                        (i.master_data ? i.master_data.dish_id : null) ===
                        dishId,
                );
                if (sourceItem) sourceItem.my_favorite = currentlyFav;
            }
            if (menu.dishes) {
                const sourceDish = menu.dishes.find((d) => d.id === dishId);
                if (sourceDish) sourceDish.my_favorite = currentlyFav;
            }
            forceUpdate++;
        } finally {
            pendingFavorites.delete(dishId);
        }
    }
</script>

<div class="meal-card" id="meal-card-{menu.id}">
    <div class="meal-card__header">
        <h2 class="meal-card__title">{title}</h2>
        <div class="meal-card__source-wrapper">
            <div
                class="meal-card__source {sourceClass}"
                data-tooltip={richTooltip}
            >
                {@html icon(sourceIcon, 14)}
                <span class="meal-card__source-text">{sourceLabel}</span>
            </div>
        </div>
    </div>

    <div class="meal-card__items">
        <!-- NOT: Boş menü durumu kart içinde render edilmez. İçeriği olmayan
             menüler TimelineView tarafından filtrelenir; empty-state yalnızca
             timeline__meal-wrapper içindeki kompakt EmptyState olarak çizilir
             (#19 mimarisi). -->
        {#if items.length > 0}
            {#each items as item}
                <!-- Master verisi olmayan (raw) yemekler için `raw-<id>` öneki
                     yerine doğrudan `item.id` (veya undefined) kullanıyoruz; hem
                     `data-dish-id` eşleşmesini hem de CSS hook'larını bozmuyor.
                     `raw-` öneki tırnak içinde string yaptığı için `typeof` kontrolü
                     de yanlış sonuç veriyordu. -->
                {@const dishes =
                    item.dishes && item.dishes.length > 0
                        ? item.dishes
                        : [{ id: item.id, name: item.name }]}
                {@const isAlternative = dishes.length > 1}

                <div
                    class="meal-card__item-row {isAlternative
                        ? 'meal-card__item-row--alternative'
                        : ''}"
                >
                    {#each dishes as dish, idx}
                        {#if idx > 0}
                            <div class="meal-card__dish-separator--yada">
                                <span class="meal-card__dish-separator-text"
                                    >- ya da -</span
                                >
                            </div>
                        {/if}
                        <div class="meal-card__item">
                            <div
                                class="meal-card__dish-part"
                                data-dish-id={dish.id}
                            >
                                <div class="meal-card__dish-info-wrapper">
                                    <span class="meal-card__dish-name"
                                        >{sanitizeText(dish.name)}</span
                                    >
                                    {#if dish.weight || dish.calories || dish.estimated_calories}
                                        <div
                                            class="text-xs color-muted weight-info"
                                        >
                                            {#if dish.weight}{sanitizeText(
                                                    dish.weight,
                                                )}{/if}
                                            {#if dish.weight && (dish.calories || dish.estimated_calories)}
                                                &bull;
                                            {/if}
                                            {#if dish.calories || dish.estimated_calories}{dish.calories ||
                                                    dish.estimated_calories} kcal{/if}
                                        </div>
                                    {/if}
                                </div>

                                {#if dish.price || typeof dish.id === "number" || dish.is_vegan || dish.is_vegetarian || dish.is_celiac}
                                    <div class="meal-card__dish-actions">
                                        {#if dish.is_celiac}
                                            <div
                                                class="meal-card__diet-icon"
                                                data-tooltip="Çölyak"
                                            >
                                                {@html icon("wheat", 18)}
                                            </div>
                                        {/if}
                                        {#if dish.is_vegan}
                                            <div
                                                class="meal-card__diet-icon"
                                                data-tooltip="Vegan"
                                            >
                                                {@html icon("check", 18)}
                                            </div>
                                        {:else if dish.is_vegetarian}
                                            <div
                                                class="meal-card__diet-icon"
                                                data-tooltip="Vejetaryen"
                                            >
                                                {@html icon("check", 18)}
                                            </div>
                                        {/if}
                                        {#if dish.price}
                                            <span
                                                class="c-badge-pill c-badge-pill--price"
                                                data-tooltip="Ekstra alındığında veya ücretli durumda geçerli fiyattır. Fiyatlar şehre ve döneme göre değişiklik gösterebilir."
                                                >{sanitizeText(
                                                    dish.price,
                                                )}</span
                                            >
                                        {/if}
                                        {#if typeof dish.id === "number"}
                                            {@const activeFav = isFav(dish)}
                                            <button
                                                class="meal-card__star-btn {activeFav
                                                    ? 'active'
                                                    : ''}"
                                                onclick={() =>
                                                    handleFavorite(dish)}
                                                aria-label={activeFav
                                                    ? "Favorilerden çıkar"
                                                    : "Favorilere ekle"}
                                                title={activeFav
                                                    ? "Favorilerden çıkar"
                                                    : "Favorilere ekle"}
                                            >
                                                {@html icon(
                                                    activeFav
                                                        ? "starFilled"
                                                        : "star",
                                                    18,
                                                )}
                                            </button>
                                        {/if}
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            {/each}
        {/if}

        {#if calorieText}
            <!-- "Kalori:" öneki yok; kcal birimi anlamı tek başına taşır. -->
            <div class="text-sm color-muted u-flex u-flex-align-center calorie-info">
                <span>{calorieText}</span>
            </div>
        {/if}

        {#if currentTakeaways.length > 0}
            <div class="meal-card__item-row meal-card__takeaways-row">
                <div class="meal-card__takeaway-list">
                    {#each currentTakeaways as t, idx}
                        {@const labelText = t.name
                            ? sanitizeText(t.name)
                            : `Al Götür ${idx + 1}`}
                        <button
                            class="meal-card__dish-part meal-card__takeaway-btn"
                            data-takeaway-id={t.name || idx}
                            data-takeaway-label={labelText}
                            onclick={() => handleTakeawayClick(t, labelText)}
                        >
                            <span class="meal-card__dish-name">{labelText}</span
                            >
                            <div class="meal-card__dish-actions">
                                {@html icon("chevronRight", 18)}
                            </div>
                        </button>
                    {/each}
                </div>
            </div>
        {/if}
    </div>

    <div class="meal-card__footer">
        <div class="meal-card__votes">
            <button
                class="meal-card__vote-btn {myVote === 'positive'
                    ? 'is-active'
                    : ''}"
                data-vote="up"
                aria-label="Menüyü Beğen"
                onclick={() => handleVote("positive")}
            >
                {@html icon(
                    myVote === "positive" ? "voteUpFilled" : "voteUp",
                    18,
                )}
            </button>
            <span class="meal-card__vote-count {scoreClass}">{ratingSum}</span>
            <button
                class="meal-card__vote-btn {myVote === 'negative'
                    ? 'is-active'
                    : ''}"
                data-vote="down"
                aria-label="Menüyü Beğenme"
                onclick={() => handleVote("negative")}
            >
                {@html icon(
                    myVote === "negative" ? "voteDownFilled" : "voteDown",
                    18,
                )}
            </button>
        </div>

        <div class="meal-card__actions">
            {#if !hideComment && commentUrl}
                <a
                    href={commentUrl}
                    class="meal-card__action-btn"
                    data-link
                    data-tooltip="Yorumlar"
                    aria-label="Yorumlar"
                >
                    {@html icon("chat", 18)}
                    {#if menu.comment_count > 0}
                        <span class="meal-card__action-badge"
                            >{menu.comment_count}</span
                        >
                    {/if}
                </a>
            {/if}
            <button
                class="meal-card__action-btn"
                onclick={(e) => {
                    if (!globalState?.user) {
                        authActions.triggerLogin();
                        return;
                    }
                    openMenuReportModal(menu, e.currentTarget);
                }}
                data-tooltip="Hata bildir"
                aria-label="Hata bildir"
            >
                {@html icon("warning", 18)}
            </button>
        </div>
    </div>
</div>
